#!/usr/bin/env bash
#
# Script to check the logs of the nightly backup cron job
#

LOG_FILE="/var/log/jeebs_backup.log"

echo "🔍 Checking JeebsAI Backup Logs ($LOG_FILE)..."
echo "==================================================="

if [ -f "$LOG_FILE" ]; then
    # Show the last 50 lines
    tail -n 50 "$LOG_FILE"
else
    echo "❌ Log file not found at $LOG_FILE"
    echo "   (The cron job may not have run yet, or permissions may be incorrect)"
fi
echo "==================================================="
echo ""
echo "To watch logs in real-time during a run:"
echo "tail -f $LOG_FILE"