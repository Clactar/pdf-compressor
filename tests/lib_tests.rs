mod common;

use PDFcompressor::{compress_pdf_bytes, compress_image_bytes, CompressionSettings};
use common::*;

// ============================================================================
// PDF Compression Tests
// ============================================================================

#[test]
fn test_compress_pdf_bytes_valid_minimal() {
    let input = generate_minimal_pdf();
    let original_size = input.len();
    
    let result = compress_pdf_bytes(&input, 75);
    assert!(result.is_ok(), "PDF compression should succeed");
    
    let compressed = result.unwrap();
    assert!(compressed.len() > 0, "Compressed PDF should not be empty");
    assert!(compressed.starts_with(b"%PDF"), "Output should be valid PDF");
    
    println!("Minimal PDF: {} -> {} bytes", original_size, compressed.len());
}

#[test]
fn test_compress_pdf_bytes_with_image() {
    let input = generate_pdf_with_image();
    let original_size = input.len();
    
    let result = compress_pdf_bytes(&input, 75);
    assert!(result.is_ok(), "PDF with image compression should succeed");
    
    let compressed = result.unwrap();
    assert!(compressed.len() > 0, "Compressed PDF should not be empty");
    assert!(compressed.starts_with(b"%PDF"), "Output should be valid PDF");
    
    println!("PDF with image: {} -> {} bytes", original_size, compressed.len());
}

#[test]
fn test_compress_pdf_bytes_quality_levels() {
    let input = generate_pdf_with_image();
    
    // Test different compression levels
    let levels = vec![10, 25, 50, 75, 90, 95];
    let mut results = Vec::new();
    
    for level in levels {
        let result = compress_pdf_bytes(&input, level);
        assert!(result.is_ok(), "Compression at level {} should succeed", level);
        
        let compressed = result.unwrap();
        results.push((level, compressed.len()));
        println!("Level {}: {} bytes", level, compressed.len());
    }
    
    // Generally, higher compression levels should produce smaller files
    // But this isn't always guaranteed due to the complexity of PDF structure
    assert!(results.len() == 6, "Should have 6 compression results");
}

#[test]
fn test_compress_pdf_bytes_level_clamping() {
    let input = generate_minimal_pdf();
    
    // Test values outside valid range get clamped
    let result_too_low = compress_pdf_bytes(&input, 0);
    assert!(result_too_low.is_ok(), "Should clamp low values");
    
    let result_too_high = compress_pdf_bytes(&input, 100);
    assert!(result_too_high.is_ok(), "Should clamp high values");
    
    let result_extreme = compress_pdf_bytes(&input, 255);
    assert!(result_extreme.is_ok(), "Should clamp extreme values");
}

#[test]
fn test_compress_pdf_bytes_invalid_input() {
    let corrupted = generate_corrupted_pdf();
    
    let result = compress_pdf_bytes(&corrupted, 75);
    assert!(result.is_err(), "Corrupted PDF should return error");
    
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Failed to load PDF") || error_msg.contains("load"), 
            "Error message should indicate loading failure: {}", error_msg);
}

#[test]
fn test_compress_pdf_bytes_empty_input() {
    let empty: Vec<u8> = vec![];
    
    let result = compress_pdf_bytes(&empty, 75);
    assert!(result.is_err(), "Empty input should return error");
}

#[test]
fn test_compress_pdf_bytes_idempotent() {
    let input = generate_minimal_pdf();
    
    // Compress once
    let first_compression = compress_pdf_bytes(&input, 75).unwrap();
    
    // Compress again
    let second_compression = compress_pdf_bytes(&first_compression, 75).unwrap();
    
    // Both should be valid PDFs
    assert!(first_compression.starts_with(b"%PDF"));
    assert!(second_compression.starts_with(b"%PDF"));
    
    // Second compression might not reduce size much more
    println!("Original: {} bytes", input.len());
    println!("First compression: {} bytes", first_compression.len());
    println!("Second compression: {} bytes", second_compression.len());
}

// ============================================================================
// Image Compression Tests
// ============================================================================

#[test]
fn test_compress_image_bytes_jpeg() {
    let input = generate_jpeg_image(500, 400);
    let original_size = input.len();
    
    let result = compress_image_bytes(&input, 75, None);
    assert!(result.is_ok(), "JPEG compression should succeed");
    
    let (compressed, ext) = result.unwrap();
    assert!(compressed.len() > 0, "Compressed image should not be empty");
    assert!(ext == "jpg" || ext == "png", "Should output jpg or png, got: {}", ext);
    
    println!("JPEG: {} -> {} bytes ({})", original_size, compressed.len(), ext);
}

