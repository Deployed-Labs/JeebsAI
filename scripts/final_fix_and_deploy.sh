#!/usr/bin/env bash
set -euo pipefail

# Final fix: Clean up crash loop, rebuild with all fixes, deploy to production.
# - Kills hanging processes
# - Deletes corrupted database
# - Rebuilds the release binary
# - Restarts the service
# - Verifies it's working

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

echo "=== Cleaning up... ==="
pkill -9 -f "target/release/jeebs" || true
sleep 1

echo "=== Removing corrupted database... ==="
rm -f /var/lib/jeebs/jeebs.db /var/lib/jeebs/jeebs.db-shm /var/lib/jeebs/jeebs.db-wal
mkdir -p /var/lib/jeebs

echo "=== Building release binary... ==="
cd "$REPO_DIR"
cargo build --release

echo "=== Ensuring env file is correct... ==="
mkdir -p /var/lib/jeebs
cat > /etc/jeebs.env <<EOF
PORT=8080
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
RUST_LOG=info
EOF

echo "=== Syncing webui assets... ==="
rm -rf "$REPO_DIR/webui"
cp -R "$REPO_DIR/webui" "$REPO_DIR/webui"

echo "=== Restarting service... ==="
systemctl daemon-reload
systemctl restart "$SERVICE_NAME"
sleep 3

echo "=== Verifying... ==="
systemctl status "$SERVICE_NAME" --no-pager

echo ""
echo "✓ Deployment complete."
echo ""
echo "Test login at: http://your-vps-ip:8080/"
echo "After PGP login, you should be redirected to the chat UI."
