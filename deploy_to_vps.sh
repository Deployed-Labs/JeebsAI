#!/usr/bin/env bash
#
# JeebsAI VPS Deployment Script
# This script pulls the latest main branch and rebuilds JeebsAI on the VPS
#
set -euo pipefail

echo "🚀 JeebsAI VPS Deployment Script"
echo "================================"
echo ""

# Configuration
APP_DIR="${APP_DIR:-/opt/jeebs}"
REPO_URL="${REPO_URL:-https://github.com/Deployed-Labs/JeebsAI.git}"
SERVICE_NAME="jeebs"
BACKUP_DIR="/var/backups/jeebs"
DB_PATH="${DB_PATH:-/var/lib/jeebs/jeebs.db}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   error "This script must be run as root (use sudo)"
   exit 1
fi

# Backup database before deployment
backup_database() {
    info "Creating database backup..."

    mkdir -p "$BACKUP_DIR"

    if [ -f "$DB_PATH" ]; then
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        cp "$DB_PATH" "$BACKUP_DIR/jeebs_${TIMESTAMP}.db"
        success "Database backed up to $BACKUP_DIR/jeebs_${TIMESTAMP}.db"

        # Keep only last 10 backups
        cd "$BACKUP_DIR"
        ls -t jeebs_*.db | tail -n +11 | xargs -r rm
        info "Cleaned up old backups (keeping last 10)"
    else
        warn "No existing database found at $DB_PATH"
    fi
}

# Stop the service
stop_service() {
    info "Stopping JeebsAI service..."

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        systemctl stop "$SERVICE_NAME"
        success "Service stopped"
    else
        warn "Service was not running"
    fi
}

# Pull latest code
pull_code() {
    info "Pulling latest code from main branch..."

    cd "$APP_DIR"

    # Stash any local changes
    if [ -d .git ]; then
        git stash push -m "Auto-stash before deployment $(date)"
        git fetch origin
        git checkout main
        git pull origin main
        success "Code updated to latest main branch"
    else
        error "Not a git repository. Please ensure $APP_DIR is a git clone."
        exit 1
    fi
}

# Build the application
build_app() {
    info "Building JeebsAI (release mode)..."

    cd "$APP_DIR"

    # Ensure Rust is available
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Clean build (optional - remove if you want faster builds)
    # cargo clean

    # Build release binary
    cargo build --release

    success "Build complete!"
}

# Run database migrations
run_migrations() {
    info "Running database migrations..."

    cd "$APP_DIR"

    # Ensure database directory exists
    mkdir -p "$(dirname "$DB_PATH")"

    # Check if migrations directory exists
    if [ -d "migrations" ]; then
        # Run all migration SQL files in order
        for migration in migrations/*.sql; do
            if [ -f "$migration" ]; then
                info "Running migration: $(basename "$migration")"
                sqlite3 "$DB_PATH" < "$migration" || warn "Migration $(basename "$migration") may have already been applied"
            fi
        done
        success "Migrations complete"
    else
        warn "No migrations directory found"
    fi
}

# Start the service
start_service() {
    info "Starting JeebsAI service..."

    systemctl daemon-reload
    systemctl start "$SERVICE_NAME"

    # Wait a moment for service to start
    sleep 2

    if systemctl is-active --quiet "$SERVICE_NAME"; then
        success "Service started successfully!"
    else
        error "Service failed to start. Check logs with: journalctl -u $SERVICE_NAME -n 50"
        exit 1
    fi
}

# Check service status
check_status() {
    info "Checking service status..."

    systemctl status "$SERVICE_NAME" --no-pager -l || true

    info ""
    info "Recent logs:"
    journalctl -u "$SERVICE_NAME" -n 20 --no-pager || true
}

# Health check
health_check() {
    info "Performing health check..."

    sleep 3

    # Try to connect to the service
    if curl -f http://localhost:8080/webui/index.html > /dev/null 2>&1; then
        success "✅ Health check passed! JeebsAI is responding."
    else
        warn "⚠️  Health check inconclusive. Service may still be starting up."
        info "Check manually: curl http://localhost:8080/webui/index.html"
    fi
}

# Main deployment flow
main() {
    echo ""
    info "Starting deployment process..."
    echo ""

    # Step 1: Backup
    backup_database
    echo ""

    # Step 2: Stop service
    stop_service
    echo ""

    # Step 3: Pull code
    pull_code
    echo ""

    # Step 4: Build
    build_app
    echo ""

    # Step 5: Migrations
    run_migrations
    echo ""

    # Step 6: Start service
    start_service
    echo ""

    # Step 7: Check status
    check_status
    echo ""

    # Step 8: Health check
    health_check
    echo ""

    success "=========================================="
    success "🎉 Deployment Complete!"
    success "=========================================="
    echo ""
    info "Service is running with latest code from main branch"
    info "Database backup: $BACKUP_DIR"
    info "Logs: journalctl -u $SERVICE_NAME -f"
    echo ""
}

# Run main deployment
main

exit 0