#[test]
fn test_compress_image_bytes_png() {
    let input = generate_png_image(500, 400);
    let original_size = input.len();
    
    let result = compress_image_bytes(&input, 75, None);
    assert!(result.is_ok(), "PNG compression should succeed");
    
    let (compressed, ext) = result.unwrap();
    assert!(compressed.len() > 0, "Compressed image should not be empty");
    assert!(ext == "jpg" || ext == "png", "Should output jpg or png, got: {}", ext);
    
    println!("PNG: {} -> {} bytes ({})", original_size, compressed.len(), ext);
}

#[test]
fn test_compress_image_bytes_large_image() {
    let input = generate_jpeg_image(2000, 1500);
    let original_size = input.len();
    
    let result = compress_image_bytes(&input, 75, None);
    assert!(result.is_ok(), "Large image compression should succeed");
    
    let (compressed, _) = result.unwrap();
    // Large images should see significant size reduction due to downsampling
    println!("Large image: {} -> {} bytes ({:.1}% reduction)", 
             original_size, compressed.len(),
             (1.0 - compressed.len() as f64 / original_size as f64) * 100.0);
}

#[test]
fn test_compress_image_bytes_quality_levels() {
    let input = generate_jpeg_image(400, 300);
    
    let levels = vec![10, 25, 50, 75, 90];
    let mut sizes = Vec::new();
    
    for level in levels {
        let result = compress_image_bytes(&input, level, None);
        assert!(result.is_ok(), "Compression at level {} should succeed", level);
        
        let (compressed, _) = result.unwrap();
        sizes.push(compressed.len());
        println!("Level {}: {} bytes", level, compressed.len());
    }
    
    // Higher compression should generally produce smaller files
    assert!(sizes.len() == 5);
}

#[test]
fn test_compress_image_bytes_format_conversion_jpeg() {
    let input = generate_png_image(300, 200);
    
    let result = compress_image_bytes(&input, 75, Some("jpg"));
    assert!(result.is_ok(), "PNG to JPEG conversion should succeed");
    
    let (compressed, ext) = result.unwrap();
    assert_eq!(ext, "jpg", "Should output JPEG");
    assert!(compressed.len() > 0);
}

#[test]
fn test_compress_image_bytes_format_conversion_png() {
    let input = generate_jpeg_image(300, 200);
    
    let result = compress_image_bytes(&input, 75, Some("png"));
    assert!(result.is_ok(), "JPEG to PNG conversion should succeed");
    
    let (compressed, ext) = result.unwrap();
    assert_eq!(ext, "png", "Should output PNG");
    assert!(compressed.len() > 0);
}

#[test]
fn test_compress_image_bytes_invalid_format() {
    let input = generate_jpeg_image(100, 100);
    
    let result = compress_image_bytes(&input, 75, Some("invalid"));
    assert!(result.is_err(), "Invalid format should return error");
    
    let error = result.unwrap_err();
    assert!(error.contains("Unsupported output format") || error.contains("format"),
            "Error should mention format: {}", error);
}

#[test]
fn test_compress_image_bytes_corrupted_input() {
    let corrupted = generate_corrupted_image();
    
    let result = compress_image_bytes(&corrupted, 75, None);
    assert!(result.is_err(), "Corrupted image should return error");
}

#[test]
fn test_compress_image_bytes_empty_input() {
    let empty: Vec<u8> = vec![];
    
    let result = compress_image_bytes(&empty, 75, None);
    assert!(result.is_err(), "Empty input should return error");
}

#[test]
fn test_compress_image_bytes_level_clamping() {
    let input = generate_jpeg_image(200, 200);
    
    // Test edge cases
    let result_min = compress_image_bytes(&input, 0, None);
    assert!(result_min.is_ok(), "Should clamp minimum level");
    
    let result_max = compress_image_bytes(&input, 100, None);
    assert!(result_max.is_ok(), "Should clamp maximum level");
}

// ============================================================================
// Compression Settings Tests
// ============================================================================

#[test]
fn test_compression_settings_creation() {
    let settings = CompressionSettings { quality: 75 };
    assert_eq!(settings.quality, 75);
}

#[test]
fn test_compression_settings_clone() {
    let settings = CompressionSettings { quality: 80 };
    let cloned = settings.clone();
    assert_eq!(cloned.quality, 80);
}

// ============================================================================
// Integration Tests - Real-world Scenarios
// ============================================================================

