"""Drive the built page in real browsers and walk the deploy gate.

`cargo test` cannot reach any of this: whether the wasm instantiates in a
browser, whether the ES module import resolves after cache-busting rewrote the
URLs, whether `<a download>` actually produces a file, and whether
`file.arrayBuffer()` feeds it back in.

The gate M1 has to pass is a sequence, not a state, so this walks it:

    change a number -> download -> reload the page -> upload -> the number is back

and it checks the harder half at the same time. The random stream's position
has to survive too: a save that stored the seed instead of the position would
restore the purse perfectly and then hand the player a draw they had already
seen. So the walk takes a draw before saving, and asserts the *next* draw after
loading is the one that would have come next.

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
    page.wait_for_function("document.getElementById('gold').textContent !== '…'", timeout=20000)

    # The board has to be there, or the save has nothing interesting in it.
    items = page.locator("#items tbody tr").count()
    if items < 5:
        fails.append(f"{name}: the preset assembled {items} items in the browser")

    # --- change a number -----------------------------------------------------
    for _ in range(3):
        page.click("#plus")
    page.click("#roll")
    gold_before = page.text_content("#gold")
    draw_before = page.text_content("#draw")
    rng_before = page.text_content("#rng")
    names_before = page.locator("#items tbody tr td:first-child").all_text_contents()

    if gold_before in (None, "0"):
        fails.append(f"{name}: the purse did not move; it reads {gold_before!r}")
    if draw_before in (None, "", "—"):
        fails.append(f"{name}: the stream produced no draw")

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
    page.wait_for_function("document.getElementById('gold').textContent !== '…'", timeout=20000)
    if page.text_content("#gold") == gold_before:
        fails.append(f"{name}: the purse survived a cleared reload, so the upload proves nothing")

    # --- upload --------------------------------------------------------------
    page.set_input_files("#file", str(path))
    page.wait_for_function(
        f"document.getElementById('gold').textContent === {gold_before!r}", timeout=20000)

    if page.text_content("#rng") != rng_before:
        fails.append(f"{name}: the stream position did not come back "
                     f"({page.text_content('#rng')} vs {rng_before})")

    names_after = page.locator("#items tbody tr td:first-child").all_text_contents()
    if names_after != names_before:
        fails.append(f"{name}: the board came back as different items\n"
                     f"    before: {names_before}\n    after:  {names_after}")

    # The next draw must be the one that would have come next, not a repeat.
    page.click("#roll")
    if page.text_content("#draw") == draw_before:
        fails.append(f"{name}: the stream restarted — the next draw repeated {draw_before}")

    # --- a bad file is refused with a sentence -------------------------------
    junk = ROOT / "dist" / "not-a-save.json"
    junk.write_text('{"format":"gm2d-theme","version":1}')
    page.set_input_files("#file", str(junk))
    page.wait_for_selector("#says.bad", timeout=10000)
    msg = page.text_content("#says")
    if "gm2d-save" not in (msg or ""):
        fails.append(f"{name}: a wrong file was refused with {msg!r}, which names nothing")
    junk.unlink()
    if page.text_content("#gold") != gold_before:
        fails.append(f"{name}: a refused file still changed the game")

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
