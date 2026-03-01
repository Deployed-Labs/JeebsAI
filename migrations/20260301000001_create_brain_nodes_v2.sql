-- BrainNode table for storing extracted facts (not raw source code)
CREATE TABLE IF NOT EXISTS brain_nodes_v2 (
    id TEXT PRIMARY KEY NOT NULL,
    fact TEXT NOT NULL,
    source_url TEXT,
    vector_id TEXT,
    category TEXT NOT NULL DEFAULT 'general',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_brain_nodes_v2_category ON brain_nodes_v2(category);
CREATE INDEX IF NOT EXISTS idx_brain_nodes_v2_vector_id ON brain_nodes_v2(vector_id);
