# Troubleshooting: ascii-cam Issues

*Last Updated: 2026-03-31*

---

## Table of Contents

1. [Release Build Resize Crash](#1-release-build-resize-crash)
2. [Camera MJPEG Decode Error](#2-camera-mjpeg-decode-error)

---

## 1. Release Build Resize Crash

*Status: Under Investigation*

### Symptoms

- **Debug build**: Works fine during terminal resize
- **Release build**: Crashes immediately when resizing (scale up or down)
- Crash occurs during resize event, not from initial values
- Crash happens regardless of debug panel state (`D` key)

### Behavior

| Build Type | Initial Render | Resize Up | Resize Down |
|------------|----------------|-----------|--------------|
| Debug | ✅ Works | ✅ Works | ✅ Works |
| Release | ✅ Works | ❌ Crashes | ❌ Crashes |

### Possible Causes

#### 1. Release Optimization Behavior
Release mode (`opt-level = 3`) has aggressive optimizations that can:
- Reorder memory operations
- Eliminate bounds checks
- Inline functions aggressively
- Change timing of concurrent operations

#### 2. Integer Overflow/Division Issues
```rust
// Potential issue in render loop
let ascii_cols = (term_cols as u32 / scale).max(1);
let ascii_rows = (term_rows.saturating_sub(4) as u32 / scale).max(1);
```
If `scale` becomes 0 or causes overflow during resize calculations.

#### 3. SIGWINCH Signal Handling
Terminal resize sends `SIGWINCH` signals. Release mode might handle signals differently:
- Signal handlers could be optimized away
- Signal handling might race with render thread
- crossterm's internal state could become corrupted during resize

#### 4. Thread Safety Issue
Race condition during resize:
```rust
// Potential race: read terminal size while state changes
let (term_cols, term_rows) = terminal::size()?;
// ← Audio/MIDI thread might access terminal here
render_frame(&gray, ascii_cols, ascii_rows, ...);
```

#### 5. crossterm Internal State
During resize, crossterm's alternate screen and cursor state can become inconsistent:
- Debug: Properly handles state transitions
- Release: State corruption during fast resize events

#### 6. `debug_mode` Conditional Code
```rust
// Debug: true, Release: false
debug_mode: cfg!(debug_assertions),
```
Different code paths might expose latent bugs in release mode.

### Investigation Commands

```bash
# Capture stderr to log file
./target/release/ascii-cam 2>&1 | tee resize_crash.log
# Resize terminal multiple times
# Ctrl+C to exit
# Check: cat resize_crash.log

# Or redirect to file
./target/release/ascii-cam > output.log 2>&1
# Resize, then Ctrl+C
# cat output.log
```

### Proposed Solutions

#### Solution A: Add Terminal Size Guards
```rust
let term_cols = term_cols.max(10);  // Minimum width
let term_rows = term_rows.max(5);   // Minimum height
let scale = app_state.char_size.max(1);
```

#### Solution B: Skip Frames During Resize
```rust
use std::sync::atomic::{AtomicBool, Ordering};

static RESIZE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

// When resize detected
RESIZE_IN_PROGRESS.store(true, Ordering::Relaxed);
std::thread::sleep(Duration::from_millis(50));
continue; // Skip frame during resize

// After render complete
RESIZE_IN_PROGRESS.store(false, Ordering::Relaxed);
```

#### Solution C: Add Resize Event Debouncing
```rust
let mut last_resize_time = std::time::Instant::now();
let (term_cols, term_rows) = terminal::size()?;
let now = std::time::Instant::now();

if now.duration_since(last_resize_time) < Duration::from_millis(100) {
    // Too soon since last resize, skip frame
    continue;
}
last_resize_time = now;
```

#### Solution D: Catch Unwind in Release
```rust
use std::panic::catch_unwind;

std::panic::set_hook(Box::new(|info| {
    eprintln!("[PANIC] {}", info);
}));

let result = catch_unwind(|| {
    render_frame(...)
});

if result.is_err() {
    eprintln!("[ERROR] Render panic during resize!");
    continue;
}
```

#### Solution E: Test with Different Optimization Level
```toml
# Cargo.toml - try opt-level = 2 for release
[profile.release]
opt-level = 2
```

### Debug Build vs Release Differences

```toml
# Cargo.toml
[profile.release]
opt-level = 3  # Aggressive optimizations

# Code difference
debug_mode: cfg!(debug_assertions),  # true in debug, false in release
```

### Related Issues

- crossterm signal handling
- nokhwa threaded output mode during resize
- Release mode undefined behavior

### Status: Not Yet Fixed

Need additional testing to identify exact crash point.

---

## 2. Camera MJPEG Decode Error

*(RESOLVED)*

---

## Root Cause Analysis

### Problem
The webcam returns frames in **MJPG (Motion-JPEG)** compressed format, but nokhwa's `decode_image::<RgbFormat>()` fails to decompress it, returning a WASM-related error message.

### Camera Capabilities
```
v4l2-ctl --list-formats
Type: Video Capture
    [0]: 'MJPG' (Motion-JPEG, compressed)
    [1]: 'YUYV' (YUYV 4:2:2)
```

Available resolutions for MJPG: 1280x720, 800x600, 640x480, etc.

### Why It Happens
- nokhwa's V4L backend doesn't properly handle MJPEG decompression
- The WASM error message is misleading - it actually indicates a missing implementation in the V4L backend
- Buffer size doesn't match RGB expectations (1280x720 × 3 bytes = 2,764,800 bytes needed, but MJPG is compressed)

---

## Solution Implemented

### Manual MJPEG Decoding with zune-jpeg

Added `zune-jpeg` crate for pure-Rust JPEG decompression:

**Cargo.toml:**
```toml
zune-jpeg = "0.5"
```

**main.rs code:**
```rust
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
            Ok(rgb_data) => {
                match image::RgbImage::from_raw(width, height, rgb_data) {
                    Some(img) => img,
                    None => {
                        eprintln!("Buffer size mismatch: {}x{}", width, height);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("JPEG decode error: {}", e);
                continue;
            }
        }
    }
};
```

---

## Alternative Solutions Considered

### 1. Request YUYV Format
Request uncompressed YUYV format instead of MJPEG:
```rust
use nokhwa::utils::{CameraFormat, RequestedFormat, RequestedFormatType, YuyvFormat};
let format = RequestedFormat::new::<YuyvFormat>(
    RequestedFormatType::Exact(CameraFormat::new(640, 480, 30, YuyvFormat::default()))
);
```
**Pros**: No extra dependencies, faster decoding
**Cons**: Higher bandwidth (640×480×2 bytes vs compressed MJPEG), may fail on some cameras

### 2. Use image crate's JPEG decoder
```rust
use image::io::Reader as ImageReader;
let img = ImageReader::new(Cursor::new(raw_buf))
    .with_guessed_format().unwrap()
    .decode().unwrap();
```
**Pros**: Uses existing `image` dependency
**Cons**: Less efficient than zune-jpeg

### 3. Use v4l2-rs directly
```toml
v4l2-rs = "0.3"
```
**Pros**: Full control over camera format
**Cons**: Linux-only, significant code changes

---

## Debug Commands

```bash
# Check camera formats supported
v4l2-ctl --list-formats -d /dev/video0

# Check frame sizes for MJPG
v4l2-ctl --list-framesizes=MJPG -d /dev/video0

# Check current settings
v4l2-ctl -d /dev/video0 --all

# Force YUYV format (for testing alternatives)
v4l2-ctl -d /dev/video0 --set-fmt-video=width=640,height=480,pixelformat=YUYV
```

---

## Related Dependencies

| Crate | Purpose |
|-------|---------|
| `nokhwa` | Cross-platform camera access |
| `nokhwa-bindings-linux` | V4L backend |
| `zune-jpeg` | Pure-Rust JPEG decoder (chosen solution) |
| `image` | Image processing and buffer conversion |

---

## Related nokhwa Issues

- [#100](https://github.com/l1npengtul/nokhwa/issues/100): Unable to get camera frame on macbook
- [#146](https://github.com/l1npengtul/nokhwa/issues/146): YUV422 format handling confusion
- [#167](https://github.com/l1npengtul/nokhwa/issues/167): BGR format support request

---

## Future Improvements

1. **Format negotiation**: Try multiple formats (MJPEG → YUYV → fallback)
2. **Resolution selection**: Allow user to select camera resolution
3. **Frame rate control**: Set desired frame rate explicitly
4. **Error recovery**: Automatically retry with different format on decode failure
