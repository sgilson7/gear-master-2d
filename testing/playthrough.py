"""Play the demo from a new game through the door and back, and write it down.

**Not the gate.** `drive.py` walks a route chosen to exercise checks: it
teleports, plants saves, and asserts. This does none of that. It starts a new
game, buys with the money it has, packs with the button a player is given,
walks where a player would walk, reads every screen it is shown, and stops when
it has been through the door in the western wall, read the two roads on the
other side and come back — or when it cannot get any further.

The output is a transcript, and the transcript is the deliverable. What it
catches is the thing a suite cannot: a game that is green and unplayable.

    python testing/playthrough.py [chromium] > transcript.txt
"""
import functools
import os
import http.server
import json
import socketserver
import sys
import threading
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "dist" / "web"
PORT = 8131
ORIGIN = f"http://127.0.0.1:{PORT}"

# How long the run is allowed to be. A demo nobody can finish in this many
# steps is a demo nobody finishes.
#
# **Twenty thousand since M11.3.** Nine was written for a game that ended at a
# door thirty tiles from the pit. The run now walks four maps and climbs a
# five-floor tower, and the last one to be given nine thousand cleared one floor
# and ran out of presses on the step that put it back outside.
STEPS = 20000
# Shorter, for finding out why a run is doing something. A full one is nine
# minutes and most of what goes wrong in a walker goes wrong in the first two
# thousand presses.
STEPS = int(os.environ.get("GM2D_STEPS", STEPS))

# What the walk has to have done before it comes home for good: met the woman
# at the turn, read the post at the reach, and been inside Kettleworks. The
# first two are the Treyway's; the third is the map behind the road west, and
# it is a town rather than a card because reaching a *town* on a third map is
# the thing M11.2 ships.
TREYWAY_PROMISES = {"the-kettleworks-road", "the-wextreen-reach"}


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass



def last_said(page):
    """The last thing the game said, out of M11.0's log strip.

    The transcript used to read `#says`, the slot below the save panel that
    nothing owned. There is one place the game talks now, and this is it."""
    lines = page.eval_on_selector_all("#tape li", "els => els.map(e => e.textContent)")
    return (lines[-1].strip() if lines else "")


