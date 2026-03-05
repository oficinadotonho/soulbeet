# Test Suite Implementation Plan (`feat/test-suite`)

## Goal

Create a reliable automated test suite for Soulbeet that covers:

- API behavior and auth/session flows
- DB migrations and schema compatibility
- History/search core regressions
- Minimal UI smoke coverage

The plan is optimized for your real test environment in `/opt/soulbeet` and for Codex execution constraints.

## Current Constraints

- `/opt/soulbeet` is root-owned, so some operations require `sudo` from you.
- External dependencies (slskd, music providers, network) can make tests flaky.
- We need deterministic tests for CI/local repeatability.
- Existing issues showed cross-layer failures (migrations + auth cookie + FK + UI health signal).

## Test Strategy

### Layer 1: Migration and DB Safety (first priority)

Purpose:
- Prevent startup regressions like `VersionMismatch(...)`.
- Ensure old DB states migrate cleanly to current schema.

Coverage:
- Fresh DB migration up to head.
- Upgrade from pre-history schema snapshot.
- Upgrade from intermediate history schema snapshot.
- Verify final schema shape and indexes.

Implementation:
- Add `scripts/test_migrations.sh` using `sqlite3`.
- Keep fixture `.sql` files in `tests/fixtures/migrations/`.
- Assertions:
  - `_sqlx_migrations` contains expected versions.
  - `download_history` columns match expected action schema.
  - `search_attempt_history` exists and constraints are valid.

### Layer 2: API Integration Tests (second priority)

Purpose:
- Validate server functions and DB side effects end-to-end.

Coverage (minimum set):
- `auth`: register, login, logout, refresh, `/me`.
- `history`: query/get/delete/clear for both kinds.
- `search`:
  - metadata album/track success path
  - no-results path
  - search-attempt persistence row created
- `system/health`:
  - downloader false when no slskd config
  - beets health is surfaced

Implementation:
- Rust integration tests under `api/tests/`.
- Use isolated DB per test (`sqlite:/tmp/soulbeet_test_<uuid>.db`).
- Create a test harness:
  - sets required env vars (`DATABASE_URL`, `SECRET_KEY`, etc.)
  - initializes app and migrations
  - executes HTTP requests against test server/router
- Avoid live slskd calls in integration tests:
  - inject/mimic backend behavior where possible
  - focus on metadata + history paths first

### Layer 3: UI Smoke E2E (third priority)

Purpose:
- Catch regressions that API tests miss (loading/error states, broken flows).

Coverage (small but critical):
- Login screen -> login success.
- Search page renders and can perform a metadata search.
- History page loads and filters can be applied.
- Clear-all confirmation modal appears and can cancel/confirm.

Implementation:
- Playwright setup in `tests/e2e/`.
- Use a deterministic local server + seeded DB.
- Run headless by default.
- Keep UI tests limited to smoke scenarios to reduce flakiness.

## Environment Configuration

### A. Local workspace (`/opt/dev/soulbeet`)

Install tooling:

```bash
# Rust + sqlite tools
cargo --version
sqlite3 --version

# Node tools (for Playwright)
node --version
npm --version
```

Project test deps:

```bash
# if Playwright is adopted
npm i -D @playwright/test
npx playwright install --with-deps chromium
```

### B. Real test environment (`/opt/soulbeet`)

Expected layout:

- `/opt/soulbeet/repo` (synced source)
- `/opt/soulbeet/data/soulbeet.db` (runtime DB)
- `/opt/soulbeet/docker-compose.yml`

Compose requirements:
- Build from `/opt/soulbeet/repo`.
- Stable `DATABASE_URL=sqlite:/data/soulbeet.db`.
- Optional slskd vars when testing downloader-online behavior.

## Branch Plan and Commits

### Phase 1: Test foundation

Commit:
- `test(infra): add migration test script and fixtures`

Changes:
- `scripts/test_migrations.sh`
- migration fixture SQL files
- `make test-migrations` (or equivalent command)

### Phase 2: API integration harness

Commit:
- `test(api): add integration harness for auth history and search`

Changes:
- `api/tests/common/mod.rs` harness
- first integration tests for auth/history/search

### Phase 3: System and regression tests

Commit:
- `test(api): add regression tests for migration and fk edge cases`

Changes:
- tests for stale token/user mismatch behavior
- tests for history/search FK-safe persistence expectations

### Phase 4: UI smoke tests

Commit:
- `test(e2e): add playwright smoke flows for login search and history`

Changes:
- Playwright config + smoke specs
- helper scripts for seed/reset DB

### Phase 5: DX and docs

Commit:
- `docs(testing): document local and /opt/soulbeet test workflows`

Changes:
- `docs/testing-workflow.md`
- command matrix for fast, full, and pre-PR runs

## Execution Commands (target state)

Fast pre-commit:

```bash
./scripts/test_migrations.sh
cargo test -p api --test auth_history_search -- --nocapture
```

Full local:

```bash
./scripts/test_migrations.sh
cargo test -p api --features server
npx playwright test
```

Real env smoke:

```bash
docker compose -f /opt/soulbeet/docker-compose.yml build soulbeet
docker compose -f /opt/soulbeet/docker-compose.yml up -d soulbeet
docker compose -f /opt/soulbeet/docker-compose.yml logs --tail=200 soulbeet
```

## Risks and Mitigations

- Flaky external services:
  - Mitigate with mocks/stubs and deterministic fixtures.
- Permission friction in `/opt/soulbeet`:
  - Keep build/test execution mostly in `/opt/dev`.
  - Use `/opt/soulbeet` for deployment smoke only.
- UI E2E brittleness:
  - Keep selectors stable (`data-testid` where needed).
  - Keep scenario count intentionally small.

## Definition of Done

- Migration test script passes on fresh and upgrade fixtures.
- API integration tests cover auth + history + search-attempt persistence.
- UI smoke tests pass headless locally.
- One command each for fast and full test runs is documented.
- Real environment smoke deploy to `/opt/soulbeet` succeeds.
