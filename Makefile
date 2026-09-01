ROOT := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))

.PHONY: help test test-ui test-ui-setup check web serve publish art clean

## test: run the engine suite (native, no browser needed)
test:
	@cargo test --workspace

## check: fast type-check, no binaries produced
check:
	@cargo check --workspace --all-targets

## test-ui: drive the built page in a real browser
test-ui: web
	@$(ROOT)/.venv-test/bin/python $(ROOT)/testing/drive.py

## test-ui-setup: one-time install of headless Chromium for test-ui
test-ui-setup:
	@python3 -m venv $(ROOT)/.venv-test
	@$(ROOT)/.venv-test/bin/pip -q install playwright
	@$(ROOT)/.venv-test/bin/playwright install chromium
	@echo "ready: make test-ui"

## web: build the browser app into dist/web/
web:
	@$(ROOT)/packaging/package-web.sh

## serve: build and open the app locally
serve: web
	@echo "Serving http://localhost:8080/ - Ctrl-C to stop"
	@(sleep 1 && open http://localhost:8080/) >/dev/null 2>&1 &
	@cd $(ROOT)/dist/web && python3 -m http.server 8080

## publish: push to GitHub; Actions builds and publishes to Pages (humans only)
# Only a human runs this. The agent never pushes - see CLAUDE.md.
publish:
	@cargo test --workspace --quiet
	@git push
	@echo "Pushed. Actions builds and publishes; live in about two minutes."
	@echo "Watch: gh run watch"

## art: compile the TikZ sources in art/ to SVGs in web/assets/ (M6)
art:
	@$(ROOT)/packaging/build-art.sh

## clean: remove build output
clean:
	@rm -rf $(ROOT)/dist $(ROOT)/target

help:
	@grep -hE '^## ' $(MAKEFILE_LIST) | sed 's/## //' | awk -F': ' '{printf "  \033[1m%-14s\033[0m %s\n", $$1, $$2}'
