#!/bin/bash
# Example script for PGP login to 1090mb account
# 
# Prerequisites:
# 1. Have the private key corresponding to the public key in src/auth/pgp.rs
# 2. GPG installed and configured with your private key
# 3. JeebsAI server running (e.g., on http://localhost:8080)

set -e

# Configuration
USERNAME="1090mb"
SERVER_URL="${JEEBS_SERVER_URL:-http://localhost:8080}"

echo "=== JeebsAI PGP Login Example ==="
echo "Username: $USERNAME"
echo "Server: $SERVER_URL"
echo

# Step 1: Create message with current timestamp
TIMESTAMP=$(date +%s)
MESSAGE="LOGIN:${USERNAME}:${TIMESTAMP}"
echo "Step 1: Creating message..."
echo "Message: $MESSAGE"
echo "$MESSAGE" > /tmp/jeebs_login_message.txt
echo

# Step 2: Sign the message
echo "Step 2: Signing message with GPG..."
echo "Note: You may be prompted for your GPG key passphrase"
gpg --clearsign --armor /tmp/jeebs_login_message.txt
SIGNED_MESSAGE=$(cat /tmp/jeebs_login_message.txt.asc)
echo "Message signed successfully"
echo

# Step 3: Send login request
echo "Step 3: Sending login request to server..."
cat > /tmp/jeebs_login_request.json <<EOF
{
  "username": "${USERNAME}",
  "signed_message": $(echo "$SIGNED_MESSAGE" | jq -Rs .),
  "remember_me": false
}
EOF

echo "Request payload created"
echo

# Step 4: Make the API call
echo "Step 4: Calling $SERVER_URL/api/login_pgp..."
RESPONSE=$(curl -s -X POST "$SERVER_URL/api/login_pgp" \
  -H "Content-Type: application/json" \
  -d @/tmp/jeebs_login_request.json \
  -c /tmp/jeebs_cookies.txt \
  -w "\nHTTP_CODE:%{http_code}")

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | grep -v "HTTP_CODE:")

echo "HTTP Status: $HTTP_CODE"
echo "Response: $BODY"
echo

# Step 5: Check result
if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Login successful!"
    echo "Session cookie saved to: /tmp/jeebs_cookies.txt"
    echo
    echo "You can now make authenticated requests using:"
    echo "  curl -b /tmp/jeebs_cookies.txt $SERVER_URL/api/..."
else
    echo "❌ Login failed"
    echo "Check the error message above for details"
    exit 1
fi

# Cleanup temporary files
rm -f /tmp/jeebs_login_message.txt /tmp/jeebs_login_message.txt.asc /tmp/jeebs_login_request.json

echo
echo "=== Done ==="
