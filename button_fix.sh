#!/bin/bash
cd /root/JeebsAI
# Remove the broken onclick text and fix the button label
sed -i 's/Login".*"/Login/g' home.html
sed -i 's/onclick=.*Connecting...\x27/onclick="this.innerText=\x27Connecting...\x27"/g' home.html
echo "✅ Button text fixed."
