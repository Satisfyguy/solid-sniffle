#!/bin/bash
# Quick Start Script for NEXUS Marketplace on Ubuntu
# Usage: ./UBUNTU_QUICK_START.sh

set -e  # Exit on error

echo "🚀 NEXUS Marketplace - Ubuntu Quick Start"
echo "=========================================="
echo ""

# 1. Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust installed"
else
    echo "✅ Rust already installed ($(rustc --version))"
fi

# 2. Check if diesel_cli is installed
if ! command -v diesel &> /dev/null; then
    echo "📦 Installing diesel_cli..."
    cargo install diesel_cli --no-default-features --features sqlite
    echo "✅ diesel_cli installed"
else
    echo "✅ diesel_cli already installed"
fi

# 3. Check if SQLite is installed
if ! command -v sqlite3 &> /dev/null; then
    echo "📦 Installing SQLite..."
    sudo apt update
    sudo apt install -y sqlite3 libsqlite3-dev
    echo "✅ SQLite installed"
else
    echo "✅ SQLite already installed"
fi

# 4. Setup .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cat > .env << 'EOF'
# Database
DATABASE_URL=marketplace.db

# Encryption (DEV MODE - NOT FOR PRODUCTION)
DB_ENCRYPTION_KEY=dev_key_32_bytes_minimum_length_required_here_1234567890

# Session
SESSION_SECRET_KEY=development_key_do_not_use_in_production_minimum_64_bytes_required

# Server
RUST_LOG=info,actix_web=info,server=debug

# Monero RPC (testnet)
MONERO_RPC_URL=http://127.0.0.1:18082/json_rpc
MONERO_RPC_PORT=18082
EOF
    echo "✅ .env created"
else
    echo "✅ .env already exists"
fi

# 5. Run database migrations
echo "🗄️  Setting up database..."
if [ ! -f marketplace.db ]; then
    echo "Creating new database..."
fi

diesel migration run 2>/dev/null || {
    echo "⚠️  Migration failed, but continuing..."
}

# 6. Build the project
echo "🔨 Building server (this may take a few minutes)..."
cargo build --package server

# 7. Kill any existing server process
if lsof -ti:8080 > /dev/null 2>&1; then
    echo "⚠️  Killing existing server on port 8080..."
    kill -9 $(lsof -ti:8080) 2>/dev/null || true
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "=========================================="
echo "🎉 Starting NEXUS Marketplace Server..."
echo "=========================================="
echo ""
echo "📍 Server will be available at: http://127.0.0.1:8080"
echo ""
echo "🎨 What you'll see:"
echo "   - NEXUS logo on the left"
echo "   - Browse, Categories, Vendors in the center"
echo "   - 🔓 LOGIN and ➕ SIGN UP buttons on the right"
echo ""
echo "✨ The SIGN UP button has a cool shine animation!"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""
echo "=========================================="
echo ""

# 8. Run the server
cargo run --package server
