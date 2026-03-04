# Download History Implementation Plan (Refactored)

## 1. Problem and Goals

Soulbeet currently streams download/import progress over websocket but does not persist historical results.

This branch should add durable per-user history with:
- Paginated browsing
- Search and filtering
- Explicit download/import outcomes
- Retry support grounded in persisted request context

## 2. Scope

### In scope (MVP)
- Persist history for each queued downloadable item
- Group entries in UI by day and by batch/album context
- Filter by status/date/search
- Retry failed items with predictable semantics
- Keep strict user isolation (`user_id` scoped queries)

### Out of scope (MVP)
- Cross-user/admin history views
- Full-text search engine (FTS)
- Automatic retention/archival jobs
- Bulk retry orchestration

## 3. Key Design Decisions

### D1. Persist item-level records; derive groups in UI/API
- Persist one history row per downloadable item attempt.
- Include `batch_id` to group items that were queued together.
- UI can render album-like cards by grouping rows by `(batch_id, album/title/artist)`.

Why:
- Matches current pipeline (queue and monitor are item-driven)
- Preserves detail for partial failures
- Avoids lossy aggregate-only rows

### D2. Retry creates a new attempt row (immutable history)
- Do not overwrite prior failed rows.
- On retry, create a new row linked via `parent_history_id` and incremented `attempt_no`.
- Keep old attempts for audit/debug visibility.
- MVP retry action re-runs both download and import together (`DownloadAndImport`).

Why:
- Prevents status ambiguity
- Supports clear timeline
- Simplifies debugging repeated failures

### D3. Use constrained status enums aligned to current runtime states
Current runtime states are in `lib/shared/src/download.rs` (`Queued`, `InProgress`, `Completed`, `Importing`, `Imported`, `ImportSkipped`, `Failed`, `Cancelled`).

Persist as two columns:
- `download_status`: `queued | in_progress | completed | failed | cancelled | timeout`
- `import_status`: `not_started | in_progress | completed | skipped | failed | timeout`

Mapping rules:
- `Queued` -> `download_status=queued`, `import_status=not_started`
- `InProgress` -> `download_status=in_progress`
- `Completed` -> `download_status=completed`
- `Importing` -> `import_status=in_progress`
- `Imported` -> `import_status=completed`
- `ImportSkipped` -> `import_status=skipped`
- `Failed(msg)` during download phase -> `download_status=failed`
- `Failed(msg)` during import phase -> `import_status=failed`
- `Cancelled` -> `download_status=cancelled`

### D4. API contract should be server-fn-friendly and explicit
Use struct-based parameters for server-fns instead of relying on path parameter complexity.

Recommended endpoints:
- `POST /api/history/query` -> paginated list with filters
- `POST /api/history/get` -> single item details
- `POST /api/history/retry` -> retry item
- `POST /api/history/delete` -> delete single item
- `POST /api/history/clear` -> clear all (optional, guarded)

### D5. Offset pagination for MVP with stable ordering
- Use `ORDER BY queued_at DESC, id DESC`.
- Keep offset pagination now; move to cursor only if large-history performance demands it.

## 4. Data Model

## 4.1 Migration

Create: `api/migrations/<timestamp>_download_history.sql`

```sql
CREATE TABLE IF NOT EXISTS download_history (
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
        download_status IN ('queued','in_progress','completed','failed','cancelled','timeout')
    ),
    import_status TEXT NOT NULL CHECK (
        import_status IN ('not_started','in_progress','completed','skipped','failed','timeout')
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

CREATE INDEX IF NOT EXISTS idx_download_history_user_queued
    ON download_history(user_id, queued_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_download_history_user_download_status
    ON download_history(user_id, download_status);

CREATE INDEX IF NOT EXISTS idx_download_history_user_import_status
    ON download_history(user_id, import_status);

CREATE INDEX IF NOT EXISTS idx_download_history_user_batch
    ON download_history(user_id, batch_id);
```

Notes:
- `downloadable_item_json` is required to make retry deterministic.
- `item_label` stores the runtime `DownloadProgress.item` string for correlation.
- Keep timestamps in UTC ISO-8601.

## 4.2 Shared types (`lib/shared/src`)

Add `history.rs` (or extend `download.rs`) with:
- `HistoryEntry`
- `HistoryQuery`
- `PaginatedHistory`
- `RetryHistoryRequest`
- `RetryMode` (`DownloadAndImport` for MVP; `ImportOnly` can be added later)

