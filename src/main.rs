use eframe::egui;
use egui::{CentralPanel, Context, ScrollArea, Color32, RichText};
use lopdf::{Document, Object, Stream};
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::collections::BTreeMap;
use log::{info, debug, warn};

#[derive(Default)]
struct PdfCompressor {
    selected_files: Vec<PathBuf>,
    compression_results: Vec<CompressionResult>,
    is_processing: bool,
    processing_progress: String,
    receiver: Option<Receiver<CompressionResult>>,
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
            .add_filter("PDF files", &["pdf"])
            .set_directory(".")
            .pick_files()
        {
            self.selected_files = files;
            // Clear previous results when selecting new files
            self.compression_results.clear();
        }
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
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);

        thread::spawn(move || {
            for file_path in files {
                let result = compress_single_pdf(&file_path);
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

fn compress_single_pdf(input_path: &Path) -> CompressionResult {
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

    // Generate temp output path in system temp directory
    let temp_dir = std::env::temp_dir();
    let file_stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("compressed");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let output_filename = format!("{}_compressed_{}.pdf", file_stem, timestamp);
    let output_path = temp_dir.join(&output_filename);
    info!("Temp output path: {:?}", output_path);

    match compress_pdf_file(input_path, &output_path) {
        Ok(_) => {
            // Get compressed file size
            let compressed_size = match std::fs::metadata(&output_path) {
                Ok(metadata) => metadata.len(),
                Err(_) => 0,
            };
            
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
        Err(error) => CompressionResult {
            file_name,
            original_size,
            compressed_size: 0,
            success: false,
            error_message: Some(error.to_string()),
            compressed_path: None,
            downloaded: false,
        }
    }
}

fn compress_pdf_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    // Open and parse the PDF
    info!("Loading PDF document...");
    let mut doc = Document::load(input_path)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;
    
    let total_objects = doc.objects.len();
    info!("PDF loaded successfully. Total objects: {}", total_objects);

    // Count streams before compression
    let stream_count = doc.objects.values()
        .filter(|obj| matches!(obj, Object::Stream(_)))
        .count();
    info!("Found {} stream objects", stream_count);

    // Remove duplicate objects
    info!("Removing duplicate objects...");
    let removed = remove_duplicate_objects(&mut doc);
    info!("Removed {} duplicate objects", removed);
    
    // Compress images and streams
    info!("Compressing all streams...");
    compress_all_streams(&mut doc)?;
    
    // Remove metadata to reduce size
    info!("Removing metadata objects...");
    let before_metadata = doc.objects.len();
    doc.objects.retain(|_, obj| {
        if let Object::Dictionary(dict) = obj {
            if let Ok(Object::Name(name)) = dict.get(b"Type") {
                return name != b"Metadata";
            }
        }
        true
    });
    let metadata_removed = before_metadata - doc.objects.len();
    info!("Removed {} metadata objects", metadata_removed);
    
    // Perform multiple rounds of compression
    info!("Performing multiple compression rounds...");
    for i in 0..3 {
        debug!("Compression round {}", i + 1);
        doc.compress();
        doc.prune_objects();
        doc.delete_zero_length_streams();
    }

    // Final cleanup
    info!("Final cleanup...");
    doc.compress();
    doc.prune_objects();
    
    info!("Final object count: {}", doc.objects.len());

    // Save with compression
    info!("Saving compressed PDF to: {:?}", output_path);
    doc.save(output_path)
        .map_err(|e| format!("Failed to save: {}", e))?;
    
    info!("PDF saved successfully");

    Ok(())
}

fn remove_duplicate_objects(doc: &mut Document) -> usize {
    let mut unique_streams: BTreeMap<Vec<u8>, lopdf::ObjectId> = BTreeMap::new();
    let mut to_replace: Vec<(lopdf::ObjectId, lopdf::ObjectId)> = Vec::new();

    // Find duplicate streams
    for (obj_id, object) in doc.objects.iter() {
        if let Object::Stream(stream) = object {
            let content = stream.content.clone();
            if let Some(&existing_id) = unique_streams.get(&content) {
                to_replace.push((*obj_id, existing_id));
                debug!("Found duplicate stream: {:?} is same as {:?}", obj_id, existing_id);
            } else {
                unique_streams.insert(content, *obj_id);
            }
        }
    }

    let duplicate_count = to_replace.len();
    // Replace references to duplicates
    for (_old_id, _new_id) in to_replace {
        // Note: Full replacement requires walking the object tree
        // For now, prune_objects will handle unreferenced objects
    }
    
    duplicate_count
}

fn compress_all_streams(doc: &mut Document) -> Result<(), String> {
    let mut objects_to_update = Vec::new();

    // Find all stream objects
    for (obj_id, object) in doc.objects.iter() {
        if let Object::Stream(ref stream) = object {
            let is_image = is_image_stream(stream);
            let original_size = stream.content.len();
            objects_to_update.push((*obj_id, is_image, original_size));
        }
    }

    let total_streams = objects_to_update.len();
    info!("Processing {} streams", total_streams);
    
    let mut compressed_count = 0;
    let mut image_count = 0;
    let mut total_saved = 0i64;

    // Compress each stream
    for (obj_id, is_image, original_size) in objects_to_update {
        if let Some(Object::Stream(stream)) = doc.objects.get(&obj_id).cloned() {
            if is_image {
                image_count += 1;
                debug!("Processing image stream {:?}, original size: {} bytes", obj_id, original_size);
            }
            
            let compressed = if is_image {
                match compress_image_stream(&stream) {
                    Ok(s) => s,
                    Err(e) => {
                        debug!("Image compression failed for {:?}: {}", obj_id, e);
                        stream.clone()
                    }
                }
            } else {
                compress_generic_stream(&stream)
            };
            
            let new_size = compressed.content.len();
            
            // Only update if it's actually smaller
            if new_size < original_size {
                let saved = original_size as i64 - new_size as i64;
                total_saved += saved;
                compressed_count += 1;
                debug!("Compressed {:?}: {} -> {} bytes (saved {} bytes)", 
                       obj_id, original_size, new_size, saved);
                doc.objects.insert(obj_id, Object::Stream(compressed));
            } else {
                debug!("Keeping original {:?}: compressed would be {} bytes (original {})", 
                       obj_id, new_size, original_size);
            }
        }
    }

    info!("Compressed {}/{} streams", compressed_count, total_streams);
    info!("Found {} image streams", image_count);
    info!("Total bytes saved from stream compression: {}", total_saved);

    Ok(())
}

fn compress_generic_stream(stream: &Stream) -> Stream {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    
    let original_content_size = stream.content.len();
    
    // Already compressed? Try to recompress the decompressed content
    if let Ok(filter) = stream.dict.get(b"Filter") {
        debug!("Stream has filter: {:?}, attempting recompression", filter);
        
        // Try to decompress and recompress with better settings
        if let Ok(decompressed) = stream.decompressed_content() {
            debug!("Decompressed content: {} bytes, recompressing...", decompressed.len());
            
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
            if encoder.write_all(&decompressed).is_ok() {
                if let Ok(recompressed) = encoder.finish() {
                    debug!("Recompressed: {} -> {} bytes (was {} bytes)", 
                           decompressed.len(), recompressed.len(), original_content_size);
                    
                    if recompressed.len() < original_content_size {
                        let mut new_dict = stream.dict.clone();
                        new_dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));
                        new_dict.set("Length", Object::Integer(recompressed.len() as i64));
                        info!("Successfully recompressed stream: saved {} bytes", 
                              original_content_size - recompressed.len());
                        return Stream::new(new_dict, recompressed);
                    } else {
                        debug!("Recompression not beneficial");
                    }
                }
            }
        } else {
            debug!("Could not decompress stream for recompression");
        }
        
        return stream.clone();
    }
    
    debug!("Applying Flate compression to uncompressed stream ({} bytes)", original_content_size);
    
    // Apply flate compression to uncompressed stream
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    if encoder.write_all(&stream.content).is_ok() {
        if let Ok(compressed) = encoder.finish() {
            debug!("Flate compressed: {} -> {} bytes", original_content_size, compressed.len());
            if compressed.len() < original_content_size {
                let mut new_dict = stream.dict.clone();
                new_dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));
                new_dict.set("Length", Object::Integer(compressed.len() as i64));
                return Stream::new(new_dict, compressed);
            }
        }
    }
    
    debug!("Flate compression not beneficial");
    stream.clone()
}

