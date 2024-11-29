.PHONY: test.sql.sqlite
test.sql.sqlite:
	cargo test -p opsml-sql test_sqlite -- --nocapture

.PHONY: test.sql.postgres
test.sql.postgres:
	cargo test -p opsml-sql test_postgres -- --nocapture