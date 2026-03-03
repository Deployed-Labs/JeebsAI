#!/bin/bash
set -e

# Configuration
VPS_HOST="192.227.193.148"
VPS_USER="root"
SSH_KEY="/Users/shoup/.ssh/jeebs_vps"
APP_DIR="/root/JeebsAI"
DISCORD_WEBHOOK_URL="" # Optional: Paste your Discord Webhook URL here

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 JeebsAI - Push and Deploy${NC}"
echo "Target: $VPS_USER@$VPS_HOST:$APP_DIR"

# 1. Git Push
echo -e "${BLUE}📦 Pushing to GitHub...${NC}"
git add .
git commit -m "Update: Deployment $(date +'%Y-%m-%d %H:%M')" || echo "Nothing to commit"
git push origin main

# 2. Remote Deploy
echo -e "${BLUE}📡 Connecting to VPS...${NC}"
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" << EOF
    set -e
    echo "📂 Navigating to $APP_DIR..."
    
    # Ensure directory exists
    mkdir -p "$APP_DIR"
    cd "$APP_DIR"
    
    # Check if it's a git repo, if not clone it (or init and pull if empty)
    if [ ! -d ".git" ]; then
        echo "⬇️  Cloning repository..."
        # Assuming public or auth configured on VPS
        git clone https://github.com/Deployed-Labs/JeebsAI.git . || echo "Clone failed, trying to pull if dir not empty"
    fi

    echo "⬇️  Pulling latest code..."
    git fetch origin
    git reset --hard origin/main
    
    echo "🧹 Cleaning up macOS metadata..."
    find . -type f -name '._*' -print -delete
    
    echo "🔨 Building release..."
    cargo build --release
    
    echo "🔄 Restarting JeebsAI service..."
    systemctl restart jeebs
    
    echo "✅ Service Status:"
    systemctl status jeebs --no-pager | grep "Active:"
EOF

echo -e "${GREEN}🎉 Deployment Complete!${NC}"

# Post-deployment notification
if [ -n "$DISCORD_WEBHOOK_URL" ]; then
    echo -e "${BLUE}🔔 Sending Discord notification...${NC}"
    curl -s -H "Content-Type: application/json" \
         -d "{\"content\": \"🚀 **JeebsAI Deployment Complete!**\n\n✅ Target: $VPS_HOST\n🕒 Time: $(date)\"}" \
         "$DISCORD_WEBHOOK_URL" > /dev/null
fi
