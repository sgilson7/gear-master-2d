"""Drive the built page in real browsers and walk the deploy gate.

`cargo test` cannot reach any of this: whether the wasm instantiates in a
browser, whether the ES module import resolves after cache-busting rewrote the
URLs, whether `<a download>` actually produces a file, and whether
`file.arrayBuffer()` feeds it back in.

The gate is a sequence, not a state, so this walks the whole loop:

    town -> buy -> pack -> walk -> a fight -> the board -> the replay ->
    the receipt -> download -> reload -> upload -> everything is back

and it checks the harder halves at the same time. The fit preview has to come
from core rather than from the page. A save taken mid-fight has to reopen the
same fight. And the tiles walked, the purse, the board and the answered events
all have to cross the file together, because a save that restored the position
and not the stream would put the player back on the same tile facing a
different map.

A console error or a request that leaves the origin fails the run, so "nothing
is uploaded" is tested rather than asserted.

    python testing/drive.py [chromium|firefox|webkit ...]
"""
import functools
import os
import http.server
import socketserver
import sys
import threading
import json
import re
import traceback
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
# The directory to serve. `GM2D_WEB` points the gate at a build of its own —
# see the same variable in `packaging/package-web.sh`. M11.8 put a long-running
# playtest on dist/web, and a gate that rebuilds it swaps the game out from
# under a run in progress.
WEB = Path(os.environ.get("GM2D_WEB") or ROOT / "dist" / "web")
PORT = 8127
# Where to walk. `GM2D_ORIGIN` points the gate at a page that is already
# being served — which in practice means the **live one**. CLAUDE.md has said
# since M8 that a deploy is not finished until somebody loads the live URL and
# asks a player's question of it, and every one of those notes was written by
# hand. This is that step, done by the thing that already knows the questions.
LIVE = (os.environ.get("GM2D_ORIGIN") or "").rstrip("/")
ORIGIN = LIVE or f"http://127.0.0.1:{PORT}"
JSON_NULL = None


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    handler = functools.partial(Quiet, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def close_fight(page):
    """Leave the fight screen however it was opened.

    **Every check that opens it has to close it, on every path out.** A check
    that appended a failure and returned early left the screen over the page,
    and the next check died on a click it could not land — so one real failure
    arrived as a Playwright traceback with the failure list never printed at
    all. `try`/`finally` around the body, and this in the `finally`.
    """
    if page.is_visible("#fight"):
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)


def dismiss_card(page):
    """Close an event card if one opened, so the walk continues."""
    if page.is_visible("#card"):
        if page.is_visible("#card-choices button"):
            page.click("#card-choices button")
        page.wait_for_selector("#card-close", state="visible", timeout=5000)
        page.click("#card-close")
        page.wait_for_selector("#card", state="hidden", timeout=5000)
        return True
    return False


def head_for_town(page):
    """One step towards the nearest town, or None when there is none to take.

    **The patrol cannot get you home.** It is six steps east and six west,
    which finds fights and, from more than six tiles away, never finds the town
    again. That did not matter while a fight levelled you on the spot; it is
    the whole loop now, and it is what made the grind pass on one run and fail
    on the next.
    """
    here = page.evaluate("""() => {
      const c = document.getElementById('coords').textContent.split(',');
      const at = [parseInt(c[0]), parseInt(c[1])];
      const towns = (window.__places() ?? []).filter(p => p.kind === 'town');
      if (!towns.length) return null;
      const d = (t) => Math.abs(t.at[0] - at[0]) + Math.abs(t.at[1] - at[1]);
      towns.sort((a, b) => d(a) - d(b));
      return { at, to: towns[0].at };
    }""")
    if not here:
        return None
    ax, ay = here["at"]
    tx, ty = here["to"]
    if ax == tx and ay == ty:
        return None
    # One axis at a time; a blocked step draws nothing, so a wall just costs a
    # turn of the loop rather than stalling it.
    if ax != tx:
        return "ArrowRight" if tx > ax else "ArrowLeft"
    return "ArrowDown" if ty > ay else "ArrowUp"


# Every banking receipt the walk has seen, in order.
#
# A level lands wherever the walk happens to be standing in a town, which is
# not necessarily where a check is looking — the level-up section used to read
# the receipt of the fight that caused it, and a fight causes none of this now.
# Recording them as they happen is what makes "did a level-up say which frame
# grew" answerable from anywhere.
BANKINGS = []


def bank_here(page):
    """Spend what is carried, if this town will take it. Returns the receipt."""
    if page.is_disabled("#bank"):
        return ""
    page.click("#bank")
    page.wait_for_timeout(90)
    said = page.text_content("#town-says") or ""
    BANKINGS.append(said)
    return said


def leave_town(page, bank=True):
    """Leave, banking what is carried on the way out.

    **A town is the only place experience becomes a level**, so a walk that
    never spends is a walk that stays level one for ever. Banking on the way
    out is what a player does and what this loop has to do.
    """
    if page.is_visible("#town"):
        if bank:
            bank_here(page)
        # **Banking is where a level lands, and a level is where the fork
        # opens** — on top of the town, because it is the one screen that does
        # not come off. Leave it to the caller rather than clicking Back out
        # through it.
        if page.is_visible("#fork"):
            return True
        page.click("#leave")
        page.wait_for_selector("#town", state="hidden", timeout=5000)
        return True
    return False


# East, north, west, north: a serpentine that keeps finding new ground. A path
# that walks into a wall would stall the search, because a blocked step draws
# nothing — which is correct, and would make this loop run forever.
# East and west along the pit's road: where a starting kit can actually win,
# which is where a player grinds and where the pacing was calibrated.
PATROL = ["ArrowRight"] * 6 + ["ArrowLeft"] * 6


def walk_until_a_fight(page, limit=200):
    for i in range(limit):
        if page.is_visible("#fight"):
            return True
        if leave_town(page) or dismiss_card(page):
            continue
        page.keyboard.press(PATROL[i % len(PATROL)])
    return False


# Laid down per tick and reset each fight — never something you are wearing.
PER_ACTIVATION = ["cork", "the funny", "fury", "devotion", "harvest",
                  "idiot mode", "physical damage", "magic damage"]


def check_the_card_halves(page, name, fails):
    """Cork is per activation, not a standing stat.

    Reported from a real session. The card lumped every stat an item carried
    into one "out of combat" list, which said a piece granting 29 cork was
    armour you wear — when it is laid down on each tick and reset every fight.
    Upstream splits the two at the one place a blow is worked out; this checks
    the split rather than trusting it.
    """
    # Only finished items. An item that has not come together is a name and
    # core's sentence about what it is missing, and has no halves to split.
    got = page.evaluate("""() => [...document.querySelectorAll('#made .made-item:not(.short)')].map(c => {
      const heads = [...c.querySelectorAll('.head')].map(h => h.textContent.trim());
      const lists = [...c.querySelectorAll('.stats')].map(
        u => [...u.querySelectorAll('li')].map(li => li.textContent.trim()));
      return { name: c.querySelector('b')?.textContent ?? '?', heads, lists };
    })""")
    if not got:
        return
    for card in got:
        if len(card["heads"]) < 2 or len(card["lists"]) < 2:
            fails.append(f"{name}: {card['name']} has no two halves: {card['heads']}")
            continue
        standing = " ".join(card["lists"][0]).lower()
        for stat in PER_ACTIVATION:
            if stat in standing:
                fails.append(f"{name}: {card['name']} lists {stat!r} as standing still, "
                             f"and it is laid down per activation")
        if "standing" not in card["heads"][0].lower():
            fails.append(f"{name}: the first half is headed {card['heads'][0]!r}")
        if "activation" not in card["heads"][1].lower():
            fails.append(f"{name}: the second half is headed {card['heads'][1]!r}")


def check_hovering_an_item(page, name, fails):
    """Pointing at an item on the board reads it in the panel."""
    got = page.evaluate("""() => {
      const b = window.__board;
      const s = b.state.slots.find(s => s.placed.length && s.items.some(i => i.assembled));
      if (!s) return { skipped: true };
      const item = s.items.find(i => i.assembled);
      b.point({ slot: s.slot, x: item.cells[0][0], y: item.cells[0][1] }, 0, 0);
      const lit = [...document.querySelectorAll('#panel-yours .made-item.pointed')]
        .map(e => e.dataset.key);
      return { key: item.pieces.join(','), lit, ring: b.pointed };
    }""")
    if got.get("skipped"):
        return
    if got["ring"] != got["key"]:
        fails.append(f"{name}: the board rings {got['ring']!r}, not the item under the cursor")
    if got["lit"] != [got["key"]]:
        fails.append(f"{name}: hovering lit {got['lit']!r}, expected [{got['key']!r}]")


def check_their_gear_is_on_the_screen(page, name, fails):
    """You can see what you are about to fight, item by item.

    Reported from a real session. `encounter_json` carried the creature's item
    names from the first milestone and the page printed none of them — which
    for forty-nine of the fifty creatures meant the entire fight was invisible
    before it started, because only the Cave Rat has innate attacks and every
    other one fights purely out of its gear.
    """
    page.click("#tab-theirs")
    wait_for_images(page, "#theirs-art")
    got = page.evaluate("""() => {
      const cards = [...document.querySelectorAll('#theirs-cards .made-item')];
      const c = document.getElementById('theirs-board');
      return {
        shown: !document.getElementById('panel-theirs').hidden,
        cards: cards.length,
        named: cards.map(e => e.querySelector('b')?.textContent ?? ''),
        body: [...document.querySelectorAll('#theirs-body li')].map(li => li.textContent),
        title: document.getElementById('theirs-title').textContent,
        grid: { w: c.width, h: c.height },
        art: (() => { const a = document.getElementById('theirs-art');
                      return a.hidden ? null : a.getAttribute('src'); })(),
      };
    }""")
    if not got["shown"]:
        fails.append(f"{name}: the other side's panel would not open")
    if got["title"] != page.text_content("#fight-name"):
        fails.append(f"{name}: the panel is headed {got['title']!r}, "
                     f"not {page.text_content('#fight-name')!r}")
    if not got["body"]:
        fails.append(f"{name}: nothing said about the creature's own body")
    # A creature with gear must show it; one with none says so rather than
    # printing an empty panel. Which of the two is a property of the creature,
    # so the check is that the two agree.
    # What core says it is wearing, against what the panel drew. Two sources,
    # so the check compares the page's answer with the engine's rather than
    # with itself.
    listed = page.evaluate("() => window.__encounter()?.items.length ?? 0")
    if listed and not got["cards"]:
        fails.append(f"{name}: it is wearing {listed} items and the panel shows none")
    if got["cards"] and got["grid"]["w"] < 40:
        fails.append(f"{name}: it has {got['cards']} items and its board drew "
                     f"{got['grid']['w']}px wide")
    if got["cards"] and not got["art"]:
        fails.append(f"{name}: no portrait for {got['title']!r} — every creature has one")
    page.click("#tab-yours")


def check_the_portrait_shows(page, name, fails):
    """Every creature has a figure, and the figure is on the screen.

    `data/art.json` mapped three creatures out of fifty, so the art compiled by
    `make art` was reachable from almost nowhere. The map is generated from
    `art/creatures.json` now and this is what says so from the browser's side.
    """
    wait_for_images(page, "#fight-art")
    got = page.evaluate("""() => {
      const a = document.getElementById('fight-art');
      return { hidden: a.hidden, src: a.getAttribute('src'),
               w: a.naturalWidth, name: document.getElementById('fight-name').textContent };
    }""")
    if got["hidden"] or not got["src"]:
        fails.append(f"{name}: no portrait drawn for {got['name']!r}")
    elif got["w"] == 0:
        fails.append(f"{name}: {got['name']!r} points at {got['src']}, which did not load")


def check_every_skill_says_what_it_does(page, name, fails):
    """A node states its effect in numbers, unthemed, and explains it on hover.

    Reported from a real session: the tree described itself only in the world's
    words, so "Nine hundred feet of Deep Chocolate mine" was the whole of what
    a player had to decide sixty max health on. The spec is derived from the
    effect in core, never typed, which is also why it cannot be wrong.
    """
    THEMED = ("fnorp", "the funny", "cork", "fury", "devotion", "harvest")
    page.click("#skills")
    page.wait_for_selector("#tree", state="visible", timeout=8000)
    got = page.evaluate("""() => [...document.querySelectorAll('#nodes .wares')].map(b => ({
      name: b.querySelector('b')?.textContent ?? '?',
      spec: b.querySelector('.spec')?.textContent ?? null,
    }))""")
    if not got:
        fails.append(f"{name}: the tree has no nodes")
    for n in got:
        if not n["spec"]:
            fails.append(f"{name}: {n['name']!r} says nothing about what it does")
            continue
        if not any(c.isdigit() for c in n["spec"]):
            fails.append(f"{name}: {n['name']!r} spec {n['spec']!r} names no number")
        for w in THEMED:
            if w in n["spec"].lower():
                fails.append(f"{name}: {n['name']!r} spec is themed: {n['spec']!r}")

    # --- and it is drawn as a tree ------------------------------------------
    #
    # It was one flat rack of buttons, which told you what existed and nothing
    # about what led to what. Rows are depth: nothing on the top row asks for
    # anything first, and every node sits below the deepest thing it needs.
    shape = page.evaluate("""() => {
      const tiers = [...document.querySelectorAll('#nodes .tier')];
      const depth = new Map();
      tiers.forEach((t, d) => t.querySelectorAll('.node').forEach(n => depth.set(n.dataset.node, d)));
      const all = window.__trees();
      const tree = all.trees.find(t => t.id === document.querySelector('#tree-tabs button.on')
                                        ?.dataset.tree) ?? all.trees[0];
      const bad = [];
      for (const n of tree.nodes) {
        const d = depth.get(n.id);
        if (d === undefined) { bad.push(`${n.id} is not drawn`); continue; }
        if (n.requires.length === 0 && d !== 0) bad.push(`${n.id} needs nothing and is on row ${d}`);
        for (const r of n.requires) {
          const p = depth.get(r);
          if (p === undefined) bad.push(`${n.id} requires ${r}, which is not drawn`);
          else if (p >= d) bad.push(`${n.id} is on row ${d}, at or above its prerequisite ${r} on ${p}`);
        }
      }
      return { tiers: tiers.map(t => t.querySelectorAll('.node').length),
               wires: document.querySelectorAll('#nodes .wires path').length,
               edges: tree.nodes.reduce((n, x) => n + x.requires.length, 0),
               bad };
    }""")
    if shape["bad"]:
        fails.append(f"{name}: the tree is not a tree: {shape['bad'][:4]}")
    if len(shape["tiers"]) < 2:
        fails.append(f"{name}: the tree drew {len(shape['tiers'])} row(s), so it is still a list")
    if shape["wires"] != shape["edges"]:
        fails.append(f"{name}: {shape['edges']} prerequisites and {shape['wires']} lines drawn")

    # And hovering one opens the card that explains the words in it.
    page.hover("#nodes .wares")
    page.wait_for_selector("#node-detail", state="visible", timeout=4000)
    detail = page.evaluate("""() => {
      const d = document.getElementById('node-detail');
      const r = d.getBoundingClientRect();
      return { text: d.textContent.trim(), paras: d.querySelectorAll('p').length,
               inside: r.left >= 0 && r.top >= 0
                    && r.right <= innerWidth + 1 && r.bottom <= innerHeight + 1 };
    }""")
    if detail["paras"] < 2:
        fails.append(f"{name}: the hover card explains nothing: {detail['text']!r}")
    if not detail["inside"]:
        fails.append(f"{name}: the hover card is drawn off the edge of the window")
    page.click("#tree-done")
    page.wait_for_selector("#tree", state="hidden", timeout=8000)


def wait_for_images(page, selector, timeout=8000):
    """Let the images under `selector` finish decoding before measuring them.

    `naturalWidth` is 0 until a decode completes, so sampling it the instant a
    screen opens is a race — one that passes on this machine and fails on a
    cold CI runner, which is the worst shape a test can have. It failed two
    deploys before anybody looked at why.
    """
    try:
        page.wait_for_function(
            "(sel) => [...document.querySelectorAll(sel + ' img, ' + sel)]"
            "  .filter(e => e.tagName === 'IMG' && !e.hidden).every(i => i.complete)",
            arg=selector, timeout=timeout)
    except Exception:
        pass


def check_the_barrel_is_under_the_counter(page, name, fails):
    """The barrel: same in every town, never sells out, and cheap.

    **M12.1.** The shelf is the designed curve and this is the floor under it.
    Three things a screen can be wrong about and core cannot: that it is drawn
    at all, that buying from it does not consume the entry, and that it is told
    apart from the shelf.
    """
    entries = page.evaluate("""() => [...document.querySelectorAll('#barrel .wares')].map(b => ({
      name: b.querySelector('b')?.textContent,
      cost: b.querySelector('.cost')?.textContent,
      off: b.disabled,
    }))""")
    if len(entries) < 9:
        fails.append(f"{name}: the barrel shows {len(entries)} entries")
        return
    for e in entries:
        n = "".join(c for c in (e["cost"] or "") if c.isdigit())
        if not n:
            fails.append(f"{name}: a barrel entry costs {e['cost']!r}, which is not a price")
        elif int(n) > 12:
            fails.append(f"{name}: {e['name']} is in the barrel at {n} Fnorp")
        if "Fnorp" not in (e["cost"] or ""):
            fails.append(f"{name}: a barrel price does not say Fnorp: {e['cost']!r}")
    # **Told apart from the shelf, and measured rather than read.** They are
    # two lists of the same buttons one above the other; if they paint the
    # same the section heading is the only difference and that is not enough.
    same = page.evaluate("""() => {
      const a = document.querySelector('#shelf .wares');
      const b = document.querySelector('#barrel .wares');
      if (!a || !b) return null;
      const ga = getComputedStyle(a), gb = getComputedStyle(b);
      return ga.backgroundColor === gb.backgroundColor;
    }""")
    if same is None:
        fails.append(f"{name}: could not compare the shelf and the barrel")
    elif same:
        fails.append(f"{name}: the barrel paints the same as the shelf")

    # **It never sells out.** The shelf greys an entry after you buy it and
    # leaves it in place; the barrel must not, because you took a copy.
    live = page.locator("#barrel .wares:not(:disabled)")
    if live.count() == 0:
        fails.append(f"{name}: nothing in the barrel is affordable")
        return
    bought = live.first.locator("b").text_content()
    shown = "".join(c for c in (live.first.locator(".cost").text_content() or "")
                    if c.isdigit())
    purse = int(page.text_content("#town-gold"))
    live.first.click()
    page.wait_for_timeout(120)
    after = int(page.text_content("#town-gold"))
    if after >= purse:
        fails.append(f"{name}: buying from the barrel cost nothing ({purse} -> {after})")
    elif shown and purse - after != int(shown):
        # §C.3 again. The barrel is the one tier that is *not* marked up, so
        # this is also how a mark-up leaking onto it would be caught.
        fails.append(f"{name}: the barrel said {shown} Fnorp and took {purse - after}")
    still = page.evaluate("""(want) => {
      const b = [...document.querySelectorAll('#barrel .wares')]
        .find(e => e.querySelector('b')?.textContent === want);
      return b ? !b.disabled || b.className.includes('sold') : null;
    }""", bought)
    if still is None:
        fails.append(f"{name}: {bought} left the barrel after it was bought")
    elif still is False:
        fails.append(f"{name}: {bought} greyed out after one purchase; the barrel ran out")
    said = last_said(page)
    if bought and bought not in (said or ""):
        fails.append(f"{name}: buying {bought} said {said!r}, which does not name it")


