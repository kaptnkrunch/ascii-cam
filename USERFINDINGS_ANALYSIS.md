# USERFINDINGS Implementation Plan

## Overview
This document structures user findings from USERFINDINGS.md into actionable solutions and implementations.

---

## Priority 1: Critical Bugs & Crashes

### 1.1 MIDI Menu Crash
**Finding**: Entering MIDI options crashes the application
**Solution**: Add null checks and error handling for MIDI device initialization
**Implementation**:
- [ ] Wrap MIDI initialization in try-catch
- [ ] Add graceful fallback when no MIDI device available
- [ ] Add device availability check before opening menu

### 1.2 Global Return Key (`g`)
**Finding**: Sub-menus don't enable `g` for global return
**Solution**: Add `g` key handler to all sub-menus
**Implementation**:
- [ ] Add `g` key binding to MIDI menu
- [ ] Add `g` key binding to OSC menu  
- [ ] Add `g` key binding to Camera settings menu
- [ ] Add `g` key binding to Audio device menu

---

## Priority 2: Core Feature Improvements

### 2.1 BPM Counter Stability
**Finding**: BPM counter is unstable/inaccurate (needs 80% accuracy)
**Solution**: Implement weighted averaging with beat history tracking
**Implementation**:
- [ ] Increase beat history buffer size (16 → 32)
- [ ] Add confidence threshold filtering (reject < 60%)
- [ ] Implement adaptive tempo smoothing for gradual changes
- [ ] Add double-beat detection to filter false positives
- [ ] Weight recent beats more heavily than historical data

### 2.2 Quit Confirmation
**Finding**: `q` exits instantly without confirmation
**Solution**: Add confirmation dialog before exit
**Implementation**:
- [ ] Add `q` confirmation prompt in global mode
- [ ] Add countdown timer (auto-exit in 5 seconds if no input)
- [ ] Keep `Q` as instant quit alternative

---

## Priority 3: New Features

### 3.1 Separate GUI Window for Controls
**Finding**: Need video output handling in separate window with full-screen value display
**Solution**: Implement dual-window architecture
**Implementation**:
- [ ] Create separate control window (imGui or crossterm alternate terminal)
- [ ] Move all UI controls to separate window
- [ ] Keep terminal output purely for ASCII rendering
- [ ] Add full-screen mode for value displays
- [ ] Implement window communication protocol

---

## Implementation Notes

### For SVG Architecture Diagram
- Add "Quit Confirmation" block near terminal output
- Add "Window Manager" block between inputs and output sections

### For TODO.md
- Merge with existing TODO items
- Mark completed items
- Add new items from this analysis

---

*Generated: 2026-04-01*
