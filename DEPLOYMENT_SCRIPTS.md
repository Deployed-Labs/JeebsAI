# 🚀 Deployment Scripts - Quick Reference

## Available Scripts

| Script | Purpose | Run On | Description |
|--------|---------|--------|-------------|
| `push_to_main.sh` | Push to Git | **Local** | Commit changes and push to main branch |
| `push_and_deploy.sh` | Full Deploy | **Local** | Push to git + deploy to VPS in one go |
| `deploy_to_vps.sh` | VPS Deploy | **VPS** | Pull latest and rebuild on VPS |

---

## 🎯 Quick Start (Choose One)

### Option A: Everything in One Script (Easiest)

**On your local machine:**
```bash
chmod +x push_and_deploy.sh

# Edit configuration (first time only)
nano push_and_deploy.sh
# Set: VPS_HOST, VPS_USER

# Run it
./push_and_deploy.sh
```

This will:
1. Show git status
2. Optionally commit your changes
3. Push to main
4. Deploy to VPS automatically

---

### Option B: Two Steps (More Control)

#### Step 1: Push to Main (Local)
```bash
chmod +x push_to_main.sh
./push_to_main.sh
```

#### Step 2: Deploy on VPS
```bash
ssh root@your-vps.com
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

---

### Option C: Manual (Full Control)

**Local machine:**
```bash
git add .
git commit -m "Your message"
git push origin main
```

**On VPS:**
```bash
ssh root@your-vps.com
cd /opt/jeebs
git pull origin main
cargo build --release
sudo systemctl restart jeebs
```

---

## 📋 Script Details

### 1. `push_to_main.sh` - Git Push Script

**What it does:**
- ✅ Checks current branch
- ✅ Shows git status
- ✅ Adds all changes
- ✅ Commits with custom or default message
- ✅ Pushes to origin/main
- ✅ Optionally creates release tag

**Usage:**
```bash
./push_to_main.sh
```

**Prompts:**
- Switch to main? (if on different branch)
- Add and commit all changes?
- Enter commit message
- Push to origin/main?
- Create a release tag?

---

### 2. `push_and_deploy.sh` - Local Push + Remote Deploy

**What it does:**
- ✅ All features of `push_to_main.sh`
- ✅ Uploads `deploy_to_vps.sh` to VPS
- ✅ Executes deployment remotely via SSH

**Configuration needed:**
```bash
export VPS_HOST="your-server.com"
export VPS_USER="root"
export VPS_APP_DIR="/opt/jeebs"
```

**Usage:**
```bash
# Method 1: Set environment variables
export VPS_HOST="my-server.com"
export VPS_USER="ubuntu"
./push_and_deploy.sh

# Method 2: Edit script directly
nano push_and_deploy.sh  # Edit VPS_HOST and VPS_USER
./push_and_deploy.sh

# Method 3: One-liner
VPS_HOST="my-server.com" VPS_USER="root" ./push_and_deploy.sh
```

**Requirements:**
- SSH access to VPS
- SSH key authentication (recommended) or password
- `scp` and `ssh` commands available

---

### 3. `deploy_to_vps.sh` - VPS Deployment Script

**What it does:**
- ✅ Backs up database to `/var/backups/jeebs/`
- ✅ Stops jeebs service
- ✅ Pulls latest from main
- ✅ Builds release binary
- ✅ Runs database migrations
- ✅ Starts jeebs service
- ✅ Checks service status
- ✅ Performs health check

**Usage on VPS:**
```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

**Configuration (environment variables):**
```bash
export APP_DIR="/opt/jeebs"
export DB_PATH="/var/lib/jeebs/jeebs.db"
export BACKUP_DIR="/var/backups/jeebs"
sudo ./deploy_to_vps.sh
```

**What gets backed up:**
- Database file (keeps last 10 backups)
- Location: `/var/backups/jeebs/jeebs_TIMESTAMP.db`

---

## 🔒 Security Best Practices

### SSH Key Authentication (Recommended)

**Set up SSH key (one-time):**
```bash
# On local machine
ssh-keygen -t ed25519 -C "your-email@example.com"

# Copy to VPS
ssh-copy-id root@your-vps.com

# Test
ssh root@your-vps.com "echo 'SSH key works!'"
```

Now `push_and_deploy.sh` won't need password!

---

### Using a Deploy User (More Secure)

Instead of `root`, create a deploy user:

**On VPS:**
```bash
# Create deploy user
sudo useradd -m -s /bin/bash deploy
sudo usermod -aG sudo deploy

# Set up permissions
sudo chown -R deploy:deploy /opt/jeebs
sudo visudo
# Add: deploy ALL=(ALL) NOPASSWD: /bin/systemctl restart jeebs
```

**On local machine:**
```bash
# Update script
export VPS_USER="deploy"
./push_and_deploy.sh
```

---

## 🔧 Customization

### Change Default Commit Message

Edit `push_to_main.sh`:
```bash
commit_message="Your default message here - $(date '+%Y-%m-%d')"
```

### Change Backup Retention

