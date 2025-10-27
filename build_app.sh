#!/bin/bash
# Script to build and package PDF Compressor as a macOS app

echo "🔨 Building release version..."
cargo build --release

APP_PATH="/Applications/PDF Compressor.app"

echo "🛑 Killing any running instances..."
pkill -9 PDFcompressor 2>/dev/null || true
killall "PDF Compressor" 2>/dev/null || true
sleep 1

echo "📦 Creating app bundle structure..."
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

echo "📋 Copying executable..."
cp target/release/pdfcompressor-gui "$APP_PATH/Contents/MacOS/PDFcompressor"
chmod +x "$APP_PATH/Contents/MacOS/PDFcompressor"

echo "🔐 Creating Info.plist..."
cat > "$APP_PATH/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>PDFcompressor</string>
    <key>CFBundleIdentifier</key>
    <string>com.local.pdfcompressor</string>
    <key>CFBundleName</key>
    <string>PDF Compressor</string>
    <key>CFBundleDisplayName</key>
    <string>PDF Compressor</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
</dict>
</plist>
EOF

echo "🔓 Removing quarantine attribute..."
xattr -cr "$APP_PATH" 2>/dev/null || true

echo "✍️  Code signing app..."
codesign --force --deep --sign - "$APP_PATH" 2>/dev/null

echo "✅ Done! App installed to /Applications"
echo ""
echo "🚀 Launching app..."
open "$APP_PATH"

