# Architecture

## Workspace shape

Soulbeet is a Rust workspace with Dioxus fullstack app composition.

- Workspace members in [`Cargo.toml`](../Cargo.toml):
  - [`api`](../api/Cargo.toml): server functions, auth, DB, service registry.
  - [`web`](../web/Cargo.toml): Dioxus app, routes, auth provider, websocket client.
  - [`ui`](../ui/Cargo.toml): shared UI/components/context.
  - [`lib/soulbeet`](../lib/soulbeet/Cargo.toml): domain integrations (slskd, beets, metadata providers).
- Shared DTOs and protocol types are in [`lib/shared`](../lib/shared/Cargo.toml), consumed by both API and UI.

`desktop`/`mobile` crates exist but are commented out from workspace members in [`Cargo.toml`](../Cargo.toml).

## Runtime composition

Primary runtime is the `web` package using Dioxus with server features.

- Server bootstrap in [`web/src/main.rs`](../web/src/main.rs):
  - Starts Dioxus server router.
  - Adds cookie middleware (`CookieManagerLayer`).
  - Starts background user-channel cleanup task (`api::globals::start_channel_cleanup_task`).
- API endpoints are Dioxus server functions declared in [`api/src/server_fns`](../api/src/server_fns).

## External systems

- **SQLite** for app state/users/settings/config:
  - Pool and migrations in [`api/src/db.rs`](../api/src/db.rs).
  - Schema in [`api/migrations`](../api/migrations).
- **slskd** as download/search backend:
  - Client and batching/circuit-breaker in [`lib/soulbeet/src/slskd/client.rs`](../lib/soulbeet/src/slskd/client.rs).
  - Wired via service registry in [`api/src/services.rs`](../api/src/services.rs).
- **beets CLI** for import and duplicate scanning:
  - Import/timeout/parsing behavior in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs).
- **Metadata providers**:
  - MusicBrainz: [`lib/soulbeet/src/musicbrainz.rs`](../lib/soulbeet/src/musicbrainz.rs)
  - Last.fm: [`lib/soulbeet/src/lastfm.rs`](../lib/soulbeet/src/lastfm.rs)

## Internal layering

1. **UI/web route & state layer**
   - Auth/settings providers in [`web/src/auth.rs`](../web/src/auth.rs) and [`ui/src/settings_context.rs`](../ui/src/settings_context.rs).
2. **Server-fn API layer**
   - Endpoint modules in [`api/src/server_fns/mod.rs`](../api/src/server_fns/mod.rs).
3. **Service registry layer**
   - Lazy, cached provider/backend/importer instances in [`api/src/services.rs`](../api/src/services.rs).
4. **Domain integration layer**
   - Trait implementations + integrations in [`lib/soulbeet/src`](../lib/soulbeet/src).
5. **Shared contracts**
   - Cross-crate serializable types in [`lib/shared/src`](../lib/shared/src).

## State and concurrency primitives

- Global config singleton: [`CONFIG`](../api/src/config.rs) (LazyLock).
- DB pool singleton: [`DB`](../api/src/db.rs) (Dioxus Lazy async init).
- Per-user websocket broadcast channels and cancellation tokens:
  - [`api/src/globals.rs`](../api/src/globals.rs).
- Download monitor tasks are spawned per queue request:
  - [`api/src/server_fns/download/mod.rs`](../api/src/server_fns/download/mod.rs).

## Security-relevant architecture notes

- Auth is cookie + JWT claims extraction:
  - Token create/verify: [`api/src/auth.rs`](../api/src/auth.rs).
  - Request guard: [`api/src/server_fns/guard.rs`](../api/src/server_fns/guard.rs).
- No role model exists in DB schema or user model (only authenticated vs unauthenticated).

