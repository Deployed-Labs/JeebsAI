#!/bin/bash
set -euo pipefail

# Install dependencies
if ! command -v git >/dev/null 2>&1; then
  echo "Installing git..." && sudo apt-get update && sudo apt-get install -y git
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..." && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source $HOME/.cargo/env
fi
if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "Installing sqlite3..." && sudo apt-get update && sudo apt-get install -y sqlite3
fi
if ! command -v systemctl >/dev/null 2>&1; then
  echo "Systemd is required. Exiting." && exit 1
fi

# Clone repo if not already present
if [ ! -d "JeebsAI" ]; then
  git clone https://github.com/Deployed-Labs/JeebsAI.git
  cd JeebsAI
else
  cd JeebsAI
  git pull
fi

# Build and install
cargo build --release
sudo cp target/release/jeebs /usr/local/bin/jeebs

# Setup systemd service
sudo cp packaging/jeebs.service /etc/systemd/system/jeebs.service
sudo cp packaging/jeebs.env.example /etc/jeebs.env
sudo systemctl daemon-reload
sudo systemctl enable --now jeebs

echo "JeebsAI deployed and running! View logs: sudo journalctl -u jeebs -f"
