.PHONY: test-migrations test-api test-fast test-full check-fork-graph fork-hygiene

test-migrations:
	./scripts/test_migrations.sh

test-api:
	cargo test -p api --features server --test auth_history_search -- --nocapture
	cargo test -p api --features server --test history_fk_regressions -- --nocapture
	cargo test -p api --features server --test pipeline_lifecycle -- --nocapture

test-fast: test-migrations test-api

test-full: test-fast
	npm run test:e2e

check-fork-graph:
	./scripts/check_fork_graph.sh

fork-hygiene:
	./scripts/monthly_fork_hygiene.sh
