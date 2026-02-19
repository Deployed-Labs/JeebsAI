#!/bin/bash
cd /root/JeebsAI

# Force everything to relative paths - no more http/https prefixes in the JS
# This ensures the browser just uses "jeebs.club" automatically.
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i -E 's|https?://[a-zA-Z0-9\.-]+:8080||g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i -E 's|https?://localhost:8080||g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i -E 's|https?://127.0.0.1:8080||g' {} +

# One more check for the /api/ prefix
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|"/login"|"/api/login"|g' {} +

sudo systemctl restart nginx
echo "✅ Paths flattened. Mobile should now connect."
