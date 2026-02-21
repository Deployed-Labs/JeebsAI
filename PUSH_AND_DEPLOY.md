# 🚀 PUSH TO MAIN AND DEPLOY - QUICK INSTRUCTIONS

## Step 1: Push Everything to Main

Run this on your **LOCAL machine** (macOS):

```bash
chmod +x push_to_main.sh
./push_to_main.sh
```

Follow the prompts:
1. Confirm you want to commit changes → **y**
2. Enter commit message or press Enter for default
3. Confirm push to main → **y**

**Done!** Your code is now on the main branch.

---

## Step 2: Deploy to VPS

### Option A: Automatic Deploy from Local Machine

**First time setup (edit once):**
```bash
nano push_and_deploy.sh
```
Change these lines:
```bash
VPS_HOST="your-vps-hostname-or-ip"  # e.g., "my-server.com" or "123.45.67.89"
VPS_USER="root"                      # or "ubuntu" or your username
```

**Then run:**
```bash
chmod +x push_and_deploy.sh
./push_and_deploy.sh
```

It will push to git AND deploy to VPS automatically!

---

### Option B: Manual Deploy on VPS

**SSH into your VPS:**
```bash
ssh root@your-vps.com
```

**Copy deployment script (first time only):**
```bash
cd /opt/jeebs
wget https://raw.githubusercontent.com/YOUR-USERNAME/JeebsAI/main/deploy_to_vps.sh
chmod +x deploy_to_vps.sh
```

Or copy from local:
```bash
# On local machine:
scp deploy_to_vps.sh root@your-vps.com:/opt/jeebs/
```

**Run deployment:**
```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

---

## What Happens During Deployment

✅ Backs up database  
✅ Stops the service  
✅ Pulls latest code from main  
✅ Builds release binary  
✅ Runs database migrations  
✅ Starts the service  
✅ Checks everything works  

---

## After Deployment

**Check if it's working:**
```bash
# On VPS:
sudo systemctl status jeebs
sudo journalctl -u jeebs -n 20
```

**Test the web interface:**
```bash
curl http://localhost:8080/webui/index.html
```

Or visit in browser: `http://your-vps.com` (or IP address)

---

## If Something Goes Wrong

**Restore from backup:**
```bash
sudo systemctl stop jeebs
sudo cp /var/backups/jeebs/jeebs_*.db /var/lib/jeebs/jeebs.db
sudo systemctl start jeebs
```

**Check logs:**
```bash
sudo journalctl -u jeebs -n 100
```

**Rebuild manually:**
```bash
cd /opt/jeebs
cargo build --release
sudo systemctl restart jeebs
```

---

## 📚 More Info

- **Detailed guide**: See `DEPLOYMENT.md`
- **Script reference**: See `DEPLOYMENT_SCRIPTS.md`
- **New features**: See `LEARNING_SYSTEM.md`

---

## ⚡ Super Quick Deploy (One Command)

If everything is already set up:

```bash
./push_and_deploy.sh
```

That's it! Answer the prompts and you're done.

---

## 🎯 First Time Setup Checklist

Before first deployment:

- [ ] VPS is running Ubuntu/Debian
- [ ] Rust installed on VPS (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- [ ] Git installed on VPS (`sudo apt install git`)
- [ ] Repository cloned to `/opt/jeebs` on VPS
- [ ] SSH access configured (preferably with key)
- [ ] Scripts are executable (`chmod +x *.sh`)

If this is your first deployment, see `DEPLOYMENT.md` for full setup instructions.

---

**That's all! Push to main and deploy. Easy! 🚀**
