CREATE TABLE IF NOT EXISTS pipeline_hashes (
    pipeline   TEXT PRIMARY KEY,
    hash       TEXT NOT NULL,
    saved_at   TEXT NOT NULL
);
