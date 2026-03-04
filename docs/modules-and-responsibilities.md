# Modules and Responsibilities

## `api` crate

- Entry exports: [`api/src/lib.rs`](../api/src/lib.rs)
- Responsibility: server-side app orchestration (auth, DB access, endpoint handlers, service wiring).

### Core modules

- Config/env parsing and singleton:
  - [`api/src/config.rs`](../api/src/config.rs)
- DB pool + migrations:
  - [`api/src/db.rs`](../api/src/db.rs)
- Per-user websocket channel registry / cleanup:
  - [`api/src/globals.rs`](../api/src/globals.rs)
- JWT issue/verify and claim model:
  - [`api/src/auth.rs`](../api/src/auth.rs)
- Runtime service lookup/cache:
  - [`api/src/services.rs`](../api/src/services.rs)

### Models (`api/src/models`)

- Users + password hashing and verification:
  - [`api/src/models/user.rs`](../api/src/models/user.rs)
- Folder records for user libraries:
  - [`api/src/models/folder.rs`](../api/src/models/folder.rs)
- Per-user settings:
  - [`api/src/models/user_settings.rs`](../api/src/models/user_settings.rs)
- Key/value app config persisted in DB:
  - [`api/src/models/app_config.rs`](../api/src/models/app_config.rs)

### Server function endpoints (`api/src/server_fns`)

- Module index and shared error wrapper:
  - [`api/src/server_fns/mod.rs`](../api/src/server_fns/mod.rs)
- Auth endpoints (register/login/logout/me/refresh):
  - [`api/src/server_fns/auth.rs`](../api/src/server_fns/auth.rs)
- Auth extractor (`AuthSession`):
  - [`api/src/server_fns/guard.rs`](../api/src/server_fns/guard.rs)
- User CRUD-ish admin actions:
  - [`api/src/server_fns/user.rs`](../api/src/server_fns/user.rs)
- Folder CRUD + duplicate detection:
  - [`api/src/server_fns/folder.rs`](../api/src/server_fns/folder.rs)
- Metadata/search/download-search endpoints:
  - [`api/src/server_fns/search.rs`](../api/src/server_fns/search.rs)
- User settings + app config endpoints:
  - [`api/src/server_fns/settings.rs`](../api/src/server_fns/settings.rs)
- Health/backend capability endpoints:
  - [`api/src/server_fns/system.rs`](../api/src/server_fns/system.rs)
- Download queue + websocket updates + monitor/import pipeline:
  - [`api/src/server_fns/download/mod.rs`](../api/src/server_fns/download/mod.rs)
  - [`api/src/server_fns/download/monitor.rs`](../api/src/server_fns/download/monitor.rs)
  - [`api/src/server_fns/download/process.rs`](../api/src/server_fns/download/process.rs)
  - [`api/src/server_fns/download/import.rs`](../api/src/server_fns/download/import.rs)
  - [`api/src/server_fns/download/utils.rs`](../api/src/server_fns/download/utils.rs)

## `lib/soulbeet` crate

- Entry and trait re-exports: [`lib/soulbeet/src/lib.rs`](../lib/soulbeet/src/lib.rs)
- Responsibility: external integration logic behind stable traits.

### Trait contracts and composition

- `MetadataProvider`, `DownloadBackend`, `MusicImporter`, `ImportResult`:
  - [`lib/soulbeet/src/traits.rs`](../lib/soulbeet/src/traits.rs)
- Optional in-memory service aggregator/builder:
  - [`lib/soulbeet/src/services.rs`](../lib/soulbeet/src/services.rs)

### Implementations

- Beets importer and duplicate scanning:
  - [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs)
- MusicBrainz provider:
  - [`lib/soulbeet/src/musicbrainz.rs`](../lib/soulbeet/src/musicbrainz.rs)
- Last.fm provider:
  - [`lib/soulbeet/src/lastfm.rs`](../lib/soulbeet/src/lastfm.rs)
- slskd backend implementation:
  - Client: [`lib/soulbeet/src/slskd/client.rs`](../lib/soulbeet/src/slskd/client.rs)
  - Search result processing/ranking: [`lib/soulbeet/src/slskd/processing.rs`](../lib/soulbeet/src/slskd/processing.rs)
  - Utility matching helpers: [`lib/soulbeet/src/slskd/utils.rs`](../lib/soulbeet/src/slskd/utils.rs)

## `lib/shared` crate

- Responsibility: serde-ready API/shared contracts used by both server and UI.
- Module map:
  - [`lib/shared/src/download.rs`](../lib/shared/src/download.rs)
  - [`lib/shared/src/metadata.rs`](../lib/shared/src/metadata.rs)
  - [`lib/shared/src/library.rs`](../lib/shared/src/library.rs)
  - [`lib/shared/src/system.rs`](../lib/shared/src/system.rs)
  - [`lib/shared/src/slskd.rs`](../lib/shared/src/slskd.rs)

## `web` crate

- App entrypoint, routes, top-level providers:
  - [`web/src/main.rs`](../web/src/main.rs)
- Auth provider and user bootstrap:
  - [`web/src/auth.rs`](../web/src/auth.rs)
- Browser websocket resilience/reconnect:
  - [`web/src/websocket.rs`](../web/src/websocket.rs)

## `ui` crate

- Shared components + contexts:
  - Exports: [`ui/src/lib.rs`](../ui/src/lib.rs)
  - Settings context/provider: [`ui/src/settings_context.rs`](../ui/src/settings_context.rs)

## Infra/build artifacts

- Container image build/runtime shape:
  - [`Dockerfile`](../Dockerfile)
- Compose baseline:
  - [`docker-compose.yml`](../docker-compose.yml)
- Tailwind watcher script:
  - [`css.sh`](../css.sh)
- API compile-time env forwarding:
  - [`api/build.rs`](../api/build.rs)

