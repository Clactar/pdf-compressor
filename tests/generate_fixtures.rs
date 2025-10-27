// This test generates fixture files for other tests to use
mod common;

#[test]
fn generate_test_fixtures() {
    use common::*;
    
    println!("Generating test fixtures...");
    
    // Generate PDFs
    let minimal_pdf = generate_minimal_pdf();
    save_fixture("minimal.pdf", &minimal_pdf);
    println!("✓ Generated minimal.pdf ({} bytes)", minimal_pdf.len());
    
    let pdf_with_image = generate_pdf_with_image();
    save_fixture("with_image.pdf", &pdf_with_image);
    println!("✓ Generated with_image.pdf ({} bytes)", pdf_with_image.len());
    
    // Generate images
    let small_jpeg = generate_jpeg_image(100, 100);
    save_fixture("small.jpg", &small_jpeg);
    println!("✓ Generated small.jpg ({} bytes)", small_jpeg.len());
    
    let large_jpeg = generate_jpeg_image(2000, 1500);
    save_fixture("large.jpg", &large_jpeg);
    println!("✓ Generated large.jpg ({} bytes)", large_jpeg.len());
    
    let small_png = generate_png_image(100, 100);
    save_fixture("small.png", &small_png);
    println!("✓ Generated small.png ({} bytes)", small_png.len());
    
    let large_png = generate_png_image(1800, 1200);
    save_fixture("large.png", &large_png);
    println!("✓ Generated large.png ({} bytes)", large_png.len());
    
    // Generate corrupted files
    let corrupted_pdf = generate_corrupted_pdf();
    save_fixture("corrupted.pdf", &corrupted_pdf);
    println!("✓ Generated corrupted.pdf ({} bytes)", corrupted_pdf.len());
    
    let corrupted_image = generate_corrupted_image();
    save_fixture("corrupted.jpg", &corrupted_image);
    println!("✓ Generated corrupted.jpg ({} bytes)", corrupted_image.len());
    
    // Generate empty file
    save_fixture("empty.bin", &[]);
    println!("✓ Generated empty.bin");
    
    println!("\n✅ All test fixtures generated successfully!");
}

