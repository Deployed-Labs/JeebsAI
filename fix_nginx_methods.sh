#!/bin/bash
CONF_PATH="/etc/nginx/sites-available/jeebs"
# This ensures POST and OPTIONS are fully supported
sudo sed -i '/proxy_pass/a \        proxy_set_header X-Forwarded-Method $request_method;' $CONF_PATH
sudo systemctl restart nginx
echo "✅ Nginx headers refreshed."
