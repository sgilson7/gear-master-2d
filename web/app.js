// The page. It asks core questions and prints the answers; it decides nothing.
//
// No bundler and no framework, per the house pattern: an ES module, a wasm
// module beside it, and one stylesheet.
import init, { piece_count, monster_count, preset_items, version } from './pkg/gm2d_wasm.js';

const $ = (id) => document.getElementById(id);

function fail(e) {
  $('status').textContent = `the engine did not load: ${e}`;
  $('status').classList.add('bad');
  // Rethrow so the failure reaches the console and the UI test, rather than
  // leaving a page that merely looks quiet.
  throw e;
}

async function main() {
  try {
    await init();
  } catch (e) {
    fail(e);
    return;
  }

  $('pieces').textContent = piece_count().toLocaleString();
  $('monsters').textContent = monster_count().toLocaleString();

  const rows = preset_items().split('\n').filter(Boolean);
  const body = $('items').querySelector('tbody');
  for (const row of rows) {
    const [name, rating, rarity] = row.split('\t');
    const tr = document.createElement('tr');
    tr.innerHTML =
      `<td>${name}</td>` +
      `<td class="num">${rating}</td>` +
      `<td><span class="rarity r-${rarity.toLowerCase()}">${rarity}</span></td>`;
    body.appendChild(tr);
  }

  // The gate's sentence, in the words the plan wrote it in.
  $('status').textContent = `core: ${piece_count()} pieces · v${version()}`;
}

main();
