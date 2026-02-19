#!/bin/bash
set -euo pipefail

# Install sshpass if not present
if ! command -v sshpass >/dev/null 2>&1; then
  brew install hudochenkov/sshpass/sshpass
fi

# Run the deploy script
./scripts/deploy_to_server.sh root@192.227.193.148 /opt/jeebs -p '0N8m0X70HFpFC5imce'
