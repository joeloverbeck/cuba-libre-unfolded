# Available Forces ledger

The owner selected layout **B · Libro** and piece graphics **G1 · Sólido** on 2026-11-23 ([decision record](../proposals/available-forces/README.md), sheet [`layout-b-libro.png`](../proposals/available-forces/layout-b-libro.png)). Spec [#157](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/157) specifies the ledger; [#159](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/159) built it at rest. This file owns its visual facts.

## What it shows

- **Place:** a second ledger directly left of the Resources ledger ([TOP.md](TOP.md)), at its top and height, captioned "AVAILABLE FORCES". It is the main view's ninth area, `available-forces`, anchored top right ([MAIN-VIEW.md](MAIN-VIEW.md#areas)).
- **Rows:** one per faction, in the Resources ledger's order and rhythm: Government, 26July, Directorio, Syndicate. Each begins with the faction's victory emblem, the Resources ledger's emblem at its size.
- **Columns:** Troops or Guerrillas; Police (Government only); Bases or Casinos. Bases and Casinos share the third column for all four factions. Each filled cell is the kind's piece and its count beside it.
- **Counts:** the rules authority's Available Forces (`observation.available`), summed by faction and kind over every state. The ledger computes nothing else. It draws the committed position, so a hovered or pinned preview leaves it unchanged.
- **None left:** the cell stays. Its piece is drawn at 40% opacity beside "0", as the frame `zero state` draws it. No reachable position in this build empties an inventory, so the native app has not shown it; a unit test builds it from an observation.
- **Pointer:** the ledger's area takes the pointer: a press, a scroll or a drag on it neither moves the camera nor reaches a space or piece. Space tags and popovers keep off it, as they keep off every area.

## Files

- **Graphics source:** `atlas-available-forces.vectorcraft`, authored through the VectorCraft MCP from the study's `available-forces-graphics.vectorcraft`: G1's nine pieces, copied and scaled with their strokes. Each artboard is drawn at 6× the 1920×1280 reference and exported at 0.5, which is 3× the reference, as the top display's are ([TOP.md](TOP.md)). Each piece fills its artboard's height (the bases its width) and is centred in it.
- **Layout source:** `atlas-available-forces.figcraft`, authored through the FigCraft MCP from the study's `available-forces-layouts.figcraft`, with the other studies, the hover tag and the crop removed. Two frames:
  - **`16:9 available forces` (3840×2160):** the owner's display. The other areas are the study's locked context. The ledger's nodes are named as the runtime names its parts. `plate` groups the panel (fill `paper`, inside stroke `rule`, 2 wide, corner radius 10), the header rule (`ink`, 2 high) and the three row rules (`rule`, 1.5 high).
  - **`zero state`:** the Government's row with its Bases at 0: the piece at opacity 0.4 beside "0".
- **Palette and type:** the file's `Atlas` variables and two text styles. `Ledger/caption` is Lato Bold 24 with 3 tracking in `ink-secondary`, centred in the plate. `Ledger/count` is Lato Regular 43.875 (26 px at the reference, as the Resources ledger's values) in `ink`, left-aligned in a 56-wide box beside its piece. `atlas-available-forces.variables.json` is the `Atlas` export ([README.md, "Interface palette"](README.md#interface-palette)).
- **Layout record:** `atlas-available-forces.layout.json`: FigCraft's `absoluteBounds` of every part in `parts` (images) and `text`, in frame units, which are pixels of the owner's display; the `anchor`, top right; the `zero` opacity; and the `exports` with their sizes. `tolerance` is 2.
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

`area:available-forces` is the ledger's area. The development seat reports the ledger as `table.available_forces`: its `rows` by faction, with each cell's `kind` and `count`, and its drawn `parts` by their names in the record, each with its `bounds`, a count's `text` and whether a piece is `dimmed` ([seat reference](../../docs/agents/seat.md#what-the-table-reports)).

## Provenance and limits

- **Tools:** VectorCraft CLI 0.24.1 and FigCraft CLI 0.18.1, the binaries [art-tools.md](../../docs/agents/art-tools.md) records.
- **The Resources tag:** it opens directly left of the Resources ledger at its top ([TOP.md](TOP.md)), which is now this ledger's place, so while it is open it covers this ledger. #159 left it there; the owner judges it in #157's native review.
- **What this establishes:** placement and count evidence from the native regressions `native_available_forces_ledger_matches_layout_and_follows_a_removal` and `native_available_forces_follow_a_rally_and_not_its_preview`, with the main view's regressions for the area in other windows. It is not owner play acceptance: the owner judges the ledger's legibility on the display in #157's review.
