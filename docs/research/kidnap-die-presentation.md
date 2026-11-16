# Kidnap die presentation

Bounded research on 2026-11-16 against Cuba Libre commit `5c84f2526289fdcc0333ec44c7927d26fa182ea4`. This file is reference evidence for a future implementation specification and design round. It is not an accepted specification, not a design selection and not a claim that anything was built or rendered. No executable, Blender file, asset or native capture was produced.

Consulted constitution: `EVID-1`, `EVID-2`, `EVID-7` (core revision 2026-11-15.1). Foundations read: [04 art and assets](../foundation/04-art-and-assets.md) in full, [02 rules](../foundation/02-rules-and-verification.md) source-authority section, and [05 players](../foundation/05-players-and-proclamations.md) for the absence of built-in Non-players. Local inputs: [`art/atlas/TERROR.md`](../../art/atlas/TERROR.md), README Kidnap and development-seat sections, [rules baseline](rules-baseline-and-candidates.md) for source precedence.

**Owner disposition, 2026-11-16, recorded in [PR #57](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/57):**
- **Adopted, as [ADR 0001](../adr/0001-baked-die-tumbles-land-on-the-committed-face.md):** the baked route, with the conditions the ADR names:
  - the face rotation below the animated node (condition 1);
  - a flat tray outside the map relief (part of condition 4);
  - the clip and yaw chosen from saved data, never a fresh random draw (part of condition 6);
  - each clip ends flat with one face up, has a rest tail and a recorded landed face, and is hashed (part of condition 3).
- **Adopted:** the die takes the rolling faction's colour, red for this Kidnap. Government's die and Event rolls are deferred.
- **Adopted for the design round:** the three directions below, each landing in a flat tray outside the map relief. The layout round places the tray.
- **Deferred:** a reduced-motion setting.
- **Still proposals:** every other **[P]** item, until the implementation spec accepts it. This includes the rest of conditions 3, 4 and 6, and the review's addition that the clip data be independent of the die value. The art-tools checks in condition 3 are a repository standard, not a proposal.

Review on 2026-11-16, after those decisions and before merge:
- `EVID-3`, `EVID-4`, `EVID-6` and `REP-2` were read. Under `EVID-4` this study supports a plan, not a working die. `EVID-6` does not apply, because nothing is shared across repositories.
- [Art tools](../agents/art-tools.md) was read. Its Interpenetration and Negative-control checks apply as a repository standard and are listed in condition 3 below. Its Seam check is reported as n/a, because a tumble does not loop.

Labels used below: **[S]** a direct source statement; **[D]** my derived conclusion; **[P]** proposed local policy, not accepted until the owner accepts a specification.

## Decision and framing (EVID-2)

- **Accepted inputs (owner):** the Kidnap result is committed and saved first; then a 3D die tumbles and lands showing that committed face. A click or key skips to the landed result. A restart never rerolls and shows the landed result. The development seat must capture it deterministically. The die must be thematic.
- **Blocked decision:** what the implementation specification should require of the roll presentation (technique, constraints, pitfalls), and which three distinct thematic directions the later design round should explore.
- **Claim tested:** a Blender-modelled die with a few baked rigid-body tumbles exported as glTF, a static rotation chosen at runtime so the committed face ends on top, and playback seeked from the shared clock as the Terror bell does, can meet every accepted input without a runtime physics engine.
- **Sufficient evidence:** primary sources showing (1) that established tools land dice on precomputed results by a reusable mechanism, (2) that Blender can bake and export a single rigid-body tumble as a glTF object animation, (3) that Bevy 0.20 can seek, hold and compose such a clip under a runtime rotation, (4) any pitfall that would force a different route.
- **Appetite:** about one active hour. Spent: about one hour, including a delegated theme-source search.
- **Terminal outcomes:** adopt (with named conditions), reject for a named alternative, or unresolved with named missing evidence. **Result: adopt, with the conditions under "Recommendation".** One part of the proposal is corrected: the face-selecting rotation must sit *below* the animated node, not above it.

## Findings: which factions roll, and die colours

| Source | Statement **[S]** |
|---|---|
| R p.2, components | "3 6-sided dice—1 red, 1 yellow, 1 green." |
| P p.2, setup | Resources cylinders: Government blue, Syndicate green, 26July red, Directorio yellow; pieces "for 26July (red), Directorio (yellow), and Syndicate (green)". |
| R 3.3, p.6 | Insurgents choose Rally, March, Terror and "(for 26July and Directorio) Attack or (for Syndicate) Construct". |
| R 3.3.3, p.7 | Attack: the executing Faction "must be 26July or Directorio"; in each space "roll a die"; a roll of "1" adds a Guerrilla (Captured Goods). |
| R 4.3.2/4.4.2, p.8 | Ambush (26July, and Directorio "the same as 26July"): "automatically succeeds (do not roll…)" "as if a '1' had been rolled". |
| R 4.3.3, p.8 | Kidnap is a 26July Special Activity: "Roll a die and transfer to 26July…"; Cash Ransom replaces the roll. |
| Card 22 Raúl (P p.33; `card-22.png`) | Unshaded Capability: "26July may reroll each Attack or Kidnap." |
| Card 11 Batista Flees (P p.32; `card-11.png`) | "Select and remove a die roll of Troops." |
| Card 34 US Speaking Tour (P p.34; `card-34.png`) | Unshaded: "An Insurgent Faction adds a die roll in Resources." |
| Card 44 Rebel Air Force (P p.35; `card-44.png`) | Shaded: "Select 26July or DR and transfer 1 die roll of their Resources to Syndicate." |
| R 8.2, p.12 | Non-player random spaces: "Roll the green and yellow dice … green shows column, yellow row." |

P here is the third-printing playbook whose hash `dcaa5afd…015fe8` matches the baseline; the local second printing and the supplied card faces carry the same four die texts. **[D]** The brief's reading is partly wrong. Operations and Special Activities roll only for **26July (Attack, Kidnap) and Directorio (Attack)**; the Syndicate has no Attack (R 3.3) and Government no rolling Operation. But **Events make other rolls**: card 34 can give any Insurgent, including the Syndicate, a die roll; cards 11 and 44 are executed by whichever faction plays them, which can be Government. The rules never assign a coloured die to a faction; they name colours only for Non-player random-space rolls (R 8.2), and this game has no built-in Non-players (foundation 5). The box's three dice merely match the three Insurgent colours; there is no blue die. **[P]** "Die in the executing faction's colour" is therefore local policy. Event rolls need a separate decision on whose colour shows and what Government uses, because the physical game supplies no Government die. Raúl means the presentation must later support a second committed roll replacing the first. Attack rolls once per selected space, so several dice per Operation can occur.

## Findings: technique

**Established tools land dice on precomputed results.**

- **Dice So Nice!** 6.4.5, GitLab `riccisi/foundryvtt-dice-so-nice` commit `b365efc7433e` (2026-11-04).
  - **[S]** Its API animates "An evaluated Roll" (`docs/src/content/docs/api/roll.md:24`). Each mesh stores `forcedResult = dicedata.result` (`module/engine/ThrowEngine.js:241`).
  - **[S]** A cannon-es worker simulates the throw to rest and records per-step positions and quaternions (`ThrowEngine.js:479-484`; `module/web-workers/PhysicsWorker.js:1,507-671`). It reads the landed value as the face normal closest to world up (`PhysicsWorker.js:465-490`).
  - **[S]** `swapDiceFace` looks up a per-shape table of face-pair rotations (d6 at `module/engine/DiceModels.js:503-525`). `bakeSwapIntoQuaternionBuffer` right-multiplies that rotation into every recorded quaternion (`ThrowEngine.js:152-197,560-575`); the buffers are then replayed.
  - **[S]** By default the chat result card waits for the 3D roll to finish (`immediatelyDisplayChatMessages`, default `false`, `module/main.js:133-139`; hint at `module/lang/en.json:101`).
- **dice-box-threejs**, GitHub `3d-dice` commit `6945e00` (2022-10-26).
  - **[S]** Its README offers "Predetermined Outcomes" (`Box.roll("6d6@4,…")`) and credits Teall dice for "predeterministic rolling".
  - **[S]** It simulates silently (`simulateThrow`, `src/DiceBox.js:808-815`), resets the dice to the same start vectors, re-runs the physics visibly, and swaps the face *materials* of the landed and forced values (`swapDiceFace`, `DiceBox.js:511-596,1027-1050`).
- **dice-box** (BabylonJS), commit `eea7f1b`: **[S]** its README advertises no predetermined-result option. I did not audit its source.
- **Common pattern [D].** All of these resolve the motion fully before showing it, find the face that landed up, and remap so the forced value is on that face. Two remap mechanisms exist:
  - **Rotation in the die's own frame (DSN).** This preserves the die's real layout, opposite faces summing to 7.
  - **Relabelling two faces (dice-box-threejs).** If the two values are adjacent (for example 1 and 4), this breaks that layout. It also depends on the physics replaying identically.
- **Not found:** corrective nudging, meaning steering a live simulation toward a face. No examined primary source uses it, so it is not established here.
- **Mapping to the proposal [D].** The proposed route is DSN's mechanism with the simulation moved offline into Blender and a small fixed set of trajectories. It needs no runtime physics.

**Blender bake and glTF export.** Sources: the Blender 5.2 LTS manual for glTF, and the 5.2 Python API for the bake operator; the Terror bell used Blender 5.2.3 LTS. The Rigid Body pages were read only at the unpinned `/manual/en/latest/` address, the 5.3 manual on 2026-11-16. Recheck them against 5.2 LTS before relying on them.

- **[S]** `bpy.ops.rigidbody.bake_to_keyframes(frame_start=1, frame_end=250, step=1)` will "Bake rigid body transformations of selected objects to keyframes" (API, Rigidbody Operators).
- **[S]** The glTF exporter ignores the physics itself: "Animation of other properties, like physics, lights, or materials, will be ignored." It exports object transform animation. Its options include:
  - "Sampling Rate: How often to evaluate animated values (in frames)";
  - "Set all glTF Animation starting at 0";
  - "Optimize Animation Size … removing duplicate keyframes when all identical";
  - "Bake All Objects Animations: Useful when some objects are constrained…";
  - "Use Current Frame as Object Rest Transformations … When off, frame 0 is used" (manual, glTF 2.0, Animations/Export and Animation options).
- **[S]** Rigid Body World exposes "Substeps Per Frame" and "Solver Iterations" (higher is more accurate). Its Cache can bake a frame range; an unsaved file's cache lives in memory (manual, Rigid Body World). The collision margin is "embedded" for Box shapes and is "best results when nonzero" (manual, Rigid Body Collisions).
- **Pitfalls [D]:**
  - Bake explicitly to keyframes; do not rely on export-time sampling of a physics cache.
  - Use one key per frame at a 60 fps scene rate. This matches the seat's 1/60 s step (`unfolded-tabletop` `063e811`, `src/seat.rs:125`; the value is the same at Cuba's pinned `29de68f`).
  - Start the clip at 0.
  - Use a Box collider rather than a convex hull of a bevelled mesh, so the rest pose is flat and every cube symmetry is exact.
  - Make sure the clip has channels only on the tumble node (see Bevy below).
  - **Scale:** baking only records kinematics. A trajectory simulated at *k* times true size and then shrunk falls as if gravity were divided by *k*. Simulate at true size with enough substeps, or scale gravity together with size. I found no Blender manual statement on minimum stable size; this needs a probe.