def check_a_choice_says_what_it_pays(page, name, fails):
    """The outcomes box, and that it agrees with the receipt.

    **M12.5, and the highest-risk thing in the block.** A box is a promise
    printed on a screen, and this project has shipped four promises that
    reached nothing — `Showstopper`, `Recycler`, the ench rack, event
    experience. `PLAN-M12-EXEC.md` §6 entry 7 says to treat any disagreement
    between the box, the receipt and the character's actual state as a **fifth
    instance rather than a rendering bug**, and that is what this asks: read
    the box, take the choice, read the receipt, and compare.
    """
    card = page.evaluate("""() => {
      const box = document.getElementById('card');
      if (!box || box.hidden) return null;
      return [...document.querySelectorAll('#card-choices .choice')].map(b => ({
        label: b.querySelector('b')?.textContent,
        off: b.disabled,
        spec: [...b.querySelectorAll('.outcome li')].map(li => (li.textContent || '').trim()),
        wants: [...b.querySelectorAll('.outcome li.wants')].map(li => (li.textContent || '').trim()),
      }));
    }""")
    if card is None:
        return
    if not card:
        # A card whose choices are spent reopens with none, which is the
        # design: the place may still have an errand on it. Nothing to read
        # here, and that is not a fault.
        return
    for c in card:
        if not c["spec"]:
            fails.append(f"{name}: the choice {c['label']!r} says nothing about what it pays")
        # A locked choice says what it wants as well as what it would pay.
        if c["off"] and not c["wants"]:
            fails.append(f"{name}: {c['label']!r} is refused and never says what would open it")
    themed = ("cork", "nut freeze", "the funny", "semuta")
    for c in card:
        for line in c["spec"]:
            if any(t in line.lower() for t in themed):
                fails.append(f"{name}: an outcomes box says {line!r}, which is the theme's word")

    # **The box against the receipt.** Take a choice that can be taken and
    # compare what it promised with what the game says it did.
    live = [i for i, c in enumerate(card) if not c["off"]]
    if not live:
        return
    i = live[0]
    promised = card[i]["spec"]
    gold_before = int(page.text_content("#gold"))
    page.locator("#card-choices .choice").nth(i).click()
    page.wait_for_timeout(200)
    receipt = page.locator("#card-receipt p").all_text_contents()
    if not receipt:
        fails.append(f"{name}: taking {card[i]['label']!r} produced no receipt at all")
        return
    for line in promised:
        m = re.search(r"([+-]?\d+) Fnorp", line)
        if not m:
            continue
        want = int(m.group(1))
        got = int(page.text_content("#gold")) - gold_before
        if got != want:
            fails.append(f"{name}: the box promised {want} Fnorp and the purse moved {got}")
    # Experience is carried, so the receipt is where it shows.
    for line in promised:
        if "experience" in line and not any("experience" in r for r in receipt):
            fails.append(f"{name}: the box promised {line!r} and the receipt never mentions it")
        if line.startswith("Gained:") and not any("Gained" in r or "Took" in r for r in receipt):
            fails.append(f"{name}: the box promised {line!r} and the receipt does not")


def check_a_grid_says_what_it_takes(page, name, fails):
    """Every grid names its recipe, and an empty grid names it too.

    **M12.B.** `piece::recipe_parts` has read the recipe table since the fork
    and no screen ever printed it for a *grid* — so a player who had not read
    the source had no way to learn that a chest wants a base and one to three
    layers. The M12.0 probe measured what that costs: the greaves grid sits at
    0% for fourteen levels of a whole playthrough.

    The empty grid is the point. The panel used to skip a slot with no items,
    which meant the one place the question is asked was the one place nothing
    was said — so this counts five grids rather than however many are packed.

    **Read off the screen and not out of the page's objects.** The first
    version of this reached for `window.__board.slots`, which is the canvas
    painter rather than the payload, and failed in all three engines on the
    word `undefined`. What is being checked is whether a player can read it.
    """
    got = page.evaluate("""() => ({
      heads: [...document.querySelectorAll('#panel-yours .grid-of')].map(e => e.textContent),
      boxes: [...document.querySelectorAll('#panel-yours .recipe')].map(e => ({
        lines: [...e.querySelectorAll('li')].map(li => (li.textContent || '').trim()),
      })),
    })""")
    want = {"weapon", "helmet", "chest", "gloves", "greaves"}
    heads = set(got["heads"])
    if not want <= heads:
        fails.append(f"{name}: the packing panel names {sorted(heads)}, "
                     f"and never says what {sorted(want - heads)} take")
    if len(got["boxes"]) != len(got["heads"]):
        fails.append(f"{name}: {len(got['heads'])} grid headings and "
                     f"{len(got['boxes'])} recipe boxes")
    # **Not vacuous:** every box carries a count, and the set names more than
    # one kind of part — a box reading "1 thing" everywhere would pass a check
    # that only counted boxes.
    kinds = set()
    for head, box in zip(got["heads"], got["boxes"]):
        if not box["lines"]:
            fails.append(f"{name}: the {head} grid has a recipe box with nothing in it")
            continue
        for line in box["lines"]:
            if not any(c.isdigit() for c in line):
                fails.append(f"{name}: the {head} recipe reads {line!r}, which names no count")
            kinds.update(w for w in line.replace("+", " ").split() if w.isalpha())
    for wanted in ("handle", "base", "mold", "plating"):
        if not any(wanted in k for k in kinds):
            fails.append(f"{name}: no recipe mentions a {wanted}: {sorted(kinds)}")
    # A themed word here would be TONE 13a broken on a new screen.
    themed = {"cork", "funny", "fnorp", "roast"}
    for k in kinds:
        if k.lower() in themed:
            fails.append(f"{name}: a recipe says {k!r}, which is the theme's word for it")


def check_the_frozen_save_is_playable(page, name, fails):
    """A player's save that used to trap the module, loaded and walked.

    **M12.B, and it is the reported bug rather than a reconstruction of it.**
    `quest::guide` asked all eleven maps whether a crossing stood between the
    player and an errand and handed each of them `world.at` — a position that
    belongs to one map. At (4, 16) on the 20x20 field, the 16x16 Treyway was
    asked for index 260 of 256 and the wasm trapped, so the page logged
    `unreachable` and the game stopped taking input.

    **Check the second keypress.** One press was always going to work, because
    the trap is sprung by whatever reads the quest log after it. A check that
    plants this save and steps once is the check that would have passed.
    """
    fixture = ROOT / "crates" / "core" / "tests" / "fixtures" / "frozen-on-the-field.json"
    if not fixture.exists():
        fails.append(f"{name}: the reported save is missing from {fixture}")
        return
    errors = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.set_input_files("#file", str(fixture))
    try:
        page.wait_for_function(
            "() => Array.from(document.querySelectorAll('#tape li'))"
            ".some(e => (e.textContent || '').includes('Loaded'))", timeout=10000)
    except Exception:
        said = page.eval_on_selector_all("#tape li", "e => e.map(x => x.textContent)")
        fails.append(f"{name}: the reported save never loaded. The strip says {said[-1:]!r}")
        return
    where = json.loads(page.evaluate("() => window.__position()"))
    if where["map"] != "kettleworks-field":
        fails.append(f"{name}: the save is on the field and the game opened on {where['map']!r}")
    # The log is what trapped, so read it before anything else.
    try:
        page.evaluate("() => window.__log()")
    except Exception as e:
        fails.append(f"{name}: reading the quest log threw {str(e).splitlines()[0][:60]}")
    walked = []
    for key in ("ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"):
        # **Close a card without answering it.** The field is thick with events
        # since M12.5 and one of them is a warp: `dismiss_card` takes the first
        # choice, which on the cistern is *go down*, and this check would end
        # up under the lake proving nothing. Close only.
        if page.is_visible("#card"):
            page.click("#card-close")
            page.wait_for_selector("#card", state="hidden", timeout=5000)
        page.click("#map")
        page.keyboard.press(key)
        page.wait_for_timeout(180)
        walked.append(page.text_content("#coords"))
    if page.is_visible("#card"):
        page.click("#card-close")
        page.wait_for_selector("#card", state="hidden", timeout=5000)
    if len(set(walked)) < 2:
        fails.append(f"{name}: four presses and the character stood still: {walked}")
    if errors:
        fails.append(f"{name}: the reported save still throws: {errors[:2]}")


def check_the_shelf_is_the_shelf(page, name, fails):
    """A town sells what it sells, and what you buy is gone from it.

    The shop used to be one randomised stock for the whole world with a reroll
    button, which made three towns one slot machine in three costumes. Stock is
    `data/shops.json` now: what has to hold is that leaving and coming back
    finds the same shelf, minus what you took.
    """
    if page.query_selector("#reroll"):
        fails.append(f"{name}: the reroll button is still there")
    before = page.evaluate("""() => [...document.querySelectorAll('#shelf .wares')].map(b => ({
      name: b.querySelector('b').textContent,
      sold: b.classList.contains('sold'),
    }))""")
    if not before:
        fails.append(f"{name}: the town sells nothing")
        return
    buyable = next((i for i, w in enumerate(before) if not w["sold"]
                    and not page.locator("#shelf .wares").nth(i).is_disabled()), None)
    if buyable is None:
        fails.append(f"{name}: nothing on the shelf is affordable")
        return
    page.locator("#shelf .wares").nth(buyable).click()
    page.click("#leave")
    page.wait_for_selector("#town", state="hidden", timeout=8000)
    # Back in. Out one tile first: leaving a town leaves you standing on it,
    # and the starting town is against the western edge — walking further west
    # is a blocked step, which by design draws nothing and moves nobody.
    def step(key):
        page.keyboard.press(key)
        if page.is_visible("#fight"):
            page.click("#run")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
        dismiss_card(page)

    step("ArrowRight")
    for _ in range(12):
        if page.is_visible("#town"):
            break
        step("ArrowLeft")
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not get back into the town")
        return
    after = page.evaluate("""() => [...document.querySelectorAll('#shelf .wares')].map(b => ({
      name: b.querySelector('b').textContent,
      sold: b.classList.contains('sold'),
    }))""")
    if [w["name"] for w in before] != [w["name"] for w in after]:
        fails.append(f"{name}: the shelf turned over on its own")
    if not after[buyable]["sold"]:
        fails.append(f"{name}: {after[buyable]['name']!r} was bought and is still for sale")
    resold = [w["name"] for i, w in enumerate(after) if w["sold"] and i != buyable
              and not before[i]["sold"]]
    if resold:
        fails.append(f"{name}: entries sold themselves while you were out: {resold}")


def check_the_errand_board(page, name, fails):
    """The starter town asks for something, and taking it on says what.

    The ask is derived from the goal and unthemed — a count and a creature —
    for the same reason a skill node's is: somebody deciding whether to walk
    four streets for it is reading a number.
    """
    got = page.evaluate("""() => [...document.querySelectorAll('#quests .wares')].map(b => ({
      name: b.querySelector('b')?.textContent ?? '',
      asks: b.querySelector('.spec')?.textContent ?? '',
      foot: b.querySelector('.cost')?.textContent ?? '',
      pays: b.querySelector('.meta')?.textContent ?? '',
    }))""")
    if not got:
        fails.append(f"{name}: the starting town has no errand")
        return
    q = got[0]
    if not any(c.isdigit() for c in q["asks"]):
        fails.append(f"{name}: the errand asks {q['asks']!r}, which names no number")
    if not q["pays"].strip():
        fails.append(f"{name}: the errand says nothing about what it pays")
    if q["foot"].strip().lower() != "take it on":
        fails.append(f"{name}: an untaken errand reads {q['foot']!r}")
    # The first one you can actually act on here. An errand taken in a field
    # and reported in town shows in both places and is only clickable in one.
    live = page.locator("#quests .wares:not(:disabled)")
    if live.count() == 0:
        fails.append(f"{name}: the starting town offers nothing you can take")
        return
    live.first.click()
    foot = page.locator("#quests .wares .cost").first.text_content()
    if "of" not in foot:
        fails.append(f"{name}: after taking it the errand reads {foot!r}, not a tally")

    # And handing in early is refused with a sentence rather than a silence.
    # The grind to five toads is not worth walking here — the whole loop is in
    # `tests/quests.rs` — but the button being wired to the answer is.
    page.locator("#quests .wares:not(:disabled)").first.click()
    said = page.text_content("#town-says") or ""
    if not said.strip():
        fails.append(f"{name}: handing in an unfinished errand said nothing")
    elif "of" not in said:
        fails.append(f"{name}: the refusal does not say how far along you are: {said!r}")


def check_the_replay_shows_both_sides(page, name, fails):
    """Both boards tick, and pointing at a row reads that item.

    Only the Cave Rat has innate attacks — every other creature fights purely
    out of its gear — so a replay showing one side's cooldowns was showing half
    the fight with no way to tell which half.
    """
    got = page.evaluate("""() => {
      const r = window.__replay, log = r.log;
      const rows = (id) => [...document.querySelectorAll(`#${id} .tick`)].map(e => ({
        name: e.querySelector('.tick-name').textContent,
        card: e.classList.contains('has-card'),
      }));
      const last = log.entries[log.entries.length - 1] ?? {};
      return {
        you: rows('ticks-you'), them: rows('ticks-them'),
        wantYou: log.player.items.length, wantThem: log.enemy?.items?.length ?? 0,
        // The four pools and both armours have to arrive as numbers, whatever
        // this particular fight happened to bank.
        shape: ['pa', 'ea'].every(k => typeof last[k] === 'number')
            && ['pp', 'ep'].every(k => Array.isArray(last[k]) && last[k].length === 4),
      };
    }""")
    if len(got["you"]) != got["wantYou"]:
        fails.append(f"{name}: {len(got['you'])} rows for your {got['wantYou']} items")
    if len(got["them"]) != got["wantThem"]:
        fails.append(f"{name}: {len(got['them'])} rows for its {got['wantThem']} items")
    if not got["them"]:
        fails.append(f"{name}: the creature's side of the replay is empty")
    if not got["shape"]:
        fails.append(f"{name}: the replay is not carrying armour and pools")

    # Pointing at a row with gear behind it opens that item's card, in the same
    # two halves the packing panel uses.
    row = page.query_selector("#ticks-them .tick.has-card") \
       or page.query_selector("#ticks-you .tick.has-card")
    if row is None:
        return
    row.hover()
    page.wait_for_selector("#tick-card", state="visible", timeout=4000)
    card = page.evaluate("""() => {
      const b = document.getElementById('tick-card');
      return { heads: [...b.querySelectorAll('.head')].map(h => h.textContent.trim()),
               name: b.querySelector('b')?.textContent ?? '' };
    }""")
    if len(card["heads"]) < 2:
        fails.append(f"{name}: the replay card has no two halves: {card}")
    elif "standing" not in card["heads"][0].lower():
        fails.append(f"{name}: the replay card leads with {card['heads'][0]!r}")


def check_a_component_is_a_shape(page, name, fails):
    """The shelf shows what you are actually buying.

    A component is a shape: two blades at one price are not the same purchase
    when one is four cells in a line and the other is a cross. The shelf used
    to give a name, a slot and a price, which is everything except that.
    """
    got = page.evaluate("""() => [...document.querySelectorAll('#shelf .wares')].map(b => ({
      name: b.querySelector('b')?.textContent ?? '',
      shape: !!b.querySelector('canvas.shape'),
      w: b.querySelector('canvas.shape')?.width ?? 0,
      meta: b.querySelector('.meta')?.textContent ?? '',
    }))""")
    if not got:
        fails.append(f"{name}: nothing on the shelf to look at")
        return
    for w in got:
        if not w["shape"] or w["w"] == 0:
            fails.append(f"{name}: {w['name']!r} is for sale with no shape drawn")
        # slot · kind · rating, and the kind is the word the engine uses.
        if w["meta"].count("·") < 2:
            fails.append(f"{name}: {w['name']!r} says {w['meta']!r}, not slot and type")

    # And hovering one reads the component, not the item it might become.
    page.locator("#shelf .wares").first.hover()
    page.wait_for_selector("#piece-card", state="visible", timeout=4000)
    card = page.evaluate("""() => {
      const b = document.getElementById('piece-card');
      return { name: b.querySelector('b')?.textContent ?? '',
               built: b.querySelector('.built')?.textContent ?? '',
               lines: b.querySelectorAll('.stats li').length };
    }""")
    if card["name"] != got[0]["name"]:
        fails.append(f"{name}: hovering {got[0]['name']!r} read {card['name']!r}")
    if not card["built"].strip():
        fails.append(f"{name}: the component card does not say what kind of thing it is")
    if card["lines"] == 0:
        fails.append(f"{name}: {card['name']!r} explains nothing at all")


def check_the_bag_shows_shapes(page, name, fails):
    """And so does the bag under the board.

    It drew a single-cell swatch for everything, so a one-cell ring and a
    twelve-cell base looked identical — hiding the only property of a loose
    component that decides where it can go.
    """
    got = page.evaluate("""() => {
      const b = window.__board;
      const bag = b.state?.bag ?? [];
      return { n: bag.length,
               cells: bag.map(p => (p.cells ?? []).length),
               kinds: bag.map(p => p.kind ?? ''),
               lines: bag.map(p => (p.lines ?? []).length) };
    }""")
    if not got["n"]:
        return
    if any(c == 0 for c in got["cells"]):
        fails.append(f"{name}: a loose component arrived with no shape")
    if any(not k for k in got["kinds"]):
        fails.append(f"{name}: a loose component arrived with no type")
    if all(l == 0 for l in got["lines"]):
        fails.append(f"{name}: no loose component can explain itself")


def check_both_boards_are_in_the_replay(page, name, fails):
    """A fight is two boards, and the one that fires jolts.

    The shake is what makes six cooldown bars legible: it says *that one, now*
    on the board itself, and nothing else on the board moves.
    """
    got = page.evaluate("""() => {
      const r = window.__replay;
      const w = (id) => document.getElementById(id)?.width ?? 0;
      return { you: w('board-you'), them: w('board-them'),
               mine: (r.log.player.slots ?? []).length,
               theirs: (r.log.enemy?.slots ?? []).length };
    }""")
    if got["mine"] == 0:
        fails.append(f"{name}: the replay did not carry your board")
    if got["theirs"] == 0:
        fails.append(f"{name}: the replay did not carry the creature's board")
    if got["you"] == 0:
        fails.append(f"{name}: your board drew nothing in the replay")

    # Park the head just after something fired and check it is shaking.
    #
    # Only activations that *have* cells count. A creature's innate attack —
    # the Cave Rat's bite — stands on no gear, so there is nothing on a board
    # for it to move, and a fight short enough that only the bite went off is a
    # fight with nothing to assert. That is a property of the fight, not a
    # failure, so it skips rather than fails.
    shook = page.evaluate("""() => {
      const r = window.__replay;
      r.playing = false;
      const shakeable = new Set();
      for (const side of ['player', 'enemy']) {
        r.rows[side].forEach((row, i) => { if (row.cells?.length) shakeable.add(`${side}:${i}`); });
      }
      const fired = r.log.entries.filter(
        e => e.kind === 'activate' && e.at > 0 && shakeable.has(`${e.side}:${e.index}`));
      if (!fired.length) return null;
      for (const e of fired) {
        r.t = e.at + 40;
        r.draw();
        const n = (r.boards.player?.shaking.length ?? 0)
                + (r.boards.enemy?.shaking.length ?? 0);
        if (n > 0) return n;
      }
      return 0;
    }""")
    if shook == 0:
        fails.append(f"{name}: an item with cells on a board went off and nothing moved")


def check_the_sheet_says_what_you_are(page, name, fails):
    """A point spent shows up somewhere.

    Reported from a real session: four skills taken, and no way to tell whether
    any of them had landed. The tree grants +6 strength and +60 max health, and
    there was nowhere in the game showing either — which is indistinguishable
    from a skill that does nothing.
    """
    got = page.evaluate("""() => {
      const c = window.__character();
      return { rows: [...document.querySelectorAll('#sheet li')].map(li => li.textContent.trim()),
               stats: c.stats ?? [], held: c.held ?? {} };
    }""")
    if not got["rows"]:
        fails.append(f"{name}: the sheet says nothing about the character")
        return
    text = " ".join(got["rows"]).lower()
    for want in ("max health", "strength"):
        if want not in text:
            fails.append(f"{name}: the sheet never mentions {want}: {got['rows']}")
    # Every non-zero stat core reports has to be on it.
    for st in got["stats"]:
        if st["n"] and st["label"].lower() not in text:
            fails.append(f"{name}: core reports {st['label']!r} and the sheet drops it")


