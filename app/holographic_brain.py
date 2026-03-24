import json
import hashlib
from datetime import datetime
import numpy as np
from .models import get_db


class HolographicBrain:
    """A lightweight HRR-like holographic brain for JeebsAI.

    - Encodes text into fixed-size vectors using token-hashed random vectors.
    - Stores memories in the app sqlite DB as JSON-serialized vectors.
    - Retrieves best responses by cosine similarity.
    """

    def __init__(self, dim=512, table_name='holographic_memories'):
        self.dim = dim
        self.table_name = table_name

    def _token_vector(self, token: str) -> np.ndarray:
        # Deterministic pseudorandom vector per token via hashed seed
        h = int(hashlib.sha1(token.encode('utf-8')).hexdigest()[:8], 16)
        rng = np.random.RandomState(h)
        v = rng.normal(size=self.dim)
        v /= (np.linalg.norm(v) + 1e-9)
        return v

    def encode(self, text: str) -> np.ndarray:
        tokens = [t for t in text.lower().split() if t]
        if not tokens:
            return np.zeros(self.dim, dtype=float)
        vecs = [self._token_vector(t) for t in tokens]
        vec = np.sum(vecs, axis=0)
        vec /= (np.linalg.norm(vec) + 1e-9)
        return vec

    def _ensure_table(self):
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"""
        CREATE TABLE IF NOT EXISTS {self.table_name} (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER,
            key_text TEXT,
            response_text TEXT,
            vector_json TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        """)
        conn.commit()
        conn.close()

    def save_memory(self, conversation_id: int, key_text: str, response_text: str):
        self._ensure_table()
        vec = self.encode(key_text).tolist()
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"INSERT INTO {self.table_name} (conversation_id, key_text, response_text, vector_json) VALUES (?,?,?,?)",
                    (conversation_id, key_text, response_text, json.dumps(vec)))
        conn.commit()
        conn.close()

    def query(self, text: str, top_k: int = 1):
        self._ensure_table()
        probe = self.encode(text)
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"SELECT id, conversation_id, key_text, response_text, vector_json FROM {self.table_name}")
        rows = cur.fetchall()
        conn.close()

        results = []
        for r in rows:
            try:
                vec = np.array(json.loads(r['vector_json']), dtype=float)
                sim = float(np.dot(probe, vec) / ((np.linalg.norm(probe) * np.linalg.norm(vec)) + 1e-9))
                results.append((sim, r['response_text']))
            except Exception:
                continue

        results.sort(key=lambda x: x[0], reverse=True)
        return results[:top_k]


# Singleton instance used by the Flask app
brain = HolographicBrain()
