#!/bin/bash
# Build script for WASM module
# Production-grade build with optimizations
# Compatible: Linux, macOS, WSL2

set -e

echo "🦀 Building reputation-wasm module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg/

# Build for web target with optimizations
echo "🔨 Building WASM module (release mode)..."
wasm-pack build \
    --target web \
    --release \
    --out-dir pkg \
    --out-name reputation_wasm

# Check build success
if [ -f "pkg/reputation_wasm_bg.wasm" ]; then
    # Cross-platform file size detection
    if command -v stat &> /dev/null; then
        # Try BSD stat (macOS)
        WASM_SIZE=$(stat -f%z "pkg/reputation_wasm_bg.wasm" 2>/dev/null || \
                    stat -c%s "pkg/reputation_wasm_bg.wasm" 2>/dev/null || \
                    echo "unknown")

        # Format size if numfmt available
        if command -v numfmt &> /dev/null && [ "$WASM_SIZE" != "unknown" ]; then
            WASM_SIZE_FORMATTED=$(numfmt --to=iec-i --suffix=B "$WASM_SIZE" 2>/dev/null || echo "${WASM_SIZE} bytes")
        else
            WASM_SIZE_FORMATTED="${WASM_SIZE} bytes"
        fi

        echo "✅ Build successful! WASM size: $WASM_SIZE_FORMATTED"
    else
        echo "✅ Build successful!"
    fi
else
    echo "❌ Build failed - WASM file not found"
    exit 1
fi

# Copy to static assets
STATIC_DIR="../../static/wasm"
mkdir -p "$STATIC_DIR"

echo "📦 Copying to static assets..."
cp pkg/reputation_wasm_bg.wasm "$STATIC_DIR/"
cp pkg/reputation_wasm.js "$STATIC_DIR/"

echo "✅ WASM module built and copied to static/wasm/"
echo ""
echo "📝 Files created:"
echo "   - static/wasm/reputation_wasm_bg.wasm"
echo "   - static/wasm/reputation_wasm.js"
echo ""
echo "🌐 Usage in browser:"
echo "  <script type=\"module\">"
echo "    import init, { verify_reputation_file } from '/static/wasm/reputation_wasm.js';"
echo "    await init();"
echo "    const result = verify_reputation_file(reputationJson);"
echo "  </script>"
