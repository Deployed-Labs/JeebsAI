#!/usr/bin/env bash
set -euo pipefail
# master_restore.sh - restore a master backup tar.gz created by master_backup.sh

BACKUP_FILE=${1:-}
if [ -z "$BACKUP_FILE" ]; then
  echo "Usage: $0 /path/to/jeebs_master_backup_YYYYMMDDTHHMMSSZ.tar.gz"
  exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Backup file not found: $BACKUP_FILE"
  exit 1
fi

read -p "This will restore files and may overwrite existing data. Continue? (yes/no) ": yn
if [ "$yn" != "yes" ]; then
  echo "Aborting."
  exit 0
fi

TS=$(date -u +%Y%m%dT%H%M%SZ)
BACKUP_DIR="restore_backup_before_${TS}"
mkdir -p "$BACKUP_DIR"

echo "Backing up current project files to $BACKUP_DIR"
cp -a migrations "$BACKUP_DIR/" 2>/dev/null || true
cp -a webui "$BACKUP_DIR/" 2>/dev/null || true
cp -a src "$BACKUP_DIR/" 2>/dev/null || true
cp -a jeebs.db "$BACKUP_DIR/" 2>/dev/null || true

echo "Extracting $BACKUP_FILE"
TMPDIR=$(mktemp -d)
tar -C "$TMPDIR" -xzf "$BACKUP_FILE"

if [ -d "$TMPDIR/jeebs_backup" ]; then
  echo "Copying files from backup into project root"
  cp -a "$TMPDIR/jeebs_backup/." ./
  echo "Extraction complete."
else
  echo "Unexpected archive structure. Inspecting contents:"; ls -la "$TMPDIR"
fi

rm -rf "$TMPDIR"
echo "Restore finished. Consider running migrations and restarting the service."
