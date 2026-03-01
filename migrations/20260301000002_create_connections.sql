-- Connections between BrainNodes with relationship strength
CREATE TABLE IF NOT EXISTS connections (
    id TEXT PRIMARY KEY NOT NULL,
    from_node_id TEXT NOT NULL,
    to_node_id TEXT NOT NULL,
    strength REAL NOT NULL DEFAULT 1.0,
    FOREIGN KEY (from_node_id) REFERENCES brain_nodes_v2(id),
    FOREIGN KEY (to_node_id) REFERENCES brain_nodes_v2(id)
);

CREATE INDEX IF NOT EXISTS idx_connections_from ON connections(from_node_id);
CREATE INDEX IF NOT EXISTS idx_connections_to ON connections(to_node_id);
