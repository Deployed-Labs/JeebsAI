path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# Swap the broken import for a more likely one or a generic one
content = content.replace("use crate::admin::user::User;", "use crate::models::User;")
# If it's not in models, it might be in the schema or admin mod
# We will also add a fallback if the first one fails

with open(path, "w") as f:
    f.write(content)
