"""Hands and eyes for somebody who is not the builder.

**One command per invocation, and the browser outlives all of them.** An agent's
turns are separate processes, so this cannot be a script that opens a browser
and drives it: it is a script that *connects* to a browser somebody already
opened, does one thing, prints what it saw, and exits. Chrome is launched once
with a remote debugging port and Playwright reaches it over CDP.

`drive.py` is the deploy gate and asserts; `playthrough.py` is a walker and
decides for itself. This is neither. It has no opinions, makes no assertions and
never picks a move — every decision belongs to whoever is reading the output.

    python testing/agent_driver.py start        # serve, launch, open
    python testing/agent_driver.py look         # a screenshot, path printed
    python testing/agent_driver.py panel        # the standing panel, as text
    python testing/agent_driver.py screens      # what is open on top of what
    python testing/agent_driver.py log          # the last few lines of the log
    python testing/agent_driver.py history      # the whole sitting
    python testing/agent_driver.py key ArrowUp  # one keypress
    python testing/agent_driver.py click "#go"  # one click, by selector
    python testing/agent_driver.py text "#shelf"  # read any part of the page
    python testing/agent_driver.py buttons      # every clickable thing, listed
    python testing/agent_driver.py save         # download the save, path printed
    python testing/agent_driver.py stop         # close the browser and the server

**What it deliberately does not have.** No `walk_to`, no `fight`, no `pack`, no
"do the sensible thing" of any kind. The M8.8 and M9.4 findings both came from
watching a walker do something a person would not, and a driver that could only
express what the builder expected would find neither.
"""
import functools
import http.server
import json
import os
import shutil
import socketserver
import subprocess
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "dist" / "web"
SHOTS = ROOT / "dist" / "playtest"
STATE = ROOT / "dist" / "playtest" / "session.json"
PORT = 8123
CDP = 9222


def state():
    try:
        return json.loads(STATE.read_text())
    except Exception:
        return {}


def save_state(d):
    SHOTS.mkdir(parents=True, exist_ok=True)
    STATE.write_text(json.dumps(d))


# ------------------------------------------------------------------ the server


def serve_forever():
    """A static server that survives this process exiting.

    Its own process, detached, because the agent's next turn is a different
    process and the page has to still be there.
    """
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(WEB))
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), handler)
    httpd.serve_forever()


