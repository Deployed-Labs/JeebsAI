#!/usr/bin/env bash
# Rename a username across common tables in a local sqlite DB (jeebs.db by default)
# Usage: ./scripts/rename_username.sh <old_username> <new_username> [db_path]
set -euo pipefail
OLD=${1:-}
NEW=${2:-}
DB=${3:-jeebs.db}
if [ -z "$OLD" ] || [ -z "$NEW" ]; then
  echo "Usage: $0 <old_username> <new_username> [db_path]" >&2
  exit 2
fi
if [ ! -f "$DB" ]; then
  echo "DB file $DB not found." >&2
  exit 2
fi
echo "Updating user_sessions..."
sqlite3 "$DB" "UPDATE user_sessions SET username = '$NEW' WHERE username = '$OLD';"
echo "Updating system_logs messages containing username (best effort, free-text replace)..."
sqlite3 "$DB" "UPDATE system_logs SET message = replace(message, '$OLD', '$NEW') WHERE message LIKE '%$OLD%';"
# Note: jeebs_store may contain encoded/encrypted blobs; manual migration may be required for values stored there.
echo "Done. Please inspect $DB for remaining references (jeebs_store blobs may need manual updates)."
