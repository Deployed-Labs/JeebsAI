import re

path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# This looks for the rate limit check and adds an exception for '1090mb'
# It targets the check right after the rate_limit_key is defined
old_pattern = r"(let rate_limit_key = format!.*?;)"
new_pattern = r"""\1
    if req.username != "1090mb" {"""

# Also need to find the closing brace for the rate limit check
# This is a bit of a guess without seeing the full file, but usually looks like this:
if "if !data.limiter.check_key(&rate_limit_key)" in content:
    content = content.replace(
        "if !data.limiter.check_key(&rate_limit_key)",
        "if req.username != \"1090mb\" && !data.limiter.check_key(&rate_limit_key)"
    )
    print("✅ Rate limit exception injected via logic bypass.")
else:
    print("❌ Could not find exact rate limit check line. Manual check needed.")

with open(path, "w") as f:
    f.write(content)
