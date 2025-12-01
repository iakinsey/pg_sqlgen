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
	./scripts/test-packaging.sh

clean:
	cargo clean

serve-docs:
	mdbook serve docs

docs:
	cd docs && mdbook build