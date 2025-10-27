# API Reference

Complete reference for the PDF & Image Compressor API.

---

## Authentication

The API uses API key authentication. Include your API key in the request header:

```bash
X-API-Key: your_api_key_here
```

Or using Bearer token format:

```bash
Authorization: Bearer your_api_key_here
```

> **Note:** If no `API_KEY` environment variable is set on the server, authentication is disabled for backward compatibility.

---

## Endpoints

### Compress File

Compress a PDF or image file with configurable quality settings.

**Endpoint:** `POST /api/compress`

**Legacy Alias:** `POST /api/pdf` _(deprecated, use `/api/compress` instead)_

#### Request

**Content-Type:** `multipart/form-data`

#### Parameters

| Parameter         | Type    | Required | Default                       | Description                                                                                                                     |
| ----------------- | ------- | -------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `file`            | file    | **Yes**  | —                             | The PDF or image file to compress. Accepted formats: PDF, JPEG, PNG, WebP, TIFF                                                 |
| `compression`     | integer | No       | `75`                          | Compression level (10-95). Higher values = more compression. Maps to quality inversely.                                         |
| `output_format`   | string  | No       | `auto`                        | Output format for images. Options: `jpeg`, `png`, `webp`, `auto`. PDF files ignore this parameter.                              |
| `output_filename` | string  | No       | `{original-filename}-compressed` | Custom name for output file (extension auto-appended). Only alphanumeric, hyphens, underscores, and spaces allowed. Max 255 characters. |

**Alternative parameter names:**

- `file` can also be `pdf` or `image`
- `compression` can also be `quality` or `level`
- `output_format` can also be `format`
- `output_filename` can also be `filename`

#### Compression Levels

| Level | Quality Range                                | Use Case                                      |
| ----- | -------------------------------------------- | --------------------------------------------- |
| 10-25 | Minimal compression<br/>JPEG quality: 90-100 | High quality, minimal size reduction          |
| 25-50 | Moderate compression<br/>JPEG quality: 70-90 | Good balance between quality and size         |
| 50-75 | High compression<br/>JPEG quality: 50-70     | Noticeable size reduction, acceptable quality |
| 75-95 | Maximum compression<br/>JPEG quality: 25-50  | Smallest files, visible quality loss          |

> **Default:** `75` (recommended for most use cases)

#### Output Format (Images Only)

When compressing images, the API can automatically select the best output format or use your specified format:

| Value          | Behavior                                                                                                                                             |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `auto`         | **Smart selection:** For JPEG/WebP sources → JPEG output. For PNG/TIFF → compares JPEG vs PNG, chooses best ratio (if within 10% → PNG for lossless) |
| `jpeg` / `jpg` | Force JPEG output (lossy compression)                                                                                                                |
| `png`          | Force PNG output (lossless compression)                                                                                                              |
| `webp`         | Force WebP output (lossless in v0.24)                                                                                                                |

> **Note:** PDF files always output as PDF regardless of this parameter.

#### Request Example

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "X-API-Key: your_api_key_here" \
  -F "file=@document.pdf" \
  -F "compression=75"
```

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "X-API-Key: your_api_key_here" \
  -F "file=@photo.png" \
  -F "compression=60" \
  -F "output_format=jpeg"
```

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "X-API-Key: your_api_key_here" \
  -F "file=@invoice.pdf" \
  -F "compression=70" \
  -F "output_filename=my-invoice" \
  -o my-invoice.pdf
