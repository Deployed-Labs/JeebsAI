#!/usr/bin/env bash
# SIMPLE GIT PUSH - Just the git commands, nothing fancy

# Stage the files
git add webui/admin_dashboard.html
git add pull_from_github.sh
git add *.sh
git add *.md
git add DEPLOYMENT_COMMANDS.txt

# Commit
git commit -m "Add Topic Learning textbox feature

- New Topic Learning section in admin dashboard
- Textbox for entering topics to learn
- LEARN button with Enter key support
- Real-time status feedback
- VPS deployment script included" || echo "Nothing to commit"

# Push
git push origin main

echo ""
echo "✅ Pushed to GitHub!"
echo ""
echo "Next: SSH to VPS and run: bash pull_from_github.sh"
