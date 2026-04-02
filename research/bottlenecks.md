# Detailed Bottleneck Analysis

## Hot Path Analysis

The main loop in `main.rs` executes these steps per frame:

```
1. Lock audio state (Mutex)
2. Lock MIDI state (Mutex)
3. Lock OSC state (Mutex)
4. Get camera frame (blocking)
5. Decode frame to RGB (MJPEG → RGB)
6. Convert to grayscale
7. Resize image (image::imageops::resize)
8. Layer detection:
   a. Convert pixels to f32
   b. Sobel magnitude (edge detection)
   c. Laplacian variance (fine detail)
   d. Laplacian variance (coarse detail)
   e. Classify pixels into layers
9. Build band lookup table
10. Calculate band contrasts
11. For each pixel:
    a. Calculate luma with contrast
    b. Lookup band
    c. Apply band mode (add/sub/mul/div/xor/xand)
    d. Calculate detail adjustment
    e. Select character
    f. Select color
12. Render to terminal (crossterm)
13. Flush stdout
```

Steps 7-11 are the CPU-intensive part. Let's analyze each.

---

## 1. Image Resize (Step 7)

**Location**: `main.rs:586`

```rust
let resized = image::imageops::resize(gray, ascii_cols, ascii_rows, FilterType::Nearest);
```

**Analysis**:
- `image` crate uses optimized resize but `Nearest` is already fast
- Downscale from 1920x1080 to ~120x40 is 45x reduction
- **Not a bottleneck** - this is O(n) simple copy with nearest-neighbor

---

## 2. Layer Detection (Step 8)

### 2.1 Pixel Conversion

**Location**: `layers.rs:50`

```rust
let pixels: Vec<f32> = gray.pixels().map(|Luma([v])| *v as f32 / 255.0).collect();
```

**Issue**: Creates new `Vec<f32>` every frame. For 120x40 = 4800 pixels, that's ~19KB.

**Bottleneck severity**: MEDIUM

### 2.2 Sobel Magnitude

**Location**: `layers.rs:82-95`

```rust
fn sobel_magnitude(pixels: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let p = |dy: isize, dx: isize| -> f32 { /* ... */ };
            let gx = /* ... */;
            let gy = /* ... */;
            out[y * w + x] = (gx.abs() + gy.abs()) / 4.0;
        }
    }
    out
}
```

**Issues**:
- Sequential iteration (no SIMD)
- Boundary checks inside loop
- New allocation every call

**Bottleneck severity**: HIGH

### 2.3 Laplacian Variance (twice)

**Location**: `layers.rs:97-120`

```rust
fn laplacian_variance(pixels: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let r = radius as isize;
    
    for y in r..(h as isize - r) {
        for x in r..(w as isize - r) {
            let center = pixels[(y as usize) * w + (x as usize)];
            let mut sum = 0.0f32;
            let mut count = 0.0f32;
            
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx != 0 || dy != 0 {
                        let neighbor = pixels[(y + dy) as usize * w + (x + dx) as usize];
                        sum += (center - neighbor).powi(2);
                        count += 1.0;
                    }
                }
            }
            out[(y as usize) * w + (x as usize)] = (sum / count).sqrt();
        }
    }
    out
}
```

**Issues**:
- Called TWICE per frame (radius 1 and 2)
- O(r²) per pixel (r=1: 8 neighbors, r=2: 24 neighbors)
- `powi(2)` slower than `*`
- No SIMD

**Bottleneck severity**: HIGH

---

## 3. Band Lookup (Step 9)

**Location**: `main.rs:603-628`

```rust
let band_lookup: [Option<usize>; 6] = [
    state.bands.iter().position(|b| b.layer == VisualLayer::Black),
    state.bands.iter().position(|b| b.layer == VisualLayer::Background),
    state.bands.iter().position(|b| b.layer == VisualLayer::Foreground),
    state.bands.iter().position(|b| b.layer == VisualLayer::Edge),
    state.bands.iter().position(|b| b.layer == VisualLayer::White),
    state.bands.iter().position(|b| b.layer == VisualLayer::IrEnhanced),
];
```

