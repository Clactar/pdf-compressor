use eframe::egui;
use egui::{CentralPanel, Context, ScrollArea};
use lopdf::Document;
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

#[derive(Default)]
struct PdfCompressor {
    selected_files: Vec<PathBuf>,
    compression_results: Vec<CompressionResult>,
    is_processing: bool,
    processing_progress: String,
}

#[derive(Clone)]
struct CompressionResult {
    file_name: String,
    original_size: u64,
    compressed_size: u64,
    success: bool,
    error_message: Option<String>,
    compressed_path: Option<PathBuf>,
}

impl Default for CompressionResult {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            original_size: 0,
            compressed_size: 0,
            success: false,
            error_message: None,
            compressed_path: None,
        }
    }
}

impl PdfCompressor {
    fn new() -> Self {
        Self::default()
    }

    fn select_files(&mut self) {
        if let Some(files) = FileDialog::new()
            .add_filter("PDF files", &["pdf"])
            .set_directory(".")
            .pick_files()
        {
            self.selected_files = files;
        }
    }

    fn compress_files(&mut self) {
        if self.selected_files.is_empty() || self.is_processing {
            return;
        }

        self.is_processing = true;
        self.compression_results.clear();

        let files = self.selected_files.clone();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            for file_path in files {
                let result = compress_single_pdf(&file_path);
                tx.send(result).unwrap();
            }
        });

        // Handle results in the UI thread
        while let Ok(result) = rx.recv() {
            self.compression_results.push(result);
            self.processing_progress = format!(
                "Processed {}/{} files",
                self.compression_results.len(),
                self.selected_files.len()
            );
        }

        self.is_processing = false;
        self.processing_progress.clear();
    }
}

fn compress_single_pdf(input_path: &Path) -> CompressionResult {
    let file_name = input_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Get original file size
    let original_size = match input_path.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => 0,
    };

    // Generate output path
    let file_stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("compressed");
    let output_path = input_path.with_file_name(format!("{}_compressed.pdf", file_stem));

    match compress_pdf_file(input_path, &output_path) {
        Ok(_) => {
            // Get compressed file size
            let compressed_size = match std::fs::metadata(&output_path) {
                Ok(metadata) => metadata.len(),
                Err(_) => 0,
            };

            CompressionResult {
                file_name,
                original_size,
                compressed_size,
                success: true,
                error_message: None,
                compressed_path: Some(output_path),
            }
        }
        Err(error) => CompressionResult {
            file_name,
            original_size,
            compressed_size: 0,
            success: false,
            error_message: Some(error.to_string()),
            compressed_path: None,
        }
    }
}

fn compress_pdf_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    // Open and parse the PDF
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;

    // Perform multiple rounds of compression for better results
    for _ in 0..3 {
        doc.compress();
        doc.prune_objects();
        doc.delete_zero_length_streams();
    }

    // Final cleanup
    doc.compress();

    // Save the compressed PDF
    doc.save(output_path)
        .map_err(|e| format!("Failed to save compressed PDF: {}", e))?;

    Ok(())
}

impl eframe::App for PdfCompressor {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("PDF Compressor");
            ui.separator();

            // File selection section
            ui.horizontal(|ui| {
                if ui.button("Select PDF Files").clicked() {
                    self.select_files();
                }

                ui.label(format!("{} files selected", self.selected_files.len()));
            });

            if !self.selected_files.is_empty() {
                ui.separator();
                ui.label("Selected files:");

                ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for file in &self.selected_files {
                        ui.label(format!("📄 {}", file.file_name().unwrap_or_default().to_string_lossy()));
                    }
                });
            }

            ui.separator();

            // Processing section
            if !self.processing_progress.is_empty() {
                ui.label(&self.processing_progress);
            }

            ui.horizontal(|ui| {
                if ui.button("Compress PDFs").clicked() {
                    self.compress_files();
                }

                if self.is_processing {
                    ui.spinner();
                }
            });

            ui.separator();

            // Results section
            if !self.compression_results.is_empty() {
                ui.label("Compression Results:");

                ScrollArea::vertical().show(ui, |ui| {
                    for result in &self.compression_results {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                if result.success {
                                    ui.label("✅");
                                    ui.label(&result.file_name);
                                } else {
                                    ui.label("❌");
                                    ui.label(&result.file_name);
                                }
                            });

                            if result.success {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "Original: {} → Compressed: {} ({}% reduction)",
                                        format_file_size(result.original_size),
                                        format_file_size(result.compressed_size),
                                        if result.original_size > 0 {
                                            (result.original_size - result.compressed_size) * 100 / result.original_size
                                        } else {
                                            0
                                        }
                                    ));

                                    if let Some(ref compressed_path) = result.compressed_path {
                                        if ui.button("Open").clicked() {
                                            open_file(compressed_path);
                                        }
                                    }
                                });
                            } else if let Some(ref error) = result.error_message {
                                ui.colored_label(egui::Color32::RED, error);
                            }
                        });
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("PDF Compressor")
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
        "PDF Compressor",
        options,
        Box::new(|_cc| {
            // Set up the app with context
            let app = PdfCompressor::new();
            Ok(Box::new(app))
        }),
    )
}
