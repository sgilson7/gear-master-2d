#!/usr/bin/env bash
# Compile art/*.tex to web/assets/*.svg.
#
# Every figure is a standalone TikZ document written by filling in
# tikz_figure_prompt.md, which is the only sanctioned way to make art here. The
# point of that is not ceremony: a figure that is text can be reviewed, diffed
# and corrected in one line, and a figure that is a PNG can only be re-rolled
# and hoped over.
#
# The toolchain is optional. If pdflatex or pdftocairo is missing this says
# exactly what to install or where to compile instead, and exits 0 — a missing
# LaTeX must not fail a build whose game does not need one.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ART="$ROOT/art"; OUT="$ROOT/web/assets"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
note() { printf '\033[1;33mnote:\033[0m %s\n' "$*" >&2; }

shopt -s nullglob
FIGURES=("$ART"/*.tex)
if [ ${#FIGURES[@]} -eq 0 ]; then
  say "no figures in art/"; exit 0
fi

if ! command -v pdflatex >/dev/null || ! command -v pdftocairo >/dev/null; then
  note "pdflatex and/or pdftocairo not found, so nothing was compiled."
  note "Either install a TeX distribution and poppler:"
  note "    brew install --cask basictex && brew install poppler"
  note "    tlmgr init-usertree && tlmgr --usermode install standalone"
  note "or paste each art/*.tex into Overleaf, recompile, and drop the SVG"
  note "into web/assets/ under the same name. The game ships the SVGs, not"
  note "the sources, so it does not need this to build."
  exit 0
fi

# standalone.cls is not in BasicTeX and is what every figure's first line asks
# for. Checked here so the failure is one sentence rather than forty lines of
# TeX telling you a file was not found.
if ! kpsewhich standalone.cls >/dev/null 2>&1; then
  note "standalone.cls is missing, which every figure needs. Install it with:"
  note "    tlmgr init-usertree && tlmgr --usermode install standalone"
  exit 0
fi

mkdir -p "$OUT"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# One figure, optionally with a colourway defined on the command line.
#
# `\def` before `\input` is the standard idiom: the definition lands before
# the figure's own `\providecommand`, which then declines to overwrite it. A
# family therefore ships one silhouette and as many palettes as it has
# creatures, and neither is duplicated anywhere.
compile_one() {
  local tex="$1" out="$2" defs="$3" name
  name="$(basename "$out" .svg)"
  if ! pdflatex -interaction=nonstopmode -halt-on-error \
       -output-directory "$WORK" -jobname "$name" \
       "${defs}\\input{$tex}" >"$WORK/$name.out" 2>&1; then
    printf '\033[1;31merror:\033[0m %s did not compile:\n' "$name" >&2
    grep -E '^!' -A 3 "$WORK/$name.out" | head -12 >&2
    exit 1
  fi
  pdftocairo -svg "$WORK/$name.pdf" "$OUT/$name.svg"
}

made=0
for tex in "${FIGURES[@]}"; do
  name="$(basename "$tex" .tex)"
  say "$name"
  compile_one "$tex" "$OUT/$name.svg" ""
  made=$((made + 1))
done

# The creature colourways. Every creature in `enemies.json` has an entry in
# `art/creatures.json`, and a figure of its own comes out of it — a family
# silhouette in that creature's palette. Fifty portraits from thirteen
# drawings, none of them a re-roll of a PNG.
MANIFEST="$ROOT/art/creatures.json"
if [ -f "$MANIFEST" ] && command -v python3 >/dev/null; then
  say "creature colourways"
  while IFS=$'\t' read -r slug family defs; do
    [ -z "$slug" ] && continue
    # A creature drawn straight from a family figure with no palette of its
    # own is that figure; nothing to compile twice.
    if [ -z "$defs" ]; then continue; fi
    compile_one "$ART/$family.tex" "$OUT/$slug.svg" "$defs"
    made=$((made + 1))
  done < <(python3 "$ROOT/packaging/art-manifest.py" --build)
  python3 "$ROOT/packaging/art-manifest.py" --write-map "$ROOT/data/art.json"
  say "data/art.json rewritten from the manifest"
fi

say "Done: $made figures in web/assets/"
du -sh "$OUT" | sed 's/^/    /'
