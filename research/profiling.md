# Profiling Guide

This document provides instructions for profiling ascii-cam to validate optimization opportunities.

---

## Quick Start

### 1. Basic Performance Check

```bash
# Build release binary
cd /home/crunch/ascii-cam
cargo build --release

# Time execution
time ./target/release/ascii-cam
```

### 2. CPU Profiling with perf

```bash
# Record profiling data
perf record -g ./target/release/ascii-cam

# View report
perf report

# Generate annotated source
perf annotate --stdio
```

### 3. Flamegraph

```bash
# Install cargo-flamegraph if needed
cargo install cargo-flamegraph

# Generate flamegraph
cargo flamegraph --bin ascii-cam

# Or for release build
cargo flamegraph --release --bin ascii-cam

# Output to file
cargo flamegraph --release --bin ascii-cam > flamegraph.svg
```

---

## Detailed Profiling Steps

### Step 1: Identify Hot Functions

Run flamegraph and look for:
- Functions taking most time (widest in flamegraph)
- Allocation hotspots
- Lock contention

Expected hot functions:
- `render_frame`
- `detect_layers`
- `sobel_magnitude`
- `laplacian_variance`
- Main loop body

### Step 2: Measure Frame Time

Add frame timing to `main.rs`:

```rust
use std::time::Instant;

fn main() {
    let mut frame_times = Vec::new();
    
    loop {
        let frame_start = Instant::now();
        
        // ... frame processing ...
        
        let frame_duration = frame_start.elapsed();
        frame_times.push(frame_duration);
        
        // Print every 60 frames
        if frame_times.len() % 60 == 0 {
            let avg = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
            println!("Avg frame time: {:?}", avg);
            frame_times.clear();
        }
    }
}
```

### Step 3: Allocation Profiling

```bash
# Install dhat
cargo install dhat

# Add to main.rs
use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOCATOR: DhatAlloc = DhatAlloc;

fn main() {
    let _dhat = Dhat::start();
    // ... your code
}
```

Then run and check output for:
- Total allocations per frame
- Allocation hotspots
- Memory growth over time

### Step 4: Lock Contention

```bash
# Install loom or parking_lot for lock analysis
# Or add timing to mutex operations:

use std::time::Instant;

let lock_start = Instant::now();
let band = shared_audio.lock().unwrap();
let lock_time = lock_start.elapsed();

if lock_time > Duration::from_millis(1) {
    eprintln!("Audio lock wait: {:?}", lock_time);
}
```

---

## Benchmarking Specific Functions

### Benchmark: sobel_magnitude

```rust
use std::time::Instant;

fn benchmark_sobel() {
    let sizes = [(80, 40), (160, 80), (320, 160)];
    
    for (w, h) in sizes {
        let pixels: Vec<f32> = (0..w*h).map(|_| rand::random::<f32>()).collect();
        
        let start = Instant::now();
        for _ in 0..1000 {
            sobel_magnitude(&pixels, w, h);
        }
        let elapsed = start.elapsed();
        
        println!("{}x{}: {:?}", w, h, elapsed / 1000);
    }
}
```

### Benchmark: render_frame

```rust
fn benchmark_render() {
    let gray = GrayImage::new(80, 40);
    let state = AppState::default();
    
    let start = Instant::now();
    for _ in 0..100 {
        let _ = render_frame(&gray, 80, 40, &state, [0.5; 3], 0.0, None, &PALETTES[0]);
    }
    let elapsed = start.elapsed();
    
    println!("render_frame: {:?}", elapsed / 100);
}
```

---

## Interpreting Results

### Target Metrics

- Frame time: < 33ms (30 FPS minimum)
- Frame time: < 16ms (60 FPS target)
- Audio latency: < 50ms

### Common Issues

| Symptom | Likely Cause |
|---------|--------------|
| High CPU, low GPU | Image processing in main loop |
| Variable frame time | Allocation in hot path |
| Audio lag | Mutex contention |
| Memory growth | Missing buffer reuse |
| Slow startup | Large dependency init |

---

## Linux-Specific Tools

### perf

```bash
# Basic CPU profiling
perf record -g ./target/release/ascii-cam
perf report

# Sample every 1000 events
perf record -c 1000 -g ./target/release/ascii-cam

# Show hot functions
perf top -g

# Annotate source
perf annotate -d src/main.rs
```

### /usr/bin/time

```bash
# Measure resources
/usr/bin/time -v ./target/release/ascii-cam

# Output shows:
# - Maximum resident set size (RSS)
# - User time
# - System time
# - Page faults
```

---

## Profiling in CI

Add to your CI pipeline:

```bash
# Build with profiling
cargo build --release

# Run basic perf
perf record -g --call-graph dwarf ./target/release/ascii-cam &
PERF_PID=$!
sleep 10
kill $PERF_PID
wait $PERF_PID

# Generate report
perf report --stdio | head -50
```

---

## Optimizing Based on Results

### If Sobel/Laplacian is hot:
- Add SIMD with `packed_simd` crate
- Pre-allocate output buffers
- Consider GPU processing

### If allocations are hot:
- Use thread-local buffers
- Reuse buffers between frames
- Use `String::from_utf8` with capacity

### If mutex is hot:
- Switch to atomics
- Use read-write locks
- Reduce lock frequency

---

*Generated: 2026-04-01*
