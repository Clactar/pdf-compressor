#!/bin/bash
# Script to build and package PDF Compressor as a macOS app

echo "🔨 Building release version..."
cargo build --release

echo "📦 Creating app bundle..."
rm -rf "PDF Compressor.app"
mkdir -p "PDF Compressor.app/Contents/MacOS"
mkdir -p "PDF Compressor.app/Contents/Resources"

echo "📋 Copying files..."
cp target/release/PDFcompressor "PDF Compressor.app/Contents/MacOS/"
chmod +x "PDF Compressor.app/Contents/MacOS/PDFcompressor"

echo "✅ Done! You can now:"
echo "   1. Drag 'PDF Compressor.app' to your Applications folder"
echo "   2. Double-click it to launch"
echo ""
echo "📍 App location: $(pwd)/PDF Compressor.app"

