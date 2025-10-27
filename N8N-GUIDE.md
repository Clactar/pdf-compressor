# n8n Integration Guide for PDF Compressor API

## 🔄 Setup Methods

### Method 1: HTTP Request Node (Recommended)

This is the most flexible approach for n8n workflows.

#### Configuration:

1. **Add HTTP Request Node**
2. **Configure as follows:**

```
┌─────────────────────────────────────────────────┐
│ HTTP Request Node                               │
├─────────────────────────────────────────────────┤
│ Method:              POST                       │
│ URL:                 http://localhost:3000/api/pdf │
│                      (or your domain)           │
│                                                 │
│ Authentication:      None (or add if secured)  │
│                                                 │
│ Send Body:           ✓ Yes                     │
│ Body Content Type:   Multipart Form Data       │
│                                                 │
│ Body Parameters:                                │
│   - Name: file                                  │
│     Value: {{ $binary.data }}                  │
│   - Name: compression                           │
│     Value: 75                                   │
│                                                 │
│ Response Format:     File                       │
│ Binary Property:     data                       │
└─────────────────────────────────────────────────┘
```

#### Step-by-Step:

**1. Method & URL:**
- Method: `POST`
- URL: `http://localhost:3000/api/pdf` (or `https://your-domain.com/api/pdf` for production)

