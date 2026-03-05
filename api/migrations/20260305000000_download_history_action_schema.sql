PRAGMA foreign_keys = OFF;

CREATE TABLE download_history_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,

    action_id TEXT NOT NULL,
    backend_id TEXT NOT NULL,
    queue_item_id TEXT NOT NULL,
    source TEXT NOT NULL,

    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    release_name TEXT,
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

    started_at TEXT NOT NULL,
    downloaded_at TEXT,
    imported_at TEXT,
    ended_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO download_history_new (
    id,
    user_id,
    action_id,
    backend_id,
    queue_item_id,
    source,
    title,
    artist,
    release_name,
    item_label,
    size_bytes,
    target_folder,
    download_status,
    import_status,
    error_message,
    needs_manual_action,
    downloadable_item_json,
    tracks_json,
    started_at,
    downloaded_at,
    imported_at,
    ended_at,
    created_at,
    updated_at
)
SELECT
    id,
    user_id,
    batch_id AS action_id,
    backend_id,
    queue_item_id,
    source,
    title,
    artist,
    album AS release_name,
    item_label,
    size_bytes,
    target_folder,
    download_status,
    import_status,
    error_message,
    needs_manual_action,
    downloadable_item_json,
    tracks_json,
    queued_at AS started_at,
    downloaded_at,
    imported_at,
    CASE
        WHEN imported_at IS NOT NULL THEN imported_at
        WHEN downloaded_at IS NOT NULL
            AND download_status IN ('completed', 'failed', 'cancelled', 'timeout') THEN downloaded_at
        WHEN download_status IN ('failed', 'cancelled', 'timeout') THEN updated_at
        WHEN import_status IN ('failed', 'timeout') THEN updated_at
        ELSE NULL
    END AS ended_at,
    created_at,
    updated_at
FROM download_history;

DROP TABLE download_history;
ALTER TABLE download_history_new RENAME TO download_history;

CREATE INDEX IF NOT EXISTS idx_download_history_user_started
    ON download_history(user_id, started_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_download_history_user_action
    ON download_history(user_id, action_id);

CREATE INDEX IF NOT EXISTS idx_download_history_user_download_status
    ON download_history(user_id, download_status);

CREATE INDEX IF NOT EXISTS idx_download_history_user_import_status
    ON download_history(user_id, import_status);

PRAGMA foreign_keys = ON;
