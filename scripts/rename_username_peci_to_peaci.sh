#!/usr/bin/env bash
set -euo pipefail
DB=${1:-jeebs.db}
OLD=peci
NEW=peaci

if [ ! -f "$DB" ]; then
  echo "DB not found: $DB"
  exit 1
fi

echo "Backing up $DB -> ${DB}.bak"
cp "$DB" "${DB}.bak"

sqlite3 "$DB" <<SQL
-- 1) Update user_sessions primary key
UPDATE user_sessions SET username = '$NEW' WHERE username = '$OLD';

-- 2) Copy user record key if exists, then delete old
INSERT OR REPLACE INTO jeebs_store (key, value)
  SELECT 'user:$NEW', value FROM jeebs_store WHERE key = 'user:$OLD';
DELETE FROM jeebs_store WHERE key = 'user:$OLD';

-- 3) Rename any jeebs_store keys containing :peci -> :peaci
UPDATE jeebs_store SET key = replace(key, ':$OLD', ':$NEW') WHERE key LIKE '%:$OLD%';

-- 4) Replace textual occurrences in jeebs_store values where safe
-- Cast to text for JSON blobs and attempt replacement for simple cases
UPDATE jeebs_store SET value = replace(CAST(value AS TEXT), '"$OLD"', '"$NEW"')
  WHERE CAST(value AS TEXT) LIKE '%"$OLD"%';

UPDATE system_logs SET message = replace(message, '$OLD', '$NEW') WHERE message LIKE '%$OLD%';


SQL

HAS_RT=$(sqlite3 "$DB" "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='reasoning_traces';" 2>/dev/null || echo 0)

HAS_RT=$(sqlite3 "$DB" "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='reasoning_traces';" 2>/dev/null || echo 0)
if [ "$HAS_RT" -eq "1" ]; then
  sqlite3 "$DB" "UPDATE reasoning_traces SET username = '$NEW' WHERE username = '$OLD';"
fi

sqlite3 "$DB" <<SQL
-- 7) Update any other tables with username column if present
UPDATE user_sessions SET username = '$NEW' WHERE username = '$OLD';

PRAGMA wal_checkpoint(TRUNCATE);
SQL
echo "Done. Please inspect ${DB}.bak and ${DB} for remaining references (blobs may need manual fixes)."
