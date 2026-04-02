use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use ringbuf::{
    traits::{Consumer, Observer, Producer, Split},
    HeapRb,
};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{Arc, Mutex};

// ── Shared State ──────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct BandEnergy {
    pub bass: f32,
    pub mid: f32,
    pub high: f32,
    pub bpm: f32,
    pub beat: bool,
    pub confidence: f32,
    pub ir_intensity: f32,
    pub ir_depth: Option<f32>,
}

pub type SharedAudio = Arc<Mutex<BandEnergy>>;

// ── Device API ────────────────────────────────────────────────────────────────

pub struct AudioDevice {
    pub name: String,
    pub is_loopback: bool,
    pub device: Device,
}

fn is_loopback_device(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("monitor")
        || lower.contains("stereo mix")
        || lower.contains("what u hear")
        || lower.contains("wave out")
        || lower.contains("blackhole")
        || lower.contains("soundflower")
}

/// Liste aller Input-Devices: (name, is_loopback)
pub fn list_devices() -> Vec<(String, bool)> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devs| {
            devs.filter_map(|d| {
                let name = d.name().ok()?;
                let loopback = is_loopback_device(&name);
                Some((name, loopback))
            })
            .collect()
        })
        .unwrap_or_default()
}

pub fn open_device_by_name(name: &str) -> Option<AudioDevice> {
    let host = cpal::default_host();
    let device = host
        .input_devices()
        .ok()?
        .find(|d| d.name().ok().as_deref() == Some(name))?;
    let is_loopback = is_loopback_device(name);
    Some(AudioDevice {
        name: name.to_owned(),
        is_loopback,
        device,
    })
}

/// Monitor des Default-Outputs → beliebiges Monitor → Default-Input
pub fn default_device() -> Option<AudioDevice> {
    let host = cpal::default_host();

    let output_name = host
        .default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let monitor_name = format!("{}.monitor", output_name);
    if let Some(dev) = open_device_by_name(&monitor_name) {
        return Some(dev);
    }

    let all = list_devices();
    if let Some((name, _)) = all.iter().find(|(_, lb)| *lb) {
        if let Some(dev) = open_device_by_name(name) {
            return Some(dev);
        }
    }

    let d = host.default_input_device()?;
    let name = d.name().unwrap_or_else(|_| "default".into());
    let is_loopback = name.to_lowercase().contains("monitor");
    Some(AudioDevice {
        name,
        is_loopback,
        device: d,
    })
}

// ── BPM Detector ─────────────────────────────────────────────────────────────

struct BpmDetector {
    energy_history: Vec<f32>,
    beat_intervals: Vec<f32>,
    last_beat_frame: usize,
    frame_count: usize,
    frame_rate: f32,
    pub bpm: f32,
    pub confidence: f32,
    energy_mean: f32,
    energy_variance: f32,
}

impl BpmDetector {
    fn new(frame_rate: f32) -> Self {
        BpmDetector {
            energy_history: vec![0.0; 43],
            beat_intervals: Vec::with_capacity(16),
            last_beat_frame: 0,
            frame_count: 0,
            frame_rate,
            bpm: 0.0,
            confidence: 0.0,
            energy_mean: 0.0,
            energy_variance: 0.0,
        }
    }

