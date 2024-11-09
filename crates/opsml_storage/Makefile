
.PHONY: format
format:
	cargo fmt

.PHONY: lint
lints:
	cargo clippy --workspace --all-targets --features "google_storage" -- -D warnings

.PHONY: test
test:
	cargo test --features "google_storage" -- --nocapture  --test-threads=1 