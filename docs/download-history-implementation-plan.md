# Download History Implementation Plan (History Core)

## Summary
This branch implements durable per-user history for two attempt kinds:
- `download_attempt`
- `search_attempt`

Implemented scope is intentionally limited to history core behavior. Retry/cancel flows are explicitly deferred.

## Scope

### In scope
- Persist download attempt lifecycle (`queued -> in_progress -> completed/failed/timeout/cancelled`) with import lifecycle (`not_started -> in_progress -> completed/skipped/failed/timeout`).
- Persist metadata search attempts (`metadata_album`, `metadata_track`) and source download-search attempts (`source_download`).
- Persist accurate source-search timeout status (`timed_out`, not `completed`).
- Expose history API for query/get/delete/clear.
- Add History UI with kind switching pills:
  - `Download Attempts`
  - `Search Attempts`
- Group download attempts in UI by hierarchy:
  - `Date > Release > Action > Tracks`
- Hard-delete semantics for single delete and clear all.

### Out of scope
- Retry endpoint or retry UI in this branch.
- Cancel endpoint or cancel UI in this branch.
- Retention policy / auto-pruning.

## Data Model

### `download_history`
One row per queued item attempt:
- Identity/correlation: `id`, `user_id`, `action_id`, `queue_item_id`
- Content: `source`, `title`, `artist`, `release_name`, `item_label`, `size_bytes`
- Context: `backend_id`, `target_folder`, `downloadable_item_json`, `tracks_json`
- Lifecycle:
  - `download_status`: `queued | in_progress | completed | failed | cancelled | timeout`
  - `import_status`: `not_started | in_progress | completed | skipped | failed | timeout`
  - `started_at`, `downloaded_at`, `imported_at`, `ended_at`
  - `error_message`, `needs_manual_action`

### `search_attempt_history`
One row per search attempt:
- Identity/correlation: `id`, `user_id`, `search_id` (nullable)
- Classification: `attempt_type` (`metadata_album | metadata_track | source_download`)
- Context: `provider_id`, `backend_id`, `query_text`, `artist_text`
- Lifecycle:
  - `status`: `in_progress | completed | timed_out | failed | no_results`
  - `result_count`, `error_message`
  - `started_at`, `ended_at`

## API Surface

Server-fn routes:
- `POST /api/history/query`
- `POST /api/history/get`
- `POST /api/history/delete`
- `POST /api/history/clear`

No `/api/history/retry` route in this branch.

## Write-Path Integration

### Download attempts
- Queue stage inserts rows for both successful queued items and immediate queue failures.
- Monitor stage persists download state transitions without writing on every poll (state-change driven).
- Timeout path explicitly persists `download_status=timeout`.
- Import stage persists import transitions and terminal results (`completed/skipped/failed/timeout`) including error metadata.

### Search attempts
- Metadata album/track searches persist completed/no-results/failed attempts with timing and counts.
- Source search start inserts `in_progress` record with `search_id` correlation.
- Source poll updates attempt status and result counts, including `timed_out` and `not_found -> failed` handling.

## UI Behavior

### Route and navigation
- Route: `/history`
- Navbar includes a History link.

### Layout and interaction
- Pill switch between `Download Attempts` and `Search Attempts`.
- Filters are kind-aware:
  - Download view: search text, date range, download status, import status.
  - Search view: search text, date range, attempt type, attempt status.
- Initial browser-refresh load error is suppressed on first mount only.
- Explicit action errors remain visible (apply filters/load more/delete/clear).

### Grouping
- Download attempts: `Date > Release > Action > Tracks`
- Search attempts: grouped by `Date`
- `release_name` fallback bucket: `Unknown Release`

### Delete/Clear
- Single-item hard delete.
- Clear-all confirmation modal text:
  - “Are you sure you want to delete ALL history? This action is irreversible.”

## Validation Gates

### Build gate
- `cargo check` must pass.

### Manual acceptance matrix
- Browser refresh on `/history` shows no initial "Failed to load" banner.
- Download lifecycle transitions appear correctly in history.
- Import outcomes (`completed/skipped/failed/timeout`) persist correctly.
- Source search timeout persists as `timed_out`.
- Kind pills switch correctly and filters apply to each kind.
- Clear-all confirmation appears and hard-deletes rows after confirmation.

## Deferred follow-up branches
- `feat/retry`: retry endpoint/UI and attempt requeue semantics.
- `feat/cancel`: explicit cancel endpoint/UI and state flow integration.