**Issues**:
- `iter().position()` is O(n) - but n ≤ 6, so max 36 iterations
- Called every frame but bands rarely change

**Bottleneck severity**: LOW (but easy to fix)

---

## 4. Per-Pixel Rendering (Step 11)

**Location**: `main.rs:651-819`

This is the innermost loop. Let's trace through what happens per-pixel:

```rust
for row in 0..h {
    for col in 0..w {
        // 1. Get luma
        let luma_raw = resized.get_pixel(col as u32, row as u32).0[0] as f32 / 255.0;
        
        // 2. Apply global contrast
        let luma_contrasted = ((luma_raw - 0.5) * state.global_contrast + 0.5).clamp(0.0, 1.0);
        
        // 3. Check invert
        let luma = if state.inverted { 1.0 - luma_contrasted } else { luma_contrasted };
        
        // 4. Get layer
        let pixel_layer = &layer_map[row * w + col];
        
        // 5. Determine target visual layer (complex logic)
        let target_visual = /* 20+ lines of logic */;
        
        // 6. Band lookup (already done)
        let band_idx = match target_visual { /* ... */ };
        
        // 7. Band energy calculation
        let e = if band.muted { 0.0 } else { /* calculation */ };
        
        // 8. Mode application (6 different modes)
        let eff_luma = match band.mode {
            BandMode::Additive => (luma + e * 0.5).clamp(0.0, 1.0),
            // ... 5 more modes
        };
        
        // 9. Detail calculation
        let detail = detail_map[row * w + col];
        let fine = fine_detail[row * w + col];
        let coarse = coarse_detail[row * w + col];
        
        // 10. Character selection
        let adjusted_luma = (eff_luma + detail_adjust).min(1.0);
        let char_idx = ((adjusted_luma * (local_len - 1) as f32).round() as usize)
            .min(local_len - 1);
        let ch = charset_subset[char_idx];
        
        // 11. Color calculation
        let color = base_col.modulate(color_modulation);
        
        // 12. Build output
        line.push(PixelOut { ch, color });
    }
}
```

**Issues**:
- ~40 operations per pixel
- `get_pixel()` involves bounds checking (could use `get_pixelunchecked`)
- Multiple array index calculations
- Character slice bounds checking

**Bottleneck severity**: MEDIUM-HIGH (but hard to optimize further)

---

## 5. Terminal Output (Step 12-13)

**Location**: `main.rs:2098-2118`

```rust
queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

for (row_idx, pixels) in pixel_rows.iter().enumerate() {
    queue!(stdout, cursor::MoveTo(0, row_idx as u16))?;
    let mut last_color: Option<Color> = None;
    for px in pixels {
        if last_color != Some(px.color) {
            queue!(stdout, SetForegroundColor(px.color))?;
            last_color = Some(px.color);
        }
        
        // STRING BUILDING - BOTTLENECK
        let ch_str: String = std::iter::repeat(px.ch).take(scale as usize).collect();
        queue!(stdout, Print(ch_str))?;
    }
    queue!(stdout, ResetColor)?;
}

stdout.flush()?;
```

**Issues**:
- `Clear(ClearType::All)` clears entire screen every frame (expensive)
- New String allocation for every character when scale > 1
- Too many `queue!` calls (could batch)

**Bottleneck severity**: HIGH for string building

---

## Summary Table

| Component | Severity | Type | Fix Difficulty |
|-----------|----------|------|----------------|
| Laplacian variance | HIGH | CPU/SIMD | Medium |
| Sobel magnitude | HIGH | CPU/SIMD | Medium |
| Pixel conversion | MEDIUM | Allocation | Low |
| String building | HIGH | Allocation | Low |
| Band lookup | LOW | Algorithm | Low |
| Terminal clear | MEDIUM | I/O | Low |
| Per-pixel loop | MEDIUM-HIGH | Algorithm | Hard |

---

## Recommendations

1. **Immediate**: Fix string building in render loop (5-10% speedup)
2. **Quick**: Pre-allocate pixel buffers (10-15% speedup)
3. **Medium**: Implement frame skipping (variable improvement)
4. **Long-term**: SIMD for Sobel/Laplacian (50%+ speedup potential)

---

*Generated: 2026-04-01*
