#!/usr/bin/env bash
#
# SIMPLE UPDATE SCRIPT - Run this on your VPS after pushing changes
# Just run: ./update.sh (or bash update.sh)
#
set -e

echo "🚀 Updating JeebsAI..."
echo ""

# Stash any local changes automatically
echo "📦 Stashing local changes..."
git stash save "Auto-stash $(date +%Y%m%d_%H%M%S)" 2>/dev/null || echo "Nothing to stash"

# Pull latest code
echo "📥 Pulling from GitHub..."
git pull origin main



# Deploy static site files
echo "🌐 Updating static site files..."
cp index.html style.css register.html chat.html /var/www/html/
chown www-data:www-data /var/www/html/index.html /var/www/html/style.css /var/www/html/register.html /var/www/html/chat.html
chmod 644 /var/www/html/index.html /var/www/html/style.css /var/www/html/register.html /var/www/html/chat.html

# Restart nginx for static site
echo "🔄 Restarting nginx..."
systemctl restart nginx
echo "✅ Static site updated and nginx restarted!"

echo ""
echo "════════════════════════════════════════════════════════"
echo "✅ UPDATE COMPLETE!"
echo "════════════════════════════════════════════════════════"
echo ""
echo "Changes are now live on your VPS!"
echo ""
