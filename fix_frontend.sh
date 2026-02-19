#!/bin/bash
cd /root/JeebsAI
# Change all login paths to the /api/ prefix
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|/login|/api/login|g' {} +
# Remove any hardcoded localhost URLs causing Network Errors
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|http://127.0.0.1:8080||g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|http://localhost:8080||g' {} +
echo "✅ Frontend paths synced to /api/login."
