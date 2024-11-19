
.PHONY: format
format:
	cargo fmt

.PHONY: lint
lints:
	cargo clippy --workspace --all-targets --features "google_storage, aws_storage" -- -D warnings

.PHONY: test.file
test.file:
	cargo test file -- --nocapture 