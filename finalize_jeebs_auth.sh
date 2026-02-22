#!/bin/bash
CONF_PATH="/etc/nginx/sites-available/jeebs"
APP_DIR=${APP_DIR:-"/root/JeebsAI"}

echo "🛠️ Applying CORS and Login Fixes..."

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

    root ${APP_DIR};
    index index.html;

    # Security headers for logins
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";

    location / {
        try_files \$uri \$uri/ @proxy;
    }

    location @proxy {
        proxy_pass http://127.0.0.1:8080;
        
        # CORS Headers - Allows the browser to trust the API calls
        add_header 'Access-Control-Allow-Origin' 'https://jeebs.club' always;
        add_header 'Access-Control-Allow-Methods' 'GET, POST, OPTIONS, PUT, DELETE' always;
        add_header 'Access-Control-Allow-Headers' 'DNT,User-Agent,X-Requested-With,If-Modified-Since,Cache-Control,Content-Type,Range,Authorization' always;
        add_header 'Access-Control-Allow-Credentials' 'true' always;

        if (\$request_method = 'OPTIONS') {
            add_header 'Access-Control-Max-Age' 1728000;
            add_header 'Content-Type' 'text/plain; charset=utf-8';
            add_header 'Content-Length' 0;
            return 204;
        }

        # Standard Proxy Headers
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # Session/Cookie support
        proxy_set_header X-Forwarded-Host \$host;
        proxy_set_header X-Forwarded-Port \$server_port;
        proxy_cookie_path / "/; Secure; HttpOnly; SameSite=Lax";
    }
}
EON

# Ensure Nginx permissions are still solid
chmod 755 /root
chmod 755 "$APP_DIR"

if sudo nginx -t; then
    sudo systemctl restart nginx
    echo "✅ Success! Nginx is now CORS-enabled and ready for logins."
    echo "🔗 Try logging in at https://jeebs.club"
else
    echo "❌ Config test failed. Reverting to backup recommended."
fi
