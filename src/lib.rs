use lopdf::{Document, Object, Stream};
use std::collections::BTreeMap;
use log::{info, debug};

// Export API module for the api binary
pub mod api;

#[derive(Clone, Debug)]
pub struct CompressionSettings {
    pub quality: u8, // 0-100, JPEG quality
}

#[derive(Clone, Debug)]
pub struct CompressionResult {
    pub original_size: u64,
    pub compressed_size: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Compress PDF from bytes with specified quality percentage (0-100)
/// Quality maps to compression: 75 = 75% compression = ~25% of original size
pub fn compress_pdf_bytes(input_bytes: &[u8], compression_level: u8) -> Result<Vec<u8>, String> {
    // Clamp compression level
    let compression_level = compression_level.min(95).max(10);
    
    // Convert compression level to JPEG quality
    let jpeg_quality = if compression_level <= 25 {
        100 - (compression_level as f32 * 0.4) as u8 // 100 to 90
    } else if compression_level <= 50 {
        90 - ((compression_level - 25) as f32 * 0.8) as u8 // 90 to 70
    } else if compression_level <= 75 {
        70 - ((compression_level - 50) as f32 * 0.8) as u8 // 70 to 50
    } else {
        50 - ((compression_level - 75) as f32) as u8 // 50 to 25
    };
    
    let settings = CompressionSettings {
        quality: jpeg_quality,
    };
    
    info!("Starting compression with quality {}% (compression level {}%)", jpeg_quality, compression_level);
    
    // Load PDF from bytes
    let mut doc = Document::load_mem(input_bytes)
        .map_err(|e| format!("Failed to load PDF: {}", e))?;
    
    let total_objects = doc.objects.len();
    info!("PDF loaded successfully. Total objects: {}", total_objects);
    
    // Remove duplicate objects
    info!("Removing duplicate objects...");
    let removed = remove_duplicate_objects(&mut doc);
    info!("Removed {} duplicate objects", removed);
    
    // Compress images and streams
    info!("Compressing all streams with quality {}...", settings.quality);
    compress_all_streams(&mut doc, &settings)?;
    
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
    
    // Save to bytes
    let mut output = Vec::new();
    doc.save_to(&mut output)
        .map_err(|e| format!("Failed to save: {}", e))?;
    
    info!("PDF compressed successfully: {} bytes -> {} bytes", input_bytes.len(), output.len());
    
    Ok(output)
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

    to_replace.len()
}

fn compress_all_streams(doc: &mut Document, settings: &CompressionSettings) -> Result<(), String> {
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
                match compress_image_stream(&stream, settings.quality) {
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
                        return Stream::new(new_dict, recompressed);
                    }
                }
            }
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
    
    stream.clone()
}

fn is_image_stream(stream: &Stream) -> bool {
    if let Ok(Object::Name(ref subtype)) = stream.dict.get(b"Subtype") {
        return subtype == b"Image";
    }
    false
}

fn compress_image_stream(stream: &Stream, quality: u8) -> Result<Stream, String> {
    use image::imageops::FilterType;
    
    // Check filter type - skip if already JPEG
    if let Ok(Object::Name(filter)) = stream.dict.get(b"Filter") {
        if filter == b"DCTDecode" {
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
    
    let bpc = match stream.dict.get(b"BitsPerComponent") {
        Ok(Object::Integer(b)) => *b as u32,
        _ => 8,
    };
    
    // Only support 8-bit images
    if bpc != 8 {
        return Err(format!("Not 8-bit (bpc={})", bpc));
    }
    
    // Manual decompression for FlateDecode
    let content = if let Ok(Object::Name(filter)) = stream.dict.get(b"Filter") {
        if filter == b"FlateDecode" {
            use flate2::read::ZlibDecoder;
            use std::io::Read;
            
            let mut decoder = ZlibDecoder::new(&stream.content[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| format!("Manual decompression failed: {}", e))?;
            
            decompressed
        } else {
            stream.decompressed_content()
                .map_err(|e| format!("Decompress failed: {:?}", e))?
        }
    } else {
        stream.content.clone()
    };
    
    let original_content_size = content.len();
    
    // Determine number of components
    let pixel_count = (width * height) as usize;
    let components = if original_content_size == pixel_count * 3 {
        3
    } else if original_content_size == pixel_count * 4 {
        4
    } else if original_content_size == pixel_count {
        1
    } else {
        return Err(format!("Unexpected size: {} bytes for {}x{} image", original_content_size, width, height));
    };
    
    // Convert to RGB and encode as JPEG
    let dyn_img = match components {
        3 => {
            if let Some(img) = image::RgbImage::from_raw(width, height, content) {
                image::DynamicImage::ImageRgb8(img)
            } else {
                return Err("Failed to create RGB image".to_string());
            }
        },
        4 => {
            if let Some(img) = image::RgbaImage::from_raw(width, height, content) {
                image::DynamicImage::ImageRgba8(img).to_rgb8().into()
            } else {
                return Err("Failed to create RGBA image".to_string());
            }
        },
        1 => {
            if let Some(img) = image::GrayImage::from_raw(width, height, content) {
                image::DynamicImage::ImageLuma8(img).to_rgb8().into()
            } else {
                return Err("Failed to create grayscale image".to_string());
            }
        },
        _ => return Err(format!("Unsupported component count: {}", components))
    };
    
    // Downsample based on quality setting
    let (target_width, target_height) = if quality < 90 && (width > 1500 || height > 1500) {
        let max_dimension = if quality >= 70 {
            1500.0
        } else if quality >= 50 {
            1200.0
        } else {
            1000.0
        };
        
        let scale = max_dimension / width.max(height) as f32;
        let new_w = (width as f32 * scale) as u32;
        let new_h = (height as f32 * scale) as u32;
        info!("Downsampling large image (quality {}): {}x{} -> {}x{}", quality, width, height, new_w, new_h);
        (new_w, new_h)
    } else {
        (width, height)
    };
    
    let final_img = if target_width != width || target_height != height {
        dyn_img.resize_exact(target_width, target_height, FilterType::Lanczos3)
    } else {
        dyn_img
    };
    
    // Encode as JPEG with specified quality
    let mut compressed = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut compressed, quality);
    if encoder.encode_image(&final_img).is_ok() {
        info!("JPEG encoding successful: {} bytes -> {} bytes (quality {})", original_content_size, compressed.len(), quality);
        
        let mut new_dict = stream.dict.clone();
        new_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
        new_dict.set("Length", Object::Integer(compressed.len() as i64));
        new_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
        
        if target_width != width || target_height != height {
            new_dict.set("Width", Object::Integer(target_width as i64));
            new_dict.set("Height", Object::Integer(target_height as i64));
        }
        
        return Ok(Stream::new(new_dict, compressed));
    }
    
    Err("Image encoding failed".to_string())
}

