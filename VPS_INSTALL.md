# 🚀 VPS Fresh Installation Guide

## Quick Install (One Command)

**On your VPS (Ubuntu/Debian):**

```bash
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

Or download and run:

```bash
wget https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh
chmod +x vps_fresh_install.sh
sudo ./vps_fresh_install.sh
```

**That's it!** The script will install everything automatically.

---

## What the Script Does

1. ✅ **Updates system packages**
2. ✅ **Installs dependencies** (build-essential, git, sqlite3, etc.)
3. ✅ **Installs Rust** (if not present)
4. ✅ **Clones repository** to `/opt/jeebs`
5. ✅ **Creates database** and runs migrations
6. ✅ **Builds release binary** (takes 5-10 minutes)
7. ✅ **Creates systemd service**
8. ✅ **Starts JeebsAI** automatically
9. ✅ **Performs health check**
10. ✅ **Shows access information**

---

## Requirements

- **OS**: Ubuntu 20.04+ or Debian 10+ (recommended)
- **RAM**: 1GB minimum, 2GB+ recommended
- **Disk**: 2GB free space minimum
- **Access**: Root or sudo privileges

---

## Customization

You can customize the installation with environment variables:

```bash
# Custom installation directory
export APP_DIR="/home/jeebs/app"

# Custom database location
export DB_PATH="/home/jeebs/data/jeebs.db"

# Custom port
export APP_PORT="3000"

# Custom repository (if you forked)
export REPO_URL="https://github.com/YOUR_USERNAME/JeebsAI.git"

