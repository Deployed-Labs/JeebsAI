#!/bin/bash
cd /root/JeebsAI

# Find any JS/HTML files and make ALL API calls relative
# This prevents the phone from trying to talk to "127.0.0.1" (which is the phone itself!)
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|http://127.0.0.1:8080||g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|http://localhost:8080||g' {} +
find . -type f \( -name "*.html" -o -name "*.js" \) -exec sed -i 's|https://jeebs.club:8080||g' {} +

# Clear Nginx and restart
sudo systemctl restart nginx
echo "✅ Mobile compatibility fix applied."
