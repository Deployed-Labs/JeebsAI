#!/usr/bin/env bash
set -euo pipefail

# Deploy staging build to production jeebs service.
# - Builds release binary.
# - Replaces the production binary.
# - Syncs webui assets into the production working directory.
# - Restarts jeebs and probes the chat UI.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs"}
APP_DIR=${APP_DIR:-"/root/JeebsAI"}
PORT=${PORT:-"8080"}
ENV_FILE=${ENV_FILE:-"/etc/jeebs.env"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"
cargo build --release

# If production env file exists, sync it; otherwise use defaults.
mkdir -p "$(dirname "$ENV_FILE")"
if [[ ! -f "$ENV_FILE" ]]; then
  cat >"$ENV_FILE" <<EOF
PORT=$PORT
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
RUST_LOG=info
EOF
fi

systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$REPO_DIR/target/release/jeebs"
chmod 755 "$REPO_DIR/target/release/jeebs"

rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
curl -I "http://127.0.0.1:${PORT}/webui/"
