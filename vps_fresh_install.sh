#!/usr/bin/env bash
#
# JeebsAI VPS Fresh Installation Script
# This script clones the repo and sets up JeebsAI from scratch on a VPS
#
set -euo pipefail

echo "🚀 JeebsAI VPS Fresh Installation"
echo "=================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration (edit these as needed)
REPO_URL="${REPO_URL:-https://github.com/Deployed-Labs/JeebsAI.git}"
APP_DIR="${APP_DIR:-/opt/jeebs}"
APP_USER="${APP_USER:-root}"
APP_PORT="${APP_PORT:-8080}"
DB_PATH="${DB_PATH:-/var/lib/jeebs/jeebs.db}"
SERVICE_NAME="jeebs"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   error "This script must be run as root (use sudo)"
   exit 1
fi

info "Installation Configuration:"
echo "  Repository: $REPO_URL"
echo "  Install Directory: $APP_DIR"
echo "  Database Path: $DB_PATH"
echo "  Port: $APP_PORT"
echo "  Service User: $APP_USER"
echo ""

read -p "Continue with installation? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    info "Installation cancelled"
    exit 0
fi

echo ""

# Step 1: Update system
info "Updating system packages..."
apt-get update -qq
success "System updated"

echo ""

# Step 2: Install system dependencies
info "Installing system dependencies..."
apt-get install -y -qq \
    build-essential \
    pkg-config \
    libssl-dev \
    sqlite3 \
    git \
    curl \
    wget \
    ca-certificates \
    > /dev/null 2>&1

success "System dependencies installed"

echo ""

# Step 3: Install Rust (if not already installed)
if ! command -v cargo &> /dev/null; then
    info "Installing Rust..."

    # Install for the specified user
    if [ "$APP_USER" = "root" ]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        sudo -u "$APP_USER" bash -c 'curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
        sudo -u "$APP_USER" bash -c 'source $HOME/.cargo/env'
    fi

    # Add to PATH for this script
    export PATH="$HOME/.cargo/bin:$PATH"

    success "Rust installed successfully"
else
    info "Rust already installed ($(cargo --version))"
fi

echo ""

# Step 4: Clone repository
if [ -d "$APP_DIR" ]; then
    warn "Directory $APP_DIR already exists"
    read -p "Remove and re-clone? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Removing existing directory..."
        rm -rf "$APP_DIR"
        success "Directory removed"
    else
        info "Using existing directory"
    fi
fi

if [ ! -d "$APP_DIR" ]; then
    info "Cloning repository from $REPO_URL..."
    git clone "$REPO_URL" "$APP_DIR"
    success "Repository cloned"
else
    info "Updating existing repository..."
    cd "$APP_DIR"
    git fetch origin
    git checkout main
    git pull origin main
    success "Repository updated"
fi

echo ""

# Step 5: Create database directory
info "Creating database directory..."
mkdir -p "$(dirname "$DB_PATH")"
mkdir -p /var/backups/jeebs
chmod 755 "$(dirname "$DB_PATH")"
success "Database directory created"

echo ""

# Step 6: Run database migrations
info "Setting up database..."
cd "$APP_DIR"

