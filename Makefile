.PHONY: test-unit
test-unit:
	cargo test --lib

.PHONY: test-integration
test-integration:
	# Disable parallelism as this causes concurrency issues on Windows when uv cache is accessed.
	cargo test --test '*' -- --test-threads 1

.PHONY: test
test: test-unit test-integration

.PHONY: doc-serve
doc-serve:
	uv run --only-group docs zensical serve
