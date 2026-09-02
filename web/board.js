// The gear board, in Canvas 2D.
//
// This file draws and hit-tests. It answers no rules and it chooses no colours.
//
// Which cells a component may occupy, which pieces form an item, whether that
// item assembled, what it is called, what it is worth — and, since the board
// was rebuilt against the original's design, **what colour it is and what mark
// it wears** — are all core's, fetched through `board_json`, `legal_anchors`
// and `look_json`. The accessibility contract lives in `core::look` and is held
// by `tests/look.rs`; a page that picked its own colours would be a page with
// its own untested story about colour blindness.
//
// The system it draws, in one line: **slot → a motif and a hue, role →
// brightness.** Any two of those three channels can be lost and the board still
// says which grid a piece belongs to and which part of a recipe it is.

const BAG_THUMB = 34;   // a bag thumbnail's box
const BAG_COL = 190;    // one bag entry's width
const BAG_ROW = 42;     // and its height
const CELL = 34;
const GAP = 2;
const PAD = 8;

/// The mark, at the fractions the original uses so it reads at any size.
///
/// Module-level rather than a method: the motif is the shape half of the
/// colourblind triple-encoding, and everything that draws a cell — this board,
/// and the read-only grids on the fight screen — has to draw the same one.
export function paintMotif(g, x, y, cell, kind, ink, alpha) {
  const t = Math.max(cell * 0.11, 1.5);
  g.save();
  g.globalAlpha = alpha ?? 0.4;
  g.strokeStyle = ink ?? '#fff';
  g.fillStyle = ink ?? '#fff';
  g.lineWidth = t;
  g.lineCap = 'butt';
  const f = (n) => n * cell;
  const line = (x1, y1, x2, y2) => {
    g.beginPath(); g.moveTo(x + f(x1), y + f(y1)); g.lineTo(x + f(x2), y + f(y2)); g.stroke();
  };
  if (kind === 'diagonal') line(0.24, 0.76, 0.76, 0.24);
  else if (kind === 'dome') {
    g.beginPath(); g.arc(x + f(0.5), y + f(0.52), f(0.20), 0, Math.PI * 2); g.fill();
  } else if (kind === 'bands') { line(0.22, 0.34, 0.78, 0.34); line(0.22, 0.64, 0.78, 0.64); }
  else if (kind === 'weave') { line(0.5, 0.22, 0.5, 0.78); line(0.22, 0.5, 0.78, 0.5); }
  else if (kind === 'straps') { line(0.34, 0.22, 0.34, 0.78); line(0.64, 0.22, 0.64, 0.78); }
  else if (kind === 'shared') {
    const r = 0.26, p = [[0.5, 0.5 - r], [0.5 + r, 0.5], [0.5, 0.5 + r], [0.5 - r, 0.5]];
    for (let i = 0; i < 4; i++) {
      const a = p[i], b = p[(i + 1) % 4];
      line(a[0], a[1], b[0], b[1]);
    }
  }
  g.restore();
}

export class Board {
  constructor(canvas, api) {
    this.c = canvas;
    this.api = api;   // { boardJson, legalAnchors, place, pickUp, rotate, toggleLock, look, lookOver }
    this.state = null;
    this.look = JSON.parse(api.look());
    this.held = null;
    this.legal = null;
    this.legalSlot = null;
    this.hover = null;
    this.onchange = () => {};
    this.onhold = () => {};
    this.onpoint = () => {};
    /// Somebody else's chance at a click on a component. Returns true when it
    /// took it, and the board does nothing further with that press.
    this.onclaim = null;
    this.pointed = null;   // pieces of the item under the cursor
    this.slotOrder = ['weapon', 'helmet', 'chest', 'gloves', 'greaves'];

    canvas.addEventListener('mousemove', (e) => this.move(e));
    canvas.addEventListener('mouseleave', () => {
      this.hover = null;
      if (this.pointed) { this.pointed = null; this.onpoint(null); }
      if (this.pointedPiece) { this.pointedPiece = null; this.onpiece?.(null); }
      this.draw();
    });
    canvas.addEventListener('mousedown', (e) => this.press(e));
    canvas.addEventListener('contextmenu', (e) => { e.preventDefault(); this.rotateHeld(); });

    // The assembled outline pulses, so the board has to keep drawing.
    const tick = () => { if (this.state) this.draw(); requestAnimationFrame(tick); };
    requestAnimationFrame(tick);

    // And it has to re-fit when the window changes, or the cells go back to
    // being scaled by CSS.
    addEventListener('resize', () => { this.fit(); this.draw(); });
  }

