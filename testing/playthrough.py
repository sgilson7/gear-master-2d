"""Play the demo from a new game to the ending, and write down what it said.

**Not the gate.** `drive.py` walks a route chosen to exercise checks: it
teleports, plants saves, and asserts. This does none of that. It starts a new
game, buys with the money it has, packs with the button a player is given,
walks where a player would walk, reads every screen it is shown, and stops when
the door opens or when it cannot get any further.

The output is a transcript, and the transcript is the deliverable. What it
catches is the thing a suite cannot: a game that is green and unplayable.

    python testing/playthrough.py [chromium] > transcript.txt
"""
import functools
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
STEPS = 9000


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


SHUT = {"rock", "water"}
KEYS = {(1, 0): "ArrowRight", (-1, 0): "ArrowLeft", (0, 1): "ArrowDown", (0, -1): "ArrowUp"}


def toward(world, here, target):
    """One step towards a tile, around whatever is in the way.

    **A real path, not a bearing.** Walking one axis at a time walks into the
    Burnwarp and stays there: a blocked step draws nothing and costs nothing,
    so a walker that keeps pressing into a cliff presses into it for ever. The
    first version of this managed a hundred and forty-one moves out of four
    thousand presses.
    """
    if tuple(here) == tuple(target):
        return None
    rows = world["rows"]
    w, h = world["width"], world["height"]
    ok = lambda x, y: 0 <= x < w and 0 <= y < h and rows[y][x] not in SHUT
    from collections import deque
    seen = {tuple(target)}
    q = deque([tuple(target)])
    # From the target outwards, so the first neighbour of `here` that is
    # reached is a step along a shortest path.
    while q:
        cx, cy = q.popleft()
        for dx, dy in KEYS:
            nx, ny = cx + dx, cy + dy
            if not ok(nx, ny) or (nx, ny) in seen:
                continue
            if (nx, ny) == tuple(here):
                return KEYS[(-dx, -dy)]
            seen.add((nx, ny))
            q.append((nx, ny))
    return None


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
                    if fight(page):
                        wins += 1
                    else:
                        losses += 1
                    continue
                if page.is_visible("#card"):
                    card(page)
                    continue
                if page.is_visible("#town"):
                    say(f"  in town: {panel(page)}")
                    in_town(page)
                    spend_points(page)
                    target = None
                    # A level lands in a town, and a level is what a crossing
                    # asks for — so everything that was out of reach is worth
                    # trying again from here.
                    done_marks.clear()
                    stuck.clear()
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

                door = next((p for p in world["places"] if p["kind"] == "door"), None)
                boss = next((p for p in world["places"] if p["kind"] == "boss"), None)
                gate = next((p for p in world["places"] if p["kind"] == "gate"), None)
                keys = page.evaluate(
                    "() => JSON.parse(window.__save()).state.character.registry"
                    ".map(r => r.def)")
                have_deep = "The Deep Gate Key" in keys
                have_witch = "The Witch's Key" in keys

                want = None
                if world["id"] != "west-bambulon":
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
                elif door and have_deep:
                    want, why = door["at"], "door"
                elif have_witch and c["level"] >= 9 and gate:
                    # **No tiredness gate on setting out.** The Cave's mouth is
                    # thirty tiles from the only town and there are fights on
                    # the way, so a walker that only set out while fresh turned
                    # round after two of them and never once arrived. A player
                    # with the key and a level sets out, and drinks on the road.
                    want, why = gate["at"], "cave"
                elif c["carried"] >= max(25, c["needed"] // 3) or c["fatigue"] >= 44:
                    # **What is worth the walk grows.** Twenty-five experience
                    # is a level at the start and a rounding error at eighteen,
                    # so a flat threshold sends a late character home after
                    # every second fight and it never gets four streets from
                    # the pit — which is how a walk never reaches the far
                    # corner an errand is pointing at.
                    want, why = town["at"], "home"
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
                                 if tuple(m["at"]) not in done_marks]
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
                    lane = 17 if c["level"] < 5 else (14 if c["level"] < 9 else 11)
                    if here[1] != lane:
                        press = toward(world, here, (here[0], lane)) or "ArrowUp"
                    else:
                        press = "ArrowRight" if (patrol // 8) % 2 == 0 else "ArrowLeft"
                        patrol += 1
                else:
                    target = want
                    press = toward(world, here, target) or "ArrowRight"
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
                        say(f"  shut: {tuple(want)} is not reachable yet"
                            f" — {last_said(page) or 'no reason given'}")
                        done_marks.add(tuple(want))
                        done_marks.add(here)
                        errand_turn += 1
                        stuck.clear()
            else:
                say("")
                say(f"DID NOT REACH THE ENDING in {STEPS} steps. {panel(page)}")
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
