#!/usr/bin/env bash
# Build the browser app into dist/web/.
#
# No node, no npm, no bundler: the page is a hand-written ES module and
# wasm-bindgen emits the only generated file. Copied from the house pattern in
# sgilson7/pdf-redactor and sgilson7/perturbation-workbench, which agree with
# each other; gear-master's own web build predates both and ships macroquad.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"; WEB="$DIST/web"
CRATE=gm2d-wasm
WASM=gm2d_wasm
TARGET=wasm32-unknown-unknown

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

rustup target list --installed 2>/dev/null | grep -q "^$TARGET$" \
  || die "missing target. Run: rustup target add $TARGET"

# wasm-bindgen-cli and the wasm-bindgen crate must be the same version or the
# generated glue will not match the module. Read the version the lockfile
# actually pinned rather than trusting whatever happens to be installed.
NEED=$(awk '/^name = "wasm-bindgen"$/{f=1} f&&/^version = /{gsub(/"/,"");print $3;exit}' "$ROOT/Cargo.lock")
command -v wasm-bindgen >/dev/null \
  || die "wasm-bindgen not found. Run: cargo install wasm-bindgen-cli --version $NEED"
HAVE=$(wasm-bindgen --version | awk '{print $2}')
[ "$HAVE" = "$NEED" ] \
  || die "wasm-bindgen $HAVE installed but the lockfile pins $NEED.
  Run: cargo install wasm-bindgen-cli --version $NEED --force"

say "Building $CRATE for $TARGET"
cargo build --release --target "$TARGET" -p "$CRATE"

say "Assembling $WEB"
rm -rf "$WEB"; mkdir -p "$WEB"
cp -R "$ROOT/web/." "$WEB/"
rm -rf "$WEB/pkg"

# The page fetches data/art.json at startup — the one data file it reads
# directly, because a name-to-filename lookup has no rule in it and does not
# need to cross the wasm boundary. Everything else in data/ is compiled in.
mkdir -p "$WEB/data"
cp "$ROOT/data/art.json" "$WEB/data/"

say "Generating JS bindings"
wasm-bindgen --target web --no-typescript --out-dir "$WEB/pkg" \
  "$ROOT/target/$TARGET/release/$WASM.wasm"

if command -v wasm-opt >/dev/null; then
  say "Optimising wasm"
  wasm-opt -Oz -o "$WEB/pkg/${WASM}_bg.wasm" "$WEB/pkg/${WASM}_bg.wasm"
fi

# --- cache busting -----------------------------------------------------------
# GitHub Pages serves everything with `Cache-Control: max-age=600`, so for ten
# minutes after a deploy a reload keeps using the previous app.js and .wasm
# from disk cache. That reads as "the fix did not deploy", and worse, a browser
# can mix a fresh script with a stale module. Stamping the content hash into
# every internal asset URL means a changed build is simply a different URL.
say "Stamping build version"
sha256() { if command -v shasum >/dev/null; then shasum -a 256; else sha256sum; fi }
bust() { S="$2" R="$3" perl -0777 -pi -e 's/\Q$ENV{S}\E/$ENV{R}/g' "$1"; }

# **Everything the browser caches**, whatever it is called. Two holes have been
# found in this line by hand:
#
#   1. Listing the modules by name left `shape.js` out when it was added.
#   2. Leaving `index.html` and `styles.css` out meant a markup- or CSS-only
#      change produced an *identical* stamp — so `styles.css?v=…` kept its old
#      URL and was served from cache for ever, and the entry point's self-heal
#      never fired because the stamps matched. A deployed fix that a returning
#      player cannot receive is not a delivered fix.
#
# Hashed before stamping, which is what makes it stable: the `?v=` values and
# `__BUILD__` are written into these files afterwards.
BUILD=$(cat "$WEB"/*.js "$WEB/index.html" "$WEB/styles.css" \
            "$WEB/pkg/$WASM.js" "$WEB/pkg/${WASM}_bg.wasm" | sha256 | cut -c1-8)

bust "$WEB/app.js"       "from './pkg/$WASM.js'" "from './pkg/$WASM.js?v=$BUILD'"

# **Every relative import in every shipped module**, rather than a list of the
# ones somebody remembered. Written out by name twice, and both times the next
# module added went unstamped — `theirs.js` the first time and `shape.js` the
# second, each importing `board.js` two hops from the entry point, which is
# exactly where a stale mix hides because the page looks fresh.
for js in "$WEB"/*.js; do
  perl -0777 -pi -e "s{from '\./([A-Za-z0-9_-]+\.js)'}{from './\$1?v=$BUILD'}g" "$js"
done
bust "$WEB/pkg/$WASM.js" "new URL('${WASM}_bg.wasm', import.meta.url)" \
                         "new URL('${WASM}_bg.wasm?v=$BUILD', import.meta.url)"
bust "$WEB/index.html"   'src="app.js"'      "src=\"app.js?v=$BUILD\""
bust "$WEB/index.html"   'href="styles.css"' "href=\"styles.css?v=$BUILD\""
bust "$WEB/index.html"   '__BUILD__'         "$BUILD"
bust "$WEB/app.js"       '__BUILD__'         "$BUILD"

# Fail loudly rather than shipping a page that silently serves stale assets.
grep -q "app.js?v=$BUILD" "$WEB/index.html" || die "cache-busting did not apply"
grep -q "?v=$BUILD" "$WEB/pkg/$WASM.js"     || die "wasm URL not stamped"
# And nothing anywhere still imports a bare module. This is the check that
# catches the *next* one, rather than the one that was just added.
if grep -RnoE "from '\./[A-Za-z0-9_-]+\.js'" "$WEB"/*.js; then
  die "an import above is unstamped, and will be served stale after a deploy"
fi
grep -q "const BUILD = '$BUILD'" "$WEB/app.js" || die "app.js build stamp not applied"

# Jekyll would otherwise skip the pkg/ directory and mangle assets.
touch "$WEB/.nojekyll"

say "Done: $WEB (build $BUILD)"
du -sh "$WEB" | sed 's/^/    /'
ls -la "$WEB/pkg/${WASM}_bg.wasm" | awk '{printf "    wasm: %.0f KB\n", $5/1024}'
