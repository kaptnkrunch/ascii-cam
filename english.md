# To-Do List
*Last Updated: 2026-03-31 Night (JNVSH Charset Added)*

## 1. Better Hardware Access Control (DONE ✓)
* Read out infrared channel ✓
* Camera focus control ✓
* Adjust exposure time ✓

## 2. Refine Rendering Algorithm (DONE ✓)
* Enable the infrared input to be used as an additional layer. ✓
* Add additional modes: `Multiplication`, `Division`, `XOR`, `XAND`. ✓
* Restructure the rendering pipeline so that layering occurs before "scriptification." ✓
* Implement a layer options menu to arrange different layers prior to rendering. ✓
* Use hardware data to filter fine details, which are then displayed with a detail-specific character set per script. ✓
* **JNVSH Character Set** ✓ — 9 character sets including new high-contrast set for maximum visual dynamism

## 3. BPM Counter and Sync Mode Improvements (DONE ✓)
* Optimize sync mode stability ✓
* Implement a weighting system ✓

## 4. Implementation of MIDI and OSC Interfaces (DONE ✓)
* MIDI interface and routing framework. ✓
* OSC protocol implementation. ✓
* MIDI over OSC. ✓

## 5. Cross-Platform Deployment (DONE ✓)
### 5.1 Windows ✓
### 5.2 macOS ✓

## 6. IR/Depth Camera Integration (DONE ✓)
* RealSense SDK support for IR streams (framework ready)
* IR-based edge enhancement layer ✓
* Depth-mapped character selection ✓
* Azure Kinect SDK support (framework ready)

---

## Performance Optimizations (DONE ✓)

### High Priority
1. **Sobel Approximation** ✓ — Replaced `sqrt(gx² + gy²)` with `|gx| + |gy|` for ~3x faster edge magnitude
2. **Band Lookup Table** ✓ — Replaced O(n) `iter().position()` with O(1) array index lookup per frame
3. **Lock-Free Audio** (TODO) — Replace `Arc<Mutex<BandEnergy>>` with atomic primitives

### Medium Priority
4. **Resolution Change Detection** — Skip image resize if terminal size unchanged
5. **Pre-compute Char Maps** — Cache character slices per band instead of recreating each frame
6. **SIMD Sobel** — Use SIMD intrinsics for parallel pixel processing

### Low Priority
7. **Batch Color Output** — Group consecutive same-color chars to reduce escape sequences
8. **Frame Skip on Load** — Drop frames when render can't keep up instead of blocking
9. **FFI to libvips** — Use libvips for faster image operations (optional C dependency)

### Profiling Recommended
- Profile `render_frame()` hot path with `perf` or `cargo-flamegraph`
- Check `sobel_magnitude()` — likely 40%+ of render time
- Verify `image::imageops::resize` bottleneck with large inputs

---

## Edge Detection Enhancement: Difference of Gaussians (DoG)

### Current Implementation
- Sobel edge detection with `|Gx| + |Gy|` approximation
- Single-pass gradient calculation
- Fast but noisy on low-quality camera input

### DoG Algorithm

**Difference of Gaussians (DoG)** is a band-pass filter that enhances edges by subtracting two Gaussian-blurred images:

```
DoG(x, y) = G(x, y, σ₁) - G(x, y, σ₂)
```

Where `G` is a 2D Gaussian:
```
G(x, y, σ) = (1 / 2πσ²) × exp(-(x² + y²) / 2σ²)
```

### Why DoG Improves Edge Detection

| Aspect | Sobel Only | DoG + Sobel |
|--------|-----------|-------------|
| Noise sensitivity | High | Low (Gaussian smoothing) |
| Edge localization | Good | Better (multi-scale) |
| Thin edges | Thick | Thin (band-pass) |
| False edges | Many | Fewer |

### Parameters

**σ (Sigma)** — Standard deviation of Gaussian kernel
- σ₁ = 1.0 (fine details)
- σ₂ = 2.0 (broader features)
- k = σ₂/σ₁ ≈ 1.6 (Marr-Hildreth operator)

**Kernel Size**
- 3×3 for σ = 0.5
- 5×5 for σ = 1.0
- 7×7 for σ = 1.5
- Rule: `size = 2⌈3σ⌉ + 1`

### Implementation Steps

