# 🚀 Quick VPS Installation

## One-Command Install

**Run this on your VPS:**

```bash
curl -sSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh | sudo bash
```

Or:

```bash
wget https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/vps_fresh_install.sh
chmod +x vps_fresh_install.sh
sudo ./vps_fresh_install.sh
```

---

## What It Does

✅ Installs all dependencies  
✅ Installs Rust  
✅ Clones the repository  
✅ Builds JeebsAI  
✅ Creates systemd service  
✅ Starts the server  

**Installation takes 5-10 minutes**

---

## After Installation

**Access JeebsAI:**
```
http://YOUR_VPS_IP:8080
```

**Check status:**
```bash
sudo systemctl status jeebs
```

**View logs:**
```bash
sudo journalctl -u jeebs -f
```

---

## Full Documentation

See **VPS_INSTALL.md** for:
- Detailed installation guide
- SSL/domain setup
- Firewall configuration
- Troubleshooting
- Security recommendations

---

## Update Later

```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

---

**That's it! Simple installation! 🎉**
