# Optimization Implementation Guide

This document provides concrete implementation examples for the optimizations identified in `analysis.md`.

---

## 1. Zero-Copy Frame Buffer Reuse

### Current (layers.rs:83)
```rust
fn sobel_magnitude(pixels: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];  // NEW ALLOCATION EVERY CALL
    // ...
}
```

### Optimized - Use Thread-Local Buffer
```rust
use std::cell::LazyLock;

thread_local! {
    static SOBEL_BUF: std::cell::RefCell<Vec<f32>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn sobel_magnitude(pixels: &[f32], w: usize, h: usize) -> Vec<f32> {
    SOBEL_BUF.with(|buf| {
        let buf = &mut *buf.borrow_mut();
        if buf.len() != w * h {
            buf.resize(w * h, 0.0);
        }
        buf.fill(0.0);
        
        // Process into existing buffer
        // ... (same algorithm)
        
        buf.clone()  // Still clones, but avoids reallocation
    })
}
```

### Better - Mutable Reference Pattern
```rust
fn sobel_magnitude(pixels: &[f32], w: usize, h: usize, out: &mut [f32]) {
    out.fill(0.0);
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            // ... algorithm writes to out[y * w + x]
        }
    }
}

// Call site - pre-allocate once
let mut sobel_buf = vec![0.0f32; ascii_cols * ascii_rows];
sobel_magnitude(&resized_floats, ascii_cols as usize, ascii_rows as usize, &mut sobel_buf);
```

---

## 2. String Caching for Character Scaling

### Current (main.rs:2114)
```rust
let ch_str: String = std::iter::repeat(px.ch).take(scale as usize).collect();
```

### Optimized
```rust
struct StringCache {
    cache: Vec<String>,
    scale: u32,
}

impl StringCache {
    fn new(scale: u32) -> Self {
        let scale = scale.max(1) as usize;
        let cache: Vec<String> = (0..256)
            .map(|i| std::iter::repeat(i as char).take(scale).collect())
            .collect();
        StringCache { cache, scale: scale as u32 }
    }
    
    fn get(&self, ch: char) -> &str {
        self.cache.get(ch as usize).map(|s| s.as_str()).unwrap_or(" ")
    }
    
    fn resize(&mut self, new_scale: u32) {
        if new_scale != self.scale {
            *self = StringCache::new(new_scale);
        }
    }
}

// Usage in render loop:
let mut char_cache = StringCache::new(app_state.char_size);

// In the render loop:
if char_cache.scale != app_state.char_size {
    char_cache.resize(app_state.char_size);
}
queue!(stdout, Print(char_cache.get(px.ch)))?;
```

---

## 3. Lock-Free Audio with Atomics

### Current (audio.rs + main.rs)
```rust
// audio.rs
pub type SharedAudio = Arc<Mutex<BandEnergy>>;

// main.rs
let band = shared_audio.lock().map(|g| g.clone()).unwrap_or_default();
```

### Optimized with Atomics
```rust
use std::sync::atomic::{AtomicF32, Ordering};

#[derive(Clone)]
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

// Atomic version for lock-free reads
pub struct AtomicBandEnergy {
    pub bass: AtomicF32,
    pub mid: AtomicF32,
    pub high: AtomicF32,
    pub bpm: AtomicF32,
    pub beat: AtomicBool,
    pub confidence: AtomicF32,
    pub ir_intensity: AtomicF32,
    // ir_depth needs special handling (Option<f32>)
}

impl AtomicBandEnergy {
    pub fn new() -> Self {
        Self {
            bass: AtomicF32::new(0.0),
            mid: AtomicF32::new(0.0),
            high: AtomicF32::new(0.0),
            bpm: AtomicF32::new(0.0),
            beat: AtomicBool::new(false),
            confidence: AtomicF32::new(0.0),
            ir_intensity: AtomicF32::new(0.0),
        }
    }
    
    pub fn load(&self) -> BandEnergy {
        BandEnergy {
            bass: self.bass.load(Ordering::Relaxed),
            mid: self.mid.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
            bpm: self.bpm.load(Ordering::Relaxed),
            beat: self.beat.load(Ordering::Relaxed),
            confidence: self.confidence.load(Ordering::Relaxed),
            ir_intensity: self.ir_intensity.load(Ordering::Relaxed),
            ir_depth: None,  // Handle separately if needed
        }
    }
}
```

---

## 4. SIMD Sobel Operator (Advanced)

### Using portable_simd
```rust
use std::simd::{f32x4, SimdFloat};

fn sobel_magnitude_simd(pixels: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            // Load 3x3 neighborhood
            let row0 = &pixels[(y - 1) * w + x - 1..(y - 1) * w + x + 2];
            let row1 = &pixels[y * w + x - 1..y * w + x + 2];
            let row2 = &pixels[(y + 1) * w + x - 1..(y + 1) * w + x + 2];
            
            // Vectorized Sobel
            // gx = -1*TL + 1*TR - 2*ML + 2*MR - 1*BL + 1*BR
            // Process 4 pixels at once with SIMD
            // ... (implementation detail)
            
            out[y * w + x] = (gx.abs() + gy.abs()) / 4.0;
        }
    }
    out
}
```

### Alternative: Use Existing SIMD Crate
```rust
// Add to Cargo.toml
// simd = "0.4"
// Or use image crate's SIMD features

// image = { version = "0.25", features = ["simd"] }
```

---

## 5. Frame Skipping Mechanism

### Implementation
```rust
use std::time::{Duration, Instant};

const TARGET_FPS: f32 = 30.0;
const FRAME_TIME: Duration = Duration::from_secs_f32(1.0 / TARGET_FPS);

fn main_loop() {
    let mut last_frame = Instant::now();
    let mut frame_count: u64 = 0;
    
    loop {
        let now = Instant::now();
        let elapsed = now - last_frame;
        
        // Skip frame if we're behind
        if elapsed < FRAME_TIME {
            let sleep_time = FRAME_TIME - elapsed;
            std::thread::sleep(sleep_time);
            continue;
        }
        
        last_frame = now;
        frame_count += 1;
        
        // Process frame...
    }
}
```

### Adaptive Frame Skipping
```rust
fn adaptive_skip(last_duration: Duration, target_frame_time: Duration) -> bool {
    // Skip if last frame took too long
    last_duration > target_frame_time * 2
}
```

---

## 6. MIDI Connection Proper Cleanup

### Current (midi.rs)
```rust
impl Drop for MidiHandler {
    fn drop(&mut self) {
        self.connection = None;  // WRONG
    }
}
```

### Fixed
```rust
impl Drop for MidiHandler {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            // Close the connection properly
            drop(conn);  // midir connections close on drop, but explicit is better
        }
    }
}
```

Actually, midir's `MidiInputConnection` should close on drop. The issue might be:
1. Connection not being stored properly
2. MidiInput not being dropped

---

## 7. Profiling Setup

### Add to Cargo.toml for profiling
```toml
[dependencies]
dhat = "0.3"
perf-env = "0.1"

[profile.release]
debug = true  # For perf symbols
lto = true
codegen-units = 1
```

### Example with dhat (heap profiling)
```rust
use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOCATOR: DhatAlloc = DhatAlloc;

fn main() {
    let _dhat = Dhat::start();
    // ... your code
}
```

---

*Generated: 2026-04-01*
