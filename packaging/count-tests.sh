#!/usr/bin/env bash
# How many tests there are, counted so the answer is the same twice.
#
# **`cargo test`'s own output cannot be summed.** Test binaries and their
# threads write to one stream, so a summary line gets cut in half —
#
#     test result: ok. 28 passtest every_place_on_a_shot_map... ... ok
#
# — and totting up the fourth field gives a different answer every time the
# regex changes: 978, 1,025 and 1,053 came off one run. A suite figure was
# published three times as 1,137 and could never be got back.
#
# So the binaries are asked one at a time, with `--list`, which prints one line
# per test and no summary to garble. Nothing is executed: this counts, and
# `cargo test` decides whether they pass.
set -euo pipefail
cd "$(dirname "$0")/.."
PKG="${1:-gm2d-core}"

# `--no-run` builds them and `--message-format=json` says where they are, which
# is the one part of cargo's output that is machine-readable by design.
# `bash` on this machine is 3.2 and has no `mapfile`; a while-read loop is the
# portable shape and does not need one.
BINS="$(cargo test -p "$PKG" --no-run --message-format=json 2>/dev/null |
  python3 -c '
import json, sys
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("profile", {}).get("test") and m.get("executable"):
        print(m["executable"])
')"

total=0
count=0
while IFS= read -r b; do
  [ -z "$b" ] && continue
  n="$("$b" --list 2>/dev/null | grep -cE ': test$' || true)"
  total=$(( total + n ))
  count=$(( count + 1 ))
done <<< "$BINS"
printf '%s: %d tests in %d binaries\n' "$PKG" "$total" "$count"
