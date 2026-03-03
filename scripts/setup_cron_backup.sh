#!/usr/bin/env bash
#
# Script to set up nightly backup cron job for JeebsAI
#
set -euo pipefail

APP_DIR="${APP_DIR:-/root/JeebsAI}"
BACKUP_SCRIPT="$APP_DIR/scripts/backup.sh"
LOG_FILE="/var/log/jeebs_backup.log"

echo "Setting up nightly backup for JeebsAI..."

if [ ! -f "$BACKUP_SCRIPT" ]; then
    echo "❌ Error: Backup script not found at $BACKUP_SCRIPT"
    exit 1
fi

chmod +x "$BACKUP_SCRIPT"

# Job: Run at 3:00 AM every day
JOB="0 3 * * * $BACKUP_SCRIPT >> $LOG_FILE 2>&1"

if crontab -l 2>/dev/null | grep -Fq "$BACKUP_SCRIPT"; then
    echo "⚠️  Backup cron job already exists. No changes made."
else
    (crontab -l 2>/dev/null; echo "$JOB") | crontab -
    echo "✅ Cron job added! Backups will run nightly at 3:00 AM."
    echo "   Logs will be written to: $LOG_FILE"
fi