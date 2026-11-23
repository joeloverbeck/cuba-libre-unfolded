# Available Forces ledger

The owner selected layout **B · Libro** and piece graphics **G1 · Sólido** on 2026-11-23 ([decision record](../proposals/available-forces/README.md), sheet [`layout-b-libro.png`](../proposals/available-forces/layout-b-libro.png)). Spec [#157](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/157) specifies the ledger; [#159](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/159) built it at rest and [#160](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/160) its tag. This file owns its visual facts.

## What it shows

- **Place:** a second ledger directly left of the Resources ledger ([TOP.md](TOP.md)), at its top and height, captioned "AVAILABLE FORCES". It is the main view's ninth area, `available-forces`, anchored top right ([MAIN-VIEW.md](MAIN-VIEW.md#areas)).
- **Rows:** one per faction, in the Resources ledger's order and rhythm: Government, 26July, Directorio, Syndicate. Each begins with the faction's victory emblem, the Resources ledger's emblem at its size.
- **Columns:** Troops or Guerrillas; Police (Government only); Bases or Casinos. Bases and Casinos share the third column for all four factions. Each filled cell is the kind's piece and its count beside it.
- **Counts:** the rules authority's Available Forces (`observation.available`), summed by faction and kind over every state. The ledger computes nothing else. It draws the committed position, so a hovered or pinned preview leaves it unchanged.
- **None left:** the cell stays. Its piece is drawn at 40% opacity beside "0", as the frame `zero state` draws it. No reachable position in this build empties an inventory, so the native app has not shown it; a unit test builds it from an observation.
- **Pointer:** the ledger's area takes the pointer: a press, a scroll or a drag on it neither moves the camera nor reaches a space or piece. Space tags and popovers keep off it, as they keep off every area.

## The tag

The study's [`hover-tag.png`](../proposals/available-forces/hover-tag.png) drew it, on the accepted top-display tag ([TOP.md](TOP.md)), so it is not a new design.

- **Opening:** hovering a faction's row, the band across the plate between the rules around it, or focusing it with Tab, opens that faction's tag after the shared 400 ms dwell. It closes when the pointer or focus leaves, as the top display's tags do. There is no pin; reading it changes nothing.
- **Words:** the title "Available <faction> Forces"; one row per kind, its plural name and "N of T · M on the map": N Available, T the faction's inventory of that kind from the Available Forces chart (rules, p. 20; `INVENTORY` in the rules model) and M its pieces on the map in any state, a closed Casino included. Then the rule (R 1.4.1): "Pieces are placed only from Available Forces. With none of a kind left, a placement is skipped and a replacement only removes (R 1.4.1). While executing, a faction may return its own pieces from the map to Available." N + M is T in every position the tests reach.
- **Geometry and export:** the top-display tag's: `atlas-top-tag.png` stretched to the tag, 468 reference pixels wide; the title, the rule `atlas-victory-rule.png`, rows from 76 every 28 with values at 230, and notes 12 below the rows at 23 per line, broken by hand at 50 characters. The Government's, three rows and five note lines, is 790×508 on the owner's display; the others are 47 shorter.
- **Place:** every faction's tag opens at one place, anchored top right as the ledger is: 8 reference pixels (13.5 on the owner's display) left of the ledger, its top 8 below the standings area's foot, so 2125.5, 209.25 on the owner's display. It covers neither ledger, the Cash supply, the standings nor the decision panel, and stays inside the window at 3:2, 16:9 and 21:9: it lies over the map only (see "Provenance and limits").
- **Tab order:** the panel's controls, then the four rows (`available:government`, `available:26july`, `available:directorio`, `available:syndicate`), then the cards and the piles. The rows stand above the panel, so they follow it and come before the card tray at the bottom left.

## Files

