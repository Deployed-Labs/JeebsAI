
# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI and persistent storage.
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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
- Set up environment configuration at `/etc/jeebs/config.env`
- Enable and start the service automatically

#### One-Click Deploy (all-in-one)

Paste this script on a fresh Ubuntu/Debian VPS to install Rust, build, and run JeebsAI as a systemd service in one go:

```bash
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
```

Usage (single line, optional overrides):

```bash
chmod +x one-click.sh
APP_DIR=/opt/jeebs APP_USER=jeebs APP_PORT=8080 DOMAIN=example.com EMAIL=admin@example.com ./one-click.sh

# For non-interactive overwrite of existing config/systemd/nginx files:
FORCE=1 ./one-click.sh

# Skip UFW rule changes (leave firewall untouched):
UFW_ALLOW=0 ./one-click.sh
```

#### Manual Installation Steps

If you prefer manual installation, follow these steps:

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Set up the systemd service:**
   ```bash
   # Copy the service template
   sudo cp jeebs.service /etc/systemd/system/jeebs.service
   
   # Edit the service file with your actual paths and username
   sudo nano /etc/systemd/system/jeebs.service
   
   # Enable and start the service
   sudo systemctl daemon-reload
   sudo systemctl enable jeebs
   sudo systemctl start jeebs
   ```

3. **Configure Nginx reverse proxy:**
   ```bash
   # Copy the nginx configuration
   sudo cp jeebs.nginx /etc/nginx/sites-available/jeebs
   
   # Edit the configuration with your domain
   sudo nano /etc/nginx/sites-available/jeebs
   
   # Enable the site
   sudo ln -s /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/
   
   # Test configuration and reload
   sudo nginx -t
   sudo systemctl reload nginx
   ```

4. **Set up SSL with Certbot:**
   ```bash
   sudo certbot --nginx -d your_domain.com
   ```

5. **Initialize the database:**
   ```bash
   sqlite3 jeebs.db < 20240101000000_initial_setup.sql
   ```

## Configuration

### Environment Variables

Edit `/etc/jeebs/config.env` to configure the application:

```bash
# Server configuration
PORT=8080

# Database configuration
DATABASE_URL=sqlite:jeebs.db

# Logging (optional)
# RUST_LOG=info,actix_web=info
```

### Service Management

Control the JeebsAI service:

```bash
# Check service status
sudo systemctl status jeebs

# Start the service
sudo systemctl start jeebs

# Stop the service
sudo systemctl stop jeebs

# Restart the service
sudo systemctl restart jeebs

# View logs
sudo journalctl -u jeebs -f
```

### Database Backup

A backup script is provided for automated database backups:

```bash
# Make the backup script executable
chmod +x backup.sh

# Run manual backup
./backup.sh

# Set up automated nightly backups with cron
sudo crontab -e
# Add this line to run backup at 2 AM daily:
# 0 2 * * * /path/to/JeebsAI/backup.sh
```

Backups are stored in the `backups/` directory and automatically compressed. Backups older than 7 days are automatically deleted.

## Usage

### Starting the Application

**Development mode:**
```bash
cargo run
```

**Production mode (manual start):**
```bash
./start.sh
```

**Production mode (systemd service):**
```bash
sudo systemctl start jeebs
```

### Accessing the Web UI

- **Local development:** http://localhost:8080
- **Production (with Nginx):** https://your_domain.com

## Project Structure

- `src/`
	- `main.rs` — Application entry point, web server, and CLI
	- `admin/` — Admin features (user management, now in `admin/user/`)
	- `brain/` — Knowledge graph and training logic
	- `auth/` — Authentication, registration, and password reset
- `webui/` — Web user interface (HTML, JS, CSS)
- `install.sh` — Automated installation script for VPS deployment
- `start.sh` — Manual startup script
- `backup.sh` — Database backup script
- `jeebs.service` — systemd service template
- `jeebs.nginx` — Nginx reverse proxy configuration template

## Modularity

All major features are separated into modules and submodules for maintainability:

- `admin::user` — Admin user management endpoints and types
- `brain` — Knowledge graph, training, and storage
- `auth` — Registration, login, password reset

## Development

- All business logic is modularized for easy extension
- See each module for details and add new features in their own modules/submodules
- Run tests: `cargo test`
- Run with logging: `RUST_LOG=debug cargo run`

### Adding New Features

1. Create a new module in `src/`
2. Add necessary endpoints and logic
3. Update the main application to include the module
4. Add tests for the new functionality

## Security

- Rate limiting is configured in the Nginx reverse proxy
- Login endpoints have strict rate limits (1 req/s with burst of 5)
- General API endpoints are limited to 10 req/s with burst of 20
- SSL/TLS encryption via Let's Encrypt (Certbot)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development workflow guidelines
- Code standards and style guide
- Testing requirements
- Pull request process

## CI/CD

This project uses GitHub Actions for continuous integration and deployment:

- **CI Pipeline:** Automatically runs tests, linting, and security checks on every push and pull request
- **Deployment Pipeline:** Automatically deploys to production VPS on merges to the `main` branch

For detailed information on setting up and using the CI/CD pipelines, see [.github/GITHUB_ACTIONS_SETUP.md](.github/GITHUB_ACTIONS_SETUP.md).

## Troubleshooting

### Service won't start
```bash
# Check service status and logs
sudo systemctl status jeebs
sudo journalctl -u jeebs -n 50

# Verify the binary exists
ls -l /path/to/JeebsAI/target/release/jeebs
```

### Database issues
```bash
# Check database file permissions
ls -l jeebs.db

# Restore from backup
./restore.sh
```

### Port conflicts
```bash
# Check if port 8080 is in use
sudo netstat -tlnp | grep 8080

# Or use:
sudo lsof -i :8080
```

### Local development with Docker Compose (recommended) ⚙️

You can run Jeebs in a live-development container (source mounted, automatic rebuilds).

- Use the standalone dev compose file:

  ```sh
  docker compose -f docker-compose.dev.yml up --build
  ```
  - Runs `cargo watch -x run` inside the container for automatic rebuild & restart.
  - Source is mounted into the container; changes trigger a restart.

- Or use the override file (already included) which works with the main compose file:

  ```sh
  docker compose up --build
  ```
  (the `docker-compose.override.yml` mounts your source and runs `cargo watch`)

Notes:
- Persistent DB is stored in the `data` Docker volume (mounted to `/data`).
- For faster iteration, `./target` is mounted into the container to reuse artifacts.
- Healthcheck pings `/` (the web UI); `/health` requires an authenticated admin session.

Quick cleanup:
- Reset DB: stop containers then `docker volume rm <project>_data` (or remove the `data/` folder if you bind-mounted it).

### Docker (production)

Run the release container with a persistent data volume:

```sh
docker compose up --build -d
```

Notes:
- The container exposes port `8080` by default.
- SQLite data lives in the `data` Docker volume mounted at `/data`.
- Update `.env` to override `DATABASE_URL` or `RUST_LOG`.


## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Issue Templates
When opening an issue, please use the provided templates:
- **Bug Report**: For reporting errors or unexpected behavior.
- **Feature Request**: For suggesting new ideas or improvements.

## Roadmap

- [ ] **v0.2.0**: Enhanced Plugin System with hot-reloading.
- [ ] **v0.3.0**: Distributed Brain (P2P knowledge sharing).
- [ ] **v1.0.0**: Full Self-Evolution capabilities enabled.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
This project is modularized and ready for further extension.
