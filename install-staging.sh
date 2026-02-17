#!/bin/bash

# JeebsAI Staging Environment Setup Script
# Run this on your VPS to prepare for the staging CI/CD pipeline.

set -e

# Configuration
SERVICE_NAME="jeebs-staging"
APP_DIR="/opt/jeebs-staging"
ENV_FILE_PATH="/etc/jeebs-staging/config.env"
STAGING_PORT=8081

# Detect User
CURRENT_USER=$(whoami)
if [ "$CURRENT_USER" == "root" ] && [ -n "$SUDO_USER" ]; then
    TARGET_USER="$SUDO_USER"
else
    TARGET_USER="$CURRENT_USER"
fi

echo "------------------------------------------------"
echo "Setting up '$SERVICE_NAME'"
echo "User:              $TARGET_USER"
echo "App Directory:     $APP_DIR"
echo "Port:              $STAGING_PORT"
echo "------------------------------------------------"

# 1. Create Directory Structure
echo "Creating application directories..."
# We need to create the directory where the binary will live so SCP doesn't fail
sudo mkdir -p "$APP_DIR/target/release"
# Set ownership to the deployment user
sudo chown -R "$TARGET_USER:$TARGET_USER" "$APP_DIR"

# 2. Create Environment File
echo "Checking for environment file at $ENV_FILE_PATH..."
if [ ! -f "$ENV_FILE_PATH" ]; then
    echo "Creating default staging environment file..."
    sudo mkdir -p "$(dirname "$ENV_FILE_PATH")"
    echo "# Environment variables for JeebsAI Staging
PORT=$STAGING_PORT
DATABASE_URL=sqlite:jeebs.db
# RUST_LOG=info,actix_web=info
" | sudo tee "$ENV_FILE_PATH" > /dev/null
    echo "Created $ENV_FILE_PATH"
else
    echo "Environment file already exists."
fi

# 3. Generate Service File
SERVICE_CONTENT="[Unit]
Description=JeebsAI Staging Server
After=network.target

[Service]
Type=simple
User=$TARGET_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/target/release/jeebs
EnvironmentFile=-$ENV_FILE_PATH
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target"

# 4. Write Service File
echo "Writing service file to /etc/systemd/system/$SERVICE_NAME.service..."
echo "$SERVICE_CONTENT" | sudo tee "/etc/systemd/system/$SERVICE_NAME.service" > /dev/null

# 5. Enable Service
echo "Reloading systemd..."
sudo systemctl daemon-reload
echo "Enabling service..."
sudo systemctl enable "$SERVICE_NAME"

echo "------------------------------------------------"
echo "Setup complete!"
echo "1. The service '$SERVICE_NAME' is enabled."
echo "2. It will fail to start until the CI/CD pipeline deploys the binary."
echo "3. Ensure '$TARGET_USER' has passwordless sudo permissions for systemctl commands."
echo "------------------------------------------------"