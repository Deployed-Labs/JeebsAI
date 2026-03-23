#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
  echo "Run as root: sudo $0"
  exit 1
fi

DB=/var/lib/jeebs/jeebs.db
MIG_ID=20240101000001
MIG_DESC="add_created_at_to_brain_nodes"
BACKUP=/var/lib/jeebs/jeebs.db.bak.$(date -u +%Y%m%d%H%M%S)

echo "Backing up DB to $BACKUP"
cp "$DB" "$BACKUP"

echo "Checking _sqlx_migrations table exists"
if ! sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations';" | grep -q '_sqlx_migrations'; then
  echo "_sqlx_migrations table not found in $DB; aborting"
  exit 1
fi

echo "Existing migration versions (last 10):"
sqlite3 "$DB" "SELECT rowid,version,description,installed_on,success FROM _sqlx_migrations ORDER BY rowid DESC LIMIT 10;"

exists=$(sqlite3 "$DB" "SELECT count(1) FROM _sqlx_migrations WHERE version='$MIG_ID';")
if [ "$exists" -ge 1 ]; then
  echo "Migration $MIG_ID already recorded; nothing to do."
else
  echo "Inserting migration record $MIG_ID"
  sqlite3 "$DB" "INSERT INTO _sqlx_migrations (version,description,installed_on,success,checksum,execution_time) VALUES ('$MIG_ID','$MIG_DESC', datetime('now'), 1, NULL, 0);"
  echo "Inserted. New migrations (last 5):"
  sqlite3 "$DB" "SELECT rowid,version,description,installed_on,success FROM _sqlx_migrations ORDER BY rowid DESC LIMIT 5;"
fi

echo "Ensure /etc/jeebs.env has SESSION_KEY_B64"
if [ ! -f /etc/jeebs.env ] || ! grep -q '^SESSION_KEY_B64=' /etc/jeebs.env; then
  echo "Creating /etc/jeebs.env with SESSION_KEY_B64"
  KEY=$(openssl rand -base64 48)
  printf 'SESSION_KEY_B64=%s\n' "$KEY" > /etc/jeebs.env
  chown root:root /etc/jeebs.env
  chmod 600 /etc/jeebs.env
else
  echo "/etc/jeebs.env already contains SESSION_KEY_B64"
fi

echo "Reload systemd and restart jeebs"
systemctl daemon-reload
systemctl restart jeebs

sleep 1
systemctl status jeebs --no-pager -l | sed -n '1,12p'
sleep 1

echo "Testing HTTP endpoint..."
if curl -sS -I http://127.0.0.1:8080/webui/brain_nodes.html | grep -q '200 OK'; then
  echo "OK: webui endpoint returned 200"
else
  echo "Warning: webui endpoint did not return 200; showing last 50 journal lines:"
  journalctl -u jeebs -n 50 --no-pager
  exit 2
fi

echo "Done."
