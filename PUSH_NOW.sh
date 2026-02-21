#!/usr/bin/env bash
# FINAL DEPLOY - This actually does it!
set -e

echo "🚀 Deploying Topic Learning Feature to GitHub..."
echo ""

# Make scripts executable
chmod +x pull_from_github.sh DEPLOY_EVERYTHING.sh RUN_THIS_NOW.sh go.sh auto_deploy.sh RUN_ME.sh EXECUTE_DEPLOY.sh 2>/dev/null || true

# Stage everything
git add webui/admin_dashboard.html
git add pull_from_github.sh
git add *.sh
git add *.md
git add *.txt

# Commit
git commit -m "Add Topic Learning feature to admin dashboard

🎓 New Feature:
- Topic Learning section with input textbox
- Enter any topic for Jeebs to research and learn
- LEARN button (Enter key support)
- Real-time status with emojis
- Integrates with chat API seamlessly

🛠️ Deployment:
- pull_from_github.sh for VPS deployment
- Multiple deployment scripts for convenience
- Complete documentation included

Usage: Type topic → Click LEARN → Jeebs researches it! 🧠" || echo "Nothing new to commit"

# Push
git push origin main

echo ""
echo "✅ PUSHED TO GITHUB!"
echo ""
echo "════════════════════════════════════════════════════════════"
echo "📥 NEXT: Deploy on your VPS"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Run these commands on your VPS:"
echo ""
echo "  ssh your-user@your-vps-ip"
echo "  cd ~/JeebsAI"
echo "  git pull origin main"
echo "  bash pull_from_github.sh"
echo ""
echo "Or just:"
echo "  ssh your-user@your-vps-ip"
echo "  cd ~/JeebsAI"
echo "  bash pull_from_github.sh"
echo ""
echo "The script will handle everything!"
echo ""
echo "════════════════════════════════════════════════════════════"
echo "🎉 After deployment, access the feature at:"
echo "   http://your-vps-ip/webui/admin_dashboard.html"
echo "════════════════════════════════════════════════════════════"