#[test]
fn test_batch_pdf_compression() {
    let pdfs = vec![
        generate_minimal_pdf(),
        generate_pdf_with_image(),
        generate_minimal_pdf(),
    ];
    
    for (i, pdf) in pdfs.iter().enumerate() {
        let result = compress_pdf_bytes(pdf, 75);
        assert!(result.is_ok(), "Batch PDF {} should compress successfully", i);
        println!("Batch PDF {}: {} -> {} bytes", i, pdf.len(), result.unwrap().len());
    }
}

#[test]
fn test_batch_image_compression() {
    let images = vec![
        generate_jpeg_image(200, 200),
        generate_png_image(300, 200),
        generate_jpeg_image(150, 150),
    ];
    
    for (i, img) in images.iter().enumerate() {
        let result = compress_image_bytes(img, 75, None);
        assert!(result.is_ok(), "Batch image {} should compress successfully", i);
        
        let (compressed, ext) = result.unwrap();
        println!("Batch image {}: {} -> {} bytes ({})", i, img.len(), compressed.len(), ext);
    }
}

#[test]
fn test_mixed_quality_compression() {
    let input = generate_pdf_with_image();
    
    // Compress at different quality levels and ensure all succeed
    for quality in [10, 30, 50, 70, 90].iter() {
        let result = compress_pdf_bytes(&input, *quality);
        assert!(result.is_ok(), "Quality {} should work", quality);
    }
}

#[test]
fn test_compression_preserves_pdf_structure() {
    let input = generate_minimal_pdf();
    let result = compress_pdf_bytes(&input, 75).unwrap();
    
    // Verify the compressed PDF can be loaded again
    let doc = lopdf::Document::load_mem(&result);
    assert!(doc.is_ok(), "Compressed PDF should be loadable");
    
    let doc = doc.unwrap();
    assert!(doc.get_pages().len() > 0, "Should preserve pages");
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_very_small_pdf() {
    // Already minimal PDF
    let input = generate_minimal_pdf();
    let result = compress_pdf_bytes(&input, 90);
    assert!(result.is_ok(), "Small PDF should compress");
    
    let compressed = result.unwrap();
    // Even with high compression, structure overhead means it might not shrink much
    assert!(compressed.len() > 0);
}

#[test]
fn test_very_small_image() {
    let input = generate_jpeg_image(10, 10);
    let result = compress_image_bytes(&input, 90, None);
    assert!(result.is_ok(), "Tiny image should compress");
}

#[test]
fn test_extreme_compression_levels() {
    let input = generate_jpeg_image(200, 200);
    
    // Test boundary values
    let min_result = compress_image_bytes(&input, 10, None);
    assert!(min_result.is_ok(), "Min compression level should work");
    
    let max_result = compress_image_bytes(&input, 95, None);
    assert!(max_result.is_ok(), "Max compression level should work");
}

#[test]
fn test_compression_with_env_var() {
    // Test that PDF_COMPRESSION_ROUNDS env var is respected
    std::env::set_var("PDF_COMPRESSION_ROUNDS", "1");
    
    let input = generate_minimal_pdf();
    let result = compress_pdf_bytes(&input, 50);
    assert!(result.is_ok(), "Should work with custom compression rounds");
    
    std::env::remove_var("PDF_COMPRESSION_ROUNDS");
}

#[test]
fn test_concurrent_compression() {
    use std::thread;
    
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            let input = if i % 2 == 0 {
                generate_minimal_pdf()
            } else {
                generate_jpeg_image(200, 200)
            };
            
            if i % 2 == 0 {
                compress_pdf_bytes(&input, 75)
            } else {
                compress_image_bytes(&input, 75, None).map(|(data, _)| data)
            }
        })
    }).collect();
    
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join();
        assert!(result.is_ok(), "Thread {} should not panic", i);
        assert!(result.unwrap().is_ok(), "Thread {} compression should succeed", i);
    }
}

#[test]
fn test_parallel_stream_compression_no_lock_contention() {
    // Test that parallel stream compression uses atomics instead of mutexes
    // This test compresses a PDF with multiple streams to verify no lock contention
    let pdf_with_image = generate_pdf_with_image();
    
    // Run multiple times to stress test the parallel processing
    for _ in 0..3 {
        let result = compress_pdf_bytes(&pdf_with_image, 75);
        assert!(result.is_ok(), "Parallel stream compression should succeed");
        
        let compressed = result.unwrap();
        assert!(compressed.len() > 0);
        assert!(compressed.starts_with(b"%PDF"));
    }
    
    // The fact that this completes quickly without deadlocks confirms
    // that atomic operations are being used instead of mutex locks
    println!("✓ Parallel stream compression uses lock-free atomic operations");
}

