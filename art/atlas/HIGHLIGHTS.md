# Map highlights and the space popover

The owner selected **HF-A, Survey ink** in #58's combined design round ([#68 selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)). The proposal, its rationale and the unselected alternatives remain in [`../proposals/issue-66/`](../proposals/issue-66/README.md). #72 ships the space states and the space popover; #73 adds removal mode's piece markers and piece popover from the same proposal.

## Space states

While 26July executes Terror, `cuba_libre_unfolded::targets::space_targets` reads every space's state from the rules authority's own choices:
- **Selectable:** a supported choice acts there now. The graphic is a 5-unit `action/primary` rule just inside the border, with a hairline ink rule inside it.
- **Blocked with a remedy:** nothing there is supported, but a choice there names an in-task remedy. The graphic is the same red rule, dashed.
- **Not involved:** neither. The graphic is a 62% `desk` veil.

Outside these states (the action choice, either ending, an opened Kidnap, a Resource transfer under way and removal mode), no space is a target, no highlight shows and a region click does nothing.

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

- **Source:** [`../proposals/issue-66/map-highlights.vectorcraft`](../proposals/issue-66/map-highlights.vectorcraft) (VectorCraft CLI 0.24.1), the HF-A artboards named above. Each export is the proposal's `graphics/map-{artboard}.png`, byte for byte, at 4 px per map unit.
- **Presentation manifest:** [`highlights.json`](highlights.json) gives each export's state, the spaces it covers, its atlas rectangle and its height. A graphic shows while every space it covers is in its state. The runtime places it as a flat unlit plane on the region tops: `map_at(x, y, height)` with the atlas source's own conversion, 0.002 above the region top and 0.002 below the region labels, as the proposal's Blender pass placed it. It is sorted behind the labels, so region names and population badges stay over every highlight; pieces, count labels, tags and Terror bells draw over it, and it never intercepts picks.
- **Admission:** [`highlights.sha256`](highlights.sha256) binds the eight PNGs and the manifest. Like the status tags, it is presentation, not part of the save-bound identity, so saves stay compatible. A missing or changed file refuses at launch.
- **Tests:** `tests/highlights.rs` checks the hashes, the byte identity with the proposal exports and the export sizes against the rectangles; `tests/map_targets.rs` checks the states against the rules.

**Known gap.** The proposal veils only the spaces that are never targets. Santiago and Oriente become not involved when a target has nothing left in this task (a resolved Terror with no Kidnap left, or a removal that leaves no Underground Guerrilla). No HF-A export veils them, so they then show no highlight. The two veils (`A-dim-santiago`, `A-dim-oriente`) need exporting in VectorCraft; the runtime picks them up from this manifest without code changes.

## The space popover

Hovering a selectable or blocked space shows its popover after the region tag's dwell; clicking the space keeps it open, and clicking it again, or a not-involved space, closes it. Any decision closes it. The popover lists every action the authority offers in that space:
- **Selectable** rows first. The chosen row (the first by default) is shaded `desk` and shows its preview above the only button that acts; choosing another row only changes the preview.
- **Not yet** rows: the reason and the remedies as chips.
- **Unavailable** rows: the reason, and that nothing in this task can change it.
- **Done** rows: a resolved Terror there, and the rolled Kidnap with its result.

The layout is [`../proposals/issue-66/highlight-popover-layouts.figcraft`](../proposals/issue-66/highlight-popover-layouts.figcraft), page "Popover masters (1920x1280 reference units)", frames A-P1, A-P3 and A-P4 (FigCraft CLI 0.18.1). `src/tabletop/popover.rs` implements its values in reference pixels:
- **Card:** 340 wide, `paper` fill, 1 px `ink` stroke, radius 2, drop shadow `ink` at 20% (0, 3, blur 10), 10 bottom padding; a 3 px `action/primary` top rule.
- **Header:** padding 9/14/8/14, gap 2. Title `Popover/Bold 15` in `ink`; subtitle Regular 11 in `ink-secondary`.
- **Rows:** separated by 1 px `rule` dividers. Padding 8/14/8/14 and gap 5; the chosen row 8/14/10/14 and gap 6.
  - **Head:** a 10 px state mark, gap 8, action in `Popover/Bold 13`, kind Regular 11.
  - **Preview:** indented 18, gap 4, lines `Popover/Regular 12.5`; the outcome line bold, rule notes Regular 11 in `ink-secondary`.
  - **Remedies:** "In this task:" Regular 11, then chips with padding 3/7, a 1 px `action/primary` stroke, radius 2 and Regular 12 labels.
  - **Button:** `action/primary`, radius 2, padding 7/12, label `Popover/Bold 14` in `paper`.
- **Footer:** padding 6/14/0/14, Regular 11 in `ink-secondary`.
- **Anchor:** the accepted status anchor dot and leader (`atlas-status-anchor.png`, `atlas-status-leader.png`), 8 and 39 reference pixels as in the 16:9 frame. The shared `LabelAnchor` keeps the dot on the region's end. The card sits to the right of the region where it fits before the Operation panel, else to its left, and inside the map vertically.

**Deviation.** FigCraft's "not yet" mark is a dashed ring. Bevy's UI draws only solid borders, and no VectorCraft export of the four row marks exists, so the runtime draws it as a solid `action/primary` ring; the red "not yet" label still distinguishes it. Exporting the four marks would close this.

The popover spells faction names, as the selected layout does, rather than #60's inline emblems.