Edit `deploy_to_vps.sh`:
```bash
# Change this line (default: 10)
ls -t jeebs_*.db | tail -n +11 | xargs -r rm
# To keep 20 backups:
ls -t jeebs_*.db | tail -n +21 | xargs -r rm
```

### Skip Health Check

Edit `deploy_to_vps.sh`:
```bash
# Comment out the health_check call in main()
# health_check
```

### Clean Build Every Time

Edit `deploy_to_vps.sh`:
```bash
# Uncomment this line in build_app()
cargo clean
```

---

## 🐛 Troubleshooting

### "Permission denied" when running script

```bash
chmod +x push_to_main.sh
chmod +x push_and_deploy.sh
chmod +x deploy_to_vps.sh
```

### "Not a git repository"

```bash
cd /path/to/JeebsAI
git init
git remote add origin https://github.com/YOUR-USERNAME/JeebsAI.git
```

### SSH connection fails

```bash
# Test SSH
ssh -v root@your-vps.com

# Check SSH config
cat ~/.ssh/config

# Verify VPS_HOST is correct
echo $VPS_HOST
```

### Build fails on VPS

```bash
# Check Rust installation
cargo --version

# Reinstall if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Service won't start after deployment

```bash
# Check logs
sudo journalctl -u jeebs -n 100

# Check permissions
sudo chown -R root:root /opt/jeebs
sudo chmod +x /opt/jeebs/target/release/jeebs

# Try manual start
cd /opt/jeebs
./target/release/jeebs
```

### Rollback deployment

```bash
# On VPS
sudo systemctl stop jeebs
cd /opt/jeebs
git log --oneline  # Find previous commit
git checkout <commit-hash>
cargo build --release
sudo cp /var/backups/jeebs/jeebs_TIMESTAMP.db /var/lib/jeebs/jeebs.db
sudo systemctl start jeebs
```

---

## 📊 Monitoring After Deployment

### Watch logs in real-time
```bash
ssh root@your-vps.com "sudo journalctl -u jeebs -f"
```

### Check if deployed successfully
```bash
ssh root@your-vps.com "systemctl is-active jeebs && echo 'Service is running!'"
```

### View recent logs
```bash
ssh root@your-vps.com "sudo journalctl -u jeebs -n 50 --no-pager"
```

### Test web interface
```bash
ssh root@your-vps.com "curl -f http://localhost:8080/webui/index.html && echo 'Web UI is up!'"
```

---

## 🎯 Workflow Examples

### Regular Update Workflow
```bash
# 1. Make changes to code
nano src/cortex.rs

# 2. Test locally
cargo run

# 3. Push and deploy
./push_and_deploy.sh
# Answer prompts: yes, yes, yes

# 4. Monitor
ssh root@vps "journalctl -u jeebs -f"
```

### Emergency Hotfix
```bash
# 1. Quick fix
nano src/auth/mod.rs

# 2. Commit and push
git add .
git commit -m "hotfix: Fix critical auth bug"
git push origin main

# 3. Deploy immediately
ssh root@vps "cd /opt/jeebs && sudo ./deploy_to_vps.sh"
```

### Release Process
```bash
# 1. Push all changes
./push_to_main.sh
# Create tag: v2.1.0

# 2. Deploy to VPS
./push_and_deploy.sh

# 3. Verify deployment
ssh root@vps "systemctl status jeebs"

# 4. Test new features
curl http://your-vps.com/api/knowledge/stats
```

---

## 📝 Checklist

Before running deployment:

- [ ] All changes tested locally
- [ ] Database migrations created (if needed)
- [ ] `.gitignore` excludes sensitive files
- [ ] Commit message is descriptive
- [ ] VPS has enough disk space
- [ ] VPS backup is recent
- [ ] Know how to rollback if needed

After deployment:

- [ ] Service started successfully
- [ ] No errors in logs
- [ ] Web UI accessible
- [ ] API endpoints working
- [ ] Database migrations applied
- [ ] New features tested

---

## 💡 Pro Tips

1. **Test locally first**: Always run `cargo test` and `cargo run` before deploying

2. **Use tags for releases**: Tag important versions with `git tag v1.0.0`

3. **Monitor during deployment**: Watch logs while deploying with `journalctl -f`

4. **Keep backups**: Deployment script auto-backs up database before changes

5. **Gradual rollout**: Test on staging VPS before production

6. **Document changes**: Update CHANGELOG.md with each deployment

7. **Set up monitoring**: Use tools like Prometheus/Grafana for production

---

## 🆘 Getting Help

If deployment fails:

1. **Check the logs**: `journalctl -u jeebs -n 100`
2. **Verify the build**: `cd /opt/jeebs && cargo build --release`
3. **Check permissions**: `ls -la /opt/jeebs`
4. **Restore backup**: `cp /var/backups/jeebs/latest.db /var/lib/jeebs/jeebs.db`
5. **Ask for help**: Include logs and error messages

---

## 📚 Related Documentation

- **DEPLOYMENT.md** - Detailed deployment guide
- **README.md** - Main project documentation
- **LEARNING_SYSTEM.md** - New features documentation
- **QUICK_START.md** - User quick start guide

---

**Ready to deploy? Choose a script above and get started! 🚀**
