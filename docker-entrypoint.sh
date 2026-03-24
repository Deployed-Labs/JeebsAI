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

# If DATABASE_URL is set, optionally wait for DB (DB_HOST/DB_PORT can be provided)
if [ -n "$DATABASE_URL" ]; then
  DB_HOST=${DB_HOST:-db}
  DB_PORT=${DB_PORT:-5432}
  echo "Waiting for database $DB_HOST:$DB_PORT..."
  while ! nc -z "$DB_HOST" "$DB_PORT"; do
    echo "Waiting for $DB_HOST:$DB_PORT..."
    sleep 1
  done
  echo "Database is available."
fi

exec "$@"
