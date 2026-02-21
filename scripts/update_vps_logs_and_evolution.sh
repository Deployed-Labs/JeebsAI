#!/usr/bin/env bash
set -euo pipefail

# JeebsAI canonical VPS updater.
#
# Modes:
# 1) Local launcher (from your laptop/dev machine):
#    ./scripts/update_vps_logs_and_evolution.sh --host 192.227.193.148
#
# 2) Remote in-place (already on the VPS):
#    sudo ./update_vps_logs_and_evolution.sh
#
# Optional local args:
#   --host <ip-or-hostname>   (if set, script runs in SSH mode)
#   --user <ssh-user>         (default: root)
#   --ssh-port <port>         (default: 22)
#
# Optional remote args:
#   --service <name>          (default: jeebs)
#   --repo-dir <path>         (default: detect from systemd, fallback /root/JeebsAI)
#   --repo-url <url>          (default: https://github.com/Deployed-Labs/JeebsAI.git)
#   --branch <name>           (default: main)
#   --env-file <path>         (default: detect from systemd)
#   --default-db <path>       (default: ./jeebs.db)

detect_working_dir() {
  local service="$1"
  systemctl show -p WorkingDirectory --value "$service" 2>/dev/null || true
}

detect_env_file() {
  local service="$1"
  local raw
  raw="$(systemctl show -p EnvironmentFiles --value "$service" 2>/dev/null || true)"
  # Example: "/etc/jeebs/config.env (ignore_errors=yes)"
  echo "$raw" | awk '{print $1}'
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

coerce_db_path() {
  local repo_dir="$1"
  local db_url="$2"
  local db_path
  db_path="${db_url#sqlite:}"
  if [[ "$db_path" == "$db_url" ]]; then
    echo ""
    return 0
  fi
  if [[ "$db_path" == ///* ]]; then
    db_path="/${db_path#///}"
  elif [[ "$db_path" == //* ]]; then
    db_path="/${db_path#//}"
  fi
  if [[ "$db_path" != /* ]]; then
    db_path="$repo_dir/$db_path"
  fi
  echo "$db_path"
}

wait_for_route() {
  local port="$1"
  local path="$2"
  local max_attempts="${3:-30}"
  local attempt=1
  while [[ "$attempt" -le "$max_attempts" ]]; do
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 3 "http://127.0.0.1:${port}${path}" 2>/dev/null || true)"
    if [[ "$code" == "200" || "$code" == "401" || "$code" == "403" ]]; then
      return 0
    fi
    sleep 1
    attempt=$((attempt + 1))
  done
  return 1
}

check_route() {
  local port="$1"
  local path="$2"
  local method="${3:-GET}"
  local code
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 10 -X "$method" "http://127.0.0.1:${port}${path}" || true)"
  case "$code" in
    200|401|403)
      echo "ok ${path} -> ${code}"
      ;;
    *)
      echo "ERROR ${path} -> ${code}" >&2
      exit 1
      ;;
  esac
}

run_remote_update() {
  local service_name="$1"
  local repo_dir="$2"
  local repo_url="$3"
  local branch="$4"
  local env_file="$5"
  local default_db_path="$6"

  need_cmd git
  need_cmd curl
  need_cmd systemctl
  need_cmd sqlite3

  if [[ ! -d "$repo_dir/.git" ]]; then
    rm -rf "$repo_dir"
    git clone "$repo_url" "$repo_dir"
  fi

  cd "$repo_dir"
  echo "Updating repository in $repo_dir..."
  git fetch --all --prune
  git reset --hard
  git clean -fd
  git checkout -B "$branch" "origin/$branch"
  git reset --hard "origin/$branch"

  # Ensure cargo/rustup for whichever account is executing this script.
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi

  echo "Building release binary..."
  cargo build --release

  local db_url=""
  if [[ -n "$env_file" && -f "$env_file" ]]; then
    db_url="$(grep -E '^DATABASE_URL=' "$env_file" | tail -n1 | cut -d= -f2- || true)"
  fi
  if [[ -z "$db_url" ]]; then
    db_url="sqlite:${default_db_path}"
  fi

  local db_path
  db_path="$(coerce_db_path "$repo_dir" "$db_url")"
  if [[ -n "$db_path" ]]; then
    mkdir -p "$(dirname "$db_path")"
    touch "$db_path"
    echo "Ensuring system_logs table/indexes exist at $db_path..."
    sqlite3 "$db_path" <<'SQL'
CREATE TABLE IF NOT EXISTS system_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp TEXT NOT NULL,
  level TEXT NOT NULL,
  category TEXT NOT NULL,
  message TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_system_logs_timestamp ON system_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_system_logs_category ON system_logs(category);
CREATE INDEX IF NOT EXISTS idx_system_logs_level ON system_logs(level);
SQL
  else
    echo "DATABASE_URL is not sqlite-based ('$db_url'); skipping sqlite repair."
  fi

  echo "Restarting service: $service_name"
  systemctl daemon-reload
  systemctl restart "$service_name"
  systemctl is-active "$service_name"

  local port="8080"
  if [[ -n "$env_file" && -f "$env_file" ]]; then
    local env_port
    env_port="$(grep -E '^PORT=' "$env_file" | tail -n1 | cut -d= -f2- || true)"
    if [[ -n "$env_port" ]]; then
      port="$env_port"
    fi
  fi

  if ! wait_for_route "$port" "/api/auth/status" 30; then
    echo "ERROR service did not become ready on /api/auth/status in time" >&2
    systemctl status --no-pager "$service_name" || true
    exit 1
  fi

  echo "Running endpoint checks on 127.0.0.1:$port..."
  check_route "$port" "/api/auth/status"
  check_route "$port" "/api/admin/logs"
  check_route "$port" "/api/admin/evolution/updates"
  check_route "$port" "/api/admin/evolution/status"
  check_route "$port" "/api/admin/training/status"
  check_route "$port" "/api/admin/crawl/random" "POST"
  check_route "$port" "/webui/evolution.html"

  if [[ -n "$db_path" ]]; then
    local count
    count="$(sqlite3 "$db_path" "SELECT COUNT(*) FROM system_logs;" 2>/dev/null || echo "unknown")"
    echo "system_logs_rows: $count"
  fi

  echo "Deploy complete."
  echo "repo_dir: $repo_dir"
  echo "env_file: ${env_file:-<not set>}"
  echo "commit: $(git rev-parse --short HEAD)"
}

run_local_launcher() {
  local host="$1"
  local user="$2"
  local ssh_port="$3"
  local service_name="$4"
  local repo_dir="$5"
  local repo_url="$6"
  local branch="$7"
  local env_file="$8"
  local default_db_path="$9"

  need_cmd ssh

  # Helpful guardrail: this script deploys from remote GitHub branch, not local dirty files.
  if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    if ! git diff --quiet || ! git diff --cached --quiet; then
      echo "WARNING: local working tree has uncommitted changes." >&2
      echo "WARNING: this deploy uses origin/${branch} on VPS and will NOT include uncommitted local changes." >&2
    fi
    if git show-ref --verify --quiet "refs/remotes/origin/${branch}"; then
      local local_head remote_head
      local_head="$(git rev-parse "${branch}" 2>/dev/null || true)"
      remote_head="$(git rev-parse "origin/${branch}" 2>/dev/null || true)"
      if [[ -n "$local_head" && -n "$remote_head" && "$local_head" != "$remote_head" ]]; then
        echo "WARNING: local ${branch} differs from origin/${branch}." >&2
        echo "WARNING: push your branch first if you want those commits on VPS." >&2
      fi
    fi
  fi

  if [[ -z "$host" ]]; then
    cat <<'USAGE' >&2
Usage:
  ./scripts/update_vps_logs_and_evolution.sh --host <ip-or-hostname> [--user root] [--ssh-port 22]

Examples:
  ./scripts/update_vps_logs_and_evolution.sh --host 192.227.193.148
  ./scripts/update_vps_logs_and_evolution.sh --host jeebs.club --user root --branch main
USAGE
    exit 1
  fi

  echo "Launching remote update on ${user}@${host}:${ssh_port}..."
  ssh -p "$ssh_port" "${user}@${host}" bash -s -- \
    --remote \
    --service "$service_name" \
    --repo-dir "$repo_dir" \
    --repo-url "$repo_url" \
    --branch "$branch" \
    --env-file "$env_file" \
    --default-db "$default_db_path" < "$0"
}

main() {
  local mode="auto"
  local host=""
  local user="root"
  local ssh_port="22"

  local service_name="jeebs"
  local repo_url="https://github.com/Deployed-Labs/JeebsAI.git"
  local branch="main"
  local env_file=""
  local default_db_path="./jeebs.db"

  local detected_repo_dir=""
  if command -v systemctl >/dev/null 2>&1; then
    detected_repo_dir="$(detect_working_dir "$service_name")"
  fi
  local repo_dir="${detected_repo_dir:-/root/JeebsAI}"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --remote)
        mode="remote"
        shift
        ;;
      --host)
        host="${2:-}"
        shift 2
        ;;
      --user)
        user="${2:-}"
        shift 2
        ;;
      --ssh-port)
        ssh_port="${2:-}"
        shift 2
        ;;
      --service)
        service_name="${2:-}"
        shift 2
        ;;
      --repo-dir)
        repo_dir="${2:-}"
        shift 2
        ;;
      --repo-url)
        repo_url="${2:-}"
        shift 2
        ;;
      --branch)
        branch="${2:-}"
        shift 2
        ;;
      --env-file)
        env_file="${2:-}"
        shift 2
        ;;
      --default-db)
        default_db_path="${2:-}"
        shift 2
        ;;
      -*)
        echo "Unknown option: $1" >&2
        exit 1
        ;;
      *)
        # Backward-compatible positional host shortcut.
        if [[ -z "$host" ]]; then
          host="$1"
          shift
        else
          echo "Unexpected argument: $1" >&2
          exit 1
        fi
        ;;
    esac
  done

  if [[ -z "$env_file" ]] && command -v systemctl >/dev/null 2>&1; then
    env_file="$(detect_env_file "$service_name")"
  fi

  if [[ "$mode" == "remote" ]]; then
    run_remote_update "$service_name" "$repo_dir" "$repo_url" "$branch" "$env_file" "$default_db_path"
    return
  fi

  if [[ -n "$host" ]]; then
    run_local_launcher "$host" "$user" "$ssh_port" "$service_name" "$repo_dir" "$repo_url" "$branch" "$env_file" "$default_db_path"
    return
  fi

  # No host provided: run in-place on current machine (useful on VPS directly).
  run_remote_update "$service_name" "$repo_dir" "$repo_url" "$branch" "$env_file" "$default_db_path"
}

main "$@"
