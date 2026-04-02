# macOS CI Target Migration - FINAL

## CI Matrix (All Latest)

| Platform | Runner | Target | Build Method |
|----------|--------|--------|--------------|
| Linux | ubuntu-latest | x86_64-unknown-linux-gnu | Native |
| macOS x86_64 | **macos-latest** (ARM host) | x86_64-apple-darwin | Cross-compile |
| macOS ARM | macos-latest | aarch64-apple-darwin | Native |
| Windows | windows-latest | x86_64-pc-windows-msvc | Native |

## Changes Made

- All platforms now use `-latest` runners
- macOS x86_64 builds on ARM host via Rust cross-compilation (automatic)

## Build Method Note

GitHub Actions' `macos-latest` is Apple Silicon (ARM). Building x86_64 targets works via Rust's cross-compilation - cargo handles this automatically with the target flag.

## Test Command

```bash
gh workflow run release.yml -f tag_name=v0.1.0
```

## Verification

- [x] release.yml updated to all -latest runners
- [ ] Run CI to verify
- [ ] Verify all 4 artifacts