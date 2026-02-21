#!/usr/bin/env bash
#
# COMPLETE AUTO-DEPLOY - Runs everything automatically
#

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${BLUE}║    🚀 JeebsAI Topic Learning Auto-Deploy 🚀          ║${NC}"
echo -e "${BOLD}${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if in git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Not a git repository${NC}"
    exit 1
fi

# Make scripts executable
echo -e "${YELLOW}[1/5]${NC} Making scripts executable..."
chmod +x pull_from_github.sh 2>/dev/null || true
chmod +x deploy_now.sh 2>/dev/null || true
chmod +x quick_push.sh 2>/dev/null || true
chmod +x deploy_topic_learning.sh 2>/dev/null || true
echo -e "${GREEN}✓${NC} Scripts are executable"
echo ""

# Stage changes
echo -e "${YELLOW}[2/5]${NC} Staging changes..."
git add webui/admin_dashboard.html
git add pull_from_github.sh
git add deploy_now.sh
git add deploy_topic_learning.sh
git add quick_push.sh
git add TOPIC_LEARNING_DEPLOYMENT.md
git add auto_deploy.sh 2>/dev/null || true
echo -e "${GREEN}✓${NC} Files staged"
echo ""

# Commit
echo -e "${YELLOW}[3/5]${NC} Committing changes..."
if git diff --cached --quiet; then
    echo -e "${BLUE}ℹ${NC}  No changes to commit (already committed)"
else
    git commit -m "Add Topic Learning feature to admin dashboard

✨ New Features:
- Topic Learning section with textbox for entering topics
- LEARN button to trigger research on any subject
- Real-time status updates with emoji feedback
- Keyboard support (Enter key to submit)
- Color-coded success/error messages
- Preview of Jeebs' learning response

🛠️ Deployment Tools:
- pull_from_github.sh: VPS deployment script
- deploy_now.sh: Quick local deployment
- Auto-detection of systemd/Docker deployments
- Comprehensive deployment documentation

🎯 Usage:
Users can now teach Jeebs about any topic directly from the admin
dashboard. Jeebs will research the web and store knowledge in its brain."

    echo -e "${GREEN}✓${NC} Changes committed"
fi
echo ""

# Push
echo -e "${YELLOW}[4/5]${NC} Pushing to GitHub..."
git push origin main
echo -e "${GREEN}✓${NC} Pushed to GitHub"
echo ""

# Summary
echo -e "${YELLOW}[5/5]${NC} Deployment Summary"
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║            ✅  SUCCESSFULLY DEPLOYED!  ✅             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BOLD}📋 Next Steps for VPS Deployment:${NC}"
echo ""
echo -e "1️⃣  SSH into your VPS:"
echo -e "   ${BLUE}ssh your-user@your-vps-ip${NC}"
echo ""
echo -e "2️⃣  Navigate to JeebsAI:"
echo -e "   ${BLUE}cd ~/JeebsAI${NC}"
echo ""
echo -e "3️⃣  Pull and deploy:"
echo -e "   ${BLUE}bash pull_from_github.sh${NC}"
echo ""
echo -e "${BOLD}🎓 Access the feature at:${NC}"
echo -e "   ${YELLOW}http://your-vps-ip/webui/admin_dashboard.html${NC}"
echo ""
echo -e "${BOLD}📚 Documentation:${NC}"
echo -e "   See ${YELLOW}TOPIC_LEARNING_DEPLOYMENT.md${NC} for full details"
echo ""
echo -e "${GREEN}✨ Ready to teach Jeebs new topics! ✨${NC}"
echo ""
