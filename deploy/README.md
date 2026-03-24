# VPS deployment instructions

These commands assume an Ubuntu-like VPS and that you control DNS for `jeebs.club` (point an A record to the VPS IP).

1) Connect to the VPS (replace `vps-user` and `vps-host`):

```bash
ssh vps-user@vps-host
```

2) Install Docker and the Compose plugin (Ubuntu 22.04+):

```bash
sudo apt update
sudo apt install -y ca-certificates curl gnupg lsb-release
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
sudo usermod -aG docker $USER
```

Log out and back in if you want to run docker without `sudo`.

3) Configure firewall (optional):

```bash
sudo ufw allow 22/tcp
sudo ufw allow 80,443/tcp
sudo ufw enable
```

4) Clone the repo and switch to `main`:

```bash
sudo mkdir -p /opt/jeebsai && sudo chown $USER:$USER /opt/jeebsai
git clone https://github.com/Deployed-Labs/JeebsAI.git /opt/jeebsai
cd /opt/jeebsai
git fetch --all && git checkout main
```

5) Review `deploy/Caddyfile` and update the `tls` email address if needed.

6) Start the stack with Docker Compose:

```bash
sudo docker compose -f deploy/docker-compose.prod.yml up -d --build
```

7) Verify services and logs:

```bash
sudo docker compose -f deploy/docker-compose.prod.yml ps
sudo docker compose -f deploy/docker-compose.prod.yml logs -f caddy
sudo docker compose -f deploy/docker-compose.prod.yml logs -f web
```

8) To update after pushing new commits:

```bash
cd /opt/jeebsai
git pull origin main
sudo docker compose -f deploy/docker-compose.prod.yml up -d --build
```

Notes:
- Make sure DNS A record for `jeebs.club` points to the VPS public IP. Caddy will automatically obtain TLS certs for the domain.
- If you are behind a firewall or cloud provider, open ports 80 and 443.
