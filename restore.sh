#!/bin/bash

# JeebsAI Database Restore Script
# Usage: ./restore.sh backups/jeebs_backup_YYYY-MM-DD_HH-MM-SS.db.gz

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <path_to_backup_file.gz>"
    exit 1
fi

BACKUP_FILE="$1"
# Automatically determine the project directory
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
DB_FILE="$PROJECT_DIR/jeebs.db"
SERVICE_NAME="jeebs"

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file '$BACKUP_FILE' not found."
    exit 1
fi

echo "--- Restoring Database ---"
echo "Target: $DB_FILE"
echo "Source: $BACKUP_FILE"

# 1. Stop the service to release the database lock
echo "[1/5] Stopping $SERVICE_NAME service..."
sudo systemctl stop $SERVICE_NAME

# 2. Create a safety backup of the current live database
if [ -f "$DB_FILE" ]; then
    SAFETY_BACKUP="${DB_FILE}.pre_restore_$(date +%s)"
    echo "[2/5] Creating safety backup of current database at $SAFETY_BACKUP..."
    cp "$DB_FILE" "$SAFETY_BACKUP"
fi

# 3. Restore the database
echo "[3/5] Decompressing and restoring database..."
gunzip -c "$BACKUP_FILE" > "$DB_FILE"

# 4. Fix permissions
# Ensure the database is owned by the user who owns the project directory
OWNER=$(stat -c '%U' "$PROJECT_DIR")
GROUP=$(stat -c '%G' "$PROJECT_DIR")
echo "[4/5] Setting ownership to $OWNER:$GROUP..."
sudo chown "$OWNER:$GROUP" "$DB_FILE"

# 5. Start the service
echo "[5/5] Starting $SERVICE_NAME service..."
sudo systemctl start $SERVICE_NAME

echo "Restore complete!"