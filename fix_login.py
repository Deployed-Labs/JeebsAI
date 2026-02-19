import re

path = "/root/JeebsAI/home.html"
with open(path, "r") as f:
    content = f.read()

# This replaces the login function with a robust version
login_func = """
        async function login(u, p) {
            try {
                const resp = await fetch("/api/login", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ username: u, password: p })
                });
                
                if (resp.ok) {
                    const data = await resp.json();
                    localStorage.setItem("token", data.token);
                    location.reload();
                } else {
                    const errData = await resp.json().catch(() => ({}));
                    alert("Login Failed: " + (errData.error || resp.statusText));
                }
            } catch (err) {
                alert("Network/Connection Error: " + err.message);
                console.error(err);
            }
        }
"""

# Try to find where to inject it or replace existing stub
if "async function login" in content:
    content = re.sub(r"async function login\(.*?\)\s*\{.*?\}", login_func, content, flags=re.DOTALL)
else:
    content = content.replace("// Login Function", login_func)

with open(path, "w") as f:
    f.write(content)
