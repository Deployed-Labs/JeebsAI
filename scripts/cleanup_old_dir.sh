#!/usr/bin/env bash
#
# This script checks for and removes the old /opt/jeebs directory.
#
set -euo pipefail

OLD_APP_DIR="/opt/jeebs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   error "This script must be run as root (use sudo) to remove system directories."
   exit 1
fi

info "Checking for old installation directory: $OLD_APP_DIR..."

if [ -d "$OLD_APP_DIR" ]; then
    info "Old directory found. Removing $OLD_APP_DIR..."
    rm -rf "$OLD_APP_DIR"
    success "Successfully removed old directory: $OLD_APP_DIR"
else
    success "Old directory not found. No action needed."
fi

echo ""
info "Cleanup check complete."