# Ascii-Cam Workflow Documentation

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ASCII-CAM APPLICATION                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
        ┌──────────────────────────────┼──────────────────────────────┐
        │                              │                              │
        ▼                              ▼                              ▼
┌───────────────┐            ┌───────────────┐            ┌───────────────┐
│    CAMERA     │            │    AUDIO      │            │    MIDI/OSC   │
│   INPUT      │            │   INPUT      │            │    INPUT      │
│  (nokhwa)    │            │   (cpal)     │            │  (midir/rosc) │
└──────┬────────┘            └──────┬────────┘            └──────┬────────┘
       │                           │                           │
       ▼                           ▼                           │
┌───────────────┐            ┌───────────────┐                 │
│  MJPEG/RGB   │            │    FFT        │                 │
│   DECODE     │            │  PROCESSING   │                 │
└──────┬────────┘            └───────┬───────┘                 │
       │                            │                         │
       ▼                            ▼                         │
┌───────────────┐            ┌───────────────┐            ┌───────────────┐
│  GRAYSCALE   │◄───────────►│  BAND ENERGY  │            │  MIDI STATE   │
│  CONVERSION  │            │  DETECTION    │            │    UPDATE     │
└──────┬────────┘            └───────┬───────┘            └───────┬───────┘
       │                            │                           │
       ▼                            ▼                           ▼
┌───────────────┐            ┌───────────────┐            ┌───────────────┐
│ LAYER         │            │   BPM         │            │  OSC STATE    │
│ DETECTION     │            │   DETECTOR    │            │   UPDATE      │
│ (Sobel/Lap)   │            └───────────────┘            └───────────────┘
───────┬────────┘                         │                       
       │                                 │                        
       ▼                                 │                        
┌───────────────┐                        │                        
│ PER-PIXEL     │◄────────────────────────┘                        
│ RENDERING     │                                             
───────┬────────┘                                             
       │                                                      
       ▼                                                      
┌───────────────┐                                             
│ TERMINAL      │                                             
│ OUTPUT       │                                             
│ (crossterm)  │                                             
└───────────────┘                                             
```

---

## Component Flow

### 1. Camera Pipeline (24-60 FPS)

```
Camera Frame (1280x720)
        │
        ▼
┌───────────────────┐
│  Frame Decode    │  ← nokhwa::Buffer::decode_image
│  (MJPEG → RGB)   │    or zune-jpeg fallback
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ Grayscale Conv   │  ← image::DynamicImage::to_luma8
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│   Image Resize    │  ← image::imageops::resize (Nearest)
│  (→ ASCII grid)  │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Layer Detection │  ← sobel_magnitude + laplacian_variance
│  (5 layers)      │
└───────────────────┘
```

### 2. Audio Pipeline (Continuous)

```
Audio Input Stream
        │
        ▼
┌───────────────────┐
│  Ring Buffer      │  ← HeapRb<f32> (4x FFT_SIZE)
│  (async)         │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  FFT             │  ← rustfft (1024-point FFT)
│  Spectrum        │
└────────┬──────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│ Bass │ │  Mid  │  ← band_rms (frequency ranges)
│ 20-  │ │300-4k │
│ 300Hz│ │  Hz   │
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│ High │ │ Beat  │  ← BpmDetector
│4k-20k│ │ Detect│
└───┬───┘ └───┬───┘
    │         │
    └────┬────┘
         │
         ▼
┌───────────────────┐
│  Shared State     │  ← Arc<Mutex<BandEnergy>>
│  (atomic update)  │
└───────────────────┘
```

### 3. Render Pipeline (Per Frame)

```
┌───────────────────┐
│ Frame Start       │  ← Main loop timing
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Lock Audio       │  ← Mutex (blocking)
│  State            │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Lock MIDI/OSC    │  ← Mutex (blocking)
│  State            │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Process Camera   │  ← Steps 1-5 from Camera Pipeline
│  Frame            │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Build Band       │  ← iter().position() lookup
│  Lookup           │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Per-Pixel        │  ← For each cell in ASCII grid
│  Loop             │    - Luma + contrast
│                   │    - Layer + band selection
│                   │    - Mode application
│                   │    - Character selection
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Terminal Render  │  ← ANSI escape sequences
│  (crossterm)      │    - Move cursor
│                   │    - Set color
│                   │    - Print characters
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Status Overlay   │  ← FPS, BPM, energies
│  (conditional)    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  Flush stdout    │  ← Critical for performance
└────────┬──────────┘
         │
         ▼
    Frame Complete
