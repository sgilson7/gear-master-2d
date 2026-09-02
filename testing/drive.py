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
import http.server
import socketserver
import sys
import threading
import json
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "dist" / "web"
PORT = 8127
ORIGIN = f"http://127.0.0.1:{PORT}"
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


def plant(page, base_path, edit, stem="probe"):
    """Load a save built by editing a downloaded one. Returns nothing.

    The pattern the gate check invented and this file now shares: walking to a
    particular state costs twenty minutes of fighting, and what wants proving
    is what happens *at* that state rather than the road to it.
    """
    save = json.loads(Path(base_path).read_text())
    edit(save.get("state", save))
    out = Path(base_path).parent / f"{stem}.json"
    out.write_text(json.dumps(save))
    page.set_input_files("#file", str(out))
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
    said = page.text_content("#says") or ""
    if not said.strip():
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


def check_the_door_ends_the_demo(page, name, fails):
    """A wall with no door in it grows one, and opening it ends the demo.

    Three firsts in one tile: a place that is not there until it is, a lock
    answered against the bag rather than against the map, and the one screen in
    the game that is not a loop. Planted at each stage rather than walked,
    because the walk to it is the whole game.
    """
    with page.expect_download(timeout=20000) as dl:
        page.click("#download")
    base = dl.value.path()

    door = page.evaluate("""() => JSON.parse(window.__save()).state.world""")
    del door  # only the save's shape is wanted; the door is not on the map yet

    if page.evaluate("""() => (window.__world().places ?? [])
            .some(p => p.kind === 'door')"""):
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
    spot = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'door')""")
    if not spot:
        fails.append(f"{name}: the Cave's boss is down and the wall still has no door")
        return

    # Stand beside it and walk in without the key.
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if page.is_visible("#ending"):
        fails.append(f"{name}: the door opened with no key in the bag")
        page.click("#ending-close")
    said = (page.text_content("#says") or "").strip()
    if not said:
        fails.append(f"{name}: a locked door said nothing")

    # --- and with the key ----------------------------------------------------
    plant(page, base, lambda b: cleared(b, key=True), stem="door-open")
    page.evaluate("(at) => window.__standAt(at)", [spot["at"][0] + 1, spot["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(350)
    if page.is_hidden("#ending"):
        fails.append(f"{name}: carried the key onto the door and nothing happened")
        return
    prose = page.evaluate("""() => [...document.querySelectorAll('#ending-prose p')]
        .map(p => p.textContent).join(' ')""")
    if "demo" not in prose.lower():
        fails.append(f"{name}: the ending does not say the demo is over: {prose[:80]!r}")
    # It is not the fork: you can back out of it, because the world is still
    # there and there is an errand about the door to hand in.
    page.click("#ending-close")
    page.wait_for_selector("#ending", state="hidden", timeout=5000)
    if page.text_content("#coords") != f"{spot['at'][0]}, {spot['at'][1]}":
        fails.append(f"{name}: the ending left the player at "
                     f"{page.text_content('#coords')}, not on the door")

    # And the errand about it is on offer now, and only now.
    log = page.evaluate("() => window.__log()")
    del log
    marks = page.evaluate("() => window.__errandMarks()")
    if not any(m["id"] == "the-end-of-all-gears" for m in marks):
        fails.append(f"{name}: the boss is down and the clerk has nothing new to say: {marks}")


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
    def licensed(body):
        body["character"]["class"] = "Recycler"
        body["character"]["gold"] = 400

    plant(page, base, licensed, stem="rack-probe")
    rack = page.evaluate("() => JSON.parse(window.__rack())")
    if not rack["licensed"]:
        fails.append(f"{name}: took the Kaklon Patent and the rack is shut")
        return

    # Walk into the town and buy one off the bench.
    town = page.evaluate("""() => (window.__world().places ?? []).find(p => p.kind === 'town')""")
    page.evaluate("(at) => window.__standAt(at)", [town["at"][0] + 1, town["at"][1]])
    page.keyboard.press("ArrowLeft")
    page.wait_for_timeout(300)
    if not page.is_visible("#town"):
        fails.append(f"{name}: could not get into a town to reach the bench")
        return
    if page.is_hidden("#bench-wrap"):
        fails.append(f"{name}: a licensee walked into a town and there is no bench")
        return
    bench = page.locator("#bench .wares:not(:disabled)")
    if bench.count() == 0:
        fails.append(f"{name}: 400 Fnorp and nothing on the bench is affordable")
        return
    purse = int(page.text_content("#town-gold"))
    bench.first.click()
    page.wait_for_timeout(150)
    if int(page.text_content("#town-gold")) >= purse:
        fails.append(f"{name}: an ench cost nothing")
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

    # Two frames, a second apart, and both have to be orientations core named.
    seen = []
    for _ in range(2):
        seen.append(page.evaluate("""(key) => {
          window.__board.draw();
          const sp = window.__board.spun.find(s => s.key === key);
          return sp ? sp.cells.map(c => c.join(',')).sort().join(' ') : null;
        }""", got["key"]))
        # **Longer than one turn**, or the two samples can land inside the same
        # second and the check reports a still picture as a broken feature.
        # Five hundred and sixty milliseconds did exactly that, about twice in
        # five runs.
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
    if seen[0] == seen[1]:
        fails.append(f"{name}: two frames half a second apart drew the same footprint "
                     f"({seen[0]!r}), so nothing is turning")

    # And the card says what it is worth, in a number.
    said = page.evaluate("""(key) => {
      const el = [...document.querySelectorAll('#panel-yours .made-item')]
        .find(e => e.dataset.key === key);
      return el ? el.textContent : null;
    }""", got["key"])
    if not said or "turns every second" not in said:
        fails.append(f"{name}: the card does not say the item turns: {said!r}")


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
    said = (page.text_content("#says") or "").strip()
    if not said:
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
      const walk = w.rows.flat().filter(t => t !== 'rock' && t !== 'water').length;
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

    terrain = page.text_content("#terrain")
    if terrain in ("rock", "water"):
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
        purse = int(page.text_content("#town-gold"))
        wares = page.locator("#shelf .wares:not(:disabled)")
        if wares.count() == 0:
            fails.append(f"{name}: nothing on the shelf is affordable with {purse} Fnorp")
        else:
            wares.first.click()
            after = int(page.text_content("#town-gold"))
            if after >= purse:
                fails.append(f"{name}: buying cost nothing ({purse} -> {after})")
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
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)

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
    page.wait_for_selector("#says.bad", timeout=10000)
    msg = page.text_content("#says")
    if "gm2d-save" not in (msg or ""):
        fails.append(f"{name}: a wrong file was refused with {msg!r}, which names nothing")
    junk.unlink()
    if page.text_content("#coords") != pos_before:
        fails.append(f"{name}: a refused file still moved the player")

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
        page.keyboard.press(PATROL[i % len(PATROL)])

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
        # **Four now.** The Kaklon Patent arrived with the ench system, and
        # the licence is the class rather than a node inside it.
        offered = page.locator("#fork-choices .wares").count()
        if offered != 4:
            fails.append(f"{name}: the fork offers {offered} classes, and there are four")
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
    check_the_door_ends_the_demo(page, name, fails)
    check_the_rack(page, name, fails)
    check_the_spin_animates(page, name, fails)
    check_a_town_takes_the_tiredness_off(page, name, fails)
    check_the_replay_reports_a_curse(page, name, fails)
    check_the_log_points_somewhere(page, name, fails)
    check_an_errand_can_be_handed_in_where_it_was_taken(page, name, fails)
    check_the_cave_is_shut_until_it_is_not(page, name, fails)

    # --- scouting ------------------------------------------------------------
    check_scouting_is_earned(page, name, fails)

    ctx.close()
    if problems:
        fails.append(f"{name}: the page reported errors:\n  " + "\n  ".join(problems))
    if offsite:
        fails.append(f"{name}: the page left the origin:\n  " + "\n  ".join(sorted(set(offsite))))
    return fails


def main():
    if not (WEB / "index.html").exists():
        sys.exit("dist/web is not built. Run: make web")
    wanted = sys.argv[1:] or ["chromium"]
    httpd = serve()
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
                    mine.append(f"{name}: the walk stopped: {str(e).splitlines()[0]}")
                fails += mine
                b.close()
                if not any(f.startswith(name + ":") for f in fails):
                    print(f"ok: {name} walked the gate")
    finally:
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
    print("ok: the wall grows a door, the key opens it, and the demo ends there")
    print("ok: the class fork opens on top of the town it is offered in")
    print("ok: your own figure becomes your class's when you take one")
    print("ok: a mid-fight save reopens the same fight")
    print("ok: walk, download, reload, upload — position and stream both came back")
    print("ok: a wrong file was refused with a sentence and changed nothing")
    print("ok: a save from an older build does not wedge the player in the scenery")
    print("ok: no console errors, no off-origin requests")


main()
