#!/bin/bash
# Install Node.js v22.21.0 from downloaded archive
set -e

echo "========================================="
echo "  Node.js v22.21.0 Installation"
echo "========================================="
echo ""

# Installation directory
INSTALL_DIR="$HOME/.local/node-v22.21.0"
ARCHIVE="$HOME/Downloads/node-v22.21.0-linux-x64.tar.xz"

# Check if archive exists
if [ ! -f "$ARCHIVE" ]; then
    echo "Error: Archive not found at $ARCHIVE"
    exit 1
fi

# Extract to installation directory
echo "Extracting Node.js to $INSTALL_DIR..."
mkdir -p "$HOME/.local"
tar -xJf "$ARCHIVE" -C "$HOME/.local"

# Create symlink for easier updates
cd "$HOME/.local"
ln -sfn node-v22.21.0-linux-x64 nodejs

echo ""
echo "Node.js extracted to: $INSTALL_DIR"
echo ""

# Add to PATH in .bashrc
echo "Configuring PATH..."
if ! grep -q '.local/nodejs/bin' ~/.bashrc; then
    echo '' >> ~/.bashrc
    echo '# Node.js v22' >> ~/.bashrc
    echo 'export PATH="$HOME/.local/nodejs/bin:$PATH"' >> ~/.bashrc
    echo "Added to ~/.bashrc"
else
    echo "PATH already configured in ~/.bashrc"
fi

# Add to current session
export PATH="$HOME/.local/nodejs/bin:$PATH"

# Verify installation
echo ""
echo "========================================="
echo "  Installation Complete!"
echo "========================================="
echo ""
echo "Node.js version: $(node --version)"
echo "npm version: $(npm --version)"
echo ""
echo "Installation location: $INSTALL_DIR"
echo ""
echo "To use in current terminal:"
echo "  export PATH=\"\$HOME/.local/nodejs/bin:\$PATH\""
echo ""
echo "Or reload your shell:"
echo "  source ~/.bashrc"
echo ""
echo "To verify:"
echo "  node --version"
echo "  npm --version"
