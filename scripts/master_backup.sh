#!/usr/bin/env bash
set -euo pipefail
# master_backup.sh - create a roll-up backup of important server files

OUT_DIR=${1:-/tmp}
DEST_SCP=${DEST_SCP:-}
TS=$(date -u +%Y%m%dT%H%M%SZ)
NAME="jeebs_master_backup_${TS}.tar.gz"
OUT_PATH="$OUT_DIR/$NAME"

echo "Creating master backup -> $OUT_PATH"
TMPDIR=$(mktemp -d)

echo "Copying important files..."
mkdir -p "$TMPDIR/jeebs_backup"
cp -a migrations "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a webui "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a src "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a scripts "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a Cargo.toml Cargo.lock "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a docker-compose*.yml .env* "$TMPDIR/jeebs_backup/" 2>/dev/null || true
cp -a .github "$TMPDIR/jeebs_backup/" 2>/dev/null || true

# Database and store
if [ -f jeebs.db ]; then
  echo "Including jeebs.db"
  cp jeebs.db "$TMPDIR/jeebs_backup/"
fi

# Include system files if present
if [ -f /etc/systemd/system/jeebs.service ]; then
  mkdir -p "$TMPDIR/jeebs_backup/systemd"
  cp /etc/systemd/system/jeebs.service "$TMPDIR/jeebs_backup/systemd/"
fi

echo "Creating archive..."
tar -C "$TMPDIR" -czf "$OUT_PATH" jeebs_backup

echo "Calculating checksum..."
sha256sum "$OUT_PATH" > "$OUT_PATH".sha256

if [ -n "$DEST_SCP" ]; then
  echo "Copying to remote destination $DEST_SCP"
  scp "$OUT_PATH" "$DEST_SCP"
  scp "$OUT_PATH".sha256 "$DEST_SCP"
  echo "Uploaded to $DEST_SCP"
else
  echo "No DEST_SCP provided. To copy to your desktop, set DEST_SCP=user@host:/path and re-run."
fi

echo "Backup ready: $OUT_PATH"
echo "Backup checksum: $(cat "$OUT_PATH".sha256)"
rm -rf "$TMPDIR"
