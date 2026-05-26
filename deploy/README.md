# VPS deployment instructions (no Docker)

These commands assume an Ubuntu-like VPS.

## 1) SSH into your VPS

```bash
ssh vps-user@vps-host
```

## 2) Install system packages

```bash
sudo apt update
sudo apt install -y git python3 python3-venv python3-pip
```

## 3) Clone JeebsAI and install

```bash
sudo mkdir -p /opt/jeebsai && sudo chown $USER:$USER /opt/jeebsai
git clone https://github.com/Deployed-Labs/JeebsAI.git /opt/jeebsai
cd /opt/jeebsai
./install.sh
```

`install.sh` creates `venv`, installs dependencies, creates `.env`, and initializes the database.

Optional: set `DATABASE_PATH` in `/opt/jeebsai/.env` if you want the SQLite file somewhere else.

## 4) Start JeebsAI immediately

```bash
cd /opt/jeebsai
source venv/bin/activate
gunicorn -w 4 -b 0.0.0.0:8000 app.app:app
```

## 5) Run JeebsAI automatically with systemd

Create `/etc/systemd/system/jeebsai.service`:

```ini
[Unit]
Description=JeebsAI Gunicorn Service
After=network.target

[Service]
Type=simple
User=<replace-with-your-vps-username-e.g.-ubuntu>
WorkingDirectory=/opt/jeebsai
EnvironmentFile=/opt/jeebsai/.env
ExecStart=/opt/jeebsai/venv/bin/gunicorn -w 4 -b 0.0.0.0:8000 app.app:app
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Then enable/start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable jeebsai
sudo systemctl start jeebsai
sudo systemctl status jeebsai
```

## 6) Update after new commits

```bash
cd /opt/jeebsai
git pull origin main
source venv/bin/activate
pip install -r requirements.txt
sudo systemctl restart jeebsai
```
