CREATE TABLE IF NOT EXISTS runs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pipeline    TEXT    NOT NULL,
    branch      TEXT    NOT NULL,
    commit_sha  TEXT    NOT NULL,
    status      TEXT    NOT NULL DEFAULT 'pending',
    started_at  TEXT    NOT NULL,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS audit_log (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id      INTEGER NOT NULL,
    event       TEXT    NOT NULL,
    detail      TEXT,
    occurred_at TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS step_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id      INTEGER NOT NULL,
    stage       TEXT    NOT NULL,
    step        TEXT    NOT NULL,
    output      TEXT    NOT NULL,
    exit_code   INTEGER NOT NULL,
    logged_at   TEXT    NOT NULL
);
