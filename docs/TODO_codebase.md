# TODO Codebase (PR-Structured)

## PR 1 - Authorization and Tenant Isolation (Critical)

### 1.1 Lock down user-management endpoints
- [ ] Add role/permission checks to `/api/users`, `/api/users/password`, `/api/users/delete`.
- [ ] Restrict these endpoints to admins only (or remove from non-admin UI/API surface).
- [ ] Add audit logging for password resets and user deletions.

Files:
- `api/src/server_fns/user.rs`

### 1.2 Enforce ownership checks for folder mutations
- [ ] Update folder update/delete flows to require both `folder_id` and authenticated `user_id`.
- [ ] Change SQL to `WHERE id = ? AND user_id = ?`.
- [ ] Return explicit not-found/forbidden behavior when ownership check fails.

Files:
- `api/src/server_fns/folder.rs`
- `api/src/models/folder.rs`

### 1.3 Tighten target folder write surface
- [ ] Validate `target_folder` against user-owned configured folders.
- [ ] Reject arbitrary absolute paths not in user folder configuration.
- [ ] Add normalization + traversal checks before `create_dir_all`.

Files:
- `api/src/server_fns/download/mod.rs`
- `api/src/models/folder.rs`

---

## PR 2 - Download/Import Reliability and Correctness (High)

### 2.1 Fix cleanup path correctness after import failures
- [ ] Ensure cleanup uses resolved filesystem paths, not display/download item identifiers.
- [ ] Validate that cleanup only touches files created by the active operation.
- [ ] Add structured logging for cleanup success/failure per path.

Files:
- `api/src/server_fns/download/import.rs`
- `api/src/server_fns/download/process.rs`
- `api/src/server_fns/download/utils.rs`

### 2.2 Strengthen monitor completion semantics
- [ ] Avoid silently exiting monitor after empty polls without a terminal state update.
- [ ] Emit terminal failure state for unresolved tracked items at timeout/abandon points.
- [ ] Define deterministic behavior when backend drops completed entries quickly.

Files:
- `api/src/server_fns/download/monitor.rs`

---

## PR 3 - Security Hygiene and Operational Guardrails (Medium)

### 3.1 Remove sensitive token logging
- [ ] Stop logging raw auth token values on verification failure.
- [ ] Keep minimal metadata (reason, request context) for diagnostics.

Files:
- `api/src/server_fns/guard.rs`

### 3.2 Add regression tests around authz and folder ownership
- [ ] Add tests for cross-user folder update/delete denial.
- [ ] Add tests for privileged user-management endpoints.
- [ ] Add tests for invalid `target_folder` rejection.

Suggested locations:
- `api/src/server_fns/*` test modules or integration tests under `api/tests/`

---

## PR 4 - Follow-up Hardening (Optional)

### 4.1 Centralized authorization helpers
- [ ] Introduce helper guard/utilities for role checks and ownership checks to reduce endpoint drift.

### 4.2 Security configuration tightening
- [ ] Consider fail-fast startup in non-dev if `SECRET_KEY` is default/insecure.
- [ ] Add environment-mode-aware security assertions.

Files:
- `api/src/config.rs`
- `api/src/server_fns/*`

---

## PR 5 - Docker Image Size Optimization (Medium)

### 5.1 Baseline and identify largest layers
- [ ] Compare `docker history` between local image and upstream reference image.
- [ ] Export and inspect rootfs size breakdown (`/opt/venv` and `/app/server` expected hotspots).
- [ ] Capture before/after image sizes as acceptance criteria.

Commands:
- `docker history soulbeet:download_history`
- `docker history docccccc/soulbeet:latest`
- `docker export <container> | tar -C /tmp/sb_rootfs -xf -`
- `du -h -d3 /tmp/sb_rootfs/opt/venv`
- `du -h -d3 /tmp/sb_rootfs/app/server`

### 5.2 Reduce Python venv footprint in runtime image
- [ ] Remove non-runtime Python packaging artifacts (`pip`, `setuptools`, `wheel`) from copied venv.
- [ ] Remove `__pycache__` and `*.pyc` files in builder stage before runtime copy.
- [ ] Re-verify `beet --version` works in final image.

Files:
- `Dockerfile`

### 5.3 Reduce Dioxus/runtime artifact footprint
- [ ] Verify exactly which files under `/app/target/dx/web/release/web` are required at runtime.
- [ ] Copy only required artifacts (server binary + required static assets), avoid broad directory copy.
- [ ] Re-test app startup and static asset loading.

Files:
- `Dockerfile`

---

## Hold - refactor/search-results

- [ ] Create branch `refactor/search-results` for search quality overhaul (on hold).
- [ ] Review ranking strategy and candidate quality for difficult/rare albums.
- [ ] Add typo-tolerance strategy for user input normalization/correction (investigate MusicBrainz-assisted candidate expansion vs local fuzzy matching).
- [ ] Investigate and fix degradation when optional artist input is provided (artist filter can currently reduce relevance).

---

## Backlog - History Follow-ups

- [ ] Build retry flow in branch `feat/retry` (new attempt semantics + endpoint + UI).
- [ ] Build cancel flow in branch `feat/cancel` (endpoint + UI + terminal state handling).
