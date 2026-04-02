mod audio;
mod charset;
#[path = "ir.rs"]
mod ir;
mod layers;
mod midi;
mod osc;

use audio::{BandEnergy, SharedAudio};
use charset::{Charset, DetailLevel};
use ir::SharedIr;
use layers::{detect_layers, get_detail_scale, PixelLayer};
use midi::{MidiHandler, MidiState, SharedMidi};
use osc::{OscState, SharedOsc};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};
use image::{imageops::FilterType, GrayImage};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

fn apply_midi_mappings(state: &mut AppState, midi: &MidiState) {
    if !state.midi_config.enabled {
        return;
    }
    for mapping in &state.midi_config.mappings {
        let cc_val = midi.cc[mapping.cc as usize];
        let normalized = mapping.min_val + cc_val * (mapping.max_val - mapping.min_val);

        match mapping.param {
            MidiParam::None => {}
            MidiParam::BassScale => {
                if let Some(band) = state.bands.get_mut(0) {
                    band.energy_scale = normalized.clamp(0.0, 3.0);
                }
            }
            MidiParam::MidScale => {
                if let Some(band) = state.bands.get_mut(1) {
                    band.energy_scale = normalized.clamp(0.0, 3.0);
                }
            }
            MidiParam::HighScale => {
                if let Some(band) = state.bands.get_mut(2) {
                    band.energy_scale = normalized.clamp(0.0, 3.0);
                }
            }
            MidiParam::GlobalContrast => {
                state.global_contrast = normalized.clamp(0.2, 4.0);
            }
            MidiParam::EnergyReact => {
                state.energy_responsiveness = normalized.clamp(0.0, 3.0);
            }
            MidiParam::BpmScale => {}
            MidiParam::EdgeThreshold => {
                state.edge_threshold = normalized.clamp(0.05, 0.5);
            }
            MidiParam::BgThreshold => {
                state.bg_threshold = normalized.clamp(0.1, 0.6);
            }
        }
    }
    state.midi_config.last_note = midi.note;
    state.midi_config.last_velocity = midi.velocity;
}

fn apply_osc_mappings(state: &mut AppState, osc: &OscState) {
    if !state.osc_config.enabled {
        return;
    }
    if let Some(band) = state.bands.get_mut(0) {
        band.energy_scale = (osc.bass * 3.0).clamp(0.0, 3.0);
    }
    if let Some(band) = state.bands.get_mut(1) {
        band.energy_scale = (osc.mid * 3.0).clamp(0.0, 3.0);
    }
    if let Some(band) = state.bands.get_mut(2) {
        band.energy_scale = (osc.high * 3.0).clamp(0.0, 3.0);
    }
    if osc.trigger {
        state.last_beat = true;
    }
}

fn list_cameras() -> Vec<String> {
    (0..4)
        .filter_map(|i| {
            let format = nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
                nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
            );
            nokhwa::Camera::new(nokhwa::utils::CameraIndex::Index(i), format)
                .ok()
                .map(|_| format!("Camera {}", i))
        })
        .collect()
}

// ── Farb-System ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub struct LayerColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl LayerColor {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        LayerColor { r, g, b }
    }

    /// Moduliert Helligkeit mit t (0..1). Minimum-Floor verhindert komplettes Schwarz.
    fn modulate(self, t: f32) -> Color {
        let t = t.clamp(0.04, 1.0);
        Color::Rgb {
            r: (self.r as f32 * t) as u8,
            g: (self.g as f32 * t) as u8,
            b: (self.b as f32 * t) as u8,
        }
    }
}

/// Eine Farbpalette: 4 Farben für Base / BG / FG / Edge.
#[derive(Clone, Copy)]
pub struct Palette {
    pub name: &'static str,
    pub base: LayerColor,
    pub black: LayerColor,
    pub bg: LayerColor,
    pub fg: LayerColor,
    pub edge: LayerColor,
    pub white: LayerColor,
}

impl Palette {
    const fn new(
        name: &'static str,
        base: LayerColor,
        black: LayerColor,
        bg: LayerColor,
        fg: LayerColor,
        edge: LayerColor,
        white: LayerColor,
    ) -> Self {
        Palette {
            name,
            base,
            black,
            bg,
            fg,
            edge,
            white,
        }
    }
}

/// Alle verfügbaren Paletten — mit `c` durchschalten.
/// Jede Palette ist um komplementäre Farbpaare aufgebaut:
/// Black/White = Extreme, BG/FG = Komplementärpaar, Edge = Akzent.
pub const PALETTES: &[Palette] = &[
    // 0 · Neon Noir — Cyan ↔ Magenta (komplementär), Indigo-BG
    Palette::new(
        "Neon Noir",
        LayerColor::new(20, 20, 30),    // base
        LayerColor::new(5, 0, 15),      // black: ultra-dunkel violett
        LayerColor::new(20, 10, 70),    // bg:    tief-indigo
        LayerColor::new(0, 230, 210),   // fg:    cyan  ←→ komplementär zu
        LayerColor::new(220, 0, 180),   // edge:  magenta
        LayerColor::new(200, 255, 250), // white: eis-weiß
    ),
    // 1 · Infrarot — Orange ↔ Blau (komplementär), Kohle-BG
    Palette::new(
        "Infrarot",
        LayerColor::new(15, 5, 0),      // base
        LayerColor::new(5, 0, 0),       // black: fast schwarz
        LayerColor::new(50, 10, 0),     // bg:    sehr dunkles rot-braun
        LayerColor::new(255, 90, 0),    // fg:    orange  ←→ komplementär zu
        LayerColor::new(0, 120, 255),   // edge:  royal-blau
        LayerColor::new(255, 230, 150), // white: warmes gelb-weiß
    ),
    // 2 · Phosphor — Grün ↔ Magenta (komplementär), CRT-Ästhetik
    Palette::new(
        "Phosphor",
        LayerColor::new(2, 10, 2),      // base
        LayerColor::new(0, 5, 0),       // black: fast schwarz
        LayerColor::new(0, 20, 0),      // bg:    sehr dunkles grün
        LayerColor::new(50, 205, 50),   // fg:    phosphor-grün  ←→
        LayerColor::new(200, 0, 180),   // edge:  magenta
        LayerColor::new(180, 255, 180), // white: helles grün
    ),
    // 3 · Dusk — Gold ↔ Violett (komplementär), Abendhimmel
    Palette::new(
        "Dusk",
        LayerColor::new(20, 10, 20),    // base
        LayerColor::new(8, 0, 12),      // black: nacht
        LayerColor::new(50, 15, 60),    // bg:    tief-violett
        LayerColor::new(240, 180, 30),  // fg:    gold  ←→ komplementär zu
        LayerColor::new(80, 20, 180),   // edge:  violett
        LayerColor::new(255, 230, 170), // white: warm-weiß
    ),
    // 4 · Arctic — Orange ↔ Blau-Cyan (komplementär), Eis-Ästhetik
    Palette::new(
        "Arctic",
        LayerColor::new(8, 15, 25),     // base
        LayerColor::new(0, 5, 15),      // black: nacht-schwarz
        LayerColor::new(10, 30, 70),    // bg:    tief-ocean
        LayerColor::new(140, 210, 255), // fg:    eis-blau  ←→
        LayerColor::new(255, 120, 20),  // edge:  orange (komplementär zu blau)
        LayerColor::new(225, 245, 255), // white: polar-weiß
    ),
    // 5 · Acid — Lime ↔ Hot-Pink (komplementär), maximaler Kontrast
    Palette::new(
        "Acid",
        LayerColor::new(5, 5, 5),       // base
        LayerColor::new(0, 0, 0),       // black: schwarz
        LayerColor::new(15, 0, 30),     // bg:    ultra-violett
        LayerColor::new(190, 255, 0),   // fg:    acid-lime  ←→
        LayerColor::new(255, 0, 140),   // edge:  hot-pink
        LayerColor::new(230, 255, 100), // white: hell-lime
    ),
    // 6 · Ember — Gold ↔ Teal (komplementär), Feuer-Ästhetik
    Palette::new(
        "Ember",
        LayerColor::new(12, 3, 0),     // base
        LayerColor::new(5, 0, 0),      // black: kohle
        LayerColor::new(35, 5, 0),     // bg:    dunkel-rot
        LayerColor::new(220, 80, 0),   // fg:    glut  ←→
        LayerColor::new(0, 180, 140),  // edge:  teal (komplementär zu orange)
        LayerColor::new(255, 210, 80), // white: funken-gold
    ),
    // 7 · Mono — Graustufen, klassisch
    Palette::new(
        "Mono",
        LayerColor::new(20, 20, 20),    // base
        LayerColor::new(0, 0, 0),       // black
        LayerColor::new(40, 40, 40),    // bg
        LayerColor::new(180, 180, 180), // fg
        LayerColor::new(255, 255, 255), // edge
        LayerColor::new(240, 240, 240), // white
    ),
    // 8 · Infrared — Nachtsicht-Ästhetik
    Palette::new(
        "Infrared",
        LayerColor::new(0, 10, 0),      // base
        LayerColor::new(0, 0, 0),       // black
        LayerColor::new(0, 20, 10),     // bg
        LayerColor::new(0, 180, 80),    // fg: IR-grün
        LayerColor::new(0, 255, 120),   // edge: hell-IR
        LayerColor::new(150, 255, 200), // white: fast weiß-grün
    ),
];
// ── Pixel-Output: Char + Farbe ────────────────────────────────────────────────

