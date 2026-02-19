#!/bin/bash
cd /root/JeebsAI

echo "🔗 Aligning frontend with /api/login..."

# Replace any old login paths with the correct /api/login
find . -maxdepth 2 -name "*.html" -exec sed -i 's|/login"|/api/login"|g' {} +
find . -maxdepth 2 -name "*.html" -exec sed -i "s|'/login'|'/api/login'|g" {} +
find . -maxdepth 2 -name "*.js" -exec sed -i 's|/login|/api/login|g' {} +

echo "✅ Paths aligned. Restarting Nginx..."
sudo systemctl restart nginx
