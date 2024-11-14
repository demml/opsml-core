
.PHONY: format
format:
	cargo fmt

.PHONY: lint
lints:
	cargo clippy --workspace --all-targets --features "google_storage, aws_storage" -- -D warnings

.PHONY: test.aws
test.aws:
	cargo test aws --features "aws_storage" -- --nocapture 

.PHONY: test.gcs
test.gcs:
	cargo test google --features "google_storage" -- --nocapture 