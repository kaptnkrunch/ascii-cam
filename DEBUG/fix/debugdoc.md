# DEBUG FIX Workflow Documentation

*For debugging agents and developers*

---

## Workflow Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    ascii-cam Project                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────┐     ┌─────────┐     ┌─────────┐          │
│   │  src/   │ ←── │  DEBUG/ │ ←── │  fix/   │          │
│   │ (main)  │     │ (test)  │     │ (proposed)│          │
│   └─────────┘     └─────────┘     └─────────┘          │
│       ↑               ↑               ↑                   │
│       │               │               │                   │
│       │    ┌─────────┴───────────────┘                   │
│       │    │                                             │
│       │    ▼                                             │
│       │    ┌─────────────────────────────────┐           │
│       └─── │       Merge after testing       │           │
│            └─────────────────────────────────┘           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
ascii-cam/
├── src/              # Main source code (production)
├── DEBUG/            # Testing environment
│   ├── src/          # Copy of source for testing
│   ├── fix/          # Proposed fixes
│   │   ├── debugdoc.md    # This file
│   │   ├── <fix-name>/   # Individual fix directory
│   │   │   ├── fix.md     # Fix description
│   │   │   ├── src/      # Modified files
│   │   │   └── test.sh    # Test script
│   │   └── README.md       # Fix index
│   └── ...
└── ...
```

---

## Fix Workflow

### Step 1: Identify Issue
- Reproduce the issue in main codebase
- Document in `TROUBLESHOOTING.md`

### Step 2: Create Fix Directory
```bash
cd ascii-cam/DEBUG/fix
mkdir -p <fix-name>
# Example: mkdir -p resize-crash-fix
```

### Step 3: Document the Fix
Create `fix.md` in the fix directory:
```markdown
# Fix: <Title>

## Issue
Description of the problem.

## Root Cause
What causes the issue.

## Solution
How the fix addresses the problem.

## Files Modified
- `src/main.rs` - changes

## Testing
How to verify the fix works.
```

### Step 4: Implement and Test
```bash
cd ascii-cam/DEBUG
# Copy fix files to DEBUG/src/
cp -r fix/<fix-name>/src/* src/

# Build and test
./debug.sh build
./debug.sh run

# Test both builds
./debug.sh release
```

### Step 5: Verify Fix
- [ ] Debug build works
- [ ] Release build works
- [ ] No regressions
- [ ] Document any new issues

### Step 6: Merge to Main
```bash
cd ascii-cam
# Copy fixed files from DEBUG
cp DEBUG/src/*.rs src/
cp DEBUG/src/*.rs src/

# Update documentation
# Update TROUBLESHOOTING.md
```

---

## Fix Naming Convention

```
<type>-<short-description>-<date>

Examples:
├── resize-crash-fix-20260331/
├── camera-mjpeg-fix-20260330/
├── terminal-size-guard-20260329/
```

**Types:**
- `fix` - Bug fix
- `feat` - New feature
- `opt` - Optimization
- `refactor` - Code improvement

---

## Current Fixes

| Fix | Status | Description |
|-----|--------|-------------|
| (none yet) | - | - |

---

## Testing Requirements

Every fix MUST be tested against:

### Build Types
- [ ] `cargo build` (debug)
- [ ] `cargo build --release` (release with opt-level 3)

### Environments
- [ ] Linux
- [ ] Terminal resize test
- [ ] Camera input test
- [ ] Audio input test

### Regression Checklist
- [ ] No new warnings
- [ ] No performance regression
- [ ] No breaking changes to existing features

---

## Merge Criteria

A fix can be merged when:

1. ✅ Compiles without errors in both debug and release
2. ✅ Fixes the reported issue
3. ✅ No new warnings introduced
4. ✅ Documentation updated
5. ✅ Added to `TROUBLESHOOTING.md` if new issue discovered
6. ✅ Tested on actual hardware (not just CI)

---

## Rollback Procedure

If a fix causes problems:

```bash
# Revert to previous state
git checkout HEAD~1

# Or restore from backup
cp src/*.rs.bak src/
```

---

## Debugging Agent Guidelines

When working on fixes:

1. **Always work in DEBUG/fix/**
2. **Test thoroughly before merging**
3. **Document everything**
4. **Keep fixes minimal and focused**
5. **One fix per directory**

---

*Last Updated: 2026-03-31*
