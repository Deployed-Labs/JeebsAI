#!/bin/bash
cd /root/JeebsAI

echo "🔧 Forcing frontend to use /api/login..."
# This finds any mention of the login path in your HTML/JS and fixes it
find . -maxdepth 2 -name "*.html" -exec sed -i 's|/login|/api/login|g' {} +
find . -maxdepth 2 -name "*.js" -exec sed -i 's|/login|/api/login|g' {} +

echo "🔌 Updating Nginx to handle the API prefix..."
CONF_PATH="/etc/nginx/sites-available/jeebs"
sudo tee $CONF_PATH > /dev/null <<EON
server {
    listen 80;
    server_name jeebs.club;
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl;
    server_name jeebs.club;

    ssl_certificate /etc/letsencrypt/live/jeebs.club/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/jeebs.club/privkey.pem;

    root /root/JeebsAI;
    index index.html;

    location / {
        try_files \$uri \$uri/ @proxy;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location @proxy {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
}
EON

sudo systemctl restart nginx
echo "✅ Nginx and Frontend synced. Try logging in now."
