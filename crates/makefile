.PHONY: test.storage.gcs
test.storage.gcs:
	TEST_NAME=test_filesystemstorage_with_http_google  ./test.sh

.PHONY: test.storage.aws
test.storage.aws:
	TEST_NAME=test_filesystemstorage_with_http_aws  ./test.sh

.PHONY: test.storage.local
test.storage.local:
	TEST_NAME=test_filesystemstorage_with_http_local  ./test.sh

.PHONY: test.storage.azure
test.storage.azure:
	TEST_NAME=test_filesystemstorage_with_http_azure  ./test.sh

.PHONY: test.storage.gcs.client
test.storage.gcs.client:
	cargo test -p opsml-storage test_gcs_storage_client -- --nocapture

.PHONY: lints
lints:
	cargo clippy --workspace --all-targets -- -D warnings
