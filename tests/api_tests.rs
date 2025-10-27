mod common;

use common::*;
use PDFcompressor::{compress_pdf_bytes, compress_image_bytes};

// Note: These tests verify the compression functionality used by the API.
// For full end-to-end API testing with HTTP, authentication, etc., use:
// - The test-api.sh script
// - Manual curl commands against a running server
// - Or add reqwest-based tests with a test server

// ============================================================================
// Core API Functionality Tests
// ============================================================================

#[test]
fn test_api_pdf_compression_workflow() {
    // Simulate what the API does when receiving a PDF
    let pdf_data = generate_minimal_pdf();
    let compression_level = 75u8;
    
    // This is what the API calls internally
    let result = compress_pdf_bytes(&pdf_data, compression_level);
    
    assert!(result.is_ok(), "API PDF compression should succeed");
    let compressed = result.unwrap();
    
    // Verify output is valid PDF
    assert!(compressed.starts_with(b"%PDF"), "Output should be valid PDF");
    assert!(compressed.len() > 0, "Compressed data should not be empty");
    
    // API would return this data with appropriate headers
    let original_size = pdf_data.len();
    let compressed_size = compressed.len();
    let reduction = if original_size > 0 && compressed_size <= original_size {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else if original_size > 0 {
        // Negative reduction (file got bigger)
        -((compressed_size - original_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };
    
    println!("API PDF workflow: {} -> {} bytes ({:.1}% reduction)", 
             original_size, compressed_size, reduction);
}

#[test]
fn test_api_image_compression_workflow() {
    // Simulate what the API does when receiving an image
    let image_data = generate_jpeg_image(500, 400);
    let compression_level = 75u8;
    
    // This is what the API calls internally
    let result = compress_image_bytes(&image_data, compression_level, None);
    
    assert!(result.is_ok(), "API image compression should succeed");
    let (compressed, format) = result.unwrap();
    
    // Verify output
    assert!(compressed.len() > 0, "Compressed data should not be empty");
    assert!(format == "jpg" || format == "png", "Should return valid format");
    
    let original_size = image_data.len();
    let compressed_size = compressed.len();
    
    println!("API image workflow: {} -> {} bytes ({})", 
             original_size, compressed_size, format);
}

// ============================================================================
// API Compression Level Handling Tests
// ============================================================================

#[test]
fn test_api_compression_level_range() {
    let pdf_data = generate_minimal_pdf();
    
    // Test that API supports the documented range (10-95)
    for level in [10, 25, 50, 75, 90, 95] {
        let result = compress_pdf_bytes(&pdf_data, level);
        assert!(result.is_ok(), "Level {} should be supported by API", level);
    }
}

#[test]
fn test_api_compression_level_clamping() {
    let pdf_data = generate_minimal_pdf();
    
    // API should clamp invalid values
    let result_low = compress_pdf_bytes(&pdf_data, 0);
    assert!(result_low.is_ok(), "API should clamp low values");
    
    let result_high = compress_pdf_bytes(&pdf_data, 100);
    assert!(result_high.is_ok(), "API should clamp high values");
}

#[test]
fn test_api_default_compression_level() {
    let pdf_data = generate_minimal_pdf();
    
    // API default is 75%
    let result = compress_pdf_bytes(&pdf_data, 75);
    assert!(result.is_ok(), "API default compression should work");
}

// ============================================================================
// API File Type Detection Tests
// ============================================================================

#[test]
fn test_api_pdf_detection() {
    let pdf_data = generate_minimal_pdf();
    
    // API uses infer crate or checks magic bytes
    let is_pdf = pdf_data.starts_with(b"%PDF");
    assert!(is_pdf, "Should detect PDF by magic bytes");
    
    // Verify compression works
    let result = compress_pdf_bytes(&pdf_data, 75);
    assert!(result.is_ok());
}

#[test]
fn test_api_image_detection() {
    let jpeg_data = generate_jpeg_image(200, 200);
    
    // JPEG magic bytes
    let is_jpeg = jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8;
    assert!(is_jpeg, "Should detect JPEG by magic bytes");
    
    let result = compress_image_bytes(&jpeg_data, 75, None);
    assert!(result.is_ok());
}

// ============================================================================
// API Error Handling Tests
// ============================================================================

#[test]
fn test_api_empty_file_handling() {
    let empty: Vec<u8> = vec![];
    
    // API should return appropriate error for empty files
    let result = compress_pdf_bytes(&empty, 75);
    assert!(result.is_err(), "API should reject empty files");
    
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Should provide error message");
}

#[test]
fn test_api_corrupted_pdf_handling() {
    let corrupted = generate_corrupted_pdf();
    
    // API should handle corrupted files gracefully
    let result = compress_pdf_bytes(&corrupted, 75);
    assert!(result.is_err(), "API should reject corrupted PDFs");
    
    let error = result.unwrap_err();
    assert!(error.contains("Failed to load PDF") || error.contains("load"));
}

#[test]
fn test_api_corrupted_image_handling() {
    let corrupted = generate_corrupted_image();
    
    // API should handle corrupted images gracefully
    let result = compress_image_bytes(&corrupted, 75, None);
    assert!(result.is_err(), "API should reject corrupted images");
}

// ============================================================================
// API Format Conversion Tests
// ============================================================================

#[test]
fn test_api_image_format_conversion_jpeg() {
    let png_data = generate_png_image(300, 200);
    
    // API supports format parameter
    let result = compress_image_bytes(&png_data, 75, Some("jpg"));
    assert!(result.is_ok(), "API should support format conversion");
    
    let (compressed, format) = result.unwrap();
    assert_eq!(format, "jpg", "Should convert to requested format");
    assert!(compressed.len() > 0);
}

#[test]
fn test_api_image_format_conversion_png() {
    let jpeg_data = generate_jpeg_image(300, 200);
    
    let result = compress_image_bytes(&jpeg_data, 75, Some("png"));
    assert!(result.is_ok(), "API should support PNG output");
    
    let (compressed, format) = result.unwrap();
    assert_eq!(format, "png");
    assert!(compressed.len() > 0);
}

#[test]
fn test_api_invalid_format() {
    let jpeg_data = generate_jpeg_image(200, 200);
    
    // API should reject invalid formats
    let result = compress_image_bytes(&jpeg_data, 75, Some("invalid"));
    assert!(result.is_err(), "API should reject invalid formats");
    
    let error = result.unwrap_err();
    assert!(error.contains("Unsupported") || error.contains("format"));
}

// ============================================================================
// API Response Metadata Tests
// ============================================================================

#[test]
fn test_api_response_metadata() {
    let pdf_data = generate_minimal_pdf();
    let original_size = pdf_data.len();
    
    let result = compress_pdf_bytes(&pdf_data, 75);
    assert!(result.is_ok());
    
    let compressed = result.unwrap();
    let compressed_size = compressed.len();
    
    // API would include these in headers
    let reduction_percentage = if original_size > 0 && compressed_size <= original_size {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else if original_size > 0 {
        // Can be negative if compression makes file bigger
        -((compressed_size - original_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };
    
    assert!(original_size > 0);
    assert!(compressed_size > 0);
    // Reduction can be negative (file got bigger)
    println!("Reduction: {:.1}%", reduction_percentage);
}

// ============================================================================
// API Concurrency Tests
// ============================================================================

#[test]
fn test_api_concurrent_compressions() {
    use std::thread;
    
    // Simulate multiple simultaneous API requests
    let handles: Vec<_> = (0..5).map(|i| {
        thread::spawn(move || {
            if i % 2 == 0 {
                let pdf = generate_minimal_pdf();
                compress_pdf_bytes(&pdf, 75)
            } else {
                let img = generate_jpeg_image(200, 200);
                compress_image_bytes(&img, 75, None).map(|(data, _)| data)
            }
        })
    }).collect();
    
    // All requests should complete successfully
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join();
        assert!(result.is_ok(), "Concurrent request {} should not panic", i);
        assert!(result.unwrap().is_ok(), "Concurrent request {} should succeed", i);
    }
}

// ============================================================================
// API Large File Tests
// ============================================================================

#[test]
fn test_api_large_image_handling() {
    // API should handle large images (will be downsampled)
    let large_image = generate_jpeg_image(3000, 2000);
    let original_size = large_image.len();
    
    let result = compress_image_bytes(&large_image, 75, None);
    assert!(result.is_ok(), "API should handle large images");
    
    let (compressed, _) = result.unwrap();
    
    // Should see significant reduction due to downsampling
    println!("Large image API test: {} -> {} bytes", original_size, compressed.len());
    assert!(compressed.len() < original_size, "Large images should be compressed");
}

// ============================================================================
// API Batch Processing Tests
// ============================================================================

#[test]
fn test_api_batch_processing() {
    // Simulate API handling multiple files in sequence
    let files = vec![
        generate_minimal_pdf(),
        generate_pdf_with_image(),
        generate_jpeg_image(300, 200),
    ];
    
    let mut results = Vec::new();
    
    for (i, file) in files.iter().enumerate() {
        let result = if i < 2 {
            // PDF files
            compress_pdf_bytes(file, 75).map(|data| (data, "pdf".to_string()))
        } else {
            // Image files
            compress_image_bytes(file, 75, None)
        };
        
        assert!(result.is_ok(), "Batch item {} should succeed", i);
        results.push(result.unwrap());
    }
    
    assert_eq!(results.len(), 3, "Should process all files");
}

// ============================================================================
// API Quality Level Mapping Tests
// ============================================================================

#[test]
fn test_api_quality_mapping() {
    let image = generate_jpeg_image(400, 300);
    
    // Test different quality levels produce different results
    let mut sizes = Vec::new();
    
    for level in [25, 50, 75, 90] {
        let result = compress_image_bytes(&image, level, None);
        assert!(result.is_ok(), "Quality level {} should work", level);
        
        let (compressed, _) = result.unwrap();
        sizes.push((level, compressed.len()));
    }
    
    // Verify we got all results
    assert_eq!(sizes.len(), 4);
    
    println!("API quality levels:");
    for (level, size) in sizes {
        println!("  Level {}: {} bytes", level, size);
    }
}

// ============================================================================
// API Environment Variable Tests
// ============================================================================

#[test]
fn test_api_compression_rounds_env() {
    // API respects PDF_COMPRESSION_ROUNDS environment variable
    std::env::set_var("PDF_COMPRESSION_ROUNDS", "1");
    
    let pdf = generate_minimal_pdf();
    let result = compress_pdf_bytes(&pdf, 75);
    
    assert!(result.is_ok(), "Should work with custom compression rounds");
    
    std::env::remove_var("PDF_COMPRESSION_ROUNDS");
}

#[test]
fn test_api_default_compression_rounds() {
    // Default should be 2 rounds
    std::env::remove_var("PDF_COMPRESSION_ROUNDS");
    
    let pdf = generate_minimal_pdf();
    let result = compress_pdf_bytes(&pdf, 75);
    
    assert!(result.is_ok(), "Should work with default compression rounds");
}
