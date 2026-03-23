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

Notes:
- The service exposes port 8080 by default.
- SQLite database is stored in a named Docker volume `jeebs_data`.
- Use `.env.example` as a starting point for environment variables.
