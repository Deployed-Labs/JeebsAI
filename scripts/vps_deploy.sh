#!/usr/bin/env bash
set -euo pipefail

# VPS deploy script: pull a branch, build (if present), and restart systemd service.
# Usage on VPS: sudo ./scripts/vps_deploy.sh [branch] [deploy_dir]

# Can accept: branch or tag. If TAG_RELEASE=true we'll treat first arg as a release tag.
ARG=${1:-main}
DEPLOY_DIR=${2:-/root/JeebsAI}
SERVICE_NAME=${3:-jeebs}
GITHUB_REPO=${4:-Deployed-Labs/JeebsAI}
USE_RELEASE=${USE_RELEASE:-1}

# Treat first arg as branch or tag
BRANCH="$ARG"

echo "Deploying branch/tag '$BRANCH' into $DEPLOY_DIR and ensuring service '$SERVICE_NAME' is running"

mkdir -p "$(dirname "$DEPLOY_DIR")"

if [ ! -d "$DEPLOY_DIR" ] || [ -z "$(ls -A "$DEPLOY_DIR")" ]; then
  echo "Cloning repository into $DEPLOY_DIR"
  git clone --depth 1 "https://github.com/$GITHUB_REPO.git" "$DEPLOY_DIR"
fi

cd "$DEPLOY_DIR"

# Ensure repository is present
if [ ! -d .git ]; then
  echo "No git repository found in $DEPLOY_DIR" >&2
  exit 3
fi

git fetch --all --prune

# Checkout or create a local branch/tracking branch for BRANCH
if git rev-parse --verify --quiet "origin/$BRANCH" >/dev/null 2>&1; then
  git checkout -B "$BRANCH" "origin/$BRANCH"
else
  # If tag exists locally or remotely, check it out detached
  if git rev-parse --verify --quiet "$BRANCH" >/dev/null 2>&1 || git ls-remote --tags origin | grep -q "refs/tags/$BRANCH$"; then
    git checkout "$BRANCH" || git checkout --detach "origin/$BRANCH" || true
  else
    echo "Branch or tag '$BRANCH' not found on origin; defaulting to origin/main"
    git checkout -B main "origin/main"
    BRANCH=main
  fi
fi

# Reset to exact origin state for the branch if possible
if git rev-parse --verify --quiet "origin/$BRANCH" >/dev/null 2>&1; then
  git reset --hard "origin/$BRANCH"
fi

# Ensure environment file exists for systemd service
if [ ! -f /etc/jeebs.env ]; then
  echo "Creating /etc/jeebs.env with random SESSION_KEY_B64"
  SESSION_KEY_B64=$(head -c 24 /dev/urandom | base64 | tr -d '\n')
  echo "SESSION_KEY_B64=$SESSION_KEY_B64" > /etc/jeebs.env
  chmod 600 /etc/jeebs.env || true
fi

# Build from source on the VPS
if [ -f Cargo.toml ]; then
  echo "Building release on VPS (cargo build --release)"
  cargo build --release
fi

# Verify binary
BINARY_PATH="$DEPLOY_DIR/target/release/jeebs"
if [ ! -f "$BINARY_PATH" ]; then
  echo "Build failed or binary not found at $BINARY_PATH" >&2
  exit 4
fi

# Install or update systemd unit
SERVICE_PATH="/etc/systemd/system/$SERVICE_NAME.service"
if [ -f deploy/jeebs.service ]; then
  echo "Installing service unit from deploy/jeebs.service -> $SERVICE_PATH"
  cp deploy/jeebs.service "$SERVICE_PATH"
else
  echo "Writing minimal systemd unit to $SERVICE_PATH"
  cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=JeebsAI service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$DEPLOY_DIR
EnvironmentFile=/etc/jeebs.env
ExecStart=$BINARY_PATH
Restart=always
RestartSec=5
TimeoutStopSec=20

[Install]
WantedBy=multi-user.target
EOF
fi

chmod 644 "$SERVICE_PATH" || true
systemctl daemon-reload || true
systemctl enable "$SERVICE_NAME" || true

echo "Restarting service: $SERVICE_NAME"
systemctl restart "$SERVICE_NAME"

sleep 1
systemctl status "$SERVICE_NAME" --no-pager -l || true

if [ -f VERSION ]; then
  echo "Current VERSION file content:"
  cat VERSION || true
fi

echo "Deploy finished"
