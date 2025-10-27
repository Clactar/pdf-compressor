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
        .route("/api/pdf", post(compress_pdf))
        .route("/health", axum::routing::get(health_check))
        .layer(middleware::from_fn(auth_middleware))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)); // 100 MB max
    
    // Bind to 0.0.0.0:3000 for container deployment
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    info!("Server listening on {}", addr);
    info!("Endpoints:");
    info!("  POST /api/pdf - Compress PDF (multipart/form-data) [Protected]");
    info!("  GET  /health  - Health check [Public]");
    
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
    
    // Skip authentication for health check
    if path == "/health" {
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

async fn compress_pdf(mut multipart: Multipart) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let mut pdf_data: Option<Vec<u8>> = None;
    let mut compression_level: u8 = 75; // Default 75%
    
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
            "file" | "pdf" => {
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
                
                pdf_data = Some(data.to_vec());
                info!("Received PDF file: {} bytes", data.len());
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
            _ => {
                // Ignore unknown fields
            }
        }
    }
    
    // Ensure we have PDF data
    let pdf_data = pdf_data.ok_or_else(|| {
        error!("No PDF file provided in request");
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No PDF file provided. Use 'file' or 'pdf' field name.".to_string(),
            }),
        )
    })?;
    
    let original_size = pdf_data.len() as u64;
    
    // Compress the PDF
    info!("Starting compression: {} bytes, level {}%", original_size, compression_level);
    
    let compressed_data = crate::compress_pdf_bytes(&pdf_data, compression_level).map_err(|e| {
        error!("Compression failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Compression failed: {}", e),
            }),
        )
    })?;
    
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
    
    // Return compressed PDF with metadata in headers
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "application/pdf"),
            ("Content-Disposition", "attachment; filename=\"compressed.pdf\""),
            ("X-Original-Size", &original_size.to_string()),
            ("X-Compressed-Size", &compressed_size.to_string()),
            ("X-Reduction-Percentage", &format!("{:.2}", reduction)),
        ],
        compressed_data,
    )
        .into_response())
}

