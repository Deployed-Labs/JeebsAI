#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $0 user@host [remote_dir] [-i ssh_key]

Example:
  $0 ubuntu@1.2.3.4 /opt/jeebs -i ~/.ssh/id_rsa

This script builds the release, packages the binary and webui, uploads to the server,
and installs the systemd unit + env file and starts the `jeebs` service.
EOF
  exit 1
}

if [ "$#" -lt 1 ]; then
  usage
fi

DEST="$1"
REMOTE_DIR="${2:-/opt/jeebs}"
SSH_KEY=""

shift || true
shift || true

while [[ $# -gt 0 ]]; do
  case "$1" in
    -i|--identity)
      SSH_KEY="$2"; shift 2;;
    -h|--help)
      usage;;
    *) echo "Unknown arg: $1"; usage;;
  esac
done

SSH_OPTS=("")
if [ -n "$SSH_KEY" ]; then
  SSH_OPTS+=("-i" "$SSH_KEY")
fi

echo "Building release..."
cargo build --release

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Packaging files..."
mkdir -p "$TMPDIR/jeebs-deploy"
cp target/release/jeebs "$TMPDIR/jeebs-deploy/" \
   packaging/jeebs.service packaging/jeebs.env.example || true
cp -r webui "$TMPDIR/jeebs-deploy/" || true

tar -C "$TMPDIR" -czf "$TMPDIR/jeebs-deploy.tar.gz" jeebs-deploy

echo "Uploading to ${DEST}..."
scp ${SSH_OPTS[@]} "$TMPDIR/jeebs-deploy.tar.gz" "${DEST}:/tmp/jeebs-deploy.tar.gz"

echo "Installing on remote host..."
ssh ${SSH_OPTS[@]} "${DEST}" bash -s <<'REMOTE'
set -euo pipefail
REMOTE_DIR="${REMOTE_DIR}"
sudo mkdir -p "${REMOTE_DIR}"
sudo tar -C "${REMOTE_DIR}" -xzf /tmp/jeebs-deploy.tar.gz
sudo install -m 755 "${REMOTE_DIR}/jeebs" /usr/local/bin/jeebs
sudo useradd --system --no-create-home --shell /usr/sbin/nologin --user-group jeebs 2>/dev/null || true
sudo mkdir -p /var/lib/jeebs/webui
sudo cp -r "${REMOTE_DIR}/webui/"* /var/lib/jeebs/webui/ || true
sudo chown -R jeebs:jeebs /var/lib/jeebs || true
if [ ! -f /etc/jeebs.env ]; then
  sudo cp "${REMOTE_DIR}/packaging/jeebs.env.example" /etc/jeebs.env
  sudo chmod 640 /etc/jeebs.env
fi
sudo cp "${REMOTE_DIR}/packaging/jeebs.service" /etc/systemd/system/jeebs.service
sudo systemctl daemon-reload
sudo systemctl enable --now jeebs
sudo systemctl status --no-pager jeebs || true
REMOTE

echo "Deployment finished. Check service logs with: sudo journalctl -u jeebs -f on the server."
