#!/usr/bin/env bash
#
# Script to set up Nginx and SSL (Let's Encrypt) for JeebsAI
#
set -euo pipefail

# Configuration
APP_PORT="${APP_PORT:-8080}"
NGINX_CONFIG="/etc/nginx/sites-available/jeebs"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}Error: This script must be run as root (use sudo).${NC}"
   exit 1
fi

echo -e "${GREEN}JeebsAI SSL Setup (Nginx + Certbot)${NC}"
echo "========================================"

# Check for Nginx
if ! command -v nginx &> /dev/null; then
    echo "Installing Nginx..."
    apt-get update
    apt-get install -y nginx
fi

# Check for Certbot
if ! command -v certbot &> /dev/null; then
    echo "Installing Certbot..."
    apt-get install -y certbot python3-certbot-nginx
fi

# Get Domain
read -p "Enter your domain name (e.g., jeebs.example.com): " DOMAIN
if [[ -z "$DOMAIN" ]]; then
    echo -e "${RED}Domain name is required.${NC}"
    exit 1
fi

# Get Email
read -p "Enter your email for SSL renewal (e.g., admin@example.com): " EMAIL
if [[ -z "$EMAIL" ]]; then
    echo -e "${RED}Email is required for Let's Encrypt.${NC}"
    exit 1
fi

echo "Configuring Nginx for $DOMAIN on port $APP_PORT..."

# Create Nginx Config
cat > "$NGINX_CONFIG" <<EOF
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:$APP_PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable Site
ln -sf "$NGINX_CONFIG" /etc/nginx/sites-enabled/jeebs
rm -f /etc/nginx/sites-enabled/default

# Open Firewall if active
if command -v ufw >/dev/null 2>&1; then
    if ufw status | grep -q "Status: active"; then
        echo "Allowing HTTP/HTTPS through firewall..."
        ufw allow 80/tcp
        ufw allow 443/tcp
    fi
fi

# Test Nginx
nginx -t

# Reload Nginx
systemctl reload nginx

# Run Certbot
echo "Obtaining SSL certificate..."
certbot --nginx -d "$DOMAIN" -m "$EMAIL" --agree-tos --non-interactive --redirect

echo -e "${GREEN}SSL Setup Complete!${NC}"
echo "Access your site at: https://$DOMAIN"