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

---

## 🚀 Deployment

### 1. Fresh Installation (New VPS)
Run this single command on your Ubuntu/Debian VPS to install dependencies, build the app, and start the service.

```bash
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

### 2. Deploying Updates (From Local)
To push your local changes and automatically deploy them to your VPS:

```bash
# 1. Configure the script (first time only)
nano scripts/push_and_deploy.sh
# Set VPS_HOST="your-ip" and VPS_USER="root"

# 2. Run it
./scripts/push_and_deploy.sh
```

This script will:
1. Push your code to GitHub
2. SSH into your VPS
3. Pull the latest code
4. Rebuild and restart the service

### 3. Manual Deployment (On VPS)
If you are already logged into the VPS and want to update manually:

```bash
cd /root/JeebsAI
sudo ./deploy_to_vps.sh
```

### 4. Service Management

```bash
# Check status
sudo systemctl status jeebs

# View logs (real-time)
sudo journalctl -u jeebs -f

# Restart service
sudo systemctl restart jeebs
```

### 5. Backups & SSL

**SSL Setup (HTTPS):**
```bash
sudo ./scripts/setup_ssl.sh
```

**Database Backups:**
```bash
# Manual backup
sudo ./scripts/backup.sh

# Restore from backup
sudo ./scripts/restore.sh
```

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
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

## Configuration

### Environment Variables

Edit `/etc/jeebs.env` (on VPS) or `.env` (local) to configure the application:

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
# Make the backup script executable (if not already)
chmod +x scripts/backup.sh

# Run manual backup
# From the project root:
./scripts/backup.sh

# Set up automated nightly backups with cron
sudo crontab -e
# Add this line to run backup at 2 AM daily:
# 0 2 * * * /path/to/JeebsAI/scripts/backup.sh
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

## Recent changes (Feb 2026)

- Improved VPS deployment and nginx handling: `deploy_vps.sh` now
   supports optional `DOMAIN` configuration, ensures nginx is started,
   and uses `127.0.0.1` for upstreams to avoid IPv6 connection issues.
- Evolution UI updated to match the admin dashboard look-and-feel and
   backend guards were added so denied proposals cannot be applied.
- Rate limiting is now configurable with `RATE_PER_SECOND` and
   `RATE_BURST` environment variables; client-side retries/backoff were
   improved to reduce 429 noise in the UI.

These changes make deployment and management on modest VPS hosts
more reliable. See `deploy_vps.sh`, `setup_jeebs_nginx.sh`, and
`webui/evolution.html` for implementation details.

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
- **Deployment Pipeline:** Automatically deploys to production VPS when a new **Release** is published

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
