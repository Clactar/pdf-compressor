# PDF Compressor API

REST API service for compressing PDF files, built with Rust and Axum.

## Features

- 🚀 Fast PDF compression using Rust
- 🔧 Configurable compression levels (10-95%)
- 🐳 Docker & Docker Compose support
- ☁️ Ready for deployment on Coolify
- 📦 Binary response (compressed PDF)
- 📊 Compression statistics in response headers

## API Endpoints

### `POST /api/pdf`

Compress a PDF file.

**Request:**
- Content-Type: `multipart/form-data`
- Fields:
  - `file` or `pdf` (required): PDF file binary
  - `compression` or `quality` or `level` (optional): Compression level 10-95 (default: 75)

**Response:**
- Content-Type: `application/pdf`
- Body: Compressed PDF binary
- Headers:
  - `X-Original-Size`: Original file size in bytes
  - `X-Compressed-Size`: Compressed file size in bytes
  - `X-Reduction-Percentage`: Percentage reduction

**Example with curl:**

```bash
# Basic compression (75% default)
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@input.pdf" \
  -o compressed.pdf

# Custom compression level (90% = aggressive compression)
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@input.pdf" \
  -F "compression=90" \
  -o compressed.pdf

# View compression stats
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@input.pdf" \
  -D headers.txt \
  -o compressed.pdf && cat headers.txt
```

**Example with JavaScript/Fetch:**

```javascript
const formData = new FormData();
formData.append('file', pdfFile); // File object
formData.append('compression', 75);

const response = await fetch('http://localhost:3000/api/pdf', {
  method: 'POST',
  body: formData
});

const compressedPdf = await response.blob();
const originalSize = response.headers.get('X-Original-Size');
const compressedSize = response.headers.get('X-Compressed-Size');
const reduction = response.headers.get('X-Reduction-Percentage');

console.log(`Reduced by ${reduction}%: ${originalSize} → ${compressedSize} bytes`);
```

**Example with Python:**

```python
import requests

with open('input.pdf', 'rb') as f:
    files = {'file': f}
    data = {'compression': 75}
    
    response = requests.post(
        'http://localhost:3000/api/pdf',
        files=files,
        data=data
    )
    
    if response.status_code == 200:
        with open('compressed.pdf', 'wb') as out:
            out.write(response.content)
        
        print(f"Original: {response.headers['X-Original-Size']} bytes")
        print(f"Compressed: {response.headers['X-Compressed-Size']} bytes")
        print(f"Reduction: {response.headers['X-Reduction-Percentage']}%")
```

### `GET /health`

Health check endpoint.

**Response:**
- Status: 200 OK
- Body: `OK`

## Compression Levels

- **10-25%**: Low compression, best quality, larger files
- **25-50%**: Medium compression, good balance
- **50-75%**: High compression, good size reduction (default: **75%**)
- **75-95%**: Maximum compression, smallest files, visible quality loss

## Local Development

### Prerequisites

- Rust 1.82+ 
- Cargo

### Run Locally

```bash
# Build and run the API server
cargo run --bin pdfcompressor-api

# Or build release version
cargo build --bin pdfcompressor-api --release
./target/release/pdfcompressor-api
```

The server will start on `http://localhost:3000` by default.

### Environment Variables

- `PORT`: Server port (default: 3000)
- `RUST_LOG`: Log level (default: info)

## Docker Deployment

### Build and Run with Docker

```bash
# Build the image
docker build -t pdfcompressor-api .

# Run the container
docker run -p 3000:3000 pdfcompressor-api
```

### Using Docker Compose

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the service
docker-compose down
```

## Coolify Deployment

### Method 1: GitHub Repository

1. Push your code to a GitHub repository
2. In Coolify, create a new resource → Docker Compose
3. Connect your GitHub repository
4. Coolify will auto-detect the `docker-compose.yml`
5. Set environment variables if needed:
   - `PORT=3000`
   - `RUST_LOG=info`
6. Deploy!

### Method 2: Dockerfile

1. In Coolify, create a new resource → Dockerfile
2. Connect your repository
3. Point to the `Dockerfile`
4. Configure port mapping: 3000
5. Deploy!

### Coolify Configuration

**Dockerfile path:** `Dockerfile`  
**Port:** `3000`  
**Health check:** `http://localhost:3000/health`

**Environment Variables:**
```
PORT=3000
RUST_LOG=info
```

**Buildpacks:** Not needed (native Dockerfile)

## Resource Requirements

**Minimum:**
- CPU: 1 core
- RAM: 512MB
- Storage: 500MB

**Recommended:**
- CPU: 2 cores
- RAM: 2GB
- Storage: 1GB

## Performance

- Processing speed depends on PDF size and complexity
- Typical 5MB PDF: ~1-3 seconds
- Maximum file size: 100MB (configurable in code)
- Concurrent requests supported via Tokio async runtime

## Security Considerations

- No file storage (all processing in memory)
- Request size limited to 100MB by default
- CORS enabled (configure in production)
- Runs as non-root user in container
- No authentication by default (add reverse proxy with auth if needed)

## Troubleshooting

**Container won't start:**
```bash
docker-compose logs pdfcompressor-api
```

**Out of memory:**
- Increase container memory limit in `docker-compose.yml`
- Reduce concurrent requests
- Lower max file size in `src/api.rs` (DefaultBodyLimit)

**Slow compression:**
- Check CPU allocation
- Increase `deploy.resources.limits.cpus` in docker-compose.yml

## GUI Version

The original GUI application is still available:

```bash
cargo run --bin pdfcompressor-gui --features gui
```

## License

[Your License Here]

## Support

For issues or questions, please open an issue on GitHub.