Use enums instead of free-form strings for statuses at Rust type level.

## 5. Write Path Integration

## 5.1 Queue phase (`api/src/server_fns/download/mod.rs`)
- Generate `batch_id` per queue request.
- For each queued result, insert a history row with `download_status=queued`.
- Persist `downloadable_item_json` from request item.
- Return runtime map `{queue_item_id -> history_id}` to monitoring pipeline.

## 5.2 Monitor phase (`api/src/server_fns/download/monitor.rs`)
- Update only on state transitions (avoid DB writes on every poll).
- On terminal unresolved timeout path, write explicit failed/timeout status.
- Preserve last known error string.

## 5.3 Import phase (`api/src/server_fns/download/import.rs`)
- On `Importing`, set `import_status=in_progress`.
- On importer result, set `completed | skipped | failed | timeout`.
- Set `needs_manual_action=1` for failed/timeout states.

## 6. Read API Design

## 6.1 Query endpoint
`POST /api/history/query` input:
- `search: Option<String>` (title/artist/album)
- `download_status: Option<DownloadHistoryStatus>`
- `import_status: Option<ImportHistoryStatus>`
- `date_from_utc: Option<String>`
- `date_to_utc: Option<String>`
- `page: i32` (default 1)
- `per_page: i32` (default 20, max 100)

Output:
- `entries`
- `total`
- `page`
- `per_page`
- `total_pages`

## 6.2 Single-item endpoint
`POST /api/history/get` input: `id`

Return full row with parsed payload fields needed for retry UI details.

## 6.3 Retry endpoint
`POST /api/history/retry` input:
- `history_id`
- `mode: RetryMode` (MVP value: `DownloadAndImport`)

Behavior:
- Validate ownership by `user_id`
- Validate retry eligibility (`failed/cancelled/timeout/skipped` depending on mode)
- Clone relevant fields into new attempt row
- Requeue backend call and wire new history row to monitor/import updates
- Cancelled items are retryable in MVP.

## 7. UI Plan

## 7.1 Route and navigation
- Add route `/history` in `web/src/main.rs`
- Add `HistoryPage` in `web/src/views/history.rs`
- Export view from `web/src/views/mod.rs`
- Add nav link in `web/src/main.rs` navbar block (shared `ui::Navbar` wrapper remains unchanged)

## 7.2 Components
- `HistoryPage` (state + fetch)
- `HistoryFilters`
- `HistoryDayGroup`
- `HistoryItemCard`
- `HistoryRetryDialog`

## 7.3 UX details
- Show combined badge summary per card:
  - Download: queued/in-progress/completed/failed
  - Import: not-started/in-progress/completed/skipped/failed
- Manual action badge for `needs_manual_action=1`
- Load more button for pagination
- Empty-state and error-state screens

## 8. Query and Performance Guidance

- Build SQL dynamically with bound params only.
- Escape `%` and `_` for LIKE search terms to avoid wildcard surprises.
- Use `LOWER(...) LIKE LOWER(?)` initially.
- If query latency grows with volume, add optional FTS5 later.

## 9. Error Handling and Observability

- Include structured logs with `history_id`, `batch_id`, `user_id`, state transitions.
- Do not log secrets/token values.
- Return user-safe error messages from server-fn boundary.

## 10. Implementation Steps

1. Migration + model + typed status enums
2. History repository functions (`create`, `update_transition`, `query`, `get`, `delete`, `retry_seed`)
3. Queue integration in `download/mod.rs`
4. Monitor transition integration in `download/monitor.rs`
5. Import transition integration in `download/import.rs`
6. History API endpoints in new `api/src/server_fns/history.rs`
7. Register `history` module in `api/src/server_fns/mod.rs`
8. UI route/page/components and navbar link
9. End-to-end manual verification

## 11. Testing Checklist (Branch-level)

- Queue success path writes rows and transitions to imported
- Queue download failure writes failed row
- Import skipped writes skipped status
- Retry creates new attempt row, old row remains immutable
- Cross-user access to history item is denied
- Pagination and filters return stable ordered results

## 12. Confirmed Branch Decisions

Captured from implementation goals on 2026-03-04:
- Group primary history cards by `batch_id`.
- Retry action should perform both download and import in one flow (full retry).
- Cancelled items are retryable.
- History delete behavior in MVP is hard-delete (row is removed).
- No automatic retention policy in this branch (history kept indefinitely).