```

#### Response

**Success Response**

**Status Code:** `200 OK`

**Content-Type:**

- `application/pdf` for PDF files
- `image/jpeg` for JPEG images
- `image/png` for PNG images
- `image/webp` for WebP images

**Headers:**

| Header                   | Type    | Description                          |
| ------------------------ | ------- | ------------------------------------ |
| `Content-Type`           | string  | MIME type of the compressed file     |
| `Content-Disposition`    | string  | Attachment with suggested filename   |
| `X-Original-Size`        | integer | Original file size in bytes          |
| `X-Compressed-Size`      | integer | Compressed file size in bytes        |
| `X-Reduction-Percentage` | float   | Percentage reduction (e.g., `67.45`) |

**Body:** Binary data of the compressed file

**Example Response Headers:**

```http
HTTP/1.1 200 OK
Content-Type: application/pdf
Content-Disposition: attachment; filename="invoice-compressed.pdf"
X-Original-Size: 2457600
X-Compressed-Size: 614400
X-Reduction-Percentage: 75.00
```

When `output_filename` is provided:

```http
HTTP/1.1 200 OK
Content-Type: application/pdf
Content-Disposition: attachment; filename="my-invoice.pdf"
X-Original-Size: 2457600
X-Compressed-Size: 614400
X-Reduction-Percentage: 75.00
```

#### Error Responses

**Authentication Error**

**Status Code:** `401 Unauthorized`

```json
{
  "error": "Invalid API key"
}
```

```json
{
  "error": "Missing API key. Provide X-API-Key header or Authorization: Bearer <key>"
}
```

**Validation Error**

**Status Code:** `400 Bad Request`

```json
{
  "error": "No file provided. Use 'file', 'pdf', or 'image' field name."
}
```

```json
{
  "error": "Empty file"
}
```

```json
{
  "error": "Invalid multipart data: <details>"
}
```

```json
{
  "error": "Invalid output filename: only alphanumeric, hyphens, underscores, and spaces allowed"
}
```

```json
{
  "error": "Invalid output filename: maximum 255 characters allowed"
}
```

**Processing Error**

**Status Code:** `500 Internal Server Error`

```json
{
  "error": "PDF compression failed: <details>"
}
```

```json
{
  "error": "Image compression failed: <details>"
}
```

---

### Health Check

Check if the API server is running.

**Endpoint:** `GET /health`

**Authentication:** None required (public endpoint)

#### Request Example

```bash
curl https://your-domain.com/health
```

#### Response

**Status Code:** `200 OK`

**Content-Type:** `text/plain`

**Body:** `OK`

---

### LLM Documentation

Get LLM-optimized API documentation in plain text format.

**Endpoint:** `GET /llm.txt`

**Authentication:** None required (public endpoint)

#### Request Example

```bash
curl https://your-domain.com/llm.txt
```

#### Response

**Status Code:** `200 OK`

**Content-Type:** `text/plain; charset=utf-8`

**Body:** Complete API documentation formatted for LLM consumption, including:

- Service overview
- Authentication details
- All endpoints with parameters
- Request/response formats
- Compression algorithms
- Best practices for LLM agents
- Common edge cases
- Example code

---

## Code Examples

### cURL

**Compress a PDF:**

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "X-API-Key: sk_live_abc123..." \
  -F "file=@invoice.pdf" \
  -F "compression=80" \
  -o compressed_invoice.pdf
```

**Compress an image with format selection:**

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "Authorization: Bearer sk_live_abc123..." \
  -F "file=@photo.jpg" \
  -F "compression=70" \
  -F "output_format=png" \
  -o compressed_photo.png
```

**Compress with custom output filename:**

```bash
curl -X POST https://your-domain.com/api/compress \
  -H "X-API-Key: sk_live_abc123..." \
  -F "file=@report.pdf" \
  -F "compression=75" \
  -F "output_filename=quarterly-report-2025" \
  -o quarterly-report-2025.pdf
```

### JavaScript (Node.js)

```javascript
const FormData = require("form-data");
const fs = require("fs");
const axios = require("axios");

const form = new FormData();
form.append("file", fs.createReadStream("document.pdf"));
form.append("compression", "75");

axios
  .post("https://your-domain.com/api/compress", form, {
    headers: {
      ...form.getHeaders(),
      "X-API-Key": "sk_live_abc123...",
    },
    responseType: "arraybuffer",
  })
  .then((response) => {
    // Save compressed file
    fs.writeFileSync("compressed.pdf", response.data);

    // Access metadata
    console.log("Original size:", response.headers["x-original-size"]);
    console.log("Compressed size:", response.headers["x-compressed-size"]);
    console.log("Reduction:", response.headers["x-reduction-percentage"] + "%");
  })
  .catch((error) => {
    console.error("Error:", error.response?.data || error.message);
  });
```

### Python

```python
import requests

url = 'https://your-domain.com/api/compress'
headers = {'X-API-Key': 'sk_live_abc123...'}

with open('document.pdf', 'rb') as f:
    files = {'file': f}
    data = {'compression': 75}

    response = requests.post(url, headers=headers, files=files, data=data)