def check_a_starting_balance_is_on_the_bar(page, name, fails):
    """What you begin a fight holding is on screen from the first frame.

    The armour bar opened at zero however much the tree had granted, because
    the only armour event in the log reports what is *left* after a hit —
    nothing announces a balance nobody had to earn. A player with `Corked`
    watched an empty bar and concluded the skill was broken.
    """
    got = page.evaluate("""() => {
      const r = window.__replay, log = r.log;
      const opened = r.track.player[0];
      return {
        armor: log.player.armor ?? null,
        pools: log.player.pools ?? null,
        trackArmor: opened[3], trackPools: opened[4],
        enemyArmor: log.enemy?.armor ?? null,
      };
    }""")
    if got["armor"] is None or got["pools"] is None:
        fails.append(f"{name}: the log does not say what the fight opened with")
        return
    if got["trackArmor"] != got["armor"]:
        fails.append(f"{name}: the fight opened holding {got['armor']} armour and the bar "
                     f"drew {got['trackArmor']}")
    if got["trackPools"] != got["pools"]:
        fails.append(f"{name}: the fight opened holding {got['pools']} and the row "
                     f"drew {got['trackPools']}")

    # **And prove the seeding rather than comparing zero with zero.**
    #
    # A character on this walk has usually banked nothing, so the check above
    # is satisfied by a replay that hard-codes an empty bar — which is exactly
    # the bug it exists to catch, and it passed a deliberately reverted build.
    # Feed the same log back with a balance on it and read the opening row.
    seeded = page.evaluate("""() => {
      const r = window.__replay;
      const log = JSON.parse(JSON.stringify(r.log));
      log.player.armor = 37;
      log.player.pools = [11, 5, 3, 2];
      r.load(log);
      const row = r.track.player[0];
      r.t = 0;
      return { armor: row[3], pools: row[4] };
    }""")
    if seeded["armor"] != 37 or seeded["pools"] != [11, 5, 3, 2]:
        fails.append(f"{name}: a fight opening with 37 armour drew {seeded} — the bar is "
                     f"not reading what the fight began with")


def check_the_result_is_a_pocket_not_a_level(page, name, fails, lines, before):
    """A win fills your pocket and moves nothing else.

    Called around the fight the walk always runs, so it is not waiting for a
    state that may not arrive: the first version of this checked the town while
    the pocket was usually empty and returned without asserting anything, which
    is the vacuous-check trap in a fresh coat.

    That a town *can* spend it is proved by the level-up grind further down —
    it only ever reaches level two by banking, and it failed loudly when the
    button was not yet wired.
    """
    after = page.evaluate("() => window.__character()")
    won = any("Fnorp" in l for l in lines)
    # **A fight never crosses a level and never spends a point.** Whatever it
    # did to the pocket, the two numbers a town owns must be untouched.
    if after["level"] != before["level"]:
        fails.append(f"{name}: a fight took the character from level "
                     f"{before['level']} to {after['level']} — only a town does that")
    if after["xp"] != before["xp"]:
        fails.append(f"{name}: a fight spent experience: {before['xp']} -> {after['xp']}")
    if won:
        if after["carried"] <= before["carried"]:
            fails.append(f"{name}: a win carried nothing: "
                         f"{before['carried']} -> {after['carried']}")
        if not any("carried" in l for l in lines):
            fails.append(f"{name}: the receipt never says the experience is carried: {lines}")
    elif after["carried"] != 0:
        fails.append(f"{name}: a defeat left {after['carried']} in the pocket")


def tape(page):
    """Every line the game has said this sitting, newest last.

    **M11.0 moved the slot.** Refusals, quest movement, pickups and the save
    all used to print into a `<p id="says">` hanging off the bottom of the save
    panel — a slot that existed because nothing owned it. They go through one
    `log()` now, into a strip that keeps the last few and a HISTORY overlay
    that keeps the sitting. The strip is what a player is looking at, so it is
    what these checks read.
    """
    return [t.strip() for t in page.eval_on_selector_all(
        "#tape li", "els => els.map(e => e.textContent)")]


def last_said(page):
    """The last thing the game said, which is what the old `#says` held."""
    lines = tape(page)
    return lines[-1] if lines else ""


def plant(page, base_path, edit, stem="probe"):
    """Load a save built by editing a downloaded one. Returns nothing.

    The pattern the gate check invented and this file now shares: walking to a
    particular state costs twenty minutes of fighting, and what wants proving
    is what happens *at* that state rather than the road to it.

    **Waits for the load rather than sleeping through it.** The page's file
    handler is `async` — it awaits `arrayBuffer()` — so four hundred
    milliseconds is a guess, and a guess that is wrong reads every assertion
    after it against the *previous* game. It says `Loaded <name>.` when it is
    done, so that is what this waits for.
    """
    save = json.loads(Path(base_path).read_text())
    edit(save.get("state", save))
    out = Path(base_path).parent / f"{stem}.json"
    out.write_text(json.dumps(save))
    page.set_input_files("#file", str(out))
    try:
        page.wait_for_function(
            "name => Array.from(document.querySelectorAll('#tape li'))"
            ".some(e => (e.textContent || '').includes('Loaded ' + name))",
            arg=out.name, timeout=8000)
    except Exception:
        # Older callers plant files whose load says something else; the sleep is
        # what they always had.
        page.wait_for_timeout(400)


def check_an_errand_can_be_handed_in_where_it_was_taken(page, name, fails):
    """The reported blocker: Marbulon's tile went dead once her card was read.

    `world::step` opened an event only when its id was absent from `answered`,
    so answering the card once made the tile inert — and her two errands, the
    questline that unlocks the Cave, could never be taken or handed in.

    Planted rather than walked. What the browser has to prove is that a place
    with an errand on it reopens **after it has been answered**, still shows
    the errand, and takes it back; the questline itself is `tests/quests.rs`.
    """
    door = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.id === 'marbulons-door')""")
    if not door:
        fails.append(f"{name}: the overworld has no marbulons-door on it")
        return
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    # Standing one west of her door, having already read her card, carrying
    # the errand and the three jars it asked for. Exactly the reported state.
    def used(body):
        w = body.setdefault("world", {})
        w["at"] = [door["at"][0] - 1, door["at"][1]]
        w["map"] = ""
        w["answered"] = list(w.get("answered", [])) + ["marbulons-door"]
        w["quests_taken"] = ["marbulon-asks-first"]
        w["quests_done"] = []
        reg = body["character"].setdefault("registry", [])
        for _ in range(3):
            reg.append({"def": "Whisper Jar", "rot": 0})
            body["character"]["owned"].append(len(reg) - 1)

    plant(page, base, used, stem="errand-probe")
    page.keyboard.press("ArrowRight")
    page.wait_for_timeout(350)

    if page.is_hidden("#card"):
        fails.append(f"{name}: an answered event tile with an errand on it opened nothing")
        return
    # A spent door is not offered again — the choices were answered once and
    # that was right. Only the errand board comes back.
    if page.locator("#card-choices button").count():
        fails.append(f"{name}: reopening a spent card offered its choices again")
    errands = page.locator("#card-errands .wares")
    if errands.count() == 0:
        fails.append(f"{name}: her door reopened with no errand board on it")
        return
    foot = (page.locator("#card-errands .wares .cost").first.text_content() or "").strip()
    if foot.lower() != "hand it in":
        fails.append(f"{name}: an errand ready to hand in reads {foot!r}")
    live = page.locator("#card-errands .wares:not(:disabled)")
    if live.count() == 0:
        fails.append(f"{name}: the errand is ready and the button is dead")
        return
    live.first.click()
    page.wait_for_timeout(200)
    said_it = last_said(page)
    if not said_it:
        fails.append(f"{name}: handing in at the door said nothing")
    done = page.evaluate("""() => JSON.parse(window.__save()).state.world.quests_done""")
    if "marbulon-asks-first" not in (done or []):
        fails.append(f"{name}: handed the errand in and the save says {done}")

    # And the ring on the map moved on: the first is done, so what is drawn is
    # the second one waiting to be taken.
    marks = page.evaluate("() => window.__errandMarks()")
    if not any(m["id"] == "marbulons-door" and m["mark"] == "take" for m in marks):
        fails.append(f"{name}: her door is not marked for the next errand: {marks}")
    # Her card is still open, and it is over everything.
    page.click("#card-close")
    page.wait_for_selector("#card", state="hidden", timeout=5000)


def check_the_fork_is_on_top(page, name, fails):
    """The class fork is clickable when it opens, which is in a town.

    Found by playing it. A level lands when you bank, banking happens with the
    town screen up, and every `.screen` sat at the same z-index — so the town,
    which comes later in the file, painted over the fork. Four class cards on
    screen, all of them under something, none of them clickable, and the game
    unfinishable from level five.

    **Measured with `elementFromPoint`**, because reading the source says every
    card is a button and it was. This is the third time that has been the only
    way to see it: `.card` made every item card a full-viewport overlay, and
    `.screen.framed` kept a hidden fight screen swallowing clicks.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()
    town = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'town')""")

    def owed(body):
        body["character"]["class"] = None
        body["character"]["xp"] = 0
        body["character"]["carried"] = 600
        body["world"]["at"] = [town["at"][0] + 1, town["at"][1]]
        body["world"]["map"] = ""

    plant(page, base, owed, stem="fork-probe")
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not get into a town to bank a level")
        return
    page.click("#bank")
    page.wait_for_timeout(300)
    if page.is_hidden("#fork"):
        fails.append(f"{name}: banked past level five in a town and was never asked")
        return
    hidden = page.evaluate("""() => [...document.querySelectorAll('#fork-choices .wares')]
        .map(b => {
          const r = b.getBoundingClientRect();
          const mid = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
          return b.contains(mid) ? null
               : { name: b.querySelector('b').textContent,
                   under: mid ? (mid.closest('.screen')?.id ?? mid.tagName) : 'nothing' };
        }).filter(Boolean)""")
    if hidden:
        fails.append(f"{name}: the fork opened under something: {hidden}")
        return
    # Take it, so the walk carries on with a class rather than a screen it
    # cannot dismiss — and taking it opens the tree, over the same town,
    # which is the same bug one screen along.
    page.locator("#fork-choices .wares").first.click()
    page.wait_for_selector("#fork", state="hidden", timeout=8000)
    if page.is_visible("#tree"):
        buried = page.evaluate(
            "() => {"
            "  const b = document.querySelector('#tree-done');"
            "  const r = b.getBoundingClientRect();"
            "  const mid = document.elementFromPoint(r.x + r.width / 2,"
            "                                        r.y + r.height / 2);"
            "  return b.contains(mid) ? null : (mid?.closest('.screen')?.id ?? 'nothing');"
            "}")
        if buried:
            fails.append(f"{name}: the tree opened from the fork is under {buried}")
            return
        page.click("#tree-done")
        page.wait_for_selector("#tree", state="hidden", timeout=8000)
    if page.is_visible("#town"):
        page.click("#leave")
        page.wait_for_selector("#town", state="hidden", timeout=5000)


def check_the_door_opens_on_the_treyway(page, name, fails):
    """A wall with no door in it grows one, and opening it crosses a map.

    Three firsts in one tile, and M11.1 gives it a fourth. The three: a place
    that is not there until it is, a lock answered against the bag rather than
    against the map, and a paragraph you read once. The fourth is that it is a
    **border** — a gate that names no landing tile, so the far side puts you
    where you left off and its start the first time.

    Planted at each stage rather than walked, because the walk to it is the
    whole game.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    on_map = page.evaluate("""() => (window.__world().places ?? [])
            .some(p => p.id === 'the-door-in-the-wall')""")
    if on_map:
        fails.append(f"{name}: the door is on the map before the Cave's boss is down")

    # --- the boss is down, so the wall has a door in it ----------------------
    def cleared(body, key=False):
        w = body.setdefault("world", {})
        w["map"] = ""
        w["answered"] = list(w.get("answered", [])) + ["the-bottom-of-the-cave"]
        if key:
            reg = body["character"].setdefault("registry", [])
            reg.append({"def": "The Deep Gate Key", "rot": 0})
            body["character"]["owned"].append(len(reg) - 1)

    plant(page, base, cleared, stem="door-shut")
    spot = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.id === 'the-door-in-the-wall')""")
    if not spot:
        fails.append(f"{name}: the Cave's boss is down and the wall still has no door")
        return
    if spot.get("kind") != "gate":
        fails.append(f"{name}: the door is a {spot.get('kind')!r} and not a way through")

    # Stand beside it and walk in without the key.
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    dismiss_card(page)
    if page.evaluate("() => window.__world().id") != "west-bambulon":
        fails.append(f"{name}: the door opened with no key in the bag")
        return
    said_it = last_said(page)
    if not said_it:
        fails.append(f"{name}: a locked door said nothing")

    # --- and with the key ----------------------------------------------------
    plant(page, base, lambda b: cleared(b, key=True), stem="door-open")
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(400)
    where = page.evaluate("() => window.__world().id")
    if where == "west-bambulon":
        fails.append(f"{name}: carried the key onto the door and stayed in Bambulon")
        return
    if where != "the-treyway":
        fails.append(f"{name}: the door opened onto {where!r}")
        return

    # It says something on the way through, once, and the card is a card:
    # you walk on from it rather than being held in it.
    prose = page.evaluate("""() => [...document.querySelectorAll('#card-prose p')]
        .map(p => p.textContent).join(' ')""")
    if len(prose) < 80:
        fails.append(f"{name}: crossing out of Bambulon said {prose!r}")
    if "demo" in prose.lower():
        fails.append(f"{name}: the crossing still says the demo ends here")
    dismiss_card(page)

    # --- the key turned, and it is gone --------------------------------------
    #
    # **Reported by the human**: a key you have used sits in the bag for ever,
    # because the bag was only ever *asked* about it. `Game::unlock` spends it.
    # Checked here rather than in a check of its own because this is the only
    # place in the walk where a key actually turns.
    # `canonical`, not `name` — `name` is the themed one, and matching against
    # the catalogue's wording there is how this check first shipped vacuous.
    # **Read off the save, which is the only screen-independent answer.**
    # This check shipped vacuous twice before it worked: `character_json` has
    # no `bag` field at all, and `window.__board.state.bag` is not populated
    # until the packing screen has painted, which this check never opens. The
    # save always knows what you own, and it names components canonically.
    carrying = page.evaluate("""() => {
        const s = JSON.parse(window.__save());
        const c = s.character ?? s.state?.character ?? {};
        const reg = c.registry ?? [];
        return (c.owned ?? []).map(i => (reg[i] || {}).def || '?');
    }""")
    if "The Deep Gate Key" in carrying:
        fails.append(f"{name}: the key opened the door and is still in the bag: {carrying}")
    # Not `last_said`: "You go through." is logged after it, and the order is
    # right — the key turns, then you walk.
    said_key = " ".join(tape(page))
    if "Key" not in said_key or "gone" not in said_key:
        fails.append(
            f"{name}: the key vanished out of the bag and nothing said so: {said_key!r}"
        )

    # A different map: its own size, its own ground, its own regions.
    shape = page.evaluate("() => { const w = window.__world(); "
                          "return [w.width, w.height, (w.places||[]).length]; }")
    if shape[0] != 16 or shape[1] != 16:
        fails.append(f"{name}: the Treyway came back {shape[0]}x{shape[1]}")
    here = page.text_content("#region") or ""
    if "Bambulon" in here or not here.strip():
        fails.append(f"{name}: standing on the Treyway and the panel says {here!r}")

    # --- and the lock stays open, which is what stops a soft-lock -------------
    #
    # The door in the wall is the only way to the back half of the game, and a
    # defeat in the Treyway walks you home to West Bambulon. A key that were
    # spent *and* re-locked would end the run there, and there is no second
    # key anywhere. So: the boss down, the door remembered, and **nothing in
    # the bag**.
    def spent_and_open(body):
        cleared(body)  # note: no key
        body["world"]["answered"].append("the-door-in-the-wall")

    plant(page, base, spent_and_open, stem="door-stays-open")
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    if page.evaluate("() => window.__world().id") != "the-treyway":
        fails.append(
            f"{name}: the door re-locked itself after the key was spent — "
            "there is no second key and the run is over"
        )
        return

    # --- a border remembers ---------------------------------------------------
    #
    # **Planted, and it has to be.** Walking back out through `the-door-back`
    # records the tile you were standing on, which is the door itself — so a
    # there-and-back walk would land you on the Treyway's start and pass
    # whether or not `positions` exists at all. The case that matters is the
    # one you cannot walk to on purpose: you were carried off this map by a
    # defeat, and the door puts you back where you fell.
    fell_at = [7, 8]

    def carried_off(body):
        cleared(body, key=True)
        body["world"]["positions"] = [["the-treyway", fell_at]]

    plant(page, base, carried_off, stem="door-remembers")
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    if page.evaluate("() => window.__world().id") != "the-treyway":
        fails.append(f"{name}: the door stopped opening on the second crossing")
        return
    landed = (page.text_content("#coords") or "").strip()
    if landed != f"{fell_at[0]}, {fell_at[1]}":
        fails.append(f"{name}: fell at {fell_at} and the door landed you at {landed!r}")

    # **Put the walk back where it was standing.** Every check after this one
    # downloads the current save as its own base, and this is the first check
    # in the gate that leaves the player on a *different map* — so without the
    # restore, the next plant is a Treyway save and the one after it looks for
    # a town on a map that has none. It cost one run to find and it is the
    # eighth trap in a new coat: a check that changes the world puts it back.
    plant(page, base, lambda body: None, stem="door-restore")


def check_the_road_west_reaches_a_town(page, name, fails):
    """Two maps out from Bambulon there is a town, and it trades.

    **M11.2.** Kettleworks has had a shelf and two errands since M8 and no
    ground under it. What a browser has to prove is the part the engine tests
    cannot: that a player can walk the whole way — door, Treyway, road west —
    and end up on a shelf. Planted onto the Treyway rather than walked from the
    pit, because the walk from the pit is the whole game.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    road = [3, 13]

    def on_the_treyway(body):
        w = body.setdefault("world", {})
        w["map"] = "the-treyway"
        w["at"] = [road[0], road[1] - 1]
        w["answered"] = list(w.get("answered", [])) + ["the-bottom-of-the-cave"]

    plant(page, base, on_the_treyway, stem="road-west")
    if page.evaluate("() => window.__world().id") != "the-treyway":
        fails.append(f"{name}: could not stand on the Treyway")
        plant(page, base, lambda body: None, stem="road-west-restore")
        return
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    where = page.evaluate("() => window.__world().id")
    if where != "kettleworks-field":
        fails.append(f"{name}: the road west led to {where!r}")
        plant(page, base, lambda body: None, stem="road-west-restore")
        return

    # The dense map: one tile in ten carries something.
    dense = page.evaluate("""() => {
      const w = window.__world();
      return { places: w.places.length, tiles: w.width * w.height,
               curd: w.rows.flat().filter(t => t === 'curd').length };
    }""")
    if dense["places"] < 40:
        fails.append(f"{name}: {dense['places']} of {dense['tiles']} tiles answer")
    if dense["curd"] != 16:
        fails.append(f"{name}: the Drambus Stack is {dense['curd']} tiles")

    # An examinable is a card with nothing to answer, and you walk on from it.
    # **`the-flies` rather than `the-milestone` since M12.5**: the milestone
    # asks something now, and an examinable check needs an event that is still
    # furniture. What is being checked is unchanged — a card with nothing to
    # answer that you walk on from.
    look = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.id === 'the-flies')""")
    if not look:
        fails.append(f"{name}: the field has no flies on it")
    else:
        # **Stand beside it and take one step.** Standing *on* it and stepping
        # off and back is two steps, and the first of them can roll a fight —
        # which swallows the second keypress and leaves the fight screen up for
        # every check after this one. That is trap eight in a new coat and it
        # cost a run: `close_fight` on every path out.
        page.evaluate("(at) => window.__standHere(at)", [look["at"][0] + 1, look["at"][1]])
        close_fight(page)
        dismiss_card(page)
        page.keyboard.press("ArrowLeft")
        page.wait_for_timeout(350)
        if page.is_visible("#card"):
            n = page.locator("#card-choices button").count()
            if n:
                fails.append(f"{name}: an examinable offers {n} choices")
            if page.is_hidden("#card-bar"):
                fails.append(f"{name}: an examinable has no way out of it")
            dismiss_card(page)
        else:
            fails.append(f"{name}: standing on the flies opened nothing")
        close_fight(page)

    # And the town on it opens, sells, and wants something.
    town = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'town')""")
    if not town or town["id"] != "kettleworks":
        fails.append(f"{name}: the field's town is {town}")
    else:
        page.evaluate("(at) => window.__standHere(at)", [town["at"][0] + 1, town["at"][1]])
        close_fight(page)
        dismiss_card(page)
        page.keyboard.press("ArrowLeft")
        page.wait_for_timeout(400)
        if not page.is_visible("#town"):
            fails.append(f"{name}: walked onto Kettleworks and no town opened")
        else:
            wares = page.locator("#shelf .wares").count()
            errands = page.locator("#quests .wares").count()
            if wares < 10:
                fails.append(f"{name}: Kettleworks' shelf has {wares} things on it")
            if errands < 1:
                fails.append(f"{name}: Kettleworks wants nothing")
            page.click("#leave")
            page.wait_for_selector("#town", state="hidden", timeout=5000)

    plant(page, base, lambda body: None, stem="road-west-restore")