**Bevy 0.20 seek, hold and composition** (`bevy_animation`/`bevy_gltf` 0.20.0, `bevy_math` 0.20.0, `glam` 0.33.12, as locked in `Cargo.lock`).

- **Seek and hold [S]:**
  - `set_seek_time` sets the seek time without triggering events (`bevy_animation/src/lib.rs:713-717`).
  - A paused animation is not ticked (`lib.rs:1071-1073`) but is still sampled at its `seek_time` (`lib.rs:1269-1294`).
  - Sampling uses `sample_clamped` (`animation_curves.rs:381`), so a seek at or beyond `duration()` holds the last key.
  - The Terror bell already does `play(node).pause()` and then a per-frame `set_seek_time` in `PostUpdate` before `AnimationSystems` (`src/tabletop.rs:601-632,313`).
- **Composition [S]:**
  - glTF rotation channels become curves on that node's local `Transform::rotation` (`bevy_gltf/src/loader/mod.rs:400-423`).
  - Curves apply only to targets present in the clip (`lib.rs:1264-1266`).
  - Mesh primitives spawn as *child* entities of the node, carrying a coordinate-conversion transform (`loader/mod.rs:1631-1686`).
- **Interpolation [S]:** linear quaternion keys interpolate with `slerp` (`bevy_math/src/common_traits.rs:494-498`). glam's `slerp` flips to "the short way" (`glam/src/f32/sse2/quat.rs:808-818`).
- **Correction to the proposal [D].** World pose is `P · A(t) · R`:
  - `P` is a static placement. It may include a yaw about the vertical and scale.
  - `A(t)` is the animated tumble node.
  - `R` is the face remap, one of the cube's 24 rotations. It must sit on a **child** of the animated node, with the die centred on that node's origin.

  A remap placed on the parent (`R · A(t)`) rotates the whole trajectory, tilting the floor it bounced on, unless `R` is a pure yaw. Author an explicit named child (for example "Die body" under an animated "Die tumble" empty) rather than editing the generated primitive entity's transform. Keys must sit well under 180° apart, or the short-way `slerp` makes a fast spin reverse.
