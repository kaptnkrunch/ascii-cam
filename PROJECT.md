# ascii-cam Project

Terminal-based ASCII audio visualizer with camera input

## Project Overview

| Field | Value |
|-------|-------|
| **Name** | ascii-cam |
| **Type** | Terminal-based audio visualizer |
| **Language** | Rust |
| **Target** | Linux/macOS/Windows |
| **Design Philosophy** | Real-time, low-latency, cross-platform terminal rendering |
| **Target Hardware** | Laptop with webcam + speakers |
| **Target Performance** | 30+ FPS at 80x24 (standard terminal) |

## Design Principles

1. **Real-time** - Sub-100ms audio-to-visual latency
2. **Terminal-native** - Pure text output, no GUI dependencies
3. **Audio-reactive** - BPM detection, frequency analysis
4. **Cross-platform** - Linux (ALSA/PipeWire), macOS (CoreAudio), Windows (WASAPI)
5. **Feature-rich** - Multiple character sets, layers, effects

## Tasks

| # | Status | Description |
|---|--------|-------------|
| 1 | [ ] | Fix MIDI menu crash (try-catch + null checks) |
| 2 | [ ] | Add `g` key to all sub-menus for global return |
| 3 | [ ] | Implement BPM counter stability (80%+ accuracy) |
| 4 | [ ] | Add quit confirmation prompt |
| 5 | [ ] | Create separate control window |
| 6 | [ ] | Implement MIDI interface |
| 7 | [ ] | Implement OSC protocol |
| 8 | [ ] | Add hardware access (IR channel, focus, exposure) |
| 9 | [ ] | Implement rendering layers (multiplication, division, XOR, XAND) |
| 10 | [ ] | Windows cross-platform build |

## Progress
- Completed: 0/10

---

## Recent Work

- macOS x86_64 CI migration complete
- Audio architecture implemented with CPAL + rustfft
- Character sets: Latin, Cyrillic, Hiragana, Katakana, Arabic, Braille

---

## Research

- `research/` - Technical research documents

---

## Implementation Phases

### Phase 1: Bug Fixes & UI
- Fix MIDI crash
- Add `g` key to all menus
- Quit confirmation

### Phase 2: BPM Stability
- Beat history buffer expansion (16→32)
- Confidence threshold filtering
- Adaptive tempo smoothing

### Phase 3: External Interfaces
- MIDI input
- OSC protocol
- OSC over MIDI

### Phase 4: Advanced Rendering
- Multi-layer rendering
- Additional blend modes
- IR channel as layer

### Phase 5: Cross-Platform
- Windows build
- Dependencies resolution

---

## First Feature Demo

- **Planned:** ASCII audio visualizer with frequency bars
- **Target:** Terminal 80x24, 30 FPS
- **Audio:** System loopback via PipeWire/PulseAudio

---

*Last Updated: 2026-04-12*