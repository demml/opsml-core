.PHONY: test.storage.gcs.http
test.storage.gcs.http:
	cargo test -p opsml-storage test_filesystemstorage_with_http_google -- --nocapture

.PHONY: test.storage.aws.http
test.storage.aws.http:
	cargo test -p opsml-storage test_filesystemstorage_with_http_aws -- --nocapture

.PHONY: test.storage.local.http
test.storage.local.http:
	cargo test -p opsml-storage test_filesystemstorage_with_http_local -- --nocapture

.PHONY: test.storage.azure.http
test.storage.azure.http:
	cargo test -p opsml-storage test_filesystemstorage_with_http_azure -- --nocapture

.PHONY: test.storage.gcs.client
test.storage.gcs.client:
	cargo test -p opsml-storage test_gcs_storage_client -- --nocapture

.PHONY: test.storage.aws.client
test.storage.aws.client:
	cargo test -p opsml-storage test_aws_storage_client -- --nocapture

.PHONY: test.storage.az.client
test.storage.az.client:
	cargo test -p opsml-storage test_azure_storage_client -- --nocapture

.PHONY: test.storage.local.client
test.storage.local.client:
	cargo test -p opsml-storage test_local_storage_client -- --nocapture --test-threads 1

.PHONY: lints
lints:
	cargo clippy --workspace --all-targets -- -D warnings
