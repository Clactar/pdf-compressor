# PDF Compressor API - Deployment Guide for Coolify

## Quick Start

Your PDF compression app has been transformed into a REST API service ready for deployment on Coolify.

### 🎯 What You Can Do Now

**Call the API with any PDF file:**

```bash
curl -X POST http://your-domain.com/api/pdf \
  -F "file=@document.pdf" \
  -F "compression=75" \
  -o compressed.pdf
```

The API will:

1. Accept your PDF file
2. Compress it using the specified level (default 75%)
3. Return the compressed PDF as binary
4. Include compression statistics in response headers

---

## 📁 Project Structure

```
PDFcompressor/
├── src/
│   ├── lib.rs           # Core compression logic (shared library)
│   ├── main.rs          # GUI application (original)
│   ├── api.rs           # REST API server logic
│   └── bin/
│       └── api.rs       # API binary entry point
├── Dockerfile           # Container image definition
├── docker-compose.yml   # Docker Compose configuration
├── .dockerignore        # Files to exclude from image
├── Cargo.toml           # Dependencies & binary configs
├── README-API.md        # API documentation
└── test-api.sh          # Test script
```

---

## 🚀 Deployment Options

### Option 1: Coolify with Docker Compose (Recommended)

**Steps:**

1. Push your code to GitHub
2. In Coolify:
   - Create new resource → **Docker Compose**
   - Connect your GitHub repo
   - Coolify auto-detects `docker-compose.yml`
   - Configure domain & environment variables
   - Deploy!

**Environment Variables in Coolify:**

```
PORT=3000
RUST_LOG=info
```

**Port Mapping:** `3000`

**Health Check URL:** `/health`

### Option 2: Coolify with Dockerfile

**Steps:**

1. In Coolify:
   - Create new resource → **Dockerfile**
   - Connect your repository
   - Dockerfile path: `Dockerfile`
   - Port: `3000`
   - Deploy!

---

## 🔧 Local Testing

### Run Locally

```bash
# Build and run
cargo run --bin pdfcompressor-api

# Or use release build
cargo build --bin pdfcompressor-api --release
./target/release/pdfcompressor-api
```

Server starts at `http://localhost:3000`

### Test with Script

```bash
# Test with a PDF file
./test-api.sh your-document.pdf 75

# The script will:
# - Check health endpoint
# - Compress the PDF
# - Show statistics
# - Save output as your-document_compressed.pdf
```

### Manual Test

```bash
# Health check
curl http://localhost:3000/health

# Compress PDF
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@test.pdf" \
  -F "compression=75" \
  -o compressed.pdf -v
```

---

## 🐳 Docker Testing

### Build and Test Locally

```bash
# Build image
docker build -t pdfcompressor-api .

# Run container
docker run -p 3000:3000 pdfcompressor-api

# Or use Docker Compose
docker-compose up
```

### Test the Container

```bash
# Wait for container to start, then:
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@test.pdf" \
  -o compressed.pdf
```

---

## 📡 API Reference

### Endpoints

#### `POST /api/pdf`

Compress a PDF file.

**Request:**

- Method: `POST`
- Content-Type: `multipart/form-data`
- Body:
  - `file` or `pdf` (required): PDF binary
  - `compression` or `quality` or `level` (optional): 10-95 (default: 75)

**Response:**

- Status: `200 OK`
- Content-Type: `application/pdf`
- Body: Compressed PDF binary
- Headers:
  - `X-Original-Size`: Original size in bytes
  - `X-Compressed-Size`: Compressed size in bytes
  - `X-Reduction-Percentage`: Reduction percentage

**Example:**

```bash
curl -X POST https://your-api.com/api/pdf \
  -F "file=@invoice.pdf" \
  -F "compression=80" \
  -o invoice_compressed.pdf
```

#### `GET /health`

Health check endpoint.

**Response:**

- Status: `200 OK`
- Body: `OK`

---

## 🎚️ Compression Levels

| Level | Description                               | Use Case                        |
| ----- | ----------------------------------------- | ------------------------------- |
| 10-25 | Low compression, best quality             | Documents with important images |
| 25-50 | Medium compression, good balance          | General purpose                 |
| 50-75 | High compression (default: 75)            | **Recommended for most cases**  |
| 75-95 | Maximum compression, visible quality loss | Large files, archival           |

---

## ⚙️ Configuration

### Environment Variables

| Variable   | Default | Description                                 |
| ---------- | ------- | ------------------------------------------- |
| `PORT`     | `3000`  | Server port                                 |
| `RUST_LOG` | `info`  | Log level (error, warn, info, debug, trace) |

### Resource Limits

**Minimum:**

- CPU: 1 core
- RAM: 512MB
- Storage: 500MB

**Recommended for Production:**

- CPU: 2 cores
- RAM: 2GB
- Storage: 1GB

**Adjust in `docker-compose.yml`:**

```yaml
deploy:
  resources:
    limits:
      cpus: "2"
      memory: 2G
```

### File Size Limits

Default max file size: **100MB**

To change, edit `src/api.rs`:

```rust
.layer(DefaultBodyLimit::max(200 * 1024 * 1024)); // 200MB
```

---

## 🔐 Security Considerations

### Current Implementation

- ✅ No file storage (all in-memory processing)
- ✅ Runs as non-root user in container
- ✅ Request size limits enforced
- ✅ CORS enabled (all origins)

### Production Recommendations

1. **Add Authentication:**

   - Use reverse proxy (nginx/Traefik) with basic auth
   - Or implement API key middleware

