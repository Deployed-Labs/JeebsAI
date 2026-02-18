# Refactoring / Major rewrite — checklist

This document helps prepare the repository and CI for large-scale changes (rebases, rewrites, restructuring).

Before you start
- Create a new branch for your work: `git checkout -b refactor/<short-name>`
- Run the repo prep script: `./scripts/prep-refactor.sh`
  - Ensures `rustfmt`, `clippy`, `cargo test` pass and that the working tree is clean
  - Scans for obvious secret/token patterns

Plugin development (Python / Node)
- New polyglot plugin model: place each plugin under `plugins/<plugin-name>/` and provide a runner:
  - `run` (executable) or `run.py` (Python) or `run.js` / `index.js` (Node)
  - Contract: reads JSON from stdin `{ "input": "..." }` and writes `{ "response": "..." }` to stdout
- Examples: `plugins/python-echo` and `plugins/node-hello` (included in `refactor/prep`).
- The Rust core discovers and executes these plugins via `load_dynamic_plugins("plugins")` — no Rust code changes are required to add a new plugin.

Continuous integration
- CI already runs `cargo fmt`, `clippy`, build, tests and a security audit on PRs.
- A `Secret scan` workflow (gitleaks) is included to block accidental secret commits.

Pull request process
1. Push your branch and open a PR against `main`.
2. Re-run CI until green.
3. Rebase onto `main` only when CI & tests pass locally.
4. Use small, reviewable commits where possible.

If you need help
- Ask the maintainer to enable `GHCR_PAT` repository secret (CI publishing).
- If you want, I can create the refactor branch and open a draft PR for you.
