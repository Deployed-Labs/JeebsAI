#!/bin/bash
CONF_PATH="/etc/nginx/sites-available/jeebs"

echo "📝 CURRENT NGINX CONFIG:"
cat $CONF_PATH

echo -e "\n📂 VERIFYING STATIC FILES IN /root/JeebsAI:"
ls -F /root/JeebsAI | grep -E 'index.html|css/|js/|webui/'

echo -e "\n🛡️ FIXING PERMISSIONS (just in case):"
# Nginx needs +x on parent directories to "walk" into them
chmod 755 /root
chmod 755 /root/JeebsAI
