#!/usr/bin/env bash
# Lightweight deploy helper for JeebsAI
# Usage on the VPS after cloning the repo:
#   chmod +x scripts/deploy.sh
#   sudo ./scripts/deploy.sh --path /home/ubuntu/JeebsAI --port 8080 --service jeebs

set -euo pipefail

REPO_PATH="$(pwd)"
PORT=8080
SERVICE_NAME=jeebs
USE_SYSTEMD=true

print_usage() {
  cat <<EOF
Usage: $0 [--path /path/to/repo] [--port PORT] [--service name] [--no-systemd]

This script pulls latest changes, builds in release mode, and restarts the service.
If systemd is available and a service with the given name exists, it will use systemd.
Otherwise it will kill any process listening on the port and run the binary in background.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path) REPO_PATH="$2"; shift 2;;
    --port) PORT="$2"; shift 2;;
    --service) SERVICE_NAME="$2"; shift 2;;
    --no-systemd) USE_SYSTEMD=false; shift 1;;
    -h|--help) print_usage; exit 0;;
    *) echo "Unknown arg: $1"; print_usage; exit 2;;
  esac
done

echo "Deploy: repo=${REPO_PATH}, port=${PORT}, service=${SERVICE_NAME}, systemd=${USE_SYSTEMD}"

cd "$REPO_PATH"

echo "Pulling latest from origin/main..."
git fetch origin
git checkout main
git pull origin main

echo "Building release..."
cargo build --release

if $USE_SYSTEMD && command -v systemctl >/dev/null 2>&1; then
  if systemctl list-units --type=service --all | grep -q "${SERVICE_NAME}.service"; then
    echo "Stopping systemd service ${SERVICE_NAME}..."
    sudo systemctl stop ${SERVICE_NAME}.service || true
    echo "Starting systemd service ${SERVICE_NAME}..."
    sudo systemctl daemon-reload || true
    sudo systemctl start ${SERVICE_NAME}.service
    sudo journalctl -u ${SERVICE_NAME}.service -n 100 --no-pager || true
    echo "Service restarted via systemd."
    exit 0
  fi
fi

echo "No systemd unit found or disabled; killing any process on port ${PORT}..."
pids=$(lsof -ti:"${PORT}" || true)
if [ -n "$pids" ]; then
  echo "Killing: $pids"
  echo "$pids" | xargs -r kill -9
fi

BIN_PATH="$REPO_PATH/target/release/jeebs"
if [ ! -x "$BIN_PATH" ]; then
  echo "Binary not found at $BIN_PATH"; exit 3
fi

echo "Starting binary in background (PORT=${PORT})..."
nohup env PORT=${PORT} "$BIN_PATH" >> "$REPO_PATH/jeebs.log" 2>&1 &
echo "Started. Logs: $REPO_PATH/jeebs.log"

exit 0
