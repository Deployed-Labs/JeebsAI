#!/bin/bash
set -e

# Ensure we are running from the project root
cd "$(dirname "$0")/.."

# Configuration
VPS_HOST="192.227.193.148"
VPS_USER="root"
SSH_KEY="/Users/shoup/.ssh/jeebs_vps"
APP_DIR="/root/JeebsAI"
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/1476367489490358272/k5Kn7xztOWFsXYOSnmwuHCiF3CQ1WXMxvYvzt4KSf_t0zdx36mVNbj7II9y-vwA6oEOd"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Failure handling
function notify_failure {
    local exit_code=$?
    local line_num=$BASH_LINENO
    echo -e "${RED}❌ Deployment Failed! (Exit code: $exit_code on line $line_num)${NC}"
    if [ -n "$DISCORD_WEBHOOK_URL" ]; then
        echo -e "${RED}🔔 Sending failure notification...${NC}"
        curl -s -H "Content-Type: application/json" \
             -d "{\"content\": \"❌ **JeebsAI Deployment FAILED!**\n\n- Target: $VPS_HOST\n- Exit Code: $exit_code at line $line_num\n- Time: $(date)\"}" \
             "$DISCORD_WEBHOOK_URL" > /dev/null
    fi
}
trap notify_failure ERR

echo -e "${BLUE}🚀 JeebsAI - Push and Deploy${NC}"
echo "Target: $VPS_USER@$VPS_HOST:$APP_DIR"

# Pre-deployment notification
if [ -n "$DISCORD_WEBHOOK_URL" ]; then
    echo -e "${BLUE}🔔 Sending start notification...${NC}"
    curl -s -H "Content-Type: application/json" \
         -d "{\"content\": \"⏳ **JeebsAI Deployment Started**\n\n🚀 Target: $VPS_HOST\n🕒 Time: $(date)\"}" \
         "$DISCORD_WEBHOOK_URL" > /dev/null
fi

# Fix macOS metadata corruption in .git folder (common on external drives)
echo -e "${BLUE}🧹 Cleaning up git metadata...${NC}"
find .git -name "._*" -delete 2>/dev/null || true

# 1. Git Push
echo -e "${BLUE}📦 Pushing to GitHub...${NC}"
git add .
git commit -m "Update: Deployment $(date +'%Y-%m-%d %H:%M')" || echo "Nothing to commit"
git push origin main

# 2. Remote Deploy
echo -e "${BLUE}📡 Connecting to VPS...${NC}"
ssh -o ConnectTimeout=30 -i "$SSH_KEY" "$VPS_USER@$VPS_HOST" << EOF
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

    echo "⬇️  Updating deployment script..."
    git fetch origin
    git reset --hard origin/main
    
    echo "🚀 Running full VPS deployment..."
    chmod +x deploy_to_vps.sh
    sudo ./deploy_to_vps.sh
EOF

echo -e "${GREEN}🎉 Deployment Complete!${NC}"

# Post-deployment notification
if [ -n "$DISCORD_WEBHOOK_URL" ]; then
    echo -e "${BLUE}🔔 Sending Discord notification...${NC}"
    curl -s -H "Content-Type: application/json" \
         -d "{\"content\": \"🚀 **JeebsAI Deployment Complete!**\n\n✅ Target: $VPS_HOST\n🕒 Time: $(date)\"}" \
         "$DISCORD_WEBHOOK_URL" > /dev/null
fi