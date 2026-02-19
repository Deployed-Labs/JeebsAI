#!/bin/bash
cd /root/JeebsAI
# Inject correct library services into main.rs
sed -i 's|.service(auth::login)|.service(jeebs::auth::login)|g' src/main.rs
sed -i 's|.service(auth::login_pgp)|.service(jeebs::auth::login_pgp)|g' src/main.rs
# Restart the service
sudo systemctl daemon-reload
sudo systemctl restart jeebs
echo "✅ Backend rebuilding. Check status: sudo journalctl -u jeebs -f"
