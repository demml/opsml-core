
.PHONY: format
format:
	cargo fmt

.PHONY: lint
lints:
	cargo clippy --workspace --all-targets --features "google_storage, aws_storage" -- -D warnings

.PHONY: test.gcs
test.gcs:
	cargo test --features "google_storage" -- --nocapture  --test-threads=1 

.PHONY: test.aws
test.aws:
	cargo test aws --features "aws_storage" -- --nocapture  --test-threads=1 