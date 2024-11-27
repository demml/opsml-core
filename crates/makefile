.PHONY: test.storage.gcs.http
test.storage.gcs:
	TEST_NAME=test_filesystemstorage_with_http_google  ./test.sh

.PHONY: test.storage.aws.http
test.storage.aws:
	TEST_NAME=test_filesystemstorage_with_http_aws  ./test.sh

.PHONY: test.storage.local.http
test.storage.local:
	TEST_NAME=test_filesystemstorage_with_http_local  ./test.sh

.PHONY: test.storage.azure.http
test.storage.azure:
	TEST_NAME=test_filesystemstorage_with_http_azure  ./test.sh

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
