#!/bin/bash

# JeebsAI Service Installer
# Run this script to install JeebsAI as a systemd service.

set -e

# 1. Build the project first
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