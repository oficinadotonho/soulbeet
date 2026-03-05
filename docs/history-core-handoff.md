# History Core Handoff (2026-03-05)

## What Was Done In This Chat

### 1) Docker build failures were fixed

- Fixed Tailwind build failure (`@tailwindcss/oxide` native binding missing) in `Dockerfile`.
- `npm install` now includes optional dependencies and a fallback install for `@tailwindcss/oxide-linux-x64-gnu`.
- Additional build blockers found during `dx bundle` were fixed:
  - Added `serde_json` optional dependency to `api/Cargo.toml` and wired it into the `server` feature.
  - Fixed non-`Clone` error usage in `api/src/server_fns/search.rs`.
  - Fixed partial move issue in `api/src/server_fns/download/history.rs`.
- Result: Docker build completed successfully and produced image `soulbeet:history-core`.

### 2) Migration panic (`VersionMismatch(20260304000000)`) was fixed

- Root cause: migration `20260304000000_download_history.sql` had two historical versions (`batch_id` schema vs `action_id` schema).
- Existing DBs with the older checksum crashed at startup.
- Fix applied:
  - Restored `api/migrations/20260304000000_download_history.sql` to the original `batch_id` version.
  - Added new migration `api/migrations/20260305000000_download_history_action_schema.sql` to transform old schema into current action-based schema.
- Smoke-tested with `sqlite3`:
  - Fresh migration chain ends with expected final schema.
  - Old-schema data upgrades correctly.

## Current Problems

### 1) `DOWNLOADER OFFLINE` in UI

- This is currently expected in your compose as posted:
  - `SLSKD_URL` and `SLSKD_API_KEY` are commented out.
- Also, current backend initialization reads slskd config from `app_config` table (`AppConfig::get(...)`), not directly from process env for runtime lookup.
- With a fresh DB, `app_config` starts empty, so downloader health reports offline until values are saved in app config.

### 2) Search attempt persistence warning

- Warning seen:
  - `Failed to persist search attempt ... FOREIGN KEY constraint failed (code 787)`
- Likely cause:
  - `search_attempt_history.user_id` references `users(id)`.
  - Auth guard currently validates JWT signature only, not whether `claims.sub` still exists in `users`.
  - After deleting DB, a stale but valid cookie token can pass auth and then fail FK when writing search attempts.

## How To Proceed In The Real Test Folder

### Step 1: Build and run in the new test folder

Run:

```bash
docker build -t soulbeet:history-core .
```

Then run with your compose stack as usual.

### Step 2: Configure slskd before testing downloader status

Either:

- Set values in app config via Settings UI (Config tab), or
- Seed `app_config` table manually in DB.

Without these values, `DOWNLOADER OFFLINE` is expected.

### Step 3: Clear auth state after DB reset

After deleting `soulbeet.db`, clear the browser cookie/session for `auth_token` before testing authenticated actions.

### Step 4: Verify critical flows

- Register user -> login -> metadata search.
- Download search start/poll.
- History query and rendering.
- Restart server and re-test authenticated endpoints.

### Step 5: Implement next hardening fix (recommended)

Implement user existence verification inside auth guard (`api/src/server_fns/guard.rs`) so stale JWTs are rejected with `401` early, instead of failing later on FK writes.

Optional follow-up:

- Add startup bootstrap to copy `SLSKD_URL` / `SLSKD_API_KEY` env values into `app_config` when missing, so fresh DBs can come up online without manual config.

## Notes For Continuation

- This repo currently contains uncommitted working changes from the debugging session.
- In the real test folder, run full checks there (`cargo check`, docker build, and manual flow tests) since that environment has the required network/service access.
