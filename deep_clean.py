import re

files_to_clean = ["/root/JeebsAI/src/auth/mod.rs", "/root/JeebsAI/src/security.rs", "/root/JeebsAI/src/lib.rs"]

for file_path in files_to_clean:
    try:
        with open(file_path, "r") as f:
            content = f.read()
        
        # Remove Argon2 and password-hash related imports
        content = re.sub(r'use argon2::.*?;', '', content)
        content = re.sub(r'use password_hash::.*?;', '', content)
        content = re.sub(r'use std::env;', '', content) # Cleaning the warning you saw
        
        # If verify_password exists as a function definition, make it plaintext
        if "fn verify_password" in content:
            content = re.sub(
                r'fn verify_password\(.*?\)\s*->\s*bool\s*\{.*?\}',
                'fn verify_password(password: &str, hash: &str, _pepper: &str) -> bool { password == hash }',
                content, flags=re.DOTALL
            )
            
        with open(file_path, "w") as f:
            f.write(content)
    except FileNotFoundError:
        continue

print("🧹 Imports scrubbed and logic simplified.")
