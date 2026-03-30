use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, StreamConfig,
};
use ringbuf::{traits::{Consumer, Producer, Split, Observer}, HeapRb};
use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::{Arc, Mutex};

// ── Public shared state ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct BandEnergy {
    /// Normalised 0.0–1.0 energy per band (smoothed)
    pub bass: f32,  // 20–250 Hz
    pub mid: f32,   // 250–4000 Hz
    pub high: f32,  // 4000–20000 Hz
}

pub type SharedAudio = Arc<Mutex<BandEnergy>>;

// ── Device listing ────────────────────────────────────────────────────────────

pub struct AudioDevice {
    pub name: String,
    pub is_loopback: bool, // PipeWire monitor sources end with ".monitor"
    device: Device,
}

pub fn list_devices() -> Vec<AudioDevice> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devs| {
            devs.filter_map(|d| {
                let name = d.name().unwrap_or_else(|_| "unknown".into());
                let is_loopback = name.to_lowercase().contains("monitor");
                Some(AudioDevice { name, is_loopback, device: d })
            })
            .collect()
        })
        .unwrap_or_default()
}

pub fn default_device() -> Option<AudioDevice> {
    let host = cpal::default_host();
    let d = host.default_input_device()?;
    let name = d.name().unwrap_or_else(|_| "default".into());
    let is_loopback = name.to_lowercase().contains("monitor");
    Some(AudioDevice { name, is_loopback, device: d })
}

// ── Stream builder ────────────────────────────────────────────────────────────

const FFT_SIZE: usize = 2048;
const SMOOTH: f32 = 0.15; // lower = smoother / slower response

pub fn start_capture(dev: AudioDevice, shared: SharedAudio) -> anyhow::Result<Stream> {
    let config = dev.device.default_input_config()?;
    let sample_rate = config.sample_rate().0 as f32;

    // Ring buffer: audio thread writes, FFT thread reads
    let rb = HeapRb::<f32>::new(FFT_SIZE * 4);
    let (mut prod, mut cons) = rb.split();

    // Spawn FFT analysis thread
    let shared_clone = shared.clone();
    std::thread::spawn(move || {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut scratch = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
        let mut buf = vec![0.0f32; FFT_SIZE];
        let mut smooth = BandEnergy::default();

        loop {
            // Wait until we have a full window
            let available = cons.occupied_len();
            if available < FFT_SIZE {
                std::thread::sleep(std::time::Duration::from_millis(5));
                continue;
            }
            // Drain the oldest samples (skip if we lag behind)
            let skip = available.saturating_sub(FFT_SIZE);
            for _ in 0..skip { cons.try_pop(); }

            // Read FFT_SIZE samples
            for s in buf.iter_mut() {
                *s = cons.try_pop().unwrap_or(0.0);
            }

            // Apply Hann window
            let mut spectrum: Vec<Complex<f32>> = buf
                .iter()
                .enumerate()
                .map(|(i, &s)| {
                    let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32
                        / (FFT_SIZE - 1) as f32).cos());
                    Complex::new(s * w, 0.0)
                })
                .collect();

            fft.process_with_scratch(&mut spectrum, &mut scratch);

            // Convert to magnitude (only first half = positive freqs)
            let mags: Vec<f32> = spectrum[..FFT_SIZE / 2]
                .iter()
                .map(|c| c.norm())
                .collect();

            // Bin → Hz: freq = bin * sample_rate / FFT_SIZE
            let bin_hz = sample_rate / FFT_SIZE as f32;

            let bass  = band_energy(&mags, bin_hz, 20.0,   250.0);
            let mid   = band_energy(&mags, bin_hz, 250.0,  4000.0);
            let high  = band_energy(&mags, bin_hz, 4000.0, 20000.0);

            // Smooth with exponential moving average
            smooth.bass  = smooth.bass  * (1.0 - SMOOTH) + bass  * SMOOTH;
            smooth.mid   = smooth.mid   * (1.0 - SMOOTH) + mid   * SMOOTH;
            smooth.high  = smooth.high  * (1.0 - SMOOTH) + high  * SMOOTH;

            if let Ok(mut state) = shared_clone.lock() {
                *state = smooth.clone();
            }
        }
    });

    // Build CPAL stream (f32 samples only; convert i16/u16 if needed)
    let stream = match config.sample_format() {
        SampleFormat::F32 => build_stream_f32(&dev.device, &config.into(), prod)?,
        SampleFormat::I16 => build_stream_i16(&dev.device, &config.into(), prod)?,
        _ => anyhow::bail!("Unsupported sample format"),
    };
    stream.play()?;
    Ok(stream)
}

fn band_energy(mags: &[f32], bin_hz: f32, lo: f32, hi: f32) -> f32 {
    let lo_bin = (lo / bin_hz) as usize;
    let hi_bin = ((hi / bin_hz) as usize).min(mags.len() - 1);
    if lo_bin >= hi_bin { return 0.0; }
    let sum: f32 = mags[lo_bin..=hi_bin].iter().sum();
    let avg = sum / (hi_bin - lo_bin + 1) as f32;
    // Soft normalisation — tuned for typical mic levels
    (avg / 500.0).clamp(0.0, 1.0)
}

type RbProd = ringbuf::wrap::caching::Caching<Arc<ringbuf::storage::Heap<f32>>, true, false>;
type RbCons = ringbuf::wrap::caching::Caching<Arc<ringbuf::storage::Heap<f32>>, false, true>;

fn build_stream_f32(
    device: &Device,
    config: &StreamConfig,
    mut prod: RbProd,
) -> anyhow::Result<Stream> {
    let channels = config.channels as usize;
    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _| {
            // Mix to mono
            for frame in data.chunks(channels) {
                let mono = frame.iter().sum::<f32>() / channels as f32;
                let _ = prod.try_push(mono);
            }
        },
        |e| eprintln!("Audio stream error: {e}"),
        None,
    )?;
    Ok(stream)
}

fn build_stream_i16(
    device: &Device,
    config: &StreamConfig,
    mut prod: RbProd,
) -> anyhow::Result<Stream> {
    let channels = config.channels as usize;
    let stream = device.build_input_stream(
        config,
        move |data: &[i16], _| {
            for frame in data.chunks(channels) {
                let mono = frame.iter().map(|&s| s as f32 / i16::MAX as f32).sum::<f32>()
                    / channels as f32;
                let _ = prod.try_push(mono);
            }
        },
        |e| eprintln!("Audio stream error: {e}"),
        None,
    )?;
    Ok(stream)
}
