#!/bin/bash

# Nightly Backup Script for JeebsAI SQLite Database

set -e

# --- Configuration ---
# These paths are consistent with deploy_to_vps.sh and vps_fresh_install.sh
APP_DIR="${APP_DIR:-/root/JeebsAI}"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/jeebs}"
DB_PATH="${DB_PATH:-/var/lib/jeebs/jeebs.db}"
SERVICE_NAME="jeebs" # Not directly used here, but for consistency

# --- Script ---

echo "Starting JeebsAI database backup..."

mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILE="$BACKUP_DIR/jeebs_backup_$TIMESTAMP.db"

echo "Backing up $DB_PATH to $BACKUP_FILE"
sqlite3 "$DB_PATH" ".backup '$BACKUP_FILE'"

echo "Compressing backup file..."
gzip "$BACKUP_FILE"

echo "Cleaning up old backups (keeping last 3)..."
cd "$BACKUP_DIR"
ls -t jeebs_backup_*.db.gz | tail -n +4 | xargs -r rm

echo "Backup complete: $BACKUP_FILE.gz"