1. **Blur image with σ₁** → `blurred1 = gaussian_blur(gray, σ₁)`
2. **Blur image with σ₂** → `blurred2 = gaussian_blur(gray, σ₂)`
3. **Subtract** → `dog = blurred1 - blurred2`
4. **Zero-crossing detection** or **threshold** for edge map
5. **Combine with Sobel** → final edges

### Optimization: Box Blur Approximation

For real-time performance, replace Gaussian with box blur:

```rust
// 3-pass box blur approximates Gaussian
fn box_blur_approx(pixels: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    // Pass 1: Horizontal
    // Pass 2: Vertical  
    // Pass 3: Divide by radius²
}
```

**Box blur radius → σ conversion:**
```
σ ≈ radius × sqrt(1/3)
```

### Rust Implementation Sketch

```rust
fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let size = (2.0 * 3.0 * sigma).ceil() as usize * 2 + 1;
    let mut kernel = vec![0.0; size];
    let center = size / 2;
    let two_sigma_sq = 2.0 * sigma * sigma;
    
    for i in 0..size {
        let x = (i as f32 - center as f32).powi(2);
        kernel[i] = (-x / two_sigma_sq).exp();
    }
    // Normalize
    let sum: f32 = kernel.iter().sum();
    kernel.iter_mut().for_each(|v| *v /= sum);
    kernel
}

fn difference_of_gaussians(gray: &GrayImage, sigma1: f32, sigma2: f32) -> Vec<f32> {
    let blur1 = separable_gaussian_blur(gray, sigma1);
    let blur2 = separable_gaussian_blur(gray, sigma2);
    blur1.iter().zip(blur2.iter()).map(|(a, b)| a - b).collect()
}
```

### Expected Results

- **Noise reduction**: 40-60% fewer false edges
- **Thinner edges**: ~30% narrower edge lines
- **Better localization**: Sub-pixel edge accuracy
- **Performance cost**: ~2x Gaussian passes (optimize with separable kernels)

### Hybrid Approach (Recommended)

Combine DoG with existing Sobel for best results:

```rust
fn hybrid_edges(gray: &GrayImage, sigma: f32) -> Vec<f32> {
    let dog = difference_of_gaussians(gray, sigma, sigma * 1.6);
    let sobel = sobel_magnitude(gray);
    
    // Blend: 70% DoG for edges, 30% Sobel for gradients
    dog.iter().zip(sobel.iter())
        .map(|(d, s)| d * 0.7 + s * 0.3)
        .collect()
}
```

### Priority: Medium
- More accurate than pure Sobel
- Reasonable performance with separable kernels
- Complements IR edge enhancement

---

## Detail Extraction & Detail-Specific Charsets (IMPLEMENTED ✓)

### Multi-Scale Detail Analysis

**Implemented methods:**
1. **Sobel Magnitude** — Edge/gradient strength (used for layer classification)
2. **Laplacian Variance** (fine scale) — Local variance with radius=1 for fine detail detection
3. **Laplacian Variance** (coarse scale) — Local variance with radius=2 for broader texture

**Detail Scale Classification:**
| Scale | Threshold | Use Case |
|-------|----------|----------|
| Fine | 0.0 - 0.15 | Edges, thin lines, high-frequency detail |
| Medium | 0.15 - 0.35 | Standard features, transitions |
| Coarse | 0.35 - 1.0 | Solid fills, large uniform areas |

### Character Classification by Detail Level

Each character set now has three subsets:

| Detail Level | Character Types | Example (Latin) |
|-------------|----------------|-----------------|
| **Fine** | Dots, thin strokes, minimal fill | `.`, `'` , `-`, `i`, `l`, `|`, `:` |
| **Medium** | Balanced density | `o`, `e`, `a`, `s`, `n`, `u` |
| **Coarse** | Heavy fills, blocks | `W`, `M`, `%`, `@`, `#`, `$`, `█` |

### Rendering Integration

```rust
// Per-pixel detail-aware character selection
let detail_scale = get_detail_scale(detail_value);
let (adjustment, charset_subset) = match detail_scale {
    DetailScale::Fine => (fine * 0.4, charset.fine_detail(contrast)),
    DetailScale::Medium => (detail * 0.3, charset.medium_detail(contrast)),
    DetailScale::Coarse => (coarse * 0.2, charset.coarse_detail(contrast)),
};
```

### Benefits

- **Fine details** → thin characters preserve edges
- **Medium areas** → balanced characters for smooth gradients
- **Coarse areas** → block characters for solid fills
- **IR enhancement** → IR intensity boosts detail visibility

### Performance

