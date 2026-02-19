#!/bin/bash
CONF="/etc/nginx/sites-available/jeebs"
sudo tee $CONF > /dev/null <<EON
server {
    listen 80; server_name jeebs.club;
    return 301 https://\$host\$request_uri;
}
server {
    listen 443 ssl; server_name jeebs.club;
    ssl_certificate /etc/letsencrypt/live/jeebs.club/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/jeebs.club/privkey.pem;
    root /root/JeebsAI; index index.html;
    location / { try_files \$uri \$uri/ @proxy; }
    location /api/ {
        add_header 'Access-Control-Allow-Origin' '*' always;
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
    }
    location @proxy { proxy_pass http://127.0.0.1:8080; }
}
EON
sudo systemctl restart nginx
echo "✅ Nginx reconfigured for CORS and SSL."
