# Docker quickstart for JeebsAI


Build and run locally with Docker Compose:

```bash
# build image and start container
docker compose build --pull --no-cache
docker compose up -d

# view logs
docker compose logs -f

# stop and remove
docker compose down
```

Publish images (CI)

This repo includes a `publish` workflow which builds and pushes images to GitHub Container Registry (GHCR) on pushes to `main`.
To enable pushing to Docker Hub as well, set `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` in GitHub Secrets.

Systemd auto-start

To install a systemd unit that will auto-start `docker compose` on boot, run as root:

```bash
sudo ./scripts/install_service.sh
```

The unit file is `packaging/jeebs-docker.service`.

Notes:
- The service exposes port 8080 by default.
- SQLite database is stored in a named Docker volume `jeebs_data`.
- Use `.env.example` as a starting point for environment variables.

