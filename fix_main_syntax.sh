#!/bin/bash
cd /root/JeebsAI

# Create a clean version of the App block
python3 -c '
import sys
content = open("src/main.rs").read()
# Find the App::new() block and replace it with a clean, working version
start_marker = "App::new()"
end_marker = ".run()"

new_app_block = """App::new()
            .app_data(web::Data::new(state.clone()))
            .service(auth::login)
            .service(auth::login_pgp)
            .service(auth::change_password)
            .service(cortex::admin_train)
            .service(cortex::admin_crawl)
            .service(cortex::search_brain)
            .service(cortex::reindex_brain)
            .service(cortex::visualize_brain)
            .service(cortex::get_logic_graph)
    })
    .bind(("0.0.0.0", port))?"""

import re
pattern = re.compile(r"App::new\(\).*?\.run\(\)\?", re.DOTALL)
updated_content = pattern.sub(new_app_block + "\n    .run()", content)

with open("src/main.rs", "w") as f:
    f.write(updated_content)
'

echo "✅ main.rs cleaned up. Restarting for final compile..."
sudo systemctl restart jeebs
sudo journalctl -u jeebs -f
