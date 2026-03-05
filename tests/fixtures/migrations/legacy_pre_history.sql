PRAGMA foreign_keys = ON;

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE app_config (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE user_settings (
    user_id TEXT PRIMARY KEY NOT NULL,
    default_metadata_provider TEXT NOT NULL DEFAULT 'musicbrainz',
    last_search_type TEXT NOT NULL DEFAULT 'track',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO users (id, username, password_hash)
VALUES ('u1', 'tonho', 'fake-hash');
