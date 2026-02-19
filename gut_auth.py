import re

path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    lines = f.readlines()

new_lines = []
skip_block = False

for line in lines:
    # If we hit an Argon2 or PasswordHash block, we need to simplify it
    if "Argon2::default()" in line or "PasswordHash::new" in line:
        # Instead of hashing, we just use the raw password string
        # This handles the 'new_password' hashing logic
        if "hash_password" in line:
            new_lines.append('                let hashed_password = req.new_password.clone();\n')
        # This handles the verification logic in other parts of the file
        elif "verify_password" in line or "verify" in line:
            new_lines.append('            if req.password == stored_hash {\n')
        continue
    
    # Cleaning up the 'match' arms that usually follow the Argon2 calls
    if "Ok(parsed_hash)" in line or "Err(e)" in line or "PasswordHash" in line:
        continue
        
    new_lines.append(line)

with open(path, "w") as f:
    f.writelines(new_lines)
