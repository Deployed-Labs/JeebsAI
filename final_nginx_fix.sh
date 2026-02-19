#!/bin/bash
CONF_PATH="/etc/nginx/sites-available/jeebs"

sudo tee $CONF_PATH > /dev/null <<EON
server {
    listen 80;
    server_name jeebs.club;

    # 1. Try to serve the HTML files directly from the root
    root /root/JeebsAI;
    index index.html;

    location / {
        # This checks if the file exists; if not, it sends it to the Rust app
        try_files \$uri \$uri/ @proxy;
    }

    # 2. This handles the API calls to your Rust binary
    location @proxy {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EON

sudo nginx -t && sudo systemctl restart nginx
echo "✅ Nginx is now serving your HTML files directly!"