def check_the_tower_drops(page, name, fails):
    """The door opens onto a different floor every time, and then onto nothing.

    **M11.3.** Which floor you get is derived from the boss tiles in
    `answered`, so a browser can plant the answers and walk in — which is the
    only way to see all six values without five end-game fights. What has to
    hold from out here: the entrance opens somewhere different at each count,
    the map behind it is a *different* map each time, clearing the last one
    leaves a stump that says so, and a save taken inside a floor reopens
    outside it.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    floors = [f"the-drambus-stack-{n}" for n in (5, 4, 3, 2, 1)]
    door = [8, 12]

    def outside_the_stack(cleared):
        def edit(body):
            w = body.setdefault("world", {})
            w["map"] = "kettleworks-field"
            w["at"] = [door[0], door[1] + 1]
            w["answered"] = list(w.get("answered", [])) + [f"{f}-boss" for f in cleared]
        return edit

    seen = []
    for i in range(5):
        plant(page, base, outside_the_stack(floors[:i]), stem=f"stack-{i}")
        if page.evaluate("() => window.__world().id") != "kettleworks-field":
            fails.append(f"{name}: could not stand outside the Stack with {i} floors down")
            break
        page.keyboard.press("ArrowUp")
        page.wait_for_timeout(400)
        dismiss_card(page)
        where = page.evaluate("() => window.__world().id")
        seen.append(where)
        if where != floors[i]:
            fails.append(f"{name}: with {i} floors down the door opened onto {where!r}, "
                         f"not {floors[i]!r}")
            break
        # A floor is a floor: ten by ten, one thing standing on it, no way out.
        shape = page.evaluate("""() => {
          const w = window.__world();
          return [w.width, w.height,
                  w.places.filter(p => p.kind === 'boss').length,
                  w.places.filter(p => p.kind === 'gate').length];
        }""")
        if shape != [10, 10, 1, 0]:
            fails.append(f"{name}: {where} is {shape}, not a ten-by-ten with one boss on it")
        close_fight(page)

    if len(set(seen)) != len(seen):
        fails.append(f"{name}: the door opened onto the same floor twice: {seen}")

    # --- the stump ------------------------------------------------------------
    plant(page, base, outside_the_stack(floors), stem="stack-done")
    page.keyboard.press("ArrowUp")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    if page.evaluate("() => window.__world().id") != "kettleworks-field":
        fails.append(f"{name}: the Stack came all the way down and the door still opened")
    else:
        said = last_said(page)
        if not said or "stump" not in said.lower():
            fails.append(f"{name}: nothing is left of the Stack and it said {said!r}")

    # --- a floor is one sitting ----------------------------------------------
    #
    # Planted *inside* a floor, which is a save a player cannot make on purpose
    # and which the game has to open anyway: a tab closed mid-climb.
    def inside_a_floor(body):
        w = body.setdefault("world", {})
        w["map"] = floors[0]
        w["at"] = [1, 8]

    plant(page, base, inside_a_floor, stem="stack-inside")
    dismiss_card(page)
    close_fight(page)
    where = page.evaluate("() => window.__world().id")
    if where != "kettleworks-field":
        fails.append(f"{name}: a save taken inside a floor reopened on {where!r}")

    plant(page, base, lambda body: None, stem="stack-restore")


def check_the_lake_drains_and_the_demo_ends_under_it(page, name, fails):
    """The Stack comes down, the lake empties, and there is a door at the bottom.

    **M11.4.** Terrain that is derived from what has happened is new, and the
    thing a browser has to prove is the half no engine test can: that the page
    *redraws* it. The lake is twenty-eight tiles the canvas has painted blue
    since M2, and the map is a cached object — a drain the page never re-read
    would be a lake that is bed in core and water on the screen.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def count_water():
        return page.evaluate("""() => {
          const w = window.__world();
          return { water: w.rows.flat().filter(t => t === 'water').length,
                   bed: w.rows.flat().filter(t => t === 'lakebed').length };
        }""")

    def outside_the_lake(body, tower_down):
        strip_the_boards(body)
        w = body.setdefault("world", {})
        w["map"] = ""
        w["at"] = [7, 8]
        answered = list(w.get("answered", []))
        if tower_down:
            answered += [f"the-drambus-stack-{n}-boss" for n in (5, 4, 3, 2, 1)]
        w["answered"] = answered

    plant(page, base, lambda b: outside_the_lake(b, False), stem="lake-full")
    full = count_water()
    if full["water"] != 28 or full["bed"]:
        fails.append(f"{name}: the lake before the Stack comes down is {full}")

    plant(page, base, lambda b: outside_the_lake(b, True), stem="lake-dry")
    dry = count_water()
    if dry["water"] or dry["bed"] != 28:
        fails.append(f"{name}: the Stack is down and the lake is {dry}")
        plant(page, base, lambda body: None, stem="lake-restore")
        return

    # And you can walk out onto it wearing nothing.
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(300)
    dismiss_card(page)
    close_fight(page)
    if page.text_content("#coords").strip() != "7, 9":
        fails.append(f"{name}: the lake drained and is still a wall: "
                     f"{page.text_content('#coords')!r}")

    # --- the door at the bottom of it ----------------------------------------
    def under_it(body, beaten):
        strip_the_boards(body)
        w = body.setdefault("world", {})
        w["map"] = "under-the-lake"
        w["at"] = [7, 7]
        answered = list(w.get("answered", []))
        answered += [f"the-drambus-stack-{n}-boss" for n in (5, 4, 3, 2, 1)]
        if beaten:
            answered.append("the-bottom-of-the-lake")
        w["answered"] = answered

    plant(page, base, lambda b: under_it(b, False), stem="lake-boss-up")
    still_there = page.evaluate("""() => (window.__world().places ?? [])
        .some(p => p.kind === 'door')""")
    if still_there:
        fails.append(f"{name}: the door under the lake is there before the boss is down")

    plant(page, base, lambda b: under_it(b, True), stem="lake-boss-down")
    door = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'door')""")
    if not door:
        fails.append(f"{name}: the boss is down and there is nothing behind it")
        plant(page, base, lambda body: None, stem="lake-restore")
        return
    page.evaluate("(at) => window.__standHere(at)", [door["at"][0] - 1, door["at"][1]])
    close_fight(page)
    dismiss_card(page)
    page.keyboard.press("ArrowRight")
    page.wait_for_timeout(400)
    if page.is_hidden("#ending"):
        fails.append(f"{name}: walked onto the last door in the game and nothing happened")
    else:
        prose = page.evaluate("""() => [...document.querySelectorAll('#ending-prose p')]
            .map(p => p.textContent).join(' ')""")
        if "decided" not in prose.lower():
            fails.append(f"{name}: the ending does not say what it is: {prose[:80]!r}")
        # It is not the fork: you can back out of it.
        page.click("#ending-close")
        page.wait_for_selector("#ending", state="hidden", timeout=5000)

    plant(page, base, lambda body: None, stem="lake-restore")


def check_an_instrument_takes_the_grid(page, name, fails):
    """A compass in the weapon grid grants a rule, and the grid takes no blade.

    **M11.5**, the block's first save seam. What a browser has to prove is the
    half no engine test can: that the refusal reaches a player where they will
    read it. "Does not fit there" belongs on the board and clears in two and a
    half seconds; *the grid is doing something else* is a rule about what you
    may be, it happens twice in a playthrough, and it goes in the log.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def with_a_compass(body):
        seat_a_set(body, COMPASS, "weapon")
        # And a handle in the bag, unworn, to try to seat beside it.
        reg = body["character"]["registry"]
        reg.append({"def": "Oak Handle", "rot": 0})
        body["character"]["owned"].append(len(reg) - 1)

    plant(page, base, with_a_compass, stem="compass-probe")
    rules = page.evaluate("() => window.__character().rules ?? []")
    if not any("survey" in str(r).lower() or "compass" in str(r).lower() for r in rules):
        fails.append(f"{name}: a compass is on the board and grants {rules}")

    # The sheet prints it, because a rule moves no bar and a screen that shows
    # nothing is a rule that cannot be told from a bug.
    sheet = page.evaluate("""() => [...document.querySelectorAll('#sheet li')]
        .map(li => li.textContent).join(' | ')""")
    if "compass" not in sheet.lower():
        fails.append(f"{name}: the sheet says nothing about the compass: {sheet!r}")

    plant(page, base, lambda body: None, stem="compass-restore")