#[derive(Clone)]
struct PixelOut {
    ch: char,
    color: Color,
}

// ── Visual Layer ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VisualLayer {
    Black,
    Background,
    Foreground,
    Edge,
    White,
    IrEnhanced,
}

impl VisualLayer {
    fn name(self) -> &'static str {
        match self {
            VisualLayer::Black => "Black",
            VisualLayer::Background => "BG",
            VisualLayer::Foreground => "FG",
            VisualLayer::Edge => "Edge",
            VisualLayer::White => "White",
            VisualLayer::IrEnhanced => "IR",
        }
    }
    fn cycle(self) -> VisualLayer {
        match self {
            VisualLayer::Black => VisualLayer::Background,
            VisualLayer::Background => VisualLayer::Foreground,
            VisualLayer::Foreground => VisualLayer::Edge,
            VisualLayer::Edge => VisualLayer::White,
            VisualLayer::White => VisualLayer::IrEnhanced,
            VisualLayer::IrEnhanced => VisualLayer::Black,
        }
    }
    fn palette_color(self, p: &Palette) -> LayerColor {
        match self {
            VisualLayer::Black => p.black,
            VisualLayer::Background => p.bg,
            VisualLayer::Foreground => p.fg,
            VisualLayer::Edge => p.edge,
            VisualLayer::White => p.white,
            VisualLayer::IrEnhanced => LayerColor::new(0, 100, 80),
        }
    }
}

// ── Band Modus ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BandMode {
    Additive,
    Subtractive,
    InvertOnBeat,
    Multiply,
    Divide,
    Xor,
    Xand,
}

impl BandMode {
    fn name(self) -> &'static str {
        match self {
            BandMode::Additive => "+add",
            BandMode::Subtractive => "-sub",
            BandMode::InvertOnBeat => "~inv",
            BandMode::Multiply => "*mul",
            BandMode::Divide => "/div",
            BandMode::Xor => "^xor",
            BandMode::Xand => "^xnd",
        }
    }
    fn cycle(self) -> BandMode {
        match self {
            BandMode::Additive => BandMode::Subtractive,
            BandMode::Subtractive => BandMode::InvertOnBeat,
            BandMode::InvertOnBeat => BandMode::Multiply,
            BandMode::Multiply => BandMode::Divide,
            BandMode::Divide => BandMode::Xor,
            BandMode::Xor => BandMode::Xand,
            BandMode::Xand => BandMode::Additive,
        }
    }
}

// ── Band Config ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BandConfig {
    pub charset: Charset,
    pub layer: VisualLayer,
    pub mode: BandMode,
    pub energy_scale: f32,
    pub contrast_lo: f32,
    pub contrast_hi: f32,
    pub muted: bool,
    /// None = Palette-Farbe der Layer verwenden.
    /// Some(c) = diese Farbe überschreibt die Palette für dieses Band.
    pub color_override: Option<LayerColor>,
}

impl Default for BandConfig {
    fn default() -> Self {
        BandConfig {
            charset: Charset::Latin,
            layer: VisualLayer::Foreground,
            mode: BandMode::Additive,
            energy_scale: 1.0,
            contrast_lo: 0.0,
            contrast_hi: 1.0,
            muted: false,
            color_override: None,
        }
    }
}

// ── Base Layer Config ─────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BaseLayerConfig {
    pub charset: Charset,
    pub contrast: f32, // fester Kontrast, kein Audio-Mapping
    pub enabled: bool,
}

impl Default for BaseLayerConfig {
    fn default() -> Self {
        BaseLayerConfig {
            charset: Charset::Latin,
            contrast: 0.8,
            enabled: true,
        }
    }
}

// ── UI Mode ───────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum UiMode {
    Global,
    Band(usize),
    BaseLayer,
    DeviceMenu,
    CameraMenu,
    CameraSettings,
    LayerMenu,
    MidiMenu,
    OscMenu,
}

// ── App State ─────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct MidiMapping {
    pub cc: u8,
    pub param: MidiParam,
    pub min_val: f32,
    pub max_val: f32,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum MidiParam {
    #[default]
    None,
    BassScale,
    MidScale,
    HighScale,
    GlobalContrast,
    EnergyReact,
    BpmScale,
    EdgeThreshold,
    BgThreshold,
}

#[derive(Clone, Default)]
pub struct MidiConfig {
    pub enabled: bool,
    pub mappings: Vec<MidiMapping>,
    pub last_note: u8,
    pub last_velocity: u8,
}

#[derive(Clone)]
pub struct OscConfig {
    pub enabled: bool,
    pub target_host: String,
    pub target_port: u16,
    pub listen_port: u16,
    pub channels: [f32; 8],
}

impl Default for OscConfig {
    fn default() -> Self {
        OscConfig {
            enabled: false,
            target_host: "localhost".to_string(),
            target_port: 8000,
            listen_port: 7000,
            channels: [0.0; 8],
        }
    }
}

pub struct AppState {
    pub bands: [BandConfig; 3],
    pub base: BaseLayerConfig,

    pub global_contrast: f32,
    pub energy_responsiveness: f32,
    pub char_size: u32,
    pub inverted: bool,
    pub debug_mode: bool,

    pub bg_threshold: f32,
    pub edge_threshold: f32,

    pub mode: UiMode,
    pub palette_idx: usize,
    pub bpm_sync: bool,
    pub last_beat: bool,

    pub device_list: Vec<(String, bool)>,
    pub device_cursor: usize,

    pub camera_list: Vec<String>,
    pub camera_cursor: usize,

    pub camera_settings: CameraSettings,
    pub layer_cursor: usize,

    pub midi_config: MidiConfig,
    pub osc_config: OscConfig,
    pub midi_osc_cursor: usize,
}

