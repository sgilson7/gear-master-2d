// The page. It asks core questions, shows the answers, and decides nothing.
//
// The two save calls are the whole of the boundary: `save_json()` produces the
// file and `load_json()` consumes one. The page never parses a save, never
// validates one, and never writes an error message of its own — when a load
// fails, core has already said why in a sentence and the page shows that.
import init, {
  save_json, load_json, new_game, apply_preset,
  gold, add_gold, draw, rng_state, items,
  piece_count, version, save_version,
} from './pkg/gm2d_wasm.js';

const $ = (id) => document.getElementById(id);

// The convenience autosave. Deliberately not the real save: it lives in this
// browser, it is lost when site data is cleared, and it cannot be handed to
// anybody. It exists so a reload during play is not a punishment.
const AUTOSAVE = 'gm2d.autosave';

function says(text, bad = false) {
  const el = $('says');
  el.textContent = text;
  el.hidden = !text;
  el.classList.toggle('bad', bad);
}

function autosave() {
  try {
    localStorage.setItem(AUTOSAVE, save_json());
  } catch {
    // A private window, or storage the browser has switched off. The real save
    // is a file, so losing this one costs nothing worth reporting.
  }
}

function paint() {
  $('gold').textContent = gold().toLocaleString();
  $('rng').textContent = rng_state();

  const body = $('items').querySelector('tbody');
  body.replaceChildren();
  for (const row of items().split('\n').filter(Boolean)) {
    const [name, rating, rarity] = row.split('\t');
    const tr = document.createElement('tr');
    tr.innerHTML =
      `<td>${name}</td><td class="num">${rating}</td>` +
      `<td><span class="rarity r-${rarity.toLowerCase()}">${rarity}</span></td>`;
    body.appendChild(tr);
  }
  autosave();
}

// Blob -> object URL -> <a download>, straight from the house pattern in
// pdf-redactor. Revoked on the next tick: a URL that is never revoked pins the
// whole file in memory for the life of the tab.
function download(name, text) {
  const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 0);
}

function stamp() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, '0');
  return `gear-master-2d-${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}` +
         `-${p(d.getHours())}${p(d.getMinutes())}.json`;
}

async function main() {
  try {
    await init();
  } catch (e) {
    $('status').textContent = `the engine did not load: ${e}`;
    $('status').classList.add('bad');
    throw e;
  }

  // Restore the convenience copy if there is one. A stale or unreadable one is
  // dropped rather than argued with — it was never the real save.
  let restored = false;
  try {
    const saved = localStorage.getItem(AUTOSAVE);
    if (saved) { load_json(saved); restored = true; }
  } catch {
    localStorage.removeItem(AUTOSAVE);
  }
  if (!restored) apply_preset();

  $('plus').onclick = () => { add_gold(10); paint(); says(''); };
  $('minus').onclick = () => { add_gold(-10); paint(); says(''); };
  $('roll').onclick = () => { $('draw').textContent = draw(); paint(); says(''); };

  $('download').onclick = () => {
    const name = stamp();
    download(name, save_json());
    says(`Saved ${name}. Reload this page and load it back.`);
  };

  $('reset').onclick = () => {
    new_game(Date.now());
    apply_preset();
    $('draw').textContent = '—';
    paint();
    says('New game.');
  };

  $('file').onchange = async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    try {
      const text = new TextDecoder().decode(await f.arrayBuffer());
      load_json(text);
      $('draw').textContent = '—';
      paint();
      says(`Loaded ${f.name}.`);
    } catch (err) {
      // The sentence came from core. Showing it unchanged is the point.
      says(String(err?.message ?? err), true);
    }
    e.target.value = '';
  };

  paint();
  $('status').textContent =
    `core: ${piece_count()} pieces · v${version()} · save v${save_version()}`;
}

main();