def check_the_reach_reads_through_what_you_carry(page, name, fails):
    """The edge refuses without an instrument, and each of the three changes it.

    **M11.6.** The engine's half is `tests/reach.rs`; what a browser has to
    prove is the half no engine test can — that the survey *reaches the screen*.
    A survey moves the encounter rate, what falls off a win and what a win pays,
    and a player sees none of those directly, so the panel has to say what it is
    reading the map through or the whole feature is indistinguishable from
    nothing happening.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    edge = [6, 1]

    def at_the_edge(body, instrument=None):
        # **The board first, the world after.** `strip_the_boards` resets
        # `world.map` to the first map — it always has, and it is right to,
        # because most plants want the overworld. A plant that wants a
        # different map has to set it *after* every stripper has run, and
        # `seat_a_set` strips too.
        strip_the_boards(body)
        if instrument:
            seat_a_set(body, instrument, "weapon")
        w = body.setdefault("world", {})
        w["map"] = "the-treyway"
        # **From the east, not the south.** The tile below the edge is a
        # mountain range: the Treyway's terrain has walls in it that West
        # Bambulon's does not, and a plant that stands in one is repaired away
        # before the first keypress.
        w["at"] = [edge[0] + 1, edge[1]]
        w["answered"] = list(w.get("answered", [])) + ["the-bottom-of-the-cave"]

    # --- with nothing ---------------------------------------------------------
    plant(page, base, lambda b: at_the_edge(b), stem="reach-shut")
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(350)
    dismiss_card(page)
    close_fight(page)
    if page.evaluate("() => window.__world().id") != "the-treyway":
        fails.append(f"{name}: the reach opened with nothing to read it with")
    else:
        said = last_said(page)
        if not said:
            fails.append(f"{name}: the reach refused in silence")

    # --- with a compass -------------------------------------------------------
    plant(page, base, lambda b: at_the_edge(b, COMPASS), stem="reach-compass")
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    if page.evaluate("() => window.__world().id") != "the-reach":
        fails.append(f"{name}: a compass did not open the reach")
        plant(page, base, lambda body: None, stem="reach-restore")
        return

    lens = page.evaluate("() => window.__world().survey")
    if not lens or lens.get("kind") != "compass":
        fails.append(f"{name}: standing on the reach and the lens is {lens}")
    elif lens["encounter_pct"] >= 0:
        fails.append(f"{name}: a compass reads the ground at {lens['encounter_pct']}%")
    # And the panel says so, because a number with nowhere it is shown cannot
    # be told from a bug.
    shown = (page.text_content("#survey") or "").lower()
    if "compass" not in shown:
        fails.append(f"{name}: the panel says {shown!r} about the survey")
    if page.is_hidden("#survey-row"):
        fails.append(f"{name}: the survey row is hidden while a survey is on")

    # --- and a different instrument is a different map ------------------------
    plant(page, base, lambda b: at_the_edge(b, GOLEM), stem="reach-golem")
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(400)
    dismiss_card(page)
    close_fight(page)
    second = page.evaluate("() => window.__world().survey")
    if not second or second.get("kind") != "golem":
        fails.append(f"{name}: walked in with a golem and read it as {second}")
    elif second["encounter_pct"] == lens["encounter_pct"] and not second["golem"]:
        fails.append(f"{name}: two instruments read the reach identically")

    plant(page, base, lambda body: None, stem="reach-restore")


def check_the_long_way_back(page, name, fails):
    """A whole set takes you home, drinks a tin doing it, and refuses without one.

    **M11.9**, and the block's one piece of new travel. The engine's half is
    `tests/bestiary.rs`; what a browser has to prove is that the button is
    *there when it should be and not when it should not*. Offering a click that
    is going to be refused is a worse screen than not offering it, which is the
    ench rack's lesson and the same answer.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    # --- wearing nothing: no button ------------------------------------------
    def bare(body):
        strip_the_boards(body)
        body["world"]["at"] = [5, 16]

    plant(page, base, bare, stem="home-bare")
    if page.is_visible("#homeward"):
        fails.append(f"{name}: the way home is offered to somebody wearing nothing")

    # --- the whole set, and a tin --------------------------------------------
    def striding(body, tins=1):
        seat_a_set(body, STRIDE, "greaves")
        w = body.setdefault("world", {})
        w["at"] = [5, 16]
        w["last_town"] = "the-end-of-all-gears"
        body["character"]["supplies"] = [["cork-tea", tins]] if tins else []

    plant(page, base, lambda b: striding(b, 0), stem="home-skint")
    if page.is_hidden("#homeward"):
        fails.append(f"{name}: a whole Drover's Stride is on the board and there is no button")
        plant(page, base, lambda body: None, stem="home-restore")
        return
    page.click("#homeward")
    page.wait_for_timeout(300)
    said = last_said(page)
    if "restorative" not in said.lower():
        fails.append(f"{name}: refused with no tin and said {said!r}")
    if page.text_content("#coords").strip() != "5, 16":
        fails.append(f"{name}: a refusal moved the player anyway")

    plant(page, base, lambda b: striding(b, 1), stem="home-fare")
    page.click("#homeward")
    page.wait_for_timeout(400)
    dismiss_card(page)
    town = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'town')""")
    at = (page.text_content("#coords") or "").strip()
    if not town or at != f"{town['at'][0]}, {town['at'][1]}":
        fails.append(f"{name}: the gear took you to {at!r} and the town is at {town}")
    tins = page.evaluate(
        "() => JSON.parse(window.__save()).state.character.supplies ?? []")
    if any(n > 0 for _, n in tins):
        fails.append(f"{name}: it went home and did not drink the fare: {tins}")
    said = last_said(page)
    if "chair" not in said.lower() and "drinks" not in said.lower():
        fails.append(f"{name}: it went home and said {said!r}")

    plant(page, base, lambda body: None, stem="home-restore")


def check_the_rack(page, name, fails):
    """A licensee buys an ench, bolts it on, switches it off, and takes it back.

    Planted, for the same reason scouting's check is: reaching the rack by play
    means levelling to five and being dealt one class of four. What has to hold
    is that the licence gates it, that the board marks what is bolted on, and
    that the card says so — the arithmetic is `tests/enchs.rs`.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    # --- unlicensed: no bench, no rack ---------------------------------------
    #
    # A Gorillathon rather than a character with no class at all: this walk is
    # past level five, and a classed level-five save with the class stripped
    # reopens onto the fork, which is a screen over everything.
    def plain(body):
        body["character"]["class"] = "Berserker"
        body["character"]["gold"] = 400

    plant(page, base, plain, stem="rack-none")
    if page.evaluate("() => JSON.parse(window.__rack()).licensed") is not False:
        fails.append(f"{name}: an unlicensed character is licensed")

    # --- licensed ------------------------------------------------------------
    #
    # **The tree hands the ench over, not a town.** M10 took the bench off every
    # town shelf, and the node at the root of the Patent's spin spine is called
    # Bench Rights and grants The Ponkey Turn — which is what stops the class
    # being inert from level five to level ten.
    def licensed(body):
        body["character"]["class"] = "Recycler"
        body["character"]["gold"] = 400
        taken = list(body["character"].get("skills_taken", []))
        if "k-bench-rights" not in taken:
            taken.append("k-bench-rights")
        body["character"]["skills_taken"] = taken

    plant(page, base, licensed, stem="rack-probe")
    rack = page.evaluate("() => JSON.parse(window.__rack())")
    if not rack["licensed"]:
        fails.append(f"{name}: took the Kaklon Patent and the rack is shut")
        return
    if not rack["loose"]:
        fails.append(f"{name}: took Bench Rights and the rack is empty, so the class is inert")
        return

    # Into a town to pack, which is where the rack lives.
    page.evaluate("() => document.getElementById('map').focus()")
    town = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'town')""")
    page.evaluate("(at) => window.__standAt(at)", [town["at"][0] + 1, town["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not get into a town to pack in")
        return
    page.click("#pack")
    page.wait_for_selector("#fight", state="visible", timeout=8000)
    page.click("#preset")
    page.wait_for_timeout(150)
    try:
        rack_gestures(page, name, fails)
    finally:
        close_fight(page)


def rack_gestures(page, name, fails):
    """Everything on the packing screen; the caller closes it after."""
    if page.is_hidden("#rack"):
        fails.append(f"{name}: a licensee is packing and there is no rack")
        return
    loose = page.locator("#rack-loose .wares")
    if loose.count() == 0:
        fails.append(f"{name}: bought an ench and the rack is empty")
        return
    spec = page.locator("#rack-loose .wares .spec").first.text_content() or ""
    if not any(c.isdigit() for c in spec):
        fails.append(f"{name}: the rack says {spec!r}, which names no number")

    # **Measured, not read.** The board carrying `p.ench` proves core told it;
    # it does not prove anything was drawn. The mark's live colour is a cyan
    # nothing else on this board uses, so counting it answers "is it obvious
    # which component is enched" in the only way a screenshot could.
    def cyan():
        return page.evaluate("""() => {
          window.__board.draw();
          const c = document.getElementById('board');
          const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data;
          let n = 0;
          for (let i = 0; i < d.length; i += 4) {
            if (Math.abs(d[i] - 87) < 5 && Math.abs(d[i + 1] - 179) < 5
                && Math.abs(d[i + 2] - 200) < 5) n++;
          }
          return n;
        }""")

    before = cyan()
    if before:
        fails.append(f"{name}: {before} pixels of the ench mark are on a board with no ench")

    # Pick it up, then click a seated component: two gestures on one target.
    loose.first.click()
    got = page.evaluate("""() => {
      const b = window.__board;
      const s = b.state.slots.find(s => s.placed.length);
      if (!s) return { none: true };
      const p = s.placed[0];
      // Straight at the component's first cell, the way a click lands.
      return { ok: b.onclaim(p.id), id: p.id };
    }""")
    if got.get("none"):
        fails.append(f"{name}: nothing is seated, so there is nothing to bolt anything to")
        return
    if not got["ok"]:
        fails.append(f"{name}: clicking a component with an ench in hand did nothing")
        return
    page.wait_for_timeout(150)

    # The board marks it, the card says so, and core agrees.
    marked = page.evaluate("""(id) => {
      const b = window.__board;
      const p = b.state.slots.flatMap(s => s.placed).find(p => p.id === id);
      const r = JSON.parse(window.__rack());
      return { ench: p?.ench ?? null, on: r.on, loose: r.loose.length };
    }""", got["id"])
    if not marked["ench"]:
        fails.append(f"{name}: bolted an ench on and the board draws nothing on the component")
    elif not marked["ench"]["active"]:
        fails.append(f"{name}: a freshly bolted ench arrived switched off")
    if len(marked["on"]) != 1:
        fails.append(f"{name}: {len(marked['on'])} enchs bolted on, and one was bolted")
    lit = cyan()
    if lit <= before:
        fails.append(f"{name}: bolted an ench on and the board painted no mark "
                     f"({before} -> {lit} pixels)")

    # Switching it off changes the mark and the fight; taking it back empties
    # the component and fills the rack.
    page.locator("#rack-on .wares").first.click()
    page.wait_for_timeout(150)
    off = page.evaluate("""(id) => {
      const b = window.__board;
      const p = b.state.slots.flatMap(s => s.placed).find(p => p.id === id);
      return p?.ench?.active ?? null;
    }""", got["id"])
    if off is not False:
        fails.append(f"{name}: switched an ench off and the board still says {off!r}")
    greyed = cyan()
    if greyed >= lit:
        fails.append(f"{name}: switched an ench off and the mark stayed lit "
                     f"({lit} -> {greyed} pixels)")
    page.locator("#rack-on .wares").first.click()
    page.wait_for_timeout(150)
    back = page.evaluate("""(id) => {
      const b = window.__board;
      const p = b.state.slots.flatMap(s => s.placed).find(p => p.id === id);
      const r = JSON.parse(window.__rack());
      return { ench: p?.ench ?? null, on: r.on.length, loose: r.loose.length };
    }""", got["id"])
    if back["ench"]:
        fails.append(f"{name}: took the ench back and the component still carries it")
    if back["on"] != 0 or back["loose"] == 0:
        fails.append(f"{name}: took the ench back and the rack says {back}")


def check_the_spin_animates(page, name, fails):
    """An item with the turn on it turns, and turns to somewhere core named.

    **The cells are core's.** `Slot::turn_cycle` works out which of the four
    orientations an arrangement can reach *in place*, and the board only picks
    which entry of that list the clock is on — a page rotating a shape itself
    would be a second answer to "where does it fit", and it would disagree the
    first time something was packed against it.

    Planted, like the rack's check, and for the same reason: a licensee at
    level five with ninety Fnorp is a long walk.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def spinning(body):
        body["character"]["class"] = "Recycler"
        body["character"]["gold"] = 400
        body["character"]["enchs_owned"] = ["the-ponkey-turn"]

    plant(page, base, spinning, stem="spin-probe")
    # Straight to the board: the town is not where this is decided.
    page.evaluate("() => document.getElementById('map').focus()")
    town = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'town')""")
    page.evaluate("(at) => window.__standAt(at)", [town["at"][0] + 1, town["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not reach a town to pack in")
        return
    page.click("#pack")
    page.wait_for_selector("#fight", state="visible", timeout=8000)
    page.click("#preset")
    page.wait_for_timeout(150)
    try:
        spin_gestures(page, name, fails)
    finally:
        close_fight(page)


def spin_gestures(page, name, fails):
    loose = page.locator('#rack-loose .wares[data-ench="the-ponkey-turn"]')
    if loose.count() == 0:
        fails.append(f"{name}: the planted turn is not in the rack")
        return
    loose.first.click()
    got = page.evaluate("""() => {
      const b = window.__board;
      // Onto a component of an assembled item, so there is a footprint to turn.
      for (const s of b.state.slots) {
        const item = s.items.find(i => i.assembled && (i.turns ?? []).length > 1);
        if (!item) continue;
        const p = s.placed.find(p => item.pieces.includes(p.id));
        if (p) return { ok: b.onclaim(p.id), key: item.pieces.join(','),
                        turns: item.turns };
      }
      return { none: true };
    }""")
    if got.get("none"):
        fails.append(f"{name}: nothing on this board has anywhere to turn, so the "
                     f"spin cannot be shown")
        return
    if not got["ok"]:
        fails.append(f"{name}: could not bolt the turn onto a component")
        return
    page.wait_for_timeout(120)

    # Frames a second apart, and every one of them has to be an orientation
    # core named.
    #
    # **Sampled until it moves, not exactly twice.** The board turns on the
    # wall clock, so two samples a second apart are two samples of a moving
    # thing and can land on the same face of it: a two-entry cycle sampled at
    # 1060ms lands on the other face, and at 2120ms — one hiccup on a loaded
    # machine — lands back on the first. That was a still picture reported as a
    # broken feature, which is a flaky gate, which is worse than a red one. Six
    # samples of a cycle of at least two cannot all be the same unless nothing
    # is turning.
    seen = []
    for _ in range(6):
        seen.append(page.evaluate("""(key) => {
          window.__board.draw();
          const sp = window.__board.spun.find(s => s.key === key);
          return sp ? sp.cells.map(c => c.join(',')).sort().join(' ') : null;
        }""", got["key"]))
        if len(set(seen)) > 1:
            break
        page.wait_for_timeout(1060)
    if any(x is None for x in seen):
        fails.append(f"{name}: the board drew no footprint for the spinning item")
        return
    legal = {" ".join(sorted(f"{c[0]},{c[1]}" for c in cells)) for cells in got["turns"]}
    for drawn in seen:
        if drawn not in legal:
            fails.append(f"{name}: the board drew {drawn!r}, which is not one of the "
                         f"{len(legal)} orientations core named")
            return
    if len(set(seen)) == 1:
        fails.append(f"{name}: {len(seen)} frames a second apart drew the same footprint "
                     f"({seen[0]!r}), so nothing is turning")

    # And the card says what it is worth, in a number.
    said = page.evaluate("""(key) => {
      const el = [...document.querySelectorAll('#panel-yours .made-item')]
        .find(e => e.dataset.key === key);
      return el ? el.textContent : null;
    }""", got["key"])
    if not said or "turns every second" not in said:
        fails.append(f"{name}: the card does not say the item turns: {said!r}")


def strip_the_boards(body):
    """Take everything off all five grids, and the locks with them.

    **All five, not the one being planted on.** These checks run late in the
    walk on a character who has fought fifty times, and since M9 a character who
    has fought can have *earned* a set — the drops are the block's whole point.
    So a plant that cleared only its own grid was asking `Character::rules` a
    question about the board it had just made and getting an answer about the
    board the walk had made, and CI caught exactly that: two thirds of the
    Mandate "still granted a rule", which was the Toad's Own Frame sitting in a
    chest nobody had looked at, and the lake let a dry character through for the
    same reason.

    A planted board check is about what was planted. This is what makes that
    true.
    """
    for _, board in body["character"]["boards"]:
        board["placed"] = []
        board["enchanted"] = []
    body["character"]["locks"] = []
    body["character"]["enchanted"] = []
    body.setdefault("world", {})["map"] = ""


def seat_a_set(body, pieces, slot):
    """Put a whole set on an otherwise empty board.

    `pieces` is `[(name, x, y)]`. The registry is written whole and in order,
    so a component appended to it is a `PieceId` at that index — which is what
    `owned` and every board placement are.
    """
    strip_the_boards(body)
    reg = body["character"].setdefault("registry", [])
    owned = body["character"].setdefault("owned", [])
    board = next(b for b in body["character"]["boards"] if b[0] == slot)[1]
    board["rows"] = max(board.get("rows", 3), 3)
    for piece, x, y in pieces:
        reg.append({"def": piece, "rot": 0})
        owned.append(len(reg) - 1)
        board["placed"].append([len(reg) - 1, x, y])


MANDATE = [("Ratskin Material", 0, 0), ("Ratskin Mold", 2, 0), ("Rat Signet", 4, 0)]
TOAD_SET = [("Toad Frame", 0, 0), ("Toad Hide", 3, 0)]
# The smallest of the three instruments, laid out so all three parts touch. A
# shard is two cells wide, a magnet two tall, a lens one.
COMPASS = [("Map Shard", 0, 0), ("Glass Lens", 2, 0), ("Magnet", 0, 1)]
# Three shards and two handfuls of ground, laid out so all five touch.
GOLEM = [("Map Shard", 0, 0), ("Map Shard", 0, 1), ("Map Shard", 0, 2),
         ("Living Earth", 2, 0), ("Living Earth", 2, 2)]
# The Drover's Stride: a material, a mold and a plating is the greaves recipe
# entire, laid out so all three touch.
STRIDE = [("Drover's Material", 0, 0), ("Drover's Mold", 2, 0), ("Drover's Sole", 2, 2)]


def check_a_set_reads(page, name, fails):
    """Three pieces in the bag, seated, and the card names the item and its rule.

    Planted, because collecting a set is thirty-odd wins against one creature
    and what a browser has to prove is what the screen says once you have it.
    The engine's half — that the rule reaches the fight, that two thirds of a
    set grants nothing — is `tests/sets.rs`.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def show_the_board(pieces, stem):
        plant(page, base, lambda b: seat_a_set(b, pieces, "gloves"), stem=stem)
        # **Refresh the board.** Loading a file repaints the map and the panel
        # and deliberately not the packing board, which is not on screen when a
        # save is loaded — so without this the cards below are the ones the
        # previous plant drew, and the negative half of this check would read a
        # stale name as a name that would not go away.
        page.evaluate("() => window.__board.refresh()")
        page.wait_for_timeout(150)

    show_the_board(MANDATE, "set-probe")

    got = page.evaluate("""() => {
      const c = [...document.querySelectorAll('#panel-yours .made-item')]
        .find(e => (e.querySelector('b')?.textContent ?? '').includes('Mandate'));
      return {
        rules: (window.__character().rules ?? []).map(r => r.line),
        found: !!c,
        isSet: c ? c.classList.contains('is-set') : false,
        heads: c ? [...c.querySelectorAll('.head')].map(h => h.textContent.trim()) : [],
        set: c ? [...c.querySelectorAll('.set-rules li')].map(li => li.textContent.trim()) : [],
      };
    }""")
    if not got["found"]:
        fails.append(f"{name}: the whole set is on the board and no card is called the Mandate")
        return
    if not got["isSet"]:
        fails.append(f"{name}: the Mandate's card is not marked as a set")
    if not got["set"]:
        fails.append(f"{name}: the Mandate's card says nothing about what the set does: "
                     f"{got['heads']}")
    elif not any(ch.isdigit() for ch in " ".join(got["set"])):
        fails.append(f"{name}: the set's line names no number: {got['set']}")
    # And the sheet says it too, because a rule moves no bar and a screen that
    # never printed it could not be told from a rule that does nothing.
    if not got["rules"]:
        fails.append(f"{name}: wearing a whole set and the character reports no rules")
    sheet = page.evaluate(
        "() => [...document.querySelectorAll('#sheet li.rule')].map(li => li.textContent.trim())")
    if not sheet:
        fails.append(f"{name}: the sheet prints no rule for a set that grants one")

    # Two thirds of it is a glove. Break the thing the check guards and watch
    # the name go away — the negative half, in the same run.
    show_the_board(MANDATE[:2], "set-probe-2")
    still = page.evaluate("""() => ({
      named: [...document.querySelectorAll('#panel-yours .made-item b')]
        .some(b => b.textContent.includes('Mandate')),
      rules: (window.__character().rules ?? []).length,
    })""")
    if still["named"]:
        fails.append(f"{name}: two thirds of the set still answers to the whole name")
    if still["rules"]:
        fails.append(f"{name}: two thirds of the set still grants {still['rules']} rule(s)")


def check_the_toad_walks_on_water(page, name, fails):
    """The lake is a wall to everybody, and ground to a toad — all of it.

    Planted, and walked twice: stand on the grass at the top of the lake and
    press south. Wearing nothing, that is a wall. Wearing the whole Toad set,
    it is the rim, and then the middle, and then the grating with two hundred
    and six steps under it.

    **M11.4 widened the rule.** It opened the rim and not the middle for two
    blocks — the measurement is still in `tests/rules.rs` — and there is
    something under the middle now, so the set somebody ground three Bog Toads
    for is how you reach it before the Drambus Stack drains the whole thing.
    What this proves is that the allowance reaches `world::step` in a browser.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def dry(body):
        # **Wearing nothing**, or the character the walk built may already have
        # earned the very set this half is proving they do not have.
        strip_the_boards(body)
        body.setdefault("world", {})["at"] = [7, 8]

    def wet(body):
        dry(body)
        seat_a_set(body, TOAD_SET, "chest")

    plant(page, base, dry, stem="wade-probe")
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(250)
    dismiss_card(page)
    close_fight(page)
    if page.text_content("#coords").strip() != "7, 8":
        fails.append(f"{name}: water let a frame through at {page.text_content('#coords')!r}")

    plant(page, base, wet, stem="wade-probe-2")
    if not (window_rules := page.evaluate("() => (window.__character().rules ?? []).length")):
        fails.append(f"{name}: the toad set is on the board and grants nothing")
        return
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(250)
    dismiss_card(page)
    close_fight(page)
    at = page.text_content("#coords").strip()
    if at != "7, 9":
        fails.append(f"{name}: the toad set did not open the rim; stopped at {at!r}")
        return
    # And on into the middle, which is the widening.
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(250)
    dismiss_card(page)
    close_fight(page)
    at = page.text_content("#coords").strip()
    if at != "7, 10":
        fails.append(f"{name}: the middle of the lake is shut to a whole set; stopped at {at!r}")
        assert window_rules
        return

    # And the way down is out there, and stepping on it goes down.
    grate = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.id === 'the-way-under-the-lake')""")
    if not grate:
        fails.append(f"{name}: there is no way under the lake")
        assert window_rules
        return
    page.evaluate("(at) => window.__standHere(at)",
                  [grate["at"][0], grate["at"][1] - 1])
    close_fight(page)
    dismiss_card(page)
    page.keyboard.press("ArrowDown")
    page.wait_for_timeout(400)
    dismiss_card(page)
    where = page.evaluate("() => window.__world().id")
    if where != "under-the-lake":
        fails.append(f"{name}: walked onto the grating in a Toad set and stayed on {where!r}")
    else:
        # Entered wet, the two middle rows are water and the straight run down
        # the middle is shut — the long way round is what the early way costs.
        wet = page.evaluate("""() => {
          const w = window.__world();
          return w.rows.flat().filter(t => t === 'water').length;
        }""")
        if wet < 10:
            fails.append(f"{name}: under the lake came up with {wet} tiles of water in it")
    assert window_rules
    plant(page, base, lambda body: None, stem="wade-restore")


def check_the_van_appears_at_a_level(page, name, fails):
    """He is not there below level ten, and he is there at it.

    Planted, because levelling to ten is the whole game. What a browser has to
    prove is the half no test can: that the map **redraws** when the level
    lands. A place hidden until a level appears when you bank, and banking was
    the one moment nothing re-fetched the world — so the van would have been on
    the road, in the save, steppable and invisible.
    """
    # **And no town sells one.** The bench came off the town screen entirely —
    # a thing every town sells is not a thing you went and got.
    if page.query_selector("#bench-wrap") is not None:
        fails.append(f"{name}: the town still has a bench on it")

    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def at_level(xp):
        def edit(body):
            body["character"]["xp"] = xp
            body["character"]["class"] = "Berserker"
            body.setdefault("world", {})["map"] = ""
        return edit

    # Below it first. Planted rather than assumed about the walk, which by this
    # point has levelled well past ten.
    plant(page, base, at_level(0), stem="van-none")
    if page.evaluate("""() => (window.__world().places ?? [])
        .some(p => p.kind === 'bench')"""):
        fails.append(f"{name}: the van is on the map for a level-one character")

    # Ten is a lot of experience; a hundred thousand is comfortably past it.
    plant(page, base, at_level(100_000), stem="van-probe")
    van = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'bench')""")
    if not van:
        fails.append(f"{name}: past level ten and the van is still not on the map")
        return

    page.evaluate("() => document.getElementById('map').focus()")
    page.evaluate("(at) => window.__standAt(at)", [van["at"][0], van["at"][1] + 1])
    page.keyboard.press("ArrowUp")
    page.wait_for_timeout(300)
    dismiss_card(page)
    close_fight(page)
    if not page.is_visible("#vendor"):
        fails.append(f"{name}: walked onto the van's tile and no vendor opened")
        return
    try:
        shown = page.evaluate("""() => ({
          stock: [...document.querySelectorAll('#vendor-stock .wares b')].map(b => b.textContent),
          prose: document.querySelectorAll('#vendor-prose p').length,
          buyable: document.querySelectorAll('#vendor-stock .wares:not(:disabled)').length,
        })""")
        if not shown["stock"]:
            fails.append(f"{name}: the van has nothing on the table")
        if not shown["prose"]:
            fails.append(f"{name}: the van says nothing")
        if shown["buyable"]:
            # Buy one, and watch it go off the table for good.
            first = page.locator("#vendor-stock .wares:not(:disabled)").first
            bought = first.locator("b").text_content()
            first.click()
            page.wait_for_timeout(150)
            gone = page.evaluate("""(n) => {
              const b = [...document.querySelectorAll('#vendor-stock .wares')]
                .find(e => e.querySelector('b').textContent === n);
              return b ? { sold: b.classList.contains('sold'), off: b.disabled } : null;
            }""", bought)
            if not gone or not gone["sold"] or not gone["off"]:
                fails.append(f"{name}: bought {bought!r} and it is still for sale: {gone}")
            if not JSON_NULL != page.evaluate("() => window.__rack()"):
                fails.append(f"{name}: bought an ench and the rack does not know")
    finally:
        page.click("#vendor-close")
        page.wait_for_selector("#vendor", state="hidden", timeout=5000)


def check_a_broken_item_reads(page, name, fails):
    """An item fires once, stops, and the screen says which one.

    A bar that stops with nothing said about it reads as a bug in the playback
    rather than as the thing the player bought — the same reason `Event::Stunned`
    was given a variant of its own. The engine's half is `tests/breaking.rs`.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def swinging(body):
        strip_the_boards(body)
        body["character"]["class"] = "Recycler"
        body["character"]["enchs_owned"] = ["the-chonga-swing"]
        body["character"]["skills_taken"] = []

    plant(page, base, swinging, stem="swing-probe")
    page.evaluate("() => document.getElementById('map').focus()")
    town = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'town')""")
    page.evaluate("(at) => window.__standAt(at)", [town["at"][0] + 1, town["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not reach a town to pack in")
        return
    page.click("#pack")
    page.wait_for_selector("#fight", state="visible", timeout=8000)
    page.click("#preset")
    page.wait_for_timeout(150)
    # Bolt it onto a component of an assembled item, so there is something to
    # break.
    got = page.evaluate("""() => {
      const b = window.__board;
      const loose = document.querySelector('#rack-loose .wares[data-ench="the-chonga-swing"]');
      if (!loose) return { none: 'not in the rack' };
      loose.click();
      for (const s of b.state.slots) {
        const item = s.items.find(i => i.assembled);
        if (!item) continue;
        const p = s.placed.find(p => item.pieces.includes(p.id));
        if (p) return { ok: b.onclaim(p.id), item: item.name };
      }
      return { none: 'nothing assembled to bolt it to' };
    }""")
    if got.get("none") or not got.get("ok"):
        fails.append(f"{name}: could not bolt the swing on: {got}")
        close_fight(page)
        return
    on = page.evaluate("() => JSON.parse(window.__rack()).on.map(e => e.id)")
    if "the-chonga-swing" not in on:
        fails.append(f"{name}: the bolt reported success and the rack disagrees: {on}")
        close_fight(page)
        return
    close_fight(page)
    if page.is_visible("#town"):
        page.click("#leave")
        page.wait_for_selector("#town", state="hidden", timeout=5000)

    # **A fight long enough for the item to come round**, planted rather than
    # walked into. A weapon's bar is two to four seconds and this character
    # kills a rat in less than that — the first version of this check walked
    # into whatever the ground rolled and passed or failed on how much health
    # the creature happened to have. Downloaded *after* the bolt, so the save
    # carries it, and re-planted with something big standing in front of it.
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    armed = dl.value.path()

    def in_a_long_fight(body):
        body["encounter"] = {"enemy": "Rust Colossus", "at": body["world"]["at"]}

    plant(page, armed, in_a_long_fight, stem="swing-fight")
    page.wait_for_selector("#fight", state="visible", timeout=8000)
    page.click("#go")
    page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
    try:
        broke = page.evaluate("""() => {
          const log = JSON.parse(window.__fightJson());
          const es = log.entries ?? [];
          const e = es.find(x => x.kind === 'broke');
          if (!e) return { kinds: [...new Set(es.map(x => x.kind))],
                           fragile: (window.__character().rules ?? []).length };
          return { item: e.item, side: e.side,
                   fired: es.filter(x => x.kind === 'activate'
                          && x.item && x.item.startsWith(e.item)).length };
        }""")
        if "item" not in broke:
            fails.append(f"{name}: bolted the swing on and nothing ever broke: {broke}")
        elif broke["fired"] > 1:
            fails.append(f"{name}: {broke['item']} fired {broke['fired']} times and broke once")
    finally:
        page.click("#skip")
        page.wait_for_selector("#stage-result", state="visible", timeout=20000)
        page.click("#done")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)


def check_an_ench_you_cannot_use_is_still_shown(page, name, fails):
    """An errand pays an ench to everybody, so everybody has to be able to see it.

    Reported from a real session: *"the quest the frame that stands did not pay
    the yodregar index"*. It did — core hands it over, the save carries it, the
    town's receipt names it — and then the rack, which is the only screen an
    ench appears on, was hidden outright unless the character was a Kaklon
    Licensee. Paid, and invisible, which from where the player sits is not paid.

    `quest.rs` is deliberate about handing it over regardless: *a reward that
    vanished for three players in four would be worse than one they cannot use
    yet.* This is the half of that sentence the screen owed.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def holding(licensed):
        def edit(body):
            strip_the_boards(body)
            body["character"]["enchs_owned"] = ["the-yodregar-index"]
            # **A class, and not the licensee's.** Taking the class *off* is not
            # the same as not being a licensee: a level-five character with no
            # class is owed one, so the load opens the fork — which is the one
            # screen that does not come off, and everything after it clicks into
            # a modal. Gorillathon is a real answer that is not the Patent.
            body["character"]["class"] = "Recycler" if licensed else "Berserker"
        return edit

    def open_the_board():
        # Focus the map first. A keypress goes to whatever has focus, and after
        # a file load that is the file input — `check_the_spin_animates` learned
        # this and it is the same gesture.
        page.evaluate("() => document.getElementById('map').focus()")
        town = page.evaluate("""() => (window.__world().places ?? [])
            .find(p => p.kind === 'town')""")
        page.evaluate("(at) => window.__standAt(at)", [town["at"][0] + 1, town["at"][1]])
        page.keyboard.press("ArrowLeft")
        page.wait_for_timeout(300)
        if not page.is_visible("#town"):
            return False
        page.click("#pack")
        page.wait_for_selector("#fight", state="visible", timeout=8000)
        return True

    plant(page, base, holding(False), stem="ench-unlicensed")
    if not open_the_board():
        fails.append(f"{name}: could not reach a town to pack in")
        return
    try:
        got = page.evaluate("""() => ({
          rack: !document.getElementById('rack').hidden,
          named: [...document.querySelectorAll('#rack-loose b')].map(b => b.textContent),
          note: document.getElementById('rack-note').textContent,
          buttons: document.querySelectorAll('#rack-loose button').length,
          owned: JSON.parse(window.__rack()).loose.length,
        })""")
        if not got["owned"]:
            fails.append(f"{name}: the planted ench never reached the character")
            return
        if not got["rack"]:
            fails.append(f"{name}: unlicensed and holding an ench, and the rack is hidden")
        elif not any("Yodregar" in n for n in got["named"]):
            fails.append(f"{name}: the rack is up and does not name the ench: {got['named']}")
        if "licensee" not in got["note"].lower():
            fails.append(f"{name}: the rack does not say why it cannot be used: {got['note']!r}")
        if got["buttons"]:
            fails.append(f"{name}: offered {got['buttons']} click(s) core is going to refuse")
    finally:
        close_fight(page)

    # And a rack of nothing stays hidden, which is what hiding it was for.
    def empty(body):
        strip_the_boards(body)
        body["character"]["enchs_owned"] = []
        # **And no nodes.** A tree grants an ench now, and the check before this
        # one plants a Patent who has taken Bench Rights — so a character
        # "holding none" has to have taken none either, or the rack is right to
        # be showing one.
        body["character"]["skills_taken"] = []
        body["character"]["class"] = "Berserker"

    plant(page, base, empty, stem="ench-none")
    if not open_the_board():
        fails.append(f"{name}: could not reach a town the second time")
        return
    try:
        if page.evaluate("() => !document.getElementById('rack').hidden"):
            fails.append(f"{name}: an empty rack is on the screen of somebody who cannot use one")
    finally:
        close_fight(page)