#[derive(Clone)]
pub struct CameraSettings {
    pub focus: i32,
    pub exposure: i32,
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub gain: i32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            focus: -1,
            exposure: -1,
            brightness: -1,
            contrast: -1,
            saturation: -1,
            gain: -1,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            bands: [
                BandConfig {
                    layer: VisualLayer::Background,
                    charset: Charset::Latin,
                    ..Default::default()
                },
                BandConfig {
                    layer: VisualLayer::Foreground,
                    charset: Charset::Latin,
                    ..Default::default()
                },
                BandConfig {
                    layer: VisualLayer::Edge,
                    charset: Charset::Punctuation,
                    ..Default::default()
                },
            ],
            base: BaseLayerConfig::default(),
            global_contrast: 1.0,
            energy_responsiveness: 1.0,
            char_size: 1,
            inverted: false,
            debug_mode: cfg!(debug_assertions),
            bg_threshold: 0.35,
            edge_threshold: 0.25,
            mode: UiMode::Global,
            palette_idx: 0,
            bpm_sync: false,
            last_beat: false,
            device_list: vec![],
            device_cursor: 0,
            camera_list: vec![],
            camera_cursor: 0,
            camera_settings: CameraSettings::default(),
            layer_cursor: 0,
            midi_config: MidiConfig::default(),
            osc_config: OscConfig::default(),
            midi_osc_cursor: 0,
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Gibt für jeden Pixel einen (char, Color) aus, als row-major Vec.
fn render_frame(
    gray: &GrayImage,
    ascii_cols: u32,
    ascii_rows: u32,
    state: &AppState,
    band_energies: [f32; 3],
    ir_intensity: f32,
    ir_depth: Option<f32>,
    palette: &Palette,
) -> Vec<Vec<PixelOut>> {
    let resized = image::imageops::resize(gray, ascii_cols, ascii_rows, FilterType::Nearest);
    let result = detect_layers(
        &resized,
        state.bg_threshold,
        1.0 - state.bg_threshold,
        state.edge_threshold,
    );
    let layer_map = result.layers;
    let detail_map = result.detail;
    let fine_detail = result.fine_detail;
    let coarse_detail = result.coarse_detail;

    let w = ascii_cols as usize;
    let h = ascii_rows as usize;

    let ir_threshold = 1.0 - (ir_depth.unwrap_or(ir_intensity) * 0.5);

    let band_lookup: [Option<usize>; 6] = [
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::Black),
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::Background),
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::Foreground),
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::Edge),
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::White),
        state
            .bands
            .iter()
            .position(|b| b.layer == VisualLayer::IrEnhanced),
    ];

    // Pre-compute effektive Kontraste pro Band anhand der Energie
    let band_contrast: [f32; 3] = std::array::from_fn(|i| {
        let b = &state.bands[i];
        let e = if b.muted {
            0.0
        } else {
            (band_energies[i] * state.energy_responsiveness * b.energy_scale).clamp(0.0, 1.0)
        };
        // Linearer Interpolation zwischen contrast_lo und contrast_hi basierend auf Energie
        b.contrast_lo + e * (b.contrast_hi - b.contrast_lo)
    });

    let base_all = state.base.charset.chars();
    let base_slice = Charset::slice_by_contrast(base_all, state.base.contrast);

    let _band_all_chars: [&[char]; 3] = std::array::from_fn(|i| state.bands[i].charset.chars());
    let _band_slices: [&[char]; 3] =
        std::array::from_fn(|i| Charset::slice_by_contrast(_band_all_chars[i], band_contrast[i]));

    let mut rows: Vec<Vec<PixelOut>> = Vec::with_capacity(h);

    for row in 0..h {
        let mut line: Vec<PixelOut> = Vec::with_capacity(w);
        for col in 0..w {
            let luma_raw = resized.get_pixel(col as u32, row as u32).0[0] as f32 / 255.0;
            let luma_contrasted = ((luma_raw - 0.5) * state.global_contrast + 0.5).clamp(0.0, 1.0);
            let luma = if state.inverted {
                1.0 - luma_contrasted
            } else {
                luma_contrasted
            };

            let pixel_layer = &layer_map[row * w + col];

            // ── Layer-Bänder + Base Layer ─────────────────────────────────
            let base_visual = match pixel_layer {
                PixelLayer::Black => VisualLayer::Black,
                PixelLayer::Background => VisualLayer::Background,
                PixelLayer::Foreground => VisualLayer::Foreground,
                PixelLayer::Edge => VisualLayer::Edge,
                PixelLayer::White => VisualLayer::White,
            };

            let target_visual = if ir_intensity > 0.3 {
                if matches!(pixel_layer, PixelLayer::Edge) {
                    VisualLayer::IrEnhanced
                } else if matches!(pixel_layer, PixelLayer::Foreground)
                    && ir_depth.map(|d| d > ir_threshold).unwrap_or(false)
                {
                    VisualLayer::IrEnhanced
                } else {
                    base_visual
                }
            } else {
                base_visual
            };

            // Zugehöriges Band für diese Ebene (O(1) lookup)
            let band_idx = match target_visual {
                VisualLayer::Black => band_lookup[0],
                VisualLayer::Background => band_lookup[1],
                VisualLayer::Foreground => band_lookup[2],
                VisualLayer::Edge => band_lookup[3],
                VisualLayer::White => band_lookup[4],
                VisualLayer::IrEnhanced => band_lookup[5],
            };

            let out = if let Some(idx) = band_idx {
                let band = &state.bands[idx];

                let e = if band.muted {
                    0.0
                } else {
                    (band_energies[idx] * state.energy_responsiveness * band.energy_scale)
                        .clamp(0.0, 1.0)
                };

                // Modus: Energie moduliert Luma
                let eff_luma = match band.mode {
                    BandMode::Additive => (luma + e * 0.5).clamp(0.0, 1.0),
                    BandMode::Subtractive => (luma - e * 0.5).clamp(0.0, 1.0),
                    BandMode::InvertOnBeat => {
                        if state.last_beat {
                            1.0 - luma
                        } else {
                            luma
                        }
                    }
                    BandMode::Multiply => (luma * (0.5 + e * 0.5)).clamp(0.0, 1.0),
                    BandMode::Divide => {
                        if luma > 0.01 {
                            (luma / (0.5 + e * 0.5)).clamp(0.0, 1.0)
                        } else {
                            luma
                        }
                    }
                    BandMode::Xor => {
                        let luma_int = (luma * 255.0) as u8;
                        let e_int = (e * 255.0) as u8;
                        ((luma_int ^ e_int) as f32 / 255.0).clamp(0.0, 1.0)
                    }
                    BandMode::Xand => {
                        let luma_int = (luma * 255.0) as u8;
                        let e_int = (e * 255.0) as u8;
                        let xor_result = luma_int ^ e_int;
                        let xand = !(xor_result) & 0xFF;
                        (xand as f32 / 255.0).clamp(0.0, 1.0)
                    }
                };

                let detail = detail_map[row * w + col];
                let fine = fine_detail[row * w + col];
                let coarse = coarse_detail[row * w + col];

                let detail_scale = get_detail_scale(detail);
                let (detail_adjust, charset_subset) = match detail_scale {
                    layers::DetailScale::Fine => {
                        let subset = band
                            .charset
                            .detail_chars(DetailLevel::Fine, band.contrast_lo);
                        (fine * 0.4, subset)
                    }
                    layers::DetailScale::Medium => {
                        let subset = band
                            .charset
                            .detail_chars(DetailLevel::Medium, band.contrast_lo);
                        (detail * 0.3, subset)
                    }
                    layers::DetailScale::Coarse => {
                        let subset = band
                            .charset
                            .detail_chars(DetailLevel::Coarse, band.contrast_lo);
                        (coarse * 0.2, subset)
                    }
                };

                let char_idx = if charset_subset.is_empty() {
                    0
                } else {
                    let adjusted_luma = (eff_luma + detail_adjust).min(1.0);
                    let local_len = charset_subset.len();
                    ((adjusted_luma * (local_len - 1) as f32).round() as usize).min(local_len - 1)
                };
                let ch = if charset_subset.is_empty() {
                    ' '
                } else {
                    charset_subset[char_idx]
                };

                let detail_color_boost = detail * ir_intensity.max(0.3);
                let color_modulation = (0.2 + e * 0.8 + detail_color_boost * 0.2).min(1.0);

                // Farbe: color_override hat Vorrang, sonst Palette-Farbe der Layer
                let base_col = band
                    .color_override
                    .unwrap_or_else(|| target_visual.palette_color(palette));
                let color = base_col.modulate(color_modulation);

                PixelOut { ch, color }
            } else if state.base.enabled {
                // Kein Band für diese Ebene → Base Layer
                let len = base_slice.len();
                let ch = if len == 0 {
                    ' '
                } else {
                    let idx = ((luma * (len - 1) as f32).round() as usize).min(len - 1);
                    base_slice[idx]
                };
                let color = palette.base.modulate(luma * 0.7 + 0.3);
                PixelOut { ch, color }
            } else {
                PixelOut {
                    ch: ' ',
                    color: Color::Reset,
                }
            };

            // Wide-char Korrektur
            let col_w = if let Some(idx) = band_idx {
                state.bands[idx].charset.col_width()
            } else {
                1
            };
            if col_w == 2 && col % 2 == 1 {
                continue;
            }

            line.push(out);
        }
        rows.push(line);
    }
    rows
}

