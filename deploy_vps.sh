#!/usr/bin/env bash
# deploy_vps.sh — Deploy JeebsAI to production VPS at /root/JeebsAI
# Usage: Copy this script to the VPS and run:  bash deploy_vps.sh
set -euo pipefail

APP_DIR="/root/JeebsAI"
SERVICE="jeebs"
PORT="8080"
# optional domain for nginx proxy (leave empty for wildcard)
DOMAIN="${DOMAIN:-_}"
DB_FILE="$APP_DIR/jeebs.db"

echo "=== JeebsAI VPS Deploy ==="
echo "Dir:  $APP_DIR"
echo "Port: $PORT"
echo ""

# ── 0. Pre-deploy validation (skip cargo check — we build in step 4) ──
if [ -f "$APP_DIR/validate.sh" ]; then
    echo "[0/7] Running pre-deploy validation..."
    if ! bash "$APP_DIR/validate.sh" --skip-build; then
        echo ""
        echo "ERROR: Validation failed. Fix the issues above before deploying."
        echo "To force deploy anyway: SKIP_VALIDATE=1 bash deploy_vps.sh"
        [ "${SKIP_VALIDATE:-}" = "1" ] || exit 1
    fi
    echo ""
fi

# ── 1. System dependencies + swap ────────────────────────────
echo "[1/7] Installing system dependencies..."
apt-get update -qq
apt-get install -y -qq build-essential pkg-config libssl-dev libsqlite3-dev git curl >/dev/null 2>&1
echo "  Done."

# Ensure swap exists (Rust compilation needs ~1.5GB+ RAM)
if [ ! -f /swapfile ]; then
    echo "  Creating 2GB swap (needed for Rust compilation)..."
    fallocate -l 2G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile >/dev/null
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    echo "  Swap enabled."
elif ! swapon --show | grep -q /swapfile; then
    swapon /swapfile 2>/dev/null || true
    echo "  Swap activated."
fi

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
    git clone https://github.com/Deployed-Labs/JeebsAI.git "$APP_DIR" || {
        echo "ERROR: Could not clone. Make sure $APP_DIR exists with the repo."
        exit 1
    }
    cd "$APP_DIR"
fi

# ── 4. Build release binary ─────────────────────────────────
echo "[4/7] Building release binary (this may take 5-15 minutes on first build)..."
echo "       Compiling with $(nproc 2>/dev/null || echo '?') CPU core(s), $(free -m 2>/dev/null | awk '/Mem:/{print $2}' || echo '?')MB RAM, $(free -m 2>/dev/null | awk '/Swap:/{print $2}' || echo '?')MB swap"
echo ""
cd "$APP_DIR"
CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$(nproc 2>/dev/null || echo 1)} cargo build --release 2>&1 | while IFS= read -r line; do
    # Show Compiling/Downloading/Finished lines live, skip warnings
    case "$line" in
        *Compiling*) echo "  ⚙  $line" ;;
        *Downloading*) echo "  ↓  $line" ;;
        *Finished*) echo "  ✓  $line" ;;
        *error*) echo "  ✗  $line" ;;
        *Downloaded*) echo "  ↓  $line" ;;
    esac
done
if [ ! -f "$APP_DIR/target/release/jeebs" ]; then
    echo "ERROR: Build failed — binary not found."
    echo "Check: free -m  (need ~1.5GB RAM+swap)"
    echo "Try:   CARGO_BUILD_JOBS=1 bash deploy_vps.sh"
    exit 1
fi
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
    # build appropriate site block based on DOMAIN
    if [ "$DOMAIN" = "_" ]; then
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
    else
        cat > /etc/nginx/sites-available/jeebs <<NGINX
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl;
    server_name $DOMAIN www.$DOMAIN;
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:$PORT;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_buffering off;
        proxy_cache off;
    }
}
NGINX
    fi

    ln -sf /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/jeebs
    rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true

    # make sure nginx is running before trying to reload
    systemctl enable --now nginx >/dev/null 2>&1 || true

    # permit common ports if ufw is available
    if command -v ufw &>/dev/null; then
        ufw allow 80,443/tcp || true
    fi

    if nginx -t; then
        if systemctl is-active --quiet nginx; then
            systemctl reload nginx
        else
            systemctl start nginx
        fi
        echo "  Nginx configured."
    else
        echo "  nginx configuration test failed, please inspect /etc/nginx/sites-available/jeebs"
    fi
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
