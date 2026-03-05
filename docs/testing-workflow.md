# Testing Workflow

## Scope

This document defines repeatable test workflows for:

- Local development in `/opt/dev/soulbeet`
- Real smoke deployment in `/opt/soulbeet`

## Prerequisites

- `sqlite3`
- `cargo`
- `node` + `npm`
- Docker + Docker Compose (for real-env smoke)

Quick checks:

```bash
sqlite3 --version
cargo --version
node --version
npm --version
docker --version
docker compose version
```

## Fast Local (pre-commit)

Run the fastest deterministic checks:

```bash
make test-migrations
npm run test:e2e:list
```

Notes:
- `test-migrations` validates fresh + upgrade migration paths and schema expectations.
- `test:e2e:list` validates Playwright test discovery without requiring an active app.

## Local Full (feature validation)

Run a fuller pass before PR:

```bash
make test-migrations
npm run test:e2e:list
```

If app is running and Playwright browser deps are installed, run smoke UI:

```bash
npx playwright test
```

## Real Environment Smoke (`/opt/soulbeet`)

### 1) Reset DB and optionally seed config

```bash
./scripts/e2e_reset_db.sh /opt/soulbeet/data/soulbeet.db
# optional:
SLSKD_URL=http://slskd:5030 SLSKD_API_KEY=... \
  ./scripts/e2e_seed_config.sh /opt/soulbeet/data/soulbeet.db
```

### 2) Build and restart service

```bash
docker compose -f /opt/soulbeet/docker-compose.yml build soulbeet
docker compose -f /opt/soulbeet/docker-compose.yml up -d soulbeet
docker compose -f /opt/soulbeet/docker-compose.yml logs --tail=200 soulbeet
```

### 3) Verify DB migration state

```bash
sqlite3 /opt/soulbeet/data/soulbeet.db \
  "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"
```

## Known Limitation

`cargo test -p api --features server` currently fails in this repository state due existing
`dioxus_server`/server-fn macro compile issues in `api` test target context.

This does not block migration script coverage or Playwright discovery checks, but it blocks
Rust integration test execution until the compile issue is resolved.

## Recommended Pre-PR Checklist

1. `make test-migrations`
2. `npm run test:e2e:list`
3. Real env smoke deploy (`docker compose build/up/logs`)
4. Manual auth/search/history smoke in browser:
   - login
   - metadata search
   - history query/filter/clear
