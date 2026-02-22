# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI, persistent storage, and **advanced learning capabilities**.
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ New: Learning & Knowledge System

Jeebs now features:
- 🧠 **Advanced Knowledge Retrieval** - Search across brain nodes, knowledge triples, contexts, and FAQ
- 📚 **Language Learning** - Automatically learns vocabulary and patterns from every conversation
- 💡 **Proactive Proposals** - Suggests learning topics, features, and experiments
- 📊 **Progress Tracking** - Monitor vocabulary growth and knowledge accumulation
- 🎓 **User Teaching** - Store facts, context, and custom responses

👉 **[Quick Start Guide](QUICK_START.md)** | **[Full Learning System Docs](LEARNING_SYSTEM.md)**

## 🚀 Quick VPS Install

**Deploy to VPS in one command:**
```bash
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

👉 **[Full VPS Installation Guide](VPS_INSTALL.md)**

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Local Development](#local-development)
  - [VPS Production Deployment](#vps-production-deployment)
- [Configuration](#configuration)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Development](#development)
- [Contributing](#contributing)
- [CI/CD](#cicd)

## Prerequisites

### Local Development
- Rust 1.70+ and Cargo
- SQLite 3

### VPS Production Deployment
- Ubuntu/Debian-based VPS (recommended)
- Rust 1.70+ and Cargo
- SQLite 3
- Nginx (for reverse proxy)
- Certbot (for SSL/TLS certificates)
- systemd (for process management)

### System Dependencies
Install required system packages on Ubuntu/Debian:
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev sqlite3 nginx certbot python3-certbot-nginx
```

## Installation

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Deployed-Labs/JeebsAI.git
   cd JeebsAI
   ```

2. **Build and run:**
   ```bash
   cargo run
   ```

3. **Access the web UI:**
   - Development server: [http://localhost:8080](http://localhost:8080)

### VPS Production Deployment

#### Quick Installation

Use the provided installation script to automatically set up JeebsAI as a systemd service:

```bash
# Clone the repository
git clone https://github.com/Deployed-Labs/JeebsAI.git
cd JeebsAI

# Run the installation script (requires sudo)
chmod +x install.sh
./install.sh
```

The `install.sh` script will:
- Build the release binary
- Create a systemd service file
- Set up environment configuration at `/etc/jeebs.env`
- Enable and start the service automatically

#### One-Click Deploy (all-in-one)

Paste this script on a fresh Ubuntu/Debian VPS to install Rust, build, and run JeebsAI as a systemd service in one go:

```bash
#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

REPO_URL=${REPO_URL:-"https://github.com/Deployed-Labs/JeebsAI.git"}
APP_DIR=${APP_DIR:-"/opt/jeebs"}
APP_USER=${APP_USER:-"root"}
APP_PORT=${APP_PORT:-"8080"}
DOMAIN=${DOMAIN:-""}
EMAIL=${EMAIL:-""}
DB_PATH=${DB_PATH:-"/var/lib/jeebs/jeebs.db"}
FORCE=${FORCE:-""}

if [[ $EUID -ne 0 ]]; then
   exec sudo -E "$0" "$@"
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

apt-get update
apt-get install -y \
   build-essential pkg-config libssl-dev sqlite3 git curl ca-certificates

if [[ -n "$DOMAIN" ]]; then
   apt-get install -y nginx certbot python3-certbot-nginx
fi

if [[ -n "$DOMAIN" && -z "$EMAIL" ]]; then
   echo "DOMAIN is set but EMAIL is empty. Skipping SSL setup."
fi

if [[ -z "$DOMAIN" && -n "$EMAIL" ]]; then
   echo "EMAIL is set but DOMAIN is empty. Skipping SSL setup."
fi

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

confirm_overwrite /etc/jeebs.env "config file"
cat >/etc/jeebs.env <<EOF
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
EnvironmentFile=-/etc/jeebs.env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable jeebs
systemctl restart jeebs

if [[ -n "$DOMAIN" && -n "$EMAIL" ]]; then
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
