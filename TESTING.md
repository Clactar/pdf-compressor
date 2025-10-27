# Testing Guide

Quick reference for running tests on the PDF Compressor application.

## Quick Start

```bash
# Run all tests
cargo test

# Run all tests with output
cargo test -- --nocapture

# Run specific test suite
cargo test --test lib_tests      # Core library tests (32 tests)
cargo test --test api_tests      # API integration tests (24 tests)
cargo test --test gui_tests      # GUI helper tests (30 tests)
cargo test --test generate_fixtures  # Fixture generation (5 tests)

# Run benchmarks
cargo bench

# Compile benchmarks without running
cargo bench --no-run
```

## Test Organization

```
tests/
├── common/
│   └── mod.rs              # Shared test utilities and fixture generators
├── fixtures/               # Generated test files (PDFs, images)
├── lib_tests.rs           # Core compression library tests
├── api_tests.rs           # API functionality tests
├── gui_tests.rs           # GUI helper function tests
└── generate_fixtures.rs   # Test data generation

benches/
└── compression_bench.rs   # Performance benchmarks
```

## Test Coverage

✅ **91 Total Tests**
- 32 Library unit tests
- 24 API integration tests
- 30 GUI helper tests
- 5 Fixture generation tests

## Running Specific Tests

```bash
# Run a single test by name
cargo test test_compress_pdf_bytes_valid

# Run tests matching a pattern
cargo test compress_pdf

# Run tests in a specific file
cargo test --test lib_tests

# Run with verbose output
cargo test -- --nocapture

# Run tests single-threaded (for debugging)
cargo test -- --test-threads=1
```

## Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench pdf_compression

# Save benchmark results
cargo bench > benchmark_results.txt

# Compare benchmarks
cargo bench -- --save-baseline before
# Make changes...
cargo bench -- --baseline before
```

## CI/CD Integration

```bash
# Quick test (for CI)
cargo test --quiet

# Full test with coverage (requires tarpaulin)
cargo tarpaulin --out Html

# Check that benchmarks compile
cargo bench --no-run
```

## Manual Testing

### API Server Testing
```bash
# Terminal 1: Start the server
cargo run --bin pdfcompressor-api

# Terminal 2: Test with script
./test-api.sh test.pdf 75

# Or test with curl
curl -X POST http://localhost:3000/api/compress \
  -F "file=@test.pdf" \
  -F "compression=75" \
  -o compressed.pdf
```

### GUI Application Testing
```bash
# Run the GUI
cargo run --bin pdfcompressor-gui --features gui

# Manual test checklist:
# □ Select PDF files
# □ Select image files  
# □ Adjust compression slider
# □ Compress files
# □ Verify results
# □ Download compressed files
```

## Troubleshooting

### Tests fail to compile
```bash
# Clean build
cargo clean
cargo test
```

### Fixtures missing
```bash
# Regenerate test fixtures
cargo test --test generate_fixtures -- --nocapture
```

### Benchmark issues
```bash
# Ensure benchmarks compile
cargo bench --no-run

# Run single benchmark
cargo bench pdf_compression_quality
```

## Test Output

### Successful Test Run
```
running 91 tests
test result: ok. 91 passed; 0 failed; 0 ignored
```

### With Details
```
test test_compress_pdf_bytes_valid ... ok
test test_compress_image_bytes_jpeg ... ok
test test_api_pdf_compression_workflow ... ok
test test_format_file_size_bytes ... ok
```

## Additional Resources

- **Full Test Summary**: See `TEST-SUMMARY.md` for detailed test coverage
- **API Testing**: See `test-api.sh` for API endpoint testing
- **Documentation**: See `README-API.md` for API documentation

## Status

✅ All 91 tests passing  
✅ Benchmarks compile and run  
✅ Production ready

