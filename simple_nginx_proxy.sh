#!/usr/bin/env bash
# simple_nginx_proxy.sh - ensure nginx listens on jeebs.club and proxies to the app
# run on the VPS as root (or with sudo)

set -euo pipefail

DOMAIN=${1:-jeebs.club}
PORT=${2:-8080}
CONF="/etc/nginx/sites-available/jeebs"

cat > "$CONF" <<EOF
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl;
    server_name $DOMAIN www.$DOMAIN;
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:$PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_buffering off;
        proxy_cache off;
    }
}
EOF

ln -sf "$CONF" /etc/nginx/sites-enabled/jeebs

echo "testing nginx configuration..."
if nginx -t; then
    echo "reloading nginx..."
    systemctl reload nginx
else
    echo "nginx test failed, inspect $CONF" >&2
    exit 1
fi

# open firewall ports if ufw present
if command -v ufw &>/dev/null; then
    echo "allowing 80/443 through ufw"
    ufw allow 80,443/tcp || true
fi

echo "done - proxying $DOMAIN -> 127.0.0.1:$PORT"

echo "remember to ensure the jeebs service is running and ports are
reachable from the outside (firewall/cloud rules)."