- **Graphics source:** `atlas-available-forces.vectorcraft`, authored through the VectorCraft MCP from the study's `available-forces-graphics.vectorcraft`: G1's nine pieces, copied and scaled with their strokes. Each artboard is drawn at 6× the 1920×1280 reference and exported at 0.5, which is 3× the reference, as the top display's are ([TOP.md](TOP.md)). Each piece fills its artboard's height (the bases its width) and is centred in it.
- **Layout source:** `atlas-available-forces.figcraft`, authored through the FigCraft MCP from the study's `available-forces-layouts.figcraft`, with the other studies, the study's hover tag and the crop removed. Three frames:
  - **`16:9 available forces` (3840×2160):** the owner's display. The other areas are the study's locked context. The ledger's nodes are named as the runtime names its parts. `plate` groups the panel (fill `paper`, inside stroke `rule`, 2 wide, corner radius 10), the header rule (`ink`, 2 high) and the three row rules (`rule`, 1.5 high).
  - **`zero state`:** the Government's row with its Bases at 0: the piece at opacity 0.4 beside "0".
  - **`hover tag · Government`** (#160): the Government's tag at the owner's display, the top-display tag's geometry × 1.6875, with the text styles `Tag/title` (Lato 45.5625, `ink`), `Tag/row` (30.375; labels `ink`, values `ink-secondary`) and `Tag/note` (28.6875, 38.8125 per line, `ink-secondary`). Its export is `atlas-available-forces-tag.png`, for review; no manifest admits it, since the runtime draws the admitted top-display plate. The text beside it, `tag placement rule`, states where the tag opens. The 16:9 frame holds hidden rectangles: each row's target, `<faction>-row`, and `government-tag`, where the tag opens.
- **Palette and type:** the file's `Atlas` variables, the tag's three text styles (above) and the ledger's two. `Ledger/caption` is Lato Bold 24 with 3 tracking in `ink-secondary`, centred in the plate. `Ledger/count` is Lato Regular 43.875 (26 px at the reference, as the Resources ledger's values) in `ink`, left-aligned in a 56-wide box beside its piece. `atlas-available-forces.variables.json` is the `Atlas` export ([README.md, "Interface palette"](README.md#interface-palette)).
- **Layout record:** `atlas-available-forces.layout.json`: FigCraft's `absoluteBounds` of every part in `parts` (images) and `text`, in frame units, which are pixels of the owner's display; the `anchor`, top right; the `zero` opacity; and the `exports` with their sizes. `tolerance` is 2. Its `tag` holds the rows' targets, the tag frame's parts and text relative to the frame, and the `open` rectangle.
- **Manifest:** `available-forces.sha256` admits the plate and the nine pieces. A missing or changed file refuses launch before any save is created or replaced, like every other manifest. The emblems are the admitted `atlas-victory-icon-*.png`.

| Export | From | PNG | Frame rectangle of the Government's or 26July's cell |
|---|---|---:|---|
| `atlas-available-forces-plate.png` | FigCraft group `plate`, at 3/1.6875 | 879×571 | 2928.75, 33.75, 494×320.63 |
| `atlas-available-troops.png` | artboard `troops` | 77×81 | 3027.25, 128.25, 43.31×45.56 |
| `atlas-available-police.png` | artboard `police` | 77×81 | 3157.25, 128.25, 43.31×45.56 |
| `atlas-available-<faction>-base.png` (government, 26july, directorio) | artboard `<faction>-base` | 100×61 | 3280.75, 134.25, 56.25×34.31 |
| `atlas-available-<faction>-guerrilla.png` (26july, directorio, syndicate) | artboard `<faction>-guerrilla` | 52×82 | 3034.25, 180.56, 29.25×46.13 |
| `atlas-available-syndicate-casino.png` | artboard `syndicate-casino` | 100×61 | 3280.75, 291.19, 56.25×34.31 |

Rows lie 52.31 apart (31 reference pixels), from the Government's emblem at 2952.75, 128.25, 45.56×45.56. A Base or Casino sits 6 below its row's top; a count's box starts 3.375 above it, at x 3082.75, 3212.75 or 3346.75. In reference pixels the plate is 1380, 20, 292.74×190, right-anchored: the decision panel's left edge, and the Resources ledger's top and height.

## Frame and alignment

- **Frame:** the record uses the 3840×2160 frame of the main view, the pile tags and the study, so the native regression compares it with a capture at the owner's display directly. The top display's record keeps its 6144×3456 frame.
- **Alignment:** the study drew the plate at x 2930, 1.25 right of the decision panel's left edge. Production aligns them at 2928.75, which the spec's 2-unit allowance permits, and moves every part of the ledger with it. The plate is then just right of the popovers' clear region, whose right edge is the decision panel's left.
- **Piece sizes:** each piece's rectangle is its export's size at 3× the reference, so the artboards have whole sizes. Each differs from the study's placement by at most 0.31.

## Pointer and seat

`area:available-forces` is the ledger's area. Each row is the component `available:<faction>`, which the seat points at or focuses; the open tag is `table.top`. The development seat reports the ledger as `table.available_forces`: its `rows` by faction, with each cell's `kind` and `count`, and its drawn `parts` by their names in the record, each with its `bounds`, a count's `text` and whether a piece is `dimmed` ([seat reference](../../docs/agents/seat.md#what-the-table-reports)).

## Provenance and limits

- **Tools:** VectorCraft CLI 0.24.1 and FigCraft CLI 0.18.1, the binaries [art-tools.md](../../docs/agents/art-tools.md) records.
- **The Resources tag:** it opens directly left of the Resources ledger at its top ([TOP.md](TOP.md)), which is now this ledger's place, so while it is open it covers this ledger. #159 left it there; the owner judges it in #157's native review.
- **What the tag covers:** only the map, in every window checked. At 3840×2160 every faction's tag lies over open sea between the standings and the decision panel, 13.5 left of this ledger and below the standings, and covers no space; at 1920×1280 (3:2) it lies just below the 26July, Directorio and Syndicate standings, and at 2520×1080 (21:9) over sea left of the ledger, again covering no space. A unit test checks the same against every area at six windows from 3:2 to 21:9.
- **What this establishes:** placement and count evidence from the native regressions `native_available_forces_ledger_matches_layout_and_follows_a_removal`, `native_available_forces_tags_open_on_hover_and_focus_and_follow_a_removal` and `native_available_forces_follow_a_rally_and_not_its_preview`, with the main view's regressions for the area in other windows. It is not owner play acceptance: the owner judges the ledger's legibility on the display in #157's review.
