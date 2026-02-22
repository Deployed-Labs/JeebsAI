#!/usr/bin/env bash
set -euo pipefail

# Quick staging /webui check:
# - Shows port, service status, and whether /webui is wired in source.
# - Lists the deployed webui directory.
# - Probes /webui/ on the configured port.

APP_DIR=${APP_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
ENV_FILE=${ENV_FILE:-"/etc/jeebs-staging.env"}
PORT=${PORT:-"8080"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

if [[ -f "$ENV_FILE" ]]; then
  PORT=$(awk -F= '/^PORT=/{print $2}' "$ENV_FILE" | tail -n 1 || echo "$PORT")
fi

systemctl status "$SERVICE_NAME" --no-pager

grep -n 'Files::new("/webui"' "$APP_DIR/src/main.rs" || true

ls -la "$APP_DIR/webui" || true

curl -I "http://127.0.0.1:${PORT}/webui/"