  refresh() {
    this.state = JSON.parse(this.api.boardJson());
    this.fit();
    this.draw();
    this.onchange(this.state);
    this.onhold(this.held?.name ?? null);
  }

  /// Size the canvas to the room it actually has, then lay out into it.
  ///
  /// A fixed intrinsic size is a fixed intrinsic size *scaled by CSS*: the
  /// canvas was 1240 wide displayed at 800, so every 34px cell arrived as 22
  /// and there was a third of a screen of dead space underneath. One backing
  /// pixel per CSS pixel, and a height that is whatever the content came to.
  fit() {
    if (!this.state) return;
    const w = Math.max(560, Math.round(this.c.clientWidth || this.c.width));
    if (this.c.width !== w) this.c.width = w;
    this.layout();
    // The bag is the last thing on the board, so its final row sets the height.
    const COL = BAG_COL, ROW = BAG_ROW;
    const perRow = Math.max(1, Math.floor((w - PAD * 2) / COL));
    const rows = Math.ceil(this.state.bag.length / perRow);
    const h = Math.round(this.bagY + rows * ROW + PAD);
    if (this.c.height !== h) {
      this.c.height = h;
      this.layout();
    }
    // A canvas is a replaced element, so `height: auto` re-derives its box from
    // the intrinsic aspect ratio — which squashed a 414px backing store into
    // 374 CSS pixels and drew every cell a tenth short. Pin the CSS height to
    // the backing height instead.
    //
    // Through the border, not around it: the page sets `box-sizing: border-box`
    // everywhere, so a bare `height: 372px` is a *border* box of 372 and a
    // content box of 370. Measured rather than assumed, so a change to the
    // border width cannot quietly bring the squash back.
    const chrome = this.c.offsetHeight - this.c.clientHeight;
    this.c.style.height = `${h + chrome}px`;
  }

  layout() {
    if (!this.state) return;
    const gw = 6 * (CELL + GAP) + PAD * 2;
    this.boxes = {};
    let x = PAD, y = 30, rowH = 0;
    for (const name of this.slotOrder) {
      const s = this.state.slots.find((s) => s.slot === name);
      const gh = s.rows * (CELL + GAP) + PAD * 2;
      if (x + gw > this.c.width - PAD && x > PAD) { x = PAD; y += rowH + 40; rowH = 0; }
      this.boxes[name] = { x, y, w: gw, h: gh, rows: s.rows };
      rowH = Math.max(rowH, gh);
      x += gw + PAD;
    }
    this.bagY = y + rowH + 46;
  }

  cellAt(px, py) {
    for (const name of this.slotOrder) {
      const b = this.boxes[name];
      const gx = Math.floor((px - b.x - PAD) / (CELL + GAP));
      const gy = Math.floor((py - b.y - PAD) / (CELL + GAP));
      if (gx >= 0 && gx < 6 && gy >= 0 && gy < b.rows &&
          px >= b.x && px <= b.x + b.w && py >= b.y && py <= b.y + b.h) {
        return { slot: name, x: gx, y: gy };
      }
    }
    return null;
  }

