#!/usr/bin/env bash
set -e

# Make scripts executable
chmod +x DEPLOY_EVERYTHING.sh RUN_THIS_NOW.sh pull_from_github.sh go.sh auto_deploy.sh RUN_ME.sh 2>/dev/null || true

# Add all files
git add -A

# Commit
git commit -m "Add Topic Learning feature with complete deployment system

🎓 Topic Learning Feature:
- New section in admin dashboard with textbox for entering topics
- LEARN button triggers research on any subject
- Real-time status updates with emoji feedback (🧠, ✅, ❌)
- Keyboard support (Enter key to submit)
- Color-coded success/error messages
- Preview of Jeebs' learning response
- Seamless integration with existing chat API

🚀 Complete Deployment System:
- DEPLOY_EVERYTHING.sh - Full automation
- RUN_THIS_NOW.sh - Simple push script
- pull_from_github.sh - VPS deployment with auto-restart
- RUN_ME.sh - Ultimate simple launcher
- go.sh - One-liner deployment

📚 Comprehensive Documentation:
- FINAL_INSTRUCTIONS.txt - Step-by-step guide
- README_DEPLOY.txt - Detailed instructions
- START_HERE.txt - Quick start
- PUSH_TO_GITHUB.md - Push reference
- TOPIC_LEARNING_DEPLOYMENT.md - Complete guide
- DEPLOYMENT_COMMANDS.txt - Copy-paste commands
- INSTRUCTIONS.txt - Quick reference

Users can now teach Jeebs about any topic directly from the admin
dashboard. Simply type a topic, click LEARN, and Jeebs will research
the web and store knowledge automatically." || echo "Already committed"

# Push
git push origin main

echo ""
echo "════════════════════════════════════════════════════════════"
echo "✅ SUCCESSFULLY PUSHED TO GITHUB!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📥 Next: Deploy on VPS"
echo ""
echo "SSH to VPS and run:"
echo "  cd ~/JeebsAI"
echo "  bash pull_from_github.sh"
echo ""
echo "Then access at:"
echo "  http://your-vps-ip/webui/admin_dashboard.html"
echo ""
echo "════════════════════════════════════════════════════════════"
