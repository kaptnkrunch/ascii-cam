# Bug Index Template

*For tracking bugs with exact code positions*

---

## Format

```markdown
## [BUG_ID]: [Brief Title]

**Severity**: Critical / High / Medium / Low
**Status**: Open / In Progress / Fixed / WontFix

### Location
- **File**: `src/path/to/file.rs`
- **Line**: [LINE_NUMBER]
- **Column**: [COLUMN_NUMBER]
- **Function**: `function_name()`
- **Module**: `module::path`

### Description
[Detailed description of the bug]

### Error Message
```
[Exact error message from compiler or runtime]
```

### Root Cause
[What causes this bug]

### Fix Applied
- **Date**: YYYY-MM-DD
- **By**: Agent name
- **Commit**: [commit hash if applicable]
- **Description**: [how it was fixed]

### Related
- Related issues: #1, #2
- Related files: `src/other.rs`
```

---

## Example Entries

### BUG-001: MIDI Menu Crash

**Severity**: Critical
**Status**: Fixed

#### Location
- **File**: `src/midi.rs`
- **Line**: 87
- **Column**: 12
- **Function**: `enumerate_devices()`
- **Module**: `midi`

#### Description
Application crashes when entering MIDI options menu because `devices.iter()` is called on a `None` value when no MIDI devices are available.

#### Error Message
```
thread 'main' panicked at src/midi.rs:87:12
called `Option::unwrap()` on a `None` value
```

#### Root Cause
Missing null check before iterating over devices array.

#### Fix Applied
- **Date**: 2026-04-12
- **By**: BugHunter
- Added `if let Some(devices) = &self.devices` check before iteration

---

### BUG-002: Missing g Key in Submenus

**Severity**: Medium
**Status**: Open

#### Location
- **File**: `src/main.rs`
- **Line**: 342
- **Function**: `handle_menu_input()`

#### Description
Pressing 'g' in MIDI, OSC, Camera, or Audio submenus does not return to global menu. Only works in main menu.

#### Related
- Related to USERFINDINGS item #1

---

## Index Statistics

| Severity | Count | Fixed | Open |
|----------|-------|-------|------|
| Critical | 1 | 1 | 0 |
| High | 0 | 0 | 0 |
| Medium | 1 | 0 | 1 |
| Low | 0 | 0 | 0 |
| **Total** | **2** | **1** | **1** |

---

*Last Updated: 2026-04-12*