- **Free choice of yaw [D].** Four of the 24 rotations put any given face on top. The runtime can choose the one whose face art reads upright toward the camera.

**Presentation pitfalls.**

- **Cocked die [D].** DSN reads the "closest" face without rejecting a tilted rest pose (`PhysicsWorker.js:465-490`). A baked clip can be checked once, offline. Admit only clips whose last key has one face normal within a small tolerance of up, rest on the floor, do not touch a wall, and end in an identical-key rest tail.
- **Map relief versus tray [D].** A baked path is valid only against the geometry it was simulated on. The Atlas desk's regions and pieces are solid 3D (foundation 4), so a die rolled over the map would interpenetrate or float, and it would cover pieces and counts. The shared camera can also orbit the die out of view. A flat tray, framed independently of the map camera, avoids all three problems. Its placement is a layout decision (foundation 4, Layout authority).
- **Duration and skip [S].**
  - WCAG 2.2.2 requires pause/stop/hide only for automatic motion lasting more than five seconds.
  - WCAG 2.3.3 (AAA): "Motion animation triggered by interaction can be disabled … unless the animation is essential". The understanding page suggests honouring the operating system's reduce-motion feature.
  - The Game Accessibility Guidelines list "Avoid any sudden unexpected movement or events" (Advanced) and "Ensure no essential information is conveyed by a fixed colour alone" (Basic).
