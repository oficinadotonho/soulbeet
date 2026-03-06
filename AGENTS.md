# AGENTS.md

This file defines mandatory operating rules for agentic contributors in this fork.

## Top Governance Concern: Upstream-Latency Safe Workflow

Upstream review delay or rejection is expected. Fork workflow must remain resilient.

Hard invariant:
- `origin/master` must remain clean and aligned with `upstream/master`.
- Unaccepted feature work must never be merged into `origin/master`.

Required remote setup:
- `origin` points to your fork.
- `upstream` points to the canonical upstream repo.
- Run `git fetch upstream` before graph checks.

## Branch Roles (Enforceable)

- `master`
  - Mirror branch only.
  - Must match `upstream/master` before creating new feature branches.
  - No direct feature merges.

- `feature/*`
  - One concern per PR.
  - Preferred branch type.
  - Base branch: latest `upstream/master`.

- `stack/*`
  - Allowed only when dependency is unavoidable.
  - Used for child PRs that depend on an unmerged parent PR.

- `integration/pending-prs`
  - Local/fork test integration branch only.
  - Never a PR base branch.
  - Can aggregate unmerged PR work for personal testing.

## Mandatory Rules

1. Never merge unaccepted PR branches into `origin/master`.
2. Prefer independent PRs over stacked PRs.
3. Every dependent PR must declare `Depends on: #PR-A` in PR description.
4. Every stacked PR must include a de-stack plan in the PR description.
5. If a parent PR stalls or is rejected, child work must be rebased/cherry-picked onto latest `upstream/master`.
6. Open PR branches must be rebased regularly on upstream changes.
7. Signed commits are required for new PR work.
8. Commit identity must be your user identity, not `Codex <codex@local>`.

## Standing Milestone Checkpoint: PR Dependency Risk Control

For every milestone and PR batch, run all three checks:

1. Clean graph check
- Local `master` equals `upstream/master`.
- PR branch has expected base (`upstream/master` for independent PRs).

2. Dependency transparency check
- No hidden branch dependency.
- Stacked PRs are explicitly labeled and linked.

3. De-stack readiness check
- Child PR contains concrete fallback path if parent is rejected.
- Cherry-pick replay onto clean base is documented and feasible.

## Required Pre-PR Quality Gates

1. Graph safety
- `make check-fork-graph`

2. Code/test quality (strict)
- `make test-full`
- Docker smoke build + container startup verification in real environment.

3. Scope quality
- PR diff contains only intended concern.
- No unrelated refactors or formatting churn.

## Monthly Fork Hygiene Audit

Run:
- `make fork-hygiene`

Required outcomes:
- prune stale integration branches,
- verify active branches map to active PRs,
- confirm no feature merge leaked into `master`.

## PR Description Required Sections

Every PR must include:
- `Summary`
- `Scope`
- `Depends on` (or `Depends on: none`)
- `De-stack plan` (required for `stack/*`, otherwise `not needed`)
- `Validation` (tests + manual checks)
- `Risk and rollback`

## Prohibited Patterns

- Long-lived mega branches containing multiple concerns.
- Using `master` as an integration sandbox.
- Opening stacked PRs without explicit dependency metadata.
- Force-pushing rewritten history to shared branches without explicit intent.
