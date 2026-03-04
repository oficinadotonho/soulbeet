# Configuration, Build, and Runtime

## Environment behavior

Server env parsing is centralized in [`api/src/config.rs`](../api/src/config.rs):

- `DATABASE_URL` (default: `sqlite:soulbeet.db`)
- `SECRET_KEY` (default fallback `secret`, explicitly logged as insecure)
- `DOWNLOAD_PATH` (default: `/downloads`)
- `BEETS_CONFIG` (default: `beets_config.yaml`)
- `BEETS_ALBUM_MODE` (bool parser accepts `true/1/yes`, `false/0/no`)
- `PORT` (default `9765`)
- `IP` (default `0.0.0.0`)

DB-stored runtime config (`app_config` table) is separate from process env:

- Keys in [`api/src/models/app_config.rs`](../api/src/models/app_config.rs):
  - `slskd_url`, `slskd_api_key`, `lastfm_api_key`
- Accessed by service initialization in [`api/src/services.rs`](../api/src/services.rs).

## Database initialization and migrations

- Pool + migration execution at startup: [`api/src/db.rs`](../api/src/db.rs)
- Migration files:
  - base schema + default `admin/admin`: [`api/migrations/20240523000000_init.sql`](../api/migrations/20240523000000_init.sql)
  - user settings: [`api/migrations/20250201000000_user_settings.sql`](../api/migrations/20250201000000_user_settings.sql)
  - app config kv: [`api/migrations/20250201000001_app_config.sql`](../api/migrations/20250201000001_app_config.sql)

## Build/runtime commands in repo

### Web dev serve

- README command: `dx serve --platform web` in [`README.md`](../README.md)
- Web README command: `dx serve --package web --platform web --port 9797` in [`web/README.md`](../web/README.md)

### Tailwind build/watch

- Watch script in [`css.sh`](../css.sh)
- Dependencies in [`package.json`](../package.json)

### Docker build and run

- Build and run compose baseline in [`docker-compose.yml`](../docker-compose.yml)
- Multi-stage image in [`Dockerfile`](../Dockerfile):
  - Rust + dioxus build stage
  - python venv stage with `beets`
  - distroless runtime containing web server binary + beets venv

## Runtime command behavior relevant to features

- Beets import command assembly in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs):
  - `beet -c <config> -l <target/.beets_library.db> -d <target> import -q`
  - adds `-s` unless album mode
  - import timeout: 300s
- Beets health check: `beet --version` in [`lib/soulbeet/src/beets/mod.rs`](../lib/soulbeet/src/beets/mod.rs)

## Compile-time env forwarding

- [`api/build.rs`](../api/build.rs) loads `.env` (via `dotenvy`) and sets rustc env vars during build.
- It also marks `.env` and `api/migrations` for rebuild triggers.

## Operational constraints for deployment

- `DOWNLOAD_PATH` must reflect where slskd-downloaded files are visible inside Soulbeet container (path mapping alignment required).
- Target library paths used in folder settings must exist or be creatable by server process (`create_dir_all` in folder/download paths).
- `SECRET_KEY` should be overridden in any non-local deployment; default is insecure and warned at startup.

