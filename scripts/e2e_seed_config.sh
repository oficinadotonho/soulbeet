#!/usr/bin/env bash
set -euo pipefail

DB_PATH="${1:-/opt/soulbeet/data/soulbeet.db}"
SLSKD_URL="${SLSKD_URL:-}"
SLSKD_API_KEY="${SLSKD_API_KEY:-}"

if [[ ! -f "$DB_PATH" ]]; then
  echo "Database not found: $DB_PATH"
  exit 1
fi

if [[ -n "$SLSKD_URL" ]]; then
  sqlite3 "$DB_PATH" \
    "INSERT INTO app_config (key, value) VALUES ('slskd_url', '$SLSKD_URL') ON CONFLICT(key) DO UPDATE SET value = excluded.value;"
fi

if [[ -n "$SLSKD_API_KEY" ]]; then
  sqlite3 "$DB_PATH" \
    "INSERT INTO app_config (key, value) VALUES ('slskd_api_key', '$SLSKD_API_KEY') ON CONFLICT(key) DO UPDATE SET value = excluded.value;"
fi

echo "Seeded app_config values in: $DB_PATH"
