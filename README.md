

# JeebsAI — One-Click VPS Deployment

Want to deploy JeebsAI on a fresh VPS in one step? Just run:

```bash
curl -fsSL https://raw.githubusercontent.com/Deployed-Labs/JeebsAI/main/one-click-vps.sh | bash
```

Or, if you already cloned the repo:

```bash
./one-click-vps.sh
```

This script will:
- Install Rust, git, and sqlite3 if missing
- Clone (or update) the JeebsAI repo
- Build the JeebsAI binary
- Install and start the systemd service (jeebs)
- Set up a default environment file at /etc/jeebs.env

After running, JeebsAI will be running as a systemd service. View logs with:

```bash
sudo journalctl -u jeebs -f
```

# JeebsAI — Docker deployment (VPS)

JeebsAI is a Rust-based AI assistant with a lightweight web UI and SQLite-backed storage.
This repository already includes a production `Dockerfile` and `docker-compose.yml`.

Official container image: `ghcr.io/deployed-labs/jeebs:latest` — pull instead of building if you prefer. (published by CI to GHCR)


---

## Quick — Deploy to a VPS with Docker (recommended) ✅

Prerequisites on the VPS:
- Docker Engine and Docker Compose (plugin) installed
- Port 8080 (or 80/443 if using a reverse proxy) open

Clone and run (recommended):

```bash
git clone https://github.com/Deployed-Labs/JeebsAI.git
cd JeebsAI
docker compose up -d --build
```

Run with docker (single container):

```bash
docker build -t deployed-labs/jeebs:latest .

docker run -d \
  --name jeebs \
  -p 8080:8080 \
  -v /var/lib/jeebs:/data \
  -e PORT=8080 \
  -e DATABASE_URL=sqlite:/data/jeebs.db \
  --restart unless-stopped \
  ghcr.io/deployed-labs/jeebs:latest
```

Important:
- Data path inside container: `/data` (map to host, e.g. `/var/lib/jeebs`).
- Environment vars: `PORT`, `DATABASE_URL`, `RUST_LOG`.
- Local builds require native packages: `nettle-dev`, `libgpg-error-dev`, `libgcrypt-dev`, `clang`, and `pkg-config`.
  - `./install.sh` and `one-click.sh` will install and verify these automatically; if verification fails the scripts print a clear, actionable apt command.
- Use an external reverse proxy (Nginx/Caddy/Traefik) for TLS and a domain.


## CI / GHCR (quick notes)
- CI publishes Docker images to GitHub Container Registry (GHCR) at `ghcr.io/deployed-labs/jeebs:latest`.
- To allow CI to push images automatically, add a repository secret named `GHCR_PAT` with **write:packages** scope:
  1. GitHub → Settings → Developer settings → Personal access tokens → Generate new token
  2. Select `write:packages` and create the token
  3. Repo → Settings → Secrets and variables → Actions → New repository secret → `GHCR_PAT`
- If `GHCR_PAT` is not provided, the workflow will try `GITHUB_TOKEN` (may be blocked by org policy).

Revoking a leaked token (do this immediately if you accidentally exposed a token):
1. GitHub → Settings → Developer settings → Personal access tokens → Revoke the token.
2. Remove any copies from your environment, CI secrets, or history.

Developer toolchain:
- Project requires Rust 1.88+. To update locally run:
  - `rustup toolchain install 1.88.0 && rustup override set 1.88.0`
  - Or run `./install.sh` which verifies native deps and sets the toolchain.



Verify / logs:

```bash
docker ps
docker logs -f jeebs
```

That's it — the app will persist its SQLite DB in the mounted `/data` directory.

---

## Systemd deployment (no Docker)

If you removed Docker and want to run `jeebs` directly on a Linux VPS using `systemd`, follow these steps.

1. Build and install the release binary:

```bash
cargo build --release
sudo install -m 755 target/release/jeebs /usr/local/bin/jeebs
```

2. Create a service user and runtime directories:

```bash
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --user-group jeebs || true
sudo mkdir -p /var/lib/jeebs/plugins
sudo cp -r webui /var/lib/jeebs/webui
sudo chown -R jeebs:jeebs /var/lib/jeebs
```

3. Create `/etc/jeebs.env` with required environment variables (example):

```ini
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
PORT=8080
# Optional: RUST_LOG=info
```

4. Install the systemd unit shipped in the repo and start the service:

```bash
sudo cp packaging/jeebs.service /etc/systemd/system/jeebs.service
sudo systemctl daemon-reload
sudo systemctl enable --now jeebs
sudo journalctl -u jeebs -f
```

Notes:
- The binary runs migrations on startup (no separate migration step required).
- Adjust `WorkingDirectory` or `EnvironmentFile` in `/etc/systemd/system/jeebs.service` if you prefer a different layout.
- Use a reverse proxy (nginx/Caddy) for TLS in production.


License: MIT

