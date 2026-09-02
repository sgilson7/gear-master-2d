// The fight, played back.
//
// The log is already decided — `fight_json` produced the whole thing before a
// single frame was drawn, because combat is a pure function and there is
// nothing to stream. So this is a playback head over a finished transcript, not
// a simulation, and the difference matters: scrubbing, pausing and replaying
// are free, and nothing this file does can change what happened.
//
// **Nothing here is computed that the log reports.** Health taught that once:
// an earlier version subtracted `damage` from a total it kept itself, which
// ignores `absorbed`, so armour soaked a blow, the bar dropped anyway, and both
// sides sat at zero for the rest of a fight that was still going. Armour and
// the four pools arrive the same way now — every one of them off a field the
// log already carries.
//
// The cooldown bars are the one thing reconstructed rather than read: the log
// records when an item activated, and a bar is the gap between one activation
// and the next filling up. That is a rendering of the log, not a second clock.

/// How long a component wobbles after its item goes off.
const SHAKE_MS = 260;

const POOL_COLOUR = ['#5aa8d8', '#c0553f', '#c8a33f', '#4f9e63'];

/// One colour a curse. Four kinds, and the chip names itself in words as well,
/// so the hue is a second channel rather than the only one.
const CURSE_COLOUR = {
  searing: '#c2643f',
  frost: '#5aa8d8',
  stun: '#9b7ad0',
  misfire: '#c8a33f',
};

export class Replay {
  /// `card(item)` renders one item as a card. Passed in rather than imported
  /// so this file does not reach back into `app.js` — and so the fight screen
  /// and the packing panel cannot end up with two card builders.
  constructor(canvas, { you, them, card, boards } = {}) {
    this.c = canvas;
    this.you = you;          // the element holding your item rows
    this.them = them;        // and theirs
    this.card = card;
    // The two read-only boards, if the page gave us any. A fight is two boards
    // and the replay used to show neither.
    this.boards = boards ?? {};
    this.log = null;
    this.t = 0;
    this.speed = 1;
    this.playing = false;
    this.onend = () => {};
    this.onpoint = null;
    this.rows = { player: [], enemy: [] };
  }

  load(log) {
    this.log = log;
    this.t = 0;
    this.playing = false;
    const p = log.player, e = log.enemy;
    // [t, health, max, armour, [four pools]] for each side.
    //
    // The opening row is what each fighter *began* holding, off the log's own
    // starting combatants — not zero. A character who had taken Corked watched
    // the bar open empty and concluded the skill did nothing.
    const zero = [0, 0, 0, 0];
    // The sixth column is the curses up on that side. Read off `Cursed` and
    // `Stunned`, which carry the stack count and the whole time left; nothing
    // here works one out.
    this.track = {
      player: [[0, p.max_health, p.max_health, p.armor ?? 0, p.pools ?? zero, []]],
      enemy: [[0, e?.max_health ?? 1, e?.max_health ?? 1, e?.armor ?? 0, e?.pools ?? zero, []]],
    };
    for (const x of log.entries) {
      this.track.player.push([x.at, x.ph, x.pmax, x.pa, x.pp, x.pc ?? []]);
      this.track.enemy.push([x.at, x.eh, x.emax, x.ea, x.ep, x.ec ?? []]);
    }
    this.buildRows('player', this.you, log.player?.items ?? []);
    this.buildRows('enemy', this.them, e?.items ?? []);
    this.boards.player?.load(log.player?.slots ?? []);
    this.boards.enemy?.load(e?.slots ?? []);
    this.fit();
    this.draw();
  }

  /// One row per item, in the order combat indexed them.
  ///
  /// HTML rather than canvas text, for the reason the item list moved off the
  /// canvas in the first place: a row you can point at is a row the browser
  /// can tell you about, and 11px canvas text cannot be hovered, selected or
  /// read by anything but an eye.
  buildRows(side, host, items) {
    if (!host) return;
    host.replaceChildren();
    this.rows[side] = items.map((item, i) => {
      const row = document.createElement('div');
      row.className = 'tick';
      row.innerHTML =
        `<span class="tick-name">${item.name}</span>` +
        `<span class="tick-track"><i></i></span>` +
        `<span class="tick-hit">${item.hit_for > 0 ? item.hit_for : ''}</span>`;
      if (item.card) {
        row.classList.add('has-card');
        row.tabIndex = 0;
        const show = () => this.onpoint?.(item.card, item.slot);
        row.onpointerenter = show;
        row.onfocus = show;
        row.onpointerleave = () => this.onpoint?.(null);
        row.onblur = () => this.onpoint?.(null);
      }
      host.appendChild(row);
      // The activation times for this index, so the bar is a rendering of the
      // log rather than a clock of its own.
      const acts = this.log.entries
        .filter((e) => e.kind === 'activate' && e.side === side && e.index === i)
        .map((e) => e.at);
      return { el: row, fill: row.querySelector('i'), cd: item.cooldown_ms || 1, acts,
               cells: item.cells ?? [] };
    });
  }

