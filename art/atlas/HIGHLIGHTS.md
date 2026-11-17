# Map highlights and the space popover

The owner selected **HF-A, Survey ink** in #58's combined design round ([#68 selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)). The proposal, its rationale and the unselected alternatives remain in [`../proposals/issue-66/`](../proposals/issue-66/README.md). #72 ships the space states and the space popover; #73 adds removal mode's piece markers and piece popover from the same proposal.

## Space states

While 26July executes Terror, `cuba_libre_unfolded::targets::space_targets` reads every space's state from the rules authority's own choices:
- **Selectable:** a supported choice acts there now. The graphic is a 5-unit `action/primary` rule just inside the border, with a hairline ink rule inside it.
- **Blocked with a remedy:** nothing there is supported, but a choice there names an in-task remedy. The graphic is the same red rule, dashed.
- **Not involved:** neither. The graphic is a 62% `desk` veil.

Outside these states (the action choice, either ending, an opened Kidnap, a Resource transfer under way and removal mode), no space is a target and a region click opens no space popover. Removal mode has its own states, below.

| Runtime export | Source artboard | State | Spaces | Atlas rectangle (x, y, w, h) | Height |
|---|---|---|---|---|---:|
| `atlas-highlight-selectable-santiago.png` | `A-sel-santiago` | selectable | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-blocked-santiago.png` | `A-blk-santiago` | blocked | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-selectable-oriente.png` | `A-sel-oriente` | selectable | Oriente | 880, 550, 370, 186 | 0.142 |
| `atlas-highlight-blocked-oriente.png` | `A-blk-oriente` | blocked | Oriente | 880, 550, 370, 186 | 0.142 |
| `atlas-highlight-veil-sierra-maestra.png` | `A-dim-sierra` | not involved | Sierra Maestra | 914, 648, 416, 194 | 0.142 |
| `atlas-highlight-veil-provinces.png` | `A-dim-provinces` | not involved | Pinar del Río, La Habana, Matanzas, Las Villas, Camagüey Province | 50, 317, 968, 395 | 0.142 |
| `atlas-highlight-veil-cities.png` | `A-dim-cities` | not involved | Havana, Camagüey City | 220, 150, 695, 660 | 0.242 |
| `atlas-highlight-veil-ecs.png` | `A-dim-ecs` | not involved | the three economic centers | 204, 365, 1004, 373 | 0.282 |
| `atlas-highlight-veil-santiago.png` | `A-dim-santiago` | not involved | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-veil-oriente.png` | `A-dim-oriente` | not involved | Oriente | 880, 550, 370, 186 | 0.142 |

- **Source:** [`../proposals/issue-66/map-highlights.vectorcraft`](../proposals/issue-66/map-highlights.vectorcraft) (VectorCraft CLI 0.24.1), the HF-A artboards named above. Each export is the proposal's `graphics/map-{artboard}.png`, byte for byte, at 4 px per map unit.
  - #73 added `A-dim-santiago` and `A-dim-oriente` the way the proposal made the other veils: a copy of the region's atlas profile (objects 47 and 43 on the "Masters" layer), filled `desk` (#EEE9DC) at 62% with no stroke, moved clear of the other artboards. Re-exporting `A-dim-sierra` the same way reproduced the committed `map-A-dim-sierra.png` byte for byte.
- **Presentation manifest:** [`highlights.json`](highlights.json) gives each export's state, the spaces it covers, its atlas rectangle and its height. A graphic shows while every space it covers is in its state. The runtime places it as a flat unlit plane on the region tops: `map_at(x, y, height)` with the atlas source's own conversion, 0.002 above the region top and 0.002 below the region labels, as the proposal's Blender pass placed it. It is sorted behind the labels, so region names and population badges stay over every highlight; pieces, count labels, tags and Terror bells draw over it, and it never intercepts picks.
- **Admission:** [`highlights.sha256`](highlights.sha256) binds the ten veils and rules, the two piece markers, the four popover marks and the manifest. Like the status tags, it is presentation, not part of the save-bound identity, so saves stay compatible. A missing or changed file refuses at launch.
- **Tests:** `tests/highlights.rs` checks the hashes, the byte identity with the proposal exports and the export sizes against the rectangles; `tests/map_targets.rs` checks the states against the rules.

Santiago and Oriente become not involved when a target has nothing left in this task (a resolved Terror with no Kidnap left, or a removal that leaves no Underground Guerrilla). #72 shipped no veil for them; #73 exported `A-dim-santiago` and `A-dim-oriente`, so they now dim like every other space.

## Removal mode (#73)

"Remove own pieces" opens removal mode. `cuba_libre_unfolded::targets::removal_targets` reads each 26July group's state from the authority's removal choices:
- **Removable:** at least one of the group may be removed now. Marker: `atlas-piece-removable.png`, the red survey station (a ring with four ticks), artboard `A-piece-removable`.
- **Protected:** none may, because the owed Santiago Terror keeps the last Underground Guerrilla there (accepted commitment policy, local). Marker: `atlas-piece-protected.png`, the double ink ring and lock tab, artboard `A-piece-protected`.
- **Dimmed:** a space holding no 26July piece shows its `not involved` veil.

No space rule shows in removal mode. Each marker is the proposal's `graphics/piece-{artboard}.png`, byte for byte, at 8 px per unit. The manifest's `pieces` entries place it as the proposal's Blender pass did: a flat unlit square 64 atlas units (0.64 world units) across, centred under its piece, 0.003 above the region top, so it stays under the pieces, counts and region labels and never intercepts picks.

## The piece popover (#73)

In removal mode, clicking a removable 26July piece opens its popover (#58 story 49); hovering it opens nothing. Clicking it again, Cancel, a region or any decision closes it. A protected piece shows A-P6's tag on hover (story 48), and a click on it opens nothing. Until #58's final review a hover also opened a removable piece's popover, unpinned. Removal mode stays open after each removal until the panel's Done.

The frames are A-P5 (a stack, with the quantity stepper), A-P7 (one piece) and A-P6 (the protected tag) on the same FigCraft page. `src/tabletop/popover.rs` implements their values in reference pixels:
- **Card and header:** as the space popover. A-P6 is 300 wide with an `ink` top rule; it informs and nothing on it acts.
- **Body:** padding 8/14/10/14, gap 6, after one `rule` divider.
  - **Stepper (stacks only):** "Remove" `Popover/Bold 13`, gap 8, then an 80 × 26 frame (`paper`, 1 px `ink` stroke, radius 2) of − (24), the value (30, boxed by a 1 px `rule` stroke) and + (24), glyphs Bold 14 in `ink`, or `rule` where the bound stops them; then "of N" Regular 13.
  - **Preview:** gap 4. A limit's reason Regular 12.5 in `ink-secondary`; what returns Regular 12.5; Control and what the Operation still offers Bold 12.5; the standing Bold 12.5, or "Standings: unchanged."; the rule note Regular 11 in `ink-secondary`.
  - **Actions:** gap 8. Remove fills the row (`action/primary`, padding 7/12, radius 2, Bold 14 `paper`); Cancel is outlined (1 px `ink-secondary`, padding 7/12, radius 2, Regular 14 `ink`).
- **Footer:** "Removal stays open for more pieces until Done.", or A-P6's "Shown on hover; a click opens nothing."
- **Anchor:** the same dot and leader, on the piece rather than the region's end.

Every number is the authority's preview of the removal choice for the stepper's quantity; the stepper only chooses among the choices the authority lists, and Remove submits that choice through the checked interface.

**Seat names:** each 26July piece is `piece:<space> M26 <Kind> <State>`; the popover's controls are `popover:quantity:minus`, `popover:quantity:plus`, `popover:commit` and `popover:cancel`; the panel's `<revision>:done-removal` leaves the mode. The seat reports `table.removal` (each group's state and choices, and the dimmed spaces), `table.markers` and `table.popover` (with `kind` and `piece`).

## The space popover

Clicking a selectable or blocked space opens its popover (#58: "clicking a selectable or blocked space opens one popover"); hovering a space shows only its status tag. Clicking the space again, or a not-involved space, closes it. Any decision closes it. Until #58's final review a hover also opened the popover, unpinned. The popover lists every action the authority offers in that space:
- **Selectable** rows first. The chosen row (the first by default) is shaded `desk` and shows its preview above the only button that acts; choosing another row only changes the preview.
- **Not yet** rows: the reason and the remedies as chips.
- **Unavailable** rows: the reason, and that nothing in this task can change it.
- **Done** rows: a resolved Terror there, and the rolled Kidnap with its result.

The layout is [`atlas-popover.figcraft`](atlas-popover.figcraft), page "Popover masters (1920x1280 reference units)", frames A-P1, A-P3 and A-P4 (FigCraft CLI 0.18.1). `src/tabletop/popover.rs` implements its values in reference pixels:
- **Card:** 340 wide, `paper` fill, 1 px `ink` stroke, radius 2, drop shadow `ink` at 20% (0, 3, blur 10), 10 bottom padding; a 3 px `action/primary` top rule.
- **Header:** padding 9/14/8/14, gap 2. Title `Popover/Bold 15` in `ink`; subtitle Regular 11 in `ink-secondary`.
- **Rows:** separated by 1 px `rule` dividers. Padding 8/14/8/14 and gap 5; the chosen row 8/14/10/14 and gap 6.
  - **Head:** a 10 px state mark, gap 8, action in `Popover/Bold 13`, kind Regular 11.
  - **Preview:** indented 18, gap 4, lines `Popover/Regular 12.5`; the outcome line bold, rule notes Regular 11 in `ink-secondary`.
  - **Remedies:** "In this task:" Regular 11, then chips with padding 3/7, a 1 px `action/primary` stroke, radius 2 and Regular 12 labels.
  - **Button:** `action/primary`, radius 2, padding 7/12, label `Popover/Bold 14` in `paper`.
- **Footer:** padding 6/14/0/14, Regular 11 in `ink-secondary`.
- **Anchor:** the accepted status anchor dot and leader (`atlas-status-anchor.png`, `atlas-status-leader.png`), 8 and 39 reference pixels as in the 16:9 frame. The shared `LabelAnchor` keeps the dot on the region's end. The card sits to the right of the region where it fits before the Operation panel, else to its left, and inside the map vertically.

**Row marks.** The four 10 px state marks are the layout's own nodes, exported from FigCraft at 8x (`atlas-popover-mark-{selectable,not-yet,unavailable,done}.png`, nodes 4901, 4984, 4998 and 4950), so the "not yet" ring is dashed as designed. #72 drew it solid until #73 exported them.

**Emblems.** Since #73 the popover's running text, row heads, chips and footer name factions by #60's inline emblems (story 60); titles, which introduce their subject, keep the names, and on the red Remove or commit button 26July keeps its name (#60's rule, accepted in #68). The seat still reads every name.

## Production source and plates (#58 review)

- **Source:** [`atlas-popover.figcraft`](atlas-popover.figcraft) is the #66 proposal's source with HF-B and HF-C removed: the HF-A 16:9 frame, the HF-A states sheet and the popover masters A-P1 to A-P7 (FigCraft CLI 0.18.1). It keeps the `Atlas` colour variables and the `Popover/*` text styles. A-P1's parts are named as the runtime names them (`popover-rule`, `popover-header`, `popover-divider`, `popover-row-chosen`, `popover-commit`, `popover-row`, `popover-footer`). The file still carries the proposal's unreferenced embedded images (the unselected HF-B and HF-C captures, which show no card face); FigCraft has no command that drops them.
- **Plates:** the frame "Production plates" on the masters page draws the popover's chrome at reference size in the `Atlas` variables, exported at 3x as `atlas-popover-{card,primary,cancel,chip,row-chosen,rule-primary,rule-ink,divider,stepper,stepper-value}.png`. The card plate carries the drop shadow, so its export reaches 15, 12, 15 and 18 reference pixels past the card's box (left, top, right, bottom). The runtime draws each plate behind its node, nine-sliced with its corners at reference size (the rules, the divider and the chosen row are stretched); it draws no fill, stroke or shadow of its own.
- **Layout record:** [`atlas-popover.layout.json`](atlas-popover.layout.json) records A-P1's parts relative to the card and each plate's size, outset and slice. It was read from the saved document and the exports by an editor-side script kept with the review evidence (`local/verification/review-fix-2/scripts/popover-record.py`). The native regression `native_space_popover_matches_layout` compares the running popover with it at 3840×2160 (the reference ×1.6875) within 4 px, as the transfer modal's check does; parts that hold wrapped text are compared on the edges the text does not move. The deltas are 0.2–2.2 px.
- **Admission:** [`popover.sha256`](popover.sha256) admits the ten plates at launch. A missing or changed plate refuses launch. Like the highlights, they are presentation and do not change the save-bound identity.
- **Placement margin:** the card keeps 8 reference pixels inside the map and clear of the Operation panel. That is the runtime's own placement rule (#72), not a FigCraft value: the 16:9 frame places its one card by hand.

