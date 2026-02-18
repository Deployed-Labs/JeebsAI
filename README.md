
# JeebsAI — Docker deployment (VPS)

JeebsAI is a Rust-based AI assistant with a lightweight web UI and SQLite-backed storage.
This repository already includes a production `Dockerfile` and `docker-compose.yml`.

Official container image: `ghcr.io/Deployed-Labs/jeebs:latest` — pull instead of building if you prefer.

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
  deployed-labs/jeebs:latest
```

Important:
- Data path inside container: `/data` (map to host, e.g. `/var/lib/jeebs`).
- Environment vars: `PORT`, `DATABASE_URL`, `RUST_LOG`.
- Use an external reverse proxy (Nginx/Caddy/Traefik) for TLS and a domain.

Verify / logs:

```bash
docker ps
docker logs -f jeebs
```

That's it — the app will persist its SQLite DB in the mounted `/data` directory.

---

License: MIT