fn vu_bar(energy: f32) -> String {
    let n = (energy * 8.0).round() as usize;
    format!("{}{}", "█".repeat(n), "░".repeat(8usize.saturating_sub(n)))
}

// ── Input ─────────────────────────────────────────────────────────────────────
//
// Neues Key-Layout (CLAUDE.md):
//   - Keine +/- mehr für Parameter — dedizierte Paare, nah beieinander
//   - g  → zurück zu Global aus jedem Modus
//   - 1/2/3 → Band direkt wechseln (auch aus Band-Modus heraus)
//   - Q (Shift+q) → Quit
//   - x als Band-Exit entfernt
//
// Dedizierte Paare im Global-Modus:
//   Kontrast:     r / t   (rauf / runter, neben einander)
//   Reaktion:     f / h   (links / rechts, homerow-nah)
//   Char-Size:    v / b   (nebeneinander, untere Reihe)
//   Palette:      c / z   (nah beieinander)
//
// Dedizierte Paare im Band-Modus:
//   Energy-Scale: r / t
//   Contrast-Lo:  f / h
//   Contrast-Hi:  v / b
//   Layer:        l       (cycle, bleibt)
//   Script:       s       (cycle, bleibt)
//   Mode:         m       (cycle, bleibt)
//   Mute:         u       (toggle, M wie Mute — aber 'm' ist mode → 'u')
//   Color:        o / p   (neben einander: o=prev, p=next in Palette)
//
// Base-Layer-Modus:
//   Script:       s
//   Toggle:       v
//   Kontrast:     r / t

