# Code Analysis Agent Skill

## Description

Analyzes the ascii-cam Rust codebase for performance bottlenecks, code weak spots, and optimization opportunities. Creates detailed research reports in the `research/` directory.

## Capabilities

- **Architecture Analysis**: Understand module structure and data flow
- **Performance Profiling**: Identify CPU/memory bottlenecks in hot paths
- **Code Review**: Find bugs, dead code, anti-patterns, thread safety issues
- **Optimization Research**: Find solutions and estimate speedups
- **Documentation**: Create research/*.md files with findings

## Workflow

1. **Explore Codebase**
   - Read all Rust source files in `src/`
   - Understand module dependencies and data flow
   - Check `Cargo.toml` for dependencies and features

2. **Identify Bottlenecks**
   - Focus on hot paths (main loop, audio thread, render functions)
   - Look for: allocations, locks, sequential iterations, string building
   - Check for repeated computations in loops
   - Analyze each step in the main render pipeline

3. **Find Weak Spots**
   - Error handling patterns (silent failures)
   - Thread safety issues (blocking locks)
   - Unused/hardcoded code
   - Missing config options
   - Resource cleanup (MIDI, camera, threads)

4. **Research Solutions**
   - Look for Rust SIMD/image processing best practices
   - Consider zero-copy patterns
   - Research lock-free data structures
   - Find frame-skipping strategies

5. **Document Findings**
   - Create `research/` directory
   - Create `research/analysis.md` with executive summary
   - Create `research/implementations.md` with code examples
   - Create topic-specific files: bottlenecks.md, weak-spots.md, profiling.md

## Output Structure

```
ascii-cam/
├── research/
│   ├── analysis.md         # Main findings document (REQUIRED)
│   ├── implementations.md  # Code examples for optimizations
│   ├── bottlenecks.md      # Detailed bottleneck analysis
│   ├── weak-spots.md       # Code quality issues
│   └── profiling.md        # Profiling recommendations
├── skill.md                # Debug agent skill (existing)
└── research/skill.md       # This file
```

## Key Areas to Analyze

### Main Files
- `src/main.rs` - Main loop (2161 lines), rendering pipeline, camera handling
- `src/audio.rs` - FFT processing, BPM detection, audio thread
- `src/layers.rs` - Edge detection, layer classification (Sobel, Laplacian)
- `src/charset.rs` - Character set definitions, slicing by contrast

### Hot Paths
- Render loop (runs 30-60 times per second)
- Image processing (Sobel, Laplacian, layer detection)
- Terminal output (string building, ANSI escape sequences)
- Audio processing thread (FFT, band energy)

### Dependencies
- `nokhwa` - Camera capture (check for missing features like `mjpeg`)
- `cpal` - Audio I/O
- `image` - Image processing
- `rustfft` - FFT computation

## Important Checks

### Features & Build
- [ ] Missing feature flags (e.g., `mjpeg` for nokhwa)
- [ ] Correct platform-specific features (v4l for Linux)
- [ ] Release profile optimizations

### Performance
- [ ] Memory allocations in hot paths
- [ ] Lock contention in shared state (Mutex/Arc)
- [ ] String allocations in render loop
- [ ] SIMD opportunities in image processing
- [ ] Frame timing and skip logic

### Code Quality
- [ ] Error handling (silent failures?)
- [ ] Resource cleanup (MIDI, camera connections)
- [ ] Thread safety (try_lock vs lock)
- [ ] Dead code (unused functions)
- [ ] Hardcoded values (magic numbers)

## Command Reference

```bash
# Build
cargo build --release

# Check dependencies features
cargo tree -p nokhwa -i -f "{p} {f}"

# Profile
perf record -g ./target/release/ascii-cam
perf report

# Flamegraph
cargo install cargo-flamegraph
cargo flamegraph --release --bin ascii-cam
```

## Notes

- Main loop target: 30-60 FPS
- Audio runs in separate thread with its own processing
- MJPEG decoding needs explicit `mjpeg` feature in Cargo.toml
- Terminal output can be a bottleneck with large char_size
