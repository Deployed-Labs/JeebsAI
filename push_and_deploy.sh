#!/usr/bin/env bash
#
# Push to Main Branch and Deploy
# Run this script on your LOCAL machine to push changes and deploy to VPS
#
set -e

echo "🚀 JeebsAI - Push to Main and Deploy"
echo "====================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
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

# Configuration (edit these as needed)
VPS_HOST="${VPS_HOST:-your-vps-hostname-or-ip}"
VPS_USER="${VPS_USER:-root}"
VPS_APP_DIR="${VPS_APP_DIR:-/opt/jeebs}"

# Step 1: Check git status
info "Checking git status..."
git status

echo ""
read -p "Do you want to commit all changes? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Step 2: Add all changes
    info "Adding all changes..."
    git add .

    # Step 3: Commit
    read -p "Enter commit message: " commit_message
    if [ -z "$commit_message" ]; then
        commit_message="Deploy updates - $(date '+%Y-%m-%d %H:%M:%S')"
    fi

    info "Committing changes..."
    git commit -m "$commit_message" || warn "No changes to commit or commit failed"
else
    warn "Skipping commit. Proceeding with existing commits..."
fi

echo ""

# Step 4: Push to main
info "Pushing to main branch..."
git push origin main

success "✅ Code pushed to main branch!"

echo ""
echo "=========================================="
echo ""

# Step 5: Ask about VPS deployment
read -p "Do you want to deploy to VPS now? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    info "Deploying to VPS: $VPS_USER@$VPS_HOST"

    # Upload deployment script
    info "Uploading deployment script..."
    scp deploy_to_vps.sh "$VPS_USER@$VPS_HOST:/tmp/deploy_jeebs.sh"

    # Execute deployment on VPS
    info "Executing deployment on VPS..."
    ssh -t "$VPS_USER@$VPS_HOST" "chmod +x /tmp/deploy_jeebs.sh && sudo /tmp/deploy_jeebs.sh"

    success "✅ Deployment complete!"
else
    echo ""
    warn "Skipping VPS deployment."
    echo ""
    info "To deploy later, run on your VPS:"
    echo "  cd $VPS_APP_DIR"
    echo "  sudo ./deploy_to_vps.sh"
    echo ""
    info "Or run this script again and choose 'y' for deployment."
fi

echo ""
success "=========================================="
success "🎉 All Done!"
success "=========================================="
echo ""
