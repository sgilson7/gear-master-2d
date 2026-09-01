#!/usr/bin/env python3
"""The creature figures, from one manifest.

`art/creatures.json` says which family drawing each creature is cut from and
what colours it is cut in. Two things read that: the build, which compiles a
figure per creature, and `data/art.json`, which is what the *game* looks a
portrait up in. Deriving the second from the first is the point — the map and
the files it names cannot drift, because only one of them is written by hand.
"""
import argparse, json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def slug(name: str) -> str:
    """A creature's file name. Stable, lowercase, and safe on every filesystem."""
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def manifest() -> dict:
    raw = json.loads((ROOT / "art" / "creatures.json").read_text())
    return {k: v for k, v in raw.items() if not k.startswith("_")}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", action="store_true",
                    help="tab-separated slug, family and TeX defines, for the build")
    ap.add_argument("--write-map", metavar="PATH",
                    help="rewrite data/art.json's creature half from the manifest")
    args = ap.parse_args()
    m = manifest()

    if args.build:
        for name, spec in m.items():
            fam = spec["family"]
            # No palette means the creature *is* that drawing — a-rat, the
            # crimper and the toad were drawn for themselves before the
            # families existed, and re-cutting them would only lose them.
            if not any(k in spec for k in ("main", "dark", "accent")):
                print(f"{fam}\t{fam}\t")
                continue
            defs = "".join(
                f"\\def\\{k.capitalize()}{{{spec[k]}}}"
                for k in ("main", "dark", "accent") if k in spec
            )
            print(f"{slug(name)}\t{fam}\t{defs}")
        return 0

    if args.write_map:
        path = pathlib.Path(args.write_map)
        art = json.loads(path.read_text())
        art["creatures"] = {
            name: (spec["family"]
                   if not any(k in spec for k in ("main", "dark", "accent"))
                   else slug(name))
            for name, spec in m.items()
        }
        path.write_text(json.dumps(art, indent=2) + "\n")
        return 0

    ap.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
