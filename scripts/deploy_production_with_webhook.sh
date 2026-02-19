#!/usr/bin/env bash
set -euo pipefail

# One-shot: Rebuild with GitHub webhook support, redeploy to production, and show setup steps.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs"}
APP_DIR=${APP_DIR:-"/root/JeebsAI"}
PORT=${PORT:-"8080"}
ENV_FILE=${ENV_FILE:-"/etc/jeebs.env"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"

echo "Building release binary with GitHub webhook support..."
cargo build --release

mkdir -p "$(dirname "$ENV_FILE")"
if [[ ! -f "$ENV_FILE" ]]; then
  cat >"$ENV_FILE" <<EOF
PORT=$PORT
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
RUST_LOG=info
EOF
fi

echo "Deploying to production..."
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$REPO_DIR/target/release/jeebs"
chmod 755 "$REPO_DIR/target/release/jeebs"

rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager

echo ""
echo "✓ Production deployed with GitHub webhook support."
echo ""
echo "Next: Set up the GitHub webhook by running:"
echo "  sudo ./scripts/setup_github_webhook.sh"
echo ""
