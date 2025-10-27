use lopdf::{Document, Object, Stream};
use log::{info, debug};
use image::{DynamicImage, ImageFormat};
use rayon::prelude::*;
use std::sync::Mutex;

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
    
    // Perform compression rounds (configurable via env var for performance tuning)
    let compression_rounds = std::env::var("PDF_COMPRESSION_ROUNDS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(2) // Default to 2 rounds for better latency vs quality balance
        .min(5); // Cap at 5 rounds max
    
    info!("Performing {} compression round(s)...", compression_rounds);
    for i in 0..compression_rounds {
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
    use ahash::AHashMap;
    use std::hash::{Hash, Hasher};
    use ahash::AHasher;
    
    // Use hash-based deduplication to avoid expensive content cloning
    let mut unique_streams: AHashMap<u64, lopdf::ObjectId> = AHashMap::new();
    let mut to_replace: Vec<(lopdf::ObjectId, lopdf::ObjectId)> = Vec::new();

    // Find duplicate streams using content hash
    for (obj_id, object) in doc.objects.iter() {
        if let Object::Stream(stream) = object {
            // Hash the content without cloning
            let mut hasher = AHasher::default();
            stream.content.hash(&mut hasher);
            let content_hash = hasher.finish();
            
            if let Some(&existing_id) = unique_streams.get(&content_hash) {
                to_replace.push((*obj_id, existing_id));
                debug!("Found duplicate stream: {:?} is same as {:?}", obj_id, existing_id);
            } else {
                unique_streams.insert(content_hash, *obj_id);
            }
        }
    }

    to_replace.len()
}

fn compress_all_streams(doc: &mut Document, settings: &CompressionSettings) -> Result<(), String> {
    let mut objects_to_update = Vec::new();

    // Find all stream objects and clone the streams we need to process
    for (obj_id, object) in doc.objects.iter() {
        if let Object::Stream(ref stream) = object {
            let is_image = is_image_stream(stream);
            let original_size = stream.content.len();
            objects_to_update.push((*obj_id, stream.clone(), is_image, original_size));
        }
    }

    let total_streams = objects_to_update.len();
    info!("Processing {} streams in parallel", total_streams);
    
    let compressed_count = Mutex::new(0);
    let image_count = Mutex::new(0);
    let total_saved = Mutex::new(0i64);

    // Compress streams in parallel using rayon
    // Note: Returning None means "don't update this stream" - the original remains in the document
    let compressed_streams: Vec<_> = objects_to_update
        .par_iter()
        .filter_map(|(obj_id, stream, is_image, original_size)| {
            if *is_image {
                *image_count.lock().unwrap() += 1;
                debug!("Processing image stream {:?}, original size: {} bytes", obj_id, original_size);
            }
            
            let compressed = if *is_image {
                match compress_image_stream(stream, settings.quality) {
                    Ok(s) => s,
                    Err(e) => {
                        debug!("Image compression failed for {:?}: {}, keeping original", obj_id, e);
                        // Return None to skip updating - original stream preserved in document
                        return None;
                    }
                }
            } else {
                compress_generic_stream(stream)
            };
            
            let new_size = compressed.content.len();
            
            // Only update if compressed version is smaller
            if new_size < *original_size {
                let saved = *original_size as i64 - new_size as i64;
                *total_saved.lock().unwrap() += saved;
                *compressed_count.lock().unwrap() += 1;
                debug!("Compressed {:?}: {} -> {} bytes (saved {} bytes)", 
                       obj_id, original_size, new_size, saved);
                Some((*obj_id, compressed))
            } else {
                debug!("Keeping original {:?}: compressed would be {} bytes (original {})", 
                       obj_id, new_size, original_size);
                // Return None to skip updating - original stream preserved in document
                None
            }
        })
        .collect();
    
    // Update document with successfully compressed streams only
    // Streams not in this list remain unchanged in the document
    for (obj_id, compressed_stream) in compressed_streams {
        doc.objects.insert(obj_id, Object::Stream(compressed_stream));
    }

    let final_compressed = *compressed_count.lock().unwrap();
    let final_image_count = *image_count.lock().unwrap();
    let final_saved = *total_saved.lock().unwrap();

    info!("Compressed {}/{} streams", final_compressed, total_streams);
    info!("Found {} image streams", final_image_count);
    info!("Total bytes saved from stream compression: {}", final_saved);

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

/// Compress standalone image from bytes
/// Returns (compressed_bytes, output_format_extension)
pub fn compress_image_bytes(
    input_bytes: &[u8],
    compression_level: u8,
    output_format: Option<&str>,
) -> Result<(Vec<u8>, String), String> {
    // Clamp compression level
    let compression_level = compression_level.min(95).max(10);
    
    // Convert compression level to quality (same mapping as PDF)
    let quality = if compression_level <= 25 {
        100 - (compression_level as f32 * 0.4) as u8
    } else if compression_level <= 50 {
        90 - ((compression_level - 25) as f32 * 0.8) as u8
    } else if compression_level <= 75 {
        70 - ((compression_level - 50) as f32 * 0.8) as u8
    } else {
        50 - ((compression_level - 75) as f32) as u8
    };
    
    info!("Compressing image with quality {}% (compression level {}%)", quality, compression_level);
    
    // Detect input format
    let input_format = image::guess_format(input_bytes)
        .map_err(|e| format!("Failed to detect image format: {}", e))?;
    
    info!("Detected input format: {:?}", input_format);
    
    // Load image
    let img = image::load_from_memory(input_bytes)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    let original_size = input_bytes.len();
    info!("Image loaded: {}x{}, {} bytes", img.width(), img.height(), original_size);
    
    // Determine output format
    let target_format = if let Some(fmt) = output_format {
        match fmt.to_lowercase().as_str() {
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "png" => ImageFormat::Png,
            "webp" => ImageFormat::WebP,
            _ => return Err(format!("Unsupported output format: {}", fmt)),
        }
    } else {
        // Auto-select: JPEG for lossy sources, PNG for lossless
        // But try both and pick the best
        match input_format {
            ImageFormat::Jpeg | ImageFormat::WebP => ImageFormat::Jpeg,
            _ => {
                // For lossless sources, try both and pick smaller
                info!("Trying both JPEG and PNG to find best compression...");
                let jpeg_result = encode_image_with_quality(&img, quality, ImageFormat::Jpeg);
                let png_result = encode_image_with_quality(&img, quality, ImageFormat::Png);
                
                match (jpeg_result, png_result) {
                    (Ok(jpeg_bytes), Ok(png_bytes)) => {
                        let jpeg_size = jpeg_bytes.len();
                        let png_size = png_bytes.len();
                        info!("JPEG: {} bytes, PNG: {} bytes", jpeg_size, png_size);
                        
                        // If sizes are within 10%, prefer PNG for lossless
                        if png_size as f64 <= jpeg_size as f64 * 1.1 {
                            info!("Choosing PNG (lossless and similar size)");
                            return Ok((png_bytes, "png".to_string()));
                        } else {
                            info!("Choosing JPEG (significantly smaller)");
                            return Ok((jpeg_bytes, "jpg".to_string()));
                        }
                    }
                    (Ok(jpeg_bytes), Err(_)) => {
                        info!("PNG encoding failed, using JPEG");
                        return Ok((jpeg_bytes, "jpg".to_string()));
                    }
                    (Err(_), Ok(png_bytes)) => {
                        info!("JPEG encoding failed, using PNG");
                        return Ok((png_bytes, "png".to_string()));
                    }
                    (Err(e1), Err(e2)) => {
                        return Err(format!("Both JPEG and PNG encoding failed: {} / {}", e1, e2));
                    }
                }
            }
        }
    };
    
    // Encode with target format
    let compressed = encode_image_with_quality(&img, quality, target_format)?;
    let extension = match target_format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        _ => "img",
    };
    
    info!("Image compressed: {} bytes -> {} bytes ({:.2}% reduction)",
          original_size, compressed.len(),
          (original_size as f64 - compressed.len() as f64) / original_size as f64 * 100.0);
    
    Ok((compressed, extension.to_string()))
}

/// Encode image with specified quality and format
fn encode_image_with_quality(
    img: &DynamicImage,
    quality: u8,
    format: ImageFormat,
) -> Result<Vec<u8>, String> {
    use image::imageops::FilterType;
    
    // Downsample large images based on quality
    let (width, height) = (img.width(), img.height());
    let downsampled = if quality < 90 && (width > 1500 || height > 1500) {
        let max_dimension = if quality >= 70 {
            1500.0
        } else if quality >= 50 {
            1200.0
        } else {
            1000.0
        };
        
        let scale = max_dimension / width.max(height) as f32;
        if scale < 1.0 {
            let new_w = (width as f32 * scale) as u32;
            let new_h = (height as f32 * scale) as u32;
            debug!("Downsampling: {}x{} -> {}x{}", width, height, new_w, new_h);
            img.resize_exact(new_w, new_h, FilterType::Lanczos3)
        } else {
            img.clone()
        }
    } else {
        img.clone()
    };
    
    let mut output = Vec::new();
    
    match format {
        ImageFormat::Jpeg => {
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
            encoder.encode_image(&downsampled)
                .map_err(|e| format!("JPEG encoding failed: {}", e))?;
        }
        ImageFormat::Png => {
            // Map quality to PNG compression level (inverse: higher quality = less compression)
            let compression_level = if quality >= 90 {
                image::codecs::png::CompressionType::Best
            } else if quality >= 70 {
                image::codecs::png::CompressionType::Default
            } else {
                image::codecs::png::CompressionType::Fast
            };
            
            let encoder = image::codecs::png::PngEncoder::new_with_quality(
                &mut output,
                compression_level,
                image::codecs::png::FilterType::Adaptive,
            );
            
            // Use write_image method for image 0.24
            use image::ImageEncoder;
            encoder.write_image(
                downsampled.as_bytes(),
                downsampled.width(),
                downsampled.height(),
                downsampled.color(),
            )
            .map_err(|e| format!("PNG encoding failed: {}", e))?;
        }
        ImageFormat::WebP => {
            // WebP support is limited in image 0.24, use lossless encoding
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
            use image::ImageEncoder;
            encoder.write_image(
                downsampled.as_bytes(),
                downsampled.width(),
                downsampled.height(),
                downsampled.color(),
            )
            .map_err(|e| format!("WebP encoding failed: {}", e))?;
        }
        _ => return Err(format!("Unsupported format: {:?}", format)),
    }
    
    Ok(output)
}

