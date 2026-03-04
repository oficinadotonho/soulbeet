# Testing, Conventions, Extension Points, Constraints

## Testing approach in current implementation

- No Rust unit/integration test modules were found in the codebase scan (no `#[test]`, `#[tokio::test]`, or `mod tests` matches under `*.rs`).
- Current quality strategy is runtime behavior + typed contracts + logging, not explicit automated test suites in-tree.

Implication for agent changes: prefer small, isolated changes with explicit error propagation and log points consistent with existing style.

## Coding patterns and conventions observed

- **Trait-based integration boundaries**
  - Core integration interfaces in [`lib/soulbeet/src/traits.rs`](../lib/soulbeet/src/traits.rs).
  - API layer depends on traits via service lookup, not concrete types.
- **Server feature gating**
  - Many modules/functions use `#[cfg(feature = "server")]` to separate server/runtime-only behavior.
- **Shared DTO crate for API/UI coupling**
  - Contracts centralized in [`lib/shared/src`](../lib/shared/src).
- **Error shape**
  - Server-fn endpoints normalize errors via [`server_error(...)`](../api/src/server_fns/mod.rs).
  - Integration crate uses [`SoulseekError`](../lib/soulbeet/src/error.rs).
- **Lazy globals for process-wide state**
  - Config singleton, DB lazy pool, user channel map.
- **Async + spawn for long-running background work**
  - Download monitor/import tasks in [`api/src/server_fns/download`](../api/src/server_fns/download).

## Extension points (highest leverage)

### 1) Add metadata provider

Implement [`MetadataProvider`](../lib/soulbeet/src/traits.rs), then register in [`api/src/services.rs`](../api/src/services.rs):

- Add provider id/name in `providers` module + `available_metadata_providers()`.
- Extend `init_metadata_provider(...)` to construct provider from DB/app config if needed.

### 2) Add download backend

Implement [`DownloadBackend`](../lib/soulbeet/src/traits.rs), register in [`api/src/services.rs`](../api/src/services.rs):

- Add backend id in `downloaders`.
- Add constructor in `init_download_backend(...)`.
- Existing endpoints in [`api/src/server_fns/search.rs`](../api/src/server_fns/search.rs) and [`api/src/server_fns/download/mod.rs`](../api/src/server_fns/download/mod.rs) already accept optional backend id.

### 3) Add music importer

Implement [`MusicImporter`](../lib/soulbeet/src/traits.rs), register in [`api/src/services.rs`](../api/src/services.rs):

- Add importer id in `importers`.
- Extend `init_importer(...)`.

### 4) Add persistent app config keys

- Add key constant in [`api/src/models/app_config.rs`](../api/src/models/app_config.rs).
- Extend request/response payload in [`api/src/server_fns/settings.rs`](../api/src/server_fns/settings.rs).
- Update service initialization logic in [`api/src/services.rs`](../api/src/services.rs).

## Known constraints and pitfalls

- **Auth scope is coarse**: all protected endpoints use authenticated guard, but no role-based authorization checks exist.
- **Default admin account exists by migration** (`admin/admin` bootstrap) in [`api/migrations/20240523000000_init.sql`](../api/migrations/20240523000000_init.sql).
- **SECRET_KEY default is insecure** and only warned, not hard-failed, in [`api/src/config.rs`](../api/src/config.rs).
- **Import shell-out dependency**: runtime requires `beet` executable and valid beets config path (`BEETS_CONFIG`).
- **Path resolution heuristics** for downloaded files can fail in uncommon slskd path layouts; fallback is recursive search depth 5 in [`api/src/server_fns/download/utils.rs`](../api/src/server_fns/download/utils.rs).
- **Long-running async tasks** rely on in-process memory state (`USER_CHANNELS`); restart drops transient progress channels.
- **Provider cache invalidation** happens only when settings endpoint calls `reload_providers()`.
- **Default mismatch in user settings**:
  - Migration default `last_search_type` is `'track'`.
  - Model fallback default in code is `'album'` when no row is returned.
  - See [`api/migrations/20250201000000_user_settings.sql`](../api/migrations/20250201000000_user_settings.sql) vs [`api/src/models/user_settings.rs`](../api/src/models/user_settings.rs).

## Minimal implementation examples

### Add provider id constant and availability listing

See pattern in [`api/src/services.rs`](../api/src/services.rs):

```rust
pub mod providers {
    pub const MUSICBRAINZ: &str = "musicbrainz";
    pub const LASTFM: &str = "lastfm";
}
```

### Add guarded server endpoint

See shape in [`api/src/server_fns/system.rs`](../api/src/server_fns/system.rs):

```rust
#[get("/api/system/health", _: AuthSession)]
pub async fn get_system_health() -> Result<SystemHealth, ServerFnError> { ... }
```