/// Returns true wenn Device-Wechsel gewünscht
fn handle_key(code: KeyCode, state: &mut AppState) -> Option<u8> {
    // Globale Tasten die in allen Modi funktionieren
    match code {
        // Direkt zu Band wechseln — aus jedem Modus
        KeyCode::Char('1') => {
            state.mode = UiMode::Band(0);
            return None;
        }
        KeyCode::Char('2') => {
            state.mode = UiMode::Band(1);
            return None;
        }
        KeyCode::Char('3') => {
            state.mode = UiMode::Band(2);
            return None;
        }
        // g → Global (aus jedem Modus)
        KeyCode::Char('g') => {
            if state.mode != UiMode::Global
                && state.mode != UiMode::DeviceMenu
                && state.mode != UiMode::CameraMenu
                && state.mode != UiMode::CameraSettings
                && state.mode != UiMode::LayerMenu
                && state.mode != UiMode::MidiMenu
                && state.mode != UiMode::OscMenu
            {
                state.mode = UiMode::Global;
                return None;
            }
        }
        _ => {}
    }

    match state.mode.clone() {
        UiMode::DeviceMenu => match code {
            KeyCode::Up => {
                if state.device_cursor > 0 {
                    state.device_cursor -= 1;
                }
            }
            KeyCode::Down => {
                state.device_cursor =
                    (state.device_cursor + 1).min(state.device_list.len().saturating_sub(1));
            }
            KeyCode::Esc | KeyCode::Char(' ') => {
                state.mode = UiMode::Global;
            }
            KeyCode::Enter => return Some(0), // Audio device switch
            _ => {}
        },

        UiMode::CameraMenu => match code {
            KeyCode::Up => {
                if state.camera_cursor > 0 {
                    state.camera_cursor -= 1;
                }
            }
            KeyCode::Down => {
                state.camera_cursor =
                    (state.camera_cursor + 1).min(state.camera_list.len().saturating_sub(1));
            }
            KeyCode::Esc | KeyCode::Char('g') => {
                state.mode = UiMode::Global;
            }
            KeyCode::Enter => return Some(1), // Camera switch
            _ => {}
        },

        UiMode::BaseLayer => match code {
            // Kein 'x' mehr — g übernimmt zurück-Funktion (oben behandelt)
            KeyCode::Char('s') => {
                state.base.charset = state.base.charset.next();
            }
            KeyCode::Char('v') => {
                state.base.enabled = !state.base.enabled;
            }
            // r/t: Kontrast rauf/runter
            KeyCode::Char('r') => {
                state.base.contrast = (state.base.contrast + 0.05).min(1.0);
            }
            KeyCode::Char('t') => {
                state.base.contrast = (state.base.contrast - 0.05).max(0.05);
            }
            _ => {}
        },

        UiMode::CameraSettings => match code {
            KeyCode::Char('f') => {
                state.camera_settings.focus = (state.camera_settings.focus + 10).min(100);
            }
            KeyCode::Char('v') => {
                state.camera_settings.focus = (state.camera_settings.focus - 10).max(-1);
            }
            KeyCode::Char('e') => {
                state.camera_settings.exposure = (state.camera_settings.exposure + 10).min(100);
            }
            KeyCode::Char('d') => {
                state.camera_settings.exposure = (state.camera_settings.exposure - 10).max(-1);
            }
            KeyCode::Char('b') => {
                state.camera_settings.brightness = (state.camera_settings.brightness + 10).min(100);
            }
            KeyCode::Char('g') => {
                state.camera_settings.brightness = (state.camera_settings.brightness - 10).max(-1);
            }
            KeyCode::Char('o') => {
                state.camera_settings.contrast = (state.camera_settings.contrast + 10).min(100);
            }
            KeyCode::Char('i') => {
                state.camera_settings.contrast = (state.camera_settings.contrast - 10).max(-1);
            }
            KeyCode::Char('s') => {
                state.camera_settings.saturation = (state.camera_settings.saturation + 10).min(100);
            }
            KeyCode::Char('a') => {
                state.camera_settings.saturation = (state.camera_settings.saturation - 10).max(-1);
            }
            KeyCode::Char('l') => {
                state.camera_settings.gain = (state.camera_settings.gain + 10).min(100);
            }
            KeyCode::Char('k') => {
                state.camera_settings.gain = (state.camera_settings.gain - 10).max(-1);
            }
            _ => {}
        },

        UiMode::LayerMenu => match code {
            KeyCode::Up => {
                state.layer_cursor = state.layer_cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                state.layer_cursor = (state.layer_cursor + 1).min(2);
            }
            KeyCode::Left => {
                state.bands[state.layer_cursor].layer =
                    state.bands[state.layer_cursor].layer.cycle();
            }
            KeyCode::Right => {
                for _ in 0..5 {
                    state.bands[state.layer_cursor].layer =
                        state.bands[state.layer_cursor].layer.cycle();
                }
            }
            _ => {}
        },

        UiMode::MidiMenu => match code {
            KeyCode::Esc | KeyCode::Char('g') => {
                state.mode = UiMode::Global;
            }
            KeyCode::Up => {
                state.midi_osc_cursor = state.midi_osc_cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                state.midi_osc_cursor = (state.midi_osc_cursor + 1).min(7);
            }
            KeyCode::Left => {
                if state.midi_osc_cursor < state.midi_config.mappings.len() {
                    let mapping = &mut state.midi_config.mappings[state.midi_osc_cursor];
                    mapping.param = match mapping.param {
                        MidiParam::None => MidiParam::BassScale,
                        MidiParam::BassScale => MidiParam::MidScale,
                        MidiParam::MidScale => MidiParam::HighScale,
                        MidiParam::HighScale => MidiParam::GlobalContrast,
                        MidiParam::GlobalContrast => MidiParam::EnergyReact,
                        MidiParam::EnergyReact => MidiParam::BpmScale,
                        MidiParam::BpmScale => MidiParam::EdgeThreshold,
                        MidiParam::EdgeThreshold => MidiParam::BgThreshold,
                        MidiParam::BgThreshold => MidiParam::None,
                    };
                }
            }
            KeyCode::Right => {
                if state.midi_osc_cursor < state.midi_config.mappings.len() {
                    let mapping = &mut state.midi_config.mappings[state.midi_osc_cursor];
                    mapping.param = match mapping.param {
                        MidiParam::BgThreshold => MidiParam::EdgeThreshold,
                        MidiParam::EdgeThreshold => MidiParam::BpmScale,
                        MidiParam::BpmScale => MidiParam::EnergyReact,
                        MidiParam::EnergyReact => MidiParam::GlobalContrast,
                        MidiParam::GlobalContrast => MidiParam::HighScale,
                        MidiParam::HighScale => MidiParam::MidScale,
                        MidiParam::MidScale => MidiParam::BassScale,
                        MidiParam::BassScale => MidiParam::None,
                        MidiParam::None => MidiParam::None,
                    };
                }
            }
            KeyCode::Char('e') => {
                state.midi_config.enabled = !state.midi_config.enabled;
            }
            KeyCode::Char('a') => {
                state.midi_config.mappings.push(MidiMapping {
                    cc: 1,
                    param: MidiParam::None,
                    min_val: 0.0,
                    max_val: 1.0,
                });
            }
            KeyCode::Char('d') => {
                if state.midi_osc_cursor < state.midi_config.mappings.len() {
                    state.midi_config.mappings.remove(state.midi_osc_cursor);
                    state.midi_osc_cursor = state.midi_osc_cursor.saturating_sub(1);
                }
            }
            _ => {}
        },

        UiMode::OscMenu => match code {
            KeyCode::Esc | KeyCode::Char('g') => {
                state.mode = UiMode::Global;
            }
            KeyCode::Up => {
                state.midi_osc_cursor = state.midi_osc_cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                state.midi_osc_cursor = (state.midi_osc_cursor + 1).min(9);
            }
            KeyCode::Char('e') => {
                state.osc_config.enabled = !state.osc_config.enabled;
            }
            KeyCode::Char('h') => {
                state.osc_config.target_host = if state.osc_config.target_host == "localhost" {
                    "127.0.0.1".to_string()
                } else {
                    "localhost".to_string()
                };
            }
            KeyCode::Char('p') => {
                state.osc_config.target_port =
                    (state.osc_config.target_port as usize + 100).min(65535) as u16;
            }
            KeyCode::Char('o') => {
                let new_val = (state.osc_config.target_port as usize)
                    .saturating_sub(100)
                    .max(1024);
                state.osc_config.target_port = new_val as u16;
            }
            KeyCode::Char('l') => {
                state.osc_config.listen_port =
                    (state.osc_config.listen_port as usize + 100).min(65535) as u16;
            }
            KeyCode::Char('k') => {
                let new_val = (state.osc_config.listen_port as usize)
                    .saturating_sub(100)
                    .max(1024);
                state.osc_config.listen_port = new_val as u16;
            }
            _ => {}
        },

        UiMode::Band(i) => {
            let palette_idx = state.palette_idx;
            let band = &mut state.bands[i];
            match code {
                // Kein 'x' mehr — g (oben) übernimmt

                // Mute: u (M ist mode)
                KeyCode::Char('u') => {
                    band.muted = !band.muted;
                }

                // Mode cycle: m
                KeyCode::Char('m') => {
                    band.mode = band.mode.cycle();
                }

                // Layer cycle: l
                KeyCode::Char('l') => {
                    band.layer = band.layer.cycle();
                }

                // Script cycle: s
                KeyCode::Char('s') => {
                    band.charset = band.charset.next();
                }

                // Color override: o (prev/cycle back) / p (next/cycle forward)
                KeyCode::Char('p') => {
                    let pal = &PALETTES[palette_idx];
                    let cur = band.color_override;
                    band.color_override = if cur.is_none() {
                        Some(pal.black)
                    } else if cur == Some(pal.black) {
                        Some(pal.bg)
                    } else if cur == Some(pal.bg) {
                        Some(pal.fg)
                    } else if cur == Some(pal.fg) {
                        Some(pal.edge)
                    } else if cur == Some(pal.edge) {
                        Some(pal.white)
                    } else {
                        None
                    };
                }
                KeyCode::Char('o') => {
                    let pal = &PALETTES[palette_idx];
                    let cur = band.color_override;
                    band.color_override = if cur.is_none() {
                        Some(pal.white)
                    } else if cur == Some(pal.white) {
                        Some(pal.edge)
                    } else if cur == Some(pal.edge) {
                        Some(pal.fg)
                    } else if cur == Some(pal.fg) {
                        Some(pal.bg)
                    } else if cur == Some(pal.bg) {
                        Some(pal.black)
                    } else {
                        None
                    };
                }

                // r/t: Energy-Scale rauf/runter
                KeyCode::Char('r') => {
                    band.energy_scale = (band.energy_scale + 0.1).min(3.0);
                }
                KeyCode::Char('t') => {
                    band.energy_scale = (band.energy_scale - 0.1).max(0.0);
                }

                // f/h: Contrast-Lo links/rechts (f=hoch, h=runter — f vor h im Alphabet)
                KeyCode::Char('f') => {
                    band.contrast_lo = (band.contrast_lo + 0.05).min(band.contrast_hi - 0.05);
                }
                KeyCode::Char('h') => {
                    band.contrast_lo = (band.contrast_lo - 0.05).max(0.0);
                }

                // v/b: Contrast-Hi hoch/runter
                KeyCode::Char('b') => {
                    band.contrast_hi = (band.contrast_hi + 0.05).min(1.0);
                }
                KeyCode::Char('n') => {
                    band.contrast_hi = (band.contrast_hi - 0.05).max(band.contrast_lo + 0.05);
                }

                _ => {}
            }
        }

        UiMode::Global => match code {
            KeyCode::Char('Q') => {} // handled in main (Shift+q = Quit)
            // 0 → Base Layer
            KeyCode::Char('0') => {
                state.mode = UiMode::BaseLayer;
            }
            // Space → Device Menu
            KeyCode::Char(' ') => {
                state.device_list = audio::list_devices();
                state.device_cursor = 0;
                state.mode = UiMode::DeviceMenu;
            }
            // k → Camera Menu
            KeyCode::Char('k') => {
                state.camera_list = list_cameras();
                state.camera_cursor = 0;
                state.mode = UiMode::CameraMenu;
            }
            // n → Camera Settings
            KeyCode::Char('n') => {
                state.mode = UiMode::CameraSettings;
            }
            // w → Layer Menu
            KeyCode::Char('w') => {
                state.mode = UiMode::LayerMenu;
            }
            // m → MIDI Menu
            KeyCode::Char('m') => {
                state.midi_osc_cursor = 0;
                state.mode = UiMode::MidiMenu;
            }
            // d → OSC Menu
            KeyCode::Char('d') => {
                state.midi_osc_cursor = 0;
                state.mode = UiMode::OscMenu;
            }
            // D → Debug Mode Toggle
            KeyCode::Char('D') => {
                state.debug_mode = !state.debug_mode;
                eprintln!(
                    "Debug mode: {}",
                    if state.debug_mode { "ON" } else { "OFF" }
                );
            }
            // Toggles
            KeyCode::Char('i') => {
                state.inverted = !state.inverted;
            }
            KeyCode::Char('y') => {
                state.bpm_sync = !state.bpm_sync;
            } // y neben t — BPM-sync

            // Palette: c vorwärts / z rückwärts (neben einander, untere Reihe)
            KeyCode::Char('c') => {
                state.palette_idx = (state.palette_idx + 1) % PALETTES.len();
            }
            KeyCode::Char('z') => {
                state.palette_idx = (state.palette_idx + PALETTES.len() - 1) % PALETTES.len();
            }

            // r/t: Globaler Kontrast rauf/runter
            KeyCode::Char('r') => {
                state.global_contrast = (state.global_contrast + 0.1).min(4.0);
            }
            KeyCode::Char('t') => {
                state.global_contrast = (state.global_contrast - 0.1).max(0.2);
            }

            // f/h: Energie-Reaktion rauf/runter
            KeyCode::Char('f') => {
                state.energy_responsiveness = (state.energy_responsiveness + 0.1).min(3.0);
            }
            KeyCode::Char('h') => {
                state.energy_responsiveness = (state.energy_responsiveness - 0.1).max(0.0);
            }

            // v/b: Char-Size hoch/runter
            KeyCode::Char('b') => {
                state.char_size = (state.char_size + 1).min(4);
            }
            KeyCode::Char('v') => {
                state.char_size = state.char_size.saturating_sub(1).max(1);
            }

            _ => {}
        },
    }
    None
}

// ── Status ────────────────────────────────────────────────────────────────────