def start():
    if not (WEB / "index.html").exists():
        sys.exit("dist/web is not built. Run: make web")
    SHOTS.mkdir(parents=True, exist_ok=True)

    # The server, detached.
    server = subprocess.Popen(
        [sys.executable, str(Path(__file__).resolve()), "_serve"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )

    # And a browser with a debugging port, which every later invocation
    # connects to rather than launching its own.
    from playwright.sync_api import sync_playwright

    profile = SHOTS / "chrome-profile"
    if profile.exists():
        shutil.rmtree(profile, ignore_errors=True)
    profile.mkdir(parents=True, exist_ok=True)
    exe = None
    with sync_playwright() as p:
        exe = p.chromium.executable_path
    browser = subprocess.Popen(
        [
            exe,
            f"--remote-debugging-port={CDP}",
            f"--user-data-dir={profile}",
            "--no-first-run",
            "--no-default-browser-check",
            "--window-size=1280,900",
            f"http://127.0.0.1:{PORT}/",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    save_state({"server": server.pid, "browser": browser.pid})
    # Wait for the engine to have loaded, so the first `look` is a game.
    with connect() as page:
        page.wait_for_function(
            "document.getElementById('coords').textContent !== '—'", timeout=40000
        )
    print(f"open at http://127.0.0.1:{PORT}/ — the game is up")


class connect:
    """A page on the browser that is already running."""

    def __enter__(self):
        from playwright.sync_api import sync_playwright

        self.pw = sync_playwright().start()
        last = None
        for _ in range(60):
            try:
                self.browser = self.pw.chromium.connect_over_cdp(f"http://127.0.0.1:{CDP}")
                break
            except Exception as e:  # the browser is still starting
                last = e
                time.sleep(0.5)
        else:
            self.pw.stop()
            raise SystemExit(f"no browser on {CDP}: {last}. Run: agent_driver.py start")
        ctx = self.browser.contexts[0]
        self.page = ctx.pages[0] if ctx.pages else ctx.new_page()
        return self.page

    def __exit__(self, *a):
        # **Disconnect, and say nothing about it.** Closing a CDP connection
        # races the browser's own teardown of the transport, and Playwright
        # prints an unretrieved-future traceback about it — which lands in the
        # middle of an agent's output looking exactly like a crash.
        try:
            self.browser.close()
        except Exception:
            pass
        finally:
            try:
                self.pw.stop()
            except Exception:
                pass


# ------------------------------------------------------------------- the eyes


def look():
    with connect() as page:
        SHOTS.mkdir(parents=True, exist_ok=True)
        n = state().get("shot", 0) + 1
        d = state()
        d["shot"] = n
        save_state(d)
        out = SHOTS / f"look-{n:03d}.png"
        page.screenshot(path=str(out))
        print(out)


def panel():
    with connect() as page:
        # **What a person sees**, which is not the same as what is in the DOM:
        # rows are hidden when they have nothing to say, and a driver that read
        # them anyway would report a survey to somebody who has none.
        rows = page.evaluate(
            """() => [...document.querySelectorAll('#here dl > div')]
                 .filter(d => getComputedStyle(d).display !== 'none')
                 .map(d => d.children[0].textContent + ': ' + d.children[1].textContent)"""
        )
        sheet = page.evaluate(
            "() => [...document.querySelectorAll('#sheet li')].map(li => li.textContent)"
        )
        print(page.text_content("#region"))
        for r in rows:
            print("  " + r)
        if sheet:
            print("  what you are: " + "; ".join(sheet))


def screens():
    with connect() as page:
        open_now = page.evaluate(
            """() => ['card','fight','town','tree','log','history','ending','fork','vendor']
                 .filter(id => { const e = document.getElementById(id);
                                 return e && !e.hidden; })"""
        )
        print(", ".join(open_now) if open_now else "the map")


def show_log():
    with connect() as page:
        for line in page.eval_on_selector_all(
            "#tape li", "els => els.map(e => e.textContent)"
        ):
            print(line.strip())


def history():
    with connect() as page:
        was = page.is_hidden("#history")
        if was:
            page.click("#history-open")
            page.wait_for_selector("#history", state="visible", timeout=5000)
        for line in page.eval_on_selector_all(
            "#history-list li", "els => els.map(e => e.textContent)"
        ):
            print(line.strip())
        if was:
            page.click("#history-close")


def text(selector):
    with connect() as page:
        got = page.evaluate(
            """(sel) => [...document.querySelectorAll(sel)]
                 .map(e => e.innerText).join('\\n---\\n')""",
            selector,
        )
        print(got or f"(nothing matches {selector})")


def buttons():
    """Everything you could click right now, topmost screen first."""
    with connect() as page:
        got = page.evaluate(
            """() => {
              const vis = (e) => {
                const r = e.getBoundingClientRect();
                if (!r.width || !r.height) return false;
                for (let n = e; n; n = n.parentElement) if (n.hidden) return false;
                return true;
              };
              return [...document.querySelectorAll('button, .wares, label.btn')]
                .filter(vis)
                .map(e => ({
                  id: e.id || null,
                  label: (e.innerText || '').split('\\n')[0].slice(0, 60),
                  disabled: !!e.disabled,
                }));
            }"""
        )
        for b in got:
            mark = " (greyed out)" if b["disabled"] else ""
            where = f"#{b['id']}" if b["id"] else ""
            print(f"{where:<18} {b['label']}{mark}")


# ------------------------------------------------------------------ the hands


def key(k):
    with connect() as page:
        page.keyboard.press(k)
        page.wait_for_timeout(250)
        print(f"pressed {k}")


def click(selector):
    with connect() as page:
        try:
            page.click(selector, timeout=4000)
        except Exception as e:
            print(f"could not click {selector}: {str(e).splitlines()[0]}")
            return
        page.wait_for_timeout(250)
        print(f"clicked {selector}")


def save():
    with connect() as page:
        with page.expect_download(timeout=20000) as dl:
            page.click("#download")
        out = SHOTS / "save.json"
        dl.value.save_as(str(out))
        print(out)


def load(path):
    with connect() as page:
        page.set_input_files("#file", str(Path(path).resolve()))
        page.wait_for_timeout(600)
        print("loaded")


def stop():
    d = state()
    for pid in (d.get("browser"), d.get("server")):
        if pid:
            try:
                os.killpg(os.getpgid(pid), 15)
            except Exception:
                try:
                    os.kill(pid, 15)
                except Exception:
                    pass
    save_state({})
    print("stopped")


COMMANDS = {
    "start": start,
    "_serve": serve_forever,
    "look": look,
    "panel": panel,
    "screens": screens,
    "log": show_log,
    "history": history,
    "text": text,
    "buttons": buttons,
    "key": key,
    "click": click,
    "save": save,
    "load": load,
    "stop": stop,
}

if __name__ == "__main__":
    if len(sys.argv) < 2 or sys.argv[1] not in COMMANDS:
        sys.exit(__doc__)
    COMMANDS[sys.argv[1]](*sys.argv[2:])
    # **Out, without unwinding.** Disconnecting a CDP connection races the
    # browser's own transport teardown, and the loser is an unretrieved-future
    # traceback printed at interpreter shutdown — which lands in the middle of
    # an agent's output looking exactly like a crash. There is nothing to
    # handle and nothing left to do: the work is done and printed.
    sys.stdout.flush()
    sys.stderr.flush()
    if sys.argv[1] != "_serve":
        os._exit(0)
