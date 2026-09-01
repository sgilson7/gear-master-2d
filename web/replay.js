// The fight, played back.
//
// The log is already decided — `fight_json` produced the whole thing before a
// single frame was drawn, because combat is a pure function and there is
// nothing to stream. So this is a playback head over a finished transcript, not
// a simulation, and the difference matters: scrubbing, pausing and replaying
// are free, and nothing this file does can change what happened.
//
// The cooldown bars are the one part that is reconstructed rather than read:
// the log records when an item activated, and a bar is the gap between one
// activation and the next filling up. That is a rendering of the log, not a
// second clock.

export class Replay {
  constructor(canvas) {
    this.c = canvas;
    this.log = null;
    this.t = 0;
    this.speed = 1;
    this.playing = false;
    this.onend = () => {};
  }

  load(log) {
    this.log = log;
    this.t = 0;
    this.playing = false;
    // Health over time, sampled from the log so the bars can be drawn at any
    // moment without re-deriving the fight.
    this.track = { player: [], enemy: [] };
    let ph = log.player.max_health;
    let eh = log.enemy?.max_health ?? 1;
    this.track.player.push([0, ph]);
    this.track.enemy.push([0, eh]);
    for (const e of log.entries) {
      if (e.kind === 'hit' || e.kind === 'burn') {
        if (e.side === 'player') { eh -= e.amount; this.track.enemy.push([e.at, Math.max(0, eh)]); }
        else { ph -= e.amount; this.track.player.push([e.at, Math.max(0, ph)]); }
      } else if (e.kind === 'regen') {
        if (e.side === 'player') { ph += e.amount; this.track.player.push([e.at, ph]); }
        else { eh += e.amount; this.track.enemy.push([e.at, eh]); }
      }
    }
    this.draw();
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

  at(track) {
    let v = track[0]?.[1] ?? 0;
    for (const [t, h] of track) { if (t <= this.t) v = h; else break; }
    return v;
  }

  draw() {
    if (!this.log) return;
    const g = this.c.getContext('2d');
    const cs = getComputedStyle(document.body);
    const V = (n, d) => (cs.getPropertyValue(n) || d).trim();
    const ink = V('--ink', '#171a19'), ink3 = V('--ink-3', '#6e736c');
    const line = V('--line', '#c9ccc2'), surface = V('--surface', '#f4f5ef');
    const brass = V('--brass', '#8a5c1c'), rust = V('--rust', '#8b4225');
    const verd = V('--verdigris', '#2c6759');
    const W = this.c.width;

    g.clearRect(0, 0, W, this.c.height);
    g.font = '12px ui-monospace, Menlo, monospace';

    // Two health bars.
    const bar = (y, label, now, max, colour) => {
      g.fillStyle = ink3;
      g.fillText(label, 0, y - 6);
      g.fillStyle = surface;
      g.fillRect(0, y, W, 18);
      g.strokeStyle = line;
      g.strokeRect(.5, y + .5, W - 1, 17);
      const frac = max > 0 ? Math.max(0, Math.min(1, now / max)) : 0;
      g.fillStyle = colour;
      g.fillRect(1, y + 1, (W - 2) * frac, 16);
      g.fillStyle = ink;
      g.textAlign = 'right';
      g.fillText(`${Math.max(0, Math.round(now))} / ${max}`, W - 6, y + 13);
      g.textAlign = 'left';
    };
    bar(20, 'you', this.at(this.track.player), this.log.player.max_health, verd);
    bar(62, this.log.enemy?.name ?? 'it', this.at(this.track.enemy),
        this.log.enemy?.max_health ?? 1, rust);

    // The clock.
    g.fillStyle = ink3;
    g.fillText(`${(this.t / 1000).toFixed(2)}s of ${(this.log.duration_ms / 1000).toFixed(2)}s`, 0, 100);

    // Cooldown bars, one per item, filling between activations.
    let y = 118;
    this.log.items.forEach((item, i) => {
      if (y > this.c.height - 16) return;
      const acts = this.log.entries
        .filter((e) => e.kind === 'activate' && e.side === 'player' && e.index === i)
        .map((e) => e.at);
      const last = acts.filter((t) => t <= this.t).pop() ?? 0;
      const cd = item.cooldown_ms || 1;
      const frac = Math.max(0, Math.min(1, (this.t - last) / cd));

      g.fillStyle = ink3;
      const label = item.name.length > 26 ? item.name.slice(0, 25) + '…' : item.name;
      g.fillText(label, 0, y + 10);
      const bx = 210, bw = W - bx - 46;
      g.fillStyle = surface;
      g.fillRect(bx, y, bw, 12);
      g.strokeStyle = line;
      g.strokeRect(bx + .5, y + .5, bw - 1, 11);
      g.fillStyle = frac >= 1 ? brass : ink3;
      g.fillRect(bx + 1, y + 1, (bw - 2) * frac, 10);
      if (item.hit_for > 0) {
        g.fillStyle = ink3;
        g.textAlign = 'right';
        g.fillText(String(item.hit_for), W, y + 10);
        g.textAlign = 'left';
      }
      y += 18;
    });
  }
}
