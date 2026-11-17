# Kidnap die

The owner selected the **box die** in **tray placement C** (inside the operation panel) in #68: [selection record](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089). The studies, the probe and the unselected dice and placements remain in [`../proposals/issue-67/`](../proposals/issue-67/README.md); they do not ship.

When 26July's Kidnap roll commits, the die tumbles in its tray and lands on the committed face ([ADR 0001](../../docs/adr/0001-baked-die-tumbles-land-on-the-committed-face.md)). The roll is saved before anything moves.

## Files

- **Runtime exports**, byte-identical copies of the selected proposal exports:
  - `atlas-kidnap-die.glb` (from `a-box-die.glb`): one root node, **Die tumble**, which carries all four clips (`Tumble A` to `Tumble D`), and its unanimated child **Die body**, the textured 16 mm die with its embedded face atlas;
  - `atlas-kidnap-tray.glb` (from `kidnap-tray.glb`): the static tray, a 96 × 64 mm linen floor whose top is at 0 and four low walnut walls, centred on the tumble's origin.
- **Record:** `kidnap-die.json`, which the app reads:
  - for each clip, its duration, its landing (the first key of its identical-key rest tail), its landed face with `Die body` unrotated, and the tumble node's last rotation;
  - for each face, its outward normal and the up direction of its art, in the die's glTF frame.
- **Editable sources**, unchanged in the proposal folder:
  - `../proposals/issue-67/kidnap-dice.blend`, collections `Box_die` and `Kidnap_Tray`, with the tumbles as NLA tracks;
  - `../proposals/issue-67/kidnap-die-faces.vectorcraft`, artboard 1 (the box die's atlas);
  - `../proposals/issue-67/kidnap-tray-placements.figcraft`, frame **C** (node 37): the tray viewport, panel title and result text.
- **Layout record:** `atlas-kidnap-tray.layout.json` holds frame C's bounds. The native regression compares the running app with it.
- **Admission:** `kidnap.sha256` lists both GLBs and the record, so a missing or changed die, tray or record refuses launch, like the Terror bell. It does not change what a save records.

## Runtime

- **The tumble:** clip and half turn come from a hash of the revision the roll created, which the save records with the roll (`Kidnap.revision`). They never come from the shared random generator, OS entropy or the die. The tray is symmetric under a half turn, so each clip's wall clearance holds either way.
- **The face:** a static rotation of Die body, one of the cube's 24, puts the committed face up. Of the four that show it, the runtime takes the one whose art reads most nearly upright to the tray camera (the free yaw, #67), so the star 1 and the pips sit square. Only the tray's place and the half turn sit above the animated node.
- **Playback:** the clip seeks `min(0.5 × (now − start), duration)` from the shared clock; `start` is when the running app applied the commit.
  - **Speed:** the owner asked to slow the roll down (#68). Half speed is exactly the trajectory of a re-bake under a quarter of real gravity, or the real fall of a die four times the size. Friction and restitution carry no time unit, so the bounces and the settle keep their proportions. The motion of 0.32–0.38 s becomes 0.63–0.77 s. Below about half speed a bounce's hang time starts to read as slow motion. The owner judges it in #16.
  - **Landing** is the first rest-tail key, 0.32–0.38 s of clip time, so 0.63–0.77 s after the commit. The rest of the tail is the landed pose held.
  - **Skip:** a click on the tray, Space or Enter lands it at once. Loading a save shows the landed die and never replays it.
- **On landing or skip:** the result appears under the tray: "[26] rolls N" in Lato Bold, then "Kidnap: [Government] → [26], P Resources". The Resources ledger keeps the balances before the roll while the die is in the air, then shows the transfer. The four standings stay visible throughout.
- **The panel:** while the die is shown, the operation panel holds the frame-C title, the tray and the result. The choices return when the player presses **Continue Terror** (or Enter) after landing. Continue is presentation only: it saves nothing, and a restart before the next decision shows the landed die again. Its place, at the foot of the interim panel stack, is part of that stack's WORK-2 bootstrap (`../../src/tabletop/operation.rs`), not a FigCraft design.
- **Camera and light:** the tray has its own camera, framed as #67's Blender previews (80° elevation, a 50 mm lens on a 36 mm sensor across the viewport, the inner floor fitted to 2–98% of the frame), and #67's preview sun. Both are on the tray's own render layer, drawn after the UI over the panel, so the die never shows on the map or takes a region pick.
- **Limits:** the shared count anchors follow only a single `Camera3d`. While the die is shown the map camera is held still, so the counts stay correct; a window resize during that time leaves them where they were until Continue.

## Verification

- **Clip admission** (`tests/kidnap_die.rs`, Rust, read from the GLB independently of Bevy): each clip starts at 0, has LINEAR translation and rotation channels only on Die tumble, turns at most 71.2° between keys (under 90°), ends flat within 0.1° with exactly one face up and its centre at 8 mm, holds a rest tail of at least 0.5 s, and keeps its corners at least 2 mm from every wall. Its duration, landing, landed face and final rotation match the record. The four clips land on four different faces. The die is cube-symmetric about its node origin, so the face rotation changes no collision.
- **Faces:** each face's flat vertices lie in its atlas cell, and its art's up direction is where the texture's v decreases, as the record says.
- **All six faces:** every clip, in both placements, presents every face upright within 45° of the tray camera's screen up. The same rotation on the parent fails (negative control).
- **Interpenetration (art-tools)**, run in Blender 5.2.3 on the two production GLBs, on every key of every clip in both placements, against every tray part, with the die shrunk 0.05 mm per face for coplanar resting contact: none. **Negative control:** the floor raised 4 mm is flagged on 52–55 keys of every clip. **Seam:** n/a, since a tumble does not loop. The script and its log are kept out of Git under `local/verification/issue-74/blender/`.
- **Native:** `native_kidnap_die_lands_on_each_face_and_restarts_landed` lands the die on each of the six faces and checks the layout, mid-tumble frames, the ledger and a restart during the tumble (README).
