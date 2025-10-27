use axum::{
    extract::{DefaultBodyLimit, Multipart, Request},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    routing::post,
    Router, Json,
    middleware::{self, Next},
};
use serde::Serialize;
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use log::{info, error, warn};

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    info!("PDF Compressor API starting...");
    
    // Check if API key is configured
    let api_key = std::env::var("API_KEY").ok();
    if let Some(ref key) = api_key {
        info!("🔐 API Key authentication enabled");
        info!("   Key: {}***", &key.chars().take(8).collect::<String>());
    } else {
        warn!("⚠️  No API_KEY set - API is unprotected!");
        warn!("   Set API_KEY environment variable to enable authentication");
    }
    
    // Build application with routes
    let app = Router::new()
        .route("/api/compress", post(compress_file))
        .route("/api/pdf", post(compress_file)) // Legacy alias
        .route("/health", axum::routing::get(health_check))
        .route("/llm.txt", axum::routing::get(llm_docs))
        .layer(middleware::from_fn(auth_middleware))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)); // 100 MB max
    
    // Bind to 0.0.0.0:3000 for container deployment
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    info!("Server listening on {}", addr);
    info!("Endpoints:");
    info!("  POST /api/compress - Compress PDF or Image (multipart/form-data) [Protected]");
    info!("  POST /api/pdf     - Legacy alias for /api/compress [Protected]");
    info!("  GET  /health      - Health check [Public]");
    info!("  GET  /llm.txt     - LLM-optimized API documentation [Public]");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Authentication middleware
async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let path = request.uri().path();
    
    // Skip authentication for public endpoints
    if path == "/health" || path == "/llm.txt" {
        return Ok(next.run(request).await);
    }
    
    // Check if API key is configured
    let expected_key = match std::env::var("API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            // No API key configured - allow request (backward compatibility)
            return Ok(next.run(request).await);
        }
    };
    
    // Check for API key in headers
    let provided_key = headers
        .get("X-API-Key")
        .or_else(|| headers.get("Authorization"))
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Support both "Bearer <key>" and direct key
            if s.starts_with("Bearer ") {
                &s[7..]
            } else {
                s
            }
        });
    
    match provided_key {
        Some(key) if key == expected_key => {
            // Valid key
            Ok(next.run(request).await)
        }
        Some(_) => {
            // Invalid key
            warn!("🚫 Authentication failed: Invalid API key");
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid API key".to_string(),
                }),
            ))
        }
        None => {
            // No key provided
            warn!("🚫 Authentication failed: No API key provided");
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing API key. Provide X-API-Key header or Authorization: Bearer <key>".to_string(),
                }),
            ))
        }
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn llm_docs() -> (StatusCode, [(&'static str, &'static str); 1], &'static str) {
    const LLM_DOCS: &str = include_str!("../llm.txt");
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; charset=utf-8")],
        LLM_DOCS,
    )
}

async fn compress_file(mut multipart: Multipart) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut compression_level: u8 = 75; // Default 75%
    let mut output_format: Option<String> = None;
    
    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid multipart data: {}", e),
            }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "file" | "pdf" | "image" => {
                let data = field.bytes().await.map_err(|e| {
                    error!("Failed to read file data: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read file: {}", e),
                        }),
                    )
                })?;
                
                if data.is_empty() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Empty file".to_string(),
                        }),
                    ));
                }
                
                file_data = Some(data.to_vec());
                info!("Received file: {} bytes", data.len());
            }
            "compression" | "quality" | "level" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read compression parameter: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read compression parameter: {}", e),
                        }),
                    )
                })?;
                
                compression_level = text.parse::<u8>().unwrap_or(75).min(95).max(10);
                info!("Compression level set to: {}%", compression_level);
            }
            "output_format" | "format" => {
                let text = field.text().await.map_err(|e| {
                    error!("Failed to read output format: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Failed to read output format: {}", e),
                        }),
                    )
                })?;
                output_format = Some(text);
                info!("Output format set to: {:?}", output_format);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }
    
    // Ensure we have file data
    let file_data = file_data.ok_or_else(|| {
        error!("No file provided in request");
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No file provided. Use 'file', 'pdf', or 'image' field name.".to_string(),
            }),
        )
    })?;
    
    let original_size = file_data.len() as u64;
    
    // Detect file type using magic bytes
    let file_type = infer::get(&file_data);
    let is_pdf = file_type
        .as_ref()
        .map(|t| t.mime_type() == "application/pdf")
        .unwrap_or_else(|| {
            // Fallback: check PDF magic bytes
            file_data.starts_with(b"%PDF")
        });
    
    info!("Starting compression: {} bytes, level {}%, type: {}", 
          original_size, 
          compression_level,
          if is_pdf { "PDF" } else { "Image" });
    
    // Compress based on file type - offload CPU-intensive work to blocking thread pool
    let (compressed_data, content_type, filename) = if is_pdf {
        let compressed = tokio::task::spawn_blocking(move || {
            crate::compress_pdf_bytes(&file_data, compression_level)
        })
        .await
        .map_err(|e| {
            error!("PDF compression task failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("PDF compression task failed: {}", e),
                }),
            )
        })?
        .map_err(|e| {
            error!("PDF compression failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("PDF compression failed: {}", e),
                }),
            )
        })?;
        (compressed, "application/pdf", "compressed.pdf")
    } else {
        let (compressed, ext) = tokio::task::spawn_blocking(move || {
            crate::compress_image_bytes(
                &file_data,
                compression_level,
                output_format.as_deref(),
            )
        })
        .await
        .map_err(|e| {
            error!("Image compression task failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Image compression task failed: {}", e),
                }),
            )
        })?
        .map_err(|e| {
            error!("Image compression failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Image compression failed: {}", e),
                }),
            )
        })?;
        
        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            _ => "application/octet-stream",
        };
        let fname = format!("compressed.{}", ext);
        (compressed, mime, fname.leak() as &str)
    };
    
    let compressed_size = compressed_data.len() as u64;
    let reduction = if original_size > 0 {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };
    
    info!(
        "Compression successful: {} bytes -> {} bytes ({:.2}% reduction)",
        original_size, compressed_size, reduction
    );
    
    // Return compressed file with metadata in headers
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", content_type),
            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
            ("X-Original-Size", &original_size.to_string()),
            ("X-Compressed-Size", &compressed_size.to_string()),
            ("X-Reduction-Percentage", &format!("{:.2}", reduction)),
        ],
        compressed_data,
    )
        .into_response())
}

