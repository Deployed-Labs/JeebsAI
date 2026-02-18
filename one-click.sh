#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

REPO_URL=${REPO_URL:-"https://github.com/Deployed-Labs/JeebsAI.git"}
APP_DIR=${APP_DIR:-"/opt/jeebs"}
APP_USER=${APP_USER:-"jeebs"}
APP_PORT=${APP_PORT:-"8080"}
DOMAIN=${DOMAIN:-""}
EMAIL=${EMAIL:-""}
DB_PATH=${DB_PATH:-"/var/lib/jeebs/jeebs.db"}
FORCE=${FORCE:-""}
UFW_ALLOW=${UFW_ALLOW:-"1"}

if [[ $EUID -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

if [[ -r /etc/os-release ]]; then
  . /etc/os-release
  if [[ "${ID:-}" != "ubuntu" && "${ID:-}" != "debian" ]]; then
    echo "This script supports Ubuntu/Debian only. Detected: ${ID:-unknown}."
    exit 1
  fi
fi

if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemd is required. systemctl not found."
  exit 1
fi

confirm_overwrite() {
  local target_path=$1
  local label=$2

  if [[ -e "$target_path" && -z "$FORCE" ]]; then
    if [[ -t 0 ]]; then
      read -r -p "$label exists at $target_path. Overwrite? [y/N] " reply
    else
      reply=""
    fi

    if [[ ! "$reply" =~ ^[Yy]$ ]]; then
      echo "Aborting to avoid overwriting $label."
      exit 1
    fi
  fi
}

open_ufw_port() {
  local port=$1

  if [[ "$UFW_ALLOW" != "1" ]]; then
    return 0
  fi

  if command -v ufw >/dev/null 2>&1; then
    if ufw status | grep -q "Status: active"; then
      ufw allow "$port" >/dev/null
    fi
  fi
}

apt-get update
apt-get install -y \
  build-essential clang pkg-config libssl-dev sqlite3 git curl ca-certificates \
  nettle-dev libgpg-error-dev libgcrypt-dev

# Verify native build dependencies (clear, actionable output)
verify_native_deps() {
  local missing=()

  if ! command -v pkg-config >/dev/null 2>&1; then
    missing+=("pkg-config")
  fi
  if ! command -v clang >/dev/null 2>&1 && ! command -v gcc >/dev/null 2>&1; then
    missing+=("clang/gcc")
  fi
  for lib in nettle gpg-error gcrypt; do
    if ! pkg-config --exists "$lib" 2>/dev/null; then
      missing+=("pkg-config:$lib")
    fi
  done

  if [ ${#missing[@]} -ne 0 ]; then
    echo "\nERROR: missing native packages required to build JeebsAI: ${missing[*]}\n"
    echo "Fix (Debian/Ubuntu):"
    echo "  sudo apt update && sudo apt install -y build-essential clang pkg-config nettle-dev libgpg-error-dev libgcrypt-dev"
    echo "After installing the packages re-run this script or run: ./install.sh"
    exit 1
  fi
  echo "Native build dependencies verified. Continuing..."
}

verify_native_deps

if [[ -n "$DOMAIN" ]]; then
  apt-get install -y nginx certbot python3-certbot-nginx
fi

if [[ -n "$DOMAIN" && -z "$EMAIL" ]]; then
  echo "DOMAIN is set but EMAIL is empty. Skipping SSL setup."
fi

if [[ -z "$DOMAIN" && -n "$EMAIL" ]]; then
  echo "EMAIL is set but DOMAIN is empty. Skipping SSL setup."
fi

open_ufw_port "$APP_PORT"

if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source /root/.cargo/env
fi

if ! id -u "$APP_USER" >/dev/null 2>&1; then
  useradd -r -m -d "$APP_DIR" -s /usr/sbin/nologin "$APP_USER"
fi

mkdir -p "$APP_DIR"
chown -R "$APP_USER":"$APP_USER" "$APP_DIR"

if [[ -d "$APP_DIR/.git" ]]; then
  sudo -u "$APP_USER" git -C "$APP_DIR" fetch --all
  sudo -u "$APP_USER" git -C "$APP_DIR" reset --hard origin/main
else
  sudo -u "$APP_USER" git clone "$REPO_URL" "$APP_DIR"
fi

sudo -u "$APP_USER" bash -lc "cd '$APP_DIR' && cargo build --release"

mkdir -p /etc/jeebs /var/lib/jeebs
chown -R "$APP_USER":"$APP_USER" /var/lib/jeebs

confirm_overwrite /etc/jeebs/config.env "config file"
cat >/etc/jeebs/config.env <<EOF
PORT=$APP_PORT
DATABASE_URL=sqlite:$DB_PATH
RUST_LOG=info
EOF

confirm_overwrite /etc/systemd/system/jeebs.service "systemd unit"
cat >/etc/systemd/system/jeebs.service <<EOF
[Unit]
Description=JeebsAI Server
After=network.target

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/target/release/jeebs
EnvironmentFile=-/etc/jeebs/config.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable jeebs
systemctl restart jeebs

if [[ -n "$DOMAIN" && -n "$EMAIL" ]]; then
  open_ufw_port 80
  open_ufw_port 443
  confirm_overwrite /etc/nginx/sites-available/jeebs "nginx site"
  cat >/etc/nginx/sites-available/jeebs <<EOF
server {
  listen 80;
  server_name $DOMAIN;

  location / {
    proxy_pass http://127.0.0.1:$APP_PORT;
    proxy_set_header Host \$host;
    proxy_set_header X-Real-IP \$remote_addr;
    proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto \$scheme;
  }
}
EOF

  ln -sf /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/jeebs
  nginx -t
  systemctl reload nginx
  certbot --nginx -d "$DOMAIN" -m "$EMAIL" --agree-tos --non-interactive
fi

echo "JeebsAI is up on port $APP_PORT"
systemctl --no-pager status jeebs
