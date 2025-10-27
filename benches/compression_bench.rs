use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use PDFcompressor::{compress_pdf_bytes, compress_image_bytes};
use lopdf::{Document, Object, Stream, Dictionary};
use image::{RgbImage, DynamicImage};

// Helper to generate test PDF
fn generate_test_pdf() -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    
    let pages_id = doc.new_object_id();
    let font_id = doc.new_object_id();
    let resources_id = doc.new_object_id();
    let content_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    
    let catalog_id = doc.add_object(
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Catalog".to_vec())),
            ("Pages", Object::Reference(pages_id)),
        ])
    );
    
    doc.trailer.set("Root", Object::Reference(catalog_id));
    
    doc.objects.insert(
        font_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Font".to_vec())),
            ("Subtype", Object::Name(b"Type1".to_vec())),
            ("BaseFont", Object::Name(b"Helvetica".to_vec())),
        ]).into()
    );
    
    let mut font_dict = Dictionary::new();
    font_dict.set("F1", Object::Reference(font_id));
    
    doc.objects.insert(
        resources_id,
        Dictionary::from_iter(vec![
            ("Font", font_dict.into()),
        ]).into()
    );
    
    let content = b"BT /F1 24 Tf 100 700 Td (Benchmark PDF) Tj ET";
    let mut content_dict = Dictionary::new();
    content_dict.set("Length", Object::Integer(content.len() as i64));
    
    doc.objects.insert(
        content_id,
        Object::Stream(Stream::new(content_dict, content.to_vec()))
    );
    
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
    
    doc.objects.insert(
        pages_id,
        Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Pages".to_vec())),
            ("Kids", Object::Array(vec![Object::Reference(page_id)])),
            ("Count", Object::Integer(1)),
        ]).into()
    );
    
    let mut output = Vec::new();
    doc.save_to(&mut output).unwrap();
    output
}

// Helper to generate test image
fn generate_test_image(width: u32, height: u32) -> Vec<u8> {
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
    dyn_img.write_to(&mut std::io::Cursor::new(&mut output), image::ImageFormat::Jpeg).unwrap();
    output
}

// ============================================================================
// PDF Compression Benchmarks
// ============================================================================

fn benchmark_pdf_compression_quality_levels(c: &mut Criterion) {
    let pdf_data = generate_test_pdf();
    let mut group = c.benchmark_group("pdf_compression_quality");
    
    for quality in [25, 50, 75, 90].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(quality),
            quality,
            |b, &quality| {
                b.iter(|| {
                    compress_pdf_bytes(black_box(&pdf_data), black_box(quality))
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_pdf_compression_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_compression_sizes");
    
    // Benchmark different PDF sizes (simulated by multiple operations)
    for size_multiplier in [1, 2, 4].iter() {
        let pdf_data = generate_test_pdf();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(size_multiplier),
            size_multiplier,
            |b, _| {
                b.iter(|| {
                    compress_pdf_bytes(black_box(&pdf_data), black_box(75))
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Image Compression Benchmarks
// ============================================================================

fn benchmark_image_compression_formats(c: &mut Criterion) {
    let jpeg_data = generate_test_image(500, 400);
    let mut group = c.benchmark_group("image_compression_formats");
    
    group.bench_function("jpeg_to_jpeg", |b| {
        b.iter(|| {
            compress_image_bytes(black_box(&jpeg_data), black_box(75), None)
        });
    });
    
    group.bench_function("jpeg_to_png", |b| {
        b.iter(|| {
            compress_image_bytes(black_box(&jpeg_data), black_box(75), Some("png"))
        });
    });
    
    group.finish();
}

fn benchmark_image_compression_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_compression_sizes");
    
    let sizes = [
        ("small", 200, 150),
        ("medium", 800, 600),
        ("large", 2000, 1500),
    ];
    
    for (name, width, height) in sizes.iter() {
        let image_data = generate_test_image(*width, *height);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &image_data,
            |b, data| {
                b.iter(|| {
                    compress_image_bytes(black_box(data), black_box(75), None)
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_image_compression_quality_levels(c: &mut Criterion) {
    let image_data = generate_test_image(800, 600);
    let mut group = c.benchmark_group("image_compression_quality");
    
    for quality in [25, 50, 75, 90].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(quality),
            quality,
            |b, &quality| {
                b.iter(|| {
                    compress_image_bytes(black_box(&image_data), black_box(quality), None)
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// Combined Benchmarks
// ============================================================================

fn benchmark_mixed_workload(c: &mut Criterion) {
    let pdf_data = generate_test_pdf();
    let image_data = generate_test_image(500, 400);
    
    let mut group = c.benchmark_group("mixed_workload");
    
    group.bench_function("pdf_then_image", |b| {
        b.iter(|| {
            let _ = compress_pdf_bytes(black_box(&pdf_data), black_box(75));
            let _ = compress_image_bytes(black_box(&image_data), black_box(75), None);
        });
    });
    
    group.finish();
}

// ============================================================================
// Throughput Benchmarks
// ============================================================================

fn benchmark_batch_compression(c: &mut Criterion) {
    let pdfs: Vec<Vec<u8>> = (0..10).map(|_| generate_test_pdf()).collect();
    
    let mut group = c.benchmark_group("batch_compression");
    group.sample_size(20); // Fewer samples for batch operations
    
    group.bench_function("batch_10_pdfs", |b| {
        b.iter(|| {
            for pdf in &pdfs {
                let _ = compress_pdf_bytes(black_box(pdf), black_box(75));
            }
        });
    });
    
    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    pdf_benches,
    benchmark_pdf_compression_quality_levels,
    benchmark_pdf_compression_sizes
);

criterion_group!(
    image_benches,
    benchmark_image_compression_formats,
    benchmark_image_compression_sizes,
    benchmark_image_compression_quality_levels
);

criterion_group!(
    combined_benches,
    benchmark_mixed_workload,
    benchmark_batch_compression
);

criterion_main!(pdf_benches, image_benches, combined_benches);

