CREATE TABLE IF NOT EXISTS download_history (
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

CREATE INDEX IF NOT EXISTS idx_download_history_user_started
    ON download_history(user_id, started_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_download_history_user_action
    ON download_history(user_id, action_id);

CREATE INDEX IF NOT EXISTS idx_download_history_user_download_status
    ON download_history(user_id, download_status);

CREATE INDEX IF NOT EXISTS idx_download_history_user_import_status
    ON download_history(user_id, import_status);
