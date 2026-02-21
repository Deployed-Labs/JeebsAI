#!/usr/bin/env bash
#
# ONE-LINE DEPLOYMENT
# Run this command: bash deploy_now.sh
#

set -e

echo "🚀 Deploying Topic Learning Feature..."

# Make sure we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Not in JeebsAI directory"
    exit 1
fi

# Add all new/modified files
git add webui/admin_dashboard.html pull_from_github.sh deploy_topic_learning.sh quick_push.sh 2>/dev/null || true

# Commit if there are changes
if ! git diff --cached --quiet 2>/dev/null; then
    git commit -m "Add Topic Learning textbox to admin dashboard"
fi

# Push to GitHub
git push origin main

echo ""
echo "✅ PUSHED TO GITHUB!"
echo ""
echo "📥 To deploy on VPS, run:"
echo "   bash pull_from_github.sh"
echo ""
