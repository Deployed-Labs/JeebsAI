import json
import hashlib
from datetime import datetime
import numpy as np
from .models import get_db


class HolographicBrain:
    """An enhanced HRR-like holographic brain for JeebsAI with teaching capabilities.

    - Encodes text into fixed-size vectors using token-hashed random vectors.
    - Stores memories in the app sqlite DB as JSON-serialized vectors.
    - Retrieves best responses by cosine similarity.
    - Supports explicit teaching and priority-weighted memory retrieval.
    """

    def __init__(self, dim=512, table_name='holographic_memories'):
        self.dim = dim
        self.table_name = table_name
        self.similarity_threshold = 0.55

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
            priority INTEGER DEFAULT 1,
            access_count INTEGER DEFAULT 0,
            is_taught INTEGER DEFAULT 0,
            category TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            lastused_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        """)
        conn.commit()
        conn.close()

    def save_memory(self, conversation_id: int, key_text: str, response_text: str, priority: int = 1, 
                   is_taught: bool = False, category: str = None):
        """Save a memory with optional priority and teaching flag"""
        self._ensure_table()
        vec = self.encode(key_text).tolist()
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"""INSERT INTO {self.table_name} 
                       (conversation_id, key_text, response_text, vector_json, priority, is_taught, category) 
                       VALUES (?,?,?,?,?,?,?)""",
                    (conversation_id, key_text, response_text, json.dumps(vec), priority, int(is_taught), category))
        conn.commit()
        conn.close()

    def teach(self, key_text: str, response_text: str, conversation_id: int = None, category: str = None):
        """Explicitly teach JeebsAI a fact or rule with high priority"""
        return self.save_memory(conversation_id or 0, key_text, response_text, 
                              priority=3, is_taught=True, category=category)

    def query(self, text: str, top_k: int = 1, use_priority: bool = True):
        """Query memories with similarity scoring and optional priority weighting"""
        self._ensure_table()
        probe = self.encode(text)
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"SELECT id, conversation_id, key_text, response_text, vector_json, priority, access_count FROM {self.table_name}")
        rows = cur.fetchall()
        conn.close()

        results = []
        for r in rows:
            try:
                vec = np.array(json.loads(r['vector_json']), dtype=float)
                sim = float(np.dot(probe, vec) / ((np.linalg.norm(probe) * np.linalg.norm(vec)) + 1e-9))
                
                # Apply priority weighting if enabled
                if use_priority:
                    priority = r['priority'] if r['priority'] else 1
                    # Boost score if this is taught knowledge or frequently accessed
                    sim = sim * (1 + (priority - 1) * 0.2 + min(r['access_count'] * 0.05, 0.3))
                
                if sim >= self.similarity_threshold:
                    results.append((sim, r['response_text'], r['id']))
            except Exception:
                continue

        results.sort(key=lambda x: x[0], reverse=True)
        
        # Update access count for retrieved memories
        if results and len(results) > 0:
            self._update_access_count(results[0][2])
        
        return [(sim, resp) for sim, resp, _ in results[:top_k]]

    def _update_access_count(self, memory_id: int):
        """Update access count when a memory is retrieved"""
        try:
            conn = get_db()
            cur = conn.cursor()
            cur.execute(f"UPDATE {self.table_name} SET access_count = access_count + 1, lastused_at = CURRENT_TIMESTAMP WHERE id = ?", 
                       (memory_id,))
            conn.commit()
            conn.close()
        except:
            pass

    def list_memories(self, conversation_id: int = None, limit: int = 50):
        """List all memories, optionally filtered by conversation"""
        self._ensure_table()
        conn = get_db()
        cur = conn.cursor()
        
        if conversation_id:
            cur.execute(f"""SELECT id, key_text, response_text, priority, access_count, is_taught, category, created_at 
                           FROM {self.table_name} 
                           WHERE conversation_id = ? 
                           ORDER BY lastused_at DESC 
                           LIMIT ?""", (conversation_id, limit))
        else:
            cur.execute(f"""SELECT id, key_text, response_text, priority, access_count, is_taught, category, created_at 
                           FROM {self.table_name} 
                           ORDER BY lastused_at DESC 
                           LIMIT ?""", (limit,))
        
        memories = [dict(row) for row in cur.fetchall()]
        conn.close()
        return memories

    def delete_memory(self, memory_id: int):
        """Delete a specific memory"""
        try:
            conn = get_db()
            cur = conn.cursor()
            cur.execute(f"DELETE FROM {self.table_name} WHERE id = ?", (memory_id,))
            conn.commit()
            conn.close()
            return True
        except:
            return False


# Singleton instance used by the Flask app
brain = HolographicBrain()
