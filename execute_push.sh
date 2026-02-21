#!/usr/bin/env bash
# Make scripts executable and push to GitHub

chmod +x quick_push.sh
chmod +x pull_from_github.sh

echo "✅ Made scripts executable"
echo ""

# Run the push script
./quick_push.sh
