#!/usr/bin/env bash
#
# Quick Push - Push changes to GitHub
#
set -e

echo "🚀 Pushing Topic Learning Feature to GitHub"
echo "==========================================="

# Add all changes
echo "📦 Staging changes..."
git add webui/admin_dashboard.html

# Commit
echo "💾 Committing..."
git commit -m "Add Topic Learning textbox to admin dashboard

- Added new Topic Learning section with input textbox
- Users can enter any topic for Jeebs to research and learn
- Added LEARN button with orange accent styling
- Keyboard support (Enter key) for quick learning
- Real-time status feedback with emojis and color coding
- Integrates with existing chat API for seamless learning"

# Push to GitHub
echo "⬆️  Pushing to GitHub..."
git push origin main

echo ""
echo "✅ Successfully pushed to GitHub!"
echo ""
echo "Next step: Run pull_from_github.sh on your VPS to deploy"
