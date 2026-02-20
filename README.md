# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI and persistent storage.
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Installation Matrix](#installation-matrix)
  - [Prebuilt Binary](#prebuilt-binary)
  - [Release Tarball](#release-tarball)
  - [Debian / Ubuntu Package (.deb)](#debian--ubuntu-package-deb)
  - [Docker](#docker)
  - [One-Click VPS Install](#one-click-vps-install)
  - [Local Development](#local-development)
  - [VPS Production Deployment](#vps-production-deployment)
- [Configuration](#configuration)
- [Usage](#usage)
- [Plugins](#plugins)
- [Project Structure](#project-structure)
- [Development](#development)
- [Contributing](#contributing)
- [CI/CD](#cicd)

## Prerequisites

### Local Development
- Rust 1.70+ and Cargo
- SQLite 3

### System Build Dependencies
Required to compile on Ubuntu/Debian:
```bash
sudo apt update && sudo apt install -y \
  build-essential clang pkg-config libssl-dev sqlite3 \
  nettle-dev libgpg-error-dev libgcrypt-dev
```

### VPS Production Deployment
- Ubuntu/Debian-based VPS (recommended)
- Rust 1.70+ and Cargo (only needed when building from source)
- SQLite 3
- Nginx (for reverse proxy)
- Certbot (for SSL/TLS certificates)
- systemd (for process management)

## Installation

### Installation Matrix

| Method | Platform | Requires Rust? | Notes |
|---|---|---|---|
| [Prebuilt binary](#prebuilt-binary) | Linux x86_64, macOS | No | Fastest way to get started |
| [Release tarball](#release-tarball) | Linux x86_64 | No | Binary + systemd setup in one archive |
| [.deb package](#debian--ubuntu-package-deb) | Debian / Ubuntu | No | `apt`-friendly; sets up systemd service |
| [Docker](#docker) | Any OS with Docker | No | Isolated; easiest to update |
| [One-click VPS install](#one-click-vps-install) | Ubuntu / Debian VPS | Auto-installed | All-in-one from source |
| [Local development](#local-development) | Any | Yes | For contributors |

---

### Prebuilt Binary

Download the latest binary for your platform from the [Releases page](https://github.com/Deployed-Labs/JeebsAI/releases):

```bash
# Linux x86_64
curl -fsSL https://github.com/Deployed-Labs/JeebsAI/releases/latest/download/jeebs-linux-x86_64 \
  -o jeebs && chmod +x jeebs && ./jeebs

# macOS Apple Silicon (aarch64)
curl -fsSL https://github.com/Deployed-Labs/JeebsAI/releases/latest/download/jeebs-macos-aarch64 \
  -o jeebs && chmod +x jeebs && ./jeebs

# macOS Intel (x86_64)
curl -fsSL https://github.com/Deployed-Labs/JeebsAI/releases/latest/download/jeebs-macos-x86_64 \
  -o jeebs && chmod +x jeebs && ./jeebs
```

The binary reads configuration from environment variables or `/etc/jeebs.env`. See [Configuration](#configuration) for details.

---

### Release Tarball

The tarball bundles the Linux binary, systemd service file, environment example, and an install script.

```bash
# Download the latest tarball
TAG=$(curl -s https://api.github.com/repos/Deployed-Labs/JeebsAI/releases/latest \
  | grep '"tag_name"' | cut -d'"' -f4)
curl -fsSL "https://github.com/Deployed-Labs/JeebsAI/releases/download/${TAG}/jeebs-${TAG}-linux-x86_64.tar.gz" \
  | tar -xz

# Install (sets up binary + systemd service)
cd "jeebs-${TAG}-linux-x86_64"
sudo ./install.sh
```

After installation:
- Binary: `/usr/local/bin/jeebs`
- Config:  `/etc/jeebs.env`
- Logs:    `sudo journalctl -u jeebs -f`

---

### Debian / Ubuntu Package (.deb)

Download the `.deb` from the [Releases page](https://github.com/Deployed-Labs/JeebsAI/releases) and install with `apt`:

```bash
TAG=$(curl -s https://api.github.com/repos/Deployed-Labs/JeebsAI/releases/latest \
  | grep '"tag_name"' | cut -d'"' -f4)
curl -fsSLO "https://github.com/Deployed-Labs/JeebsAI/releases/download/${TAG}/jeebs_${TAG#v}_amd64.deb"
sudo apt install ./jeebs_${TAG#v}_amd64.deb
```

The package:
- Installs the binary to `/usr/local/bin/jeebs`
- Installs and enables the systemd service automatically
- Creates `/etc/jeebs.env` from the example if it does not exist

---

### Docker

#### Pull the published image

```bash
docker pull ghcr.io/deployed-labs/jeebsai:latest
docker run -d \
  --name jeebs \
  -p 8080:8080 \
  -v jeebs-data:/var/lib/jeebs \
  ghcr.io/deployed-labs/jeebsai:latest
```

#### docker compose (recommended)

```bash
# Clone the repository (only needed for the compose file)
git clone https://github.com/Deployed-Labs/JeebsAI.git && cd JeebsAI

# Start JeebsAI
docker compose up -d

# View logs
docker compose logs -f jeebs
```

#### Production docker compose

```bash
# Copy environment file and customise
sudo cp packaging/jeebs.env.example /etc/jeebs.env
sudo nano /etc/jeebs.env

# Start with production overrides (binds only to localhost)
docker compose -f docker-compose.production.yml up -d
```

#### Build image locally

```bash
docker build -t jeebsai .
docker run -d -p 8080:8080 -v jeebs-data:/var/lib/jeebs jeebsai
```

#### Updating

```bash
docker compose pull && docker compose up -d
```

---

### One-Click VPS Install

Installs Rust, builds JeebsAI, and configures systemd on a fresh Ubuntu/Debian VPS:

```bash
curl -fsSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/one-click.sh | sudo bash
```

With optional overrides:

```bash
APP_DIR=/opt/jeebs APP_USER=jeebs APP_PORT=8080 \
  DOMAIN=example.com EMAIL=admin@example.com \
  sudo -E bash one-click.sh

# Non-interactive overwrite of existing config/systemd/nginx files:
FORCE=1 sudo bash one-click.sh
```

---

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

---

### VPS Production Deployment

#### Manual Installation Steps

If you prefer manual installation, follow these steps:

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Set up the systemd service:**
   ```bash
   sudo cp jeebs.service /etc/systemd/system/jeebs.service
   sudo nano /etc/systemd/system/jeebs.service  # update paths/user
   sudo systemctl daemon-reload
   sudo systemctl enable jeebs
   sudo systemctl start jeebs
   ```

3. **Configure Nginx reverse proxy:**
   ```bash
   sudo cp jeebs.nginx /etc/nginx/sites-available/jeebs
   sudo nano /etc/nginx/sites-available/jeebs  # set your domain
   sudo ln -s /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/
   sudo nginx -t && sudo systemctl reload nginx
   ```

4. **Set up SSL with Certbot:**
   ```bash
   sudo certbot --nginx -d your_domain.com
   ```

5. **Initialize the database:**
   ```bash
   sqlite3 /var/lib/jeebs/jeebs.db < 20240101000000_initial_setup.sql
   ```

## Configuration

### Environment Variables

Edit `/etc/jeebs.env` to configure the application:

```bash
# Server configuration
PORT=8080

# Database configuration
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db

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

## Plugins

JeebsAI supports language-agnostic plugins via subprocess execution. Each plugin lives in its own directory under `plugins/` and communicates over stdin/stdout using JSON.

See [plugins/README.md](plugins/README.md) for:
- The plugin JSON contract
- How to install, write, and remove plugins
- Supported runner types (Python, Node.js, executable)

## Project Structure

- `src/`
	- `main.rs` — Application entry point, web server, and CLI
	- `admin/` — Admin features (user management, now in `admin/user/`)
	- `brain/` — Knowledge graph and training logic
	- `auth/` — Authentication, registration, and password reset
- `webui/` — Web user interface (HTML, JS, CSS)
- `plugins/` — Language-agnostic plugin examples and documentation
- `packaging/` — Deployment packaging files (systemd unit, env example, installers)
- `install.sh` — Automated installation script for VPS deployment
- `one-click.sh` — All-in-one install script for fresh VPS
- `start.sh` — Manual startup script
- `backup.sh` — Database backup script
- `jeebs.service` — systemd service template (VPS reference copy)
- `jeebs.nginx` — Nginx reverse proxy configuration template
- `Dockerfile` — Multi-stage Docker image definition

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
- **Release Pipeline:** Triggered on version tags (`v*`); builds Linux and macOS binaries, a release tarball, and a `.deb` package, then publishes them as a GitHub Release
- **Docker Pipeline:** Triggered on version tags (`v*`); builds and pushes the Docker image to `ghcr.io/deployed-labs/jeebsai`
- **Deployment Pipeline:** Automatically deploys to production VPS on merges to the `main` branch

For detailed information on setting up and using the CI/CD pipelines, see [.github/GITHUB_ACTIONS_SETUP.md](.github/GITHUB_ACTIONS_SETUP.md).

## Troubleshooting
 
## Authentication

- **Method:** PGP-clearsign based login using the `/api/login_pgp` endpoint.
- **How to login:** Create a message in the exact format `LOGIN:<username>:<unix_ts>` (for example `LOGIN:1090mb:1650000000`), then clearsign it with your PGP private key (for example `gpg --clearsign --armor message.txt`). Paste the full clearsigned output into the web UI PGP login box or POST JSON to `/api/login_pgp` like:

```json
{"signed_message": "-----BEGIN PGP SIGNED MESSAGE-----\n..."}
```

- **Timestamp window:** The server accepts signed messages with a timestamp within 5 minutes of the server clock. If your clock is skewed, signatures will be rejected.
- **Admin detection:** A user is treated as admin when the username is `1090mb` or the user's store entry has `role: "admin"`.
- **Cookies / sessions:** Sessions are stored using `actix-session` cookies. The frontend must send credentials (`credentials: 'same-origin'`) when calling login endpoints so the session cookie is set.
- **Troubleshooting 401s:** If you get HTTP 401 on login, check server logs with `sudo journalctl -u jeebs -f` for signature verification errors. Verify the signed message format, timestamp freshness, and that the signed output includes your PGP signature.

## Deploy notes

- If local `cargo build` fails due to host toolchain/LLVM mismatches, build on the target VPS instead using the provided `deploy_jeebs.sh` script. Example (non-interactive):

```bash
FORCE=true ./deploy_jeebs.sh /root/JeebsAI main true
```

This will clone/pull the repository on the VPS, build the release binary there, and restart the service so runtime behavior can be verified.


### Service won't start
```bash
# Check service status and logs
sudo systemctl status jeebs
sudo journalctl -u jeebs -n 50

# Verify the binary exists
ls -l /usr/local/bin/jeebs
# or for source installs:
ls -l /path/to/JeebsAI/target/release/jeebs
```

### Docker container won't start
```bash
# Check container logs
docker compose logs jeebs

# Inspect the running container
docker compose ps
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
