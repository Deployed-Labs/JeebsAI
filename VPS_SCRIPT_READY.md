# ✅ COMPLETE - VPS Installation Script Ready!

## 🎯 Your VPS Installation Script

I've created **`vps_fresh_install.sh`** - a complete automated installation script for your VPS!

---

## 🚀 How to Use It

### **Option 1: Direct Install (Easiest)**

**On your VPS, run:**

```bash
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

### **Option 2: Download First**

```bash
wget https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh
chmod +x vps_fresh_install.sh
sudo ./vps_fresh_install.sh
```

### **Option 3: After Pushing to GitHub**

```bash
# First push to main (on local machine):
./push_to_main.sh

# Then on VPS:
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

---

## 📋 What the Script Does

The script automatically handles everything:

1. ✅ **Updates system** packages
2. ✅ **Installs dependencies**:
   - build-essential
   - pkg-config
   - libssl-dev
   - sqlite3
   - git
   - curl
   - wget
3. ✅ **Installs Rust** (if not present)
4. ✅ **Clones repository** to `/opt/jeebs`
5. ✅ **Creates database** directory
6. ✅ **Runs migrations** from `migrations/` folder
7. ✅ **Builds release binary** (~5-10 minutes)
8. ✅ **Creates systemd service**
9. ✅ **Enables and starts** service
10. ✅ **Performs health check**
11. ✅ **Displays access info**

**Total install time: 5-15 minutes** (depending on VPS specs)

---

## 🎛️ Customization Options

You can customize the installation with environment variables:

```bash
# Custom installation directory
export APP_DIR="/home/myuser/jeebs"

# Custom database path
export DB_PATH="/home/myuser/data/jeebs.db"

# Custom port (default: 8080)
export APP_PORT="3000"

# Custom repository URL (if forked)
export REPO_URL="https://github.com/YOUR_USERNAME/JeebsAI.git"

# Run with custom settings
sudo -E ./vps_fresh_install.sh
```

**Default configuration:**
- Install directory: `/opt/jeebs`
- Database: `/var/lib/jeebs/jeebs.db`
- Port: `8080`
- Service name: `jeebs`
- User: `root`

---

## 📁 Files Created

### Installation Script:
- **`vps_fresh_install.sh`** - Complete VPS setup script

### Documentation:
- **`VPS_INSTALL.md`** - Comprehensive installation guide
- **`INSTALL_VPS.md`** - Quick reference

### Updated:
- **`README.md`** - Added VPS quick install section

---

## 🔍 What Happens After Installation

### Service is Running:
```bash
sudo systemctl status jeebs
# Should show: Active: active (running)
```

### Access Points:
- **Local**: `http://localhost:8080`
- **Remote**: `http://YOUR_VPS_IP:8080`
- **Web UI**: `/webui/index.html`

### Useful Commands:
```bash
# View logs
sudo journalctl -u jeebs -f

# Restart service
sudo systemctl restart jeebs

# Stop service
sudo systemctl stop jeebs

# Check installation info
cat /opt/jeebs/INSTALLATION_INFO.txt
```

---

## 🛡️ Security & Production Setup

### After Installation, Set Up:

**1. Firewall:**
```bash
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable
```

**2. Domain & SSL:**
```bash
# Install nginx
sudo apt install nginx

# Configure nginx reverse proxy
# (see VPS_INSTALL.md for config)

# Install SSL certificate
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d yourdomain.com
```

**3. Set Up Backups:**
```bash
# Create backup script
cat > /root/backup_jeebs.sh <<'EOF'
#!/bin/bash
cp /var/lib/jeebs/jeebs.db /var/backups/jeebs/jeebs_$(date +%Y%m%d_%H%M%S).db
EOF

chmod +x /root/backup_jeebs.sh

# Add to crontab (daily backup at 2 AM)
echo "0 2 * * * /root/backup_jeebs.sh" | crontab -
```

---

## 🐛 Troubleshooting

### Installation Fails

**Out of memory during build:**
```bash
# Create swap file
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Retry installation
sudo ./vps_fresh_install.sh
```

**Port already in use:**
```bash
# Check what's using port 8080
sudo netstat -tlnp | grep 8080

# Install on different port
export APP_PORT="3000"
sudo -E ./vps_fresh_install.sh
```

### Service Won't Start

```bash
# Check logs
sudo journalctl -u jeebs -n 100

# Check binary exists
ls -la /opt/jeebs/target/release/jeebs

# Try manual start
cd /opt/jeebs
./target/release/jeebs
```

### Can't Access from Browser

```bash
# Check firewall
sudo ufw status

# Allow port
sudo ufw allow 8080/tcp

# Test locally first
curl http://localhost:8080
```

---

## 🔄 Future Updates

After initial installation, update using:

```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

This will:
- Back up database
- Pull latest code
- Rebuild
- Restart service

---

## 📊 System Requirements

### Minimum:
- 1 CPU core
- 1GB RAM (2GB recommended for build)
- 2GB disk space
- Ubuntu 20.04 or Debian 10

### Recommended:
- 2+ CPU cores
- 2GB+ RAM
- 5GB+ disk space
- Ubuntu 22.04 LTS

### Production:
- 4+ CPU cores
- 4GB+ RAM
- 20GB+ SSD
- Ubuntu 22.04 LTS
- Nginx reverse proxy
- SSL certificate

---

## ⚡ Complete Workflow

### Full Deployment from Scratch:

**On Local Machine:**
```bash
# 1. Make scripts executable
chmod +x push_to_main.sh
./push_to_main.sh

# This pushes to GitHub main branch
```

**On VPS:**
```bash
# 2. Run fresh install
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash

# 3. Access JeebsAI
# Visit: http://YOUR_VPS_IP:8080
```

**That's it! JeebsAI is live!**

---

## 📚 Documentation Reference

| File | Purpose |
|------|---------|
| `vps_fresh_install.sh` | Automated VPS installation script |
| `VPS_INSTALL.md` | Complete installation guide |
| `INSTALL_VPS.md` | Quick reference card |
| `deploy_to_vps.sh` | Update existing installation |
| `push_to_main.sh` | Push code to GitHub |
| `push_and_deploy.sh` | Push + deploy in one |

---

## ✨ Features Available After Install

All the new features are included:

- ✅ **Active Sessions** - Track user sessions in admin dashboard
- ✅ **Language Learning** - Automatic vocabulary tracking
- ✅ **Knowledge Retrieval** - Multi-source intelligent search
- ✅ **Proactive Proposals** - Jeebs suggests actions
- ✅ **API Endpoints** - `/api/knowledge/stats`, `/api/language/stats`

Test them:
- Visit admin dashboard: `/webui/admin_dashboard.html`
- Chat: `knowledge stats`, `vocabulary stats`
- Ask: `what do you want to do?`

---

## 🎉 Summary

**You now have:**

1. ✅ Complete VPS installation script
2. ✅ Full documentation
3. ✅ Quick reference guides
4. ✅ Update/deployment scripts
5. ✅ All features implemented

**To deploy right now:**

```bash
# On VPS:
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

**Or after pushing to GitHub:**

```bash
# Local:
./push_to_main.sh

# VPS:
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

---

## 🚀 Ready to Deploy!

Everything is ready. Just:

1. Push to GitHub: `./push_to_main.sh`
2. Run on VPS: `curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/JeebsAI/main/vps_fresh_install.sh | sudo bash`

**Your intelligent AI assistant will be live in ~10 minutes! 🎉**
