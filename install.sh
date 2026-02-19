#!/bin/bash

# JeebsAI Service Installer
# Run this script to install JeebsAI as a systemd service.

set -e

# 1. Ensure Rust toolchain (install/update when needed) and build
REQUIRED_RUST_MAJOR=1
REQUIRED_RUST_MINOR=88

echo "Checking Rust toolchain (requires >= ${REQUIRED_RUST_MAJOR}.${REQUIRED_RUST_MINOR})..."
if command -v rustc >/dev/null 2>&1; then
  ver=$(rustc --version | awk '{print $2}')
  major=$(echo "$ver" | cut -d. -f1)
  minor=$(echo "$ver" | cut -d. -f2)
else
  ver="0.0.0"
  major=0
  minor=0
fi

if [ "$major" -lt "$REQUIRED_RUST_MAJOR" ] || { [ "$major" -eq "$REQUIRED_RUST_MAJOR" ] && [ "$minor" -lt "$REQUIRED_RUST_MINOR" ]; }; then
  echo "Rust $ver is too old or missing — installing rustup + Rust ${REQUIRED_RUST_MAJOR}.${REQUIRED_RUST_MINOR}+..."
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
  rustup toolchain install 1.88.0
  rustup default 1.88.0
else
  echo "Found rustc $ver"
fi

# Install required native build dependencies (nettle / sequoia, compilers, pkg-config)
if command -v apt-get >/dev/null 2>&1; then
  echo "Installing native build dependencies via apt..."
  sudo apt-get update
  sudo apt-get install -y build-essential clang pkg-config libssl-dev sqlite3 git curl ca-certificates \
    nettle-dev libgpg-error-dev libgcrypt-dev
fi

# Quick verification of native dependencies required to compile Sequoia/nettle
verify_native_deps() {
  local missing=()

  if ! command -v pkg-config >/dev/null 2>&1; then
    missing+=("pkg-config")
  fi

  if ! command -v clang >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
    missing+=("clang/gcc")
  fi

  # check libraries used by sequoia/nettle
  for lib in nettle gpg-error libgcrypt; do
    # accept either 'libgcrypt' or legacy 'gcrypt' names where present
    if [ "$lib" = "libgcrypt" ]; then
      if ! pkg-config --exists libgcrypt 2>/dev/null && ! pkg-config --exists gcrypt 2>/dev/null; then
        missing+=("pkg-config:libgcrypt")
      fi
    else
      if ! pkg-config --exists "$lib" 2>/dev/null; then
        missing+=("pkg-config:$lib")
      fi
    fi
  done

  if [ ${#missing[@]} -ne 0 ]; then
    echo "\nERROR: missing native build dependencies: ${missing[*]}\n"
    echo "On Debian/Ubuntu install with:"
    echo "  sudo apt update && sudo apt install -y build-essential clang pkg-config nettle-dev libgpg-error-dev libgcrypt-dev"
    exit 1
  fi
  echo "Native build dependencies verified."
}

verify_native_deps

echo "Building JeebsAI (Release)..."
cargo build --release

# 2. Gather Configuration
SERVICE_NAME="jeebs"
ENV_FILE_PATH="/etc/jeebs/config.env"
CURRENT_USER=$(whoami)
# This line gets the absolute path of the directory where the script is located.
WORK_DIR=$(cd "$(dirname "$0")" && pwd)
EXEC_PATH="$WORK_DIR/target/release/jeebs"

# If running via sudo, try to detect the actual user who invoked sudo
if [ "$CURRENT_USER" == "root" ] && [ -n "$SUDO_USER" ]; then
    TARGET_USER="$SUDO_USER"
else
    TARGET_USER="$CURRENT_USER"
fi

echo "------------------------------------------------"
echo "Installing service '$SERVICE_NAME'"
echo "User:              $TARGET_USER"
echo "Working Directory: $WORK_DIR"
echo "Executable:        $EXEC_PATH"
echo "Environment File:  $ENV_FILE_PATH"
echo "------------------------------------------------"

# 3. Generate Service File
SERVICE_CONTENT="[Unit]
Description=JeebsAI Server
After=network.target

[Service]
Type=simple
User=$TARGET_USER
WorkingDirectory=$WORK_DIR
ExecStart=$EXEC_PATH
EnvironmentFile=-$ENV_FILE_PATH
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target"

# 4. Create Environment File (if it doesn't exist)
echo "Checking for environment file at $ENV_FILE_PATH..."
if [ ! -f "$ENV_FILE_PATH" ]; then
    echo "Environment file not found. Creating a default one..."
    sudo mkdir -p "$(dirname "$ENV_FILE_PATH")"
    # Create a default environment file with common variables
    echo "# Environment variables for JeebsAI
PORT=8080
DATABASE_URL=sqlite:jeebs.db
# RUST_LOG=info,actix_web=info # Uncomment to set log levels
" | sudo tee "$ENV_FILE_PATH" > /dev/null
    echo "Default environment file created. You can edit it at $ENV_FILE_PATH."
fi

# 5. Write to /etc/systemd/system (requires sudo)
echo "Writing service file to /etc/systemd/system/$SERVICE_NAME.service..."
echo "$SERVICE_CONTENT" | sudo tee /etc/systemd/system/$SERVICE_NAME.service > /dev/null

# 6. Enable and Start
echo "Reloading systemd..."
sudo systemctl daemon-reload
echo "Enabling service..."
sudo systemctl enable $SERVICE_NAME
echo "Restarting service..."
sudo systemctl restart $SERVICE_NAME

echo "Done! Check status with: sudo systemctl status $SERVICE_NAME"