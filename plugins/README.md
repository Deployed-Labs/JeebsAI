Plugin contract — CLI / language-agnostic

Place each plugin in its own directory under `plugins/`.

Contract (simple):
- Input: plugin reads JSON from stdin: { "input": "..." }
- Output: plugin writes JSON to stdout: { "response": "..." }
- If plugin writes plain text (no JSON), that text will be returned as-is.

Runners supported by the loader:
- `run` (executable file inside plugin dir)
- `run.py` (invoked with `python3`)
- `run.js` or `index.js` (invoked with `node`)

Examples are included (`python-echo`, `node-hello`).

## Installing a plugin

1. Copy or clone the plugin directory into `plugins/`:

```bash
cp -r my-plugin /var/lib/jeebs/plugins/
# or clone a packaged plugin:
git clone https://github.com/example/jeebs-plugin-foo plugins/foo
```

2. Make the runner executable (if needed):

```bash
chmod +x plugins/foo/run
```

3. Restart JeebsAI so the plugin loader picks it up:

```bash
sudo systemctl restart jeebs
# or when running via Docker:
docker compose restart jeebs
```

## Writing a plugin

Create a directory under `plugins/` with one of the supported runner files:

**Python (`run.py`)**:
```python
#!/usr/bin/env python3
import sys, json

data = json.loads(sys.stdin.read())
print(json.dumps({"response": f"Got: {data['input']}"}))
```

**Node.js (`index.js`)**:
```js
#!/usr/bin/env node
const data = JSON.parse(require('fs').readFileSync(0, 'utf-8'));
console.log(JSON.stringify({ response: `Got: ${data.input}` }));
```

**Executable (`run`)** — any language compiled to a binary or a shell script:
```bash
#!/usr/bin/env bash
INPUT=$(cat | python3 -c "import sys,json; print(json.load(sys.stdin)['input'])")
echo "{\"response\": \"Got: $INPUT\"}"
```

## Removing a plugin

```bash
rm -rf plugins/my-plugin
sudo systemctl restart jeebs
```
