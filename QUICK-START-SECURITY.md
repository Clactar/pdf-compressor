# 🔐 Quick Start with API Key Security

## ✅ What Just Got Added

Your PDF Compressor API now has **API key authentication**!

- ✅ Secure API key middleware
- ✅ Two authentication methods (X-API-Key or Bearer token)
- ✅ Health endpoint remains public
- ✅ Backward compatible (works without key, but shows warning)
- ✅ Ready for Postman & n8n

---

## 🚀 Getting Started (30 seconds)

### 1. Generate Your API Key

```bash
./generate-api-key.sh
```

This creates a secure random key like: `17cP1mOPj8MJgixWXNLbueS2CDUIi+/31OBaFCoQtS0=`

### 2. Set the Key & Start Server

```bash
# Local development
export API_KEY="your-generated-key"
cargo run --bin pdfcompressor-api

# Or Docker
API_KEY="your-generated-key" docker-compose up
```

### 3. Test It

**✅ With API Key (works):**
```bash
curl -X POST http://localhost:3000/api/pdf \
  -H "X-API-Key: your-generated-key" \
  -F "file=@document.pdf" \
  -o compressed.pdf
```

**❌ Without API Key (fails):**
```bash
curl -X POST http://localhost:3000/api/pdf \
  -F "file=@document.pdf"
  
# Returns: 401 Unauthorized
# {"error":"Missing API key..."}
```

---

## 📮 Using with Postman

### Method 1: Collection-Level Auth (Recommended)

1. **Import** `Postman-Collection.json` (already configured!)
2. **Edit Collection** → Variables tab
3. **Change** `api_key` value to your actual key
4. **Save** - All requests now use your key! ✨

### Method 2: Per-Request

Add to **Headers** tab:
```
Key: X-API-Key
Value: your-generated-key
```

---

## 🔄 Using with n8n

### HTTP Request Node

Add to **Headers**:

```
Name: X-API-Key
Value: your-generated-key
```

Or use **Credentials**:
1. Create "Header Auth" credential
2. Header Name: `X-API-Key`
3. Header Value: `your-generated-key`
4. Select credential in HTTP Request node

**Full example in:** `N8N-GUIDE.md`

---

## 🐳 Docker / Coolify Deployment

### Docker Compose

Create `.env` file:
```bash
API_KEY=your-generated-key
PORT=3000
RUST_LOG=info
```

Then:
```bash
docker-compose up
```

The `docker-compose.yml` is already configured to use `${API_KEY}`!

### Coolify

1. Go to your service
2. **Environment Variables** tab
3. Add:
   ```
   API_KEY=your-generated-key
   ```
4. **Redeploy**

---

## 🔑 Two Ways to Send API Key

Both work identically:

### Option 1: X-API-Key Header
```bash
curl -H "X-API-Key: your-key" ...
```

### Option 2: Bearer Token
```bash
curl -H "Authorization: Bearer your-key" ...
```

---

## 🏥 Health Check (No Auth Needed)

The `/health` endpoint is always public:

```bash
curl http://localhost:3000/health
# Returns: OK
```

Use this for:
- Monitoring
- Load balancer health checks
- Coolify health checks

---

## ⚙️ Configuration Options

### Run WITH Authentication (Recommended)

```bash
API_KEY="your-secure-key" cargo run --bin pdfcompressor-api
```

Output:
```
[INFO] 🔐 API Key authentication enabled
[INFO]    Key: your-sec***
```

### Run WITHOUT Authentication (Dev Only)

```bash
# No API_KEY set
cargo run --bin pdfcompressor-api
```

Output:
```
[WARN] ⚠️  No API_KEY set - API is unprotected!
```

API will work but show warnings. **Not recommended for production!**

---

## 🛠️ Quick Commands Reference

```bash
# Generate key
./generate-api-key.sh

# Test without auth (should fail)
curl -X POST http://localhost:3000/api/pdf -F "file=@test.pdf"

# Test with auth (should work)
curl -X POST http://localhost:3000/api/pdf \
  -H "X-API-Key: your-key" \
  -F "file=@test.pdf" \
  -o out.pdf

# Check health (always works)
curl http://localhost:3000/health

# View logs for auth failures
docker-compose logs -f | grep "Authentication failed"
```

---

## 📚 Full Documentation

| Document | Purpose |
|----------|---------|
| `SECURITY-SETUP.md` | Complete security guide |
| `README-API.md` | API usage & endpoints |
| `DEPLOYMENT-GUIDE.md` | Coolify deployment |
| `N8N-GUIDE.md` | n8n integration examples |
| `Postman-Collection.json` | Import to Postman |
| `generate-api-key.sh` | Key generator script |

---

## ✅ Checklist

- [ ] Generated secure API key with `./generate-api-key.sh`
- [ ] Set `API_KEY` environment variable
- [ ] Server started and shows "🔐 API Key authentication enabled"
- [ ] Tested API call with key (works)
- [ ] Tested API call without key (fails with 401)
- [ ] Updated Postman collection with key
- [ ] Updated n8n workflows with key (if using)
- [ ] Documented key securely for your team
- [ ] Set key in Coolify environment variables (for production)

---

## 🆘 Common Issues

**Issue:** "Missing API key" error
```
✓ Make sure you're sending X-API-Key header
✓ Check spelling: X-API-Key (not X-Api-Key)
```

**Issue:** "Invalid API key" error
```
✓ Verify API_KEY matches what server is using
✓ No extra spaces in the key
✓ Restart server after changing API_KEY
```

**Issue:** Server still accepts requests without key
```
✓ Make sure API_KEY environment variable is set
✓ Restart the server
✓ Check logs for "🔐 API Key authentication enabled"
```

---

## 🎉 You're Secured!

Your API is now protected with key-based authentication while remaining simple to use.

**Key benefits:**
- ✅ Prevents unauthorized access
- ✅ Easy to integrate (just add header)
- ✅ Works with all tools (Postman, n8n, curl, etc.)
- ✅ Backward compatible
- ✅ Production-ready

**Start compressing PDFs securely! 🚀**

