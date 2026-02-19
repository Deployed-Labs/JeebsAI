path = "/root/JeebsAI/src/auth/mod.rs"
with open(path, "r") as f:
    content = f.read()

# Fix the empty .json() call that's causing the error
content = content.replace('.json(\n                    )', '.json(serde_json::json!({"info": "Rate limit disabled"}))')
content = content.replace('.json()', '.json(serde_json::json!({"info": "Rate limit disabled"}))')

with open(path, "w") as f:
    f.write(content)
