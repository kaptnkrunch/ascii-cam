#!/bin/bash
# DEBUG copy - for testing fixes
# Usage: ./debug.sh [build|run|release]

set -e

cd "$(dirname "$0")"

case "${1:-build}" in
    build)
        echo "Building debug version..."
        cargo build
        echo "Done. Run with: ./target/debug/ascii-cam-debug"
        ;;
    run)
        echo "Running debug version..."
        ./target/debug/ascii-cam-debug
        ;;
    release)
        echo "Building release version..."
        cargo build --release
        echo "Done. Run with: ./target/release/ascii-cam-debug"
        ;;
    test)
        echo "Building release with opt-level 2 (debug-like)..."
        # Temporarily modify opt-level
        sed -i 's/opt-level = 2/opt-level = 3/' Cargo.toml
        cargo build --release
        # Restore
        sed -i 's/opt-level = 3/opt-level = 2/' Cargo.toml
        echo "Done. Run with: ./target/release/ascii-cam-debug"
        ;;
    clean)
        echo "Cleaning build..."
        cargo clean
        ;;
    *)
        echo "Usage: $0 [build|run|release|test|clean]"
        echo ""
        echo "Commands:"
        echo "  build   - Build debug version (default)"
        echo "  run     - Build and run debug version"
        echo "  release - Build release version"
        echo "  test    - Build release with opt-level 3"
        echo "  clean   - Clean build artifacts"
        ;;
esac