# Run with custom settings
sudo -E ./vps_fresh_install.sh
```

---

## After Installation

### Access JeebsAI

**Locally:**
```bash
curl http://localhost:8080
```

**From your browser:**
```
http://YOUR_VPS_IP:8080
```

### Check Service Status

```bash
sudo systemctl status jeebs
```

### View Logs

```bash
sudo journalctl -u jeebs -f
```

### Restart Service

```bash
sudo systemctl restart jeebs
```

---

## Set Up Domain & SSL (Recommended)

### 1. Point Domain to VPS

Create an A record:
```
jeebs.yourdomain.com  →  YOUR_VPS_IP
```

### 2. Install Nginx

```bash
sudo apt install nginx
```

### 3. Create Nginx Config

```bash
sudo nano /etc/nginx/sites-available/jeebs
```

Paste this:
```nginx
server {
    listen 80;
    server_name jeebs.yourdomain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Enable it:
```bash
sudo ln -s /etc/nginx/sites-available/jeebs /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

### 4. Install SSL Certificate

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d jeebs.yourdomain.com
```

Follow prompts. Certificate will auto-renew!

Now access via: `https://jeebs.yourdomain.com`

---

## Firewall Configuration

### UFW (Ubuntu Firewall)

```bash
# Allow SSH (important!)
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS (if using nginx)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Or allow JeebsAI port directly
sudo ufw allow 8080/tcp

# Enable firewall
sudo ufw enable
```

---

## Troubleshooting

### Installation Fails

**Check logs:**
```bash
tail -f /tmp/jeebs_install.log
```

**Common issues:**

1. **Out of memory during build:**
   ```bash
   # Create swap file
   sudo fallocate -l 2G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   
   # Then retry installation
   ```

2. **Rust installation fails:**
   ```bash
   # Install manually
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   
   # Then retry installation
   ```

3. **Port already in use:**
   ```bash
   # Check what's using port 8080
   sudo netstat -tlnp | grep 8080
   
   # Kill process or change port
   export APP_PORT="3000"
   sudo -E ./vps_fresh_install.sh
   ```

### Service Won't Start

```bash
# Check logs
sudo journalctl -u jeebs -n 50

# Check permissions
sudo chown -R root:root /opt/jeebs
sudo chmod +x /opt/jeebs/target/release/jeebs

# Check database
ls -la /var/lib/jeebs/

# Try manual start
cd /opt/jeebs
./target/release/jeebs
```

### Can't Access from Browser

```bash
# Check if service is running
sudo systemctl status jeebs

# Check if port is open
sudo netstat -tlnp | grep 8080

# Check firewall
sudo ufw status

# Test locally
curl http://localhost:8080
```

---

## Manual Installation (Alternative)

If you prefer to install manually:

```bash
# 1. Update system
sudo apt update && sudo apt upgrade -y

# 2. Install dependencies
sudo apt install -y build-essential pkg-config libssl-dev sqlite3 git curl

# 3. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 4. Clone repository
sudo mkdir -p /opt/jeebs
sudo git clone https://github.com/Deployed-Labs/JeebsAI.git /opt/jeebs
cd /opt/jeebs

# 5. Create database directory
sudo mkdir -p /var/lib/jeebs

# 6. Run migrations
for f in migrations/*.sql; do
    sudo sqlite3 /var/lib/jeebs/jeebs.db < "$f"
done

# 7. Build
cargo build --release

# 8. Create service file
sudo nano /etc/systemd/system/jeebs.service
# (paste service configuration from script)

# 9. Start service
sudo systemctl daemon-reload
sudo systemctl enable jeebs
sudo systemctl start jeebs
```

---

## Uninstall

To completely remove JeebsAI:

```bash
# Stop and disable service
sudo systemctl stop jeebs
sudo systemctl disable jeebs
sudo rm /etc/systemd/system/jeebs.service
sudo systemctl daemon-reload

# Remove files
sudo rm -rf /opt/jeebs
sudo rm -rf /var/lib/jeebs
sudo rm -rf /var/backups/jeebs

# Remove Rust (optional)
rustup self uninstall
```

---

## Update JeebsAI

After initial installation, use the update script:

```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

Or manually:

```bash
cd /opt/jeebs
sudo systemctl stop jeebs
sudo git pull origin main
cargo build --release
sudo systemctl start jeebs
```

---

## System Requirements

### Minimum:
- 1 CPU core
- 1GB RAM
- 2GB disk space
- Ubuntu 20.04 or Debian 10

### Recommended:
- 2+ CPU cores
- 2GB+ RAM
- 5GB+ disk space
- Ubuntu 22.04 or Debian 11

### For Production:
- 4+ CPU cores
- 4GB+ RAM
- 20GB+ disk space
- Nginx reverse proxy
- SSL certificate
- Regular backups

---

## Security Recommendations

1. **Use firewall**: Enable UFW and allow only necessary ports
2. **Use SSL**: Install certbot for HTTPS
3. **Regular updates**: Keep system packages updated
4. **Strong passwords**: Use strong admin passwords
5. **SSH keys**: Use SSH keys instead of passwords
6. **Fail2ban**: Install fail2ban for brute-force protection
7. **Backups**: Set up automated database backups

---

## Getting Help

If you encounter issues:

1. **Check logs**: `sudo journalctl -u jeebs -n 100`
2. **Check service**: `sudo systemctl status jeebs`
3. **Test locally**: `curl http://localhost:8080`
4. **Read troubleshooting**: See above section
5. **Check documentation**: Review README.md

---

## Quick Reference

| Command | Description |
|---------|-------------|
| `sudo systemctl status jeebs` | Check service status |
| `sudo systemctl restart jeebs` | Restart service |
| `sudo systemctl stop jeebs` | Stop service |
| `sudo systemctl start jeebs` | Start service |
| `sudo journalctl -u jeebs -f` | View logs (live) |
| `sudo journalctl -u jeebs -n 50` | View last 50 log lines |
| `cd /opt/jeebs && sudo ./deploy_to_vps.sh` | Update JeebsAI |

---

## What's Next?

After installation:

1. ✅ Access web interface at `http://YOUR_VPS_IP:8080`
2. ✅ Create your first admin account
3. ✅ Set up domain and SSL (optional but recommended)
4. ✅ Configure firewall
5. ✅ Start using JeebsAI!
6. ✅ Read `QUICK_START.md` for feature guide

---

**Installation complete! Enjoy your JeebsAI! 🎉**
