# 🔐 API Key Security Setup

Your PDF Compressor API now supports API key authentication!

## 🎯 Quick Start

### 1. Generate a Secure API Key

**Option A: Using OpenSSL (Recommended)**
```bash
openssl rand -base64 32
```
Output example: `8yHFk3mN9pQr2sT5vW7xZ0aB1cD4eF6gH9jK1lM3nP5=`

**Option B: Using Python**
```bash
python3 -c "import secrets; print(secrets.token_urlsafe(32))"
```

**Option C: Using Node.js**
```bash
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

### 2. Set the API Key

**Local Development:**
```bash
# Create .env file (already done for you)
echo "API_KEY=your-generated-key-here" > .env

# Or export directly
export API_KEY="your-generated-key-here"

# Run the server
cargo run --bin pdfcompressor-api
```

**Docker Compose:**
```bash
# Edit docker-compose.yml or create .env file:
echo "API_KEY=your-generated-key-here" > .env

# Run
docker-compose up
```

**Coolify Deployment:**
1. Go to your service in Coolify
2. Click on "Environment Variables"
3. Add: `API_KEY` = `your-generated-key-here`
4. Redeploy

### 3. Test Authentication

**Without API Key (should fail):**
```bash
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@test.pdf"
  
# Response: 401 Unauthorized
# {"error":"Missing API key. Provide X-API-Key header or Authorization: Bearer <key>"}
```

**With API Key (should work):**
```bash
curl -X POST http://localhost:3000/api/pdf \
  -H "X-API-Key: your-generated-key-here" \
  -F "file=@test.pdf" \
  -o compressed.pdf
```

---

## 📋 How to Use the API Key

### Method 1: X-API-Key Header (Recommended)

```bash
curl -X POST http://localhost:3000/api/pdf \
  -H "X-API-Key: your-api-key" \
  -F "file=@document.pdf" \
  -o compressed.pdf
```

### Method 2: Authorization Bearer Token

```bash
curl -X POST http://localhost:3000/api/pdf \
  -H "Authorization: Bearer your-api-key" \
  -F "file=@document.pdf" \
  -o compressed.pdf
```

Both methods work identically!

---

## 🔧 Configuration

### Backward Compatibility

**No API_KEY set:** API runs **without authentication** (shows warning)
```
⚠️  No API_KEY set - API is unprotected!
```

**API_KEY set:** Authentication **required** for all endpoints except `/health`
```
🔐 API Key authentication enabled
   Key: 8yHFk3mN***
```

### Public Endpoints

The `/health` endpoint is **always public** (no authentication needed):
```bash
curl http://localhost:3000/health
# Returns: OK
```

### Protected Endpoints

- `POST /api/pdf` - **Requires API key** when configured

---

## 📮 Postman Setup

### Add Authentication to Collection

1. **Open Postman Collection**
2. **Edit Collection Settings** (click "..." → Edit)
3. **Go to Authorization tab**
4. **Type:** API Key
5. **Add to:** Header
6. **Key:** `X-API-Key`
7. **Value:** `your-api-key`
8. **Save**

Now all requests in the collection will include the API key!

### Or Add to Individual Request

In each request:
1. **Headers tab**
2. **Add:** 
   - Key: `X-API-Key`
   - Value: `your-api-key`

---

## 🔄 n8n Setup

### HTTP Request Node Configuration

Add authentication to your HTTP Request node:

**Method 1: Header Authentication (Simple)**
```
Headers:
  Name: X-API-Key
  Value: your-api-key
```

**Method 2: Using Credentials**

1. **In n8n, go to Credentials**
2. **Create new "Header Auth" credential:**
   - Name: `PDF Compressor API Key`
   - Header Name: `X-API-Key`
   - Header Value: `your-api-key`
3. **In HTTP Request node:**
   - Authentication: Header Auth
   - Credential: Select "PDF Compressor API Key"

**Method 3: Using Environment Variable**

```javascript
// In HTTP Request node
Headers:
  Name: X-API-Key
  Value: ={{ $env.PDF_API_KEY }}
