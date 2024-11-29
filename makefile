.PHONY: test.sql.sqlite
test.sql.sqlite:
	cargo test -p opsml-sql test_sqlite -- --nocapture

.PHONY: test.sql.postgres
test.sql.postgres:
	cargo test -p opsml-sql test_postgres -- --nocapture

.PHONY: test.sql.mysql
test.sql.mysql:
	cargo test -p opsml-sql test_mysql -- --nocapture

.PHONY: build.postgres
build.postgres:
	docker-compose down
	docker-compose up -d --build postgres

.PHONY: build.mysql
build.mysql:
	docker-compose down
	docker-compose up -d --build mysql

.PHONY: build
format:
	cargo fmt --all

.PHONY: lints
lints:
	cargo clippy --workspace --all-targets -- -D warnings
