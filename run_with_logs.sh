#!/bin/bash
# Run the PDF compressor with detailed logging

export RUST_LOG=debug
./target/release/PDFcompressor 2>&1 | tee compression_log.txt

