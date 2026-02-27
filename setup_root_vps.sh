#!/bin/bash
# Setup script to link JeebsAI to /root/JeebsAI

APP_DIR="/root/JeebsAI"
SERVICE_FILE="/etc/systemd/system/jeebs.service"
ENV_FILE="/etc/jeebs.env"

echo "🔧 Configuring JeebsAI for operating folder: $APP_DIR"

# 1. Update systemd service to point to /root/JeebsAI
echo "Updating systemd service..."
cat > $SERVICE_FILE <<EOF
[Unit]
Description=JeebsAI Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/target/release/jeebs
EnvironmentFile=-$ENV_FILE
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 2. Reload systemd
systemctl daemon-reload

# 3. Ensure executable permissions
if [ -f "$APP_DIR/target/release/jeebs" ]; then
    chmod +x "$APP_DIR/target/release/jeebs"
fi

echo "✅ Configuration complete. Run 'sudo systemctl restart jeebs' to apply."