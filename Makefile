.PHONY: build ui-build install uninstall release

build: ui-build
	cargo build --release

ui-build:
	cd ui && npm ci && npm run build

install:
	mkdir -p /usr/local/bin
	install -m 755 target/release/bolt /usr/local/bin/bolt

uninstall:
	rm -f /usr/local/bin/bolt

release:
	@VERSION=v$$(grep '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/'); \
	echo "Releasing $$VERSION..."; \
	git tag $$VERSION; \
	git push origin $$VERSION
