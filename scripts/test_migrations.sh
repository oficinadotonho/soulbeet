#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIGRATIONS_DIR="$ROOT_DIR/api/migrations"
FIXTURES_DIR="$ROOT_DIR/tests/fixtures/migrations"

DB_FRESH="/tmp/soulbeet_mig_fresh.db"
DB_PRE="/tmp/soulbeet_mig_pre_history.db"
DB_OLD="/tmp/soulbeet_mig_old_history.db"

cleanup() {
  rm -f "$DB_FRESH" "$DB_PRE" "$DB_OLD"
}
trap cleanup EXIT

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1"
    exit 1
  }
}

apply_sql_file() {
  local db="$1"
  local file="$2"
  sqlite3 "$db" <"$file"
}

assert_eq() {
  local expected="$1"
  local got="$2"
  local message="$3"
  if [[ "$expected" != "$got" ]]; then
    echo "Assertion failed: $message"
    echo "Expected: $expected"
    echo "Got: $got"
    exit 1
  fi
}

assert_download_history_action_shape() {
  local db="$1"
  local cols
  cols="$(sqlite3 "$db" "SELECT group_concat(name, ',') FROM pragma_table_info('download_history');")"
  [[ "$cols" == *"action_id"* ]] || { echo "download_history missing action_id"; exit 1; }
  [[ "$cols" == *"release_name"* ]] || { echo "download_history missing release_name"; exit 1; }
  [[ "$cols" == *"started_at"* ]] || { echo "download_history missing started_at"; exit 1; }
  [[ "$cols" == *"ended_at"* ]] || { echo "download_history missing ended_at"; exit 1; }
  [[ "$cols" != *"batch_id"* ]] || { echo "download_history still has batch_id"; exit 1; }
}

assert_search_attempt_table_exists() {
  local db="$1"
  local exists
  exists="$(sqlite3 "$db" "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='search_attempt_history';")"
  assert_eq "1" "$exists" "search_attempt_history table should exist"
}

assert_expected_migration_files() {
  local expected
  local got
  expected="$(tr '\n' ' ' <"$FIXTURES_DIR/expected_versions.txt" | xargs)"
  got="$(ls -1 "$MIGRATIONS_DIR"/*.sql | xargs -n1 basename | sed 's/_.*$//' | tr '\n' ' ' | xargs)"
  assert_eq "$expected" "$got" "migration file version set changed"
}

run_fresh_db_test() {
  echo "[1/3] Fresh DB migration chain"
  rm -f "$DB_FRESH"

  while IFS= read -r migration; do
    apply_sql_file "$DB_FRESH" "$migration"
  done < <(ls -1 "$MIGRATIONS_DIR"/*.sql | sort)

  assert_download_history_action_shape "$DB_FRESH"
  assert_search_attempt_table_exists "$DB_FRESH"
}

run_pre_history_upgrade_test() {
  echo "[2/3] Upgrade from pre-history fixture"
  rm -f "$DB_PRE"
  apply_sql_file "$DB_PRE" "$FIXTURES_DIR/legacy_pre_history.sql"

  apply_sql_file "$DB_PRE" "$MIGRATIONS_DIR/20260304000000_download_history.sql"
  apply_sql_file "$DB_PRE" "$MIGRATIONS_DIR/20260304000001_search_attempt_history.sql"
  if [[ -f "$MIGRATIONS_DIR/20260305000000_download_history_action_schema.sql" ]]; then
    apply_sql_file "$DB_PRE" "$MIGRATIONS_DIR/20260305000000_download_history_action_schema.sql"
  fi

  assert_download_history_action_shape "$DB_PRE"
  assert_search_attempt_table_exists "$DB_PRE"
}

run_old_history_upgrade_test() {
  echo "[3/3] Upgrade from old download_history schema fixture"
  rm -f "$DB_OLD"
  apply_sql_file "$DB_OLD" "$FIXTURES_DIR/legacy_old_history_schema.sql"

  if [[ -f "$MIGRATIONS_DIR/20260305000000_download_history_action_schema.sql" ]]; then
    apply_sql_file "$DB_OLD" "$MIGRATIONS_DIR/20260305000000_download_history_action_schema.sql"
    assert_download_history_action_shape "$DB_OLD"

    local migrated
    migrated="$(sqlite3 "$DB_OLD" "SELECT action_id || '|' || release_name || '|' || started_at || '|' || COALESCE(ended_at, '') FROM download_history WHERE id='h1';")"
    assert_eq "batch-1|Album X|2026-03-01T12:00:00Z|2026-03-01T12:07:00Z" "$migrated" "legacy row should be transformed correctly"
  else
    # On branches before the action-schema migration exists, verify legacy schema is still valid.
    local has_batch
    has_batch="$(sqlite3 "$DB_OLD" "SELECT count(*) FROM pragma_table_info('download_history') WHERE name='batch_id';")"
    assert_eq "1" "$has_batch" "legacy fixture should still have batch_id before action-schema migration"
  fi
}

main() {
  require_cmd sqlite3
  assert_expected_migration_files
  run_fresh_db_test
  run_pre_history_upgrade_test
  run_old_history_upgrade_test
  echo "Migration tests passed."
}

main "$@"
