# Topic Learning Feature - Deployment Guide

## 🚀 Quick Start

### Local Machine (Push to GitHub)

```bash
bash deploy_now.sh
```

That's it! This will:
- Stage your changes
- Commit with a descriptive message
- Push to GitHub

---

### VPS (Pull from GitHub)

1. SSH into your VPS:
```bash
ssh your-user@your-vps-ip
```

2. Navigate to JeebsAI:
```bash
cd ~/JeebsAI  # or wherever you installed it
```

3. Pull latest changes:
```bash
bash pull_from_github.sh
```

The script will:
- Pull latest code from GitHub
- Detect if you're using systemd or Docker
- Offer to restart Jeebs automatically

---

## 📋 What Was Added

### New Topic Learning Section
Located in `webui/admin_dashboard.html`:

- **Input Textbox**: Enter any topic for Jeebs to learn about
- **LEARN Button**: Triggers the learning process
- **Keyboard Support**: Press Enter to submit
- **Status Feedback**: Real-time updates with emojis and colors
- **Smart Integration**: Uses existing chat API seamlessly

### Example Topics You Can Enter:
- `quantum computing`
- `machine learning algorithms`
- `Rust programming best practices`
- `Docker containerization`
- `cybersecurity fundamentals`
- Literally anything!

---

## 🛠️ Scripts Created

### Local Scripts (for pushing)
- **`deploy_now.sh`** - Simplest one-liner deployment ✨
- **`deploy_topic_learning.sh`** - Full deployment with detailed output
- **`quick_push.sh`** - Just git operations

### VPS Scripts (for pulling)
- **`pull_from_github.sh`** - Smart pull script with service restart

---

## 📍 How to Use the Feature

1. Open admin dashboard:
   ```
   http://your-vps-ip/webui/admin_dashboard.html
   ```

2. Log in as root admin (1090mb)

3. Find the **Topic Learning** section (orange border)

4. Enter a topic in the textbox

5. Click **LEARN** or press Enter

6. Watch Jeebs research and learn! 🧠

---

## 🔧 Manual Deployment (if scripts don't work)

### Push from Local:
```bash
git add webui/admin_dashboard.html pull_from_github.sh
git commit -m "Add Topic Learning feature"
git push origin main
```

### Pull on VPS:
```bash
cd ~/JeebsAI
git pull origin main
sudo systemctl restart jeebs  # or: docker-compose up -d --build
```

---

## ✅ Verification

After deployment, verify the feature works:

1. Go to admin dashboard
2. See the orange "🎓 Topic Learning" section
3. Type a topic and click LEARN
4. Should see status updates and Jeebs' response

---

## 🆘 Troubleshooting

### Script not executable?
```bash
chmod +x deploy_now.sh
chmod +x pull_from_github.sh
```

### Git permission denied?
Make sure you have:
- SSH keys set up with GitHub
- Correct remote URL: `git remote -v`

### Jeebs not restarting?
```bash
# For systemd:
sudo systemctl status jeebs
sudo journalctl -u jeebs -n 50

# For Docker:
docker-compose logs --tail=50
```

---

## 🎉 You're Done!

The Topic Learning feature is now live on your VPS. Enjoy teaching Jeebs new things!
