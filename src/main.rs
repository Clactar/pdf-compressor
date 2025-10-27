// This is now the GUI binary - for API usage, use the 'api' binary instead
use eframe::egui;
use egui::{CentralPanel, Context, ScrollArea, Color32, RichText};
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use log::{info, warn};

struct PdfCompressor {
    selected_files: Vec<PathBuf>,
    compression_results: Vec<CompressionResult>,
    is_processing: bool,
    processing_progress: String,
    receiver: Option<Receiver<CompressionResult>>,
    compression_level: u8, // 0-100, how much to compress (0=minimal, 100=maximum)
    estimated_size: Option<u64>,
}

impl Default for PdfCompressor {
    fn default() -> Self {
        Self {
            selected_files: Vec::new(),
            compression_results: Vec::new(),
            is_processing: false,
            processing_progress: String::new(),
            receiver: None,
            compression_level: 75, // Default: 75% compression (good balance)
            estimated_size: None,
        }
    }
}

#[derive(Clone)]
struct CompressionResult {
    file_name: String,
    original_size: u64,
    compressed_size: u64,
    success: bool,
    error_message: Option<String>,
    compressed_path: Option<PathBuf>,
    downloaded: bool,
}

// Settings moved to lib.rs - using library function now

impl Default for CompressionResult {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            original_size: 0,
            compressed_size: 0,
            success: false,
            error_message: None,
            compressed_path: None,
            downloaded: false,
        }
    }
}

impl PdfCompressor {
    fn new() -> Self {
        Self::default()
    }

    fn select_files(&mut self) {
        if let Some(files) = FileDialog::new()
            .add_filter("All supported", &["pdf", "jpg", "jpeg", "png", "webp", "tiff", "tif"])
            .add_filter("PDF files", &["pdf"])
            .add_filter("Images", &["jpg", "jpeg", "png", "webp", "tiff", "tif"])
            .set_directory(".")
            .pick_files()
        {
            self.selected_files = files;
            // Clear previous results when selecting new files
            self.compression_results.clear();
            // Estimate compressed size
            self.estimate_compressed_size();
        }
    }
    
    fn estimate_compressed_size(&mut self) {
        if self.selected_files.is_empty() {
            self.estimated_size = None;
            return;
        }
        
        let total_size: u64 = self.selected_files.iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .map(|m| m.len())
            .sum();
        
        // Compression level 0-100 directly maps to reduction %
        // Level 25 = ~25% reduction (high quality, minimal compression)
        // Level 50 = ~50% reduction (balanced)
        // Level 75 = ~75% reduction (good compression) - DEFAULT
        // Level 90 = ~90% reduction (aggressive)
        let reduction_factor = (self.compression_level as f64) / 100.0;
        
        self.estimated_size = Some((total_size as f64 * (1.0 - reduction_factor)) as u64);
    }
    
