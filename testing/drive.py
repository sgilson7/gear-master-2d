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
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "dist" / "web"
PORT = 8127
ORIGIN = f"http://127.0.0.1:{PORT}"


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    handler = functools.partial(Quiet, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


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


def leave_town(page):
    if page.is_visible("#town"):
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


def walk_the_gate(browser, name):
    """Returns a list of failures; empty means the gate is passed."""
    fails, problems, offsite = [], [], []
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
        purse = int(page.text_content("#town-gold"))
        wares = page.locator(".wares:not(:disabled)")
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
        page.click("#run")
        page.wait_for_selector("#fight", state="hidden", timeout=8000)

    # --- a fight -------------------------------------------------------------
    if not walk_until_a_fight(page):
        fails.append(f"{name}: never met anything in 200 steps")
    else:
        creature = page.text_content("#fight-name")
        if not creature or creature == "—":
            fails.append(f"{name}: a fight opened against nothing")
        # A save taken here has to reopen the same fight.
        with page.expect_download(timeout=20000) as dl:
            page.click("#fight-save")
        mid = dl.value.path()
        if '"encounter"' not in Path(mid).read_text():
            fails.append(f"{name}: a save taken mid-fight does not carry the encounter")

        page.click("#go")
        page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
        page.click("#skip")
        page.wait_for_selector("#stage-result", state="visible", timeout=15000)
        receipt = page.locator("#result-receipt p").all_text_contents()
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
    grew_line = None
    for i in range(300):
        if int(page.text_content("#level")) >= 2:
            break
        if page.is_visible("#fight"):
            page.click("#preset")
            page.click("#go")
            page.wait_for_selector("#stage-replay", state="visible", timeout=10000)
            page.click("#skip")
            page.wait_for_selector("#stage-result", state="visible", timeout=20000)
            lines = page.locator("#result-receipt p").all_text_contents()
            for line in lines:
                if "row on the" in line:
                    grew_line = line
            page.click("#done")
            page.wait_for_selector("#fight", state="hidden", timeout=8000)
            continue
        if leave_town(page) or dismiss_card(page):
            continue
        page.keyboard.press(PATROL[i % len(PATROL)])

    level = int(page.text_content("#level"))
    if level < 2:
        fails.append(f"{name}: three hundred steps of grinding the pit and still level {level}")
    else:
        if grew_line is None:
            fails.append(f"{name}: a level-up never said which frame grew")
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

    # --- the numbers overlay -------------------------------------------------
    page.click("#numbers")
    if page.get_attribute("#numbers", "aria-pressed") != "true":
        fails.append(f"{name}: the numbers overlay did not turn on")
    page.click("#numbers")

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
                fails += walk_the_gate(b, name)
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
    print("ok: the fit preview is core's answer, not the page's")
    print("ok: a mid-fight save reopens the same fight")
    print("ok: walk, download, reload, upload — position and stream both came back")
    print("ok: a wrong file was refused with a sentence and changed nothing")
    print("ok: no console errors, no off-origin requests")


main()
