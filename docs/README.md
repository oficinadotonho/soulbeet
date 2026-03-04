# Soulbeet Agent Docs Index

Purpose: compact, source-grounded context for agentic feature work in this repository.

## Recommended reading order

1. [`architecture.md`](architecture.md) — workspace layout, runtime composition, service boundaries.
2. [`request-and-data-flow.md`](request-and-data-flow.md) — end-to-end user/request/download/import flows.
3. [`modules-and-responsibilities.md`](modules-and-responsibilities.md) — module ownership map by crate/file.
4. [`domain-models-and-interfaces.md`](domain-models-and-interfaces.md) — core data contracts + trait/public API surfaces.
5. [`config-build-run.md`](config-build-run.md) — env/config behavior, Docker/dev commands, migrations.
6. [`testing-conventions-extension-points.md`](testing-conventions-extension-points.md) — test reality, coding patterns, extension seams, pitfalls.

## Agent usage map

- **Add or modify backend endpoints / auth / settings / DB models**
  - Start: [`modules-and-responsibilities.md`](modules-and-responsibilities.md)
  - Then: [`domain-models-and-interfaces.md`](domain-models-and-interfaces.md), [`config-build-run.md`](config-build-run.md)
- **Change search/download/import behavior**
  - Start: [`request-and-data-flow.md`](request-and-data-flow.md)
  - Then: [`modules-and-responsibilities.md`](modules-and-responsibilities.md), [`testing-conventions-extension-points.md`](testing-conventions-extension-points.md)
- **Add a provider/backend/importer implementation**
  - Start: [`domain-models-and-interfaces.md`](domain-models-and-interfaces.md)
  - Then: [`modules-and-responsibilities.md`](modules-and-responsibilities.md), [`testing-conventions-extension-points.md`](testing-conventions-extension-points.md)
- **Deploy / runtime troubleshooting**
  - Start: [`config-build-run.md`](config-build-run.md)
  - Then: [`request-and-data-flow.md`](request-and-data-flow.md)

## Scope and evidence policy

- Statements in this suite are grounded in current source under `dev/soulbeet/`.
- Where behavior depends on runtime config, docs point to the exact implementing file.
- No speculative future architecture is included.
