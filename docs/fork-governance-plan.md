# Fork Governance Plan: Upstream-Latency Safe Workflow

## Summary

This plan defines how this fork stays maintainable when upstream PR review is slow or PRs are rejected.

Primary objective:
- Keep `master` clean and upstream-aligned.
- Isolate unmerged work to feature/stack/integration branches.
- Preserve ability to detach or replay work quickly.

Prerequisite setup:
- `origin` = your fork, `upstream` = canonical repo.
- Always refresh refs with `git fetch upstream` before branch/PR checks.

## Operating Model

### Core branch policy

- `master` mirrors `upstream/master`.
- `feature/*` is default for one-concern PRs.
- `stack/*` is used only for unavoidable dependent PRs.
- `integration/pending-prs` is local/fork-only testing aggregation, never PR base.

### Dependency policy

- Independent PRs are preferred.
- Dependent PRs must declare `Depends on: #PR-A`.
- Every dependent PR must include a de-stack plan.
- If parent stalls/rejects, child is replayed on latest `upstream/master`.

### Quality and safety policy

- Signed commits required.
- Strict validation gate before opening PR.
- Branches rebased regularly to prevent long-lived drift.

## Milestone Structure

### Milestone 0: Governance Baseline

Deliverables:
- Root `AGENTS.md` with enforceable fork policy.
- Graph/hygiene scripts and `make` targets.
- PR template expectations documented.

Exit criteria:
- Contributor workflow is defined and executable.
- `master` safety constraints enforced by habit and checks.

### Milestone 1+: Feature Milestones

For each milestone, include a mandatory checkpoint:

1. PR dependency risk control
- Clean graph check.
- Dependency transparency check.
- De-stack readiness check.

2. Implementation and validation
- Strict test suite and Docker smoke validation.
- PR contains one concern and clear rollback note.

## Required Checks

Before opening any PR:

1. `make check-fork-graph`
- confirms clean base and dependency visibility.

2. `make test-full`
- strict automated gate.

3. Docker smoke in real test env
- image build,
- container startup,
- targeted manual scenario checks.

Before monthly maintenance closeout:

- `make fork-hygiene`
- prune stale integration branches,
- verify branch-to-PR mapping,
- verify no feature commits leaked into `master`.

## De-Stack Plan Template (for stacked PRs)

Each stacked PR must include:

- Parent PR reference
- Child commit set summary
- Replay command sketch (cherry-pick list or sequence)
- Conflict hotspots
- Fallback scope (what can ship if parent is rejected)

## Assumptions and Defaults

- Upstream latency is normal and expected.
- Maintainability and replayability take precedence over short-term speed.
- `master` remains protected and clean by default.
- Dependent development is allowed only in explicit `stack/*` or `integration/*` flows.