def check_the_game_talks_in_one_place(page, name, fails):
    """Everything the game says lands in the strip, and the history keeps it.

    **M11.0.** The old slot was a `<p id="says">` under the save panel — a slot
    that existed because nothing owned it, which is the shape of thing that
    ships a feature invisible. This plants a refusal, finds it in the strip,
    opens HISTORY and finds it there too, and then checks the old element is
    *gone* rather than merely unused: a stray writer has to fail loudly.

    Broken first and watched failing, per the standing habit: with `log()`
    writing only the strip, the history half fails; with the old `<p>` back,
    the last assertion fails.
    """
    if page.evaluate("() => !!document.getElementById('says')"):
        fails.append(f"{name}: the old below-save slot is still in the page")

    # Something that costs nothing to provoke and always talks. The strip
    # keeps only the last few, so what is measured is the *newest line*, not
    # how many there are — it is capped and a length check would read as a
    # pass or a fail depending on how much had already happened.
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    dl.value.path()
    page.wait_for_timeout(200)
    strip = tape(page)
    if not strip:
        fails.append(f"{name}: the strip is empty after a whole walk")
        return
    latest = strip[-1]
    if "Saved" not in latest:
        fails.append(f"{name}: the save was written and the strip says {latest!r}")

    # The strip is the last few; the history is the sitting.
    page.click("#history-open")
    page.wait_for_selector("#history", state="visible", timeout=5000)
    whole = [t.strip() for t in page.eval_on_selector_all(
        "#history-list li", "els => els.map(e => e.textContent)")]
    if latest not in whole:
        fails.append(f"{name}: the history does not hold {latest!r}")
    # And it is not just the strip under another name: this runs at the end of
    # a whole walk, which has said far more than the strip's four lines.
    if len(whole) <= len(strip):
        fails.append(f"{name}: the history ({len(whole)}) holds no more than the strip "
                     f"({len(strip)}), so it is the strip in a bigger box")
    # It is the top-most thing while it is up, like the tree and the quest log.
    # Pointed at the *last* line, because a long sitting scrolls the top of
    # the list out of the viewport and `elementFromPoint` outside it is
    # nothing at all — which reads as a failure and is only a scroll.
    box = page.evaluate("""() => {
      const li = document.querySelector('#history-list li:last-child');
      const r = li.getBoundingClientRect();
      const el = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
      return el ? (el.closest('#history') ? 'history' : el.id || el.className) : 'nothing';
    }""")
    if box != 'history':
        fails.append(f"{name}: something covers the history: {box!r}")
    page.keyboard.press("Escape")
    page.wait_for_selector("#history", state="hidden", timeout=5000)


def check_the_north_is_shut(page, name, fails):
    """A level-one character cannot walk into a region of two-thousand-rated
    creatures, and is told what the road wants.

    Planted at level one on the crossing's own tile, because the point is the
    step north out of it. The five-regions-at-nine shape is
    `tests/crossings.rs`; what a browser has to prove is that the refusal
    reaches a player where they will read it, rather than only the one-second
    flash along the bottom of the map.
    """
    cross = page.evaluate("""() => (window.__world().places ?? [])
        .filter(p => p.kind === 'crossing').sort((a, b) => b.at[1] - a.at[1])[0]""")
    if not cross:
        fails.append(f"{name}: the overworld has no crossing on it")
        return
    if not cross.get("needs_level"):
        fails.append(f"{name}: {cross['id']} asks for no level")
        return

    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def at_the_crossing(level_xp):
        def edit(body):
            body["character"]["xp"] = level_xp
            body.setdefault("world", {})["at"] = list(cross["at"])
            body["world"]["map"] = ""
        return edit

    plant(page, base, at_the_crossing(0), stem="north-probe")
    if page.text_content("#coords").strip() != f"{cross['at'][0]}, {cross['at'][1]}":
        fails.append(f"{name}: could not stand on the crossing")
        return
    page.keyboard.press("ArrowUp")
    page.wait_for_timeout(250)
    dismiss_card(page)
    close_fight(page)
    if page.text_content("#coords").strip() != f"{cross['at'][0]}, {cross['at'][1]}":
        fails.append(f"{name}: a level-one character walked north past {cross['id']}")
        return
    said_it = last_said(page)
    if not said_it:
        fails.append(f"{name}: the crossing refused in silence")
    elif str(cross["needs_level"]) not in said_it:
        fails.append(f"{name}: the refusal does not name the level it wants: {said_it!r}")

    # And at the level it asks for, it is a road. 100,000 experience is
    # comfortably past whatever the number is.
    plant(page, base, at_the_crossing(100_000), stem="north-probe-2")
    page.keyboard.press("ArrowUp")
    page.wait_for_timeout(300)
    dismiss_card(page)
    close_fight(page)
    y = int(page.text_content("#coords").split(",")[1])
    if y >= cross["at"][1]:
        fails.append(f"{name}: the crossing would not open at any level; still at row {y}")


def check_scouting_is_earned(page, name, fails):
    """The map's danger and its odds are a skill, and `#numbers` is gone.

    `Show the numbers` was a debug overlay that shipped, and it handed the
    region's danger and every tile's odds to everybody for nothing — which
    makes a node granting them a node granting nothing. The reading is the
    Worm-Fact Keeper's now: the class that files what a thing was doing.

    Planted for the second half, because reaching the node by play means
    levelling to five, being offered the right one of three classes and
    spending a point on it — and a check that only fires when the fork happens
    to deal you a Bloodletter is a check that stops firing.
    """
    if page.query_selector("#numbers") is not None:
        fails.append(f"{name}: #numbers is still on the page, so the skill grants nothing")
    shut = page.evaluate("""() => ({
      scouting: JSON.parse(window.__position()).scouting,
      chance: document.getElementById('chance').textContent,
      danger: document.getElementById('danger').textContent,
      button: !!document.querySelector('#scout:not([hidden])'),
      chances: (window.__world().chances ?? []).length,
    })""")
    if shut["scouting"]:
        fails.append(f"{name}: this character never took the node and can read the map")
    else:
        if any(c.isdigit() for c in shut["chance"]) or any(c.isdigit() for c in shut["danger"]):
            fails.append(f"{name}: unscouted and the panel prints {shut['chance']!r} / "
                         f"{shut['danger']!r}")
        if shut["button"]:
            fails.append(f"{name}: unscouted and the odds button is on the panel")
        if shut["chances"]:
            fails.append(f"{name}: unscouted and the map shipped {shut['chances']} rows of odds")

    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def keeper(body):
        body["character"]["class"] = "Bloodletter"
        body["character"]["skills_taken"] = list(
            body["character"].get("skills_taken", [])) + ["w-survey"]

    plant(page, base, keeper, stem="scout-probe")
    open_ = page.evaluate("""() => ({
      scouting: JSON.parse(window.__position()).scouting,
      chance: document.getElementById('chance').textContent,
      danger: document.getElementById('danger').textContent,
      button: !!document.querySelector('#scout:not([hidden])'),
      chances: (window.__world().chances ?? []).length,
    })""")
    if not open_["scouting"]:
        fails.append(f"{name}: took the scouting node and core still says no")
        return
    if not any(c.isdigit() for c in open_["chance"]):
        fails.append(f"{name}: scouting, and the odds read {open_['chance']!r}")
    if not any(c.isdigit() for c in open_["danger"]):
        fails.append(f"{name}: scouting, and the danger reads {open_['danger']!r}")
    if not open_["button"]:
        fails.append(f"{name}: scouting, and there is no way to see the per-tile odds")
        return
    if not open_["chances"]:
        fails.append(f"{name}: scouting, and the map carries no per-tile odds")
    page.click("#scout")
    if page.get_attribute("#scout", "aria-pressed") != "true":
        fails.append(f"{name}: the odds overlay did not turn on")
    page.click("#scout")


def check_the_replay_reports_a_curse(page, name, fails):
    """A curse lands and the screen moves.

    `Event::Cursed`, `Warded` and `Stunned` used to fall into the replay's
    `_ => ("other", ...)` arm, so a Whisperling could stack frost on you for a
    whole fight and nothing on the panel said a word. Curses have been in the
    engine since the fork; the screen was the part that was missing.

    Driven against **named** creatures rather than whatever the walk met: the
    encounter is state, so planting one is the only way to ask this question of
    a particular fight. Two of them, because a curse and a stun are two arms —
    a stun rides on one named item and a curse stacks on the fighter, and a
    check that only ever saw one would let the other rot. Both are creatures
    this map actually holds.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    for who, want in (("Whisperling", "cursed"), ("Bone Archer", "stunned")):
        def against(body, who=who):
            body["encounter"] = {"enemy": who, "at": body["world"]["at"]}

        plant(page, base, against, stem="curse-probe")
        page.wait_for_selector("#fight", state="visible", timeout=8000)

        got = page.evaluate("""(want) => {
          const log = JSON.parse(window.__fightJson());
          const lands = log.entries.filter(e => e.kind === want);
          if (!lands.length) return { none: true, kinds: [...new Set(log.entries.map(e => e.kind))] };
          const first = lands[0];
          const side = first.side === 'player' ? 'player' : 'enemy';
          const r = window.__replay;
          r.load(log);
          // At the moment it lands.
          r.t = first.at;
          const up = r.at(r.track[side]).chips.filter(c => c.until > r.t);
          // And after the last of them has run out.
          const latest = Math.max(...up.map(c => c.until), 0);
          r.t = latest + 1;
          const after = r.at(r.track[side]).chips.filter(c => c.until > r.t);
          // The stack count has to be the log's, not a tally the page kept.
          const most = Math.max(...lands.map(e => e.index));
          r.t = log.duration_ms;
          const ever = r.track[side].flatMap(row => row[5] ?? []);
          r.draw();
          return { side, at: first.at, up, after: after.length, most,
                   // What the log said landed. For a curse the entry carries
                   // the kind; for a stun it carries the item that stopped,
                   // and which item it was is the whole reason a stun is its
                   // own event rather than a `Cursed`.
                   want: want === 'cursed' ? first.item : 'stun',
                   onItem: want === 'stunned' ? first.item : null,
                   peak: Math.max(...ever.map(c => c.stacks), 0) };
        }""", want)
        if got.get("none"):
            fails.append(f"{name}: a {who} landed no {want}; kinds were {got['kinds']}")
        else:
            if not got["up"]:
                fails.append(f"{name}: a {want} landed at {got['at']}ms and no chip was up")
            else:
                c = got["up"][0]
                for k in ("kind", "stacks", "until", "effect"):
                    if k not in c:
                        fails.append(f"{name}: a {who}'s chip is missing {k}: {c}")
                if c.get("until", 0) <= got["at"]:
                    fails.append(f"{name}: a {who}'s chip landed already expired: {c}")
                if not str(c.get("effect", "")).strip():
                    fails.append(f"{name}: the chip says {c.get('kind')!r} and not what it does")
            named = [c for c in got["up"] if c.get("kind") == got["want"]
                     and (got["onItem"] is None or c.get("item") == got["onItem"])]
            if not named:
                fails.append(f"{name}: the log landed {got['want']!r}"
                             f"{'' if got['onItem'] is None else ' on ' + got['onItem']}"
                             f" and the chips up are {got['up']}")
            if got["after"] != 0:
                fails.append(f"{name}: {got['after']} of a {who}'s chip(s) outlived their clock")
            # Read, never derived: the deepest stack drawn is the deepest the
            # log reported, not one the page counted for itself.
            if want == "cursed" and got["peak"] != got["most"]:
                fails.append(f"{name}: the log reported {got['most']} stacks and the panel "
                             f"drew {got['peak']}")
        close_fight(page)


def check_the_advance_button_does_not_move(page, name, fails):
    """The primary button is in the same place on all three fight stages.

    The three stages are 15, 19 and 3 lines tall, so a button at the bottom of
    whichever one was showing put Fight, Skip to the end and Walk on at three
    different heights — the whole advance gesture was a moving target and the
    cursor had to chase it.

    Measured rather than read: this is a layout claim and layout is the one
    thing reading the source cannot settle.
    """
    def box():
        return page.evaluate("""() => {
          const b = [...document.querySelectorAll('#fight-bar .advance')]
            .find(e => !e.hidden && e.offsetParent !== null);
          if (!b) return null;
          const r = b.getBoundingClientRect();
          return { id: b.id, x: Math.round(r.x), y: Math.round(r.y),
                   w: Math.round(r.width), h: Math.round(r.height) };
        }""")

    was = page.evaluate(
        "() => ['board','replay','result']"
        ".find(s => !document.getElementById('stage-' + s).hidden) ?? 'board'")
    seen = {}
    try:
        for which in ("board", "replay", "result"):
            page.evaluate("(w) => window.__stage(w)", which)
            page.wait_for_timeout(60)
            got = box()
            if not got:
                fails.append(f"{name}: the {which} stage shows no advance button at all")
                return
            seen[which] = got
    finally:
        # Put the screen back where the walk left it. This check moves the
        # stage in order to measure it, and a walk that carried on from the
        # result stage would be clicking buttons that are not there.
        page.evaluate("(w) => window.__stage(w)", was)
    ids = {v["id"] for v in seen.values()}
    if len(ids) != 3:
        fails.append(f"{name}: the three stages share {len(ids)} advance button(s): {seen}")
    first = seen["board"]
    for which, got in seen.items():
        for k in ("x", "y", "w", "h"):
            if abs(got[k] - first[k]) > 2:
                fails.append(f"{name}: the advance button's {k} is {got[k]} on the {which} "
                             f"stage and {first[k]} on the board: {seen}")
                return


def check_the_log_points_somewhere(page, name, fails):
    """The log lists what is on you, and pinning one lights the map.

    The errands existed and there was nowhere to see them all, and no way to
    find out where a Whisperling lives. Where an errand points is core's answer
    — a page working out which regions hold a creature would be a second copy
    of the pools, which are the one thing on this map that gets retuned.
    """
    page.click("#errands-open")
    page.wait_for_selector("#log", state="visible", timeout=8000)
    rows = page.locator("#log-list .wares")
    if rows.count() == 0:
        fails.append(f"{name}: the walk has taken errands and the log is empty")
        page.click("#log-close")
        return
    said = page.locator("#log-list .wares .meta").first.text_content() or ""
    if "·" not in said:
        fails.append(f"{name}: the log's first row says {said!r}, which names no destination")

    # Hovering answers before anything is committed to.
    first = rows.first
    first.hover()
    page.wait_for_timeout(120)
    hovered = page.evaluate("() => window.__hoverGuide()")
    if not hovered:
        fails.append(f"{name}: hovering an errand lit nothing on the map")

    # And pinning it outlives the screen, which is the whole difference between
    # a reference and a tool.
    live = page.locator("#log-list .wares:not(:disabled)")
    if live.count() == 0:
        fails.append(f"{name}: every errand in the log is finished")
        page.click("#log-close")
        return
    which = live.first.get_attribute("data-errand")
    live.first.click()
    page.wait_for_timeout(150)
    log = page.evaluate("() => window.__log()")
    if log["pinned"] != which:
        fails.append(f"{name}: pinned {which} and the save says {log['pinned']!r}")
    guide = page.evaluate("(id) => window.__guide(id)", which)
    if guide and not (guide["places"] or guide["regions"]):
        fails.append(f"{name}: {which} is pinned and points at no tile at all")
    page.click("#log-close")
    page.wait_for_selector("#log", state="hidden", timeout=5000)
    after = page.evaluate("() => window.__log().pinned")
    if after != which:
        fails.append(f"{name}: the pin came off when the log closed ({after!r})")
    # A second click on the same one takes it off, and one at a time is the rule.
    page.click("#errands-open")
    page.wait_for_selector("#log", state="visible", timeout=8000)
    page.locator(f'#log-list .wares[data-errand="{which}"]').click()
    page.wait_for_timeout(120)
    if page.evaluate("() => window.__log().pinned") is not JSON_NULL:
        fails.append(f"{name}: pinning the pinned errand did not take it off")
    page.click("#log-close")
    page.wait_for_selector("#log", state="hidden", timeout=5000)


def check_the_cave_is_shut_until_it_is_not(page, name, fails):
    """A gate wants a key, says so, and opens once you have one.

    Driven by planting a save on the gate's doorstep rather than by walking
    Marbulon's questline, which is twenty minutes of fighting. The questline
    itself is `tests/dungeon.rs`; what a browser has to prove is that the door
    refuses, says what it wants, and lets you through when it is answered.
    """
    gate = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'gate' && p.needs)""")
    if not gate:
        fails.append(f"{name}: the overworld has no locked gate on it")
        return

    def stand_beside(save_path, extra=None):
        save = json.loads(Path(save_path).read_text())
        body = save.get("state", save)
        body.setdefault("world", {})["at"] = [gate["at"][0] - 1, gate["at"][1]]
        body["world"]["map"] = ""
        if extra:
            extra(body)
        out = Path(save_path).parent / "gate-probe.json"
        out.write_text(json.dumps(save))
        page.set_input_files("#file", str(out))
        page.wait_for_timeout(400)

    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    # Shut.
    stand_beside(base)
    page.keyboard.press("ArrowRight")
    page.wait_for_timeout(250)
    said_it = last_said(page)
    if not said_it:
        fails.append(f"{name}: a locked gate said nothing")
    if page.evaluate("() => window.__world().id") == 'the-great-gear-cave':
        fails.append(f"{name}: the locked gate let you through")

    # And open, with the key in the bag.
    def add_key(body):
        reg = body["character"].setdefault("registry", [])
        reg.append({"def": gate["needs"], "rot": 0})
        body["character"]["owned"].append(len(reg) - 1)

    stand_beside(base, add_key)
    page.keyboard.press("ArrowRight")
    page.wait_for_timeout(300)
    inside = page.evaluate("() => window.__world().id")
    if inside == "west-bambulon":
        fails.append(f"{name}: the gate would not open with {gate['needs']} in hand")
        return
    # The dungeon is short, has a way back, and something standing at the end.
    got = page.evaluate("""() => {
      const w = window.__world();
      const walk = w.walk.flat().filter(Boolean).length;
      return { id: w.id, walk,
               gates: w.places.filter(p => p.kind === 'gate').length,
               bosses: w.places.filter(p => p.kind === 'boss').length };
    }""")
    if got["walk"] > 40:
        fails.append(f"{name}: the first dungeon is {got['walk']} tiles, which is a second overworld")
    if got["gates"] < 1:
        fails.append(f"{name}: {got['id']} has no way out")
    if got["bosses"] != 1:
        fails.append(f"{name}: {got['id']} has {got['bosses']} things at the end of it")


