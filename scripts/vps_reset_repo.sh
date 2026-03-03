#!/bin/bash
#
# EMERGENCY SCRIPT: Wipes the current installation and re-clones from scratch.
# Run this on the VPS if git is corrupted or stuck.
#
set -e

APP_DIR="/root/JeebsAI"
DEFAULT_REPO="https://github.com/Deployed-Labs/JeebsAI.git"

echo "⚠️  WARNING: This will delete $APP_DIR and re-clone it."
echo "   Your database at /var/lib/jeebs/jeebs.db will be SAFE."
echo ""

read -p "Enter GitHub Token (leave empty if public or SSH configured): " TOKEN

REPO_URL="$DEFAULT_REPO"
if [ -n "$TOKEN" ]; then
    REPO_URL="https://oauth2:$TOKEN@github.com/Deployed-Labs/JeebsAI.git"
fi

echo "🛑 Stopping service..."
systemctl stop jeebs

echo "💾 Backing up DB just in case..."
cp /var/lib/jeebs/jeebs.db /var/backups/jeebs/jeebs_reset_$(date +%s).db || echo "No DB to backup"

echo "🗑️  Wiping directory..."
rm -rf "$APP_DIR"

echo "⬇️  Cloning fresh..."
git clone "$REPO_URL" "$APP_DIR"

echo "✅ Done. You can now run: cd $APP_DIR && ./deploy_to_vps.sh"