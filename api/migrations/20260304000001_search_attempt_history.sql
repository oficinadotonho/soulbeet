CREATE TABLE IF NOT EXISTS search_attempt_history (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,

    attempt_type TEXT NOT NULL CHECK (
        attempt_type IN ('metadata_album', 'metadata_track', 'source_download')
    ),
    provider_id TEXT,
    backend_id TEXT,
    search_id TEXT,

    query_text TEXT,
    artist_text TEXT,

    status TEXT NOT NULL CHECK (
        status IN ('in_progress', 'completed', 'timed_out', 'failed', 'no_results')
    ),
    result_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,

    started_at TEXT NOT NULL,
    ended_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_search_attempt_history_user_started
    ON search_attempt_history(user_id, started_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_search_attempt_history_user_type_status
    ON search_attempt_history(user_id, attempt_type, status);

CREATE INDEX IF NOT EXISTS idx_search_attempt_history_user_search_id
    ON search_attempt_history(user_id, search_id);
