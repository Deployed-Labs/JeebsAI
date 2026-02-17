#!/bin/bash

# Nightly Backup Script for JeebsAI SQLite Database

set -e

# --- Configuration ---
# The directory where your JeebsAI project is located.
# This line automatically determines the absolute path of the directory where the script is located.
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# The directory where backups will be stored.
BACKUP_DIR="$PROJECT_DIR/backups"

# The name of the database file.
DB_FILE="jeebs.db"

# --- Script ---

echo "Starting JeebsAI database backup..."

mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
BACKUP_FILE="$BACKUP_DIR/jeebs_backup_$TIMESTAMP.db"

echo "Backing up $PROJECT_DIR/$DB_FILE to $BACKUP_FILE"
sqlite3 "$PROJECT_DIR/$DB_FILE" ".backup '$BACKUP_FILE'"

echo "Compressing backup file..."
gzip "$BACKUP_FILE"

echo "Deleting backups older than 7 days..."
find "$BACKUP_DIR" -type f -name "*.gz" -mtime +7 -delete

echo "Backup complete: $BACKUP_FILE.gz"