#!/bin/sh
set -e

# Ensure data directory exists and DB file is present
mkdir -p /data
if [ ! -f /data/jeebs.db ]; then
  echo "Creating empty sqlite DB: /data/jeebs.db"
  touch /data/jeebs.db
fi

# Ensure VERSION exists
if [ ! -f /app/VERSION ]; then
  if [ -f /usr/local/bin/jeebs ]; then
    echo "v0.0.0" > /app/VERSION || true
  fi
fi

exec "$@"