    fn detect(&mut self, energy: f32) -> (f32, bool, f32) {
        self.energy_history.rotate_left(1);
        *self.energy_history.last_mut().unwrap() = energy;
        self.frame_count += 1;

        let n = self.energy_history.len() as f32;
        self.energy_mean = self.energy_history.iter().sum::<f32>() / n;
        self.energy_variance = self
            .energy_history
            .iter()
            .map(|&x| (x - self.energy_mean).powi(2))
            .sum::<f32>()
            / n;

        let threshold = self.energy_mean + 1.5 * self.energy_variance.sqrt().max(0.01);
        let min_gap = (self.frame_rate * 60.0 / 220.0) as usize;
        let beat = energy > threshold
            && energy > 0.1
            && self.frame_count.saturating_sub(self.last_beat_frame) > min_gap;

        if beat {
            let gap = self.frame_count - self.last_beat_frame;
            let instant = 60.0 * self.frame_rate / gap as f32;
            if (30.0..=220.0).contains(&instant) {
                self.beat_intervals.push(instant);
                if self.beat_intervals.len() > 16 {
                    self.beat_intervals.remove(0);
                }

                let (weighted_bpm, confidence) = self.calculate_weighted_bpm();
                self.bpm = weighted_bpm;
                self.confidence = confidence;
            }
            self.last_beat_frame = self.frame_count;
        }

        let silence_frames = self.frame_count.saturating_sub(self.last_beat_frame);
        if silence_frames > (self.frame_rate * 2.0) as usize {
            self.confidence *= 0.98;
            if silence_frames > (self.frame_rate * 5.0) as usize {
                self.bpm *= 0.95;
                self.confidence *= 0.9;
                if self.bpm < 30.0 {
                    self.bpm = 0.0;
                    self.confidence = 0.0;
                }
            }
        }

        (self.bpm, beat, self.confidence)
    }

    fn calculate_weighted_bpm(&self) -> (f32, f32) {
        if self.beat_intervals.len() < 2 {
            let bpm = self.beat_intervals.first().copied().unwrap_or(0.0);
            return (bpm, 0.5);
        }

        let recent_count = self.beat_intervals.len().min(8);
        let recent: Vec<f32> = self.beat_intervals[self.beat_intervals.len() - recent_count..]
            .iter()
            .copied()
            .collect();

        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance: f32 =
            recent.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / recent.len() as f32;
        let std_dev = variance.sqrt().max(0.1);

        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        for (i, &interval) in recent.iter().enumerate() {
            let distance_from_mean = ((interval - mean) / std_dev).abs();
            let weight = (-distance_from_mean * 0.5).exp().max(0.3);
            let recency_weight = 1.0 + (i as f32 * 0.1);
            let final_weight = weight * recency_weight;
            weighted_sum += interval * final_weight;
            weight_total += final_weight;
        }

        let weighted_bpm = weighted_sum / weight_total;
        let consistency = (1.0 - (std_dev / mean).min(0.5)).max(0.0);
        let confidence = consistency * (recent.len() as f32 / 8.0).min(1.0);

        (weighted_bpm, confidence)
    }
}

// ── Capture ───────────────────────────────────────────────────────────────────

const FFT_SIZE: usize = 1024;
const SMOOTH: f32 = 0.3;

