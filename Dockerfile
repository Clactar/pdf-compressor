# Multi-stage build for smaller image size
FROM rust:1.82-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the API binary in release mode
RUN cargo build --bin pdfcompressor-api --release --no-default-features

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder
COPY --from=builder /app/target/release/pdfcompressor-api /usr/local/bin/pdfcompressor-api

# Create a non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# Expose port 3000
EXPOSE 3000

# Set environment variable for port (can be overridden)
ENV PORT=3000

# Run the API server
CMD ["pdfcompressor-api"]

