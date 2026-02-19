#!/usr/bin/env bash
set -euo pipefail

# GitHub webhook setup guide.
# This script helps you configure a GitHub webhook to auto-deploy your code.

echo "=== JeebsAI GitHub Webhook Setup ==="
echo ""
echo "Step 1: Generate a webhook secret (keep it secret!)."
echo "Run this on your VPS and save the output:"
echo ""
echo "  SECRET=\$(openssl rand -hex 32); echo \$SECRET"
echo ""

read -p "Press Enter after you have your secret, or paste it now: " WEBHOOK_SECRET

if [[ -z "$WEBHOOK_SECRET" ]]; then
  WEBHOOK_SECRET=$(openssl rand -hex 32)
  echo "Generated secret: $WEBHOOK_SECRET"
fi

# Save secret to /etc/jeebs.env or /etc/jeebs-staging.env
ENV_FILE=${ENV_FILE:-"/etc/jeebs.env"}
echo ""
echo "Step 2: Adding GITHUB_WEBHOOK_SECRET to $ENV_FILE"
if [[ -f "$ENV_FILE" ]]; then
  if grep -q "^GITHUB_WEBHOOK_SECRET=" "$ENV_FILE"; then
    sed -i "s|^GITHUB_WEBHOOK_SECRET=.*|GITHUB_WEBHOOK_SECRET=$WEBHOOK_SECRET|" "$ENV_FILE"
  else
    echo "GITHUB_WEBHOOK_SECRET=$WEBHOOK_SECRET" >> "$ENV_FILE"
  fi
  echo "✓ Added to $ENV_FILE"
else
  echo "Creating $ENV_FILE with GITHUB_WEBHOOK_SECRET..."
  mkdir -p "$(dirname "$ENV_FILE")"
  cat > "$ENV_FILE" <<EOF
GITHUB_WEBHOOK_SECRET=$WEBHOOK_SECRET
PORT=8080
DATABASE_URL=sqlite:/var/lib/jeebs/jeebs.db
RUST_LOG=info
EOF
  echo "✓ Created $ENV_FILE"
fi

echo ""
echo "Step 3: Rebuild and restart the service to load the secret."
echo "  sudo systemctl restart jeebs"
echo ""
echo "Step 4: Set up webhook on GitHub:"
echo ""
echo "  1. Go to your GitHub repo settings: Settings > Webhooks"
echo "  2. Click 'Add webhook'"
echo "  3. Set:"
echo "     - Payload URL: https://your-domain.com/api/webhook/github"
echo "     - Content type: application/json"
echo "     - Secret: $WEBHOOK_SECRET"
echo "     - Events: Push events"
echo "     - Active: ✓"
echo "  4. Click 'Add webhook'"
echo ""
echo "Step 5: When you push to main branch, the webhook will:"
echo "  - Pull the latest code"
echo "  - Rebuild the release binary"
echo "  - Restart the jeebs service"
echo ""
echo "Done! Your auto-deploy is ready."
