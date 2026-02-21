#!/usr/bin/env bash
# Make all deployment scripts executable

chmod +x push_to_main.sh
chmod +x push_and_deploy.sh
chmod +x deploy_to_vps.sh

echo "✅ All deployment scripts are now executable!"
echo ""
echo "Available scripts:"
echo "  - push_to_main.sh       (Push to git main branch)"
echo "  - push_and_deploy.sh    (Push to git + deploy to VPS)"
echo "  - deploy_to_vps.sh      (Deploy on VPS only)"
echo ""
echo "Quick start: ./push_to_main.sh"
