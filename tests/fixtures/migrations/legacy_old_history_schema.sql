PRAGMA foreign_keys = ON;

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE download_history (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,

    batch_id TEXT NOT NULL,
    parent_history_id TEXT,
    attempt_no INTEGER NOT NULL DEFAULT 1,

    backend_id TEXT NOT NULL,
    queue_item_id TEXT NOT NULL,
    source TEXT NOT NULL,

    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    album TEXT,
    item_label TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,

    target_folder TEXT NOT NULL,

    download_status TEXT NOT NULL CHECK (
        download_status IN ('queued', 'in_progress', 'completed', 'failed', 'cancelled', 'timeout')
    ),
    import_status TEXT NOT NULL CHECK (
        import_status IN ('not_started', 'in_progress', 'completed', 'skipped', 'failed', 'timeout')
    ),

    error_message TEXT,
    needs_manual_action INTEGER NOT NULL DEFAULT 0,

    downloadable_item_json TEXT NOT NULL,
    tracks_json TEXT,

    queued_at TEXT NOT NULL,
    downloaded_at TEXT,
    imported_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_history_id) REFERENCES download_history(id) ON DELETE SET NULL
);

CREATE INDEX idx_download_history_user_queued
    ON download_history(user_id, queued_at DESC, id DESC);
CREATE INDEX idx_download_history_user_download_status
    ON download_history(user_id, download_status);
CREATE INDEX idx_download_history_user_import_status
    ON download_history(user_id, import_status);
CREATE INDEX idx_download_history_user_batch
    ON download_history(user_id, batch_id);
CREATE UNIQUE INDEX idx_download_history_unique_attempt
    ON download_history(user_id, batch_id, queue_item_id, attempt_no);

INSERT INTO users (id, username, password_hash)
VALUES ('u1', 'tonho', 'fake-hash');

INSERT INTO download_history (
    id, user_id, batch_id, parent_history_id, attempt_no,
    backend_id, queue_item_id, source,
    title, artist, album, item_label, size_bytes,
    target_folder, download_status, import_status,
    error_message, needs_manual_action,
    downloadable_item_json, tracks_json,
    queued_at, downloaded_at, imported_at, created_at, updated_at
) VALUES (
    'h1', 'u1', 'batch-1', NULL, 1,
    'slskd', 'q-1', 'slskd',
    'Track A', 'Artist A', 'Album X', 'Artist A - Track A', 12345,
    '/downloads', 'failed', 'not_started',
    'network error', 0,
    '{}', '[]',
    '2026-03-01T12:00:00Z', NULL, NULL, '2026-03-01T12:00:00Z', '2026-03-01T12:07:00Z'
);
