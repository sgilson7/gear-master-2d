"""Drive the built page in a real browser and check the gate.

`cargo test` cannot reach any of this: whether the wasm module actually
instantiates in a browser, whether the ES module import resolves after cache
busting has rewritten the URLs, and whether the page says the sentence the
deploy gate asks for. A console error or a failed request fails the run —
"nothing left the origin" is tested here rather than asserted.
"""
import http.server, functools, socketserver, sys, threading
from pathlib import Path
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "dist" / "web"
PORT = 8127

def serve():
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd

def main():
    if not (WEB / "index.html").exists():
        sys.exit("dist/web is not built. Run: make web")
    httpd = serve()
    problems, offsite = [], []
    try:
        with sync_playwright() as p:
            b = p.chromium.launch()
            page = b.new_page()
            page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}")
                    if m.type == "error" else None)
            page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
            page.on("request", lambda r: offsite.append(r.url)
                    if not r.url.startswith(f"http://127.0.0.1:{PORT}") else None)

            page.goto(f"http://127.0.0.1:{PORT}/", wait_until="networkidle")
            page.wait_for_function("document.getElementById('pieces').textContent !== '…'",
                                   timeout=15000)

            pieces = page.text_content("#pieces")
            monsters = page.text_content("#monsters")
            status = page.text_content("#status")
            rows = page.locator("#items tbody tr").count()
            b.close()
    finally:
        httpd.shutdown()

    fails = []
    if problems: fails.append("the page reported errors:\n  " + "\n  ".join(problems))
    if offsite:  fails.append("the page left the origin:\n  " + "\n  ".join(sorted(set(offsite))))
    if not pieces or pieces.replace(",", "") == "0":
        fails.append(f"the catalogue count is {pieces!r}; core did not answer")
    if not monsters or monsters.replace(",", "") == "0":
        fails.append(f"the ladder count is {monsters!r}")
    if "core:" not in (status or ""):
        fails.append(f"the status line is {status!r}, and the gate asks it to say 'core: N pieces'")
    if rows < 5:
        fails.append(f"the preset assembled {rows} items in the browser; it assembles more natively")

    if fails:
        print("\n".join(f"FAIL: {f}" for f in fails)); sys.exit(1)
    print(f"ok: {pieces} pieces, {monsters} creatures, {rows} assembled items")
    print(f"ok: {status}")
    print("ok: no console errors, no off-origin requests")

main()