def serve():
    handler = functools.partial(Quiet, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


LOG = []


def say(*a):
    line = " ".join(str(x) for x in a)
    LOG.append(line)
    print(line, flush=True)


def head(t):
    say("")
    say(f"--- {t} " + "-" * max(0, 68 - len(t)))


def panel(page):
    c = page.evaluate("() => window.__character()")
    p = json.loads(page.evaluate("() => window.__position()"))
    return (f"level {c['level']} · {p['x']},{p['y']} in {p['region']} · "
            f"{page.text_content('#gold')} Fnorp · carrying {c['carried']} · "
            f"worn {c['fatigue']}% · {c['class'] or 'no class'}")


def at(page):
    return page.evaluate("() => document.getElementById('coords').textContent")


# **What the ground takes a foot on is core's answer, not a list here.** This
# was `{"rock", "water"}` — a second copy of the terrain table, written in
# Python — and the Treyway added `range` and `sea`, so the walker pathed
# straight through a mountain range and pressed into it forty times. Same
# failure as the odds overlay's, found in the same run. `world_json` reports
# passability per tile now and this reads it.
KEYS = {(1, 0): "ArrowRight", (-1, 0): "ArrowLeft", (0, 1): "ArrowDown", (0, -1): "ArrowUp"}


# What the walker would rather walk on. **A preference, not a copy of the
# terrain table**: it does not have to be right or complete, only useful, and
# anything not named here is simply dearer. A player takes the road, and the
# road is where the encounter rate is lowest — the first version of this took
# the shortest line instead, walked twelve tiles of scrub through the Stack's
# Shadow every trip, and lost two thousand two hundred fights doing it.
ROADS = {"road"}
ROAD_COST = 1
ROUGH_COST = 6
# What it costs to walk over somebody. A town, a card, a gate — stepping on one
# opens a screen, and a route that goes through the shop to save two tiles
# spends four thousand presses opening and shutting it. **`town` used to be in
# `ROADS`**, which made the cheapest path to anywhere run through the counter.
PLACE_COST = 40


def toward(world, here, target, barred=()):
    """One step towards a tile, along the road where there is one.

    **A real path, not a bearing.** Walking one axis at a time walks into the
    Burnwarp and stays there: a blocked step draws nothing and costs nothing,
    so a walker that keeps pressing into a cliff presses into it for ever. The
    first version of this managed a hundred and forty-one moves out of four
    thousand presses.

    **And a path round what refused you.** `barred` is the tiles a step has
    actually been refused on — a crossing that wants a level you have not got
    reads as ordinary ground and is not, so a route that goes through one is a
    route that presses into it for ever. A player who is turned back once goes
    round; so does this.

    **And a cheap path, not a short one.** Terrain decides how likely a step is
    to start a fight, and a region multiplies it: crossing the Stack's Shadow
    off the road is twenty-eight percent a tile and on it is six. A shortest
    path does not know that and a player does.
    """
    if tuple(here) == tuple(target):
        return None
    walk = world["walk"]
    rows = world["rows"]
    w, h = world["width"], world["height"]
    ok = lambda x, y: (0 <= x < w and 0 <= y < h and walk[y][x]
                       and ((x, y) not in barred or (x, y) == tuple(target)))
    onto = {tuple(p["at"]) for p in world.get("places", [])} - {tuple(target)}

    def cost(x, y):
        c = ROAD_COST if rows[y][x] in ROADS else ROUGH_COST
        return c + PLACE_COST if (x, y) in onto else c
    import heapq
    # Dijkstra from the target outwards, so the cheapest neighbour of `here`
    # that is settled is a step along the cheapest path.
    best = {tuple(target): 0}
    heap = [(0, tuple(target))]
    while heap:
        d, (cx, cy) = heapq.heappop(heap)
        if d > best.get((cx, cy), 1 << 30):
            continue
        for dx, dy in KEYS:
            nx, ny = cx + dx, cy + dy
            if not ok(nx, ny):
                continue
            nd = d + cost(nx, ny)
            if (nx, ny) == tuple(here):
                # Do not settle the start; its own terrain is behind us. Keep
                # going so a dearer first step on a cheaper road still wins.
                best[(nx, ny)] = min(best.get((nx, ny), 1 << 30), nd)
                continue
            if nd < best.get((nx, ny), 1 << 30):
                best[(nx, ny)] = nd
                heapq.heappush(heap, (nd, (nx, ny)))
    # The neighbour of `here` with the cheapest way to the target.
    pick, pick_d = None, 1 << 30
    for dx, dy in KEYS:
        nx, ny = here[0] + dx, here[1] + dy
        if not ok(nx, ny):
            continue
        d = best.get((nx, ny))
        if d is not None and d < pick_d:
            pick, pick_d = KEYS[(dx, dy)], d
    return pick


def fight(page, note=None):
    """Pack with the button a player is given, fight, and read the receipt."""
    page.click("#preset")
    made = page.text_content("#fight-yours")
    creature = page.text_content("#fight-name")
    rating = page.text_content("#fight-rating")
    page.click("#go")
    page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
    page.click("#skip")
    page.wait_for_selector("#stage-result", state="visible", timeout=20000)
    title = page.text_content("#result-title")
    lines = page.locator("#result-receipt p").all_text_contents()
    page.click("#done")
    page.wait_for_selector("#fight", state="hidden", timeout=8000)
    say(f"  fight: {creature} (rates {rating}) with {made} items -> {title}")
    for l in lines:
        say(f"         {l}")
    return "It stops moving" in (title or "")


def in_town(page, buy=True):
    """Bank, buy what is affordable, take every errand, drink nothing."""
    said = []
    if not page.is_disabled("#bank"):
        page.click("#bank")
        page.wait_for_timeout(120)
        said.append(page.text_content("#town-says") or "")
        # **Banking is where a level lands, and level five opens the fork** —
        # over the town, because it is the one screen that does not come off.
        # Everything else in here waits.
        if page.is_visible("#fork"):
            for x in said:
                if x.strip():
                    say(f"  town: {x.strip()}")
            return
    if buy:
        for _ in range(30):
            live = page.locator("#shelf .wares:not(:disabled)")
            if live.count() == 0:
                break
            name = live.first.locator("b").text_content()
            live.first.click()
            page.wait_for_timeout(40)
            said.append(f"bought {name}")
        for _ in range(6):
            live = page.locator("#bench .wares:not(:disabled)")
            if live.count() == 0:
                break
            name = live.first.locator("b").text_content()
            live.first.click()
            page.wait_for_timeout(40)
            said.append(f"bought {name} (bench)")
        # One tin, so there is something for the road.
        tins = page.locator("#tins .wares:not(:disabled)")
        if tins.count():
            tins.first.click()
            page.wait_for_timeout(40)
            said.append("bought a tin")
    # By name, not by "the first enabled one": a taken errand stays clickable
    # on purpose — clicking it says how far along you are, which is
    # information rather than an error — so a loop that keeps pressing the
    # first live button presses the same one for ever.
    seen = set()
    for _ in range(10):
        live = None
        for i in range(page.locator("#quests .wares:not(:disabled)").count()):
            b = page.locator("#quests .wares:not(:disabled)").nth(i)
            n = b.locator("b").text_content()
            if n not in seen:
                live, name = b, n
                break
        if live is None:
            break
        seen.add(name)
        live.click()
        page.wait_for_timeout(60)
        said.append(f"errand: {name} — {page.text_content('#town-says')}")
    for s in said:
        if s.strip():
            say(f"  town: {s.strip()}")
    page.click("#leave")
    page.wait_for_selector("#town", state="hidden", timeout=5000)


def card(page):
    """Read an event card, take the first choice, take any errand."""
    title = page.text_content("#card-title")
    prose = page.locator("#card-prose p").all_text_contents()
    say(f"  card: {title}")
    for p in prose[:1]:
        say(f"        {p[:110]}")
    seen = set()
    for _ in range(6):
        live = None
        for i in range(page.locator("#card-errands .wares:not(:disabled)").count()):
            b = page.locator("#card-errands .wares:not(:disabled)").nth(i)
            n = b.locator("b").text_content()
            if n not in seen:
                live, name = b, n
                break
        if live is None:
            break
        seen.add(name)
        live.click()
        page.wait_for_timeout(60)
        say(f"        errand: {name} — {last_said(page)}")
    if page.is_visible("#card-choices button:not(:disabled)"):
        pick = page.locator("#card-choices button:not(:disabled)").first
        say(f"        chose: {pick.locator('b').text_content()}")
        pick.click()
        page.wait_for_timeout(80)
        got = page.locator("#card-receipt p").all_text_contents()
        for g in got:
            say(f"        {g}")
    page.wait_for_selector("#card-close", state="visible", timeout=5000)
    page.click("#card-close")
    page.wait_for_selector("#card", state="hidden", timeout=5000)


def fork(page):
    boxes = page.evaluate("""() => [...document.querySelectorAll('#fork-choices .wares')]
        .map((b, i) => {
          const r = b.getBoundingClientRect();
          const mid = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
          return { i, name: b.querySelector('b').textContent,
                   box: [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)],
                   ok: b.contains(mid),
                   under: mid ? (mid.textContent || '').slice(0, 24) : 'nothing' };
        })""")
    say(f"  fork: {len(boxes)} classes offered")
    for b in boxes:
        say(f"        [{b['i']}] {b['name']} at {b['box']} clickable={b['ok']} ({b['under']})")
    # **Top of the Bill**, which is the class M10 added and therefore the one
    # this block has to be played as. Falls back to the first card, so a run
    # against a build that does not offer it still plays rather than stopping.
    want = next((b["i"] for b in boxes if "Bill" in b["name"]), 0)
    card = page.locator("#fork-choices .wares").nth(want)
    card.scroll_into_view_if_needed()
    # On the name rather than in the middle: the card is four hundred pixels
    # tall and a viewport is seven hundred, so its centre can be below the fold.
    card.click(position={"x": 30, "y": 24})
    page.wait_for_selector("#fork", state="hidden", timeout=8000)
    say(f"  fork: took {page.text_content('#class')}")
    if page.is_visible("#tree"):
        page.click("#tree-done")
        page.wait_for_selector("#tree", state="hidden", timeout=8000)


def spend_points(page):
    # The fork is modal and comes first — the loop will offer it next time
    # round, and the tree opens by itself once it is answered.
    if page.is_visible("#fork"):
        return
    c = page.evaluate("() => window.__character()")
    if c["points"] < 1:
        return
    page.click("#skills")
    page.wait_for_selector("#tree", state="visible", timeout=8000)
    for _ in range(20):
        live = page.locator("#nodes .wares:not(:disabled)")
        if live.count() == 0:
            # Try the other tabs — the strip is hidden while there is only one
            # tree, which is every character before the fork.
            if page.is_hidden("#tree-tabs"):
                break
            tabs = page.locator("#tree-tabs button")
            done = True
            for i in range(tabs.count()):
                tabs.nth(i).click()
                page.wait_for_timeout(60)
                if page.locator("#nodes .wares:not(:disabled)").count():
                    done = False
                    break
            if done:
                break
            continue
        name = live.first.locator("b").text_content()
        live.first.click()
        page.wait_for_timeout(60)
        say(f"  tree: took {name}")
    page.click("#tree-done")
    page.wait_for_selector("#tree", state="hidden", timeout=8000)


def main():
    if not (WEB / "index.html").exists():
        sys.exit("dist/web is not built. Run: make web")
    which = sys.argv[1] if len(sys.argv) > 1 else "chromium"
    httpd = serve()
    problems = []
    try:
        with sync_playwright() as pw:
            b = getattr(pw, which).launch()
            ctx = b.new_context(accept_downloads=True)
            page = ctx.new_page()
            page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}")
                    if m.type == "error" else None)
            page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
            page.goto(ORIGIN + "/", wait_until="networkidle")
            page.wait_for_function(
                "document.getElementById('coords').textContent !== '—'", timeout=20000)
            page.evaluate("() => localStorage.clear()")
            page.click("#reset")
            page.wait_for_timeout(200)

            head("a new game")
            say(" ", panel(page))
            # Where the map is.
            world = page.evaluate("() => window.__world()")
            town = next(p for p in world["places"] if p["kind"] == "town")
            gate = next((p for p in world["places"] if p["kind"] == "gate"), None)
            say(f"  town at {town['at']}, cave gate at {gate['at'] if gate else '—'}")

            head("the road")
            phase = ""
            target = None
            done_marks = set()
            errand_turn = 0
            losses = 0
            wins = 0
            moved = 0
            patrol = 0
            # M11.1's three: whether the Treyway has been seen, which of its
            # promises have been read, and whether the walk has come back
            # through the door — which is where the run now finishes, because
            # the door stopped being an ending and became a border.
            seen_treyway = False
            seen_field = False
            seen_kettleworks = False
            on_floor = None
            seen_under = False
            came_back = False
            floors_down = 0
            answered = set()
            last_level, last_done = 0, -1
            # Tiles that turned a press back. Cleared when the walker changes:
            # a crossing that wanted level nine is a road once you are nine.
            barred = set()
            # The press at which the town stops being barred. See the town
            # branch: a walk that has just banked does not walk straight back
            # in, and one that did spent four fifths of a run doing it.
            town_shut_until = 0
            # **The last dozen fights.** A player who has lost four in a row
            # goes and fights something smaller; the first version of this
            # walker did not, and one run threw itself at the Stack's Shadow
            # eighteen hundred times. Not a game rule — a thing a person does,
            # and the transcript is worth nothing if the walker does not.
            from collections import deque
            recent = deque(maxlen=12)
            # Tiles that were walked to and turned out to hold nothing
            # readable. **Keyed by map**, and it has to be: a bare `(x, y)` is
            # four different tiles now, and the first version of this poisoned
            # the woman at the turn on the Treyway with a tile the walk had
            # stood on in Bambulon — so the run crossed the door back and
            # forth for six thousand presses looking for something it had
            # already crossed off. `answered` is the real record; this is the
            # walker's own give-up list, the same one `done_marks` is.
            read_over = set()
            # Targets that would not let us in, by how many presses were spent
            # finding that out.
            stuck = {}
            for step in range(STEPS):
                if page.is_visible("#ending"):
                    head("the ending")
                    for p in page.locator("#ending-prose p").all_text_contents():
                        say(f"  {p}")
                    page.click("#ending-close")
                    page.wait_for_selector("#ending", state="hidden", timeout=5000)
                    say("")
                    say(f"REACHED THE ENDING at step {step}. {panel(page)}")
                    break
                if page.is_visible("#fork"):
                    fork(page)
                    continue
                if page.is_visible("#fight"):
                    won_it = fight(page)
                    recent.append(won_it)
                    if won_it:
                        wins += 1
                    else:
                        losses += 1
                    continue
                if page.is_visible("#card"):
                    card(page)
                    continue
                if page.is_visible("#vendor"):
                    # **The man in the van.** M10 put him on the Verge road at
                    # level ten and the M10.3 walk never got there on foot, so
                    # nothing in this file knew the screen existed — and it is
                    # a `.screen`, so `walk()` refuses every keypress while it
                    # is up. The symptom was a walk that stopped dead at [4, 6]
                    # and reported the road home as unreachable.
                    head("the van")
                    say(f"  {panel(page)}")
                    for line in page.locator("#vendor-prose p").all_text_contents()[:1]:
                        say(f"  {line[:110]}")
                    n = page.locator("#vendor-stock .wares:not(:disabled)").count()
                    if n:
                        # **Not `b`.** That is the browser, four hundred lines
                        # up, and shadowing it made the run end on
                        # `'Locator' object has no attribute 'close'` after a
                        # perfectly good transcript.
                        line = page.locator("#vendor-stock .wares:not(:disabled)").first
                        what = line.locator("b").text_content()
                        line.click()
                        page.wait_for_timeout(80)
                        say(f"  bought {what} — {last_said(page)}")
                    page.click("#vendor-close")
                    page.wait_for_selector("#vendor", state="hidden", timeout=5000)
                    phase = ""
                    continue
                if page.is_visible("#town"):
                    if page.evaluate("() => window.__world().id") == "kettleworks-field":
                        seen_kettleworks = True
                    say(f"  in town: {panel(page)}")
                    in_town(page)
                    spend_points(page)
                    target = None
                    # **A level lands in a town, and a level is what a crossing
                    # asks for — so everything that was out of reach is worth
                    # trying again from here.** But only when something actually
                    # moved: clearing the give-up list on *every* visit clears
                    # the mark that says "this errand's tile is where you
                    # already are", and a walk that did that walked into
                    # Kettleworks seven hundred and forty times in one run,
                    # clicking an errand that told it each time it had not been
                    # yet.
                    after = page.evaluate("() => window.__character()")
                    log_now = page.evaluate("() => window.__log()")
                    finished = sum(1 for q in log_now["errands"] if q["stage"] == "done")
                    if after["level"] != last_level or finished != last_done:
                        done_marks.clear()
                        stuck.clear()
                        barred.clear()
                    last_level, last_done = after["level"], finished
                    # **You have just been in.** Whatever sent the walk here is
                    # answered now, and something kept sending it back: eight
                    # hundred visits in one run, four Fnorp of tin at a time,
                    # without the panel's coordinates changing once. Bar the
                    # counter for a while and go and do something.
                    here_now = tuple(int(v) for v in at(page).split(","))
                    barred.add(here_now)
                    town_shut_until = step + 60
                    continue

                c = page.evaluate("() => window.__character()")
                world = page.evaluate("() => window.__world()")
                here = tuple(int(v) for v in at(page).split(","))

                # A tin when the road has taken a quarter of you and there is
                # one in the pack. What a player does with the button they are
                # given, on the tile the decision is made on.
                if c["fatigue"] >= 24 and (c["supplies"] or []):
                    page.locator("#kit .tin:not(:disabled)").first.click()
                    page.wait_for_timeout(60)
                    say(f"  drank a tin: {last_said(page)}")
                    continue

                # **By id, not by kind.** There are two gates on West Bambulon
                # since M11.1 and they want opposite things — the Cave's mouth
                # wants Marbulon's key and a level, the door in the wall wants
                # what the Cave's boss drops — so `the first place whose kind is
                # gate` is a coin flip on file order.
                def place_by_id(pid):
                    return next((p for p in world["places"] if p["id"] == pid), None)

                boss = next((p for p in world["places"] if p["kind"] == "boss"), None)
                cave_mouth = place_by_id("the-cave-mouth")
                wall_door = place_by_id("the-door-in-the-wall")
                door_back = place_by_id("the-door-back")
                road_west = place_by_id("the-road-west")
                gate = next((p for p in world["places"] if p["kind"] == "gate"), None)
                if step >= town_shut_until and town_shut_until:
                    town_shut_until = 0
                    for p in world["places"]:
                        if p["kind"] == "town":
                            barred.discard(tuple(p["at"]))
                answered = set(page.evaluate(
                    "() => JSON.parse(window.__save()).state.world.answered || []"))
                # **How many floors are gone is derived, here as in core.** The
                # boss tiles are the record; there is no counter to read.
                floors_down = sum(1 for n in (5, 4, 3, 2, 1)
                                  if f"the-drambus-stack-{n}-boss" in answered)
                # **Read them, then come home.** The run finishes when the
                # walk has been through the door, read both roads on the other
                # side and come back — not the first time it is on Bambulon
                # again, because the first time is usually a defeat carrying it
                # home after one fight, and a run that ends on being beaten
                # once has measured nothing about the map it was beaten on.
                lake_done = "the-bottom-of-the-lake" in answered
                surveyed = "the-trig-stone" in answered
                read_the_lot = (TREYWAY_PROMISES <= answered and seen_kettleworks
                                and floors_down >= 5 and lake_done)
                if seen_treyway and world["id"] == "west-bambulon" and read_the_lot:
                    came_back = True
                    head("back through the door")
                    say(f"  {panel(page)}")
                    say("")
                    say(f"DROPPED THE STACK, DRAINED THE LAKE AND CAME BACK "
                        f"at step {step}. {panel(page)}")
                    break
                keys = page.evaluate(
                    "() => JSON.parse(window.__save()).state.character.registry"
                    ".map(r => r.def)")
                have_deep = "The Deep Gate Key" in keys
                have_witch = "The Witch's Key" in keys

                want = None
                grate = place_by_id("the-way-under-the-lake")
                reach_edge = place_by_id("the-reach-edge")
                if world["id"] == "under-the-lake":
                    if not seen_under:
                        seen_under = True
                        head("under the lake")
                        say(f"  {panel(page)}")
                        phase = "under the lake"
                    down = next((p for p in world["places"] if p["kind"] == "door"), None)
                    boss_here = next((p for p in world["places"] if p["kind"] == "boss"), None)
                    up = place_by_id("the-way-back-up-the-steps")
                    if c["fatigue"] >= 40 and up:
                        want, why = up["at"], "back up the steps"
                    elif down:
                        want, why = down["at"], "under the lake"
                    elif boss_here:
                        want, why = boss_here["at"], "under the lake"
                    else:
                        want, why = None, "under the lake"
                elif world["id"].startswith("the-drambus-stack-"):
                    # Inside a floor. One thing to do and one way out of it.
                    if world["id"] != on_floor:
                        on_floor = world["id"]
                        head(f"the stack, floor {world['id'][-1]}")
                        say(f"  {panel(page)}")
                        phase = f"the stack, floor {world['id'][-1]}"
                    top = next((p for p in world["places"] if p["kind"] == "boss"), None)
                    want, why = (top["at"], phase) if top else (None, phase)
                elif world["id"] == "kettleworks-field":
                    # **The dense map.** Forty of its four hundred tiles carry
                    # something, so the walk is worth reading rather than worth
                    # counting: head for the town first, then the door in the
                    # Stack, then whatever else is unread, then home.
                    if not seen_field:
                        seen_field = True
                        head("kettleworks field")
                        say(f"  {panel(page)}")
                        say(f"  {len(world['places'])} of "
                            f"{world['width'] * world['height']} tiles answer")
                        phase = "kettleworks field"
                    kw = place_by_id("kettleworks")
                    road = place_by_id("the-field-road")
                    stack = place_by_id("the-way-into-the-stack")
                    if c["fatigue"] >= 36 and kw:
                        want, why = kw["at"], "kettleworks field"
                    elif not seen_kettleworks and kw:
                        want, why = kw["at"], "kettleworks field"
                    elif (stack and floors_down < 5 and tuple(stack["at"]) != here
                          and (len(recent) < 6 or sum(recent) * 2 >= len(recent))):
                        # **The Drambus Stack**, if the last dozen fights say
                        # it is worth trying. Rested, because a floor is one
                        # sitting and the thing at the end of it is the hardest
                        # fight on this side of the door.
                        #
                        # **And nothing here falls through to the town.** The
                        # first version did: when the record went bad this
                        # branch failed and the next one sent the walk to the
                        # counter, which it was standing beside — so it walked
                        # in, walked out, walked in, two thousand six hundred
                        # times, buying a tin each visit until it had one
                        # Fnorp. A give-up that has somewhere to go is not a
                        # give-up; it wants to fall through to the grind.
                        want, why = stack["at"], "the stack"
                    else:
                        unread = [tuple(p["at"]) for p in world["places"]
                                  if p["kind"] == "event"
                                  and p["id"] not in answered
                                  and (world["id"], tuple(p["at"])) not in read_over]
                        if len(unread) > 30 and road:
                            # Reading forty cards is not what this run is for.
                            # Six is enough to prove the map answers.
                            unread = unread[:0]
                        if unread:
                            unread.sort(key=lambda a: abs(a[0] - here[0]) + abs(a[1] - here[1]))
                            want, why = list(unread[0]), "kettleworks field"
                        elif road and (floors_down >= 5
                                       or (len(recent) >= 6
                                           and sum(recent) * 2 < len(recent))):
                            # Home, and home is three maps east. **A walk that
                            # could not go back spent a run losing.** There is
                            # one town past the door and its shelf does not let
                            # a weak character catch up; going back is what the
                            # game is for, and the run that was forbidden it
                            # lost two thousand four hundred fights standing
                            # in a field.
                            want, why = road["at"], "back"
                elif world["id"] == "the-treyway":
                    # **Past the door.** Read what the two roads say and then
                    # go back, which is the whole of what M11.1 ships: a map at
                    # a different scale, and a border that remembers.
                    if not seen_treyway:
                        seen_treyway = True
                        head("the treyway")
                        say(f"  {panel(page)}")
                        say(f"  {world['width']}x{world['height']}, "
                            f"{len(world['regions'])} regions: "
                            + ", ".join(r["name"] for r in world["regions"]))
                        phase = "the treyway"
                    # Read, rather than merely stood on: an event's id lands in
                    # `answered` when a choice is taken, which is the same set
                    # the game itself reads and not a tally this file keeps.
                    promises = [tuple(p["at"]) for p in world["places"]
                                if p["kind"] == "event"
                                and p["id"] not in answered
                                and (world["id"], tuple(p["at"])) not in read_over]
                    if c["fatigue"] >= 36 and door_back:
                        # Worn through on a map with no town on it. The way
                        # home is a target like any other — M9.4's lesson,
                        # which is the same one one map further out.
                        want, why = door_back["at"], "back"
                    elif promises:
                        promises.sort(key=lambda a: abs(a[0] - here[0]) + abs(a[1] - here[1]))
                        want, why = list(promises[0]), "the treyway"
                    elif reach_edge and floors_down >= 5 and lake_done and not surveyed:
                        # **Last of all.** The edge refuses without an
                        # instrument and says so, which is a road being shut —
                        # the walk gives up on it after three refusals like any
                        # other and goes and does something else, which is what
                        # a player does when a door wants a thing they have not
                        # built.
                        want, why = reach_edge["at"], "the reach"
                    elif (road_west and floors_down < 5
                          and (len(recent) < 6 or sum(recent) * 2 >= len(recent))):
                        # **West until the Stack is down.** Not "west until you
                        # have seen Kettleworks": with both roads read and the
                        # town seen, the Treyway sent the walk back through the
                        # door every time, and the walk spent twenty thousand
                        # presses crossing three maps to clear one floor.
                        # Nobody walks eleven days back to the pit between
                        # storeys of a tower.
                        want, why = road_west["at"], "the road west"
                    elif door_back:
                        want, why = door_back["at"], "back"
                elif world["id"] != "west-bambulon":
                    # **In the cave, and the way out is a target too.** The
                    # boss while it is standing; the gate once the key is in
                    # hand, because what the key opens is on the other map and
                    # `toward` paths one map at a time. Without this the walker
                    # beat the boss, took the key, and wandered a nine-by-five
                    # room for the rest of the run — which is what the M8.8
                    # transcript ends on and what this one nearly repeated.
                    if have_deep and gate:
                        want, why = gate["at"], "out"
                    elif boss or gate:
                        want, why = (boss or gate)["at"], "boss"
                elif grate and floors_down >= 5 and not lake_done:
                    # The Stack is down, the lake is a bed, and the last thing
                    # written is at the bottom of it.
                    want, why = grate["at"], "the lake"
                elif wall_door and have_deep and not read_the_lot:
                    # **Bank and mend before a border.** The first run through
                    # crossed carrying nine hundred and thirty-two experience
                    # and twenty-two percent worn, lost the first fight on the
                    # other side to a four-hundred-rated creature, and was
                    # carried home having read nothing. That is not the map
                    # being too hard; it is walking into a country you have
                    # never seen with everything you own in your pocket, and a
                    # player does the other thing.
                    # **Bank once, then go.** Twenty-five carried is one
                    # fight's worth at this level, and there are fights between
                    # the pit and the door — so a walk that turned round at
                    # twenty-five turned round at every one of them and never
                    # got past the wall. Six hundred and eighty-eight wins, and
                    # not one floor of the tower. It banks when it is carrying
                    # a level's worth or is properly worn, and otherwise it
                    # keeps walking, which is what setting out means.
                    # **How much you are willing to carry depends on whether
                    # you are winning.** Four hundred is right for a walk that
                    # is beating what it meets and ruinous for one that is not:
                    # a defeat takes the lot, and a run that carried four
                    # hundred through a losing patch banked almost nothing and
                    # finished nine levels down on the run before it.
                    winning = len(recent) < 6 or sum(recent) * 2 >= len(recent)
                    limit = 400 if winning else 60
                    if c["carried"] >= limit or c["fatigue"] >= 28:
                        want, why = town["at"], "home before the door"
                    else:
                        want, why = wall_door["at"], "door"
                elif have_witch and c["level"] >= 9 and cave_mouth and not have_deep:
                    # **No tiredness gate on setting out.** The Cave's mouth is
                    # thirty tiles from the only town and there are fights on
                    # the way, so a walker that only set out while fresh turned
                    # round after two of them and never once arrived. A player
                    # with the key and a level sets out, and drinks on the road.
                    want, why = cave_mouth["at"], "cave"
                elif c["carried"] >= max(25, c["needed"] // 3) or c["fatigue"] >= 44:
                    # **The town on the map you are on.** `town` is the pit's,
                    # read once before the walk started, and heading for its
                    # tile from three maps away walks to whatever happens to be
                    # at those coordinates here — which on the field is a patch
                    # of scrub in the south-west corner.
                    # **What is worth the walk grows.** Twenty-five experience
                    # is a level at the start and a rounding error at eighteen,
                    # so a flat threshold sends a late character home after
                    # every second fight and it never gets four streets from
                    # the pit — which is how a walk never reaches the far
                    # corner an errand is pointing at.
                    here_town = next((p for p in world["places"]
                                      if p["kind"] == "town"), None)
                    want, why = (here_town or town)["at"], "home"
                else:
                    # **Where the log says.** M8.1 built this exactly so a
                    # player does not have to know where a Whisperling lives,
                    # and a playthrough that navigated by its own map would not
                    # be testing the thing the player is given.
                    want, why = None, "grind"
                    log = page.evaluate("() => window.__log()")
                    # **Round robin, not nearest.** One errand asks for four
                    # tins and points at the shelf that sells them, which is
                    # the town — so a walker that always takes the first live
                    # errand walks to the town, buys one tin, and walks to the
                    # town again for ever. Taking them in turn is what a person
                    # does with a list of four things to do.
                    live_ids = [q["id"] for q in log["errands"]
                                if q["stage"] not in ("done", "locked")]
                    order_now = live_ids[errand_turn % len(live_ids):] + \
                        live_ids[:errand_turn % len(live_ids)] if live_ids else []
                    by_id = {q["id"]: q for q in log["errands"]}
                    for qid in order_now:
                        q = by_id[qid]
                        if not q["on_this_map"]:
                            continue
                        g = page.evaluate("(id) => window.__guide(id)", q["id"])
                        if not g:
                            continue
                        spots = [tuple(a) for a in (g["places"] or [])]
                        if not spots:
                            spots = [tuple(a) for a in (g["regions"] or [])]
                        spots = [a for a in spots if a not in done_marks]
                        if not spots:
                            continue
                        spots.sort(key=lambda a: abs(a[0] - here[0]) + abs(a[1] - here[1]))
                        want, why = list(spots[0]), "errand"
                        break
                    if want is None:
                        # Nothing to chase; take the marks the map is drawing.
                        marks = [m for m in page.evaluate("() => window.__errandMarks()")
                                 if tuple(m["at"]) not in done_marks
                                 and tuple(m["at"]) not in barred]
                        spot = next((m for m in marks if m["mark"] == "hand-in"), None) \
                            or next((m for m in marks if m["mark"] == "take"), None)
                        if spot:
                            want, why = spot["at"], "errand"

                if why != phase:
                    head(why)
                    say(f"  {panel(page)}")
                    phase = why

                if want is not None and tuple(want) == here:
                    # Standing on it and nothing opened, so there is nothing
                    # here for us. Do not walk back to it.
                    read_over.add((world["id"], here))
                    #
                    # Cleared whenever an errand finishes, because what a place
                    # is worth changes when the log does — a tile with nothing
                    # on it this morning is where the next thing is handed in.
                    done_marks.add(here)
                    errand_turn += 1
                    want = None
                if want is None:
                    # **The band you can win in.** A starting kit beats what is
                    # in the pit and nothing above it, so the grind walks the
                    # pit's own road until there is a reason to leave it.
                    #
                    # On any other map the pit's rows mean nothing, so the grind
                    # is a patrol round that map's town — which is where a
                    # player who is losing goes and what they do when they get
                    # there. Without this the walk stood in the Stack's Shadow
                    # pressing east and west across the hardest band on the map.
                    home_here = next((p for p in world["places"]
                                      if p["kind"] == "town"), None)
                    if world["id"] == "west-bambulon":
                        lane = 17 if c["level"] < 5 else (14 if c["level"] < 9 else 11)
                        if here[1] != lane:
                            press = toward(world, here, (here[0], lane), barred) or "ArrowUp"
                        else:
                            press = "ArrowRight" if (patrol // 8) % 2 == 0 else "ArrowLeft"
                            patrol += 1
                    elif home_here:
                        # **Between two posts near the town, and neither of them
                        # is the town.** Patrolling the town's own row walks
                        # into it every eight presses, and a run that did spent
                        # four thousand of them opening and shutting a shop.
                        tx, ty = home_here["at"]
                        near, far = None, None
                        for step in (2, 3, 4, 5, 6, 7, 8, 9):
                            for x in (tx + step, tx - step):
                                if not (0 <= x < world["width"]):
                                    continue
                                if not world["walk"][ty][x]:
                                    continue
                                if any(tuple(p["at"]) == (x, ty) for p in world["places"]):
                                    continue
                                if near is None:
                                    near = (x, ty)
                                elif far is None and abs(x - near[0]) >= 4:
                                    far = (x, ty)
                        spot = (far or near) if (patrol // 10) % 2 else (near or far)
                        patrol += 1
                        press = ((toward(world, here, spot, barred) if spot else None)
                                 or "ArrowRight")
                    else:
                        press = "ArrowRight" if (patrol // 8) % 2 == 0 else "ArrowLeft"
                        patrol += 1
                else:
                    target = want
                    press = toward(world, here, target, barred) or "ArrowRight"
                # Where that press was trying to go, so a refusal can be
                # written down against the tile that refused it.
                heading = next((d for d, k in KEYS.items() if k == press), (0, 0))
                aiming = (here[0] + heading[0], here[1] + heading[1])
                before = page.text_content("#walked")
                page.keyboard.press(press)
                if page.text_content("#walked") != before:
                    moved += 1
                elif want is not None:
                    # **A road that is shut is a road you stop walking at.**
                    # M9.3 put two crossings on the map and this walker pressed
                    # north into the first of them for nine thousand steps,
                    # because `toward` paths over terrain and a crossing is not
                    # terrain. A player reads the refusal and goes and fights
                    # something; so does this now — the target is dropped and
                    # the grind lane takes over until a level opens it.
                    stuck[tuple(want)] = stuck.get(tuple(want), 0) + 1
                    if stuck[tuple(want)] >= 3:
                        # **Three times, not once.** A tile that keeps refusing
                        # is a wall until something about the walker changes —
                        # a crossing reads as ordinary ground and turns you
                        # back on what you *are*. Barring on the first refusal
                        # instead walled the walk into a corner of its own
                        # making: six hundred moves out of twenty thousand.
                        barred.add(aiming)
                        say(f"  shut: {tuple(want)} is not reachable yet"
                            f" — {last_said(page) or 'no reason given'}")
                        done_marks.add(tuple(want))
                        read_over.add((world["id"], tuple(want)))
                        done_marks.add(here)
                        errand_turn += 1
                        stuck.clear()
            else:
                say("")
                say(f"DID NOT CROSS AND COME BACK in {STEPS} steps. {panel(page)}")
                say(f"  read on the Treyway: "
                    f"{sorted(TREYWAY_PROMISES & answered) or 'nothing'}")
                say(f"  floors of the Drambus Stack down: {floors_down} of 5")
                say(f"  the thing under the lake: "
                    f"{'beaten' if 'the-bottom-of-the-lake' in answered else 'still down there'}")
            say(f"  ({moved} of {STEPS} presses actually moved)")

            head("what happened")
            c = page.evaluate("() => window.__character()")
            log = page.evaluate("() => window.__log()")
            say(f"  {wins} wins, {losses} losses")
            say(f"  {panel(page)}")
            say(f"  errands: " + ", ".join(
                f"{q['name']} [{q['stage']}]" for q in log["errands"]) or "none")
            say(f"  sheet: " + ", ".join(
                f"{s['n']}{s['unit']} {s['label']}" for s in c["stats"] if s["n"]))
            ctx.close()
            b.close()
    finally:
        httpd.shutdown()

    head("the page")
    if problems:
        for p in sorted(set(problems)):
            say(f"  {p}")
    else:
        say("  no console errors")


main()
