
# JeebsAI

JeebsAI is a modular Rust-based AI assistant with a web UI and persistent storage.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Local Development](#local-development)
  - [VPS Production Deployment](#vps-production-deployment)
- [Configuration](#configuration)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Development](#development)

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

---
This project is modularized and ready for further extension.
