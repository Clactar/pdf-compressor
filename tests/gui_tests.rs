// GUI unit tests - testing helper functions without GUI framework
// We can't easily test the actual GUI components without egui runtime,
// but we can test the pure functions

// Helper function from main.rs for testing
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

// Compression level to JPEG quality mapping (from main.rs)
fn compression_level_to_quality(compression_level: u8) -> u8 {
    if compression_level <= 25 {
        100 - (compression_level as f32 * 0.4) as u8
    } else if compression_level <= 50 {
        90 - ((compression_level - 25) as f32 * 0.8) as u8
    } else if compression_level <= 75 {
        70 - ((compression_level - 50) as f32 * 0.8) as u8
    } else {
        50 - ((compression_level - 75) as f32) as u8
    }
}

// ============================================================================
// File Size Formatting Tests
// ============================================================================

#[test]
fn test_format_file_size_bytes() {
    assert_eq!(format_file_size(0), "0 B");
    assert_eq!(format_file_size(1), "1 B");
    assert_eq!(format_file_size(999), "999 B");
    assert_eq!(format_file_size(1023), "1023 B");
}

#[test]
fn test_format_file_size_kilobytes() {
    assert_eq!(format_file_size(1024), "1.0 KB");
    assert_eq!(format_file_size(1536), "1.5 KB");
    assert_eq!(format_file_size(2048), "2.0 KB");
    assert_eq!(format_file_size(10240), "10.0 KB");
    assert_eq!(format_file_size(102400), "100.0 KB");
}

#[test]
fn test_format_file_size_megabytes() {
    assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
    assert_eq!(format_file_size(1024 * 1024 + 512 * 1024), "1.5 MB");
    assert_eq!(format_file_size(5 * 1024 * 1024), "5.0 MB");
    assert_eq!(format_file_size(100 * 1024 * 1024), "100.0 MB");
}

#[test]
fn test_format_file_size_gigabytes() {
    assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_file_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    assert_eq!(format_file_size((1.5 * 1024.0 * 1024.0 * 1024.0) as u64), "1.5 GB");
}

#[test]
fn test_format_file_size_precision() {
    // Test that formatting is consistent
    let size = 1536;
    let formatted = format_file_size(size);
    assert!(formatted.contains("1.5"));
    assert!(formatted.contains("KB"));
}

#[test]
fn test_format_file_size_large_values() {
    let very_large = 999 * 1024 * 1024 * 1024;
    let formatted = format_file_size(very_large);
    assert!(formatted.contains("GB"));
}

// ============================================================================
// Compression Level Mapping Tests
// ============================================================================

#[test]
fn test_compression_level_to_quality_low() {
    // Low compression (10-25%) should map to high quality (90-100)
    assert_eq!(compression_level_to_quality(10), 96);
    assert_eq!(compression_level_to_quality(25), 90);
}

#[test]
fn test_compression_level_to_quality_medium() {
    // Medium compression (25-50%) should map to medium-high quality (70-90)
    assert_eq!(compression_level_to_quality(30), 86);
    assert_eq!(compression_level_to_quality(50), 70);
}

#[test]
fn test_compression_level_to_quality_high() {
    // High compression (50-75%) should map to medium quality (50-70)
    assert_eq!(compression_level_to_quality(60), 62);
    assert_eq!(compression_level_to_quality(75), 50);
}

#[test]
fn test_compression_level_to_quality_maximum() {
    // Maximum compression (75-95%) should map to low quality (25-50)
    assert_eq!(compression_level_to_quality(80), 45);
    assert_eq!(compression_level_to_quality(90), 35);
    assert_eq!(compression_level_to_quality(95), 30);
}

#[test]
fn test_compression_level_to_quality_boundaries() {
    // Test exact boundary values
    let quality_10 = compression_level_to_quality(10);
    let quality_25 = compression_level_to_quality(25);
    let quality_50 = compression_level_to_quality(50);
    let quality_75 = compression_level_to_quality(75);
    let quality_95 = compression_level_to_quality(95);
    
    // Quality should decrease as compression increases
    assert!(quality_10 > quality_25);
    assert!(quality_25 > quality_50);
    assert!(quality_50 > quality_75);
    assert!(quality_75 > quality_95);
}

