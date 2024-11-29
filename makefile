.PHONY: test.storage.gcs.client
test.sql.sqlite:
	cargo test -p opsml-sql test_sqlite -- --nocapture