```

Then set `PDF_API_KEY` in your n8n environment variables.

---

## 🔒 Security Best Practices

### ✅ DO:

1. **Generate a strong random key** (at least 32 characters)
2. **Store API key in environment variables** (not in code)
3. **Use different keys** for dev/staging/production
4. **Rotate keys periodically** (every 90 days)
5. **Use HTTPS in production** (Coolify handles this)
6. **Monitor failed authentication attempts**
7. **Keep keys in `.env` file** (add to `.gitignore`)

### ❌ DON'T:

1. ❌ Commit API keys to Git
2. ❌ Share keys in plain text
3. ❌ Use weak keys like "password123"
4. ❌ Hardcode keys in your application
5. ❌ Use the same key across environments
6. ❌ Expose keys in client-side code
7. ❌ Log API keys

---

## 🔑 Key Rotation

### To Change API Key:

**1. Generate New Key:**
```bash
openssl rand -base64 32
```

**2. Update Environment Variable:**

**Local:**
```bash
export API_KEY="new-key-here"
# Restart server
```

**Docker:**
```bash
# Update .env or docker-compose.yml
docker-compose restart
```

**Coolify:**
1. Update `API_KEY` environment variable
2. Redeploy service

**3. Update All Clients:**
- Postman collections
- n8n workflows  
- Application code
- Documentation

### Gradual Migration (Zero Downtime)

To avoid breaking existing clients:

1. Deploy API with new key but keep supporting old key temporarily
2. Update all clients to use new key
3. Verify all clients are using new key
4. Remove old key support
5. Redeploy

---

## 🛡️ Multi-Key Support (Advanced)

Want to support multiple API keys? Modify `src/api.rs`:

```rust
// In auth_middleware function, replace:
let expected_key = std::env::var("API_KEY")?;

// With:
let valid_keys: Vec<String> = std::env::var("API_KEYS")
    .unwrap_or_default()
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();

// Then check if provided_key is in valid_keys
```

Then set multiple keys:
```bash
API_KEYS="key1,key2,key3"
```

---

## 📊 Monitoring Authentication

### Log Analysis

The API logs authentication attempts:

**Successful:**
```
[INFO] PDF Compressor API starting...
[INFO] 🔐 API Key authentication enabled
```

**Failed Attempts:**
```
[WARN] 🚫 Authentication failed: Invalid API key
[WARN] 🚫 Authentication failed: No API key provided
```

### Monitor Failed Attempts

**Using grep:**
```bash
docker-compose logs -f | grep "Authentication failed"
```

**In Coolify:**
- Go to Logs tab
- Filter by "Authentication failed"
- Set up alerts for repeated failures

---

## 🔐 Additional Security Layers

### 1. Rate Limiting

Add rate limiting middleware (optional):

```toml
# Add to Cargo.toml
tower-governor = "0.1"
```

### 2. IP Whitelisting

Configure Coolify or nginx to allow only specific IPs.

### 3. mTLS (Mutual TLS)

For extra security, use client certificates in addition to API keys.

### 4. API Key Scopes

Implement different keys for read-only vs write operations.

---

## 🧪 Testing Authentication

### Test Script

```bash
#!/bin/bash

API_URL="http://localhost:3000"
API_KEY="your-api-key"

echo "Testing without API key (should fail)..."
curl -X POST $API_URL/api/pdf \
  -F "file=@test.pdf" \
  -w "\nStatus: %{http_code}\n"

echo ""
echo "Testing with valid API key (should work)..."
curl -X POST $API_URL/api/pdf \
  -H "X-API-Key: $API_KEY" \
  -F "file=@test.pdf" \
  -o compressed.pdf \
  -w "\nStatus: %{http_code}\n"

echo ""
echo "Testing with invalid API key (should fail)..."
curl -X POST $API_URL/api/pdf \
  -H "X-API-Key: wrong-key" \
  -F "file=@test.pdf" \
  -w "\nStatus: %{http_code}\n"

echo ""
echo "Testing health endpoint (should always work)..."
curl -X GET $API_URL/health \
  -w "\nStatus: %{http_code}\n"
```

---

## 📝 Environment File Template

Create `.env` in your project root:

```bash
# PDF Compressor API Configuration

# Required: Your secure API key
API_KEY=CHANGE_THIS_TO_SECURE_KEY

# Optional: Server configuration
PORT=3000
RUST_LOG=info

# Generate secure key with:
# openssl rand -base64 32
```

**Important:** Add `.env` to `.gitignore`!

```bash
echo ".env" >> .gitignore
```

---

## ❓ Troubleshooting

### Issue: "Missing API key" error

**Cause:** API_KEY is set but you're not sending the header

**Solution:** Add `-H "X-API-Key: your-key"` to your request

### Issue: "Invalid API key" error

**Cause:** The key you're sending doesn't match the configured key

**Solution:** 
1. Check your API_KEY environment variable: `echo $API_KEY`
2. Verify you're using the correct key
3. No extra spaces or special characters

### Issue: API still works without key

**Cause:** API_KEY environment variable is not set

**Solution:** 
```bash
export API_KEY="your-key"
# Restart the server
```

### Issue: Different key needed for Docker vs local

**Cause:** Environment variables are separate

**Solution:** Use consistent `.env` file or set both

---

## 🎉 You're All Set!

Your API is now secured with API key authentication. 

**Next Steps:**
1. ✅ Generate a strong API key
2. ✅ Set it in environment variables
3. ✅ Update Postman/n8n with the key
4. ✅ Test authentication
5. ✅ Deploy to Coolify with the key
6. ✅ Document key for your team (securely!)

**Questions?** Check the logs or test with the scripts above! 🚀