fn draw_status(
    stdout: &mut impl Write,
    state: &AppState,
    energies: [f32; 3],
    bpm: f32,
    confidence: f32,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    let band_ui_colors = [Color::Red, Color::Yellow, Color::Cyan];
    let band_labels = ["1:Bass", "2:Mid ", "3:High"];

    queue!(stdout, cursor::MoveTo(0, term_rows - 4))?;

    // Base Layer Indikator
    let base_sel = matches!(&state.mode, UiMode::BaseLayer);
    queue!(
        stdout,
        if base_sel {
            SetAttribute(Attribute::Bold)
        } else {
            SetAttribute(Attribute::Reset)
        },
        SetForegroundColor(Color::DarkGrey),
        Print(format!(
            "{}0:Base {}  ",
            if base_sel { '▶' } else { ' ' },
            if state.base.enabled { "[on]" } else { "[off]" }
        )),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )?;

    // Band VU + Config
    for i in 0..3 {
        let b = &state.bands[i];
        let sel = matches!(&state.mode, UiMode::Band(x) if *x == i);
        let e = energies[i];
        let col_indicator = if b.color_override.is_some() {
            "c:!"
        } else {
            "c:·"
        };
        queue!(
            stdout,
            SetForegroundColor(band_ui_colors[i]),
            if sel {
                SetAttribute(Attribute::Bold)
            } else {
                SetAttribute(Attribute::Reset)
            },
            Print(format!(
                "{}{}{}{}|{}|{} e:{:.1} lo:{:.2} hi:{:.2} {} {}",
                if sel { '\u{25B6}' } else { ' ' },
                band_labels[i],
                if b.muted { " M" } else { "  " },
                b.layer.name(),
                b.mode.name(),
                b.charset.name(),
                vu_bar(e),
                b.energy_scale,
                b.contrast_lo,
                b.contrast_hi,
                col_indicator,
            )),
            ResetColor,
            SetAttribute(Attribute::Reset),
        )?;
    }

    // BPM
    let bpm_str = if bpm > 0.0 {
        format!("{:.0}BPM", bpm)
    } else {
        "---BPM".into()
    };
    let conf_str = if confidence > 0.0 {
        format!(" {:.0}%", confidence * 100.0)
    } else {
        String::new()
    };
    let sync_str = if state.bpm_sync { "[sync]" } else { "" };
    queue!(
        stdout,
        SetForegroundColor(Color::Magenta),
        Print(format!(" {bpm_str}{conf_str}{sync_str}")),
        ResetColor
    )?;

    // MIDI indicator
    if state.midi_config.enabled {
        let mappings_count = state.midi_config.mappings.len();
        queue!(
            stdout,
            SetForegroundColor(Color::Green),
            Print(format!(
                " MIDI:{}{}",
                if mappings_count > 0 {
                    format!("({}m)", mappings_count)
                } else {
                    String::new()
                },
                if mappings_count == 0 {
                    " [no mappings]"
                } else {
                    ""
                }
            )),
            ResetColor
        )?;
    }

    if state.osc_config.enabled {
        queue!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print(format!(
                " OSC l:{}→{}:{}",
                state.osc_config.listen_port,
                state.osc_config.target_host,
                state.osc_config.target_port
            )),
            ResetColor
        )?;
    }

    // Help line
    let help = match &state.mode {
        UiMode::Global    => " GLOBAL | r/t:kontrast  f/h:reaktion  v/b:size  c/z:palette  i:inv  y:bpm  0:base  Space:audio  k:camera  n:settings  w:layers  1/2/3:band  D:debug  Q:quit".to_string(),
        UiMode::Band(i)   => format!(" BAND {} | r/t:energy  f/h:ctr-lo  v/b:ctr-hi  l:layer({})  s:script  m:mode({})  o/p:farbe  u:mute  g:global",
            i+1, state.bands[*i].layer.name(), state.bands[*i].mode.name()),
        UiMode::BaseLayer => " BASE | s:script  v:toggle  r/t:kontrast  g:global".to_string(),
        UiMode::DeviceMenu=> " ↑↓:navigate  Enter:select  Esc:cancel".to_string(),
        UiMode::CameraMenu=> " ↑↓:navigate  Enter:select  Esc/g:cancel".to_string(),
        UiMode::CameraSettings=> " CAMERA | f/v:focus  e/d:exposure  b/g:brightness  o/i:contrast  s/a:saturation  l/k:gain  g:global".to_string(),
        UiMode::LayerMenu=> " LAYERS | ↑↓:select  ←→:change layer  g:global".to_string(),
        UiMode::MidiMenu=> " MIDI | ↑↓:select  ←→:param  e:enable  a:add  d:delete  g:global".to_string(),
        UiMode::OscMenu=> " OSC | e:enable  h:host  p/o:target-port  l/k:listen-port  g:global".to_string(),
    };

    let pal_name = PALETTES[state.palette_idx].name;
    let info = format!(
        " ctr:{:.1} react:{:.1} sz:{} {}{}{}",
        state.global_contrast,
        state.energy_responsiveness,
        state.char_size,
        pal_name,
        if state.inverted { " [inv]" } else { "" },
        if state.debug_mode { " [DEBUG]" } else { "" }
    );

    let line = format!("{info} |{help}");
    let truncated: String = line.chars().take(term_cols as usize).collect();
    queue!(stdout, cursor::MoveTo(0, term_rows - 1), Print(truncated))?;

    Ok(())
}

fn draw_debug_panel(
    stdout: &mut impl Write,
    state: &AppState,
    energies: [f32; 3],
    bpm: f32,
    confidence: f32,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    if !state.debug_mode {
        return Ok(());
    }

    let panel_width = 30.min(term_cols / 3);
    let panel_x = term_cols.saturating_sub(panel_width);
    let panel_y = 2;

    queue!(stdout, SetForegroundColor(Color::Yellow))?;
    queue!(
        stdout,
        cursor::MoveTo(panel_x, panel_y),
        Print(format!("┌{:─<w$}┐", "", w = panel_width as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(panel_x, panel_y + 1),
        Print(format!("│{:>w$}│", " DEBUG", w = panel_width as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(panel_x, panel_y + 2),
        Print(format!("├{:─<w$}┤", "", w = panel_width as usize - 2))
    )?;

    let debug_lines = [
        format!("FPS: {:.0}", 1.0 / 0.033), // placeholder
        format!("Bass: {:.2}", energies[0]),
        format!("Mid: {:.2}", energies[1]),
        format!("High: {:.2}", energies[2]),
        format!("BPM: {:.0} ({:.0}%)", bpm, confidence * 100.0),
        format!("Beat: {}", if state.last_beat { "YES" } else { "no" }),
        format!(
            "MIDI: {}",
            if state.midi_config.enabled {
                "ON"
            } else {
                "OFF"
            }
        ),
        format!(
            "OSC: {}",
            if state.osc_config.enabled {
                "ON"
            } else {
                "OFF"
            }
        ),
        format!(
            "Camera: {}",
            if state.camera_list.is_empty() {
                "none"
            } else {
                "ok"
            }
        ),
    ];

    for (i, line) in debug_lines.iter().enumerate() {
        let row = panel_y + 3 + i as u16;
        if row >= term_rows.saturating_sub(5) {
            break;
        }
        queue!(
            stdout,
            cursor::MoveTo(panel_x, row),
            Print(format!("│ {:<w$} ", line, w = panel_width as usize - 4))
        )?;
    }

    queue!(
        stdout,
        cursor::MoveTo(panel_x, panel_y + 3 + debug_lines.len() as u16),
        Print(format!("└{:─<w$}┘", "", w = panel_width as usize - 2))
    )?;
    queue!(stdout, ResetColor)?;

    Ok(())
}

fn draw_device_menu(
    stdout: &mut impl Write,
    state: &AppState,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    let box_w: u16 = (term_cols / 2).max(40).min(70);
    let box_h: u16 = (state.device_list.len() as u16 + 4).min(term_rows - 4);
    let box_x = (term_cols - box_w) / 2;
    let box_y = (term_rows - box_h) / 2;

    queue!(stdout, SetForegroundColor(Color::White))?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y),
        Print(format!("┌{:─<w$}┐", "", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 1),
        Print(format!("│{:^w$}│", " Audio Device", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 2),
        Print(format!("├{:─<w$}┤", "", w = box_w as usize - 2))
    )?;

    for (i, (name, is_loopback)) in state.device_list.iter().enumerate() {
        let row = box_y + 3 + i as u16;
        if row >= box_y + box_h - 1 {
            break;
        }
        let icon = if *is_loopback { "⟳" } else { "♪" };
        let label: String = format!("{icon} {name}")
            .chars()
            .take(box_w as usize - 4)
            .collect();
        if i == state.device_cursor {
            queue!(
                stdout,
                cursor::MoveTo(box_x, row),
                SetForegroundColor(Color::Black),
                Print(format!("│▶{:<w$}│", label, w = box_w as usize - 3)),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(box_x, row),
                SetForegroundColor(Color::White),
                Print(format!("│  {:<w$}│", label, w = box_w as usize - 4)),
                ResetColor
            )?;
        }
    }
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + box_h - 1),
        SetForegroundColor(Color::White),
        Print(format!("└{:─<w$}┘", "", w = box_w as usize - 2)),
        ResetColor
    )?;
    Ok(())
}

fn draw_camera_menu(
    stdout: &mut impl Write,
    state: &AppState,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    let box_w: u16 = (term_cols / 2).max(40).min(70);
    let box_h: u16 = (state.camera_list.len() as u16 + 4).min(term_rows - 4);
    let box_x = (term_cols - box_w) / 2;
    let box_y = (term_rows - box_h) / 2;

    queue!(stdout, SetForegroundColor(Color::White))?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y),
        Print(format!("┌{:─<w$}┐", "", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 1),
        Print(format!("│{:^w$}│", " Camera", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 2),
        Print(format!("├{:─<w$}┤", "", w = box_w as usize - 2))
    )?;

    for (i, name) in state.camera_list.iter().enumerate() {
        let row = box_y + 3 + i as u16;
        if row >= box_y + box_h - 1 {
            break;
        }
        let label: String = name.chars().take(box_w as usize - 4).collect();
        if i == state.camera_cursor {
            queue!(
                stdout,
                cursor::MoveTo(box_x, row),
                SetForegroundColor(Color::Black),
                Print(format!("│▶{:<w$}│", label, w = box_w as usize - 3)),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(box_x, row),
                SetForegroundColor(Color::White),
                Print(format!("│  {:<w$}│", label, w = box_w as usize - 4)),
                ResetColor
            )?;
        }
    }
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + box_h - 1),
        SetForegroundColor(Color::White),
        Print(format!("└{:─<w$}┘", "", w = box_w as usize - 2)),
        ResetColor
    )?;
    Ok(())
}

