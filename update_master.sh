echo "[update_master] Pulling latest from GitHub..."
git pull origin main
echo "[update_master] Restarting JeebsAI service..."
systemctl restart jeebs || echo "[update_master] Warning: Could not restart jeebs service. Please check manually."
echo "[update_master] Done!"
#!/usr/bin/env bash
# update_master.sh - Pull latest from GitHub and deploy JeebsAI from /root/JeebsAI

set -e
cd /root/JeebsAI


# Stash any local changes to avoid pull conflicts
if ! git diff-index --quiet HEAD -- || ! git diff --quiet; then
	echo "[update_master] Local changes detected. Stashing before pull..."
	git stash save "Auto-stash before update_master.sh $(date +%Y%m%d_%H%M%S)"
fi

# ensure repo is up-to-date
git fetch origin
git checkout main
git pull origin main

# make deploy helper executable and run it (uses systemd if available)
chmod +x scripts/deploy.sh
sudo ./scripts/deploy.sh --path /root/JeebsAI --port 8080 --service jeebs

# OR, if you prefer manual steps:
# build
# cargo build --release

# stop existing service (systemd) or kill process on port 8080
# sudo systemctl stop jeebs.service || true
# lsof -ti:8080 | xargs -r kill -9

# run the new binary
# PORT=8080 ./target/release/jeebs &> /root/JeebsAI/jeebs.log &

# check logs and endpoints
# tail -n 200 /root/JeebsAI/jeebs.log
# curl -s http://localhost:8080/api/evolution/stats | jq .
# curl -s http://localhost:8080/webui/evolution.html | head -n 40
