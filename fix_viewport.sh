#!/bin/bash
cd /root/JeebsAI
# Injects the mobile-friendly meta tag if it is missing
if ! grep -q "viewport" index.html; then
  sed -i '/<head>/a \    <meta name="viewport" content="width=device-width, initial-scale=1.0">' index.html
fi
echo "📱 Mobile scaling fix applied."
