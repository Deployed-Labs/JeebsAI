CREATE TABLE IF NOT EXISTS knowledge_triples (
    subject TEXT NOT NULL,
    predicate TEXT NOT NULL,
    object TEXT NOT NULL,
    confidence REAL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (subject, predicate, object)
);
CREATE INDEX IF NOT EXISTS idx_triples_subject ON knowledge_triples(subject);
CREATE INDEX IF NOT EXISTS idx_triples_object ON knowledge_triples(object);
