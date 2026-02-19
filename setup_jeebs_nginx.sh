#!/bin/bash

# Configuration
DOMAIN="jeebs.club"
APP_PORT="8080"
CONF_PATH="/etc/nginx/sites-available/jeebs"

echo "🚀 Starting Nginx cleanup for $DOMAIN..."

# 1. Create a clean, non-SSL config first (Certbot needs this)
echo "Writing fresh config to $CONF_PATH..."
sudo tee $CONF_PATH > /dev/null <<EON
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://localhost:$APP_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EON

# 2. Ensure the link exists in sites-enabled
sudo ln -sf $CONF_PATH /etc/nginx/sites-enabled/

# 3. Test and Reload Nginx
echo "Testing Nginx configuration..."
if sudo nginx -t; then
    sudo systemctl reload nginx
    echo "✅ Nginx is back online (HTTP)."
else
    echo "❌ Nginx test failed. Check the config manually."
    exit 1
fi

# 4. Run Certbot
echo "🛡️ Requesting SSL certificate from Let's Encrypt..."
sudo certbot --nginx -d $DOMAIN --non-interactive --agree-tos --email admin@$DOMAIN --redirect

echo "🎉 Setup complete! Visit https://$DOMAIN"