2. **Restrict CORS:**
   Edit `src/api.rs`:

   ```rust
   .layer(
       CorsLayer::new()
           .allow_origin("https://yourdomain.com".parse::<HeaderValue>()?)
           .allow_methods([Method::POST])
   )
   ```

3. **Rate Limiting:**

   - Use Coolify's built-in rate limiting
   - Or add tower-governor middleware

4. **HTTPS Only:**
   - Coolify handles SSL automatically
   - Ensure `X-Forwarded-Proto` is respected

---

## 🐛 Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs pdfcompressor-api

# Or in Coolify, view the deployment logs
```

**Common issues:**

- Port already in use → Change `PORT` env var
- Out of memory → Increase memory limit

### API Returns 500 Error

**Check logs for:**

- Invalid PDF format
- File too large (>100MB)
- Corrupted PDF

**Solution:** Ensure valid PDF input

### Slow Compression

**Causes:**

- Large PDF files (>50MB)
- Limited CPU allocation

**Solutions:**

- Increase CPU limit in docker-compose.yml
- Process large files asynchronously
- Use lower compression levels (faster)

### Memory Issues

**Symptoms:**

- Container crashes
- OOM (Out of Memory) errors

**Solutions:**

```yaml
# In docker-compose.yml
deploy:
  resources:
    limits:
      memory: 4G # Increase limit
```

---

## 📊 Monitoring

### Health Check

```bash
curl http://your-api.com/health
```

Returns `OK` if running properly.

**Coolify automatically:**

- Monitors health endpoint every 30s
- Restarts container if unhealthy
- Sends alerts on failures

### Logs

**View logs in Coolify:**

- Click on your service
- Go to "Logs" tab
- Real-time log streaming

**Log format:**

```
[INFO] PDF Compressor API starting...
[INFO] Server listening on 0.0.0.0:3000
[INFO] Starting compression with quality 50% (compression level 75%)
[INFO] Compressed 143/256 streams
[INFO] PDF compressed successfully: 5242880 bytes -> 1048576 bytes
```

---

## 🔄 Updates & Maintenance

### Update the API

1. Push changes to GitHub
2. In Coolify, click "Redeploy"
3. Coolify rebuilds and restarts automatically

### Rollback

Coolify supports instant rollback:

1. Go to deployment history
2. Click "Rollback" on previous version

---

## 💡 Integration Examples

### JavaScript/TypeScript

```typescript
async function compressPDF(file: File, compression = 75): Promise<Blob> {
  const formData = new FormData();
  formData.append("file", file);
  formData.append("compression", compression.toString());

  const response = await fetch("https://your-api.com/api/pdf", {
    method: "POST",
    body: formData,
  });

  if (!response.ok) {
    throw new Error("Compression failed");
  }

  const stats = {
    originalSize: response.headers.get("X-Original-Size"),
    compressedSize: response.headers.get("X-Compressed-Size"),
    reduction: response.headers.get("X-Reduction-Percentage"),
  };

  console.log("Compression stats:", stats);

  return await response.blob();
}
```

### Python

```python
import requests

def compress_pdf(input_path: str, output_path: str, compression: int = 75):
    with open(input_path, 'rb') as f:
        files = {'file': f}
        data = {'compression': compression}

        response = requests.post(
            'https://your-api.com/api/pdf',
            files=files,
            data=data
        )

        response.raise_for_status()

        with open(output_path, 'wb') as out:
            out.write(response.content)

        print(f"Original: {response.headers['X-Original-Size']} bytes")
        print(f"Compressed: {response.headers['X-Compressed-Size']} bytes")
        print(f"Reduction: {response.headers['X-Reduction-Percentage']}%")

# Usage
compress_pdf('large-document.pdf', 'compressed.pdf', compression=80)
```

### PHP

```php
<?php
function compressPDF($inputFile, $outputFile, $compression = 75) {
    $ch = curl_init('https://your-api.com/api/pdf');

    $cfile = new CURLFile($inputFile, 'application/pdf', 'file');

    curl_setopt_array($ch, [
        CURLOPT_POST => true,
        CURLOPT_POSTFIELDS => [
            'file' => $cfile,
            'compression' => $compression
        ],
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_HEADER => true
    ]);

    $response = curl_exec($ch);
    $headerSize = curl_getinfo($ch, CURLINFO_HEADER_SIZE);
    $headers = substr($response, 0, $headerSize);
    $body = substr($response, $headerSize);

    curl_close($ch);

    file_put_contents($outputFile, $body);

    // Parse headers for stats
    preg_match('/X-Reduction-Percentage: ([0-9.]+)/', $headers, $matches);
    echo "Reduction: {$matches[1]}%\n";
}

compressPDF('document.pdf', 'compressed.pdf', 75);
?>
```

---

## 📝 Next Steps

1. ✅ **Test locally** - Run and test the API on your machine
2. ✅ **Push to GitHub** - Commit and push your code
3. ✅ **Deploy to Coolify** - Follow deployment steps above
4. ✅ **Configure domain** - Set up your custom domain in Coolify
5. ✅ **Add monitoring** - Set up alerts in Coolify
6. ⚠️ **Secure the API** - Add authentication if needed
7. 🎉 **Use the API** - Start compressing PDFs!

---

## 📚 Additional Resources

- **Full API Docs:** See `README-API.md`
- **Test Script:** Use `./test-api.sh` for quick testing
- **Docker Compose:** See `docker-compose.yml` for configuration
- **Source Code:** Core logic in `src/lib.rs`, API in `src/api.rs`

---

## 🆘 Support

For issues or questions:

1. Check logs in Coolify
2. Review this guide
3. Test locally with `./test-api.sh`
4. Open an issue on GitHub

---

**Happy Compressing! 🎉**