if response.status_code == 200:
    # Save compressed file
    with open('compressed.pdf', 'wb') as f:
        f.write(response.content)

    # Access metadata
    print(f"Original size: {response.headers['X-Original-Size']} bytes")
    print(f"Compressed size: {response.headers['X-Compressed-Size']} bytes")
    print(f"Reduction: {response.headers['X-Reduction-Percentage']}%")
else:
    print(f"Error: {response.json()['error']}")
```

### PHP

```php
<?php

$url = 'https://your-domain.com/api/compress';
$apiKey = 'sk_live_abc123...';

$file = new CURLFile('document.pdf', 'application/pdf', 'document.pdf');
$data = [
    'file' => $file,
    'compression' => 75
];

$ch = curl_init();
curl_setopt($ch, CURLOPT_URL, $url);
curl_setopt($ch, CURLOPT_POST, true);
curl_setopt($ch, CURLOPT_POSTFIELDS, $data);
curl_setopt($ch, CURLOPT_HTTPHEADER, [
    'X-API-Key: ' . $apiKey
]);
curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
curl_setopt($ch, CURLOPT_HEADER, true);

$response = curl_exec($ch);
$headerSize = curl_getinfo($ch, CURLINFO_HEADER_SIZE);
$headers = substr($response, 0, $headerSize);
$body = substr($response, $headerSize);

curl_close($ch);

// Save compressed file
file_put_contents('compressed.pdf', $body);

// Parse and display metadata
preg_match('/X-Original-Size: (\d+)/', $headers, $originalSize);
preg_match('/X-Compressed-Size: (\d+)/', $headers, $compressedSize);
preg_match('/X-Reduction-Percentage: ([\d.]+)/', $headers, $reduction);

echo "Original size: {$originalSize[1]} bytes\n";
echo "Compressed size: {$compressedSize[1]} bytes\n";
echo "Reduction: {$reduction[1]}%\n";
```

### Go

```go
package main

import (
    "bytes"
    "fmt"
    "io"
    "mime/multipart"
    "net/http"
    "os"
)