fn draw_midi_menu(
    stdout: &mut impl Write,
    state: &AppState,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    let box_w: u16 = (term_cols / 2).max(50).min(80);
    let box_h: u16 = 18.min(term_rows - 4);
    let box_x = (term_cols - box_w) / 2;
    let box_y = (term_rows - box_h) / 2;

    queue!(stdout, SetForegroundColor(Color::White))?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y),
        Print(format!("┌{:─<w$}┐", "", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 1),
        Print(format!(
            "│{:^w$}│",
            format!(
                " MIDI Config {} ",
                if state.midi_config.enabled {
                    "[ON]"
                } else {
                    "[OFF]"
                }
            ),
            w = box_w as usize - 2
        ))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 2),
        Print(format!("├{:─<w$}┤", "", w = box_w as usize - 2))
    )?;

    let param_names = [
        "None",
        "BassScale",
        "MidScale",
        "HighScale",
        "GlobalCtr",
        "Energy",
        "BpmScale",
        "EdgeThresh",
        "BgThresh",
    ];

    for i in 0..8 {
        let row = box_y + 3 + i as u16;
        if row >= box_y + box_h - 1 {
            break;
        }
        let mapping = state.midi_config.mappings.get(i);
        let (cc_str, param_str) = if let Some(m) = mapping {
            (
                format!("CC{:3}", m.cc),
                param_names[m.param as usize].to_string(),
            )
        } else {
            ("---".to_string(), "".to_string())
        };
        let line = format!("│ CC:{}  →  {:<12} │", cc_str, param_str);

        let content = if i == state.midi_osc_cursor {
            format!("▶{:<w$}◀", &line[1..line.len() - 1], w = box_w as usize - 4)
        } else {
            format!(
                "│ {:<w$} │",
                &line[2..line.len() - 2],
                w = box_w as usize - 4
            )
        };
        queue!(
            stdout,
            cursor::MoveTo(box_x, row),
            SetForegroundColor(if i == state.midi_osc_cursor {
                Color::Black
            } else {
                Color::White
            }),
            Print(content),
            ResetColor
        )?;
    }

    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 3 + 8),
        Print(format!(
            "│ {:^w$} │",
            "[e] enable  [a] add  [d] delete",
            w = box_w as usize - 4
        ))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + box_h - 1),
        SetForegroundColor(Color::White),
        Print(format!("└{:─<w$}┘", "", w = box_w as usize - 2)),
        ResetColor
    )?;
    Ok(())
}

