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
- Playwright browser runtime dependencies on Linux (`libnspr4`, `libnss3`, etc.)

Quick checks:

```bash
sqlite3 --version
cargo --version
node --version
npm --version
docker --version
docker compose version
```

Install Playwright browser binaries:

```bash
npx playwright install chromium
```

If Chromium fails to launch with missing shared libraries (example: `libnspr4.so`), install OS deps:

```bash
sudo npx playwright install-deps chromium
```

## Fast Local (pre-commit)

Run the fastest deterministic checks:

```bash
make test-fast
```

Notes:
- `test-fast` runs migration tests and API integration tests.

## Local Full (feature validation)

Run a fuller pass before PR:

```bash
make test-full
```

If you only want to verify Playwright discovery:

```bash
npm run test:e2e:list
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

## Recommended Pre-PR Checklist

1. `make test-fast`
2. `make test-full` (or `npm run test:e2e:list` if browser runtime deps are not installed)
3. Real env smoke deploy (`docker compose build/up/logs`)
4. Manual auth/search/history smoke in browser:
   - login
   - metadata search
   - history query/filter/clear
