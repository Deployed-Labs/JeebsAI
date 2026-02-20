#!/usr/bin/env bash
# Install JeebsAI from a release tarball.
# Run this script from the directory containing the extracted tarball contents:
#   tar -xzf jeebs-v1.0.0-linux-x86_64.tar.gz
#   cd jeebs-v1.0.0-linux-x86_64
#   sudo ./install.sh
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "This installer must be run as root. Use: sudo ./install.sh" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_PATH="/usr/local/bin/jeebs"
SERVICE_DEST="/etc/systemd/system/jeebs.service"
ENV_DEST="/etc/jeebs.env"

if [ ! -f "${SCRIPT_DIR}/jeebs" ]; then
  echo "ERROR: binary 'jeebs' not found in ${SCRIPT_DIR}" >&2
  exit 1
fi

echo "Installing JeebsAI..."

install -m 755 "${SCRIPT_DIR}/jeebs" "${BIN_PATH}"

mkdir -p /var/lib/jeebs/plugins

if [ ! -f "${ENV_DEST}" ]; then
  echo "Installing environment file to ${ENV_DEST}"
  install -m 640 "${SCRIPT_DIR}/jeebs.env.example" "${ENV_DEST}"
fi

echo "Installing systemd unit: ${SERVICE_DEST}"
install -m 644 "${SCRIPT_DIR}/jeebs.service" "${SERVICE_DEST}"

systemctl daemon-reload
systemctl enable --now jeebs

echo "JeebsAI installed and started."
echo "Edit ${ENV_DEST} to customize the configuration."
echo "Follow logs with: sudo journalctl -u jeebs -f"
