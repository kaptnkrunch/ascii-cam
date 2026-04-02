# Fix Index

*Directory for proposed fixes before merging into main codebase*

---

## Purpose

This directory contains proposed fixes that have been tested in the `DEBUG/` environment but not yet merged into the main `src/` directory.

Each fix should be in its own subdirectory with:
- `fix.md` - Description and documentation
- `src/` - Modified source files
- `test.sh` (optional) - Test script

---

## Quick Start

```bash
# Create a new fix
cd ascii-cam/DEBUG/fix
mkdir -p my-fix-name

# Implement fix in my-fix-name/src/

# Test the fix
cd ../..
./debug.sh build
./debug.sh run

# If successful, merge:
cp -r fix/my-fix-name/src/* src/
```

---

## Current Fixes

### Pending Fixes

| Directory | Issue | Status | Date |
|-----------|-------|--------|------|
| (none) | - | - | - |

### Applied Fixes

| Issue | Fix Directory | Applied Date |
|-------|---------------|--------------|
| Camera MJPEG Decode | (integrated in main) | 2026-03-31 |

---

## Creating a New Fix

1. Create fix directory:
   ```bash
   mkdir -p fix/<fix-name>
   ```

2. Create `fix.md`:
   ```markdown
   # Fix: <Title>
   
   ## Issue
   ...
   
   ## Solution
   ...
   
   ## Testing
   ...
   ```

3. Copy and modify source files:
   ```bash
   cp src/main.rs fix/<fix-name>/src/
   # Edit the copied files
   ```

4. Test:
   ```bash
   cd ../..
   cp -r fix/<fix-name>/src/* src/
   ./debug.sh build
   ./debug.sh run
   ```

5. If successful, merge to main:
   ```bash
   cd ..
   cp -r fix/<fix-name>/src/* src/
   ```

---

## See Also

- `debugdoc.md` - Full workflow documentation
- `TROUBLESHOOTING.md` - Known issues and solutions
- `skill.md` - Debugging agent guidelines

---

*Last Updated: 2026-03-31*
