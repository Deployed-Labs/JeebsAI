#!/usr/bin/env bash
# RUN ME - Makes everything executable and deploys

echo "🔧 Making deployment scripts executable..."
chmod +x DEPLOY_EVERYTHING.sh
chmod +x RUN_THIS_NOW.sh
chmod +x pull_from_github.sh
chmod +x go.sh
chmod +x auto_deploy.sh
echo "✅ Done!"
echo ""
echo "🚀 Starting deployment..."
echo ""

# Run the main deployment
./DEPLOY_EVERYTHING.sh