**2. Authentication:**
- Select: `None` (unless you've added auth)

**3. Send Body:**
- Toggle: `ON` ✓
- Body Content Type: `Multipart-Form-Data`

**4. Body Parameters:**
Click "Add Parameter" twice:

**Parameter 1:**
- Name: `file`
- Value: `={{ $binary.data }}` (or `={{ $node["Previous Node"].binary.data }}`)
- Input Data Field Name: Leave empty or set to `file`

**Parameter 2:**
- Name: `compression`
- Value: `75` (or `={{ $json.compressionLevel }}` to use dynamic value)

**5. Options → Response:**
- Response Format: `File`
- Put Output in Field: `data`

---

### Method 2: Code Node (Advanced)

For more control, use a Function/Code node:

```javascript
// Get PDF from previous node (binary data)
const pdfBinary = items[0].binary.data;

// Prepare form data
const FormData = require('form-data');
const form = new FormData();

// Add PDF file
form.append('file', Buffer.from(pdfBinary.data, 'base64'), {
  filename: 'document.pdf',
  contentType: 'application/pdf'
});

// Add compression level
form.append('compression', '75');

// Make API call
const response = await this.helpers.httpRequest({
  method: 'POST',
  url: 'http://localhost:3000/api/pdf',
  body: form,
  headers: form.getHeaders(),
  encoding: 'arraybuffer'
});

// Get compression stats from headers
const originalSize = response.headers['x-original-size'];
const compressedSize = response.headers['x-compressed-size'];
const reduction = response.headers['x-reduction-percentage'];

// Return compressed PDF as binary
return [{
  json: {
    originalSize,
    compressedSize,
    reduction,
    status: 'compressed'
  },
  binary: {
    data: {
      data: Buffer.from(response.data).toString('base64'),
      mimeType: 'application/pdf',
      fileName: 'compressed.pdf'
    }
  }
}];
```

---

## 📋 Complete Workflow Examples

### Example 1: Simple File Upload → Compress → Save

```
[Manual Trigger] 
    ↓
[Read Binary File] → Load PDF from disk
    ↓
[HTTP Request] → Compress PDF (your API)
    ↓
[Write Binary File] → Save compressed PDF
```

**Node Details:**

**1. Manual Trigger**
- Just to start the workflow

**2. Read Binary File**
- File Path: `/path/to/input.pdf`
- Property Name: `data`

**3. HTTP Request** (PDF Compressor)
- Method: `POST`
- URL: `http://localhost:3000/api/pdf`
- Body: Multipart Form Data
  - `file`: `={{ $binary.data }}`
  - `compression`: `75`
- Response Format: `File`

**4. Write Binary File**
- File Path: `/path/to/output.pdf`
- Property Name: `data`

### Example 2: Email Attachment → Compress → Send Back

```
[Email Trigger] → Watch for emails with PDF attachments
    ↓
[HTTP Request] → Compress PDF
    ↓
[Send Email] → Reply with compressed PDF
```

**Node Details:**

**1. Email Trigger (IMAP)**
- Watch for: Attachments with `.pdf`
- Download Attachments: `Yes`

**2. HTTP Request** (PDF Compressor)
- URL: `http://localhost:3000/api/pdf`
- Body:
  - `file`: `={{ $binary.attachment_0 }}`
  - `compression`: `80`

**3. Send Email (SMTP)**
- To: `={{ $node["Email Trigger"].json.from }}`
- Subject: `Compressed PDF - {{ $node["Email Trigger"].json.subject }}`
- Attachments: `data` (binary property)

### Example 3: Webhook → Compress → Return URL

```
[Webhook] → Receive PDF upload
    ↓
[HTTP Request] → Compress PDF
    ↓
[Google Drive] → Upload compressed PDF
    ↓
[Webhook Response] → Return download link
```

**Node Details:**

**1. Webhook**
- Path: `/compress-pdf`
- Method: `POST`
- Response Mode: `Last Node`
- Binary Property: `file`

**2. HTTP Request** (PDF Compressor)
- URL: `http://localhost:3000/api/pdf`
- Body:
  - `file`: `={{ $binary.file }}`
  - `compression`: `={{ $json.body.compression || 75 }}`

**3. Google Drive**
- Operation: `Upload a file`
- File Name: `compressed_{{ $now }}.pdf`
- Binary Data: `Yes`
- Binary Property: `data`

**4. Webhook Response**
```json
{
  "success": true,
  "fileUrl": "={{ $node["Google Drive"].json.webViewLink }}",
  "originalSize": "={{ $node["HTTP Request"].json.headers['x-original-size'] }}",
  "compressedSize": "={{ $node["HTTP Request"].json.headers['x-compressed-size'] }}",
  "reduction": "={{ $node["HTTP Request"].json.headers['x-reduction-percentage'] }}%"
}
```

---

## 🎛️ Dynamic Compression Levels

### Use Expression for Dynamic Compression:

In the HTTP Request node, instead of hardcoding `75`:

**Option 1: From JSON data**
```javascript
={{ $json.compressionLevel }}
```

**Option 2: Based on file size**
```javascript
={{ $binary.data.fileSize > 5000000 ? 85 : 75 }}
// Use 85% for files > 5MB, otherwise 75%
```

**Option 3: From webhook query parameter**
```javascript
={{ $node["Webhook"].json.query.compression || 75 }}
// Use compression from ?compression=80, default to 75
```

---

## 🔐 Production Setup with Authentication

If you add authentication to your API, configure n8n:

### Using API Key (Header)

In HTTP Request node:

**Headers:**
```
Name: Authorization
Value: Bearer YOUR_API_KEY
```

### Using Basic Auth

In HTTP Request node:

**Authentication:**
- Type: `Basic Auth`
- User: `your-username`
- Password: `your-password`

---

## 🌐 Environment Variables in n8n

Store your API URL as an environment variable:

**In n8n settings:**
```
PDF_COMPRESSOR_URL=https://pdf-api.yourdomain.com
```

**In HTTP Request node:**
```
URL: ={{ $env.PDF_COMPRESSOR_URL }}/api/pdf
```

---

## 📊 Monitoring & Error Handling

### Add Error Workflow

Create a separate error workflow to handle failures:

**Main Workflow:**
- Settings → Error Workflow: `PDF Compression Error Handler`

**Error Handler Workflow:**
```
[Error Trigger]
    ↓
[IF: Check Error Type]
    ↓
[Send Notification] → Email/Slack alert
    ↓
[Log Error] → Write to database/file
```

### Check Response in IF Node

```javascript
// Check if compression was successful
{{ $node["HTTP Request"].json.statusCode === 200 }}

// Check reduction percentage
{{ parseFloat($node["HTTP Request"].json.headers['x-reduction-percentage']) > 10 }}
```

---

## 🚀 Performance Tips

1. **Process in Batches:**
   - Use "Split In Batches" node for multiple PDFs
   - Batch size: 5-10 files at a time

2. **Timeout Settings:**
   - Large PDFs (>50MB) may take time
   - Set timeout: 300000ms (5 minutes)

3. **Binary Data Handling:**
   - Enable "Binary Data as Buffer" in settings
   - Use Binary Data Mode: "filesystem" for large files

4. **Retry Logic:**
   - Enable "Retry On Fail" in HTTP Request node
   - Max Tries: 3
   - Wait Between Tries: 5000ms

---

## 🧪 Testing Your Workflow

### Test with Sample PDF

1. Create a simple workflow:
   - Manual Trigger
   - HTTP Request (compress)
   - Set Node (display results)

2. Use the Set node to show stats:
```javascript
{
  "originalSize": "={{ $node["HTTP Request"].json.headers['x-original-size'] }}",
  "compressedSize": "={{ $node["HTTP Request"].json.headers['x-compressed-size'] }}",
  "reduction": "={{ $node["HTTP Request"].json.headers['x-reduction-percentage'] }}%",
  "binaryDataExists": "={{ $binary.data ? 'Yes' : 'No' }}"
}
```

3. Execute and verify output

---

## 📦 Import Ready-Made Workflow

I've created a starter workflow for you: `n8n-workflow.json`

**To import:**
1. Open n8n
2. Click "+" → Import from File
3. Select `n8n-workflow.json`
4. Update the URL to your API endpoint
5. Execute!

---

## 🔗 API Endpoints Reference

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/pdf` | POST | Compress PDF |
| `/health` | GET | Health check (use for monitoring) |

### Response Headers

All compressed PDFs include these headers:

```
X-Original-Size: 5242880
X-Compressed-Size: 1310720
X-Reduction-Percentage: 75.00
Content-Type: application/pdf
```

Access in n8n:
```javascript
{{ $node["HTTP Request"].json.headers['x-original-size'] }}
{{ $node["HTTP Request"].json.headers['x-compressed-size'] }}
{{ $node["HTTP Request"].json.headers['x-reduction-percentage'] }}
```

---

## 🐛 Troubleshooting

### Issue: "Binary data is empty"

**Solution:** Ensure previous node outputs binary data:
```javascript
{{ $binary.data ? 'OK' : 'MISSING' }}
```

### Issue: "Request timeout"

**Solution:** Increase timeout in HTTP Request node:
- Options → Timeout: `300000` (5 minutes)

### Issue: "File not sent correctly"

**Solution:** Check Content-Type:
- Must be `Multipart-Form-Data`
- Not `application/json` or `x-www-form-urlencoded`

### Issue: "Cannot read property 'data'"

**Solution:** Check Binary Property name matches:
- Input: `data` (or name from previous node)
- Output: `data` (consistent naming)

---

## 💡 Use Cases

1. **Automated Email Attachments:** Compress PDFs in emails before forwarding
2. **Cloud Storage Optimization:** Compress before uploading to Drive/Dropbox
3. **Batch Processing:** Process entire folders on schedule
4. **API Gateway:** Expose as webhook for other services
5. **Backup Optimization:** Compress PDFs before archiving

---

## 🆘 Need Help?

- Check n8n docs: https://docs.n8n.io/
- Test API with Postman first
- Use n8n's "Execute Node" to debug step-by-step
- Check binary data with "Set" node

**Happy Automating! 🚀**

