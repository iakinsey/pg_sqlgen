.PHONY: all build run test clean

all: build

build:
	cargo build

init:
	cargo pgrx init --pg18 download

install:
	cargo pgrx install --release

package:
	./scripts/package.sh

test-packaging: package
	./scripts/test-packaging.sh

test:
	cargo pgrx test
	cargo pgrx regress
	cargo clippy

test-release: lint-sql test package test-packaging

release:
	@[ -n "$(tag)" ] || { echo "tag is required"; exit 1; }
	sed -i "/^\[package\]/,/^\[/ s/^version = \".*\"/version = \"$(tag)\"/" Cargo.toml
	$(MAKE) lint-sql
	$(MAKE) test
	$(MAKE) package
	$(MAKE) test-packaging
	git add Cargo.toml
	git commit -m "$(tag) release"
	git tag v$(tag)


clean:
	cargo clean

serve-docs:
	mdbook serve docs

docs:
	cd docs && mdbook build

lint-sql:
	sqlfluff lint sql-scripts
	sqlfluff lint tests/pg_regress/sql

fix-sql:
	sqlfluff fix sql-scripts
	sqlfluff fix tests/pg_regress/sql
