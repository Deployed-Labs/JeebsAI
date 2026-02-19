import os

# Let's look for common files where User might live
paths_to_check = ["src/admin/mod.rs", "src/db/mod.rs", "src/schema.rs", "src/admin/user.rs"]
found_path = "crate::User" # Default fallback

for p in paths_to_check:
    if os.path.exists(f"/root/JeebsAI/{p}"):
        with open(f"/root/JeebsAI/{p}", "r") as f:
            if "pub struct User" in f.read():
                # Convert file path to Rust module path
                found_path = "crate::" + p.replace("src/", "").replace(".rs", "").replace("/mod", "").replace("/", "::") + "::User"
                break

auth_path = "/root/JeebsAI/src/auth/mod.rs"
with open(auth_path, "r") as f:
    lines = f.readlines()

with open(auth_path, "w") as f:
    for line in lines:
        if "use crate::models::User;" in line or "use crate::admin::user::User;" in line:
            f.write(f"use {found_path};\n")
        else:
            f.write(line)

print(f"✅ Re-pointed User import to {found_path}")
