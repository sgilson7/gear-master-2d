"""Drive the built page in real browsers and walk the deploy gate.

`cargo test` cannot reach any of this: whether the wasm instantiates in a
browser, whether the ES module import resolves after cache-busting rewrote the
URLs, whether `<a download>` actually produces a file, and whether
`file.arrayBuffer()` feeds it back in.

The gate M1 has to pass is a sequence, not a state, so this walks it:

    walk somewhere -> download -> reload the page -> upload -> you are back there

and it checks the harder half at the same time: the tiles walked, the purse and
the events answered all have to cross the file together, because a save that
restored the position and not the stream would put the player back on the same
tile facing a different map.

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
    """Close an event or encounter card if one opened, so the walk continues."""
    if page.is_visible("#card"):
        if page.is_visible("#card-choices button"):
            page.click("#card-choices button")
        page.wait_for_selector("#card-close", state="visible", timeout=5000)
        page.click("#card-close")
        page.wait_for_selector("#card", state="hidden", timeout=5000)


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

    # --- walk ----------------------------------------------------------------
    # East along the bottom road, out of the pit. Twelve steps is enough to
    # leave the starting town and cross an event tile or two.
    start = page.text_content("#coords")
    for _ in range(12):
        page.keyboard.press("ArrowRight")
        dismiss_card(page)
    walked = page.text_content("#coords")
    if walked == start:
        fails.append(f"{name}: twelve steps east and the player is still at {start}")
    if page.text_content("#walked") in (None, "0", "—"):
        fails.append(f"{name}: the walk counter did not move")

    # Into the map's edge, which must refuse rather than wrap.
    for _ in range(30):
        page.keyboard.press("ArrowDown")
        dismiss_card(page)
    y = int(page.text_content("#coords").split(",")[1])
    if y > 18:
        fails.append(f"{name}: walked to row {y}, which is off the map or into rock")

    gold_before = page.text_content("#gold")
    rng_before = page.text_content("#rng") if page.query_selector("#rng") else None
    pos_before = page.text_content("#coords")
    walked_before = page.text_content("#walked")

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
    print("ok: change, download, reload, upload — the number and the stream both came back")
    print("ok: a wrong file was refused with a sentence and changed nothing")
    print("ok: no console errors, no off-origin requests")


main()
