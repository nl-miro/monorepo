include ../../make/rust.mk

# Local settings; a DATABASE_URL already set in the environment wins.
ifneq ($(origin TEST_DATABASE_URL),undefined)
retailer_sourcing_test_database_url := $(value TEST_DATABASE_URL)
endif
ifndef DATABASE_URL
-include .env
endif
ifneq ($(origin retailer_sourcing_test_database_url),undefined)
TEST_DATABASE_URL := $(retailer_sourcing_test_database_url)
endif
export DATABASE_URL
export TEST_DATABASE_URL

.PHONY: test-postgres serve migrate migrate-redo export-offer-sources export-sitemaps require-database-url

serve:
	cargo run

# Applies pending migrations to DATABASE_URL.
migrate: require-database-url
	diesel migration run

# Reverts and reapplies every migration on DATABASE_URL, which deletes its data.
migrate-redo: require-database-url
	diesel migration redo --all

# Writes every downloaded offer page to DATA_DIR/<retailer>/offer_details/sources.
DATA_DIR ?= ../../../data
export-offer-sources: require-database-url
	cargo run --release --bin export_offer_sources -- $(DATA_DIR)

# Writes each retailer's latest sitemap crawl to DATA_DIR/<retailer>/sitemap:
# its sources, processed tree and consolidated links.
export-sitemaps: require-database-url
	cargo run --release --bin export_sitemaps -- $(DATA_DIR)

require-database-url:
	@test -n "$(DATABASE_URL)" || { echo "DATABASE_URL must be set, e.g. in .env (make .env)"; exit 1; }

.env:
	cp .env.example .env

# Tests use isolated schemas; never reset the selected database.
test-postgres:
	@test -n "$$TEST_DATABASE_URL" || { echo "TEST_DATABASE_URL is required; use a separate test database"; exit 1; }
	cargo test --bin retailer_sourcing postgres_tests -- --ignored
