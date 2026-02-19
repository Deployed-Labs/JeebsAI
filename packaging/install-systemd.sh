#!/usr/bin/env bash
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "This installer must be run as root. Use sudo." >&2
  exit 1
fi

BIN_PATH="/usr/local/bin/jeebs"
SERVICE_SRC="$(pwd)/packaging/jeebs.service"
ENV_SRC="$(pwd)/packaging/jeebs.env.example"

echo "Installing Jeebs systemd service..."

if [ ! -f target/release/jeebs ]; then
  echo "Release binary not found. Run: cargo build --release" >&2
  exit 1
fi

install -m 755 target/release/jeebs "$BIN_PATH"

mkdir -p /var/lib/jeebs/plugins
cp -r webui /var/lib/jeebs/webui 2>/dev/null || true

# Install environment file if one does not already exist
if [ ! -f /etc/jeebs.env ]; then
  echo "Installing environment file to /etc/jeebs.env"
  cp "$ENV_SRC" /etc/jeebs.env
  chmod 640 /etc/jeebs.env
fi

# Install systemd unit
echo "Installing systemd unit: /etc/systemd/system/jeebs.service"
cp "$SERVICE_SRC" /etc/systemd/system/jeebs.service
systemctl daemon-reload
systemctl enable --now jeebs

echo "Jeebs service installed and started. Follow logs with: sudo journalctl -u jeebs -f"
