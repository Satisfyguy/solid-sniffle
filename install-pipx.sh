#!/bin/bash
# Install pipx - Python application installer
set -e

echo "========================================="
echo "  pipx Installation"
echo "========================================="
echo ""

echo "Python version: $(python3 --version)"
echo "pip version: $(pip3 --version | awk '{print $2}')"
echo ""

# Install pipx using pip
echo "Installing pipx..."
python3 -m pip install --user pipx

# Ensure pipx is in PATH
echo ""
echo "Configuring PATH..."
python3 -m pipx ensurepath

# Add to current session
export PATH="$HOME/.local/bin:$PATH"

# Verify installation
echo ""
echo "========================================="
echo "  Installation Complete!"
echo "========================================="
echo ""

if command -v pipx &> /dev/null; then
    echo "pipx version: $(pipx --version)"
    echo "Installation location: $(which pipx)"
else
    echo "pipx installed but not in PATH yet"
    echo "Run: source ~/.bashrc"
fi

echo ""
echo "To use pipx in current terminal:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Or reload your shell:"
echo "  source ~/.bashrc"
echo ""
echo "Example usage:"
echo "  pipx install black      # Install Python formatter"
echo "  pipx install poetry     # Install Poetry"
echo "  pipx list               # List installed apps"
