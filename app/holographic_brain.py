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

    def query(self, text: str, top_k: int = 1, use_priority: bool = True, use_context: bool = False, 
              conv_context: list = None):
        """Query memories with similarity scoring, optional priority weighting, and context awareness.
        
        Args:
            text: Query text
            top_k: Number of results to return
            use_priority: Apply priority and access count weighting
            use_context: Weight results higher if from same conversation
            conv_context: Optional list of dicts with 'key_text' and 'response_text' for context
        """
        self._ensure_table()
        
        # For context-aware queries, blend the query with context
        query_text = text
        if use_context and conv_context:
            # Add recent conversation context to the query for better understanding
            context_phrases = [msg.get('content', '')[:50] for msg in conv_context[-3:] if msg.get('content')]
            if context_phrases:
                query_text = text + " " + " ".join(context_phrases)
        
        probe = self.encode(query_text)
        conn = get_db()
        cur = conn.cursor()
        cur.execute(f"SELECT id, conversation_id, key_text, response_text, vector_json, priority, access_count, created_at FROM {self.table_name}")
        rows = cur.fetchall()
        conn.close()

        results = []
        current_conv_id = conv_context[0].get('conversation_id') if conv_context and isinstance(conv_context, list) and len(conv_context) > 0 else None
        
        for r in rows:
            try:
                vec = np.array(json.loads(r['vector_json']), dtype=float)
                sim = float(np.dot(probe, vec) / ((np.linalg.norm(probe) * np.linalg.norm(vec)) + 1e-9))
                
                # Apply priority weighting if enabled
                if use_priority:
                    priority = r['priority'] if r['priority'] else 1
                    # Boost score if this is taught knowledge or frequently accessed
                    sim = sim * (1 + (priority - 1) * 0.2 + min(r['access_count'] * 0.05, 0.3))
                
                # Context weighting: boost memories from same conversation
                if use_context and current_conv_id and r['conversation_id'] == current_conv_id:
                    sim = sim * 1.3  # 30% boost for same-conversation memories
                
                if sim >= self.similarity_threshold:
                    results.append((sim, r['response_text'], r['id'], r['conversation_id']))
            except Exception:
                continue

        results.sort(key=lambda x: x[0], reverse=True)
        
        # Update access count for top retrieved memory
        if results and len(results) > 0:
            self._update_access_count(results[0][2])
        
        return [(sim, resp) for sim, resp, _, _ in results[:top_k]]

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

    def extract_concepts(self, text: str) -> list:
        """Extract important concepts and entities from text for better understanding.
        
        Returns list of significant words/concepts for semantic analysis.
        """
        # Remove common stop words
        stop_words = {
            'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for',
            'of', 'with', 'by', 'from', 'is', 'are', 'was', 'were', 'be', 'been',
            'being', 'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would',
            'could', 'should', 'may', 'might', 'can', 'that', 'this', 'these',
            'those', 'i', 'you', 'he', 'she', 'it', 'we', 'they', 'as', 'if',
            'how', 'what', 'when', 'where', 'why', 'which', 'who', 'whom'
        }
        
        words = text.lower().split()
        # Keep words longer than 3 chars and not in stop words
        concepts = [w.strip('.,!?;:') for w in words 
                   if len(w.strip('.,!?;:')) > 3 and w.lower().strip('.,!?;:') not in stop_words]
        
        # Return unique concepts maintaining order
        seen = set()
        unique_concepts = []
        for c in concepts:
            if c not in seen:
                unique_concepts.append(c)
                seen.add(c)
        
        return unique_concepts

    def get_conversation_context(self, conv_id: int) -> dict:
        """Get learning context about a conversation (topics, themes, style)"""
        self._ensure_table()
        conn = get_db()
        cur = conn.cursor()
        
        # Get all memories from this conversation
        cur.execute(f"""
            SELECT key_text, response_text, priority, access_count, category
            FROM {self.table_name}
            WHERE conversation_id = ?
            ORDER BY lastused_at DESC
            LIMIT 20
        """, (conv_id,))
        
        memories = [dict(row) for row in cur.fetchall()]
        conn.close()
        
        if not memories:
            return {'conv_id': conv_id, 'topics': [], 'style': 'neutral', 'summary': None}
        
        # Extract all concepts to identify topics
        all_concepts = []
        for mem in memories:
            all_concepts.extend(self.extract_concepts(mem.get('key_text', '') + ' ' + mem.get('response_text', '')))
        
        # Count concept frequency to identify main topics
        from collections import Counter
        concept_freq = Counter(all_concepts)
        top_topics = [word for word, _ in concept_freq.most_common(5)]
        
        # Determine conversation style based on priority/access patterns
        avg_priority = sum(m.get('priority', 1) for m in memories) / len(memories) if memories else 1
        avg_access = sum(m.get('access_count', 0) for m in memories) / len(memories) if memories else 0
        
        # Classify style
        if avg_priority >= 2.5:
            style = 'formal_detailed'
        elif avg_access >= 10:
            style = 'frequently_referenced'
        else:
            style = 'conversational'
        
        return {
            'conv_id': conv_id,
            'topics': top_topics,
            'style': style,
            'memory_count': len(memories),
            'avg_priority': round(avg_priority, 2),
            'avg_access': round(avg_access, 2),
            'categories': list(set(m.get('category', 'general') for m in memories if m.get('category')))
        }


# Singleton instance used by the Flask app
brain = HolographicBrain()
