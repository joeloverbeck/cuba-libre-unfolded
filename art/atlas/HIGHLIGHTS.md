# Map highlights and the space popover

The owner selected **HF-A, Survey ink** in #58's combined design round ([#68 selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)). The proposal, its rationale and the unselected alternatives remain in [`../proposals/issue-66/`](../proposals/issue-66/README.md). #72 ships the space states and the space popover; #73 adds removal mode's piece markers and piece popover from the same proposal.

## Space states

While a decision offers choices that act on a space, `cuba_libre_unfolded::targets::space_targets` reads every space's state from the rules authority's own choices. Each such choice states the space it acts on and whether it changes anything there (`Choice.site`), so the states need no rule per Operation: 26July's Terror (#72) and Government's Assault (#169) use the same reading.
- **Selectable:** a supported choice there changes something now. The graphic is a 5-unit `action/primary` rule just inside the border, with a hairline ink rule inside it.
- **Blocked with a remedy:** no such choice is supported, but one names an in-task remedy. The graphic is the same red rule, dashed.
- **No effect (#169):** the space's only choices change nothing on the map, as Assault's plan that removes nothing. The space stays choosable (R 3.2.4), and it is shown as quietly as a space no choice acts on: its graphic is the not-involved veil. No graphic of its own was drawn.
- **Not involved:** no choice acts there, or none could. That includes a Terror with no remedy, Assault plans that would remove or close a piece where nothing can pay and no remedy is named, and a space already paid in this Assault (R 3.1). The graphic is a 62% `desk` veil, and a click opens nothing.

During Assault a space is selectable where a plan would remove or close a piece and Government can pay, blocked where one would but Government cannot pay (a Resource transfer is the remedy), not involved where one would but no faction has a Resource to give, and no effect where no plan would. Tab reaches the selectable, blocked and no-effect spaces, in that order, and passes the not-involved ones ([seat reference](../../docs/agents/seat.md#keys)).

Outside these states (the action choice, either ending, an opened Kidnap, a Resource transfer under way and removal mode), no space is a target and a region click opens no space popover. Removal mode has its own states, below.

| Runtime export | Source artboard | State | Space | Atlas rectangle (x, y, w, h) | Height |
|---|---|---|---|---|---:|
| `atlas-highlight-selectable-pinar-del-rio.png` | `A-sel-pinar-del-rio` | selectable | Pinar del Río | 50, 376, 232, 198 | 0.142 |
| `atlas-highlight-blocked-pinar-del-rio.png` | `A-blk-pinar-del-rio` | blocked | Pinar del Río | 50, 376, 232, 198 | 0.142 |
| `atlas-highlight-veil-pinar-del-rio.png` | `A-dim-pinar-del-rio` | not involved | Pinar del Río | 50, 376, 232, 198 | 0.142 |
| `atlas-highlight-selectable-la-habana.png` | `A-sel-la-habana` | selectable | La Habana | 214, 327, 230, 137 | 0.142 |
| `atlas-highlight-blocked-la-habana.png` | `A-blk-la-habana` | blocked | La Habana | 214, 327, 230, 137 | 0.142 |
| `atlas-highlight-veil-la-habana.png` | `A-dim-la-habana` | not involved | La Habana | 214, 327, 230, 137 | 0.142 |
| `atlas-highlight-selectable-matanzas.png` | `A-sel-matanzas` | selectable | Matanzas | 334, 317, 260, 215 | 0.142 |
| `atlas-highlight-blocked-matanzas.png` | `A-blk-matanzas` | blocked | Matanzas | 334, 317, 260, 215 | 0.142 |
| `atlas-highlight-veil-matanzas.png` | `A-dim-matanzas` | not involved | Matanzas | 334, 317, 260, 215 | 0.142 |
| `atlas-highlight-selectable-las-villas.png` | `A-sel-las-villas` | selectable | Las Villas | 506, 334, 280, 264 | 0.142 |
| `atlas-highlight-blocked-las-villas.png` | `A-blk-las-villas` | blocked | Las Villas | 506, 334, 280, 264 | 0.142 |
| `atlas-highlight-veil-las-villas.png` | `A-dim-las-villas` | not involved | Las Villas | 506, 334, 280, 264 | 0.142 |
| `atlas-highlight-selectable-camaguey-province.png` | `A-sel-camaguey-province` | selectable | Camagüey Province | 714, 423, 304, 289 | 0.142 |
| `atlas-highlight-blocked-camaguey-province.png` | `A-blk-camaguey-province` | blocked | Camagüey Province | 714, 423, 304, 289 | 0.142 |
| `atlas-highlight-veil-camaguey-province.png` | `A-dim-camaguey-province` | not involved | Camagüey Province | 714, 423, 304, 289 | 0.142 |
| `atlas-highlight-selectable-oriente.png` | `A-sel-oriente` | selectable | Oriente | 880, 550, 370, 186 | 0.142 |
| `atlas-highlight-blocked-oriente.png` | `A-blk-oriente` | blocked | Oriente | 880, 550, 370, 186 | 0.142 |
| `atlas-highlight-veil-oriente.png` | `A-dim-oriente` | not involved | Oriente | 880, 550, 370, 186 | 0.142 |
| `atlas-highlight-selectable-sierra-maestra.png` | `A-sel-sierra` | selectable | Sierra Maestra | 914, 648, 416, 194 | 0.142 |
| `atlas-highlight-blocked-sierra-maestra.png` | `A-blk-sierra` | blocked | Sierra Maestra | 914, 648, 416, 194 | 0.142 |
| `atlas-highlight-veil-sierra-maestra.png` | `A-dim-sierra` | not involved | Sierra Maestra | 914, 648, 416, 194 | 0.142 |
| `atlas-highlight-selectable-havana.png` | `A-sel-havana` | selectable | Havana | 220, 150, 230, 230 | 0.242 |
| `atlas-highlight-blocked-havana.png` | `A-blk-havana` | blocked | Havana | 220, 150, 230, 230 | 0.242 |
| `atlas-highlight-veil-havana.png` | `A-dim-havana` | not involved | Havana | 220, 150, 230, 230 | 0.242 |
| `atlas-highlight-selectable-camaguey-city.png` | `A-sel-camaguey-city` | selectable | Camagüey City | 695, 590, 220, 220 | 0.242 |
| `atlas-highlight-blocked-camaguey-city.png` | `A-blk-camaguey-city` | blocked | Camagüey City | 695, 590, 220, 220 | 0.242 |
| `atlas-highlight-veil-camaguey-city.png` | `A-dim-camaguey-city` | not involved | Camagüey City | 695, 590, 220, 220 | 0.242 |
| `atlas-highlight-selectable-santiago.png` | `A-sel-santiago` | selectable | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-blocked-santiago.png` | `A-blk-santiago` | blocked | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-veil-santiago.png` | `A-dim-santiago` | not involved | Santiago | 1060, 715, 240, 240 | 0.242 |
| `atlas-highlight-selectable-ec-pinar-la-habana.png` | `A-sel-ec-pinar-la-habana` | selectable | Pinar–La Habana economic center | 194, 355, 96, 96 | 0.282 |
| `atlas-highlight-blocked-ec-pinar-la-habana.png` | `A-blk-ec-pinar-la-habana` | blocked | Pinar–La Habana economic center | 194, 355, 96, 96 | 0.282 |
| `atlas-highlight-veil-ec-pinar-la-habana.png` | `A-dim-ec-pinar-la-habana` | not involved | Pinar–La Habana economic center | 194, 355, 96, 96 | 0.282 |
| `atlas-highlight-selectable-ec-las-villas-camaguey.png` | `A-sel-ec-las-villas-camaguey` | selectable | Las Villas–Camagüey economic center | 702, 487, 96, 96 | 0.282 |
| `atlas-highlight-blocked-ec-las-villas-camaguey.png` | `A-blk-ec-las-villas-camaguey` | blocked | Las Villas–Camagüey economic center | 702, 487, 96, 96 | 0.282 |
| `atlas-highlight-veil-ec-las-villas-camaguey.png` | `A-dim-ec-las-villas-camaguey` | not involved | Las Villas–Camagüey economic center | 702, 487, 96, 96 | 0.282 |
| `atlas-highlight-selectable-ec-oriente-sierra.png` | `A-sel-ec-oriente-sierra` | selectable | Oriente–Sierra economic center | 1122, 652, 96, 96 | 0.282 |
| `atlas-highlight-blocked-ec-oriente-sierra.png` | `A-blk-ec-oriente-sierra` | blocked | Oriente–Sierra economic center | 1122, 652, 96, 96 | 0.282 |
| `atlas-highlight-veil-ec-oriente-sierra.png` | `A-dim-ec-oriente-sierra` | not involved | Oriente–Sierra economic center | 1122, 652, 96, 96 | 0.282 |

- **Source:** [`../proposals/issue-66/map-highlights.vectorcraft`](../proposals/issue-66/map-highlights.vectorcraft), the HF-A artboards named above, exported as PNG at 4 px per map unit.
  - The proposal (VectorCraft CLI 0.24.1) made the Santiago and Oriente rules and `A-dim-sierra`. Each of those exports is the proposal's `graphics/map-{artboard}.png`, byte for byte.
  - #73 added `A-dim-santiago` and `A-dim-oriente` the way the proposal made the other veils: a copy of the region's atlas profile (objects 47 and 43 on the "Masters" layer), filled `desk` (#EEE9DC) at 62% with no stroke, moved clear of the other artboards. Re-exporting `A-dim-sierra` the same way reproduced the committed `map-A-dim-sierra.png` byte for byte.
  - #167 added the other 32 artboards with VectorCraft 0.24.1 built from `storytold/vectorcraft@88f5ece1`, each from its space's profile on the "Masters" layer, the way the proposal's Santiago and Oriente graphics are built (re-running the steps on Oriente reproduced their outlines exactly):
    - **Rule** (`A sel {space}: red rule`): Offset Path −5 with round joins, no fill, a 5-unit `action/primary` (#A03531) stroke with round joins. **Ink rule** (`A sel {space}: ink rule`): Offset Path −5.5 of the rule, round joins, a 1.5-unit `ink` (#203238) stroke.
    - **Dashed rule** (`A blk {space}: dashed rule`): a copy of the rule with a 4-unit #A03531 stroke, round caps and joins, dashes 12/8.
    - **Veil** (`A dim {space}: desk veil`): a copy of the profile, as #73's.
    - Each space's copies are moved by (1500 × its column, 5000) for the rule, 6200 for the dashed rule and 7400 for the veil, clear of every other artboard and of the masters. Its artboards are its profile's bounds plus 20 units on each side, moved by the same offsets, so its three graphics share one rectangle.
  - The current build renders stroke edges a little differently from the proposal's: re-exporting `A-sel-santiago` changes 3,660 pixels, all along the rules' anti-aliased edges. The proposal's ten exports therefore stay as committed rather than re-exported. The veils reproduce exactly.
  - The proposal's grouped veils (`A-dim-provinces`, `A-dim-cities`, `A-dim-ecs`) remain in the source and in `graphics/` as part of the proposal; #167 removed their runtime exports.
- **Presentation manifest:** [`highlights.json`](highlights.json) gives each export's state, its one space, its atlas rectangle and its height. A graphic shows while its space is in its state. Each space's height is the height its veil had before #167: 0.142 on a Province, 0.242 on a City and 0.282 on an economic center. The runtime places it as a flat unlit plane on the region tops: `map_at(x, y, height)` with the atlas source's own conversion, 0.002 above the region top and 0.002 below the region labels, as the proposal's Blender pass placed it. It is sorted behind the labels, so region names and population badges stay over every highlight; pieces, count labels, tags and Terror bells draw over it, and it never intercepts picks.
- **Admission:** [`highlights.sha256`](highlights.sha256) binds the 39 rules, dashed rules and veils, the two piece markers, the four popover marks and the manifest. A missing or changed file refuses at launch. For saves, see [launch admission](README.md#launch-admission).
- **Tests:** `tests/integration/highlights.rs` checks the 39 hashes, the byte identity of the seven pre-#167 graphics with their admitted bytes and proposal exports, each export's size against its rectangle, one graphic per space and state at its space's height, and that a missing or changed graphic refuses launch; `tests/integration/map_targets.rs` checks the states against the rules.

Santiago and Oriente become not involved when a target has nothing left in this task (a resolved Terror with no Kidnap left, or a removal that leaves no Underground Guerrilla). #72 shipped no veil for them; #73 exported `A-dim-santiago` and `A-dim-oriente`, so they now dim like every other space. Until #167 three veils covered groups of spaces (the five western Provinces, Havana with Camagüey City, and the three economic centers) and showed only while every space of the group was not involved; now each space dims on its own.

## Removal mode (#73)

"Remove own pieces" opens removal mode. `cuba_libre_unfolded::targets::removal_targets` reads each of the executing faction's groups from the authority's removal choices. Government uses the same markers and piece popover for its Troop and Police stacks during Assault.
- **Removable:** at least one of the group may be removed now. Marker: `atlas-piece-removable.png`, the red survey station (a ring with four ticks), artboard `A-piece-removable`.
- **Protected:** none may, because the owed Santiago Terror keeps the last Underground Guerrilla there (accepted commitment policy, local). Marker: `atlas-piece-protected.png`, the double ink ring and lock tab, artboard `A-piece-protected`.
- **Dimmed:** a space holding none of the executing faction's pieces shows its `not involved` veil.

No space rule shows in removal mode. Each marker is the proposal's `graphics/piece-{artboard}.png`, byte for byte, at 8 px per unit. The manifest's `pieces` entries place it as the proposal's Blender pass did: a flat unlit square 64 atlas units (0.64 world units) across, centred under its piece, 0.003 above the region top, so it stays under the pieces, counts and region labels and never intercepts picks.

## The piece popover (#73)

In removal mode, clicking one of the executing faction's removable pieces opens its popover (#58 story 49). Clicking it again, Cancel, a region or any decision closes it. A protected piece shows A-P6's tag on hover (story 48), and a click on it opens nothing. Until #58's final review a hover also opened a removable piece's popover, unpinned. Removal mode stays open after each removal until the panel's Done.

The frames are A-P5 (a stack, with the quantity stepper), A-P7 (one piece) and A-P6 (the protected tag) on the same FigCraft page. `src/tabletop/popover.rs` implements their values in reference pixels:
- **Card and header:** as the space popover. A-P6 is 300 wide with an `ink` top rule; it informs and nothing on it acts.
- **Body:** padding 8/14/10/14, gap 6, after one `rule` divider.
  - **Stepper (stacks only):** "Remove" `Popover/Bold 13`, gap 8, then an 80 × 26 frame (`paper`, 1 px `ink` stroke, radius 2) of − (24), the value (30, boxed by a 1 px `rule` stroke) and + (24), glyphs Bold 14 in `ink`, or `rule` where the bound stops them; then "of N" Regular 13.
  - **Preview:** gap 4. A limit's reason Regular 12.5 in `ink-secondary`; what returns Regular 12.5; Control and what the Operation still offers Bold 12.5; the standing Bold 12.5, or "Standings: unchanged."; the rule note Regular 11 in `ink-secondary`.
  - **Actions:** gap 8. Remove fills the row (`action/primary`, padding 7/12, radius 2, Bold 14 `paper`); Cancel is outlined (1 px `ink-secondary`, padding 7/12, radius 2, Regular 14 `ink`).
- **Footer:** "Removal stays open for more pieces until Done.", or A-P6's "Shown on hover; a click opens nothing."
- **Anchor:** the same dot and leader, on the piece rather than the region's end.

Every number is the authority's preview of the removal choice for the stepper's quantity; the stepper only chooses among the choices the authority lists, and Remove submits that choice through the checked interface.

The development-seat reference lists the removal mode and piece popover [components](../../docs/agents/seat.md#components), the [choices](../../docs/agents/seat.md#choices) they submit and [what the table reports](../../docs/agents/seat.md#what-the-table-reports).

## The space popover

Clicking a selectable or blocked space opens its popover (#58: "clicking a selectable or blocked space opens one popover"), and so does clicking a no-effect space (#169); hovering a space shows only its status tag. Clicking the space again, or a not-involved space, closes it. Any decision closes it. Until #58's final review a hover also opened the popover, unpinned. The popover lists every action the authority offers in that space:
- **Selectable** rows first. The chosen row (the first by default) is shaded `desk` and shows its preview above the only button that acts; choosing another row only changes the preview.
- **Not yet** rows: the reason and the remedies as chips.
- **Unavailable** rows: the reason, and that nothing in this task can change it.
- **Done** rows: a resolved Terror there, and the rolled Kidnap with its result.

**During Assault (#169)** the rows are the space's plans, in the authority's order: each plan that removes or closes at least one piece, headed "Target" and the factions it targets, with the split when the capacity is short; then the one plan that removes nothing, headed "Removes nothing". Each row's kind is "Assault". The subtitle gives the terrain, "Government executing Assault" and Government's Resources.
- **The chosen row's preview** is the authority's: the capacity arithmetic (R 3.2.4), the payment, whom the plan can affect, what it removes or closes in bold, any change of Control or standing in bold, and, when Government could then pay for no other space, a quiet note that says so.
- **The button** reads "Pay N · Assault in <space>", where N is the cost per space at the current US Alliance (R 6.3.1). For the plan that removes nothing it reads "Pay N · Assault removes nothing".
- **Without the cost**, every row is "not yet" with the authority's reason and the chip "Request a Resource transfer".
- A no-effect space's popover holds the one plan that removes nothing, chosen. A paid space has no popover.

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
- **Plates:** the frame "Production plates" on the masters page draws the popover's chrome at reference size in the `Atlas` variables, exported at 3x as `atlas-popover-{card,primary,cancel,chip,row-chosen,rule-primary,rule-ink,divider,stepper,stepper-value}.png`. `atlas-popover-card-slip.png` is the notice slip's card ([MAIN-VIEW.md](MAIN-VIEW.md#save--quit-and-the-notice-slip)): the card plate copied to a 36-unit box and sliced 30 in, so Bevy keeps its corners at reference size on a one-line slip; its corners are the card plate's pixel for pixel. The card plate carries the drop shadow, so its export reaches 15, 12, 15 and 18 reference pixels past the card's box (left, top, right, bottom). The runtime draws each plate behind its node, nine-sliced with its corners at reference size (the rules, the divider and the chosen row are stretched); it draws no fill, stroke or shadow of its own.
- **Layout record:** [`atlas-popover.layout.json`](atlas-popover.layout.json) records A-P1's parts relative to the card and each plate's size, outset and slice. It was read from the saved document and the exports by an editor-side script kept with the review evidence (`local/verification/review-fix-2/scripts/popover-record.py`). The entry for `atlas-popover-card-slip` (#166) is the exception: it was written by hand from the node, size and export FigCraft reported, a manual record and not the script's output; the popover integration test holds its export size, hash and outset, and the unit test `the_plates_are_the_layout_record_s` holds its slice. The native regression `native_space_popover_matches_layout` compares the running popover with it at 3840×2160 (the reference ×1.6875) within 4 px, as the transfer modal's check does; parts that hold wrapped text are compared on the edges the text does not move. The deltas are 0.2–2.2 px.
- **Admission:** [`popover.sha256`](popover.sha256) admits the eleven plates at launch. A missing or changed plate refuses launch. For saves, see [launch admission](README.md#launch-admission).
- **Placement margin:** the card keeps 8 reference pixels inside the part of the window that the [main view](MAIN-VIEW.md#what-it-shows) leaves clear for it ("Tags and popovers"). That is the runtime's own placement rule (#72, #142), not a FigCraft value: the 16:9 frame places its one card by hand.

