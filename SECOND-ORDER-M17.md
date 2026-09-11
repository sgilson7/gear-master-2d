# SECOND-ORDER-M17.md — the notebook

*The table's own notebook. Same convention as `SECOND-ORDER-M16.md`.*

---

## Rows

| # | row | kind | status |
|---|---|---|---|
| 1 | **`POWER_UNIT` is 30 and `PLAN-M17.md` §3 says 22.** The plan chose every constant on paper and says the recon has to check them; what the sweep found is a property the plan does not name — **distance has to be monotone in power**, because a cue where pulling back harder lands you *nearer* is a cue nobody can aim. At 22 it is not, and at every restitution below 80 it is not: a fast ball spends its extra speed on ricochets, and a bounce that costs too much makes a strong shot die at the wall it hit. **80 and 30 is the only pair in the sweep with no step backwards.** | divergence | done (M17.0) |
| 2 | **Seven hundred and twenty shots from the Treyway's start land on 164 of its 176 walkable tiles.** §10.3 asks whether 16×16 is too small to be an interesting table and says to report rather than fix; that is the number. **A table where one shot reaches ninety-three percent of the map is closer to a menu than to a course**, and it is the human's call. | finding | done (M17.0) |
| 3 | **The mean-distance measurement saturates by construction on a small map** — the maximum possible `|dx|+|dy|` is about fifteen — so *mean distance* flattens from power three whatever the constants are. The number that means something for a cue is **how many distinct tiles the shot can reach**, which is also what `aim_at` will need. Worth knowing before anybody retunes the table on the first number. | finding | done (M17.0) |
| 4 | `the_physics_has_no_floating_point_in_it` is a lint over the **source** rather than the behaviour, and it has to be: an `f32` that rounds differently in three engines produces a flight that is *nearly* right, which a hash cannot tell you about until somebody in another browser reports it. | finding | done (M17.0) |
| 5 | The angle table is a `const fn` over a nineteen-entry quarter turn rather than seventy-two typed pairs. `the_angle_table_is_a_circle` checks every entry is a unit vector and that opposite steps cancel — *a table typed by hand is a table that can be wrong in one entry*, and one wrong entry is one angle in seventy-two nobody would find. | finding | done (M17.0) |
