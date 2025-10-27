use lopdf::{Document, Object, Stream, Dictionary};
use image::{RgbImage, DynamicImage, ImageFormat};
use std::io::Write;

/// Generate a minimal valid PDF for testing
pub fn generate_minimal_pdf() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    
    // Create a simple page with text
    let pages_id = doc.new_object_id();
    let font_id = doc.new_object_id();
    let resources_id = doc.new_object_id();
    let content_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    
    // Catalog
    let catalog_id = doc.add_object(
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Catalog".to_vec())),
            ("Pages", Object::Reference(pages_id)),
        ])
    );
    
    doc.trailer.set("Root", Object::Reference(catalog_id));
    
    // Font
    doc.objects.insert(
        font_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Font".to_vec())),
            ("Subtype", Object::Name(b"Type1".to_vec())),
            ("BaseFont", Object::Name(b"Helvetica".to_vec())),
        ]).into()
    );
    
    // Resources
    let mut font_dict = Dictionary::new();
    font_dict.set("F1", Object::Reference(font_id));
    
    doc.objects.insert(
        resources_id,
        Dictionary::from_iter(vec![
            ("Font", font_dict.into()),
        ]).into()
    );
    
    // Content stream
    let content = b"BT /F1 24 Tf 100 700 Td (Test PDF) Tj ET";
    let mut content_dict = Dictionary::new();
    content_dict.set("Length", Object::Integer(content.len() as i64));
    
    doc.objects.insert(
        content_id,
        Object::Stream(Stream::new(content_dict, content.to_vec()))
    );
    
    // Page
    doc.objects.insert(
        page_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Page".to_vec())),
            ("Parent", Object::Reference(pages_id)),
            ("Resources", Object::Reference(resources_id)),
            ("MediaBox", Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ])),
            ("Contents", Object::Reference(content_id)),
        ]).into()
    );
    
    // Pages
    doc.objects.insert(
        pages_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Pages".to_vec())),
            ("Kids", Object::Array(vec![Object::Reference(page_id)])),
            ("Count", Object::Integer(1)),
        ]).into()
    );
    
    let mut output = Vec::new();
    doc.save_to(&mut output).expect("Failed to save PDF");
    output
}

/// Generate a PDF with an embedded image
pub fn generate_pdf_with_image() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    
    // Create a simple page with an image
    let pages_id = doc.new_object_id();
    let resources_id = doc.new_object_id();
    let content_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    let image_id = doc.new_object_id();
    
    // Catalog
    let catalog_id = doc.add_object(
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Catalog".to_vec())),
            ("Pages", Object::Reference(pages_id)),
        ])
    );
    
    doc.trailer.set("Root", Object::Reference(catalog_id));
    
    // Create a small RGB image (10x10 red square)
    let width = 10u32;
    let height = 10u32;
    let mut img_data = Vec::new();
    for _ in 0..(width * height) {
        img_data.extend_from_slice(&[255u8, 0, 0]); // Red pixels
    }
    
    // Image XObject
    let mut image_dict = Dictionary::new();
    image_dict.set("Type", Object::Name(b"XObject".to_vec()));
    image_dict.set("Subtype", Object::Name(b"Image".to_vec()));
    image_dict.set("Width", Object::Integer(width as i64));
    image_dict.set("Height", Object::Integer(height as i64));
    image_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    image_dict.set("BitsPerComponent", Object::Integer(8));
    image_dict.set("Length", Object::Integer(img_data.len() as i64));
    
    doc.objects.insert(
        image_id,
        Object::Stream(Stream::new(image_dict, img_data))
    );
    
    // Resources
    let mut xobject_dict = Dictionary::new();
    xobject_dict.set("Im1", Object::Reference(image_id));
    
    doc.objects.insert(
        resources_id,
        Dictionary::from_iter(vec![
            ("XObject", xobject_dict.into()),
        ]).into()
    );
    
    // Content stream (draw the image)
    let content = b"q 100 0 0 100 50 650 cm /Im1 Do Q";
    let mut content_dict = Dictionary::new();
    content_dict.set("Length", Object::Integer(content.len() as i64));
    
    doc.objects.insert(
        content_id,
        Object::Stream(Stream::new(content_dict, content.to_vec()))
    );
    
    // Page
    doc.objects.insert(
        page_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Page".to_vec())),
            ("Parent", Object::Reference(pages_id)),
            ("Resources", Object::Reference(resources_id)),
            ("MediaBox", Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ])),
            ("Contents", Object::Reference(content_id)),
        ]).into()
    );
    
    // Pages
    doc.objects.insert(
        pages_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Pages".to_vec())),
            ("Kids", Object::Array(vec![Object::Reference(page_id)])),
            ("Count", Object::Integer(1)),
        ]).into()
    );
    
    let mut output = Vec::new();
    doc.save_to(&mut output).expect("Failed to save PDF with image");
    output
}

/// Generate a test JPEG image
pub fn generate_jpeg_image(width: u32, height: u32) -> Vec<u8> {
    // Create a gradient image
    let mut img = RgbImage::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = 128;
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
    
    let mut output = Vec::new();
    let dyn_img = DynamicImage::ImageRgb8(img);
    dyn_img.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Jpeg)
        .expect("Failed to write JPEG");
    output
}

/// Generate a test PNG image
pub fn generate_png_image(width: u32, height: u32) -> Vec<u8> {
    // Create a checkerboard pattern
    let mut img = RgbImage::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let is_white = ((x / 10) + (y / 10)) % 2 == 0;
            let color = if is_white { 255 } else { 0 };
            img.put_pixel(x, y, image::Rgb([color, color, color]));
        }
    }
    
    let mut output = Vec::new();
    let dyn_img = DynamicImage::ImageRgb8(img);
    dyn_img.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)
        .expect("Failed to write PNG");
    output
}

/// Generate invalid/corrupted data for error testing
pub fn generate_corrupted_pdf() -> Vec<u8> {
    b"%PDF-1.5\n%\xE2\xE3\xCF\xD3\nThis is not a valid PDF\n%%EOF".to_vec()
}

/// Generate corrupted image data
pub fn generate_corrupted_image() -> Vec<u8> {
    b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x00CORRUPTED".to_vec()
}

/// Helper to save test fixtures to disk
pub fn save_fixture(name: &str, data: &[u8]) -> std::path::PathBuf {
    let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    
    std::fs::create_dir_all(&fixture_dir).expect("Failed to create fixtures directory");
    
    let path = fixture_dir.join(name);
    let mut file = std::fs::File::create(&path).expect("Failed to create fixture file");
    file.write_all(data).expect("Failed to write fixture");
    path
}

/// Load a test fixture from disk
pub fn load_fixture(name: &str) -> Vec<u8> {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    
    std::fs::read(&fixture_path).expect(&format!("Failed to load fixture: {}", name))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_minimal_pdf() {
        let pdf = generate_minimal_pdf();
        assert!(pdf.len() > 100);
        assert!(pdf.starts_with(b"%PDF"));
    }
    
    #[test]
    fn test_generate_pdf_with_image() {
        let pdf = generate_pdf_with_image();
        assert!(pdf.len() > 200);
        assert!(pdf.starts_with(b"%PDF"));
    }
    
    #[test]
    fn test_generate_jpeg_image() {
        let img = generate_jpeg_image(100, 100);
        assert!(img.len() > 100);
        assert!(img.starts_with(&[0xFF, 0xD8])); // JPEG magic bytes
    }
    
    #[test]
    fn test_generate_png_image() {
        let img = generate_png_image(100, 100);
        assert!(img.len() > 100);
        assert!(img.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG magic bytes
    }
}

