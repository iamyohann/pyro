#!/bin/sh

set -e

# Detect if cargo is installed
if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo is not installed. Please install Rust and Cargo to proceed."
    echo "Visit https://rustup.rs/ for installation instructions."
    exit 1
fi

echo "Building Pyro..."
cargo build --release

INSTALL_DIR="$HOME/.pyro/bin"
mkdir -p "$INSTALL_DIR"

echo "Installing to $INSTALL_DIR..."
cp target/release/pyro-cli "$INSTALL_DIR/pyro"

echo "Setting up Pyro Runner environment..."
RUSTPKG_DIR="$HOME/.pyro/rustpkg/current"
mkdir -p "$RUSTPKG_DIR/src"

# Get absolute path to pyro-core
PYRO_CORE_PATH="$(pwd)/pyro-core"

# Generate Cargo.toml for runner
cat > "$RUSTPKG_DIR/Cargo.toml" <<EOF
[package]
name = "pyro_runner"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
pyro-core = { path = "$PYRO_CORE_PATH" }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
EOF

# Generate minimal main.rs to allow build
cat > "$RUSTPKG_DIR/src/main.rs" <<EOF
fn main() {}
EOF

echo "Pre-building Pyro Runner..."
(cd "$RUSTPKG_DIR" && cargo build --release)


echo ""
echo "Pyro has been successfully installed!"
echo ""
echo "To use 'pyro' command, ensure '$INSTALL_DIR' is in your PATH."
echo "You can add the following line to your shell configuration file (e.g., ~/.zshrc or ~/.bashrc):"
echo ""
echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
echo ""