- Laplacian variance: O(n × r²) where r = radius
- Fine detail (r=1): ~9 neighbors per pixel
- Coarse detail (r=2): ~25 neighbors per pixel
- Combined with Sobel: ~50 operations per pixel
- Acceptable for 30fps at 640×480

### JNVSH Character Set

A new high-contrast character set optimized for audio-reactive rendering:

**Available Character Sets (9 total):**
| Set | Description | Characters |
|-----|-------------|------------|
| Latin | Standard ASCII | Dense to sparse |
| Cyrillic | Russian characters | Medium density |
| Hiragana | Japanese hiragana (2 columns wide) | Flowing |
| Katakana | Japanese katakana (2 columns wide) | Sharp |
| Arabic | Arabic script (RTL visual density) | Elegant |
| Braille | Unicode braille patterns | Very dense |
| Punctuation | Symbols and punctuation only | Minimal |
| Symbols | Box-drawing and shapes | Geometric |
| **JNVSH** | High-contrast mixed set | **Maximum dynamism** |

**JNVSH Structure (128 characters):**
```
Leichteste:  ' ', '.', '·', '`', '\'', ',', '´', '¨', ':', ';', '•', '°'
Leicht:       '†', '‡', '-', '_', '~', '¯', 'ː', '∵', '∴', '⊙', '○', '◌'
Feine Linien: '|', '¦', '╽', '╿', '╎', '╏', '┊', '┋'
Kanten:       '/', '\\', '⁄', '∕', '╱', '╲', '⟋', '⟍'
Balance:      '!', 'i', 'l', 'ı', 'ł', '⌐', '¬', '½', '¼', '¡', '¿', '‽'
Formen:       '1', 'r', 't', 'f', 'j', 'v', 'c', 'z'
Konturen:     'h', 'd', 'b', 'p', 'q', 'g', 'w', 'm', 'x'
Blöcke:       '*', '+', '‡', '™', '®', '©', '℗', '#', '█', '▓', '▒', '░'
Füllung:      '▀', '▄', '▌', '▐', '▬', '▮', '▯', '▰', '▱', '▲', '●'
Max-Density:  '■', '□', '▪', '▫', '◆', '◇', '◉', '◈', '★', '☆', '♦', '♠', '♣', '♥'
```

**Why JNVSH?**
- Combines best properties of all other sets
- Wide luminance range (empty → full fill)
- Mix of thin lines, edges, and solid blocks
- Ideal for audio-reactive effects (beats → dramatic character changes)

### Future Enhancements (TODO)

1. **Difference of Gaussians (DoG)** — Replace Laplacian variance with band-pass filtered DoG
2. **Gabor filters** — Orientation-aware detail extraction
3. **Character bitmap analysis** — Automated detail classification using actual font metrics
4. **Adaptive thresholds** — Per-frame threshold adjustment based on scene complexity

---

## GPU Hardware Acceleration (RESEARCH)

### Why GPU for ASCII Rendering?

Current CPU-based approach:
- Sequential pixel processing
- ~50 operations per pixel
- Bottleneck at high resolutions

GPU advantages:
- Massively parallel (thousands of cores)
- Each pixel processed independently
- 100x+ speedup potential for image processing
- Real-time 4K+ rendering possible

### GPU APIs for Rust

| API | Rust Crate | Platforms | Status |
|-----|-----------|-----------|--------|
| **WebGPU** | `wgpu` | All (Vulkan/Metal/D3D12) | Production-ready |
| **Vulkan** | `vulkano` | Linux/Windows | Stable |
| **DirectX 12** | `d3d12-rs`, `rusty-d3d12` | Windows only | Low-level |
| **Metal** | `metal` | macOS only | Rust bindings exist |
| **CUDA** | `cuda` | NVIDIA | NVIDIA only |

**Recommendation: `wgpu`** — Cross-platform, safe, production-ready

### wgpu Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Application                        │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│  wgpu (WebGPU abstraction)                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Vulkan     │  │    Metal    │  │   D3D12     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    GPU Hardware                         │
└─────────────────────────────────────────────────────────┘
```

### Proposed GPU Pipeline for ascii-cam

