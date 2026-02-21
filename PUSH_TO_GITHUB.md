# 🚀 PUSH TO GITHUB - Quick Commands

## Simplest Method (One Command):

```bash
bash auto_deploy.sh
```

This does EVERYTHING automatically:
- Makes scripts executable
- Stages all changes  
- Commits with descriptive message
- Pushes to GitHub
- Shows you next steps

---

## Alternative Quick Methods:

### Method 1: Super Simple
```bash
bash deploy_now.sh
```

### Method 2: Manual Steps
```bash
git add webui/admin_dashboard.html pull_from_github.sh
git commit -m "Add Topic Learning feature"
git push origin main
```

---

# 📥 PULL ON VPS

## On Your VPS:

```bash
# 1. SSH to VPS
ssh your-user@your-vps-ip

# 2. Go to JeebsAI directory
cd ~/JeebsAI

# 3. Pull latest changes
bash pull_from_github.sh
```

The pull script will:
✅ Fetch latest code
✅ Auto-detect your deployment type
✅ Offer to restart Jeebs service
✅ Show status and next steps

---

## 🎯 What You're Deploying

**Topic Learning Feature** - A new section in the admin dashboard where you can:
- Enter any topic in a textbox
- Click LEARN (or press Enter)
- Watch Jeebs research and learn about it
- See real-time status updates

**Location:** Admin Dashboard → Topic Learning section (orange border)

---

## ✅ Quick Verification

After deploying on VPS, test it:

1. Go to: `http://your-vps-ip/webui/admin_dashboard.html`
2. Log in as admin (1090mb)
3. Find the orange "🎓 Topic Learning" section
4. Type a topic like "machine learning"
5. Click LEARN
6. Watch Jeebs learn! 🧠

---

## 📞 Need Help?

See `TOPIC_LEARNING_DEPLOYMENT.md` for:
- Detailed instructions
- Troubleshooting tips
- Manual deployment steps
- Feature usage guide

---

**Ready?** Just run: `bash auto_deploy.sh` 🚀
