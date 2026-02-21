# Deployment Guide

## Quick Deploy from Local Machine

### Option 1: Automated Push and Deploy (Recommended)

1. **Configure the script** (edit `push_and_deploy.sh`):
   ```bash
   export VPS_HOST="your-server.com"  # or IP address
   export VPS_USER="root"              # or your user
   ```

2. **Run the script**:
   ```bash
   chmod +x push_and_deploy.sh
   ./push_and_deploy.sh
   ```

   This will:
   - Show git status
   - Ask if you want to commit changes
   - Push to main branch
   - Ask if you want to deploy to VPS
   - Upload and run deployment script on VPS
   - Build and restart the service

---

### Option 2: Manual Push, then Deploy

#### Step 1: Push to Main (Local Machine)

```bash
# Add all changes
git add .

# Commit changes
git commit -m "Add learning and knowledge systems"

# Push to main
git push origin main
```

#### Step 2: Deploy on VPS

**SSH into your VPS:**
```bash
ssh root@your-server.com
```

**Run deployment script:**
```bash
cd /opt/jeebs
sudo ./deploy_to_vps.sh
```

Or **copy the script to VPS and run:**
```bash
# From local machine:
scp deploy_to_vps.sh root@your-server.com:/opt/jeebs/

# Then on VPS:
ssh root@your-server.com
cd /opt/jeebs
chmod +x deploy_to_vps.sh
sudo ./deploy_to_vps.sh
```

---

## What the Deployment Script Does

1. ✅ **Backs up database** to `/var/backups/jeebs/`
2. ✅ **Stops the service** gracefully
3. ✅ **Pulls latest code** from main branch
4. ✅ **Builds release binary** with Cargo
5. ✅ **Runs database migrations** from `migrations/` folder
6. ✅ **Starts the service** with systemd
7. ✅ **Checks service status** and recent logs
8. ✅ **Performs health check** to verify it's working

---

## Manual Deployment Steps (if you prefer)

### On VPS:

```bash
# 1. Navigate to app directory
cd /opt/jeebs

# 2. Backup database
sudo cp /var/lib/jeebs/jeebs.db /var/backups/jeebs/jeebs_$(date +%Y%m%d_%H%M%S).db

# 3. Stop service
sudo systemctl stop jeebs

# 4. Pull latest code
git fetch origin
git checkout main
git pull origin main

# 5. Build release
cargo build --release

# 6. Run migrations (if any)
for f in migrations/*.sql; do
    sqlite3 /var/lib/jeebs/jeebs.db < "$f" 2>/dev/null || true
done

# 7. Start service
sudo systemctl start jeebs

# 8. Check status
sudo systemctl status jeebs

# 9. View logs
sudo journalctl -u jeebs -f
```

---

## Environment Variables

You can customize the deployment by setting these before running:

```bash
export VPS_HOST="my-server.com"
export VPS_USER="ubuntu"
export APP_DIR="/opt/jeebs"
export DB_PATH="/var/lib/jeebs/jeebs.db"

./push_and_deploy.sh
```

---

## Troubleshooting

### If deployment fails:

**Check logs:**
```bash
sudo journalctl -u jeebs -n 100
```

**Check service status:**
```bash
sudo systemctl status jeebs
```

**Restore from backup:**
```bash
sudo systemctl stop jeebs
sudo cp /var/backups/jeebs/jeebs_TIMESTAMP.db /var/lib/jeebs/jeebs.db
sudo systemctl start jeebs
```

**Rebuild manually:**
```bash
cd /opt/jeebs
cargo clean
cargo build --release
sudo systemctl restart jeebs
```

### If service won't start:

**Check permissions:**
```bash
sudo chown -R root:root /opt/jeebs
sudo chmod +x /opt/jeebs/target/release/jeebs
```

**Check database:**
```bash
ls -la /var/lib/jeebs/
sudo chmod 644 /var/lib/jeebs/jeebs.db
```

**Check port:**
```bash
sudo netstat -tlnp | grep 8080
# If port is in use, kill the process or change port in config
```

---

## Rollback

To rollback to a previous version:

```bash
# On VPS:
cd /opt/jeebs

# Stop service
sudo systemctl stop jeebs

# Checkout previous commit
git log --oneline -10  # Find the commit hash
git checkout <commit-hash>

# Rebuild
cargo build --release

# Restore database backup
sudo cp /var/backups/jeebs/jeebs_TIMESTAMP.db /var/lib/jeebs/jeebs.db

# Restart
sudo systemctl start jeebs
```

---

## Monitoring After Deployment

**Watch logs in real-time:**
```bash
sudo journalctl -u jeebs -f
```

**Check if service is running:**
```bash
sudo systemctl is-active jeebs
```

**Test the API:**
```bash
curl http://localhost:8080/webui/index.html
```

**Check database size:**
```bash
ls -lh /var/lib/jeebs/jeebs.db
```

---

## Automated Deployments (Future)

To set up automated deployments on git push:

1. **GitHub Actions** - Add `.github/workflows/deploy.yml`
2. **Webhook** - Set up a webhook listener on VPS
3. **CI/CD Pipeline** - Use GitLab CI, Jenkins, etc.

For now, use the provided scripts for manual deployment.

---

## Quick Reference

| Action | Command |
|--------|---------|
| Push & Deploy | `./push_and_deploy.sh` |
| Deploy Only (on VPS) | `sudo ./deploy_to_vps.sh` |
| Check Status | `sudo systemctl status jeebs` |
| View Logs | `sudo journalctl -u jeebs -f` |
| Restart Service | `sudo systemctl restart jeebs` |
| Stop Service | `sudo systemctl stop jeebs` |
| Start Service | `sudo systemctl start jeebs` |

---

## Post-Deployment Checklist

- [ ] Service is running: `systemctl is-active jeebs`
- [ ] No errors in logs: `journalctl -u jeebs -n 50`
- [ ] Web UI accessible: `curl http://localhost:8080`
- [ ] Database exists: `ls /var/lib/jeebs/jeebs.db`
- [ ] Backups created: `ls /var/backups/jeebs/`
- [ ] New features working (test active sessions, knowledge stats)

---

## Support

If you encounter issues:

1. Check the logs first: `sudo journalctl -u jeebs -n 100`
2. Verify database: `sqlite3 /var/lib/jeebs/jeebs.db ".tables"`
3. Check permissions: `ls -la /opt/jeebs`
4. Rebuild from scratch if needed
5. Restore from backup if database is corrupted

**All deployment scripts are designed to be safe with automatic backups!**
