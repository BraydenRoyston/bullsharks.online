#!/bin/bash
# Development environment setup script for BullSharks.online

set -e

echo "🏃‍♂️ BullSharks.online Development Setup"
echo "================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✅ Rust installed"
else
    echo "✅ Rust found: $(rustc --version)"
fi

# Check if just is installed
if ! command -v just &> /dev/null; then
    echo "📦 Installing just (command runner)..."
    cargo install just
    echo "✅ just installed"
else
    echo "✅ just found: $(just --version)"
fi

# Setup environment file
if [ ! -f .env ]; then
    echo "📝 Setting up environment file..."
    cp .env.example .env
    echo "⚠️  Please edit .env with your configuration"
else
    echo "✅ .env file exists"
fi

# Install cargo-watch for development
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch for hot reload..."
    cargo install cargo-watch
    echo "✅ cargo-watch installed"
else
    echo "✅ cargo-watch found"
fi

# Verify dependencies
echo "🔍 Checking dependencies..."
cargo check --quiet

echo ""
echo "🎉 Setup complete! Next steps:"
echo "   1. Edit .env with your Strava API credentials"
echo "   2. Run 'just dev' to start development server"
echo "   3. Check 'just --list' for all available commands"
echo ""
echo "📚 Documentation:"
echo "   • DEVELOPMENT.md - Development guide"
echo "   • AGENT_GUIDE.md - AI agent development guide"
echo "   • docs/API_DOCUMENTATION.md - API reference"