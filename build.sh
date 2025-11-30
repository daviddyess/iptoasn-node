#!/bin/bash

# Quick Start Script for iptoasn-server
# This script builds the project and runs examples

set -e  # Exit on error

echo "🚀 iptoasn-server Quick Start"
echo "=============================="
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install from: https://rustup.rs/"
    exit 1
fi
echo "✅ Rust found: $(cargo --version)"

if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install from: https://nodejs.org/"
    exit 1
fi
echo "✅ Node.js found: $(node --version)"

if ! command -v npm &> /dev/null; then
    echo "❌ npm not found. Please install Node.js from: https://nodejs.org/"
    exit 1
fi
echo "✅ npm found: $(npm --version)"

echo ""
echo "🔨 Building Rust workspace..."
cargo build --release
echo "✅ Rust build complete"

echo ""
echo "📦 Building Node.js addon..."
cd iptoasn-node
npm install
npm run build
echo "✅ Node.js addon built"

echo ""
echo "📚 Installing example dependencies..."
cd ../examples
npm install
echo "✅ Dependencies installed"

echo ""
echo "✨ Build complete! You can now run:"
echo ""
echo "  # Basic usage example:"
echo "  cd examples && node basic-usage.js"
echo ""
echo "  # Express server example:"
echo "  cd examples && node express-server.js"
echo ""
echo "  # Or use in your own Node.js project:"
echo "  const { IpToAsn } = require('./iptoasn-node');"
echo ""
echo "📖 For more information, see:"
echo "  - README.md - Project overview"
echo "  - BUILD.md - Detailed build instructions"
echo "  - SUMMARY.md - Complete feature guide"
echo ""