  bagAt(px, py) {
    if (py < this.bagY) return null;
    const COL = BAG_COL, ROW = BAG_ROW;
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / COL));
    const i = Math.floor((py - this.bagY) / ROW) * perRow + Math.floor((px - PAD) / COL);
    return this.state.bag[i] ?? null;
  }

  pieceAt(slot, x, y) {
    const s = this.state.slots.find((s) => s.slot === slot);
    return s.placed.find((p) => p.cells.some(([cx, cy]) => cx === x && cy === y)) ?? null;
  }

  /// The component in hand, as core currently describes it.
  ///
  /// **Looked up every time rather than cached at pick-up.** A copy taken when
  /// the piece was lifted goes stale the moment anything changes it, and the
  /// thing that changes it most is the player turning it: rotating in hand
  /// moved the shape in core and left the cursor drawing the old one. Picking
  /// a piece up unequips it, so it is always in the bag while it is in hand.
  heldPiece() {
    if (!this.held) return null;
    return this.state?.bag.find((p) => p.id === this.held.id) ?? null;
  }

  /// The cells it would occupy, relative to wherever it is dropped.
  heldCells() {
    return this.heldPiece()?.cells ?? [[0, 0]];
  }

  // --------------------------------------------------------------- input

  move(e) {
    const r = this.c.getBoundingClientRect();
    const px = (e.clientX - r.left) * (this.c.width / r.width);
    const py = (e.clientY - r.top) * (this.c.height / r.height);
    this.mouse = { px, py };
    const cell = this.cellAt(px, py);
    const changed = cell?.slot !== this.hover?.slot;
    this.hover = cell;
    if (this.held && cell && (changed || !this.legal)) this.askLegal(cell.slot);
    this.point(cell, px, py);
    this.draw();
  }

  /// Which item the cursor is over, if any.
  ///
  /// An item rather than a component: a player pointing at a blade is asking
  /// about the weapon it is part of, and the card is about the item. Reported
  /// by its piece list, which is what identifies one.
  point(cell, px, py) {
    let key = null;
    // **Two answers, not one.** The card in the panel is about the *item*,
    // because pointing at a blade is asking about the weapon it is part of.
    // The hover card is about the *component* under the cursor, because that
    // is the thing you are about to pick up. Both are true at once and neither
    // replaces the other, so both are reported.
    let piece = null;
    if (!this.held && cell) {
      const p = this.pieceAt(cell.slot, cell.x, cell.y);
      if (p) {
        piece = p;
        const slot = this.state.slots.find((s) => s.slot === cell.slot);
        const item = slot.items.find((i) => i.pieces.includes(p.id));
        if (item) key = item.pieces.join(',');
      }
    }
    // The bag, too — a loose component has no item, so it names itself.
    if (!this.held && py >= this.bagY) {
      const loose = this.bagAt(px, py);
      if (loose) {
        piece = loose;
        key = key ?? `bag:${loose.id}`;
      }
    }
    if (key !== this.pointed) {
      this.pointed = key;
      this.onpoint(key);
    }
    if ((piece?.id ?? null) !== (this.pointedPiece?.id ?? null)) {
      this.pointedPiece = piece;
      this.onpiece?.(piece, this.c.getBoundingClientRect(), px, py);
    }
  }

  // The one question this file is not allowed to answer itself.
  askLegal(slot) {
    if (!this.held) { this.legal = null; return; }
    this.legal = new Set(
      JSON.parse(this.api.legalAnchors(this.held.id, slot)).map(([x, y]) => `${x},${y}`));
    this.legalSlot = slot;
    // And how it would read once it crossed in — grey outside, the grid's
    // colour and mark inside.
    this.held.over = JSON.parse(this.api.lookOver(this.held.id, slot));
  }

  press(e) {
    if (e.button === 2) return;
    const r = this.c.getBoundingClientRect();
    const px = (e.clientX - r.left) * (this.c.width / r.width);
    const py = (e.clientY - r.top) * (this.c.height / r.height);
    const cell = this.cellAt(px, py);

    if (this.held) {
      if (cell) {
        const why = this.api.place(this.held.id, cell.slot, cell.x, cell.y);
        if (why) { this.say(why); return; }
      } else if (this.held.from) {
        this.api.pickUp(this.held.id);
      }
      this.held = null; this.legal = null;
      this.refresh();
      return;
    }

    if (cell) {
      const p = this.pieceAt(cell.slot, cell.x, cell.y);
      if (!p) return;
      // **Something else may want this click.** An ench in hand is bolted onto
      // the component rather than picking it up — one target, two gestures,
      // and which is happening is decided by whether anything is in hand. The
      // page owns that; the board asks and does as it is told.
      if (this.onclaim?.(p.id)) { this.refresh(); return; }
      if (e.shiftKey) { this.api.toggleLock(p.id); this.refresh(); return; }
      const why = this.api.pickUp(p.id);
      if (why) { this.say(why); return; }
      this.held = { id: p.id, from: cell.slot, name: p.name, slot: cell.slot };
      this.refresh();
      this.askLegal(cell.slot);
      this.draw();
      return;
    }

    const loose = this.bagAt(px, py);
    if (loose) {
      if (this.onclaim?.(loose.id)) { this.refresh(); return; }
      this.held = { id: loose.id, from: null, name: loose.name, slot: loose.slot };
      this.askLegal(loose.slot);
      this.onhold(this.held.name);
      this.draw();
    }
  }

  rotateHeld() {
    const id = this.held?.id
      ?? (this.hover && this.pieceAt(this.hover.slot, this.hover.x, this.hover.y)?.id);
    if (id === undefined || id === null) return;
    const why = this.api.rotate(id);
    if (why) { this.say(why); return; }
    // Re-read the board, then re-ask where the *turned* shape fits. Both, in
    // that order: the legal anchors for a piece on its side are a different
    // set, and asking before the refresh would answer about the old shape.
    this.refresh();
    if (this.held) this.askLegal(this.legalSlot ?? this.held.slot);
    this.draw();
  }

  say(text) { this.onsay?.(text); }

  // --------------------------------------------------------------- drawing

  /// The board draws on its own dark ground, so its text is the game's ink
  /// rather than the page's — which flips with the viewer's theme and would
  /// leave dark labels on a near-black panel half the time.
  chrome() {
    return { ink: '#e8e8f0', ink3: '#8f8fa4', line: '#2a2a38' };
  }

  /// One cell: the fill, then the slot's mark over it.
  ///
  /// No inset and no per-cell border — neighbouring cells of the same component
  /// have to meet without a seam, or a four-cell blade reads as four squares.
  cellFill(g, px, py, look) {
    g.fillStyle = look.fill;
    g.fillRect(px, py, CELL, CELL);
    this.motif(g, px, py, CELL, look.motif, look.ink, look.ink_alpha);
  }

  /// The mark. Delegates, because the fight screen paints the creature's
  /// board with the same marks and two copies of this would be two answers to
  /// "what does a helmet cell look like".
  motif(g, x, y, cell, kind, ink, alpha) {
    paintMotif(g, x, y, cell, kind, ink, alpha);
  }

  /// Trace the outside edge of a set of cells and nothing else.
  ///
  /// Used for a component's own edge, for an item's outline, and for a lock —
  /// all three want "where does this shape end", and none of them wants a line
  /// through the middle of it.
  /// `cell` defaults to the board's, and is passed in only by the bag's
  /// thumbnails, which draw the same shapes smaller.
  edge(g, cells, origin, colour, width, inset = 0, dash = null, cell = CELL) {
    const own = new Set(cells.map(([x, y]) => `${x},${y}`));
    g.save();
    g.strokeStyle = colour;
    g.lineWidth = width;
    if (dash) g.setLineDash(dash);
    g.beginPath();
    for (const [x, y] of cells) {
      const [px, py] = origin(x, y);
      const a = px - inset, b = py - inset, w = cell + inset * 2;
      if (!own.has(`${x},${y - 1}`)) { g.moveTo(a, b); g.lineTo(a + w, b); }
      if (!own.has(`${x},${y + 1}`)) { g.moveTo(a, b + w); g.lineTo(a + w, b + w); }
      if (!own.has(`${x - 1},${y}`)) { g.moveTo(a, b); g.lineTo(a, b + w); }
      if (!own.has(`${x + 1},${y}`)) { g.moveTo(a + w, b); g.lineTo(a + w, b + w); }
    }
    g.stroke();
    g.restore();
  }

  draw() {
    if (!this.state) return;
    const g = this.c.getContext('2d');
    const C = this.chrome();
    const L = this.look;
    g.clearRect(0, 0, this.c.width, this.c.height);
    // The grid colours were picked against a near-black panel, and read as
    // floating holes on anything else. The board brings its own ground.
    g.fillStyle = '#101018';
    g.fillRect(0, 0, this.c.width, this.c.height);
    g.font = '11px ui-monospace, Menlo, monospace';
    g.textBaseline = 'alphabetic';

    // The pulse the assembled outline rides on.
    const t = performance.now() / 1000;
    const [lo, hi] = L.assembled_alpha;
    const pulse = lo + (hi - lo) * (Math.sin(t * L.pulse_hz * Math.PI * 2) * 0.5 + 0.5);

    for (const name of this.slotOrder) {
      const b = this.boxes[name];
      const s = this.state.slots.find((s) => s.slot === name);
      const origin = (x, y) => [b.x + PAD + x * (CELL + GAP), b.y + PAD + y * (CELL + GAP)];

      g.fillStyle = C.ink3;
      g.fillText(name.toUpperCase(), b.x, b.y - 8);

      // The empty grid: low contrast on purpose. It is a ruler, not a subject.
      for (let y = 0; y < s.rows; y++) {
        for (let x = 0; x < 6; x++) {
          const [px, py] = origin(x, y);
          g.fillStyle = (x + y) % 2 === 0 ? L.cell_a : L.cell_b;
          g.fillRect(px, py, CELL, CELL);
        }
      }

      // Components. Fill and mark first with no inset, then each one's own
      // outer edge — so a piece reads as one shape and the lines you see
      // inside an item are the seams between its parts.
      for (const p of s.placed) {
        for (const [cx, cy] of p.cells) {
          const [px, py] = origin(cx, cy);
          this.cellFill(g, px, py, p);
        }
        this.edge(g, p.cells, origin, L.piece_edge, Math.min(Math.max(CELL * 0.09, 1.5), 3));
      }

      // Effect and trigger markers, so a board shows where its rules live.
      for (const p of s.placed) {
        if (!p.effect && !p.trigger) continue;
        const [cx, cy] = p.cells[0];
        const [px, py] = origin(cx, cy);
        let mx = px + CELL - 5;
        if (p.trigger) { g.fillStyle = L.trigger; g.beginPath(); g.arc(mx, py + 5, 2.2, 0, 7); g.fill(); mx -= 6; }
        if (p.effect) { g.fillStyle = L.effect; g.beginPath(); g.arc(mx, py + 5, 2.2, 0, 7); g.fill(); }
      }

      // **Enched.** The fourth channel, after motif, luminance and hue, and it
      // has to be told from the two lines already on this board: the lock's
      // solid gold outer edge and the assembled item's pulsing white one. So
      // it is drawn *inside* the component — a dashed inner edge and a bolt in
      // the corner — where neither of those goes.
      //
      // Greyed when it is switched off, because an ench toggled off does
      // nothing and a mark that looked the same either way would be a mark
      // that answered the wrong question.
      for (const p of s.placed) {
        if (!p.ench) continue;
        const live = p.ench.active;
        g.save();
        g.strokeStyle = live ? '#57b3c8' : '#5a5a68';
        g.lineWidth = 1.6;
        g.setLineDash([4, 3]);
        for (const [cx, cy] of p.cells) {
          const [px, py] = origin(cx, cy);
          g.strokeRect(px + 3.5, py + 3.5, CELL - 7, CELL - 7);
        }
        g.setLineDash([]);
        // The bolt, on the component's last cell so it does not sit under the
        // effect and trigger dots, which take the first.
        const [bx, by] = p.cells[p.cells.length - 1];
        const [px, py] = origin(bx, by);
        g.fillStyle = live ? '#57b3c8' : '#5a5a68';
        g.fillRect(px + 3, py + CELL - 8, 5, 5);
        g.restore();
      }

      // Item outlines. Assembled is brightness and weight — pulsing white,
      // thick — and not-assembled is near-black and thin.
      //
      // **Never gold against red.** That pair is the one distinction red-green
      // colour blindness is worst at, and the gold would collide with the
      // greaves hue besides. This board shipped that way for two milestones.
      for (const item of s.items) {
        const cells = item.cells;
        if (!cells.length) continue;
        if (item.assembled) {
          g.save();
          g.globalAlpha = pulse;
          this.edge(g, cells, origin, L.assembled, L.assembled_width, 1);
          g.restore();
          this.pips(g, origin, cells, item.rating);
        } else {
          this.edge(g, cells, origin, L.unassembled, L.unassembled_width, 1);
        }
      }

      // A ring round whatever the cursor is asking about, so the board and the
      // panel agree on which item is being read.
      if (this.pointed && !this.held) {
        const item = s.items.find((i) => i.pieces.join(',') === this.pointed);
        if (item && item.cells.length) {
          this.edge(g, item.cells, origin, '#69cdeb', 2, 3);
        }
      }

      // The drag footprint goes on last, over the pieces.
      //
      // It used to be painted onto the empty grid before anything was drawn on
      // it, so every occupied cell covered it — and the cells you most need an
      // answer about are the occupied ones, because those are where a drop
      // fails.
      if (this.held && this.legalSlot === name && this.hover?.slot === name) {
        const ok = this.legal.has(`${this.hover.x},${this.hover.y}`);
        const inside = this.heldCells()
          .map(([dx, dy]) => [this.hover.x + dx, this.hover.y + dy])
          .filter(([cx, cy]) => cx >= 0 && cy >= 0 && cx < 6 && cy < s.rows);
        g.save();
        g.globalAlpha = L.footprint_alpha;
        g.fillStyle = ok ? L.legal : L.illegal;
        for (const [cx, cy] of inside) {
          const [px, py] = origin(cx, cy);
          g.fillRect(px, py, CELL, CELL);
        }
        g.restore();
        this.edge(g, inside, origin, ok ? L.legal : L.illegal, 2.5, 1);
      }

      // A lock is solid gold, so "I decided this" reads differently from "this
      // happens to be assembled".
      const locked = s.placed.filter((p) => p.locked);
      if (locked.length) {
        const groups = new Map();
        for (const p of locked) {
          const item = s.items.find((i) => i.pieces.includes(p.id));
          const key = item ? item.pieces.join(',') : String(p.id);
          if (!groups.has(key)) groups.set(key, []);
          groups.get(key).push(...p.cells);
        }
        for (const cells of groups.values()) {
          this.edge(g, cells, origin, L.locked, L.locked_width, 1);
        }
      }

      // What the grid made is listed beside the board in HTML, where it can
      // wrap and be read. Canvas text at 11px under a grid could hold one
      // item and cut off the second.
    }

    this.drawBag(g, C);

    // The held component rides the cursor wearing what it would become.
    if (this.held && this.mouse) {
      // Grey and a diamond in the open; the grid's colour and mark once it is
      // over one that will take it. `over` is core's answer to "what would this
      // become there", asked in `askLegal`.
      const look = this.held.over ?? this.heldPiece()
        ?? { fill: '#888888', motif: 'shared', ink: '#ffffff', ink_alpha: 0.4 };
      const { px, py } = this.mouse;
      g.save();
      // Translucent, so whatever it is hovering over still reads through it.
      g.globalAlpha = 0.62;
      const shape = this.heldCells();
      // Offset down-right of the pointer so the cell being aimed at is never
      // completely under the thing being aimed with.
      const ox = px + 13, oy = py + 13;
      for (const [dx, dy] of shape) {
        this.cellFill(g, ox + dx * CELL, oy + dy * CELL, look);
      }
      this.edge(g, shape, (x, y) => [ox + x * CELL, oy + y * CELL], L.piece_edge, 2);
      g.restore();
      // The name goes under the board, not across it. It used to be drawn at
      // the cursor, which put it over whichever grid you were aiming at.
    }
  }

  /// Rarity pips: diamonds on the item's topmost-then-leftmost cell.
  ///
  /// Brightness climbs with the tier, so the pips read as a rank without the
  /// colours needing to be told apart — the count alone carries it.
  pips(g, origin, cells, rating) {
    const n = rating >= 170 ? 3 : rating >= 130 ? 2 : rating >= 90 ? 1 : 0;
    if (!n) return;
    const top = cells.reduce((a, b) => (b[1] < a[1] || (b[1] === a[1] && b[0] < a[0]) ? b : a));
    const [px, py] = origin(top[0], top[1]);
    const shade = n === 3 ? '#f0d890' : n === 2 ? '#c8c8dc' : '#9a9ab0';
    g.save();
    g.fillStyle = shade;
    for (let i = 0; i < n; i++) {
      const cx = px + 5 + i * 7, cy = py + 5, r = 2.6;
      g.beginPath();
      g.moveTo(cx, cy - r); g.lineTo(cx + r, cy); g.lineTo(cx, cy + r); g.lineTo(cx - r, cy);
      g.closePath(); g.fill();
    }
    g.restore();
  }

  /// One component at its own shape, in a box `box` pixels a side.
  ///
  /// The same three channels as a cell on a grid — fill, mark, traced edge —
  /// because it is the same component; only the scale changes.
  thumb(g, ox, oy, p, box) {
    const cells = p.cells ?? [[0, 0]];
    const xs = cells.map(([x]) => x), ys = cells.map(([, y]) => y);
    const x0 = Math.min(...xs), y0 = Math.min(...ys);
    const w = Math.max(...xs) - x0 + 1, h = Math.max(...ys) - y0 + 1;
    const cell = Math.max(3, Math.min(11, Math.floor(box / Math.max(w, h))));
    // Centred in its box, so a row of thumbnails has a common baseline rather
    // than every shape starting at a different height.
    const ax = ox + (box - w * cell) / 2, ay = oy + (box - h * cell) / 2;
    for (const [cx, cy] of cells) {
      const px = ax + (cx - x0) * cell, py = ay + (cy - y0) * cell;
      g.fillStyle = p.fill;
      g.fillRect(px, py, cell, cell);
      paintMotif(g, px, py, cell, p.motif, p.ink, p.ink_alpha);
    }
    this.edge(g, cells.map(([cx, cy]) => [cx - x0, cy - y0]),
              (cx, cy) => [ax + cx * cell, ay + cy * cell],
              this.look.piece_edge, Math.max(1, cell * 0.11), 0, null, cell);
  }

  drawBag(g, C) {
    const L = this.look;
    g.fillStyle = C.ink3;
    g.fillText(`THE BAG — ${this.state.bag.length} loose`, PAD, this.bagY - 10);
    const COL = BAG_COL, ROW = BAG_ROW;
    const perRow = Math.max(1, Math.floor((this.c.width - PAD * 2) / COL));
    this.state.bag.forEach((p, i) => {
      const x = PAD + (i % perRow) * COL;
      const y = this.bagY + Math.floor(i / perRow) * ROW;
      if (y + CELL > this.c.height - 4) return;
      const on = this.held?.id === p.id;
      if (on) {
        g.fillStyle = 'rgba(240,200,90,.16)';
        g.fillRect(x - 3, y - 3, COL - 8, CELL + 6);
      }
      // **Its actual shape, not a swatch.** A one-cell ring and a twelve-cell
      // base drew the same square before this, which hid the only thing about
      // a loose component that decides where it can go.
      this.thumb(g, x, y, p, BAG_THUMB);
      // A loose component can carry an ench too, and the bag is where a player
      // looks for the one they bolted something to.
      if (p.ench) {
        g.fillStyle = p.ench.active ? '#57b3c8' : '#5a5a68';
        g.fillRect(x, y + BAG_THUMB - 5, 5, 5);
      }
      g.fillStyle = on ? '#f0c85a' : C.ink;
      const tx = x + BAG_THUMB + 8;
      const room = COL - BAG_THUMB - 22;
      let label = p.name;
      while (g.measureText(label).width > room && label.length > 2) label = label.slice(0, -1);
      if (label !== p.name) label = `${label.slice(0, -1)}…`;
      g.fillText(label, tx, y + 12);
      g.fillStyle = C.ink3;
      g.fillText(p.kind ?? '', tx, y + 26);
    });
  }
}
