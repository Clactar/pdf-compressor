// API server binary for PDF compression service
use PDFcompressor::api::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_server().await
}

