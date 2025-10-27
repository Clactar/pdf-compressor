#!/bin/bash

echo "🔐 PDF Compressor API - Key Generator"
echo "====================================="
echo ""

if command -v openssl &> /dev/null; then
    echo "Generating secure API key..."
    API_KEY=$(openssl rand -base64 32)
    echo ""
    echo "✅ Your new API key:"
    echo ""
    echo "   $API_KEY"
    echo ""
    echo "To use this key:"
    echo ""
    echo "1. Local development:"
    echo "   export API_KEY=\"$API_KEY\""
    echo ""
    echo "2. Docker:"
    echo "   Add to docker-compose.yml or .env:"
    echo "   API_KEY=$API_KEY"
    echo ""
    echo "3. Coolify:"
    echo "   Add environment variable: API_KEY=$API_KEY"
    echo ""
    echo "4. Save to .env file:"
    echo "   echo 'API_KEY=$API_KEY' >> .env"
    echo ""
elif command -v python3 &> /dev/null; then
    echo "Generating secure API key with Python..."
    API_KEY=$(python3 -c "import secrets; print(secrets.token_urlsafe(32))")
    echo ""
    echo "✅ Your new API key: $API_KEY"
    echo ""
else
    echo "⚠️  openssl or python3 not found."
    echo ""
    echo "Generate a key manually with one of:"
    echo "  openssl rand -base64 32"
    echo "  python3 -c \"import secrets; print(secrets.token_urlsafe(32))\""
fi
