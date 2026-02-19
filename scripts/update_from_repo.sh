#!/usr/bin/env bash
set -euo pipefail

# Update JeebsAI on the VPS from the GitHub repo.
# - Pulls latest code.
# - Rebuilds release binary.
# - Restarts the jeebs service.
# - Syncs webui assets.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs"}
APP_DIR=${APP_DIR:-"/root/JeebsAI"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"

echo "Pulling latest code from GitHub..."
git fetch origin
git pull origin main

echo "Building release binary..."
cargo build --release

echo "Syncing webui assets..."
rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

echo "Restarting service..."
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$REPO_DIR/target/release/jeebs"
chmod 755 "$REPO_DIR/target/release/jeebs"
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
echo "✓ Update complete."
