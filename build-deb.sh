#!/bin/bash
set -e

echo "=== Razorshot .deb Builder ==="

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for system dependencies
MISSING=""
for pkg in libgtk-4-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev libglib2.0-dev libdbus-1-dev pkg-config; do
    if ! dpkg -s "$pkg" &> /dev/null 2>&1; then
        MISSING="$MISSING $pkg"
    fi
done

if [ -n "$MISSING" ]; then
    echo "Missing build dependencies:$MISSING"
    echo "Install them with:"
    echo "  sudo apt install$MISSING"
    exit 1
fi

# Install cargo-deb if not present
if ! command -v cargo-deb &> /dev/null; then
    echo "Installing cargo-deb..."
    cargo install cargo-deb
fi

# Build release binary
echo "Building release binary..."
cargo build --release

# Build .deb package
echo "Packaging .deb..."
cargo deb --no-build

# Find the generated .deb
DEB=$(ls -t target/debian/razorshot_*.deb 2>/dev/null | head -1)

if [ -n "$DEB" ]; then
    echo ""
    echo "=== Done! ==="
    echo "Package: $DEB"
    echo "Size: $(du -h "$DEB" | cut -f1)"
    echo ""
    echo "Install with:"
    echo "  sudo dpkg -i $DEB"
    echo "  sudo apt-get install -f  # fix any missing runtime deps"
else
    echo "Error: .deb file not found"
    exit 1
fi