#[test]
fn test_compression_level_to_quality_monotonic() {
    // Quality should be monotonically decreasing with compression level
    let mut prev_quality = 100u8;
    
    for level in 10..=95 {
        let quality = compression_level_to_quality(level);
        assert!(quality <= prev_quality, 
                "Quality should not increase: level {} has quality {}, previous was {}", 
                level, quality, prev_quality);
        prev_quality = quality;
    }
}

// ============================================================================
// Compression Estimation Tests
// ============================================================================

fn estimate_compressed_size(original_size: u64, compression_level: u8) -> u64 {
    let reduction_factor = (compression_level as f64) / 100.0;
    (original_size as f64 * (1.0 - reduction_factor)) as u64
}

#[test]
fn test_estimate_compressed_size_basic() {
    assert_eq!(estimate_compressed_size(1000, 50), 500);
    assert_eq!(estimate_compressed_size(1000, 25), 750);
    assert_eq!(estimate_compressed_size(1000, 75), 250);
}

#[test]
fn test_estimate_compressed_size_edge_cases() {
    assert_eq!(estimate_compressed_size(1000, 0), 1000);
    assert_eq!(estimate_compressed_size(1000, 100), 0);
    assert_eq!(estimate_compressed_size(0, 50), 0);
}

#[test]
fn test_estimate_compressed_size_large_files() {
    let large_size = 100 * 1024 * 1024; // 100 MB
    let estimated = estimate_compressed_size(large_size, 75);
    assert_eq!(estimated, 25 * 1024 * 1024); // Should be 25 MB
}

#[test]
fn test_estimate_compressed_size_realistic() {
    let sizes = vec![
        (10 * 1024, 50),  // 10 KB at 50%
        (1024 * 1024, 75), // 1 MB at 75%
        (50 * 1024 * 1024, 60), // 50 MB at 60%
    ];
    
    for (size, level) in sizes {
        let estimated = estimate_compressed_size(size, level);
        let expected = (size as f64 * (1.0 - level as f64 / 100.0)) as u64;
        assert_eq!(estimated, expected);
    }
}

// ============================================================================
// Compression Result Tests
// ============================================================================

#[derive(Clone, Debug)]
struct CompressionResult {
    file_name: String,
    original_size: u64,
    compressed_size: u64,
    success: bool,
    error_message: Option<String>,
}

impl Default for CompressionResult {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            original_size: 0,
            compressed_size: 0,
            success: false,
            error_message: None,
        }
    }
}

#[test]
fn test_compression_result_default() {
    let result = CompressionResult::default();
    assert_eq!(result.file_name, "");
    assert_eq!(result.original_size, 0);
    assert_eq!(result.compressed_size, 0);
    assert_eq!(result.success, false);
    assert_eq!(result.error_message, None);
}

#[test]
fn test_compression_result_success() {
    let result = CompressionResult {
        file_name: "test.pdf".to_string(),
        original_size: 1000,
        compressed_size: 500,
        success: true,
        error_message: None,
    };
    
    assert_eq!(result.file_name, "test.pdf");
    assert_eq!(result.original_size, 1000);
    assert_eq!(result.compressed_size, 500);
    assert!(result.success);
    assert!(result.error_message.is_none());
}

#[test]
fn test_compression_result_failure() {
    let result = CompressionResult {
        file_name: "bad.pdf".to_string(),
        original_size: 1000,
        compressed_size: 0,
        success: false,
        error_message: Some("Invalid PDF".to_string()),
    };
    
    assert!(!result.success);
    assert!(result.error_message.is_some());
    assert_eq!(result.error_message.unwrap(), "Invalid PDF");
}

#[test]
fn test_compression_result_clone() {
    let result = CompressionResult {
        file_name: "test.pdf".to_string(),
        original_size: 1000,
        compressed_size: 500,
        success: true,
        error_message: None,
    };
    
    let cloned = result.clone();
    assert_eq!(cloned.file_name, result.file_name);
    assert_eq!(cloned.original_size, result.original_size);
    assert_eq!(cloned.compressed_size, result.compressed_size);
}

// ============================================================================
// Reduction Percentage Calculation Tests
// ============================================================================

fn calculate_reduction_percentage(original: u64, compressed: u64) -> f64 {
    if original == 0 {
        return 0.0;
    }
    (original as i64 - compressed as i64) as f64 / original as f64 * 100.0
}

