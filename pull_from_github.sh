#!/usr/bin/env bash
#
# Pull from GitHub - Run this on your VPS to update JeebsAI
#
set -e

echo "📥 Pulling Latest JeebsAI from GitHub"
echo "====================================="

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Determine the script's directory (where JeebsAI is located)
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

info "Working directory: $SCRIPT_DIR"
echo ""

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    error "Not a git repository"
    exit 1
fi

# Show current branch
CURRENT_BRANCH=$(git branch --show-current)
info "Current branch: $CURRENT_BRANCH"

# Stash any local changes
if ! git diff-index --quiet HEAD --; then
    warn "Local changes detected - stashing them"
    git stash save "Auto-stash before pull $(date +%Y%m%d_%H%M%S)"
    STASHED=true
else
    STASHED=false
    info "No local changes to stash"
fi

# Fetch latest changes
info "Fetching from GitHub..."
git fetch origin

# Pull latest changes
info "Pulling latest changes..."
git pull origin main

success "Repository updated successfully!"
echo ""

# Check if we stashed anything
if [ "$STASHED" = true ]; then
    warn "Local changes were stashed. To restore them, run:"
    echo "    git stash pop"
    echo ""
fi

# Check if Jeebs service is running
if systemctl is-active --quiet jeebs 2>/dev/null; then
    info "Jeebs service is running"
    read -p "Restart Jeebs service to apply changes? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Restarting Jeebs service..."
        sudo systemctl restart jeebs
        sleep 2

        if systemctl is-active --quiet jeebs; then
            success "Jeebs service restarted successfully!"
        else
            error "Jeebs service failed to start. Check logs with: journalctl -u jeebs -n 50"
        fi
    else
        warn "Remember to restart Jeebs manually: sudo systemctl restart jeebs"
    fi
elif systemctl is-active --quiet jeebs-docker 2>/dev/null; then
    info "Jeebs Docker service is running"
    read -p "Rebuild and restart Docker containers? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Rebuilding Docker containers..."
        docker-compose down
        docker-compose up -d --build

        success "Docker containers rebuilt and started!"
    else
        warn "Remember to rebuild containers manually: docker-compose up -d --build"
    fi
else
    warn "Jeebs service not detected. Start it manually if needed."
fi

echo ""
success "✅ Pull complete!"
echo ""
info "View the new Topic Learning feature at: http://your-vps-ip/webui/admin_dashboard.html"