def check_fatigue_wears_and_mends(page, name, fails):
    """A fight tires you and the panel says so.

    **A defeat walks you home, and home is a town, and a town takes the
    tiredness off.** So a lost fight ends rested — the wear happened and the
    walk undid it — and this check has to know which fight it is looking at
    rather than reporting the two rules meeting as a bug. That a town mends at
    all is `check_a_town_takes_the_tiredness_off`, walked on its own.
    """
    lost = "You stop moving" in (page.text_content("#result-title") or "")
    before = page.evaluate("() => window.__character()")
    if lost:
        if before["fatigue"] != 0:
            fails.append(f"{name}: walked home beaten and arrived {before['fatigue']}% worn")
        return
    if before["fatigue"] <= 0:
        fails.append(f"{name}: fights have happened and nothing is worn off")
        return
    if "not at all" in (page.text_content("#fatigue") or ""):
        fails.append(f"{name}: {before['fatigue']}% worn and the panel says otherwise")
    worn = next((s["n"] for s in before["stats"] if s["label"] == "max health"), 0)
    if worn >= before["rested_health"]:
        fails.append(f"{name}: {before['fatigue']}% worn and the maximum did not move")


def check_a_town_takes_the_tiredness_off(page, name, fails):
    """Walking into a town undoes the wear, and says so.

    Not a rest, and there still is not one: health resets at every bell, so a
    rest would restore something that was never spent. What a town undoes is
    the one thing a fight does spend, and it is what makes the walk home worth
    taking rather than a formality.

    Planted, because the walk cannot be relied on to arrive at a town tired —
    and a check that only fires when it happens to is a check that stops
    firing.
    """
    town = page.evaluate("""() => (window.__world().places ?? [])
        .find(p => p.kind === 'town')""")
    if not town:
        fails.append(f"{name}: the overworld has no town on it")
        return
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    def tired(body):
        body["world"]["at"] = [town["at"][0] + 1, town["at"][1]]
        body["world"]["map"] = ""
        body["character"]["fatigue"] = 32

    plant(page, base, tired, stem="tired-probe")
    if page.evaluate("() => window.__character().fatigue") != 32:
        fails.append(f"{name}: the planted save did not arrive worn out")
        return
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(350)
    after = page.evaluate("() => window.__character()")
    if after["fatigue"] != 0:
        fails.append(f"{name}: walked into a town 32% worn and came out {after['fatigue']}%")
    if not page.is_visible("#town"):
        fails.append(f"{name}: the step onto the town opened no town")
    else:
        page.click("#leave")
        page.wait_for_selector("#town", state="hidden", timeout=5000)
    if "not at all" not in (page.text_content("#fatigue") or ""):
        fails.append(f"{name}: mended, and the panel still says "
                     f"{page.text_content('#fatigue')!r}")


def check_turning_in_hand(page, name, fails):
    """A component turned in hand actually turns.

    Reported from a real session: picking a piece up and right-clicking left it
    the shape it was. Core rotated it correctly every time — the board had
    copied the shape at pick-up and never read it again, so the cursor and the
    footprint kept drawing the old one.

    Checked against core rather than against a previous frame, because "it
    changed" is not the claim; "it is what core says it is" is.
    """
    got = page.evaluate("""() => {
      const b = window.__board;
      if (!b || !b.state) return null;
      // **A shape a quarter turn actually moves.** A two-by-two square comes
      // back to itself, and the "it changed" assertion below is the negative
      // test that keeps the real one honest — so a symmetric piece reports a
      // working board as a broken one. The board packs tightly now and the bag
      // is often one piece deep, so the choice has to be made when taking one
      // off the board rather than afterwards.
      const turns = (p) => {
        const xs = p.cells.map(c => c[0]), ys = p.cells.map(c => c[1]);
        return (Math.max(...xs) - Math.min(...xs)) !== (Math.max(...ys) - Math.min(...ys));
      };
      for (const s of b.state.slots) {
        const p = s.placed.find(p => p.cells.length > 1 && turns(p))
               ?? s.placed.find(p => p.cells.length > 1);
        if (p) { b.api.pickUp(p.id); b.refresh(); break; }
      }
      const loose = b.state.bag.find(p => p.cells.length > 1 && turns(p))
                 ?? b.state.bag.find(p => p.cells.length > 1);
      if (!loose) return { skipped: true };
      b.held = { id: loose.id, from: null, name: loose.name, slot: loose.slot };
      const before = JSON.stringify(b.heldCells());
      b.rotateHeld();
      const after = JSON.stringify(b.heldCells());
      const core = JSON.stringify(
        b.state.bag.find(p => p.id === loose.id)?.cells ?? null);
      b.held = null; b.legal = null; b.refresh();
      return { before, after, core, name: loose.name };
    }""")
    if got is None:
        fails.append(f"{name}: could not reach the board to turn anything")
        return
    if got.get("skipped"):
        return
    if got["after"] != got["core"]:
        fails.append(f"{name}: turning {got['name']} in hand left the board drawing "
                     f"{got['after']} while core says {got['core']}")
    elif got["after"] == got["before"]:
        fails.append(f"{name}: turning {got['name']} changed nothing ({got['before']})")


def check_fit_preview(page, name, fails):
    """The fit preview comes from core, not from the page.

    Picks up each loose piece in turn and compares what the board painted green
    against what `legal_anchors` returned. If the page ever works out for itself
    which cells are legal there are two rulebooks, and this is where it shows.

    A piece that fits nowhere is a fine answer and not a failure: the frames
    start three rows tall and plenty of components are four cells long. What
    would be a failure is the two lists disagreeing.
    """
    got = page.evaluate("""() => {
      const b = window.__board; if (!b || !b.state) return null;
      const out = [];
      for (const loose of b.state.bag.slice(0, 12)) {
        b.held = { id: loose.id, from: null, name: loose.name, slot: loose.slot };
        b.askLegal(loose.slot);
        const drawn = [...b.legal].sort();
        const core = JSON.parse(window.__legalAnchors(loose.id, loose.slot))
                       .map(([x, y]) => `${x},${y}`).sort();
        out.push({ name: loose.name, drawn, core });
      }
      b.held = null; b.legal = null;
      return out;
    }""")
    if got is None:
        fails.append(f"{name}: could not reach the board to check the fit preview")
        return
    if not got:
        return  # nothing loose to check with; the board tests cover the rest
    for row in got:
        if row["drawn"] != row["core"]:
            fails.append(f"{name}: the fit preview for {row['name']} disagrees with core "
                         f"({len(row['drawn'])} cells drawn, {len(row['core'])} legal)")
            return
    # A smoke test, and only when there is enough in the bag for it to mean
    # something. With one loose component it is a coin flip on which piece the
    # shop happened to sell — a four-cell-tall one fits nowhere on a three-row
    # frame, which is correct. CI failed on exactly that before this guard.
    if len(got) >= 5 and not any(row["core"] for row in got):
        fails.append(f"{name}: none of {len(got)} loose components fits anywhere at all")


def check_a_stale_autosave(browser, name):
    """A save from an older build does not wedge the player in the scenery.

    Reported from a real session. `world` is `#[serde(default)]` so that saves
    written before M2 still open, and a default `WorldState` stands at (0, 0) —
    which on this map is rock in the top-left corner. A returning player whose
    browser held one of those arrived unable to move in any direction.

    Planted rather than waited for: this is the exact file, and the check is
    that the game repairs it rather than trusting it.
    """
    fails = []
    BANKINGS.clear()
    ctx = browser.new_context()
    page = ctx.new_page()
    page.on("pageerror", lambda e: fails.append(f"{name}: pageerror: {e}"))
    page.goto(ORIGIN + "/", wait_until="networkidle")
    page.wait_for_function("document.getElementById('coords').textContent !== '—'", timeout=20000)

    # One step, so an autosave exists, then strip its world the way an older
    # build's file has it.
    page.keyboard.press("ArrowRight")
    leave_town(page) or dismiss_card(page)
    if page.is_visible("#fight"):
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)
    stale = page.evaluate("""() => {
      const v = JSON.parse(localStorage.getItem('gm2d.autosave'));
      delete v.state.world;
      return JSON.stringify(v);
    }""")
    page.evaluate("s => localStorage.setItem('gm2d.autosave', s)", stale)
    page.reload(wait_until="networkidle")
    page.wait_for_function("document.getElementById('coords').textContent !== '—'", timeout=20000)

    # **Ask the map, do not list the walls.** `("rock", "water")` was a copy of
    # the terrain table written in Python, and the Treyway added two more walls.
    terrain = page.text_content("#terrain")
    if not page.evaluate("""() => {
      const w = window.__world();
      const [x, y] = (document.getElementById('coords').textContent || '0, 0')
        .split(',').map(v => parseInt(v, 10));
      return !!(w.walk[y] && w.walk[y][x]);
    }"""):
        fails.append(f"{name}: a stale autosave spawned the player in {terrain}")

    walked = page.text_content("#walked")
    for key in ("ArrowRight", "ArrowUp", "ArrowLeft", "ArrowDown"):
        page.keyboard.press(key)
        if page.is_visible("#fight"):
            page.click("#run")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
        leave_town(page) or dismiss_card(page)
    if page.text_content("#walked") == walked:
        fails.append(f"{name}: after a stale autosave the player could not move "
                     f"in any direction from {page.text_content('#coords')}")
    ctx.close()
    return fails


