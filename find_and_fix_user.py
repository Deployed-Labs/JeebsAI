import os
import re

def find_user_path():
    src_dir = "/root/JeebsAI/src"
    for root, dirs, files in os.walk(src_dir):
        for file in files:
            if file.endswith(".rs"):
                full_path = os.path.join(root, file)
                with open(full_path, "r") as f:
                    if "pub struct User" in f.read():
                        # Convert /root/JeebsAI/src/admin/mod.rs -> crate::admin::User
                        rel_path = os.path.relpath(full_path, src_dir)
                        parts = rel_path.replace(".rs", "").split(os.sep)
                        if parts[-1] == "mod":
                            parts.pop()
                        return "crate::" + "::".join(parts) + "::User"
    return None

correct_path = find_user_path()
if correct_path:
    auth_file = "/root/JeebsAI/src/auth/mod.rs"
    with open(auth_file, "r") as f:
        content = f.read()
    
    # Replace the broken import with the one we just found
    new_content = re.sub(r'use crate::User;', f'use {correct_path};', content)
    with open(auth_file, "w") as f:
        f.write(new_content)
    print(f"✅ Success! Found User at: {correct_path}")
else:
    print("❌ Could not find 'pub struct User' anywhere in src/")
