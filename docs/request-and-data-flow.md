# Request and Data Flow

## Auth/session flow

1. UI bootstrap calls `get_current_user` in [`web/src/auth.rs`](../web/src/auth.rs).
2. Auth server function reads JWT cookie via guard extractor in [`api/src/server_fns/guard.rs`](../api/src/server_fns/guard.rs).
3. Token verification uses [`api/src/auth.rs`](../api/src/auth.rs) and `CONFIG.secret_key()` from [`api/src/config.rs`](../api/src/config.rs).
4. Login path:
   - Verify password hash (`argon2`) in [`api/src/models/user.rs`](../api/src/models/user.rs).
   - Issue token and set httpOnly cookie in [`api/src/server_fns/auth.rs`](../api/src/server_fns/auth.rs).

## Metadata search flow

1. UI calls:
   - `/api/metadata/search/album` or `/api/metadata/search/track` in [`api/src/server_fns/search.rs`](../api/src/server_fns/search.rs).
2. Endpoint resolves provider through registry in [`api/src/services.rs`](../api/src/services.rs).
3. Provider implementation:
   - MusicBrainz path in [`lib/soulbeet/src/musicbrainz.rs`](../lib/soulbeet/src/musicbrainz.rs), or
   - Last.fm path in [`lib/soulbeet/src/lastfm.rs`](../lib/soulbeet/src/lastfm.rs) (requires app_config key).
4. Response contract is [`SearchResults`](../lib/shared/src/metadata.rs) from `lib/shared`.

## Download search flow (finding sources)

1. UI posts [`DownloadQuery`](../lib/shared/src/download.rs) to `/api/download/search/start`.
2. Server resolves backend in [`api/src/services.rs`](../api/src/services.rs), currently `slskd` only.
3. slskd backend starts search via [`lib/soulbeet/src/slskd/client.rs`](../lib/soulbeet/src/slskd/client.rs).
4. Poll endpoint `/api/download/search/poll` returns grouped/ranked results as [`SearchResult`](../lib/shared/src/download.rs).

## Queue + monitor + import flow (core operational path)

### A) Queue

1. UI posts `/api/downloads/queue` with `items` + `target_folder` in [`api/src/server_fns/download/mod.rs`](../api/src/server_fns/download/mod.rs).
2. Server ensures target dir exists.
3. Backend queues files (`download_backend().download(...)`).
4. Failed queue entries are broadcast immediately as `DownloadProgress::failed`.
5. Successful queue entries are broadcast immediately as `DownloadProgress::queued`.

### B) Monitoring

1. Server registers per-user task/channel in [`api/src/globals.rs`](../api/src/globals.rs).
2. Spawned `DownloadMonitor` in [`api/src/server_fns/download/monitor.rs`](../api/src/server_fns/download/monitor.rs):
   - polls every 2s,
   - matches filenames with normalization/fuzzy suffix logic,
   - applies per-track timeout (1h),
   - emits live updates via broadcast channel.
3. Client receives updates over websocket `/api/downloads/updates`.

### C) Import

1. Once terminal/completed states reached, monitor invokes `process_downloads` in [`api/src/server_fns/download/process.rs`](../api/src/server_fns/download/process.rs).
2. Source path resolution heuristics in [`api/src/server_fns/download/utils.rs`](../api/src/server_fns/download/utils.rs).
3. Import invocation via `music_importer(None)` and `import_group` in [`api/src/server_fns/download/import.rs`](../api/src/server_fns/download/import.rs).
4. Beets importer executes `beet import` in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs), returning:
   - success,
   - skipped,
   - failed,
   - timed out.
5. Failed/skipped imports trigger source file cleanup in [`api/src/server_fns/download/import.rs`](../api/src/server_fns/download/import.rs).

## Data persistence flow

- DB pool init and migration at startup: [`api/src/db.rs`](../api/src/db.rs).
- Schema source of truth:
  - users/folders/admin bootstrap: [`api/migrations/20240523000000_init.sql`](../api/migrations/20240523000000_init.sql)
  - user settings: [`api/migrations/20250201000000_user_settings.sql`](../api/migrations/20250201000000_user_settings.sql)
  - app config K/V: [`api/migrations/20250201000001_app_config.sql`](../api/migrations/20250201000001_app_config.sql)

## Duplicate detection flow

1. UI/server calls `/api/folders/duplicates` in [`api/src/server_fns/folder.rs`](../api/src/server_fns/folder.rs).
2. Folder paths are loaded for current user.
3. Importer `find_duplicates` delegates to beets scanner in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs).
4. Beets CLI runs `beet ls` against each folder-local `.beets_library.db`, then groups by normalized `(artist,title)`.