pub fn start_capture(dev: AudioDevice, shared: SharedAudio) -> anyhow::Result<Stream> {
    let config = dev.device.default_input_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    let rb = HeapRb::<f32>::new(FFT_SIZE * 4);
    let (prod, mut cons) = rb.split();

    let shared_clone = shared.clone();
    std::thread::spawn(move || {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut scratch = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
        let mut buf = vec![0.0f32; FFT_SIZE];
        let mut smooth = BandEnergy::default();
        let fft_fps = sample_rate / FFT_SIZE as f32;
        let mut bpm = BpmDetector::new(fft_fps);

        // Gleitende Maxima für per-Band Normalisierung
        let mut peak_bass: f32 = 1e-4;
        let mut peak_mid: f32 = 1e-4;
        let mut peak_high: f32 = 1e-4;

        loop {
            let available = cons.occupied_len();
            if available < FFT_SIZE {
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            }
            let skip = available.saturating_sub(FFT_SIZE);
            for _ in 0..skip {
                cons.try_pop();
            }

            for s in buf.iter_mut() {
                *s = cons.try_pop().unwrap_or(0.0);
            }

            let mut spectrum: Vec<Complex<f32>> = buf
                .iter()
                .enumerate()
                .map(|(i, &s)| {
                    let w = 0.5
                        * (1.0
                            - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32)
                                .cos());
                    Complex::new(s * w, 0.0)
                })
                .collect();

            fft.process_with_scratch(&mut spectrum, &mut scratch);

            let mags: Vec<f32> = spectrum[..FFT_SIZE / 2].iter().map(|c| c.norm()).collect();
            let bin_hz = sample_rate / FFT_SIZE as f32;

            // Per-Band-Normalisierung: jedes Band pflegt sein eigenes gleitendes Maximum.
            // Dadurch reagiert High auch wenn seine absolute Magnitude viel kleiner als
            // Bass ist — es wird relativ zu sich selbst normiert.
            let bass_raw = band_rms(&mags, bin_hz, 20.0, 300.0);
            let mid_raw = band_rms(&mags, bin_hz, 300.0, 4000.0);
            let high_raw = band_rms(&mags, bin_hz, 4000.0, 20000.0);

            // Gleitende Maxima pro Band (decay ~3s bei ~43fps)
            peak_bass = (peak_bass * 0.992).max(bass_raw).max(1e-4);
            peak_mid = (peak_mid * 0.992).max(mid_raw).max(1e-4);
            peak_high = (peak_high * 0.992).max(high_raw).max(1e-4);

            let bass_n = (bass_raw / peak_bass).clamp(0.0, 1.0);
            let mid_n = (mid_raw / peak_mid).clamp(0.0, 1.0);
            let high_n = (high_raw / peak_high).clamp(0.0, 1.0);

            smooth.bass = smooth.bass * (1.0 - SMOOTH) + bass_n * SMOOTH;
            smooth.mid = smooth.mid * (1.0 - SMOOTH) + mid_n * SMOOTH;
            smooth.high = smooth.high * (1.0 - SMOOTH) + high_n * SMOOTH;

            let (detected_bpm, beat, confidence) = bpm.detect(smooth.bass);
            smooth.bpm = detected_bpm;
            smooth.beat = beat;
            smooth.confidence = confidence;

            if let Ok(mut state) = shared_clone.lock() {
                *state = smooth.clone();
            }
        }
    });

    let stream = match config.sample_format() {
        SampleFormat::F32 => build_f32(&dev.device, &config.into(), prod)?,
        SampleFormat::I16 => build_i16(&dev.device, &config.into(), prod)?,
        _ => anyhow::bail!("Unsupported sample format"),
    };
    stream.play()?;
    Ok(stream)
}

/// RMS-Energie eines Frequenzbands — unnormiert, für externe Normalisierung.
fn band_rms(mags: &[f32], bin_hz: f32, lo: f32, hi: f32) -> f32 {
    let lo_bin = (lo / bin_hz) as usize;
    let hi_bin = ((hi / bin_hz) as usize).min(mags.len() - 1);
    if lo_bin >= hi_bin {
        return 0.0;
    }
    let slice = &mags[lo_bin..=hi_bin];
    let rms = (slice.iter().map(|&x| x * x).sum::<f32>() / slice.len() as f32).sqrt();
    rms
}

type RbProd = ringbuf::wrap::caching::Caching<
    Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>,
    true,
    false,
>;

fn build_f32(device: &Device, config: &StreamConfig, mut prod: RbProd) -> anyhow::Result<Stream> {
    let ch = config.channels as usize;
    Ok(device.build_input_stream(
        config,
        move |data: &[f32], _| {
            for frame in data.chunks(ch) {
                let _ = prod.try_push(frame.iter().sum::<f32>() / ch as f32);
            }
        },
        |e| eprintln!("Audio: {e}"),
        None,
    )?)
}

fn build_i16(device: &Device, config: &StreamConfig, mut prod: RbProd) -> anyhow::Result<Stream> {
    let ch = config.channels as usize;
    Ok(device.build_input_stream(
        config,
        move |data: &[i16], _| {
            for frame in data.chunks(ch) {
                let mono = frame
                    .iter()
                    .map(|&s| s as f32 / i16::MAX as f32)
                    .sum::<f32>()
                    / ch as f32;
                let _ = prod.try_push(mono);
            }
        },
        |e| eprintln!("Audio: {e}"),
        None,
    )?)
}