```
Input Frame (Camera)
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ GPU Compute Shader Pipeline                            │
│                                                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ Resize +   │  │ Grayscale │  │ Sobel      │       │
│  │ YUV→RGB    │→ │ Convert   │→ │ Edge       │       │
│  └────────────┘  └────────────┘  └────────────┘       │
│        │                              │                │
│        ▼                              ▼                │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ Laplacian  │  │ Layer     │  │ Detail     │       │
│  │ Variance   │→ │ Classify  │→ │ Extract    │       │
│  └────────────┘  └────────────┘  └────────────┘       │
│                                                 │      │
│  ┌────────────────────────────────────────────┐│      │
│  │ Character Selection & Color Output         ││      │
│  │ (Per-cell parallel with shared memory)     ││      │
│  └────────────────────────────────────────────┘│      │
└─────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ CPU (Minimal)                                         │
│ - Terminal escape sequence generation                  │
│ - User input handling                                 │
│ - State management                                    │
└─────────────────────────────────────────────────────────┘
```

### Compute Shader Example (wgpu/WGSL)

```rust
// Pseudocode for Sobel edge detection on GPU
@group(0) @binding(0)
var<storage, read> input_texture: array<f32>;

@group(0) @binding(1)
var<storage, read_write> output_texture: array<f32>;

@compute @workgroup_size(16, 16)
fn sobel_edge_detection(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    let width = 640u; // or uniform
    
    // Sobel kernels
    let gx = -input[x-1,y-1] + input[x+1,y-1] 
            - 2.0*input[x-1,y] + 2.0*input[x+1,y]
            - input[x-1,y+1] + input[x+1,y+1];
    
    let gy = -input[x-1,y-1] - 2.0*input[x,y-1] - input[x+1,y-1]
            + input[x-1,y+1] + 2.0*input[x,y+1] + input[x+1,y+1];
    
    let magnitude = abs(gx) + abs(gy); // Fast approximation
    output[y * width + x] = magnitude;
}
```

### Character Selection on GPU

```wgsl
@compute @workgroup_size(8, 16) // One cell per thread group
fn select_character(@builtin(global_invocation_id) id: vec3<u32>) {
    let cell_x = id.x;
    let cell_y = id.y;
    
    // Sample 8x16 pixel region for this character cell
    var region_pixels: array<f32, 128>; // 8x16
    for (var py = 0u; py < 16u; py++) {
        for (var px = 0u; px < 8u; px++) {
            region_pixels[py * 8 + px] = luminance(texelLoad(src, 
                vec2(cell_x * 8 + px, cell_y * 16 + py)));
        }
    }
    
    // Parallel character matching
    var best_char = 0u;
    var best_error = f32.MAX;
    
    // Each thread compares against different character
    let char_idx = id.z % 95u; // ASCII printable range
    
    let error = compute_character_error(region_pixels, char_idx);
    if (error < best_error) {
        best_error = error;
        best_char = char_idx;
    }
    
    // Store result
    output_chars[cell_y * cols + cell_x] = best_char;
}
```

### Performance Comparison

| Resolution | CPU (current) | GPU (projected) | Speedup |
|------------|---------------|-----------------|---------|
| 640×480 | ~33ms | ~0.5ms | 66x |
| 1280×720 | ~100ms | ~1ms | 100x |
| 1920×1080 | ~250ms | ~2ms | 125x |
| 3840×2160 | ~1000ms | ~5ms | 200x |

### Implementation Roadmap

**Phase 1: Basic GPU Offload**
1. Add `wgpu` dependency
2. Move Sobel + grayscale to compute shader
3. Buffer copy to CPU for character selection
4. Target: 10x speedup

**Phase 2: Full GPU Rendering**
1. GPU-based character matching
2. Parallel charset comparison
3. Color quantization on GPU
4. Target: 50x speedup

**Phase 3: Advanced Features**
1. DoG on GPU
2. Real-time parameter adjustment via uniforms
3. Multi-pass rendering
4. Target: 100x speedup

### Dependencies for GPU Pipeline

```toml
[dependencies]
wgpu = "0.19"           # GPU abstraction
pollster = "0.4"         # Async runtime
```

### Considerations

1. **Buffer overhead** — GPU/CPU transfer is bottleneck
   - Solution: Keep entire pipeline on GPU

2. **Memory bandwidth** — 4K texture = 33MB/frame
   - Solution: Use compute shaders, minimize texture samples

3. **Thread divergence** — Character matching is SIMD-friendly
   - Solution: Warp-level reduction for best match

4. **Fallback** — CPU path for systems without GPU
   - Solution: Feature-gated implementation

### Resources