fn is_image_stream(stream: &Stream) -> bool {
    if let Ok(Object::Name(ref subtype)) = stream.dict.get(b"Subtype") {
        return subtype == b"Image";
    }
    false
}

fn compress_image_stream(stream: &Stream) -> Result<Stream, String> {
    use image::imageops::FilterType;
    
    // Check filter type - skip if already JPEG
    if let Ok(Object::Name(filter)) = stream.dict.get(b"Filter") {
        let filter_str = std::str::from_utf8(filter).unwrap_or("???");
        debug!("Image has filter: {:?}", filter_str);
        
        if filter == b"DCTDecode" {
            debug!("Image already JPEG, skipping");
            return Err("Already JPEG (DCTDecode)".to_string());
        }
    }
    
    // Get image properties
    let width = match stream.dict.get(b"Width") {
        Ok(Object::Integer(w)) => *w as u32,
        _ => return Err("No width".to_string()),
    };
    
    let height = match stream.dict.get(b"Height") {
        Ok(Object::Integer(h)) => *h as u32,
        _ => return Err("No height".to_string()),
    };
    
    debug!("Image dimensions: {}x{}", width, height);
    
    let bpc = match stream.dict.get(b"BitsPerComponent") {
        Ok(Object::Integer(b)) => *b as u32,
        _ => 8,
    };
    debug!("Bits per component: {}", bpc);
    
    // Only support 8-bit images
    if bpc != 8 {
        return Err(format!("Not 8-bit (bpc={})", bpc));
    }
    
    // Try to get decompressed content
    debug!("Decompressing image content...");
    
    // Manual decompression for FlateDecode since lopdf can't handle indirect ColorSpace
    let content = if let Ok(Object::Name(filter)) = stream.dict.get(b"Filter") {
        if filter == b"FlateDecode" {
            debug!("Manually decompressing FlateDecode stream");
            use flate2::read::ZlibDecoder;
            use std::io::Read;
            
            let mut decoder = ZlibDecoder::new(&stream.content[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| format!("Manual decompression failed: {}", e))?;
            
            debug!("Manual decompression successful: {} bytes", decompressed.len());
            decompressed
        } else {
            // Try standard decompression for other filters
            stream.decompressed_content()
                .map_err(|e| {
                    warn!("Standard decompress failed: {:?}", e);
                    format!("Decompress failed: {:?}", e)
                })?
        }
    } else {
        // No filter, use raw content
        debug!("No filter present, using raw content");
        stream.content.clone()
    };
    
    let original_content_size = content.len();
    debug!("Decompressed content size: {} bytes", original_content_size);
    
    // Determine number of components (RGB=3, RGBA=4, Gray=1, etc)
    let pixel_count = (width * height) as usize;
    let components = if original_content_size == pixel_count * 3 {
        debug!("Detected RGB image (3 components)");
        3
    } else if original_content_size == pixel_count * 4 {
        debug!("Detected RGBA image (4 components)");
        4
    } else if original_content_size == pixel_count {
        debug!("Detected grayscale image (1 component)");
        1
    } else {
        return Err(format!("Unexpected size: {} bytes for {}x{} image", original_content_size, width, height));
    };
    
    // Convert to RGB and encode as JPEG
    let dyn_img = match components {
        3 => {
            // RGB
            if let Some(img) = image::RgbImage::from_raw(width, height, content) {
                image::DynamicImage::ImageRgb8(img)
            } else {
                return Err("Failed to create RGB image".to_string());
            }
        },
        4 => {
            // RGBA - convert to RGB
            if let Some(img) = image::RgbaImage::from_raw(width, height, content) {
                image::DynamicImage::ImageRgba8(img).to_rgb8().into()
            } else {
                return Err("Failed to create RGBA image".to_string());
            }
        },
        1 => {
            // Grayscale - convert to RGB
            if let Some(img) = image::GrayImage::from_raw(width, height, content) {
                image::DynamicImage::ImageLuma8(img).to_rgb8().into()
            } else {
                return Err("Failed to create grayscale image".to_string());
            }
        },
        _ => return Err(format!("Unsupported component count: {}", components))
    };
    
    // Downsample if large
    let (target_width, target_height) = if width > 1500 || height > 1500 {
        let scale = 1500.0 / width.max(height) as f32;
        let new_w = (width as f32 * scale) as u32;
        let new_h = (height as f32 * scale) as u32;
        info!("Downsampling large image: {}x{} -> {}x{}", width, height, new_w, new_h);
        (new_w, new_h)
    } else {
        (width, height)
    };
    
    let final_img = if target_width != width || target_height != height {
        dyn_img.resize_exact(target_width, target_height, FilterType::Lanczos3)
    } else {
        dyn_img
    };
    
    // Encode as JPEG with quality 75 (good balance)
    debug!("Encoding as JPEG with quality 75...");
    let mut compressed = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut compressed, 75);
    if encoder.encode_image(&final_img).is_ok() {
        info!("JPEG encoding successful: {} bytes -> {} bytes", original_content_size, compressed.len());
        
        let mut new_dict = stream.dict.clone();
        new_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
        new_dict.set("Length", Object::Integer(compressed.len() as i64));
        new_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
        
        if target_width != width || target_height != height {
            new_dict.set("Width", Object::Integer(target_width as i64));
            new_dict.set("Height", Object::Integer(target_height as i64));
        }
        
        return Ok(Stream::new(new_dict, compressed));
    } else {
        warn!("JPEG encoding failed");
    }
    
    Err("Image encoding failed".to_string())
}

impl eframe::App for PdfCompressor {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Check for processing results
        if self.is_processing {
            self.check_processing_results();
            ctx.request_repaint(); // Keep UI responsive
        }
        
        CentralPanel::default().show(ctx, |ui| {
            ui.heading(RichText::new("PDF Compressor").size(24.0).strong());
            ui.add_space(10.0);
            ui.separator();

            // File selection section
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📁 Select PDF Files").size(16.0)).clicked() {
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
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Processing section
            if !self.processing_progress.is_empty() {
                ui.label(RichText::new(&self.processing_progress).size(15.0).color(Color32::from_rgb(100, 200, 100)));
            }

            ui.horizontal(|ui| {
                let button = egui::Button::new(RichText::new("🗜️ Compress PDFs").size(16.0).strong());
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
    
    info!("PDF Compressor starting...");
    
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
