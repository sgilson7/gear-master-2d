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


def experts() -> tuple[dict, dict]:
    """The expert papers: name -> (slug, TeX defines), and a colour a class.

    **One drawing, twenty-one colourways**, which is the creature-family
    argument applied to a thing that is literally a pair: an expert is what two
    finished trees reach, so the figure is the paper and the two seals at the
    foot of it are the two parents. Nothing about the sheet changes.
    """
    raw = json.loads((ROOT / "art" / "experts.json").read_text())
    cols = {k: v for k, v in raw["_classes"].items() if not k.startswith("_")}
    out = {}
    for name, pair in raw["experts"].items():
        a, b = pair
        if a not in cols or b not in cols:
            raise SystemExit(f"art/experts.json: {name} names a class with no colour")
        out[name] = (f"expert-{slug(name)}",
                     f"\\def\\SealA{{{cols[a]}}}\\def\\SealB{{{cols[b]}}}")
    return out, cols


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
        for _, (out, defs) in experts()[0].items():
            print(f"{out}\texpert\t{defs}")
        return 0

    if args.write_map:
        path = pathlib.Path(args.write_map)
        # Read, replace one key, write. `places`, `player` and `classes` are
        # written by a person and must survive: this script owns the creature
        # half of the file and nothing else.
        art = json.loads(path.read_text(), object_pairs_hook=__import__("collections").OrderedDict)
        art["creatures"] = {
            name: (spec["family"]
                   if not any(k in spec for k in ("main", "dark", "accent"))
                   else slug(name))
            for name, spec in m.items()
        }
        # **The expert half of `classes`, and only that half.** The seven a
        # player picks from are hand-drawn and hand-mapped; the twenty-one are
        # colourways and are written from the manifest, so the map and the
        # files it names cannot drift.
        cls = art.setdefault("classes", {})
        for name, (out, _) in experts()[0].items():
            cls[name] = out
        path.write_text(json.dumps(art, indent=2) + "\n")
        return 0

    ap.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