#[test]
fn test_calculate_reduction_percentage_basic() {
    assert_eq!(calculate_reduction_percentage(1000, 500), 50.0);
    assert_eq!(calculate_reduction_percentage(1000, 750), 25.0);
    assert_eq!(calculate_reduction_percentage(1000, 250), 75.0);
}

#[test]
fn test_calculate_reduction_percentage_edge_cases() {
    assert_eq!(calculate_reduction_percentage(1000, 1000), 0.0);
    assert_eq!(calculate_reduction_percentage(1000, 0), 100.0);
    assert_eq!(calculate_reduction_percentage(0, 0), 0.0);
}

#[test]
fn test_calculate_reduction_percentage_expansion() {
    // If compressed is larger (shouldn't happen normally)
    let result = calculate_reduction_percentage(1000, 1100);
    assert!(result < 0.0); // Negative reduction (expansion)
}

#[test]
fn test_calculate_reduction_percentage_precision() {
    let reduction = calculate_reduction_percentage(1234, 567);
    let expected = (1234 - 567) as f64 / 1234.0 * 100.0;
    assert!((reduction - expected).abs() < 0.01);
}

// ============================================================================
// State Management Tests
// ============================================================================

#[test]
fn test_progress_tracking() {
    let total_files = 10;
    let processed_files = 7;
    
    let progress = format!("Processed {}/{} files", processed_files, total_files);
    assert_eq!(progress, "Processed 7/10 files");
}

#[test]
fn test_compression_level_ranges() {
    // Test compression level descriptions
    let get_description = |level: u8| -> &str {
        if level <= 25 {
            "Low Compression"
        } else if level <= 50 {
            "Medium Compression"
        } else if level <= 75 {
            "High Compression"
        } else {
            "Maximum Compression"
        }
    };
    
    assert_eq!(get_description(10), "Low Compression");
    assert_eq!(get_description(25), "Low Compression");
    assert_eq!(get_description(26), "Medium Compression");
    assert_eq!(get_description(50), "Medium Compression");
    assert_eq!(get_description(51), "High Compression");
    assert_eq!(get_description(75), "High Compression");
    assert_eq!(get_description(76), "Maximum Compression");
    assert_eq!(get_description(95), "Maximum Compression");
}

// ============================================================================
// File Name Handling Tests
// ============================================================================

#[test]
fn test_generate_output_filename() {
    let generate_output_name = |input: &str, ext: &str| -> String {
        let stem = if let Some(pos) = input.rfind('.') {
            &input[..pos]
        } else {
            input
        };
        format!("{}_compressed.{}", stem, ext)
    };
    
    assert_eq!(generate_output_name("test.pdf", "pdf"), "test_compressed.pdf");
    assert_eq!(generate_output_name("image.jpg", "jpg"), "image_compressed.jpg");
    assert_eq!(generate_output_name("document", "pdf"), "document_compressed.pdf");
}

#[test]
fn test_file_extension_detection() {
    fn get_extension(filename: &str) -> Option<&str> {
        filename.rfind('.').map(|pos| &filename[pos + 1..])
    }
    
    assert_eq!(get_extension("test.pdf"), Some("pdf"));
    assert_eq!(get_extension("image.jpg"), Some("jpg"));
    assert_eq!(get_extension("document.tar.gz"), Some("gz"));
    assert_eq!(get_extension("noextension"), None);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validate_compression_level() {
    let is_valid_level = |level: u8| -> bool {
        level >= 10 && level <= 95
    };
    
    assert!(is_valid_level(10));
    assert!(is_valid_level(50));
    assert!(is_valid_level(95));
    assert!(!is_valid_level(0));
    assert!(!is_valid_level(9));
    assert!(!is_valid_level(96));
    assert!(!is_valid_level(100));
}

#[test]
fn test_clamp_compression_level() {
    let clamp_level = |level: u8| -> u8 {
        level.max(10).min(95)
    };
    
    assert_eq!(clamp_level(0), 10);
    assert_eq!(clamp_level(5), 10);
    assert_eq!(clamp_level(50), 50);
    assert_eq!(clamp_level(96), 95);
    assert_eq!(clamp_level(100), 95);
    assert_eq!(clamp_level(255), 95);
}