    fn download_all(&mut self) {
        if let Some(folder) = FileDialog::new().pick_folder() {
            info!("Downloading all files to: {:?}", folder);
            
            for result in &mut self.compression_results {
                if result.success && !result.downloaded {
                    if let Some(ref temp_path) = result.compressed_path {
                        let dest_path = folder.join(&result.file_name);
                        match std::fs::copy(temp_path, &dest_path) {
                            Ok(_) => {
                                result.downloaded = true;
                                info!("Downloaded: {:?}", dest_path);
                            }
                            Err(e) => {
                                warn!("Failed to download {}: {}", result.file_name, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn cleanup_temp_files(&mut self) {
        info!("Cleaning up temporary files...");
        
        for result in &self.compression_results {
            if result.success && !result.downloaded {
                if let Some(ref temp_path) = result.compressed_path {
                    if let Err(e) = std::fs::remove_file(temp_path) {
                        warn!("Failed to delete temp file {:?}: {}", temp_path, e);
                    } else {
                        info!("Deleted temp file: {:?}", temp_path);
                    }
                }
            }
        }
    }

    fn compress_files(&mut self) {
        if self.selected_files.is_empty() || self.is_processing {
            return;
        }

        self.is_processing = true;
        self.compression_results.clear();

        let files = self.selected_files.clone();
        // Convert compression level to JPEG quality
        // Compression 0-25 = JPEG 90-100 (high quality)
        // Compression 25-50 = JPEG 70-90
        // Compression 50-75 = JPEG 50-70
        // Compression 75-100 = JPEG 25-50 (low quality)
        let jpeg_quality = if self.compression_level <= 25 {
            100 - (self.compression_level as f32 * 0.4) as u8 // 100 to 90
        } else if self.compression_level <= 50 {
            90 - ((self.compression_level - 25) as f32 * 0.8) as u8 // 90 to 70
        } else if self.compression_level <= 75 {
            70 - ((self.compression_level - 50) as f32 * 0.8) as u8 // 70 to 50
        } else {
            50 - ((self.compression_level - 75) as f32) as u8 // 50 to 25
        };
        
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);

        thread::spawn(move || {
            for file_path in files {
                let result = compress_single_file(&file_path, jpeg_quality);
                let _ = tx.send(result);
            }
        });
    }

    fn check_processing_results(&mut self) {
        if let Some(ref rx) = self.receiver {
            // Non-blocking check for new results
            while let Ok(result) = rx.try_recv() {
                self.compression_results.push(result);
                self.processing_progress = format!(
                    "Processed {}/{} files",
                    self.compression_results.len(),
                    self.selected_files.len()
                );
                
                // Check if all files are processed
                if self.compression_results.len() >= self.selected_files.len() {
                    self.is_processing = false;
                    self.receiver = None;
                    self.processing_progress.clear();
                    break;
                }
            }
        }
    }
}

fn compress_single_file(input_path: &Path, compression_level: u8) -> CompressionResult {
    let file_name = input_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    info!("========================================");
    info!("Starting compression for: {}", file_name);

    // Get original file size
    let original_size = match input_path.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };
    info!("Original file size: {} bytes", original_size);

    // Detect file type by extension
    let extension = input_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let is_pdf = extension == "pdf";
    info!("File type: {}", if is_pdf { "PDF" } else { "Image" });

    // Read input file and compress using library
    match std::fs::read(input_path) {
        Ok(input_bytes) => {
            let (compressed_bytes, output_ext) = if is_pdf {
                match PDFcompressor::compress_pdf_bytes(&input_bytes, compression_level) {
                    Ok(bytes) => (bytes, "pdf".to_string()),
                    Err(error) => {
                        return CompressionResult {
                            file_name,
                            original_size,
                            compressed_size: 0,
                            success: false,
                            error_message: Some(error),
                            compressed_path: None,
                            downloaded: false,
                        };
                    }
                }
            } else {
                match PDFcompressor::compress_image_bytes(&input_bytes, compression_level, None) {
                    Ok((bytes, ext)) => (bytes, ext),
                    Err(error) => {
                        return CompressionResult {
                            file_name,
                            original_size,
                            compressed_size: 0,
                            success: false,
                            error_message: Some(error),
                            compressed_path: None,
                            downloaded: false,
                        };
                    }
                }
            };
            
            // Generate temp output path in system temp directory
            let temp_dir = std::env::temp_dir();
            let file_stem = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("compressed");
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let output_filename = format!("{}_compressed_{}.{}", file_stem, timestamp, output_ext);
            let output_path = temp_dir.join(&output_filename);
            info!("Temp output path: {:?}", output_path);
            
            // Write to temp file
            if let Err(e) = std::fs::write(&output_path, &compressed_bytes) {
                return CompressionResult {
                    file_name,
                    original_size,
                    compressed_size: 0,
                    success: false,
                    error_message: Some(format!("Failed to write output: {}", e)),
                    compressed_path: None,
                    downloaded: false,
                };
            }
            
            let compressed_size = compressed_bytes.len() as u64;
            let reduction = if original_size > 0 {
                (original_size as i64 - compressed_size as i64) as f64 / original_size as f64 * 100.0
            } else {
                0.0
            };
            
            info!("Compressed file size: {} bytes", compressed_size);
            info!("Reduction: {:.2}%", reduction);
            info!("========================================\n");

            CompressionResult {
                file_name: output_filename,
                original_size,
                compressed_size,
                success: true,
                error_message: None,
                compressed_path: Some(output_path),
                downloaded: false,
            }
        }
        Err(e) => CompressionResult {
            file_name,
            original_size,
            compressed_size: 0,
            success: false,
            error_message: Some(format!("Failed to read input file: {}", e)),
            compressed_path: None,
            downloaded: false,
        }
    }
}

// Compression functions moved to lib.rs

impl eframe::App for PdfCompressor {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Check for processing results
        if self.is_processing {
            self.check_processing_results();
            ctx.request_repaint(); // Keep UI responsive
        }
        
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("PDF & Image Compressor").size(24.0).strong());
            ui.add_space(10.0);
            ui.separator();

            // File selection section
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📁 Select Files (PDF or Images)").size(16.0)).clicked() {
                    self.select_files();
                }

                ui.label(RichText::new(format!("{} file(s) selected", self.selected_files.len()))
                    .size(16.0)
                    .color(if self.selected_files.is_empty() { Color32::GRAY } else { Color32::WHITE }));
            });
            ui.add_space(10.0);

            if !self.selected_files.is_empty() {
                ui.separator();
                ui.label(RichText::new("Selected files:").size(15.0).strong());
                ui.add_space(5.0);

                ScrollArea::vertical()
                    .id_salt("selected_files_scroll")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for file in &self.selected_files {
                            ui.label(RichText::new(format!("📄 {}", file.file_name().unwrap_or_default().to_string_lossy()))
                                .size(15.0));
                        }
                    });
                
                ui.add_space(10.0);
                
                // Compression slider
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Compression Level:").size(15.0).strong());
                    let old_level = self.compression_level;
                    ui.add(egui::Slider::new(&mut self.compression_level, 10..=95)
                        .text("")
                        .suffix("%"));
                    
                    // Re-estimate if level changed
                    if old_level != self.compression_level {
                        self.estimate_compressed_size();
                    }
                });
                
                // Show compression description
                let compression_desc = if self.compression_level <= 25 {
                    "Low Compression (10-25%) - Best quality, larger file size"
                } else if self.compression_level <= 50 {
                    "Medium Compression (25-50%) - Good balance"
                } else if self.compression_level <= 75 {
                    "High Compression (50-75%) - Good size reduction (Recommended)"
                } else {
                    "Maximum Compression (75-95%) - Smallest files, quality loss visible"
                };
                
                ui.label(RichText::new(compression_desc)
                    .size(13.0)
                    .color(Color32::from_rgb(180, 180, 180)));
                
                // Show estimated size
                if let Some(estimated) = self.estimated_size {
                    let total_original: u64 = self.selected_files.iter()
                        .filter_map(|path| std::fs::metadata(path).ok())
                        .map(|m| m.len())
                        .sum();
                    
                    let reduction = if total_original > 0 {
                        ((total_original - estimated) as f64 / total_original as f64 * 100.0) as u64
                    } else {
                        0
                    };
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Estimated result:").size(14.0));
                        ui.label(RichText::new(format!(
                            "{} → {} (~{}% reduction)",
                            format_file_size(total_original),
                            format_file_size(estimated),
                            reduction
                        ))
                        .size(14.0)
                        .strong()
                        .color(Color32::from_rgb(100, 200, 255)));
                    });
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Processing section
            if !self.processing_progress.is_empty() {
                ui.label(RichText::new(&self.processing_progress).size(15.0).color(Color32::from_rgb(100, 200, 100)));
            }

            ui.horizontal(|ui| {
                let button = egui::Button::new(RichText::new("🗜️ Compress Files").size(16.0).strong());
                if ui.add_enabled(!self.is_processing && !self.selected_files.is_empty(), button).clicked() {
                    self.compress_files();
                }

                if self.is_processing {
                    ui.spinner();
                    ui.label(RichText::new("Processing...").size(15.0));
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Results section
            if !self.compression_results.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Compression Results:").size(18.0).strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let has_undownloaded = self.compression_results.iter()
                            .any(|r| r.success && !r.downloaded);
                        
                        if has_undownloaded {
                            if ui.button(RichText::new("📥 Download All").size(16.0).strong()).clicked() {
                                self.download_all();
                            }
                            
                            if ui.button(RichText::new("🗑️ Delete All").size(16.0))
                                .on_hover_text("Delete temporary files without downloading")
                                .clicked() 
                            {
                                self.cleanup_temp_files();
                                self.compression_results.clear();
                                self.selected_files.clear();
                            }
                        }
                    });
                });
                ui.add_space(10.0);

                ScrollArea::vertical()
                    .id_salt("results_scroll")
                    .show(ui, |ui| {
                    for (idx, result) in self.compression_results.iter().enumerate() {
                        ui.group(|ui| {
                            ui.set_min_width(ui.available_width());
                            
                            ui.horizontal(|ui| {
                                if result.success {
                                    ui.label(RichText::new("✅").size(18.0));
                                    ui.label(RichText::new(&result.file_name).size(15.0).strong());
                                } else {
                                    ui.label(RichText::new("❌").size(18.0));
                                    ui.label(RichText::new(&result.file_name).size(15.0).color(Color32::RED));
                                }
                            });

                            if result.success {
                                ui.add_space(5.0);
                                let reduction = if result.original_size > 0 && result.compressed_size < result.original_size {
                                    ((result.original_size - result.compressed_size) as f64 / result.original_size as f64 * 100.0) as u64
                                } else {
                                    0
                                };
                                
                                let color = if reduction > 10 { Color32::from_rgb(40, 200, 40) } 
                                           else if reduction > 0 { Color32::from_rgb(200, 200, 40) }
                                           else { Color32::from_rgb(200, 100, 40) };
                                
                                ui.label(RichText::new(format!(
                                    "Original: {}  →  Compressed: {}",
                                    format_file_size(result.original_size),
                                    format_file_size(result.compressed_size)
                                )).size(14.0));
                                
                                ui.label(RichText::new(format!("{}% reduction", reduction))
                                    .size(16.0)
                                    .strong()
                                    .color(color));

                                ui.horizontal(|ui| {
                                    if let Some(ref compressed_path) = result.compressed_path {
                                        if ui.button(RichText::new("👁 Preview").size(14.0)).clicked() {
                                            open_file(compressed_path);
                                        }
                                    }
                                    
                                    if result.downloaded {
                                        ui.label(RichText::new("✅ Downloaded").size(14.0).color(Color32::from_rgb(40, 200, 40)));
                                    }
                                });
                            } else if let Some(ref error) = result.error_message {
                                ui.label(RichText::new(error).size(14.0).color(Color32::RED));
                            }
                        });
                        
                        if idx < self.compression_results.len() - 1 {
                            ui.add_space(8.0);
                        }
                    }
                });
            }
        });
    }
}

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

fn open_file(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg(path)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .ok();
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    
    info!("PDF & Image Compressor starting...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("PDF & Image Compressor")
            .with_icon(std::sync::Arc::new(
                egui::IconData {
                    rgba: vec![],
                    width: 0,
                    height: 0,
                }
            )),
        ..Default::default()
    };

    eframe::run_native(
        "PDF & Image Compressor",
        options,
        Box::new(|_cc| {
            // Set up the app with context
            let app = PdfCompressor::new();
            Ok(Box::new(app))
        }),
    )
}
