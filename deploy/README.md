# VPS deployment instructions

These commands assume an Ubuntu-like VPS.

1) Connect to the VPS:

```bash
ssh vps-user@vps-host
```

2) Install system dependencies:

```bash
sudo apt update
sudo apt install -y git python3 python3-venv python3-pip curl
```

3) Clone the repo:

```bash
sudo mkdir -p /opt/jeebsai && sudo chown $USER:$USER /opt/jeebsai
git clone https://github.com/Deployed-Labs/JeebsAI.git /opt/jeebsai
cd /opt/jeebsai
git checkout main
```

4) Install app dependencies and initialize config/database:

```bash
chmod +x install.sh status.sh uninstall.sh
./install.sh
```

5) Start JeebsAI:

```bash
source venv/bin/activate
gunicorn -w 4 -b 0.0.0.0:8000 app.app:app
```

6) Check status:

```bash
./status.sh
curl -fsS http://localhost:8000/health
```

7) To redeploy new code:

```bash
cd /opt/jeebsai
bash deploy/redeploy.sh
```
