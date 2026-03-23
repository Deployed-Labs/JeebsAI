#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
  echo "Run as root: sudo ./scripts/install_service.sh" >&2
  exit 2
fi

cp packaging/jeebs-docker.service /etc/systemd/system/jeebs-docker.service
systemctl daemon-reload
systemctl enable --now jeebs-docker.service
echo "jeebs-docker.service installed and started."