  play() {
    if (!this.log || this.playing) return;
    this.playing = true;
    let last = performance.now();
    const tick = (now) => {
      if (!this.playing) return;
      this.t += (now - last) * this.speed;
      last = now;
      if (this.t >= this.log.duration_ms) {
        this.t = this.log.duration_ms;
        this.playing = false;
        this.draw();
        this.onend();
        return;
      }
      this.draw();
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  finish() {
    this.playing = false;
    this.t = this.log?.duration_ms ?? 0;
    this.draw();
    this.onend();
  }

  /// `[health, max, armour, pools]` at the playback head.
  at(track) {
    let v = track[0];
    for (const row of track) { if (row[0] <= this.t) v = row; else break; }
    return { health: v[1], max: v[2], armor: v[3], pools: v[4] ?? [0, 0, 0, 0],
             chips: v[5] ?? [] };
  }

  /// The canvas sizes its own backing store to its box.
  ///
  /// A canvas is a replaced element: a fixed intrinsic width is a fixed width
  /// *scaled by CSS*, which is how a 1240-wide board once drew at 800 and left
  /// a third of a screen empty underneath it.
  fit() {
    const w = this.c.clientWidth || 900;
    // Two rows taller than it was: each side gained a line for its curses.
    const h = 200;
    const dpr = window.devicePixelRatio || 1;
    this.c.width = Math.round(w * dpr);
    this.c.height = Math.round(h * dpr);
    this.c.style.height = `${h}px`;
    this.c.getContext('2d').setTransform(dpr, 0, 0, dpr, 0, 0);
    this.w = w;
    this.h = h;
  }

  draw() {
    if (!this.log) return;
    const g = this.c.getContext('2d');
    // **Its own ground, and its own ink.** The board next door does this and
    // for the same reason: these bars sit on near-black, so taking the page's
    // ink would put dark grey labels on a dark panel every time the viewer is
    // in light mode — which is what the first version did.
    const GROUND = '#101018';
    const ink = '#e8e8f0', ink3 = '#8f8fa4', line = '#2a2a38', surface = '#1e1e2a';
    const rust = '#c2643f', verd = '#4f9e7a';
    const W = this.w ?? this.c.width;

    g.clearRect(0, 0, W, this.h);
    g.fillStyle = GROUND;
    g.fillRect(0, 0, W, this.h);
    g.font = '12px ui-monospace, Menlo, monospace';
    g.textBaseline = 'alphabetic';

    const health = (y, [now, max], colour) => {
      g.fillStyle = surface;
      g.fillRect(0, y, W, 22);
      const frac = max > 0 ? Math.max(0, Math.min(1, now / max)) : 0;
      g.fillStyle = colour;
      g.fillRect(1, y + 1, (W - 2) * frac, 20);
      g.strokeStyle = line;
      g.strokeRect(.5, y + .5, W - 1, 21);
      g.fillStyle = ink;
      g.textAlign = 'right';
      g.fillText(`${Math.max(0, Math.round(now))} / ${max}`, W - 6, y + 15);
      g.textAlign = 'left';
    };

    /// Armour, on the same scale and at the same size as the health bar above
    /// it, and **wrapping past full**.
    ///
    /// Lifted from the original, whose note says why: the two read as a pair
    /// because they are the same measurement — a full armour bar is as much
    /// armour as you have health, and a pixel is the same number of points in
    /// both. It used to clamp, so every amount from "exactly enough" to "four
    /// times over" drew an identical bar. Each complete bar is another layer,
    /// drawn darker than the one under it, so depth reads as depth without a
    /// number to parse.
    const armour = (y, amount, max) => {
      const shade = (layer) => {
        const f = Math.pow(0.72, Math.min(layer, 5));
        return `rgb(${(150 * f | 0) + 18}, ${(172 * f | 0) + 20}, ${(214 * f | 0) + 26})`;
      };
      g.fillStyle = '#1e1e2a';
      g.fillRect(0, y, W, 16);
      const cap = Math.max(1, max);
      if (amount > 0) {
        const full = Math.floor(amount / cap);
        const rest = (amount % cap) / cap;
        if (full > 0) { g.fillStyle = shade(full - 1); g.fillRect(0, y, W, 16); }
        if (rest > 0) { g.fillStyle = shade(full); g.fillRect(0, y, W * rest, 16); }
      }
      g.strokeStyle = line;
      g.strokeRect(.5, y + .5, W - 1, 15);
      if (amount > 0) {
        // Only worth the words once there is something to say; an empty track
        // reading "0" is noise on a screen that is already busy.
        //
        // Haloed rather than coloured, because the ground under the middle of
        // this bar is whatever layer the wrap landed on: the palest shade and
        // the empty track are both possible under the same label, and no one
        // ink reads on both.
        const label =
          amount > cap ? `${amount} cork  (${(amount / cap).toFixed(1)}x)` : `${amount} cork`;
        g.textAlign = 'center';
        g.lineWidth = 3;
        g.lineJoin = 'round';
        g.strokeStyle = 'rgba(240,244,255,.92)';
        g.strokeText(label, W / 2, y + 12);
        g.fillStyle = '#14141c';
        g.fillText(label, W / 2, y + 12);
        g.textAlign = 'left';
      }
    };

    /// The four pools, each behind its own dot, and only the ones there are
    /// any of. A row of zeroes is a row nobody reads.
    const pools = (y, held) => {
      let x = 0;
      const names = this.log.pools ?? ['the Funny', 'fury', 'devotion', 'harvest'];
      held.forEach((v, i) => {
        if (!v) return;
        g.fillStyle = POOL_COLOUR[i];
        g.beginPath();
        g.arc(x + 5, y - 4, 5, 0, Math.PI * 2);
        g.fill();
        const text = `${v} ${names[i]}`;
        g.fillText(text, x + 14, y);
        x += 14 + g.measureText(text).width + 18;
      });
      if (x === 0) {
        g.fillStyle = ink3;
        g.fillText('nothing banked', 0, y);
      }
    };

    /// The curses up on one side, each with its stacks, what it is doing,
    /// and how long is left.
    ///
    /// **Read, never derived.** `until` is the event's own timestamp plus the
    /// duration the event reported, and the countdown is that minus the
    /// playback head. The effect string — "30/s", "-75%", "1 in 2" — is core's,
    /// off the same constants the simulation reads, so a chip cannot drift
    /// from the fight it is describing.
    const chips = (y, up) => {
      const live = up.filter((c) => c.until > this.t);
      if (!live.length) {
        g.fillStyle = ink3;
        g.fillText('nothing on them', 0, y);
        return;
      }
      let x = 0;
      for (const c of live) {
        const left = ((c.until - this.t) / 1000).toFixed(1);
        const text = `${c.kind}${c.stacks > 1 ? ` ×${c.stacks}` : ''}` +
                     `${c.effect ? ` ${c.effect}` : ''} · ${left}s`;
        const w = g.measureText(text).width + 16;
        g.fillStyle = CURSE_COLOUR[c.kind] ?? ink3;
        g.fillRect(x, y - 11, w, 15);
        g.fillStyle = '#14141c';
        g.fillText(text, x + 8, y);
        x += w + 8;
      }
    };

    const P = this.at(this.track.player), E = this.at(this.track.enemy);
    g.fillStyle = ink3;
    g.fillText('you', 0, 12);
    health(16, [P.health, P.max], verd);
    armour(39, P.armor, P.max);
    g.fillStyle = ink;
    pools(70, P.pools);
    chips(88, P.chips);

    g.fillStyle = ink3;
    g.fillText(this.log.enemy?.name ?? 'it', 0, 112);
    health(116, [E.health, E.max], rust);
    armour(139, E.armor, E.max);
    g.fillStyle = ink;
    pools(170, E.pools);
    chips(188, E.chips);

    g.fillStyle = ink3;
    g.textAlign = 'right';
    g.fillText(`${(this.t / 1000).toFixed(2)}s of ${(this.log.duration_ms / 1000).toFixed(2)}s`,
               W, 12);
    g.textAlign = 'left';

    this.ticks();
    this.boards.player?.draw();
    this.boards.enemy?.draw();
  }

  /// Both sides' cooldown tracks, filled to where the head is — and the
  /// components on the two boards jolting on the tick their item goes off.
  ticks() {
    for (const side of ['player', 'enemy']) {
      const shaking = [];
      for (const r of this.rows[side]) {
        const last = r.acts.filter((t) => t <= this.t).pop() ?? 0;
        const frac = Math.max(0, Math.min(1, (this.t - last) / r.cd));
        r.fill.style.width = `${(frac * 100).toFixed(1)}%`;
        r.el.classList.toggle('ready', frac >= 1);
        // Fired within the shake window, and it has cells on a board.
        const since = this.t - last;
        if (last > 0 && since >= 0 && since < SHAKE_MS && r.cells?.length) {
          shaking.push({ cells: r.cells, at: since / SHAKE_MS });
        }
      }
      const board = this.boards[side];
      if (board) board.shaking = shaking;
    }
  }
}
