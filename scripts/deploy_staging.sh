#!/usr/bin/env bash
set -euo pipefail

# Deploy JeebsAI to the staging systemd service (root-friendly).
# Usage:
#   ./scripts/deploy_staging.sh            # deploy using current repo state
#   RESET_MAIN=1 ./scripts/deploy_staging.sh  # hard-reset to origin/main before build

REPO_DIR=${REPO_DIR:-"/root/JeebsAI"}
SERVICE_NAME=${SERVICE_NAME:-"jeebs-staging"}
APP_DIR=${APP_DIR:-"/opt/jeebs-staging"}
ENV_FILE=${ENV_FILE:-"/etc/jeebs-staging.env"}
DB_PATH=${DB_PATH:-"/var/lib/jeebs-staging/jeebs.db"}
RESET_MAIN=${RESET_MAIN:-""}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

cd "$REPO_DIR"

git fetch origin >/dev/null 2>&1 || true
if [[ -n "$RESET_MAIN" ]]; then
  git checkout main
  git reset --hard origin/main
  git clean -fd
fi

# Ensure migrations are clean (remove accidental conflict markers).
sed -i '/^<<<<<<< /d;/^=======/d;/^>>>>>>> /d' "$REPO_DIR/migrations/20240101000000_initial_setup.sql"

# Ensure staging env file exists and points to an absolute SQLite path.
mkdir -p "$(dirname "$ENV_FILE")"
mkdir -p "$(dirname "$DB_PATH")"
if [[ ! -f "$ENV_FILE" ]]; then
  cat >"$ENV_FILE" <<EOF
PORT=8081
DATABASE_URL=sqlite:$DB_PATH
RUST_LOG=info
EOF
fi

# Normalize DB path and port in case env was created earlier.
sed -i "s|^DATABASE_URL=.*|DATABASE_URL=sqlite:$DB_PATH|" "$ENV_FILE"
if ! grep -q '^PORT=' "$ENV_FILE"; then
  echo "PORT=8081" >> "$ENV_FILE"
fi

# Build release binary.
cargo build --release

# Deploy binary.
mkdir -p "$APP_DIR/target/release"
systemctl stop "$SERVICE_NAME"
cp "$REPO_DIR/target/release/jeebs" "$APP_DIR/target/release/jeebs"
chmod 755 "$APP_DIR/target/release/jeebs"

systemctl daemon-reload
systemctl start "$SERVICE_NAME"

systemctl status "$SERVICE_NAME" --no-pager