def walk_the_gate(browser, name, fails=None):
    """Returns a list of failures; empty means the gate is passed.

    `fails` is passed in so that a crash does not take the findings with it.
    A check that reported a real failure and left a screen up used to kill the
    next check on a click it could not land, and the whole list went unprinted
    — so the run reported a Playwright traceback and not the one sentence that
    said what was wrong.
    """
    fails = [] if fails is None else fails
    problems, offsite = [], []
    ctx = browser.new_context(accept_downloads=True)
    page = ctx.new_page()
    page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}")
            if m.type == "error" else None)
    page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
    page.on("request", lambda r: offsite.append(r.url)
            if not r.url.startswith(ORIGIN) else None)

    page.goto(ORIGIN + "/", wait_until="networkidle")
    page.wait_for_function("document.getElementById('coords').textContent !== '—'", timeout=20000)

    # The map has to be drawn, or nothing below is testing the world.
    painted = page.evaluate(
        "() => { const c = document.getElementById('map');"
        " const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data;"
        " let n = 0; for (let i = 3; i < d.length; i += 4) if (d[i] > 0) n++; return n; }")
    if painted < 640 * 640 * 0.9:
        fails.append(f"{name}: the canvas is {painted} opaque pixels of {640*640}")

    # --- the map refuses what it should --------------------------------------
    # Into the map's southern edge, which is rock and must refuse rather than
    # wrap. Done first, before anything is bought or fought, so the player is
    # still standing on the town they started from.
    for _ in range(4):
        page.keyboard.press("ArrowDown")
        if page.is_visible("#fight"):
            page.click("#run")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
        dismiss_card(page)
    y = int(page.text_content("#coords").split(",")[1])
    if y > 18:
        fails.append(f"{name}: walked to row {y}, which is off the map or into rock")

    # --- the town: buy something, and pack it ---------------------------------
    # The starting tile is a town, so leaving it and coming back opens the shop.
    # Stepping off can start a fight, which has to be walked away from before
    # the next keypress lands — and each retry moves the player another tile, so
    # the way back is "west until the town opens" rather than a fixed count.
    def step_out(key):
        page.keyboard.press(key)
        if page.is_visible("#fight"):
            page.click("#run")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
        dismiss_card(page)

    step_out("ArrowRight")
    for _ in range(12):
        if page.is_visible("#town"):
            break
        step_out("ArrowLeft")

    if not page.is_visible("#town"):
        fails.append(f"{name}: stepping back onto the starting town opened no town")
    else:
        check_a_component_is_a_shape(page, name, fails)
        check_the_errand_board(page, name, fails)
        check_the_shelf_is_the_shelf(page, name, fails)
        check_the_barrel_is_under_the_counter(page, name, fails)
        purse = int(page.text_content("#town-gold"))
        wares = page.locator("#shelf .wares:not(:disabled)")
        if wares.count() == 0:
            # **Not a failure since M12.1a, and this is the retargeting.** The
            # shelf charges five times the catalogue now precisely so that it
            # stops being where a beginner shops; with ten Fnorp there is
            # nothing on it, and that is the tier working. What must never be
            # empty is the *counter* — which is the barrel, and
            # `check_the_barrel_is_under_the_counter` is where that is asked.
            # The same move the two M4 soft-lock guards in `avail.rs` made.
            if page.locator("#barrel .wares:not(:disabled)").count() == 0:
                fails.append(f"{name}: nothing in the whole shop is affordable "
                             f"with {purse} Fnorp, shelf or barrel")
        else:
            # **§C.3, and it is not trivially true any more.** The shelf
            # charges a mark-up over the catalogue since M12.1a, so "the screen
            # shows the price actually charged" is a rule with a way to be
            # wrong: the figure on the button and the figure taken out of the
            # purse have to be the same number.
            shown = "".join(c for c in (wares.first.locator(".cost").text_content() or "")
                            if c.isdigit())
            wares.first.click()
            page.wait_for_timeout(80)
            after = int(page.text_content("#town-gold"))
            if after >= purse:
                fails.append(f"{name}: buying cost nothing ({purse} -> {after})")
            elif not shown:
                fails.append(f"{name}: a shelf line shows no price")
            elif purse - after != int(shown):
                fails.append(f"{name}: the shelf said {shown} Fnorp and took "
                             f"{purse - after} ({purse} -> {after})")
        page.click("#pack")
        page.wait_for_selector("#fight", state="visible", timeout=8000)
        page.click("#preset")
        made = page.text_content("#fight-yours")
        if made in (None, "", "0", "—"):
            fails.append(f"{name}: auto-packing the starting kit assembled {made}")
        check_fit_preview(page, name, fails)
        check_turning_in_hand(page, name, fails)
        check_the_card_halves(page, name, fails)
        check_hovering_an_item(page, name, fails)
        check_the_bag_shows_shapes(page, name, fails)
        check_a_grid_says_what_it_takes(page, name, fails)
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)

    # --- an event that says what it pays ------------------------------------
    # Stood on rather than walked into: the box is the milestone's whole
    # deliverable and a check that waits for the ground to roll one is a check
    # that runs on some seeds and not others.
    # **Stood beside and stepped onto.** `__standAt` plants a position; a card
    # opens on a *step*, which is the same reason every other planted check in
    # this file stands one tile east and presses west.
    page.evaluate("(at) => window.__standAt(at)", [17, 14])
    page.wait_for_timeout(150)
    page.click("#map")
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#card"):
        fails.append(f"{name}: stepping onto the counted heap opened no card")
    else:
        try:
            check_a_choice_says_what_it_pays(page, name, fails)
        finally:
            # **A check that opens a screen closes it on every path out.** One
            # that returned early once left the screen up, the next check died
            # on a click it could not land, and the whole failure list went
            # unprinted.
            dismiss_card(page)

    # The tree lives on the map, behind nothing — checked here rather than up
    # in the packing block, where the fight screen is over the top of it.
    check_every_skill_says_what_it_does(page, name, fails)
    check_the_sheet_says_what_you_are(page, name, fails)

    # --- a fight -------------------------------------------------------------
    if not walk_until_a_fight(page):
        fails.append(f"{name}: never met anything in 200 steps")
    else:
        creature = page.text_content("#fight-name")
        if not creature or creature == "—":
            fails.append(f"{name}: a fight opened against nothing")
        check_the_portrait_shows(page, name, fails)
        check_their_gear_is_on_the_screen(page, name, fails)
        # In a real fight, so the board stage's advance button is Fight rather
        # than the town's Done — the three the ask actually names.
        check_the_advance_button_does_not_move(page, name, fails)
        # A save taken here has to reopen the same fight.
        with page.expect_download(timeout=20000) as dl:
            page.click("#fight-save")
        mid = dl.value.path()
        if '"encounter"' not in Path(mid).read_text():
            fails.append(f"{name}: a save taken mid-fight does not carry the encounter")

        was = page.evaluate("() => window.__character()")
        page.click("#go")
        page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
        check_the_replay_shows_both_sides(page, name, fails)
        check_both_boards_are_in_the_replay(page, name, fails)
        check_a_starting_balance_is_on_the_bar(page, name, fails)
        page.click("#skip")
        page.wait_for_selector("#stage-result", state="visible", timeout=15000)
        receipt = page.locator("#result-receipt p").all_text_contents()
        check_the_result_is_a_pocket_not_a_level(page, name, fails, receipt, was)
        check_fatigue_wears_and_mends(page, name, fails)
        if not receipt:
            fails.append(f"{name}: the fight settled with an empty receipt")
        page.click("#done")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)

        # And the mid-fight save reopens into the fight, not onto the map.
        page.set_input_files("#file", str(mid))
        page.wait_for_selector("#fight", state="visible", timeout=10000)
        if page.text_content("#fight-name") != creature:
            fails.append(f"{name}: the reopened fight is against "
                         f"{page.text_content('#fight-name')}, not {creature}")
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)

    gold_before = page.text_content("#gold")
    pos_before = page.text_content("#coords")
    walked_before = page.text_content("#walked")
    level_before = page.text_content("#level")
    points_before = page.text_content("#points")

    # --- download ------------------------------------------------------------
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    saved = dl.value
    path = saved.path()
    text = Path(path).read_text()
    if '"gm2d-save"' not in text:
        fails.append(f"{name}: the downloaded file is not a save")
    if '"name_seed"' not in text or '"locks"' not in text:
        fails.append(f"{name}: the save is missing name_seed or locks")

    # --- reload, having thrown the convenience copy away ---------------------
    # Without clearing it, the page would restore from localStorage and the walk
    # would prove that localStorage works rather than that the file does.
    page.evaluate("localStorage.clear()")
    page.reload(wait_until="networkidle")
    page.wait_for_function("document.getElementById('coords').textContent !== '—'", timeout=20000)
    dismiss_card(page)
    if page.text_content("#coords") == pos_before:
        fails.append(f"{name}: the position survived a cleared reload, so the upload proves nothing")

    # --- upload --------------------------------------------------------------
    page.set_input_files("#file", str(path))
    page.wait_for_function(
        f"document.getElementById('coords').textContent === {pos_before!r}", timeout=20000)
    if page.text_content("#walked") != walked_before:
        fails.append(f"{name}: the walk counter came back as "
                     f"{page.text_content('#walked')}, not {walked_before}")
    if page.text_content("#gold") != gold_before:
        fails.append(f"{name}: the purse came back as {page.text_content('#gold')}")
    if page.text_content("#level") != level_before:
        fails.append(f"{name}: the level came back as {page.text_content('#level')}, "
                     f"not {level_before}")
    if page.text_content("#points") != points_before:
        fails.append(f"{name}: the unspent points came back as "
                     f"{page.text_content('#points')}, not {points_before}")

    # --- a bad file is refused with a sentence -------------------------------
    junk = ROOT / "dist" / "not-a-save.json"
    junk.write_text('{"format":"gm2d-theme","version":1}')
    page.set_input_files("#file", str(junk))
    page.wait_for_selector("#tape li.bad", timeout=10000)
    msg = last_said(page)
    if "gm2d-save" not in (msg or ""):
        fails.append(f"{name}: a wrong file was refused with {msg!r}, which names nothing")
    junk.unlink()
    if page.text_content("#coords") != pos_before:
        fails.append(f"{name}: a refused file still moved the player")

    # --- and the save a player sent in, which used to stop the module --------
    # Last in the upload block, because it replaces the game with somebody
    # else's: everything above wants the walk's own state.
    check_the_frozen_save_is_playable(page, name, fails)
    # Put the walk's own game back, or every check after this reads a stranger's.
    page.set_input_files("#file", str(path))
    page.wait_for_function(
        f"document.getElementById('coords').textContent === {pos_before!r}", timeout=20000)

    # --- levelling and the tree ----------------------------------------------
    # Grind the pit until a level lands. The receipt has to name which frame
    # grew, because "you levelled" without saying what it bought is the thing
    # the plan asks the level-up to say.
    # **Six hundred, because a level now costs a walk home.**
    # Experience is carried and only a town spends it, so this loop has to
    # fight *and* come back — where it used to level on the spot. Three hundred
    # was enough on a good run and not on a bad one, which is the worst budget
    # a loop can have.
    for i in range(600):
        if int(page.text_content("#level")) >= 2:
            break
        # A level is banked, not won, so the receipt that names the frame is
        # the town's now rather than the fight's.
        if page.is_visible("#town"):
            bank_here(page)
            page.click("#leave")
            page.wait_for_selector("#town", state="hidden", timeout=5000)
            continue
        if page.is_visible("#fight"):
            page.click("#preset")
            page.click("#go")
            page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
            page.click("#skip")
            page.wait_for_selector("#stage-result", state="visible", timeout=20000)
            page.click("#done")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
            continue
        if dismiss_card(page):
            continue
        # Fight until there is something worth banking, then go and bank it.
        carrying = page.evaluate("() => window.__character().carried")
        home = head_for_town(page) if carrying >= 15 else None
        page.keyboard.press(home or PATROL[i % len(PATROL)])

    level = int(page.text_content("#level"))
    if level < 2:
        fails.append(f"{name}: six hundred steps of grinding the pit and still level {level} "
                     f"({len(BANKINGS)} bankings)")
    else:
        # Read off every banking the walk has done, wherever it happened.
        if not any("row on the" in r for r in BANKINGS):
            fails.append(f"{name}: no banking ever said which frame grew: {BANKINGS[-3:]}")
        if not any("Level " in r for r in BANKINGS):
            fails.append(f"{name}: no banking ever announced a level: {BANKINGS[-3:]}")
        if int(page.text_content("#points")) < 1:
            fails.append(f"{name}: level {level} and no point to spend")

        page.click("#skills")
        page.wait_for_selector("#tree", state="visible", timeout=8000)
        nodes = page.locator("#nodes .wares").count()
        if not (10 <= nodes <= 15):
            fails.append(f"{name}: the tree shows {nodes} nodes and the plan asks for 10 to 15")
        buyable = page.locator("#nodes .wares:not(:disabled)")
        if buyable.count() == 0:
            fails.append(f"{name}: a point to spend and nothing to spend it on")
        else:
            before = int(page.text_content("#tree-points"))
            buyable.first.click()
            after = int(page.text_content("#tree-points"))
            if after >= before:
                fails.append(f"{name}: buying a node cost no points ({before} -> {after})")
            # And it cannot be bought twice: the same node is now disabled.
            if page.locator("#nodes .wares.pinned:not(:disabled)").count():
                fails.append(f"{name}: a taken node is still clickable")
        page.click("#tree-done")
        page.wait_for_selector("#tree", state="hidden", timeout=8000)

    # --- the fork -------------------------------------------------------------
    # Grind on to level five and take the class. Slower than the rest of the
    # walk by a wide margin, and worth it: this is the MVP's last line, and the
    # only place it can be checked is a browser that has actually got there.
    # There is no fast-forward, deliberately — a debug hook that skipped four
    # levels would be a cheat in shipped code, and the first thing to rot.
    #
    # **It has to go home, and it did not.** `PATROL` is six east and six west,
    # and a blocked press does not move — so from the town's own tile the
    # westward half is spent against the map's edge and the walk drifts east a
    # tile at a time until it is fifteen from home and can never find it again.
    # A fight used to level you on the spot, so that cost nothing; a town is the
    # only place experience becomes a level now, and the instrumented run that
    # found this ended **carrying 1,115 experience at level 2** after 255
    # fights. `head_for_town` is the fix, and it is the same fix the levelling
    # loop above already had — the note on it says so in as many words. This
    # failed a deploy before it was found.
    for i in range(900):
        if page.is_visible("#fork"):
            break
        if page.is_visible("#fight"):
            page.click("#preset")
            page.click("#go")
            page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
            page.click("#skip")
            page.wait_for_selector("#stage-result", state="visible", timeout=20000)
            page.click("#done")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
            continue
        if leave_town(page) or dismiss_card(page):
            continue
        carrying = page.evaluate("() => window.__character().carried")
        home = head_for_town(page) if carrying >= 15 else None
        page.keyboard.press(home or PATROL[i % len(PATROL)])

    if not page.is_visible("#fork"):
        fails.append(f"{name}: never reached the class fork "
                     f"(level {page.text_content('#level')})")
    else:
        # **Five or more**, not exactly five. Banking spends a whole pocket at
        # once and can cross several levels in one go, so a character who
        # walked home carrying a lot arrives at seven and is asked there. What
        # has to hold is that it is asked at the first opportunity, which is
        # the banking that crossed five.
        at = int(page.text_content("#level"))
        if at < 5:
            fails.append(f"{name}: the fork opened at level {at}, before it is owed")
        if not any("Level 5" in r for r in BANKINGS):
            fails.append(f"{name}: no banking announced level five: {BANKINGS[-2:]}")
        # **Five now.** Top of the Bill is the fifth, and its promise is the
        # first one in the game that is about the purse rather than the fight —
        # which is why M10.2's first deliverable was wiring it up rather than
        # adding it. The count is core's; asserting a literal here would be the
        # second copy of `class::OFFERED` all over again.
        offered = page.locator("#fork-choices .wares").count()
        want = page.evaluate("() => (window.__classOffer()?.classes ?? []).length")
        if offered != want:
            fails.append(f"{name}: core offers {want} classes and the screen drew {offered}")
        if offered < 5:
            fails.append(f"{name}: the fork offers {offered} classes, and there are five")
        promises = page.locator("#fork-choices .wares .promise").all_text_contents()
        if any(not p.strip() for p in promises):
            fails.append(f"{name}: a class promises nothing mechanical: {promises}")
        # A number, in digits or in words. Spelling small ones out is the
        # house style — TONE.md rule 12 had to learn the same thing after it
        # failed "Forty Fnorp" for naming no number.
        SPELT = ("once", "twice", "one", "two", "three", "four", "five", "six",
                 "seven", "eight", "nine", "ten", "half", "double")
        def counts(p):
            low = p.lower()
            return any(c.isdigit() for c in p) or any(w in low.split() for w in SPELT)
        if any(not counts(p) for p in promises):
            fails.append(f"{name}: a class promise names no number: {promises}")
        # Every class has a figure, and the figure loaded. Waited for rather
        # than sampled: `naturalWidth` is 0 until the decode finishes, so
        # measuring the instant the screen opens is a race that fails in one
        # engine and passes in the other two.
        wait_for_images(page, "#fork-choices")
        art = page.evaluate("""() => [...document.querySelectorAll('#fork-choices .wares')].map(b => {
          const i = b.querySelector('img');
          return { src: i?.getAttribute('src') ?? null, w: i?.naturalWidth ?? 0 };
        })""")
        for i, a in enumerate(art):
            if not a["src"]:
                fails.append(f"{name}: class {i} is offered with no figure")
            elif a["w"] == 0:
                fails.append(f"{name}: class {i} points at {a['src']}, which did not load")

        # The fork does not take Escape: it is the one decision that does not
        # come off, and a screen you can dismiss is a decision you can decline.
        page.keyboard.press("Escape")
        if not page.is_visible("#fork"):
            fails.append(f"{name}: Escape dismissed the class fork")

        before = page.get_attribute("#player-art", "src")
        page.locator("#fork-choices .wares").first.click()
        page.wait_for_selector("#fork", state="hidden", timeout=8000)
        chosen = page.text_content("#class")
        if not chosen or chosen == "—":
            fails.append(f"{name}: a class was chosen and the sheet still says {chosen!r}")

        # And your own figure becomes that class's. The Sprocketman is who you
        # are before anybody decided; the fork is where that stops being true.
        wait_for_images(page, "#player-art")
        art = page.evaluate("""() => {
          const a = document.getElementById('player-art');
          return { src: a.getAttribute('src'), w: a.naturalWidth, hidden: a.hidden };
        }""")
        if art["src"] == before:
            fails.append(f"{name}: took a class and the panel still draws {before}")
        if art["hidden"] or not art["w"]:
            fails.append(f"{name}: the class figure did not load: {art}")

        # The class brought its own tree, and it has a tab of its own.
        #
        # Counted as tabs rather than as one long rack of nodes: only the open
        # tree is drawn now, so "more nodes than the base tree" stopped being
        # the question. What matters is that the class tree is reachable and
        # full of nodes.
        page.wait_for_selector("#tree", state="visible", timeout=8000)
        tabs = page.locator("#tree-tabs button")
        if tabs.count() < 2:
            fails.append(f"{name}: after the fork the tree screen shows "
                         f"{tabs.count()} tab(s), so the class tree is unreachable")
        else:
            base = page.locator("#nodes .node").count()
            tabs.nth(1).click()
            page.wait_for_timeout(120)
            klass = page.locator("#nodes .node").count()
            if klass == 0:
                fails.append(f"{name}: the class tab opened an empty tree")
            if page.locator("#tree-tabs button.on").count() != 1:
                fails.append(f"{name}: switching tabs left {page.locator('#tree-tabs button.on').count()} selected")
            if base == 0:
                fails.append(f"{name}: the base tree drew nothing")
        page.click("#tree-done")
        page.wait_for_selector("#tree", state="hidden", timeout=8000)

        # And it is permanent: reopening offers nothing.
        if JSON_NULL != page.evaluate("() => window.__classOffer()"):
            fails.append(f"{name}: the fork is still on offer after being taken")

    # --- the way into the cave -----------------------------------------------
    # **Last, because these plant saves.** They replace the character to stand
    # them somewhere the walk would take twenty minutes to reach, so anything
    # after them would be checking a game this walk did not play.
    check_the_fork_is_on_top(page, name, fails)
    check_the_door_opens_on_the_treyway(page, name, fails)
    check_the_road_west_reaches_a_town(page, name, fails)
    check_the_tower_drops(page, name, fails)
    check_the_lake_drains_and_the_demo_ends_under_it(page, name, fails)
    check_the_rack(page, name, fails)
    check_the_spin_animates(page, name, fails)
    check_a_town_takes_the_tiredness_off(page, name, fails)
    check_the_replay_reports_a_curse(page, name, fails)
    check_the_log_points_somewhere(page, name, fails)
    check_an_errand_can_be_handed_in_where_it_was_taken(page, name, fails)
    check_the_cave_is_shut_until_it_is_not(page, name, fails)

    # --- what a creature leaves behind ---------------------------------------
    check_a_set_reads(page, name, fails)
    check_an_instrument_takes_the_grid(page, name, fails)
    check_the_long_way_back(page, name, fails)
    check_the_reach_reads_through_what_you_carry(page, name, fails)
    check_the_toad_walks_on_water(page, name, fails)

    # --- the north ------------------------------------------------------------
    check_the_north_is_shut(page, name, fails)
    check_an_ench_you_cannot_use_is_still_shown(page, name, fails)

    # --- where an ench comes from --------------------------------------------
    check_the_van_appears_at_a_level(page, name, fails)
    check_a_broken_item_reads(page, name, fails)

    # --- scouting ------------------------------------------------------------
    check_scouting_is_earned(page, name, fails)

    # --- the log ---------------------------------------------------------------
    check_the_game_talks_in_one_place(page, name, fails)

    ctx.close()
    if problems:
        fails.append(f"{name}: the page reported errors:\n  " + "\n  ".join(problems))
    if offsite:
        fails.append(f"{name}: the page left the origin:\n  " + "\n  ".join(sorted(set(offsite))))
    return fails


def main():
    if not LIVE and not (WEB / "index.html").exists():
        sys.exit(f"{WEB} is not built. Run: make web")
    wanted = sys.argv[1:] or ["chromium"]
    # Nothing to serve when the page is already up somewhere.
    httpd = None if LIVE else serve()
    if LIVE:
        print(f"walking {ORIGIN} rather than a build of its own")
    fails = []
    try:
        with sync_playwright() as p:
            for name in wanted:
                engine = getattr(p, name, None)
                if engine is None:
                    fails.append(f"{name}: no such browser")
                    continue
                try:
                    b = engine.launch()
                except Exception as e:
                    # A browser that is not installed is reported, not silently
                    # skipped: the gate names three and a quiet skip would let
                    # two of them rot.
                    fails.append(f"{name}: could not launch ({e})")
                    continue
                fails += check_a_stale_autosave(b, name)
                mine = []
                try:
                    walk_the_gate(b, name, mine)
                except Exception as e:
                    # Keep whatever was found before the crash. The crash is a
                    # failure too, and usually a consequence of the first one.
                    # The traceback too. A crash used to report one line —
                    # "'NoneType' object is not subscriptable" — with no way to
                    # tell which of forty checks it came out of.
                    mine.append(f"{name}: the walk stopped: {str(e).splitlines()[0]}\n"
                                + traceback.format_exc())
                fails += mine
                b.close()
                if not any(f.startswith(name + ":") for f in fails):
                    print(f"ok: {name} walked the gate")
    finally:
        if httpd:
            httpd.shutdown()

    if fails:
        print("\n".join(f"FAIL: {f}" for f in fails))
        sys.exit(1)
    print("ok: town, shop, pack, walk, fight, replay, receipt — the whole loop")
    print("ok: a level-up named the frame it grew, and the point could be spent")
    print("ok: level five forked the character, and the fork does not come off")
    print("ok: the fit preview is core's answer, not the page's")
    print("ok: a component turned in hand turns on the board too")
    print("ok: cork is per activation, not a standing stat")
    print("ok: pointing at an item on the board reads it in the panel")
    print("ok: every skill states its effect in numbers, unthemed, and explains it on hover")
    print("ok: the tree is drawn as a tree — free skills on top, prerequisites above what needs them")
    print("ok: the creature has a portrait, and the portrait loaded")
    print("ok: what it is wearing is on the screen — its board, its items, its own body")
    print("ok: both boards tick in the replay, and pointing at a row reads that item")
    print("ok: a town's shelf is that town's shelf, and what you buy is gone from it")
    print("ok: the starting town asks for five toads, in a number rather than a mood")
    print("ok: every class is offered with its own figure")
    print("ok: a component is a shape, on the shelf and in the bag, and it explains itself")
    print("ok: both boards are drawn in the replay, and what fires jolts")
    print("ok: the sheet says what you are, and a fight opens holding what the tree granted")
    print("ok: experience is carried out of a fight and only a town spends it")
    print("ok: every fight wears you down, and the panel and the sheet both say so")
    print("ok: the cave is shut until you have the key, and short once you are in it")
    print("ok: a door you have already read reopens, and takes its errand back")
    print("ok: the log says where every errand wants you, and a pin outlives the screen")
    print("ok: a curse lands, says what it is doing, and comes off its own clock")
    print("ok: the advance button is the same box on all three fight stages")
    print("ok: a town takes the tiredness off, and the panel says so")
    print("ok: the map's odds are a skill somebody took, not a button everybody had")
    print("ok: a licensee buys an ench, bolts it on, switches it off and takes it back")
    print("ok: a spinning item turns, and turns to somewhere core said it could")
    print("ok: the wall grows a door, the key opens it, and behind it is a map")
    print("ok: two maps out there is a town, and one field tile in ten answers")
    print("ok: the Drambus Stack opens onto a different floor every time, then a stump")
    print("ok: the Stack comes down, the lake empties, and there is a door at the bottom")
    print("ok: a compass in the weapon grid grants a rule, and the sheet says so")
    print("ok: the reach refuses without an instrument, and reads differently through each")
    print("ok: a whole set takes you home, drinks a tin doing it, and refuses without one")
    print("ok: a whole set names its own item and says what it does; two thirds of one does not")
    print("ok: the lake is ground to a toad, edge to middle, and the middle has a way down in it")
    print("ok: the north is shut to a level-one character, and says what the road wants")
    print("ok: an ench you were paid and cannot use yet is on the rack, and says why")
    print("ok: no town sells an ench, and the van on the Verge road is not there until level ten")
    print("ok: an item with the swing on it fires once, breaks, and the replay says which")
    print("ok: the class fork opens on top of the town it is offered in")
    print("ok: your own figure becomes your class's when you take one")
    print("ok: a mid-fight save reopens the same fight")
    print("ok: walk, download, reload, upload — position and stream both came back")
    print("ok: a wrong file was refused with a sentence and changed nothing")
    print("ok: a save from an older build does not wedge the player in the scenery")
    print("ok: the game talks in one place, and the history holds the sitting")
    print("ok: no console errors, no off-origin requests")


main()
