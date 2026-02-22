#!/usr/bin/env bash
set -euo pipefail

# Rebuild and redeploy staging with /webui static route.
# - Rebuilds the release binary.
# - Replaces the running staging binary.
# - Syncs webui assets into the service working directory.
# - Restarts the service and probes /webui/.

APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
REPO_DIR=${REPO_DIR:-"$APP_DIR"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
PORT=${PORT:-"8080"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"

# Quick sanity check: ensure the /webui static route exists in source.
if ! grep -q 'Files::new("/webui"' src/main.rs; then
  echo "ERROR: /webui static route not found in src/main.rs" >&2
  exit 1
fi

cargo build --release

mkdir -p "$APP_DIR/target/release"
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$APP_DIR/target/release/jeebs"
chmod 755 "$APP_DIR/target/release/jeebs"

rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
curl -I "http://127.0.0.1:${PORT}/webui/"
