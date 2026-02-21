#!/usr/bin/env bash
#
# Deploy Topic Learning Feature - Complete automation
#
set -e

echo "🚀 JeebsAI Topic Learning Deployment"
echo "====================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Step 1: Make pull script executable
echo -e "${BLUE}[1/3]${NC} Making pull_from_github.sh executable..."
chmod +x pull_from_github.sh
chmod +x quick_push.sh

# Step 2: Stage and commit changes
echo -e "${BLUE}[2/3]${NC} Staging and committing changes..."
git add webui/admin_dashboard.html
git add pull_from_github.sh
git add quick_push.sh
git add execute_push.sh

# Check if there are changes to commit
if git diff --cached --quiet; then
    echo -e "${YELLOW}No changes to commit${NC}"
else
    git commit -m "Add Topic Learning textbox to admin dashboard

Features:
- New Topic Learning section with input textbox for entering topics
- Users can type any topic for Jeebs to research and learn about
- LEARN button with orange accent styling to stand out
- Keyboard support: press Enter to trigger learning
- Real-time status feedback with emojis (🧠, ✅, ❌)
- Color-coded messages for success, errors, and progress
- Integrates seamlessly with existing chat API
- Shows preview of Jeebs' learning response

Also includes:
- pull_from_github.sh: Script for VPS to pull latest changes
- Auto-detection of systemd or Docker deployments
- Optional service restart after pulling updates"

    echo -e "${GREEN}✓${NC} Changes committed"
fi

# Step 3: Push to GitHub
echo -e "${BLUE}[3/3]${NC} Pushing to GitHub..."
git push origin main

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           ✅ SUCCESSFULLY PUSHED TO GITHUB!          ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}📋 Next Steps:${NC}"
echo ""
echo "1. SSH into your VPS:"
echo -e "   ${YELLOW}ssh your-user@your-vps-ip${NC}"
echo ""
echo "2. Navigate to JeebsAI directory:"
echo -e "   ${YELLOW}cd ~/JeebsAI${NC}  (or wherever you have it installed)"
echo ""
echo "3. Run the pull script:"
echo -e "   ${YELLOW}./pull_from_github.sh${NC}"
echo ""
echo "   Or if not executable yet:"
echo -e "   ${YELLOW}bash pull_from_github.sh${NC}"
echo ""
echo "4. Access the new feature at:"
echo -e "   ${YELLOW}http://your-vps-ip/webui/admin_dashboard.html${NC}"
echo ""
echo -e "${GREEN}The Topic Learning section is now ready!${NC} 🎓"
echo ""
