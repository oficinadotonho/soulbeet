# Domain Models and Public Interfaces

## Core shared models (`lib/shared`)

### Metadata contracts

- Provider enum and parsing/display: [`Provider`](../lib/shared/src/metadata.rs)
- Search outputs: [`SearchResults`](../lib/shared/src/metadata.rs), [`SearchResult`](../lib/shared/src/metadata.rs)
- Canonical entities: [`Track`](../lib/shared/src/metadata.rs), [`Album`](../lib/shared/src/metadata.rs), [`AlbumWithTracks`](../lib/shared/src/metadata.rs)

### Download contracts

- Query for backend source search: [`DownloadQuery`](../lib/shared/src/download.rs)
- Candidate files/grouping: [`DownloadableItem`](../lib/shared/src/download.rs), [`DownloadableGroup`](../lib/shared/src/download.rs)
- Search lifecycle: [`SearchState`](../lib/shared/src/download.rs), [`SearchResult`](../lib/shared/src/download.rs)
- Download lifecycle: [`DownloadState`](../lib/shared/src/download.rs), [`DownloadProgress`](../lib/shared/src/download.rs)
- Queue response: [`QueuedDownload`](../lib/shared/src/download.rs)

### Library/system contracts

- Duplicate report types: [`LibraryTrack`](../lib/shared/src/library.rs), [`DuplicateGroup`](../lib/shared/src/library.rs), [`DuplicateReport`](../lib/shared/src/library.rs)
- Health/backends: [`SystemHealth`](../lib/shared/src/system.rs), [`AvailableBackends`](../lib/shared/src/system.rs)

## API model layer (`api/src/models`)

- User persistence model with hidden password hash serialization field:
  - [`User`](../api/src/models/user.rs)
- User folder model:
  - [`Folder`](../api/src/models/folder.rs)
- User preferences:
  - [`UserSettings`](../api/src/models/user_settings.rs), [`UpdateUserSettings`](../api/src/models/user_settings.rs)
- Runtime-config-in-DB keys and KV values:
  - [`keys`](../api/src/models/app_config.rs), [`AppConfig`](../api/src/models/app_config.rs)

## Auth/session model

- JWT claims and auth response:
  - [`Claims`](../api/src/auth.rs)
  - [`AuthResponse`](../api/src/auth.rs)
- Cookie name constant:
  - [`AUTH_COOKIE_NAME`](../api/src/server_fns/auth.rs)

## Pluggable service interfaces (`lib/soulbeet/src/traits.rs`)

- Metadata provider interface:
  - [`MetadataProvider`](../lib/soulbeet/src/traits.rs)
- Download backend interface:
  - [`DownloadBackend`](../lib/soulbeet/src/traits.rs)
- Importer interface:
  - [`MusicImporter`](../lib/soulbeet/src/traits.rs)
- Import result abstraction:
  - [`ImportResult`](../lib/soulbeet/src/traits.rs)

These traits are the highest-value extension seam for adding integrations.

## Concrete interface implementations in current code

- `MetadataProvider`:
  - MusicBrainz provider in [`lib/soulbeet/src/musicbrainz.rs`](../lib/soulbeet/src/musicbrainz.rs)
  - Last.fm provider in [`lib/soulbeet/src/lastfm.rs`](../lib/soulbeet/src/lastfm.rs)
- `DownloadBackend`:
  - slskd client in [`lib/soulbeet/src/slskd/client.rs`](../lib/soulbeet/src/slskd/client.rs)
- `MusicImporter`:
  - beets importer in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs)

## Public HTTP/server-fn interface map

All routes below are defined in `#[get]/#[post]/#[put]/#[delete]` server functions:

- Auth: `/api/auth/register`, `/api/auth/login`, `/api/auth/refresh`, `/api/auth/logout`, `/api/auth/me`
  - [`api/src/server_fns/auth.rs`](../api/src/server_fns/auth.rs)
- Users: `/api/users`, `/api/users/password`, `/api/users/delete`
  - [`api/src/server_fns/user.rs`](../api/src/server_fns/user.rs)
- Folders: `/api/folders`, `/api/folders/update`, `/api/folders/delete`, `/api/folders/duplicates`
  - [`api/src/server_fns/folder.rs`](../api/src/server_fns/folder.rs)
- Metadata and source search:
  - `/api/metadata/search/album`, `/api/metadata/search/track`, `/api/metadata/album`
  - `/api/download/search/start`, `/api/download/search/poll`
  - [`api/src/server_fns/search.rs`](../api/src/server_fns/search.rs)
- Download queue + updates websocket:
  - `/api/downloads/queue`, `/api/downloads/updates`
  - [`api/src/server_fns/download/mod.rs`](../api/src/server_fns/download/mod.rs)
- Settings and app config:
  - `/api/settings`, `/api/settings/providers`, `/api/config`
  - [`api/src/server_fns/settings.rs`](../api/src/server_fns/settings.rs)
- System:
  - `/api/system/health`, `/api/system/backends`
  - [`api/src/server_fns/system.rs`](../api/src/server_fns/system.rs)

