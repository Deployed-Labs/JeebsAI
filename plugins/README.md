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
