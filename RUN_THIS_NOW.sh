#!/usr/bin/env bash
# RUN THIS NOW - Ultimate simple deployment

echo "🚀 Pushing Topic Learning Feature to GitHub..."
echo ""

# Make scripts executable first
chmod +x pull_from_github.sh
chmod +x auto_deploy.sh
chmod +x deploy_now.sh
chmod +x simple_push.sh

# Add everything
git add -A

# Commit
git commit -m "Add Topic Learning feature with deployment scripts" || echo "Already committed"

# Push
git push origin main

echo ""
echo "═══════════════════════════════════════════════════"
echo "✅  SUCCESSFULLY PUSHED TO GITHUB!"
echo "═══════════════════════════════════════════════════"
echo ""
echo "📥 TO DEPLOY ON VPS:"
echo ""
echo "1. SSH to your VPS:"
echo "   ssh your-user@your-vps-ip"
echo ""
echo "2. Go to JeebsAI:"
echo "   cd ~/JeebsAI"
echo ""
echo "3. Pull changes:"
echo "   bash pull_from_github.sh"
echo ""
echo "4. Access at:"
echo "   http://your-vps-ip/webui/admin_dashboard.html"
echo ""
echo "═══════════════════════════════════════════════════"
echo ""
