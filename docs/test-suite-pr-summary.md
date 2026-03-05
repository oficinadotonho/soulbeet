# Test Suite PR Summary

## Push Command

```bash
git push -u origin feat/test-suite
```

## PR Title

`test-suite: add migration/API/E2E coverage and stable test entrypoints`

## PR Body

## Summary

This PR builds a practical automated test suite for Soulbeet and makes it runnable through consistent `make` entrypoints.

It adds:
- migration safety tests (fresh + upgrade fixtures),
- API integration tests for auth/history/search and FK regressions,
- pipeline lifecycle integration tests for download/search history transitions,
- Playwright smoke tests with self-hosted app startup in local toolchain mode,
- testing workflow documentation updates.

## What Changed

### 1) Migration test coverage
- Added fixture-based migration script:
  - `scripts/test_migrations.sh`
- Added migration fixtures:
  - `tests/fixtures/migrations/legacy_pre_history.sql`
  - `tests/fixtures/migrations/legacy_old_history_schema.sql`
  - `tests/fixtures/migrations/expected_versions.txt`

### 2) API integration coverage
- Added shared test harness:
  - `api/tests/common/mod.rs`
- Added tests:
  - `api/tests/auth_history_search.rs`
  - `api/tests/history_fk_regressions.rs`
  - `api/tests/pipeline_lifecycle.rs`

New lifecycle coverage validates:
- download history state progression persistence,
- search attempt update semantics (`update_by_search_id` latest-row behavior).

### 3) E2E smoke coverage
- Added Playwright config/spec:
  - `playwright.config.mjs`
  - `tests/e2e/smoke.spec.js`
- Playwright now starts the app server for smoke runs in this environment.

### 4) Test execution entrypoints
- Expanded `Makefile` targets:
  - `test-migrations`
  - `test-api`
  - `test-fast`
  - `test-full`

### 5) Reliability fixes required by test execution
- Fixed server compile regressions in:
  - `api/src/server_fns/download/history.rs`
  - `api/src/server_fns/search.rs`
- Fixed invalid dynamic SQL update generation in:
  - `api/src/models/download_history.rs`
  - `api/src/models/search_attempt_history.rs`
- Made migration script resilient when optional migration files are absent on branch state.

### 6) Docs
- Updated:
  - `docs/testing-workflow.md`
  - `docs/test-suite-implementation-plan.md`

## Validation

Executed successfully:
- `make test-fast`
- `make test-full`

Both pass on this branch.

## Notes

This PR improves confidence significantly, but it is still a controlled smoke/integration baseline rather than full production-parity external service coverage.