fn draw_osc_menu(
    stdout: &mut impl Write,
    state: &AppState,
    term_cols: u16,
    term_rows: u16,
) -> anyhow::Result<()> {
    let box_w: u16 = (term_cols / 2).max(50).min(70);
    let box_h: u16 = 12.min(term_rows - 4);
    let box_x = (term_cols - box_w) / 2;
    let box_y = (term_rows - box_h) / 2;

    queue!(stdout, SetForegroundColor(Color::White))?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y),
        Print(format!("┌{:─<w$}┐", "", w = box_w as usize - 2))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 1),
        Print(format!(
            "│{:^w$}│",
            format!(
                " OSC Config {} ",
                if state.osc_config.enabled {
                    "[ON]"
                } else {
                    "[OFF]"
                }
            ),
            w = box_w as usize - 2
        ))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 2),
        Print(format!("├{:─<w$}┤", "", w = box_w as usize - 2))
    )?;

    let lines = [
        format!(
            " Target: {}:{}",
            state.osc_config.target_host, state.osc_config.target_port
        ),
        format!(" Listen: localhost:{}", state.osc_config.listen_port),
    ];

    for (i, line) in lines.iter().enumerate() {
        let row = box_y + 3 + i as u16;
        let selected = state.midi_osc_cursor == i;
        queue!(
            stdout,
            cursor::MoveTo(box_x, row),
            if selected {
                SetForegroundColor(Color::Black)
            } else {
                SetForegroundColor(Color::White)
            },
            Print(format!(
                "│▶ {:<w$} ◙",
                line.chars().take(box_w as usize - 6).collect::<String>(),
                w = box_w as usize - 4
            )),
            ResetColor
        )?;
    }

    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + 3 + 2),
        Print(format!(
            "│ {:^w$} │",
            "[p/o] target port  [l/k] listen port",
            w = box_w as usize - 4
        ))
    )?;
    queue!(
        stdout,
        cursor::MoveTo(box_x, box_y + box_h - 1),
        SetForegroundColor(Color::White),
        Print(format!("└{:─<w$}┘", "", w = box_w as usize - 2)),
        ResetColor
    )?;
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let shared_audio: SharedAudio = Arc::new(Mutex::new(BandEnergy::default()));
    let mut app_state = AppState::default();
    let mut _stream = None;
    let mut active_device_name = String::new();

    if let Some(dev) = audio::default_device() {
        let kind = if dev.is_loopback {
            "loopback"
        } else {
            "mikrofon"
        };
        eprintln!("Audio [{kind}]: {}", dev.name);
        active_device_name = dev.name.clone();
        _stream = Some(audio::start_capture(dev, shared_audio.clone())?);
    } else {
        eprintln!("Kein Audio-Gerät — laufe ohne Audio");
    }

    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera: Option<Camera> = match Camera::new(CameraIndex::Index(0), format) {
        Ok(mut c) => match c.open_stream() {
            Ok(_) => Some(c),
            Err(e) => {
                eprintln!("Camera stream error (continuing without camera): {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("No camera found (continuing without camera): {e}");
            None
        }
    };

    let shared_ir: SharedIr = ir::create_ir_source();
    let ir_cameras = ir::list_depth_cameras();
    if !ir_cameras.is_empty() {
        eprintln!("IR/Depth: {}", ir_cameras.join(", "));
    }

    let midi_handler = MidiHandler::new();
    let shared_midi: SharedMidi = midi_handler.get_state();
    if midi_handler.has_device() {
        app_state.midi_config.enabled = true;
    }

    let shared_osc: SharedOsc = osc::create_osc_receiver(app_state.osc_config.listen_port);

    let osc_sender = osc::OscSender::new(
        &app_state.osc_config.target_host,
        app_state.osc_config.target_port,
    )
    .ok();

    let mut stdout = io::stdout();
    if let Err(e) = terminal::enable_raw_mode() {
        eprintln!("Error: This program requires an interactive terminal.");
        eprintln!("  - Make sure you're running directly in a terminal (not SSH without -t)");
        eprintln!("  - Or redirecting output to a file");
        eprintln!("  - Check that /dev/tty exists");
        return Err(anyhow::anyhow!("No interactive terminal: {}", e));
    }
    if let Err(e) = execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide) {
        eprintln!("Warning: Could not enter alternate screen: {e}");
    }

    let mut running = true;

    while running {
        // ── Input ─────────────────────────────────────────────────────────
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                // Q (Shift+q) beendet das Programm aus jedem Modus
                if code == KeyCode::Char('Q') {
                    running = false;
                    break;
                }
                match handle_key(code, &mut app_state) {
                    Some(0) => {
                        // Audio device switch
                        if let Some((name, _)) = app_state.device_list.get(app_state.device_cursor)
                        {
                            let name = name.clone();
                            if name != active_device_name {
                                if let Some(dev) = audio::open_device_by_name(&name) {
                                    match audio::start_capture(dev, shared_audio.clone()) {
                                        Ok(s) => {
                                            _stream = Some(s);
                                            active_device_name = name;
                                        }
                                        Err(e) => eprintln!("Device-Fehler: {e}"),
                                    }
                                }
                            }
                        }
                        app_state.mode = UiMode::Global;
                    }
                    Some(1) => {
                        // Camera switch
                        let cam_idx = app_state.camera_cursor as u32;
                        let new_format = RequestedFormat::new::<RgbFormat>(
                            RequestedFormatType::AbsoluteHighestFrameRate,
                        );
                        match Camera::new(CameraIndex::Index(cam_idx), new_format) {
                            Ok(new_cam) => {
                                if let Some(ref mut c) = camera {
                                    c.stop_stream().ok();
                                }
                                camera = Some(new_cam);
                                if let Some(ref mut c) = camera {
                                    if let Err(e) = c.open_stream() {
                                        eprintln!("Camera error: {e}");
                                    }
                                }
                            }
                            Err(e) => eprintln!("Camera switch error: {e}"),
                        }
                        app_state.mode = UiMode::Global;
                    }
                    _ => {}
                }
            }
        }
        if !running {
            break;
        }

        // ── Audio ──────────────────────────────────────────────────────────
        let mut band = shared_audio.lock().map(|g| g.clone()).unwrap_or_default();

        let ir_data = shared_ir.lock().map(|g| g.clone()).unwrap_or_default();
        band.ir_intensity = ir_data.intensity;
        band.ir_depth = ir_data.depth;

        let midi_state = shared_midi
            .lock()
            .map(|g: std::sync::MutexGuard<'_, midi::MidiState>| g.clone())
            .unwrap_or_default();
        apply_midi_mappings(&mut app_state, &midi_state);

        let osc_state = shared_osc.lock().map(|g| g.clone()).unwrap_or_default();
        apply_osc_mappings(&mut app_state, &osc_state);

        app_state.last_beat = band.beat || osc_state.trigger;

        if app_state.bpm_sync && band.bpm > 0.0 {
            if band.confidence > 0.5 {
                if !band.beat {
                    let expected_gap = (60.0 / band.bpm * 1000.0) as u64;
                    let wait = expected_gap.min(30).max(4);
                    std::thread::sleep(Duration::from_millis(wait));
                    continue;
                }
            }
        }
        let energies = [band.bass, band.mid, band.high];

        if let Some(ref sender) = osc_sender {
            if app_state.osc_config.enabled {
                sender.send_float("/ascii/bass", energies[0]);
                sender.send_float("/ascii/mid", energies[1]);
                sender.send_float("/ascii/high", energies[2]);
                sender.send_float("/ascii/bpm", band.bpm);
                if band.beat {
                    sender.send_float("/ascii/beat", 1.0);
                }
            }
        }

        // ── Kamera ─────────────────────────────────────────────────────────
        let Some(ref mut cam) = camera else {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        };
        let frame = match cam.frame() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Frame error: {e}");
                continue;
            }
        };

        // Decode frame - try nokhwa first, fallback to zune-jpeg for MJPEG
        let res = frame.resolution();
        let width = res.width();
        let height = res.height();
        let raw_buf = frame.buffer();

        let rgb = match frame.decode_image::<RgbFormat>() {
            Ok(d) => d,
            Err(_) => {
                use std::io::Cursor;
                let mut decoder = zune_jpeg::JpegDecoder::new(Cursor::new(raw_buf.as_ref()));
                match decoder.decode() {
                    Ok(rgb_data) => match image::RgbImage::from_raw(width, height, rgb_data) {
                        Some(img) => img,
                        None => {
                            eprintln!("Buffer size mismatch: {}x{}", width, height);
                            continue;
                        }
                    },
                    Err(e) => {
                        eprintln!("JPEG decode error: {}", e);
                        continue;
                    }
                }
            }
        };
        let gray = image::DynamicImage::ImageRgb8(rgb).to_luma8();

        let (term_cols, term_rows) = match terminal::size() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Terminal size error: {e}");
                continue;
            }
        };
        let scale = app_state.char_size;
        let ascii_cols = (term_cols as u32 / scale).max(1);
        let ascii_rows = (term_rows.saturating_sub(4) as u32 / scale).max(1);

        let palette = &PALETTES[app_state.palette_idx];
        let pixel_rows = render_frame(
            &gray,
            ascii_cols,
            ascii_rows,
            &app_state,
            energies,
            band.ir_intensity,
            band.ir_depth,
            palette,
        );

        // ── Render ─────────────────────────────────────────────────────────
        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        )?;

        for (row_idx, pixels) in pixel_rows.iter().enumerate() {
            queue!(stdout, cursor::MoveTo(0, row_idx as u16))?;
            let mut last_color: Option<Color> = None;
            for px in pixels {
                // Nur Farb-Escape wenn sich Farbe ändert
                if last_color != Some(px.color) {
                    queue!(stdout, SetForegroundColor(px.color))?;
                    last_color = Some(px.color);
                }
                // char_size > 1: Zeichen wiederholen
                let ch_str: String = std::iter::repeat(px.ch).take(scale as usize).collect();
                queue!(stdout, Print(ch_str))?;
            }
            queue!(stdout, ResetColor)?;
        }

        draw_status(
            &mut stdout,
            &app_state,
            energies,
            band.bpm,
            band.confidence,
            term_cols,
            term_rows,
        )?;
        draw_debug_panel(
            &mut stdout,
            &app_state,
            energies,
            band.bpm,
            band.confidence,
            term_cols,
            term_rows,
        )?;
        if app_state.mode == UiMode::DeviceMenu {
            draw_device_menu(&mut stdout, &app_state, term_cols, term_rows)?;
        }
        if app_state.mode == UiMode::CameraMenu {
            draw_camera_menu(&mut stdout, &app_state, term_cols, term_rows)?;
        }
        if app_state.mode == UiMode::MidiMenu {
            draw_midi_menu(&mut stdout, &app_state, term_cols, term_rows)?;
        }
        if app_state.mode == UiMode::OscMenu {
            draw_osc_menu(&mut stdout, &app_state, term_cols, term_rows)?;
        }

        stdout.flush()?;
    }

    let _ = execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show);
    let _ = terminal::disable_raw_mode();
    if let Some(ref mut c) = camera {
        c.stop_stream().ok();
    }
    println!("Tschüss!");
    Ok(())
}
