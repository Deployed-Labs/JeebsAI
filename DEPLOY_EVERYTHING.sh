#!/usr/bin/env bash
#
# ONE COMMAND DEPLOYMENT - Does absolutely everything
# Just run: bash DEPLOY_EVERYTHING.sh
#

set -e

clear

cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║         🚀 TOPIC LEARNING FEATURE DEPLOYMENT 🚀             ║
║                                                              ║
║              Pushing to GitHub in 3 seconds...              ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
EOF

sleep 1
echo ""
echo "⏳ Starting deployment..."
sleep 1
echo ""

# Make scripts executable
echo "[1/4] 🔧 Making scripts executable..."
chmod +x pull_from_github.sh
chmod +x RUN_THIS_NOW.sh
chmod +x auto_deploy.sh
chmod +x simple_push.sh
chmod +x setup_deploy.sh
echo "      ✅ Done"
echo ""

# Stage all files
echo "[2/4] 📦 Staging files..."
git add webui/admin_dashboard.html
git add pull_from_github.sh
git add RUN_THIS_NOW.sh
git add auto_deploy.sh
git add simple_push.sh
git add DEPLOY_EVERYTHING.sh
git add setup_deploy.sh
git add PUSH_TO_GITHUB.md
git add TOPIC_LEARNING_DEPLOYMENT.md
git add DEPLOYMENT_COMMANDS.txt
git add START_HERE.txt
echo "      ✅ Done"
echo ""

# Commit
echo "[3/4] 💾 Committing..."
if git diff --cached --quiet; then
    echo "      ℹ️  No changes to commit"
else
    git commit -m "Add Topic Learning feature to admin dashboard

🎓 New Feature:
- Topic Learning section in admin dashboard
- Input textbox for entering topics
- LEARN button to trigger research (Enter key support)
- Real-time status updates with emoji feedback
- Color-coded success/error messages
- Preview of Jeebs' learning response
- Seamless integration with chat API

🛠️ Deployment Tools:
- DEPLOY_EVERYTHING.sh - One-command deployment
- RUN_THIS_NOW.sh - Simple push to GitHub
- pull_from_github.sh - VPS deployment script
- auto_deploy.sh - Full automation
- Comprehensive documentation

📚 Documentation:
- START_HERE.txt - Quick start guide
- PUSH_TO_GITHUB.md - Push reference
- TOPIC_LEARNING_DEPLOYMENT.md - Full guide
- DEPLOYMENT_COMMANDS.txt - Copy-paste commands

Users can now teach Jeebs about any topic directly from the
admin dashboard. Jeebs will research the web and store
knowledge in its brain automatically."

    echo "      ✅ Done"
fi
echo ""

# Push
echo "[4/4] ⬆️  Pushing to GitHub..."
git push origin main
echo "      ✅ Done"
echo ""

# Success message
cat << "EOF"

╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║               ✅  SUCCESSFULLY DEPLOYED!  ✅                ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

EOF

echo "📋 Next Steps for VPS:"
echo ""
echo "  1. SSH to your VPS:"
echo "     ssh your-user@your-vps-ip"
echo ""
echo "  2. Go to JeebsAI directory:"
echo "     cd ~/JeebsAI"
echo ""
echo "  3. Pull and deploy:"
echo "     bash pull_from_github.sh"
echo ""
echo "  4. Access the feature:"
echo "     http://your-vps-ip/webui/admin_dashboard.html"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "🎓 Look for the orange 'Topic Learning' section!"
echo ""
echo "   Enter any topic → Click LEARN → Watch Jeebs learn! 🧠"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