- **Applying them [D].** The landed die plus text conveys the result, so the tumble is non-essential. A reduced-motion setting could show the landed die directly. No such setting exists in the repository yet, and the owner deferred one on 2026-11-16; a click or key skip serves this spec.
- **Unambiguous, fair result.**
  - **[S]** The core commits the die, proceeds and RNG state before any presentation (`src/game.rs:786-800`, `Kidnap` at `163-169`). Today the panel shows "Kidnap rolled … Saved." immediately (`src/tabletop.rs:810-816,1473-1476`).
  - **[D]** The animation therefore cannot alter the result. Fairness as players perceive it rests on two conditions: every clip can show every face, and the choice of clip does not correlate with the value. I found no primary study of how players perceive precomputed rolls.

## Findings: theme

The theme sources are mostly secondary: Wikipedia, Flags of the World, EcuRed, CiberCuba and collector sites. Here **[S]** marks what such a source says. It is not a verified primary fact, and it is weaker than the rules, code and manuals cited above.

- **Precision casino dice.**
  - **[S]** Wikipedia (Dice, Construction › Manufacturing) describes them:
    - "Precision dice are generally made from bars of extruded cellulose acetate";
    - "Precision casino dice are transparent or translucent";
    - "edges usually are left sharp, also called square or razor edge";
    - pips are drilled and "filled with opaque paint or epoxy";
    - "each die carries a serial number and the casino's logo or name".

    That is current practice; the article gives no date.
  - **[S, delegated read, unverified by me]** A Kentucky Historical Society record, no. 2001.5.2, dated "ca. 1950", describes casino dice of "transparent red composite material" with white pips and a club name in gold. My recheck of the record got HTTP 403.
  - **WEAK [S]** Collector sites put the change from nitrate to acetate "around the 1950s" and wide serial numbering in the 1970s, and they conflict with each other.
  - **Not found:** any museum or archive die from a 1950s Havana casino. The Havana link is an inference from period US practice.
  - **Earlier materials [S].** The Science Museum Group holds cellulose-nitrate imitation-ivory dice of 1900–1930. MFA CAMEO describes Catalin as cast phenolic in bright or marbled colours, early 1930s to mid-1940s. It does not mention dice.