func main() {
    url := "https://your-domain.com/api/compress"
    apiKey := "sk_live_abc123..."

    // Open file
    file, err := os.Open("document.pdf")
    if err != nil {
        panic(err)
    }
    defer file.Close()

    // Create multipart form
    body := &bytes.Buffer{}
    writer := multipart.NewWriter(body)

    part, err := writer.CreateFormFile("file", "document.pdf")
    if err != nil {
        panic(err)
    }
    io.Copy(part, file)

    writer.WriteField("compression", "75")
    writer.Close()

    // Create request
    req, err := http.NewRequest("POST", url, body)
    if err != nil {
        panic(err)
    }

    req.Header.Set("Content-Type", writer.FormDataContentType())
    req.Header.Set("X-API-Key", apiKey)

    // Send request
    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil {
        panic(err)
    }
    defer resp.Body.Close()

    // Save compressed file
    out, err := os.Create("compressed.pdf")
    if err != nil {
        panic(err)
    }
    defer out.Close()

    io.Copy(out, resp.Body)

    // Display metadata
    fmt.Printf("Original size: %s bytes\n", resp.Header.Get("X-Original-Size"))
    fmt.Printf("Compressed size: %s bytes\n", resp.Header.Get("X-Compressed-Size"))
    fmt.Printf("Reduction: %s%%\n", resp.Header.Get("X-Reduction-Percentage"))
}
```

---

## Rate Limiting

Currently, there are no rate limits enforced by the API. However, we recommend:

- Maximum file size: **100 MB**
- Recommended concurrent requests: **≤ 10** per client

---

## File Type Detection

The API automatically detects file types using:

1. **Magic bytes** (primary method) - inspects file header
2. **File extension** (fallback) - for GUI uploads

**Supported formats:**

| Type | Extensions      | MIME Types        |
| ---- | --------------- | ----------------- |
| PDF  | `.pdf`          | `application/pdf` |
| JPEG | `.jpg`, `.jpeg` | `image/jpeg`      |
| PNG  | `.png`          | `image/png`       |
| WebP | `.webp`         | `image/webp`      |
| TIFF | `.tiff`, `.tif` | `image/tiff`      |

---

## Compression Algorithm Details

### PDF Compression

1. Remove duplicate objects (using fast hash-based deduplication)
2. Compress embedded images using JPEG encoding (parallelized across CPU cores)
3. Remove metadata objects
4. Apply FlateDecode to streams
5. Prune unused objects (configurable rounds, default: 2)
6. Final compression pass

**Performance Features:**

- Multi-core parallel image processing for 3-8x faster compression on multi-image PDFs
- Async-optimized execution prevents blocking during concurrent requests
- Configurable compression rounds for latency vs quality tuning

### Image Compression

1. Decode source image
2. Downsample large images (>1500px) based on quality:
   - Quality ≥70: max 1500px
   - Quality ≥50: max 1200px
   - Quality <50: max 1000px
3. Encode with target format and quality
4. Auto-select best format (if not specified)

---

## Environment Variables

Configure the API server with these environment variables:

| Variable                 | Required | Default | Description                                                                    |
| ------------------------ | -------- | ------- | ------------------------------------------------------------------------------ |
| `API_KEY`                | No       | —       | API key for authentication. If not set, authentication is disabled.            |
| `PORT`                   | No       | `3000`  | Port number to listen on                                                       |
| `RUST_LOG`               | No       | `info`  | Log level: `error`, `warn`, `info`, `debug`, `trace`                           |
| `PDF_COMPRESSION_ROUNDS` | No       | `2`     | Number of PDF compression rounds (1-5). Lower = faster, higher = smaller files |

**Example:**

```bash
export API_KEY="sk_live_abc123..."
export PORT="8080"
export RUST_LOG="debug"
export PDF_COMPRESSION_ROUNDS="2"
```

**Performance Tuning:**

For optimal latency (fastest compression):

```bash
export PDF_COMPRESSION_ROUNDS="1"
```

For maximum compression (slower but smaller files):

```bash
export PDF_COMPRESSION_ROUNDS="3"
```

---

## Best Practices

### Choosing Compression Level

- **Documents with text:** 70-85 (good balance)
- **Photos/Images:** 50-75 (acceptable quality loss)
- **Archival/Legal docs:** 10-30 (minimal quality loss)
- **Web optimization:** 75-90 (maximum compression)

### Output Format Selection

- **For transparency:** Always use PNG (`output_format=png`)
- **For photographs:** Use JPEG (`output_format=jpeg`)
- **For graphics/logos:** Use PNG or let API auto-select
- **Unsure?** Use `auto` (default) for best results

### Error Handling

Always check:

1. HTTP status code (200 = success)
2. `X-Compressed-Size` header (ensure file was actually compressed)
3. Error response body for detailed messages

### Performance Tips

- Compress files in batches using concurrent requests - the API handles them efficiently
- Cache compressed results when possible
- Use streaming uploads for large files (>10 MB)
- Monitor `X-Reduction-Percentage` to validate compression effectiveness
- For maximum speed, set `PDF_COMPRESSION_ROUNDS=1` (minimal quality impact)
- Multi-image PDFs benefit most from the parallelized compression engine

---

## Changelog

### v0.1.0 - Current

**Added:**

- Image compression support (JPEG, PNG, WebP, TIFF)
- `/api/compress` unified endpoint
- `output_format` parameter for images
- Auto-detection of file types via magic bytes
- Smart output format selection
- Multi-core parallel image processing (3-8x faster for multi-image PDFs)
- `PDF_COMPRESSION_ROUNDS` environment variable for performance tuning
- Async-optimized execution for better concurrent request handling

**Changed:**

- `/api/pdf` is now a legacy alias (still supported)
- Default PDF compression rounds reduced from 3 to 2 (faster with minimal quality impact)

**Performance:**

- 3-6x faster compression for PDFs with 10+ images on multi-core systems
- 1.5-2x faster for text-heavy or single-image PDFs
- Better CPU utilization and concurrent request throughput

**Deprecated:**

- None

---

## Support

For issues, questions, or feature requests:

- **GitHub Issues:** [github.com/your-repo/issues](https://github.com)
- **Email:** support@your-domain.com
- **Documentation:** [your-domain.com/docs](https://your-domain.com)

---

## License

Copyright © 2025. All rights reserved.
