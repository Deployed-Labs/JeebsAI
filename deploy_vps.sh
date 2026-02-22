#!/usr/bin/env bash
# deploy_vps.sh — Deploy JeebsAI to production VPS at /root/JeebsAI
# Usage: Copy this script to the VPS and run:  bash deploy_vps.sh
set -euo pipefail

APP_DIR="/root/JeebsAI"
SERVICE="jeebs"
PORT="8080"
DB_FILE="$APP_DIR/jeebs.db"

echo "=== JeebsAI VPS Deploy ==="
echo "Dir:  $APP_DIR"
echo "Port: $PORT"
echo ""

# ── 1. System dependencies ──────────────────────────────────
echo "[1/7] Installing system dependencies..."
apt-get update -qq
apt-get install -y -qq build-essential pkg-config libssl-dev libsqlite3-dev git curl >/dev/null 2>&1
echo "  Done."

# ── 2. Install Rust if missing ───────────────────────────────
if ! command -v cargo &>/dev/null; then
    echo "[2/7] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "[2/7] Rust already installed."
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# ── 3. Pull latest code ─────────────────────────────────────
if [ -d "$APP_DIR/.git" ]; then
    echo "[3/7] Pulling latest code..."
    cd "$APP_DIR"
    git pull --ff-only || git pull --rebase
else
    echo "[3/7] Cloning repository..."
    git clone https://github.com/1090mb/JeebsAI.git "$APP_DIR" || {
        echo "ERROR: Could not clone. Make sure $APP_DIR exists with the repo."
        exit 1
    }
    cd "$APP_DIR"
fi

# ── 4. Build release binary ─────────────────────────────────
echo "[4/7] Building release binary (this may take a few minutes)..."
cd "$APP_DIR"
cargo build --release 2>&1 | tail -5
echo "  Binary: $APP_DIR/target/release/jeebs"

# ── 5. Create SQLite database if missing ─────────────────────
if [ ! -f "$DB_FILE" ]; then
    echo "[5/7] Creating database..."
    touch "$DB_FILE"
    chmod 664 "$DB_FILE"
else
    echo "[5/7] Database exists."
fi

# ── 6. Create systemd service ───────────────────────────────
echo "[6/7] Setting up systemd service..."
cat > "/etc/systemd/system/${SERVICE}.service" <<UNIT
[Unit]
Description=JeebsAI Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$APP_DIR
Environment=PORT=$PORT
Environment=DATABASE_URL=sqlite:$DB_FILE
Environment=RUST_LOG=info
ExecStart=$APP_DIR/target/release/jeebs
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable "$SERVICE"
echo "  Service created: $SERVICE"

# ── 7. Configure nginx reverse proxy ────────────────────────
echo "[7/7] Configuring nginx..."
if command -v nginx &>/dev/null; then
    cat > /etc/nginx/sites-available/jeebs <<'NGINX'
server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 300s;
        proxy_connect_timeout 75s;

        # Disable buffering for SSE/WebSocket
        proxy_buffering off;
        proxy_cache off;
    }
}
NGINX

    ln -sf /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/jeebs
    rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true
    nginx -t && systemctl reload nginx
    echo "  Nginx configured."
else
    echo "  Nginx not installed — app will listen directly on port $PORT."
    echo "  You can install with: apt install nginx"
fi

# ── Start / restart service ──────────────────────────────────
echo ""
echo "Starting JeebsAI..."
systemctl restart "$SERVICE"
sleep 2

if systemctl is-active --quiet "$SERVICE"; then
    echo ""
    echo "=== JeebsAI is RUNNING ==="
    echo "  Service:  systemctl status $SERVICE"
    echo "  Logs:     journalctl -u $SERVICE -f"
    echo "  Port:     $PORT"
    echo "  WebUI:    http://<your-ip>/webui/index.html"
    echo ""
    curl -s -o /dev/null -w "  Health check: HTTP %{http_code}\n" "http://127.0.0.1:$PORT/health" || echo "  Health check: pending..."
else
    echo ""
    echo "ERROR: Service failed to start. Check logs:"
    echo "  journalctl -u $SERVICE --no-pager -n 30"
    systemctl status "$SERVICE" --no-pager || true
fi
