#!/bin/bash
set -e

# Configuration
APP_DIR="${APP_DIR:-/root/JeebsAI}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/jeebs}"
DB_PATH="${DB_PATH:-/var/lib/jeebs/jeebs.db}"
SERVICE_NAME="jeebs"

# Check root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

echo "🔍 Available backups in $BACKUP_DIR:"
echo "----------------------------------------"
ls -lh "$BACKUP_DIR"/*.gz 2>/dev/null || { echo "No backups found."; exit 1; }
echo "----------------------------------------"
echo ""

read -p "Enter the full path of the backup file to restore: " BACKUP_FILE

if [ ! -f "$BACKUP_FILE" ]; then
    echo "❌ File not found!"
    exit 1
fi

echo ""
echo "⚠️  WARNING: This will OVERWRITE the current database at $DB_PATH"
read -p "Are you sure you want to proceed? (y/N) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 1
fi

echo "1. Stopping service..."
systemctl stop "$SERVICE_NAME"

echo "2. Restoring database..."
# Decompress directly to the database path
gunzip -c "$BACKUP_FILE" > "$DB_PATH"

# Ensure permissions are correct (root read/write)
chmod 644 "$DB_PATH"

echo "3. Starting service..."
systemctl start "$SERVICE_NAME"

echo "✅ Restore complete!"
systemctl status "$SERVICE_NAME" --no-pager