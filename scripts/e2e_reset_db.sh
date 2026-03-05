#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIGRATIONS_DIR="$ROOT_DIR/api/migrations"
DB_PATH="${1:-/opt/soulbeet/data/soulbeet.db}"

if [[ -f "$DB_PATH" ]]; then
  rm -f "$DB_PATH"
fi

mkdir -p "$(dirname "$DB_PATH")"

while IFS= read -r migration; do
  sqlite3 "$DB_PATH" <"$migration"
done < <(ls -1 "$MIGRATIONS_DIR"/*.sql | sort)

echo "Reset DB and applied migrations: $DB_PATH"
