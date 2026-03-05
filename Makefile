.PHONY: test-migrations test-api test-fast test-full

test-migrations:
	./scripts/test_migrations.sh

test-api:
	cargo test -p api --features server --test auth_history_search -- --nocapture
	cargo test -p api --features server --test history_fk_regressions -- --nocapture

test-fast: test-migrations test-api

test-full: test-fast
	npm run test:e2e