if [ -d "migrations" ]; then
    for migration in migrations/*.sql; do
        if [ -f "$migration" ]; then
            info "Running migration: $(basename "$migration")"
            sqlite3 "$DB_PATH" < "$migration" 2>/dev/null || warn "Migration may have already been applied"
        fi
    done
    success "Database migrations complete"
else
    warn "No migrations directory found"
fi

# Ensure database permissions
chmod 644 "$DB_PATH" 2>/dev/null || true

echo ""

# Step 7: Build the application
info "Building JeebsAI (this may take several minutes)..."
cd "$APP_DIR"

# Set proper ownership if not root
if [ "$APP_USER" != "root" ]; then
    chown -R "$APP_USER:$APP_USER" "$APP_DIR"
fi

# Build release binary
if [ "$APP_USER" = "root" ]; then
    cargo build --release
else
    sudo -u "$APP_USER" bash -c "source \$HOME/.cargo/env && cd $APP_DIR && cargo build --release"
fi

success "Build complete!"

echo ""

# Step 8: Create systemd service
info "Creating systemd service..."

cat > /etc/systemd/system/${SERVICE_NAME}.service <<EOF
[Unit]
Description=JeebsAI - Intelligent Assistant
After=network.target

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
Environment="DATABASE_PATH=$DB_PATH"
Environment="BIND_ADDRESS=0.0.0.0:$APP_PORT"
Environment="RUST_LOG=info"
ExecStart=$APP_DIR/target/release/jeebs
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

success "Systemd service created"

echo ""

# Step 9: Enable and start service
info "Enabling and starting service..."

systemctl daemon-reload
systemctl enable ${SERVICE_NAME}
systemctl start ${SERVICE_NAME}

# Wait for service to start
sleep 3

if systemctl is-active --quiet ${SERVICE_NAME}; then
    success "✅ Service started successfully!"
else
    error "Service failed to start"
    info "Checking logs..."
    journalctl -u ${SERVICE_NAME} -n 20 --no-pager
    exit 1
fi

echo ""

# Step 10: Check service status
info "Service Status:"
systemctl status ${SERVICE_NAME} --no-pager -l || true

echo ""

# Step 11: Display recent logs
info "Recent Logs:"
journalctl -u ${SERVICE_NAME} -n 10 --no-pager || true

echo ""

# Step 12: Health check
info "Performing health check..."
sleep 2

if curl -f http://localhost:$APP_PORT/webui/index.html > /dev/null 2>&1; then
    success "✅ Health check passed! JeebsAI is responding on port $APP_PORT"
else
    warn "⚠️  Health check inconclusive. Service may still be starting up."
fi

echo ""

# Step 13: Display firewall info
info "Firewall Configuration:"
if command -v ufw &> /dev/null; then
    if ufw status | grep -q "Status: active"; then
        warn "UFW firewall is active. You may need to allow port $APP_PORT:"
        echo "  sudo ufw allow $APP_PORT/tcp"
    fi
else
    info "UFW not installed. If using another firewall, ensure port $APP_PORT is open."
fi

echo ""

# Final summary
success "=========================================="
success "🎉 Installation Complete!"
success "=========================================="
echo ""

info "JeebsAI is installed and running!"
echo ""
echo "📍 Installation Details:"
echo "  • Installation directory: $APP_DIR"
echo "  • Database location: $DB_PATH"
echo "  • Service name: $SERVICE_NAME"
echo "  • Port: $APP_PORT"
echo ""
echo "🔧 Useful Commands:"
echo "  • Check status:     sudo systemctl status $SERVICE_NAME"
echo "  • View logs:        sudo journalctl -u $SERVICE_NAME -f"
echo "  • Restart service:  sudo systemctl restart $SERVICE_NAME"
echo "  • Stop service:     sudo systemctl stop $SERVICE_NAME"
echo ""
echo "🌐 Access JeebsAI:"
echo "  • Local:  http://localhost:$APP_PORT"
echo "  • Remote: http://YOUR_VPS_IP:$APP_PORT"
echo ""
echo "📚 Next Steps:"
echo "  1. Configure your domain (optional)"
echo "  2. Set up SSL with nginx/certbot (recommended)"
echo "  3. Create your first admin user"
echo "  4. Start chatting with Jeebs!"
echo ""
echo "🔄 To update in the future:"
echo "  cd $APP_DIR && sudo ./deploy_to_vps.sh"
echo ""

# Save installation info
cat > "$APP_DIR/INSTALLATION_INFO.txt" <<EOF
JeebsAI Installation Information
================================
Installed: $(date)
Repository: $REPO_URL
Directory: $APP_DIR
Database: $DB_PATH
Port: $APP_PORT
Service: $SERVICE_NAME
User: $APP_USER

Commands:
  Status:  sudo systemctl status $SERVICE_NAME
  Logs:    sudo journalctl -u $SERVICE_NAME -f
  Restart: sudo systemctl restart $SERVICE_NAME
EOF

success "Installation info saved to $APP_DIR/INSTALLATION_INFO.txt"

echo ""
info "Installation log completed successfully!"

exit 0
