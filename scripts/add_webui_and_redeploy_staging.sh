#!/usr/bin/env bash
set -euo pipefail

# One-shot staging fix:
# - Adds /webui static route to src/main.rs (idempotent).
# - Builds release binary.
# - Syncs webui assets into the staging working directory.
# - Restarts jeebs-staging and probes /webui/.

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
PORT=${PORT:-"8080"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

FILE="$REPO_DIR/src/main.rs"
if [[ ! -f "$FILE" ]]; then
  echo "ERROR: file not found: $FILE" >&2
  exit 1
fi

if ! grep -q 'use actix_files::Files;' "$FILE"; then
  sed -i '/use actix_web::cookie::Key;/a use actix_files::Files;' "$FILE"
fi

if ! grep -q 'Files::new("/webui"' "$FILE"; then
  sed -i '/\.service(auth::login_pgp)/a \            .service(Files::new("/webui", "./webui").index_file("index.html"))' "$FILE"
fi

cd "$REPO_DIR"
cargo build --release

mkdir -p "$APP_DIR/target/release"
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$APP_DIR/target/release/jeebs"
chmod 755 "$APP_DIR/target/release/jeebs"

rm -rf "$APP_DIR/webui"
cp -R "$REPO_DIR/webui" "$APP_DIR/webui"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
curl -I "http://127.0.0.1:${PORT}/webui/"