```

---

## Data Structures

### BandEnergy (Shared State)

```rust
pub struct BandEnergy {
    pub bass: f32,        // 0.0 - 1.0
    pub mid: f32,         // 0.0 - 1.0  
    pub high: f32,        // 0.0 - 1.0
    pub bpm: f32,         // BPM value
    pub beat: bool,       // Beat trigger
    pub confidence: f32,   // 0.0 - 1.0
    pub ir_intensity: f32, // IR camera data
    pub ir_depth: Option<f32>,
}
```

### AppState (Main Application)

```rust
pub struct AppState {
    pub mode: UiMode,           // Current UI mode
    pub palette_idx: usize,     // Active color palette
    pub char_size: u32,         // Character scale (1-4)
    pub global_contrast: f32,   // Global contrast (0.2-4.0)
    pub energy_responsiveness: f32, // Audio reactivity
    pub bands: [Band; 3],       // Audio-reactive bands
    pub base: BaseLayer,        // Background layer
    pub midi_config: MidiConfig,
    pub osc_config: OscConfig,
    // ... 30+ fields
}
```

---

## Thread Model

```
┌─────────────────────────────────────────────────────────────────┐
│                        MAIN THREAD                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐│
│  │  UI Input   │  │  Rendering  │  │  Terminal I/O            ││
│  │  (crossterm)│  │  (frames)   │  │  (stdout)                ││
│  └──────┬──────┘  └──────┬──────┘  └────────────┬──────────────┘│
│         │                 │                      │               │
│         └────────────────┴──────────────────────┘               │
│                              │                                  │
│                              ▼                                  │
│                    ┌─────────────────────┐                     │
│                    │   Shared State      │                     │
│                    │  Arc<Mutex<...>>    │◄──────────┐          │
│                    └─────────────────────┘           │          │
└──────────────────────────────────────────────────────┼──────────┘
                                                   │          
                    ┌───────────────────────────────┘          
                    ▼                                            
┌─────────────────────────────────────────────────────────────────┐
│                      AUDIO THREAD                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  Audio     │  │   FFT      │  │  Band Energy           │ │
│  │  Capture   │──►│  Process   │──►│  Update                │ │
│  │  (cpal)    │  │  (rustfft) │  │  (Mutex write)         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Timing Analysis

| Operation | Typical Time | Target |
|-----------|--------------|--------|
| Camera frame grab | 5-15ms | < 33ms |
| JPEG decode | 2-8ms | < 33ms |
| Grayscale + resize | 0.5-2ms | < 33ms |
| Layer detection | 1-4ms | < 33ms |
| Per-pixel render | 2-8ms | < 33ms |
| Terminal output | 3-10ms | < 33ms |
| **Total per frame** | **15-50ms** | **< 33ms** |

### Bottleneck Analysis

| Component | Time | % of Frame | Priority |
|-----------|------|------------|----------|
| Terminal output | 10ms | 25% | HIGH |
| Layer detection | 4ms | 10% | HIGH |
| JPEG decode | 8ms | 20% | MEDIUM |
| Per-pixel render | 8ms | 20% | MEDIUM |
| Other | 15ms | 25% | LOW |

---

## Control Flow (UI Modes)

```
┌──────────────────────────────────────────────────────────────────┐
│                        GLOBAL MODE                                │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐            │
│  │ r/t     │  │ f/h     │  │ v/b     │  │ c/z     │            │
│  │ contrast│  │response │  │ char_sz │  │palette  │            │
│  └───┬─────┘  └───┬─────┘  └───┬─────┘  └───┬─────┘            │
│      │            │            │            │                   │
│      └────────────┴────────────┴────────────┘                   │
│                              │                                   │
│         ┌────────┬────────────┼────────────┬────────┐            │
│         │        │            │            │        │            │
│         ▼        ▼            ▼            ▼        ▼            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │Band 0-2  │ │  Base    │ │ Device  │ │ Camera  │ │  MIDI   │ │
│  │Mode     │ │  Mode    │ │ Menu    │ │  Menu   │ │  Menu   │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
│                           │                                       │
│                           ▼                                       │
│                    ┌──────────┐                                    │
│                    │  ESC/g   │  ← Return to Global              │
│                    └──────────┘                                    │
└──────────────────────────────────────────────────────────────────┘
```

---

## Startup Sequence

```
1. Initialize terminal (crossterm)
       │
       ▼
2. Load/scan devices
   ├─ Audio devices (cpal)
   ├─ Camera devices (nokhwa)
   ├─ MIDI devices (midir)
   └─ OSC config (rosc)
       │
       ▼
3. Create shared state
   ├─ Arc<Mutex<BandEnergy>>
   ├─ Arc<Mutex<MidiState>>
   └─ Arc<Mutex<OscState>>
       │
       ▼
4. Start audio thread (if enabled)
       │
       ▼
5. Open camera stream (if available)
       │
       ▼
6. Enter main loop
   └─ Process frames, handle input, render
       │
       ▼
7. Cleanup on exit
   ├─ Stop camera
   ├─ Disable raw mode
   └─ Leave alternate screen
```

---

## Error Recovery

| Error | Recovery Strategy |
|-------|-------------------|
| Camera disconnect | Continue without camera, retry open |
| Audio device error | Fall to null device, show warning |
| Decode failure | Skip frame, try fallback decoder |
| Terminal resize | Recalculate ASCII grid dimensions |
| MIDI device removal | Continue without MIDI, keep last state |

---

*Generated: 2026-04-01*
