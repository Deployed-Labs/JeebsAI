#!/bin/bash

CONF_PATH="/etc/nginx/sites-available/jeebs"

echo "Updating Nginx configuration..."

sudo tee $CONF_PATH > /dev/null <<EON
server {
    listen 80;
    server_name jeebs.club www.jeebs.club;

    # The directory where your HTML/JS files are located
    root /root/JeebsAI;
    index index.html;

    location / {
        # Try to serve static files first, fallback to the Rust app
        try_files \$uri \$uri/ @proxy;
    }

    location @proxy {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EON

# Ensure Nginx has permission to access the files in /root
echo "Setting directory permissions for Nginx..."
chmod 755 /root
chmod 755 /root/JeebsAI

# Test and Restart
echo "Testing Nginx configuration..."
if sudo nginx -t; then
    sudo systemctl restart nginx
    echo "✅ Nginx updated and restarted successfully!"
else
    echo "❌ Nginx configuration test failed. Please check the output above."
    exit 1
fi
