#!/bin/bash

# MTPScript Installation Script
# Run with: curl -o- https://raw.githubusercontent.com/anomalyco/opencode/main/install.sh | bash

set -e

echo "Installing MTPScript..."

# Check prerequisites
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is required but not installed."
    echo "Please install Rust from https://rustup.rs"
    exit 1
fi

if ! command -v gcc &> /dev/null; then
    echo "Error: GCC is required but not installed."
    echo "Please install build tools (e.g., 'apt install build-essential' on Ubuntu)."
    exit 1
fi

if ! command -v git &> /dev/null; then
    echo "Error: Git is required but not installed."
    exit 1
fi

# Clone repository if not already present
if [ ! -d "$HOME/.mtpscript" ]; then
    echo "Cloning MTPScript repository..."
    git clone https://github.com/mytechpassport/mtpscript.git "$HOME/.mtpscript"
else
    echo "MTPScript repository already exists, updating..."
    cd "$HOME/.mtpscript"
    git pull
fi

cd "$HOME/.mtpscript"

# Build Rust components
echo "Building Rust components..."
cargo build --release

# Build C runtime components
echo "Building C runtime components..."
make setup
make compile

# Add to PATH if not already
SHELL_RC=""
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_RC="$HOME/.bashrc"
else
    SHELL_RC="$HOME/.profile"
fi

if [ -n "$SHELL_RC" ] && ! grep -q "export PATH.*mtpscript" "$SHELL_RC"; then
    echo "Adding MTPScript to PATH in $SHELL_RC"
    echo "export PATH=\"\$HOME/.mtpscript/target/release:\$HOME/.mtpscript/build:\$PATH\"" >> "$SHELL_RC"
    echo "Please restart your shell or run: source $SHELL_RC"
fi

echo "MTPScript installation complete!"
echo "Run 'mtpscript --help' to get started."
echo "Compile: mtpc input.mtp output.msqs"
echo "Run: mtpjs snapshot.msqs"
echo "Execute: mtpscript execute input.mtp"