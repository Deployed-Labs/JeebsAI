#!/bin/bash

# 1. Variables
DOMAIN="jeebs.club"
APP_PORT="8080"
CONF_PATH="/etc/nginx/sites-available/jeebs"

echo "🧹 Cleaning up existing Nginx configs..."

# 2. Delete the default config if it exists
sudo rm -f /etc/nginx/sites-enabled/default
sudo rm -f /etc/nginx/sites-available/default

# 3. Write a 'Catch-All' config for your domain
# This ensures Nginx knows exactly where to send traffic
echo "Writing new config to $CONF_PATH..."
sudo tee $CONF_PATH > /dev/null <<EON
server {
    listen 80;
    listen [::]:80;
    server_name $DOMAIN www.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:$APP_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EON

# 4. Enable the site and restart
sudo ln -sf $CONF_PATH /etc/nginx/sites-enabled/

echo "🔄 Restarting Nginx..."
if sudo nginx -t; then
    sudo systemctl restart nginx
    echo "✅ Nginx restarted successfully!"
else
    echo "❌ Nginx configuration error found. See above."
    exit 1
fi

echo "🚀 Check https://$DOMAIN now. (If it fails, we will run Certbot next)"
