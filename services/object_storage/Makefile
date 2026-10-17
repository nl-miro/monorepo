include ../../make/rust.mk

# Preserve shell/command-line settings, including explicitly empty values.
ifneq ($(origin OBJECE_STORAGE__DATABASE_URL),undefined)
object_storage_database_url := $(value OBJECE_STORAGE__DATABASE_URL)
endif
ifneq ($(origin OBJECE_STORAGE_DATABASE_URL_TEST),undefined)
object_storage_test_database_url := $(value OBJECE_STORAGE_DATABASE_URL_TEST)
endif
include $(wildcard .env)
ifneq ($(origin object_storage_database_url),undefined)
OBJECE_STORAGE__DATABASE_URL := $(object_storage_database_url)
endif
ifneq ($(origin object_storage_test_database_url),undefined)
OBJECE_STORAGE_DATABASE_URL_TEST := $(object_storage_test_database_url)
endif
export OBJECE_STORAGE__DATABASE_URL
export OBJECE_STORAGE_DATABASE_URL_TEST

.PHONY: serve migrate migrate-redo migration migration-redo require-database-url test-postgres
serve:
	cargo run

migrate: require-database-url
	DATABASE_URL="$$OBJECE_STORAGE__DATABASE_URL" diesel migration run

migrate-redo: require-database-url
	DATABASE_URL="$$OBJECE_STORAGE__DATABASE_URL" diesel migration redo --all

migration: migrate

migration-redo: migrate-redo

require-database-url:
	@test -n "$$OBJECE_STORAGE__DATABASE_URL" || { echo "OBJECE_STORAGE__DATABASE_URL must be set, e.g. in .env (make .env)"; exit 1; }

# Explicitly fails if the test database is not configured or is unreachable.
test-postgres:
	@test -n "$$OBJECE_STORAGE_DATABASE_URL_TEST" || { echo "OBJECE_STORAGE_DATABASE_URL_TEST is required; use a separate test database"; exit 1; }
	cargo test --test postgres -- --ignored

.env:
	cp .env.example .env
