# Debugging Agent Skill: ascii-cam

*For use with AI debugging assistants*

---

## Project Overview

ascii-cam is a Rust application that renders ASCII video from camera input with audio reactivity, MIDI/OSC support, and real-time visualization.

**Key Technologies:**
- Rust (stable)
- nokhwa (camera)
- crossterm (terminal UI)
- cpal (audio)
- midir (MIDI)
- rosc (OSC)
- zune-jpeg (MJPEG decode)

---

## Build Commands

```bash
# Debug build (includes debug panel by default)
cargo build

# Release build
cargo build --release

# Run debug
./target/debug/ascii-cam

# Run release
./target/release/ascii-cam
```

---

## Known Issues & Solutions

### 1. Release Build Resize Crash
**Symptom:** Release build crashes during terminal resize, debug build works fine.

**Investigation Steps:**
1. Capture stderr: `./target/release/ascii-cam 2>&1 | tee crash.log`
2. Resize terminal, check crash.log
3. Test with `opt-level = 2` in Cargo.toml
4. Add terminal size guards: `term_cols.max(10)`, `term_rows.max(5)`

**Possible Causes:**
- Release optimization removing bounds checks
- SIGWINCH signal handling issues
- Thread safety during resize
- crossterm internal state corruption

### 2. Camera MJPEG Decode Error
**Symptom:** `Buffer mismatch: 1280x720` / `Decode error: Not available on WASM`

**Root Cause:** nokhwa V4L backend fails to decompress MJPEG frames.

**Solution:** Uses zune-jpeg fallback for manual MJPEG decoding.
```rust
let rgb = match frame.decode_image::<RgbFormat>() {
    Ok(d) => d,
    Err(_) => {
        // zune-jpeg fallback
        let mut decoder = zune_jpeg::JpegDecoder::new(Cursor::new(raw_buf.as_ref()));
        decoder.decode().unwrap()
    }
};
```

### 3. Camera Not Detected
**Symptom:** `No camera found` / `Camera stream error`

**Debug Commands:**
```bash
v4l2-ctl --list-formats -d /dev/video0
v4l2-ctl --list-framesizes=MJPG -d /dev/video0
```

**Solutions:**
1. Check camera index (try 1, 2, etc.)
2. Check camera permissions: `sudo chmod 666 /dev/video0`
3. Check if camera is used by another process: `lsof /dev/video0`

### 4. Audio Device Issues
**Symptom:** No audio input / device errors

**Debug Commands:**
```bash
# List audio devices
pactl list sources short
arecord -l

# Test audio
arecord -d 5 -f cd test.wav
```

**Solutions:**
1. Press `Space` to open device menu
2. Check PulseAudio/PipeWire is running
3. Enable loopback device for system audio capture

### 5. MIDI Not Detected
**Symptom:** `No MIDI devices found`

**Debug Commands:**
```bash
# List MIDI devices
amidi -l
aconnect -l
```

---

## Debug Mode Features

Press `D` to toggle debug panel:
- FPS counter
- Band energies (Bass/Mid/High)
- BPM with confidence
- Beat indicator
- MIDI/OSC status
- Camera status

Debug build enables this by default:
```rust
debug_mode: cfg!(debug_assertions),  // true in debug, false in release
```

---

## Investigation Framework

When debugging a new issue:

### Step 1: Reproduce
- Get exact error message
- Note build type (debug/release)
- Note terminal/environment
- Identify if issue is reproducible

### Step 2: Isolate
- Test minimal case
- Disable features one by one
- Test in different environments

### Step 3: Research
- Check TROUBLESHOOTING.md for known issues
- Search for similar issues in project history
- Check dependency changelogs

### Step 4: Fix
- Make minimal change
- Test both debug and release builds
- Document in TROUBLESHOOTING.md

### Step 5: Verify
- Confirm fix works
- Check for regressions
- Update documentation

---

## Code Patterns

### Error Handling
```rust
// Prefer Result types
let result = some_operation()?;
eprintln!("Operation failed: {}", e);

// Fallback patterns
let value = match primary() {
    Ok(v) => v,
    Err(_) => fallback(),
};
```

### Terminal Operations
```rust
use crossterm::{execute, terminal};

// Always restore terminal on exit
let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
let _ = terminal::disable_raw_mode();
```

### Thread-Safe State
```rust
type SharedState = Arc<Mutex<State>>;

let shared: SharedState = Arc::new(Mutex::new(State::default()));
let shared_clone = shared.clone();
thread::spawn(move || {
    let mut state = shared_clone.lock().unwrap();
    // modify state
});
```

---

## File Structure

```
ascii-cam/
├── src/
│   ├── main.rs      # Main application (UI, render loop)
│   ├── audio.rs     # Audio capture and FFT
│   ├── charset.rs   # Character set definitions
│   ├── ir.rs        # IR/depth camera support
│   ├── layers.rs    # Edge detection, layer classification
│   ├── midi.rs      # MIDI input handling
│   └── osc.rs       # OSC protocol (send/receive)
├── Cargo.toml       # Dependencies
├── README.md        # User documentation
├── TROUBLESHOOTING.md  # Known issues
└── skill.md         # This file
```

---

## Testing Checklist

Before marking issue as resolved:
- [ ] Debug build works
- [ ] Release build works
- [ ] No regression in other features
- [ ] Documentation updated
- [ ] TROUBLESHOOTING.md updated if new issue

---

## Quick Reference

| Issue | Quick Fix |
|-------|-----------|
| Release crash | Add guards, try opt-level = 2 |
| Camera error | Check v4l2-ctl, permissions |
| Audio no work | Check PulseAudio, loopback |
| MIDI not found | Check alsa/portaudio |
| Terminal glitch | Try different terminal (alacritty) |

---

## Common Environment Issues

### Wayland
Some terminals may have issues with alternate screen mode. Try:
- XWayland mode
- Different terminal (kitty, alacritty)
- X11 session

### SSH
Must have TTY allocated:
```bash
ssh host -t "ascii-cam"
```

### Container
May need:
- `--device /dev/video0`
- `--privileged`
- Audio device access

---

*Last Updated: 2026-03-31*
