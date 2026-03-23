#!/usr/bin/env bash
set -euo pipefail

# Pull the latest code from the GitHub repo on the VPS, then apply it using the
# safer build-first deployment flow.
#
# Usage:
#   ./scripts/apply_updates_from_github.sh
#   sudo ./scripts/apply_updates_from_github.sh
#   BRANCH=main APP_DIR=/root/JeebsAI ./scripts/apply_updates_from_github.sh

APP_DIR="${APP_DIR:-/root/JeebsAI}"
SERVICE_NAME="${SERVICE_NAME:-jeebs}"
BRANCH="${BRANCH:-main}"
REMOTE_NAME="${REMOTE_NAME:-origin}"
REPO_URL="${REPO_URL:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SAFE_DEPLOY_SCRIPT="${SCRIPT_DIR}/safe_production_deploy.sh"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    error "Required command not found: $1"
    exit 1
  fi
}

if [[ "${EUID}" -ne 0 ]]; then
  exec sudo -E "$0" "$@"
fi

need_cmd git
need_cmd sudo

if [[ ! -x "${SAFE_DEPLOY_SCRIPT}" ]]; then
  error "Safe deploy script not found or not executable: ${SAFE_DEPLOY_SCRIPT}"
  exit 1
fi

if [[ ! -d "${APP_DIR}/.git" ]]; then
  error "Expected a git repo at ${APP_DIR}"
  exit 1
fi

cd "${APP_DIR}"

if ! git remote get-url "${REMOTE_NAME}" >/dev/null 2>&1; then
  error "Git remote '${REMOTE_NAME}' is not configured in ${APP_DIR}"
  exit 1
fi

if [[ -n "${REPO_URL}" ]]; then
  CURRENT_REMOTE_URL="$(git remote get-url "${REMOTE_NAME}")"
  if [[ "${CURRENT_REMOTE_URL}" != "${REPO_URL}" ]]; then
    info "Updating ${REMOTE_NAME} remote URL"
    git remote set-url "${REMOTE_NAME}" "${REPO_URL}"
  fi
fi

if [[ -n "$(git status --porcelain)" ]]; then
  STASH_NAME="auto-stash-before-github-update-$(date +%Y%m%d_%H%M%S)"
  warn "Local changes found in ${APP_DIR}; stashing them as '${STASH_NAME}'"
  git stash push -u -m "${STASH_NAME}" >/dev/null
fi

CURRENT_REF="$(git rev-parse --short HEAD)"
info "Current deployed ref: ${CURRENT_REF}"

info "Fetching latest updates from ${REMOTE_NAME}/${BRANCH}"
git fetch "${REMOTE_NAME}" --prune

if ! git rev-parse --verify --quiet "${REMOTE_NAME}/${BRANCH}" >/dev/null 2>&1; then
  error "Could not resolve ${REMOTE_NAME}/${BRANCH}"
  exit 1
fi

TARGET_REF="$(git rev-parse --short "${REMOTE_NAME}/${BRANCH}")"
if [[ "${CURRENT_REF}" == "${TARGET_REF}" ]]; then
  success "Already up to date at ${TARGET_REF}; nothing to apply."
  exit 0
fi

info "Applying update ${CURRENT_REF} -> ${TARGET_REF}"

exec "${SAFE_DEPLOY_SCRIPT}" "${BRANCH}" "${APP_DIR}" "${SERVICE_NAME}"
