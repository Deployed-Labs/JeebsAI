#!/bin/bash
# 1. Capture the path from the source code
REAL_PATH=$(grep -B 3 "pub async fn login" src/auth/mod.rs | grep -oP 'post\("\K[^"]+')

if [ -z "$REAL_PATH" ]; then
    echo "❌ Could not auto-detect path. Please check Step 1 output."
    exit 1
fi

echo "✅ Detected Real Path: /$REAL_PATH"

# 2. Update the frontend files in root and webui
echo "Updating frontend files..."
find . -maxdepth 2 -name "*.html" -exec sed -i "s|/login|/$REAL_PATH|g" {} +
find . -maxdepth 2 -name "*.js" -exec sed -i "s|/login|/$REAL_PATH|g" {} +

echo "🔄 Restarting Nginx and Jeebs..."
sudo systemctl restart nginx
sudo systemctl restart jeebs
echo "🚀 Try logging in at https://jeebs.club using /$REAL_PATH"