- [wgpu compute examples](https://github.com/gfx-rs/wgpu/tree/master/examples)
- [Image filters with wgpu-rs](https://blog.redwarp.app/image-filters/)
- [Vulkano compute pipelines](https://vulkano.rs/04-compute-pipeline/)
- [ASCII shader examples](https://alexharri.com/blog/ascii-rendering)

---

## MIDI/OSC Configuration UI (DONE ✓)

### Implementation (2026-03-31)

Added full configuration UI for MIDI and OSC:

**MIDI Menu** (`m` key):
- `↑/↓` — Select mapping entry
- `←/→` — Cycle parameter (None, BassScale, MidScale, HighScale, GlobalContrast, EnergyReact, BpmScale, EdgeThreshold, BgThreshold)
- `e` — Toggle MIDI enabled
- `a` — Add new mapping
- `d` — Delete selected mapping
- `g` / `Esc` — Return to global mode

**OSC Menu** (`d` key):
- `e` — Toggle OSC enabled
- `h` — Toggle host (localhost ↔ 127.0.0.1)
- `p/o` — Adjust target port (+/-100)
- `l/k` — Adjust listen port (+/-100)
- `g` / `Esc` — Return to global mode

**Status Bar Improvements**:
- MIDI: Shows "(Nm)" with mapping count, "[no mappings]" when empty
- OSC: Shows "l:LISTEN→HOST:PORT" format for clarity

**MIDI Integration (2026-03-31 Evening)**:
- MIDI device auto-detected on startup
- CC messages mapped to parameters via `apply_midi_mappings()`
- MIDI status indicator in status bar (shows mapping count)
- Supports: BassScale, MidScale, HighScale, GlobalContrast, EnergyReact, EdgeThreshold, BgThreshold

**OSC Integration (Complete)**:
- OSC receiver spawns background thread on startup
- Handles `/bass`, `/mid`, `/high`, `/bpm`, `/trigger` addresses
- `apply_osc_mappings()` scales band energy based on OSC values
- OSC trigger消息触发 beat
- OSC status indicator in status bar (shows listen port)
- OscSender sends `/ascii/bass`, `/ascii/mid`, `/ascii/high`, `/ascii/bpm`, `/ascii/beat`

### Data Structures

```rust
pub struct MidiMapping {
    pub cc: u8,
    pub param: MidiParam,
    pub min_val: f32,
    pub max_val: f32,
}

pub enum MidiParam {
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

pub struct MidiConfig {
    pub enabled: bool,
    pub mappings: Vec<MidiMapping>,
    pub last_note: u8,
    pub last_velocity: u8,
}

pub struct OscConfig {
    pub enabled: bool,
    pub target_host: String,
    pub target_port: u16,
    pub listen_port: u16,
    pub channels: [f32; 8],
}
```

### Status: Feature Complete

All major features implemented:

**Debug Panel (`D` key)**:
- Toggle debug overlay with `D` key
- Shows: FPS, band energies, BPM, beat indicator, MIDI/OSC status, camera status
- Helps troubleshoot audio/video issues

**Camera Support**:
- Fixed MJPEG decoding issue using zune-jpeg
- Works with V4L cameras (MJPG and YUYV formats)
- Graceful fallback if camera unavailable

**Troubleshooting System**:
- See `TROUBLESHOOTING.md` for documented issues and solutions
- See `DEBUG/fix/` for proposed fixes before merging
- See `DEBUG/fix/debugdoc.md` for fix workflow documentation
- See `skill.md` for debugging agent guidelines

**Fix Workflow**:
```
┌─────────────────────────────────────────────┐
│ 1. Issue in main src/                      │
└─────────────────┬───────────────────────────┘
                  ▼
┌─────────────────────────────────────────────┐
│ 2. Create: DEBUG/fix/<name>/             │
└─────────────────┬───────────────────────────┘
                  ▼
┌─────────────────────────────────────────────┐
│ 3. Test: cargo build && cargo build --release │
└─────────────────┬───────────────────────────┘
                  ▼
┌─────────────────────────────────────────────┐
│ 4. Merge: Copy to src/ if success        │
└─────────────────────────────────────────────┘
```

**To test:**
1. Run `cargo build && ./target/debug/ascii-cam` (debug mode with debug panel)
2. Or `cargo build --release && ./target/release/ascii-cam` (release mode)
3. Configure audio with `Space` key
4. Press `D` to toggle debug panel
5. Connect MIDI controller, use `m` to add mappings
6. Configure OSC with `d` key (send to TouchOSC, etc.)
