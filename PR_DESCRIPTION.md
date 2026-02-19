## Dev container + CI

This PR adds:
- `.devcontainer/devcontainer.json` — VS Code devcontainer (uses project Dockerfile)
- `docker-compose.dev.yml` — standalone dev compose for live reload
- `.github/workflows/ci-dev.yml` — GitHub Actions workflow (build + test + clippy)
- README updates with usage instructions

Everything needed to iterate locally and run CI.
