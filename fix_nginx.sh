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
    # do NOT serve files out of /root; the app serves its own webui
    # (nginx runs as www-data and cannot read /root anyway)
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_buffering off;
        proxy_cache off;
    }
}
EON
sudo systemctl restart nginx
echo "✅ Nginx reconfigured for CORS and SSL."