- **Die conventions [S].** Opposite faces "add up to seven". Western dice are "normally right-handed". Asian dice colour the one, and often the four, red (Wikipedia, Dice › Arrangement).
- **La Charada.**
  - **[S]** EcuRed (Charada china) gives 1 Caballo, 2 Mariposa, 3 Marinero, 4 "Gato Boca", 5 Monja, 6 Jicotea. It says the game spread with Chinese coolies arriving from 1847, and that its numbers are bet in the bolita lottery. CiberCuba (2020) gives the same list, with 4 as cat or mouth.
  - **[S]** Historian Samuel Farber (Havana Times, 2012) recalls 1952 Havana. He confirms "1 meant horse, 2 butterfly, 3 sailor". He calls bolita "an informal and illegal lottery" bet on the first prize of the Saturday national lottery.
  - **Variants.** Figure 4 varies: cat, cat/mouth, "gato boca". Weak lists add alternates for 1–6. No scholarly source confirmed 4–6.
- **Bolita [S].**
  - Wikipedia (Bolita) says it was brought to Tampa in the 1880s and that the Trafficantes "figured prominently".
  - The Mob Museum (Deitche, 2019) ties Trafficante Sr. to "the game of bolita".
  - The game's own playbook says the Trafficante family "made its name in Tampa running the bolita, a lottery popular with Cuban exiles" (card 48 note, P p.35).
- **26th of July colours [S].**
  - Flags of the World (Cuba – Political Flags, "26 July Movement"; contributor reports) describes a flag with "the red above the black". Variants carry "M-26-7" in white letters or a white star. Forces wore "red and black armbands with the July 26 emblem".
  - Wikipedia (Flag of Cuba) says only "equally divided in red and black".
  - OnCuba (2016) dates the home-sewn red-and-black "M-26-7" brazalete to the Santiago rising of 30 November 1956.
  - **Not found:** a documented meaning for the colours. Red on top rests only on contributor reports.
- **The game's own component [S].** Three coloured six-sided dice, as listed above. No box dice art was inspected, and none may be copied (foundation 4).

### Three directions for the design round [P]

Each direction is a starting point for one VectorCraft face sheet plus a Blender die, under foundation 4's three-alternative rule. All three share these features:

- the same mechanism, recovered from evidence;
- a faction-parameterised material;
- the faction named in text beside the die.

| | **1. Box die, faction-cast** | **2. Havana casino precision die** | **3. Charada die** |
|---|---|---|---|
| Source basis | R p.2 dice and P p.2 faction colours; the physical game's own look | Precision casino dice (Wikipedia; KHS ca. 1950 red imprinted club die, a delegated read I could not verify); 26July Kidnap closes Casinos and takes Syndicate Cash (R 4.3.3) | EcuRed/Farber charada 1–6; bolita in 1950s Cuba and the playbook's Trafficante note |
| Look | Opaque cast die, softened edges, recessed contrasting pips; faction emblem at most as a small accent | Translucent cellulose-look body, razor edges, flush white pips, an invented gold house monogram and serial on one face | Opaque die; each face has a dominant pip group or numeral plus a small period-print figure (horse, butterfly, sailor, cat, nun, jicotea) |
| Legibility | Pips only; strongest at small size | Hidden-face pips show through the body; needs high-opacity tint or muted back pips. Bevy transparency/transmission cost and sorting to test | Figure must stay secondary; choose the upright yaw (above); figure detail may vanish at 1920×1080 |
| Faction follow | Body colour red, yellow or green; Government needs an invented colour (no box die) | Tinted red, amber or green translucent; check yellow and green contrast | Body or ink colour per faction; figures unchanged |
| Cautions | Low atmosphere | Invented names only: no real casino (for example Tropicana, Riviera, Capri, Sans Souci, Nacional) or dice-maker marks; Havana provenance unproven | Bolita is drawn, not rolled, a thematic stretch; figure 4 varies; "monja" (nun) and gambling tone beside a kidnapping need owner judgment; original drawings only |

