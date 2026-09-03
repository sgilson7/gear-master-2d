ROOT := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))

.PHONY: help test test-ui test-ui-setup check web serve publish art play dress read clean

## test: run the engine suite (native, no browser needed)
test:
	@cargo test --workspace

## check: fast type-check, no binaries produced
check:
	@cargo check --workspace --all-targets

## test-ui: walk the deploy gate in all three browser engines
test-ui: web
	@$(ROOT)/.venv-test/bin/python $(ROOT)/testing/drive.py chromium firefox webkit

## test-ui-setup: one-time install of headless Chromium for test-ui
test-ui-setup:
	@python3 -m venv $(ROOT)/.venv-test
	@$(ROOT)/.venv-test/bin/pip -q install playwright
	@$(ROOT)/.venv-test/bin/playwright install chromium firefox webkit
	@echo "ready: make test-ui"

## play: play the demo start to finish and write down what it said
#
# Not the gate. `drive.py` walks a route chosen to exercise checks; this starts
# a new game, buys with the money it has, packs with the button a player is
# given, and reads every screen. The transcript is the point — it caught an
# Auto-pack that seated the starting kit for the whole game and a class fork
# that opened underneath the town, and both were green in the suite.
play: web
	@$(ROOT)/.venv-test/bin/python $(ROOT)/testing/playthrough.py chromium

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

## dress: search the catalogue for a creature that rates near RATING
##
## The authoring bench, `crates/lab`, and it is not shipped — the wasm crate
## does not depend on it. Both of the original's guarantees come with it:
## monsters wear the catalogue, and a spec whose gear does not assemble is a
## typo rather than a difficulty.
##
##     make dress RATING=1200 ONLY=helmet,chest SKIP=8
dress:
	@RATING=$(RATING) SLOTS=$(SLOTS) ONLY=$(ONLY) SKIP=$(SKIP) PER=$(PER) cargo run -q -p gm2d-lab --bin dress -- dress

## read: print an existing creature's board, its items and its rating
##
##     make read NAME="Cog Priest"
read:
	@cargo run -q -p gm2d-lab --bin dress -- read $(NAME)

## art: compile art/*.tex to web/assets/*.svg (needs pdflatex + pdftocairo)
##
## The SVGs are checked in, so a deploy never runs this — it is what you run
## after editing a figure. Missing LaTeX prints what to install and exits 0.
art:
	@$(ROOT)/packaging/build-art.sh

## clean: remove build output
clean:
	@rm -rf $(ROOT)/dist $(ROOT)/target

help:
	@grep -hE '^## ' $(MAKEFILE_LIST) | sed 's/## //' | awk -F': ' '{printf "  \033[1m%-14s\033[0m %s\n", $$1, $$2}'
