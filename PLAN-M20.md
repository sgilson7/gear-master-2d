# PLAN-M20 — the bench, a specialization, and the Cairnworks

**The frame.** `PLAN.md` wins where it and this disagree, and a divergence from
this goes in `CLAUDE.md`'s table with its reason, in the commit that makes it.

## What was asked for

> add a dungeon to the wextreen reach, and a quest you can pick up in the
> kettleworks that asks you to complete it. it should be 3 floors of bosses that
> require unique gear slots to defeat. the first boss has 100% resists but 0%
> curse resist, so you have to sear him. The second boss has 100% all resists
> except physical, the third had 100% except magical, and the final boss
> requires you to have piercing to defeat. make sure the previous bosses can be
> somehow refought and farmed for gear that helpes with the next boss in the
> chain. add a potion making stand in cities, which lets you brew together
> ingredients that enemies drop to make bonuses that can be consumed before
> battles for a temporary benefit during that battle. each enemy should drop one
> ingredient, and pairs of ingredients provide a single benefit when brewed
> together. ingredients do not go in the inventory but instead into an
> ingredient inventory that can only be accessed in towns. the potion brewing
> window is a single slot with a bizzarre shape, that you will later be able to
> somehow increase the size of. ingredients dropped by enemies should use the
> current gear designs for layout in physical space. for beating the new
> wextreen reach dungeon, you receive an improvement to your potions that allows
> you to add in a third ingredient to a brew that acts like an ink in books ie
> increases potency. also add a new type of class called a specialization, and
> you can only have one of them, and it does not interact / form expert classes.
> the first specialization is a new tree related to potion brewing.

## Decisions taken before anything is built

1. **"100%" is 95**, because `stats::LANE_CAP` is 95 and has been since M16 for
   a reason this block must not undo: *a lane you can commit to is never one you
   can be shut out of*, and a boss literally immune in three lanes is a wall
   with a sentence on it. Ninety-five means the wrong lane does a twentieth of
   its damage, which against these healths is a fight you lose at the buzzer —
   which is what "you have to use the other lane" means in a game with a clock.
2. **Four floors, three of them "floors of bosses".** The ask names four
   creatures — curse, physical, magic, pierce. The Sump's own shape is the
   precedent: *four floors, three puzzles and the Ninth Surveyor*.
3. **A boss that can be refought is a boss that also walks its floor.** Eight of
   the nine boss creatures in this game already stand in a region pool; this is
   that, on purpose, so nothing new is needed to farm one. What it drops is
   `data/drops.json`, which is the existing per-mille table.
4. **An ingredient is not a component.** No `PieceKind`, no `CATALOG` entry, no
   grid of the five — the save fingerprint must not move, because there is a
   player mid-run. It has a **shape**, because the ask says so, and the shape
   lives in `brew.rs` beside the ingredient.
5. **Eight ingredients and all twenty-eight pairs.** `C(8,2)` is 28, which is a
   table a person can author and a lint can prove complete — the same argument
   `every_pair_of_offered_classes_reaches_an_expert` makes for the ten experts.
   Every creature drops one of the eight; *which* is derived from the creature's
   art family, so a new creature cannot arrive without one.
6. **A specialization is a third kind of class and holds one slot.**
   `ClassKind::Specialization`, outside `class::OFFERED`, outside
   `expert::EXPERTS`, and `Character::classes` yields it last. Nothing pairs
   with it, which is the ask and is also what keeps `C(7,2)` at twenty-one.

## Milestones

| # | milestone | deliverable |
|---|---|---|
| M20.0 | the errands, and a refusal that names the key | ten errands, `Goal::Clear`, `unlock.rs`, `spoke_on_arrival` |
| M20.1 | a second bag | `brew.rs`: eight ingredients with shapes, one per creature, `WorldState::larder`, dropped on every win |
| M20.2 | the retort, and twenty-eight pairs | a bizarre-shaped slot, `data/brews.json`, `Boon` at the bell |
| M20.3 | the bench in a town | the stand, the larder list, the retort on screen, drink-before-fight |
| M20.4 | a specialization | `ClassKind`, one slot, no experts, the Apothecary tree |
| M20.5 | the Cairnworks | four floors under the Reach, four bosses, four lanes, pools that refight and drop |
| M20.6 | the errand and the third slot | Kettleworks asks; clearing it grows the retort and adds the potency slot |
| M20.7 | a place says its own name | the area's name over the middle of the screen on arrival, then gone |
| M20.8 | the errand log is a tree | chains drawn by depth, the way the skill tree is — a root on the top row and every rung under the one it follows |
| M20.9 | the browser gate, the notebook, the deploy | negative-tested checks, `SECOND-ORDER-M20.md`, live |
