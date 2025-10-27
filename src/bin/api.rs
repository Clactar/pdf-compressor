// API server binary for PDF compression service
use PDFcompressor::api::run_server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Tokio runtime for CPU-bound + async I/O workload
    // More blocking threads for spawn_blocking tasks (compression work)
    let num_cpus = num_cpus::get();
    
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus)
        .max_blocking_threads(num_cpus * 2)
        .thread_name("pdfcompressor-worker")
        .enable_all()
        .build()?
        .block_on(async { run_server().await })
}

