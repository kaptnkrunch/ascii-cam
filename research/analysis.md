# ascii-cam Performance & Code Analysis

## Executive Summary

This document provides a comprehensive analysis of the ascii-cam codebase, identifying performance bottlenecks, code weak spots, and optimization opportunities.

---

## 1. Architecture Overview

### Module Structure

```
src/
├── main.rs      (2161 lines) - Main loop, UI, rendering, camera handling
├── audio.rs     (371 lines)  - Audio capture, FFT, BPM detection
├── charset.rs   (310 lines)  - Character set definitions and slicing
├── layers.rs    (171 lines)  - Edge detection, layer classification
├── midi.rs      (177 lines)  - MIDI input handling
├── osc.rs       (157 lines)  - OSC output
└── ir.rs        (131 lines)  - IR/depth camera integration
```

### Data Flow

```
Camera Frame → Decode (MJPEG/RGB) → Grayscale → Resize → Layer Detection
                                                              ↓
Audio Input → FFT → Band Energy → BPM Detection → Band State
                                                              ↓
                    ← Rendering Pipeline ←
```

---

## 2. Performance Bottlenecks

### 2.1 Image Processing (HIGH)

**Location**: `layers.rs`, `main.rs:576-822`

| Function | Complexity | Issue |
|----------|------------|-------|
| `sobel_magnitude` | O(w×h) | Naive iteration, no SIMD |
| `laplacian_variance` | O(w×h×r²) | Called twice with different radii |
| `detect_layers` | O(w×h) | Multiple passes over same data |
| `render_frame` | O(w×h) | Per-pixel allocation + character selection |

**Impact**: These run every frame (24-60 FPS). Each 1920x1080 frame processes ~2M pixels.

### 2.2 String Building (HIGH)

**Location**: `main.rs:2114`

```rust
let ch_str: String = std::iter::repeat(px.ch).take(scale as usize).collect();
```

Creates a new String for every character when `scale > 1`. This is O(n×scale) allocations per frame.

### 2.3 Band Lookup (MEDIUM)

**Location**: `main.rs:603-628`

```rust
let band_lookup: [Option<usize>; 6] = [
    state.bands.iter().position(|b| b.layer == VisualLayer::Black),
    // ... repeated 6 times
];
```

Uses `iter().position()` which is O(n) each. While bands array is small (max 6), this is called every frame.

### 2.4 Memory Allocations (MEDIUM)

**Locations**:
- `main.rs:649` - `Vec::with_capacity(h)` inside frame loop
- `layers.rs:50` - New `Vec<f32>` for pixel conversion every call
- `layers.rs:83,98` - `vec![0.0f32; w * h]` new allocations per frame

### 2.5 Audio Thread (LOW-MEDIUM)

**Location**: `audio.rs:265-275`

```rust
let mut spectrum: Vec<Complex<f32>> = buf
    .iter()
    .enumerate()
    .map(...)
    .collect();
```

Allocates new vector every FFT. Could reuse buffer.

---

## 3. Code Weak Spots & Issues

### 3.1 Error Handling

**Issue**: Silent failures in render loop

```rust
// main.rs:2040-2043
Err(e) => {
    eprintln!("Frame error: {e}");
    continue;  // Silently drops frame
}
```

**Problem**: No recovery strategy, camera errors accumulate in logs.

### 3.2 MIDI Handler Drop Implementation

**Location**: `midi.rs:118-122`

```rust
impl Drop for MidiHandler {
    fn drop(&mut self) {
        self.connection = None;  // Doesn't actually close connection
    }
}
```

**Problem**: Setting `None` doesn't close the MIDI connection. Should call `close()` on the connection.

### 3.3 Thread Safety

**Issue**: `BandEnergy` cloned on every frame

```rust
// main.rs:1997-2005
let band = shared_audio
    .lock()
    .map(|g: std::sync::MutexGuard<'_, audio::BandEnergy>| g.clone())
    .unwrap_or_default();
```

**Problem**: `Clone` on `BandEnergy` creates full copy every frame. Could use `Arc<AtomicF32>` for individual values.

### 3.4 Unused Code

**Location**: `layers.rs:122-161`

```rust
#[allow(dead_code)]
pub fn box_blur(...) { }
#[allow(dead_code)]
pub fn difference_of_gaussians(...) { }
```

**Problem**: Dead code clutters codebase. Either use or remove.

### 3.5 Hardcoded Values

**Locations**:
- `audio.rs:225` - `const FFT_SIZE: usize = 1024;`
- `audio.rs:226` - `const SMOOTH: f32 = 0.3;`
- `main.rs:98` - `.filter_map(|i| { (0..4)...`

**Problem**: Magic numbers scattered. Should be configurable or documented.

---

## 4. Optimization Opportunities

### 4.1 SIMD Image Processing (HIGH PRIORITY)

**Current**: Sequential iteration in `sobel_magnitude` and `laplacian_variance`

**Solution**: Use `packed_simd` or `autodiff` crates, or manually vectorize with `std::simd`

**Expected speedup**: 2-4x for Sobel/Laplacian

### 4.2 Zero-Copy Frame Processing

**Current**: 
```rust
let rgb = match frame.decode_image::<RgbFormat>() { ... };
let gray = image::DynamicImage::ImageRgb8(rgb).to_luma8();
```

**Solution**: Decode directly into owned buffer, then convert without intermediate `ImageBuffer`

### 4.3 Pre-allocated Buffers

**Current**: New allocations every frame in `layers.rs`

**Solution**: Use thread-local buffers or frame arena pattern

### 4.4 String Interning for Repeated Chars

**Current**: Building new String per character with `scale > 1`

**Solution**: 
```rust
// Pre-compute scaled strings
let scaled_chars: Vec<String> = (0..256)
    .map(|c| c as char)
    .map(|ch| std::iter::repeat(ch).take(max_scale).collect())
    .collect();
// Then just index into it
```

### 4.5 O(1) Band Lookup

**Current**: Array of `Option<usize>` from `iter().position()`

**Solution**: Use direct index mapping since we know the layer types:
```rust
let band_idx = match target_visual {
    VisualLayer::Black => Some(black_band_idx),
    // Already O(1) - but the lookup table is rebuilt every frame
};
```

### 4.6 Lock-Free Audio State

**Current**: `Mutex<BandEnergy>` locked every frame

**Solution**: Use atomics for individual values:
```rust
struct BandEnergy {
    bass: AtomicF32,
    mid: AtomicF32,
    // etc.
}
```

### 4.7 Frame Skipping with Timestamp

**Current**: Processes every frame regardless of timing

**Solution**: Skip frames if behind:
```rust
let frame_time = std::time::Instant::now();
if frame_time - last_frame < target_frame_time {
    continue; // Skip to reduce CPU
}
```

---

## 5. Recommended Priority Order

| Priority | Optimization | Est. Speedup | Effort |
|----------|--------------|--------------|--------|
| 1 | Add mjpeg feature | Fix crash | Low |
| 2 | Pre-allocated buffers | 10-20% | Medium |
| 3 | String caching | 5-10% | Low |
| 4 | Lock-free audio | 5-10% | Medium |
| 5 | SIMD image proc | 50-100% | High |
| 6 | Frame skipping | Variable | Low |

---

## 6. Profiling Recommendations

To validate these findings, run:

```bash
# CPU profiling
perf record -g ./target/release/ascii-cam
perf report

# Memory allocation
cargo install cargo-instrumentation
cargo instrumentation ./target/release/ascii-cam

# Flamegraph
cargo install cargo-flamegraph
cargo flamegraph ./target/release/ascii-cam
```

---

*Generated: 2026-04-01*
