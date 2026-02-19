#!/bin/bash
cd /root/JeebsAI

# 1. Restore a clean state if we broke it
git checkout src/main.rs 2>/dev/null || cp src/main.rs.bak src/main.rs 2>/dev/null

# 2. Use Python to safely swap the App block with the correct library namespaces
python3 -c '
import re
content = open("src/main.rs").read()

new_app_block = """App::new()
            .app_data(web::Data::new(state.clone()))
            .service(jeebs::auth::login)
            .service(jeebs::auth::login_pgp)
            .service(jeebs::auth::change_password)
            .service(jeebs::cortex::admin_train)
            .service(jeebs::cortex::admin_crawl)
            .service(jeebs::cortex::search_brain)
            .service(jeebs::cortex::reindex_brain)
            .service(jeebs::cortex::visualize_brain)
            .service(jeebs::cortex::get_logic_graph)"""

# This regex finds the App::new() down to the last service call before the closing bracket
pattern = r"App::new\(\).*?\.service\(jeebs::cortex::get_logic_graph\)"
updated = re.sub(pattern, new_app_block, content, flags=re.DOTALL)

with open("src/main.rs", "w") as f:
    f.write(updated)
'

echo "✅ Code fixed with correct namespaces."

# 3. Reload and Restart Systemd
sudo systemctl daemon-reload
sudo systemctl restart jeebs

echo "🚀 Jeebs is building/starting in the background."
echo "You can now do other stuff. To peek at progress later, run: sudo journalctl -u jeebs -n 50"
