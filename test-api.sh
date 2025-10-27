#!/bin/bash

# Test script for PDF Compressor API
# Usage: ./test-api.sh [pdf_file] [compression_level]

set -e

PDF_FILE=${1:-"test.pdf"}
COMPRESSION=${2:-75}
API_URL=${API_URL:-"http://localhost:3000"}

echo "======================================"
echo "PDF Compressor API Test"
echo "======================================"
echo "PDF File: $PDF_FILE"
echo "Compression: $COMPRESSION%"
echo "API URL: $API_URL"
echo "======================================"

# Check if file exists
if [ ! -f "$PDF_FILE" ]; then
    echo "❌ Error: File '$PDF_FILE' not found"
    echo ""
    echo "Usage: $0 <pdf_file> [compression_level]"
    echo "Example: $0 mydocument.pdf 75"
    exit 1
fi

# Check health endpoint
echo ""
echo "🏥 Checking health endpoint..."
if curl -f -s "$API_URL/health" > /dev/null; then
    echo "✅ Health check passed"
else
    echo "❌ API not responding. Is the server running?"
    echo "Start with: cargo run --bin pdfcompressor-api"
    exit 1
fi

# Compress PDF
echo ""
echo "🗜️  Compressing PDF..."
OUTPUT_FILE="${PDF_FILE%.pdf}_compressed.pdf"

# Use curl with verbose headers
RESPONSE=$(curl -X POST "$API_URL/api/pdf" \
  -F "file=@$PDF_FILE" \
  -F "compression=$COMPRESSION" \
  -o "$OUTPUT_FILE" \
  -w "\nHTTP_CODE:%{http_code}\nORIG_SIZE:%{header_json}{\"x-original-size\"}\nCOMP_SIZE:%{header_json}{\"x-compressed-size\"}\nREDUCTION:%{header_json}{\"x-reduction-percentage\"}\n" \
  -s)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE" | cut -d: -f2)

if [ "$HTTP_CODE" == "200" ]; then
    echo "✅ Compression successful!"
    echo ""
    echo "📊 Results:"
    echo "   Output: $OUTPUT_FILE"
    
    # Extract sizes from response if available
    ORIG_SIZE=$(echo "$RESPONSE" | grep -o '"x-original-size":"[0-9]*"' | grep -o '[0-9]*')
    COMP_SIZE=$(echo "$RESPONSE" | grep -o '"x-compressed-size":"[0-9]*"' | grep -o '[0-9]*')
    REDUCTION=$(echo "$RESPONSE" | grep -o '"x-reduction-percentage":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$ORIG_SIZE" ] && [ -n "$COMP_SIZE" ]; then
        echo "   Original: $(numfmt --to=iec-i --suffix=B $ORIG_SIZE 2>/dev/null || echo "$ORIG_SIZE bytes")"
        echo "   Compressed: $(numfmt --to=iec-i --suffix=B $COMP_SIZE 2>/dev/null || echo "$COMP_SIZE bytes")"
        echo "   Reduction: ${REDUCTION}%"
    fi
    
    # Show actual file sizes
    if [ -f "$OUTPUT_FILE" ]; then
        ACTUAL_SIZE=$(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null)
        echo "   File size: $(numfmt --to=iec-i --suffix=B $ACTUAL_SIZE 2>/dev/null || echo "$ACTUAL_SIZE bytes")"
    fi
else
    echo "❌ Compression failed (HTTP $HTTP_CODE)"
    cat "$OUTPUT_FILE" 2>/dev/null || true
    rm -f "$OUTPUT_FILE"
    exit 1
fi

echo ""
echo "======================================"
echo "✨ Test completed successfully!"
echo "======================================"