**[P]** A red-over-black split with white lettering (brazalete) fits only 26July. Treat it as an accent within direction 1 or 3, not as a fourth direction, until other factions' period colours are researched. Proposed legibility floor, for later visual checking: the landed top face at least about 70 px across at 1920×1080, about 13 px pips, and about 3.2× that on the 6144×3456 display. The seat reaches only 4096×2160 today (README; #50).

## Recommendation and its limits

**Adopt the baked-trajectory route [D, P]** under these conditions:

1. **Body-local remap.** An animated tumble node carries the clip. The face-bearing mesh is a static child whose rotation (one of 24) puts the committed face on top. Only yaw and placement go above the animated node.
2. **Symmetric die.** The collider and silhouette are cube-symmetric about the node origin, using a Box collider. Asymmetric art (monogram, serial, figures) moves with the remap.
3. **Clip admission,** like Terror's checks:
   - one face up within tolerance and resting flat, with no wall contact;
   - a rest tail;
   - a bound on rotation between keys;
   - start at 0;
   - channels only on the tumble node;
   - each clip's landed face recorded;
   - a hash file so a changed asset refuses at launch;
   - the [art-tools animated-mesh checks](../agents/art-tools.md#animated-mesh-checks) Interpenetration (on every frame the die's world-space BVH overlap with the tray is empty) and Negative control (a broken clip is flagged). Report Seam as n/a, because a tumble does not loop.
4. **Flat tray, camera-independent framing.** It never rolls across map relief, and it never intercepts region picks. Placement comes from the layout round.
5. **Shared-clock playback.**
   - `seek = min(now − start, duration)`, where `start` is set when the running app applies the commit.
   - Loading a save, or a skip by click or key, sets the landed pose and never replays.
   - The seat reports "die landing" as unsettled until landed. A plain `settle` then reaches the landed capture (default limit 1800 steps, `unfolded-tabletop` `src/seat.rs:127,978-1000`), and bounded steps capture mid-tumble frames.
6. **Deterministic choice without RNG.** Choose the clip and yaw by a pure function of saved data that is independent of the die value, for example a hash of the match revision at commit. Do not use `Kidnap::random_before`: it is the seed that produced the die (`src/game.rs:787-788` rolls from it; `:795` stores it), so the clip would correlate with the value. Never draw from the shared RNG, which would shift later results, or from OS entropy, which would break reproducible captures. A test shows every clip can show all six faces.
7. **Text reveal.** The result text appears at landing or skip, naming the faction. A reduced-motion setting that shows the landed die directly was deferred by the owner as a later candidate.
8. **Future rolls.** Raúl rerolls, one-die-per-space Attacks and Event rolls remain later work. Separate trays or sequential rolls avoid baked paths colliding.

**Rejected alternatives [D]:**

- **Runtime physics engine.** None exists here or in the siblings. It would be a new dependency and would still need the remap, with cross-machine determinism unproven.
- **Rust-scripted procedural tumble.** Foundation 4 says runtime movement "do[es] not authorize an independently invented animation or graphics system".
- **EffectCraft 2D flipbook.** Six authored variants per die, and it is not the 3D asset the owner accepted.

**Not established:**

- Bake stability at true die scale (needs a Blender probe).
- How many clips avoid visible repetition.
- Bevy transparency quality for direction 2.
- Legibility at the owner's display.
- Owner acceptance of any look.

## Sources

- Local, at `5c84f25`:
  - `src/game.rs`, `src/tabletop.rs`, `src/tabletop/seat.rs`;
  - `art/atlas/TERROR.md`, README;
  - foundations 02/04/05;
  - `docs/research/rules-baseline-and-candidates.md`.
- Sibling: `unfolded-tabletop` `063e81170b20986c9dcee79dabaf364f8911ca62`, `src/seat.rs`.
- Rules:
  - R, March 2019 Living Rules, SHA-256 `8152cb00…c606792` (local), pp.2, 6–8, 12, 14–15.
  - P, [third-printing playbook](https://gmtwebsiteassets.s3-us-west-2.amazonaws.com/cuba/CL-PLAYBOOK-2018.pdf), SHA-256 `dcaa5afd6f01c0c9ed4746c18199c1b09058823d67b627384fd3184ed4015fe8`, pp.2, 32–35.
  - Local second-printing playbook.
  - Supplied faces `card-11/22/34/44.png` (external collection; not committed).
- Dice software:
  - [Dice So Nice!](https://gitlab.com/riccisi/foundryvtt-dice-so-nice) `b365efc7433eb2db2ccbd91da2029ac6c8b78ed5`.
  - [dice-box-threejs](https://github.com/3d-dice/dice-box-threejs) `6945e0068eae27f22acd26debdb70f6ef2fd6063`.
  - [dice-box](https://github.com/3d-dice/dice-box) `eea7f1b042338b1083d4d696585b847b84f84a55`.
- Rust crates: local crates.io sources for `bevy_animation` 0.20.0, `bevy_gltf` 0.20.0, `bevy_math` 0.20.0, `glam` 0.33.12.
- Blender:
  - Manual 5.2 LTS, [glTF 2.0](https://docs.blender.org/manual/en/5.2/addons/import_export/scene_gltf2.html).
  - [Rigid Body World](https://docs.blender.org/manual/en/latest/physics/rigid_body/world.html), [Rigid Body Collisions](https://docs.blender.org/manual/en/latest/physics/rigid_body/properties/collisions.html), [Rigid Body introduction](https://docs.blender.org/manual/en/latest/physics/rigid_body/introduction.html) (unpinned `latest`, the 5.3 manual when read on 2026-11-16; not checked against 5.2 LTS).
  - Python API 5.2, [Rigidbody Operators](https://docs.blender.org/api/5.2/bpy.ops.rigidbody.html).
- Accessibility:
  - W3C, [Understanding 2.3.3 Animation from Interactions](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html); WCAG 2.2 SC 2.2.2 Pause, Stop, Hide.
  - [Game Accessibility Guidelines, full list](https://gameaccessibilityguidelines.com/full-list/).
- Theme:
  - [Wikipedia: Dice](https://en.wikipedia.org/wiki/Dice) (Construction).
  - [Kentucky Historical Society 2001.5.2](https://kyhistory.pastperfectonline.com/webobject/8ED9AF1E-C9E0-4CFE-8051-007439316691).
  - [Science Museum Group co10549](https://collection.sciencemuseum.org.uk/objects/co10549/one-of-a-pair-of-dice-made-of-ivory-cellulose-nitr-pair-of-dice).
  - [MFA CAMEO: Catalin](https://cameo.mfa.org/wiki/Catalin).
  - [EcuRed: Charada china](https://www.ecured.cu/Charada_china) (Cuban state wiki).
  - [CiberCuba 208515](https://en.cibercuba.com/node/208515).
  - [Farber, Havana Times 2012](https://havanatimes.org/features/cuba-the-death-of-the-king-of-england-and-marianao/).
  - [Wikipedia: Bolita](https://en.wikipedia.org/wiki/Bolita).
  - [Mob Museum, Deitche 2019](https://themobmuseum.org/?p=23181).
  - [Flags of the World: Cuba political flags](https://www.crwflags.com/fotw/flags/cu}.html).
  - [Wikipedia: Flag of Cuba](https://en.wikipedia.org/wiki/Flag_of_Cuba).
  - [OnCuba 2016, brazaletes](https://oncubanews.com/cuba/sociedad-cuba/historia/brazaletes-rojos-y-negros).
  - Weak collector sources were consulted but are not relied on.
