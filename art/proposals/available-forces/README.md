# Available Forces display studies

Decision-only alternatives from the owner's 2026-11-23 design interview. Spec [#157](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/157) was written from that interview and owns the work; this folder is its evidence, not a queue. The baseline is `85b4e9d`. No runtime code or admission manifest consumes these files, and selection is not native play acceptance. Foundation [4](../../../docs/foundation/04-art-and-assets.md) governs the round.

**Selected on 2026-11-23:** layout **B · Libro** and piece graphics **G1 · Sólido**. #157 specifies their implementation; nothing here is production art yet.

## Decided in the interview

- All four factions' Available Forces are always shown, and a faction's row opens the hover tag described under "Hover tag".
- Four texts that only state a count go: the victory tag's "Available: …" footer, the session panel's "29 map / 57 available", Rally's "12 Guerrillas stay in Available Forces" after "No pieces move.", and Rally's "With 12 Available Guerrillas" wherever Available is not the binding limit. Text that states a change stays, such as "This leaves 10 Guerrillas in Available Forces" and the removal messages.
- The session panel's "Terror/Sabotage 19 available" stays as text. That supply gets its own design round later, with the Cash supply.
- "Available Forces" joins the [glossary](../../../GLOSSARY.md). No ADR: the display is cheap to change.

Two statements are the agent's proposals, which the owner read and did not change: a kind at zero shows its piece dimmed beside "0", and the display shows the committed position while a pinned preview keeps stating the change in its text.

## What the rules say

Read from the March 2019 Living Rules ([foundation 2](../../../docs/foundation/02-rules-and-verification.md)), not from the implementation.

- **Nine inventories** (Available Forces chart, p. 20): Government 15 Troops, 15 Police and 2 Bases; 26July 15 Guerrillas and 4 Bases; Directorio 15 Guerrillas and 4 Bases; Syndicate 6 Guerrillas and 10 Casinos.
- **A hard limit on placing** (R 1.4.1): a piece is placed or replaced only from Available. With none of a kind left, an instruction to place is ignored and a replacement only removes. An action that would remove or replace a Base closes an open Casino instead and leaves a closed one alone (R 1.4.5). The limit binds Train (R 3.2.1), Rally (R 3.3.1), an Attack's or Ambush's captured Guerrilla (R 3.3.3, 4.3.2), Infiltrate (R 4.3.1), Construct (R 3.3.5), every Event (R 5.1.1) and a Cash Deposit taken as a Base or Casino (R 6.2.4).
- **Removal returns pieces** (R 1.4.1): a removed piece goes back to Available. While executing, a faction may return its own pieces, Casinos included; the rule's example is a Rally that returns Active Guerrillas in order to place them Underground.
- **No threshold.** No player rule rewards or penalises a count of Available pieces. The Non-Player flowcharts (R 8) test such counts, and the mission excludes them.
- **Bases on the map** are a victory input for 26July and Directorio, and open Casinos for the Syndicate (R 7.2). The physical board shows them through the holding box (R 1.4.1): Bases sit on the highest-numbered circles, so the highest uncovered number is the number on the map.
- **Public.** Forces not on the map are kept in the holding boxes (R 1.4.1), which are printed on the board. That every faction sees every other's is the board's arrangement, not a stated rule.

Terror and Sabotage markers share a supply of 20 (R 3.3.4) and Cash has 4 markers (R 4.5.2). They are markers, not forces, and are outside these studies.

## Sheets

All layouts are the selected main view ([MAIN-VIEW.md](../../atlas/MAIN-VIEW.md)) at 3840×2160 with one area added. The counts are the prepared scenario's: 3 Troops, 8 Police, 2 Bases; 12 Guerrillas, 3 Bases; 12 Guerrillas, 4 Bases; 6 Guerrillas, 7 Casinos. Sizes and distances below are pixels of the owner's display, which are the sheets' frame units. The sheets show today's session panel, with the "29 map / 57 available" line that the interview removes.

| Decision | Sheet | At the owner's display, 1:1 | Selected |
|---|---|---|---|
| Layout A · Fila | [layout-a-fila.png](layout-a-fila.png) | [layout-a-fila-top-1x.png](layout-a-fila-top-1x.png) | |
| Layout B · Libro | [layout-b-libro.png](layout-b-libro.png) | [layout-b-libro-top-1x.png](layout-b-libro-top-1x.png) | **B** |
| Layout C · Bandeja | [layout-c-bandeja.png](layout-c-bandeja.png) | [layout-c-bandeja-bottom-1x.png](layout-c-bandeja-bottom-1x.png) | |
| Piece graphics G1, G2, G3 | [graphics-pieces.png](graphics-pieces.png) | | **G1** |

The layouts and the graphics are independent: any family works in any layout. The layouts are drawn with G1.

### Layouts

| | Idea | Suits the task because | Trade-off |
|---|---|---|---|
| **A · Fila** | One strip between the standings and the Resources ledger, 1020×160: four groups under faction-coloured bars, each kind's piece over its count | The owner's suggested place. Reads left to right in the standings' order, and keeps every number on the top line | Fills the last open stretch of the top edge, 58 from the standings area and 58 from the ledger. A faction's forces and its Resources are read in two places |
| **B · Libro** | A second ledger left of the Resources ledger, 494×321, with the same four rows: emblem, then Troops or Guerrillas, Police, and Bases or Casinos in fixed columns | A faction's row reads across both ledgers: what it can place and what it can pay, directly above the decision panel. Smallest footprint, and Bases line up in one column | Repeats the four emblems. Two blocks of similar numbers sit side by side, told apart by their headers |
| **C · Bandeja** | Four holding boxes along the bottom, 1780×200, as the physical board has them: loose pieces with a count disc, and Bases on numbered circles | Follows the physical game, and the circles show Bases on the map as well as Bases Available | Largest area, over sea the camera can bring the island under. Far from the standings and Resources. The smallest lettering (20 px captions) |

In C the pieces would be the real solids in a screen-fixed tray, as the card piles are; the drawn pieces stand in for them here.

All three show every faction's nine counts at once. The alternative of showing them only in a hover tag is today's build, where they are the victory tag's footer.

### Piece graphics

| | Idea | Suits the task because | Trade-off |
|---|---|---|---|
| **G1 · Sólido** | Shaded solids in the map pieces' colours: cube, upright cylinder, low disc; the Casino carries a ring | The owner already reads these shapes and colours on the map, so the ledger needs no key | Kind is told by shape alone, and Troops from Police by their blue. The pale Police cube and the yellow pieces have the least contrast on the panel. Each cell carries the shape and nothing more |
| **G2 · Sello** | Flat stamps with an initial, like the victory emblems | The initial names the kind outright, so it reads at the smallest size and without colour | Looks least like the pieces. The letter is a second thing to read beside every count, which makes it the densest of the three |
| **G3 · Tinta** | The same solids as inked line drawings with flat colour | The ink outline holds each piece's edge on the pale panel, in the manner of the drawn Aid crate and the strongbox | The outline thickens at small sizes and darkens the row. No face is shaded, so the solids read flatter than the map's |

### Hover tag

[hover-tag.png](hover-tag.png) shows what a faction's row opens: each kind's count of its inventory, the number on the map, and the rule. It reuses the accepted top-display tag ([TOP.md](../../atlas/TOP.md)), so it is not a new design and carries no choice.

## Files and provenance

- **Graphics source:** `available-forces-graphics.vectorcraft`, authored through the VectorCraft MCP (CLI 0.24.1). One 1800×1000 artboard; `graphics-pieces.png` is its export at 2 px per point. The nine `g1-*.png` are G1's pieces exported one by one at the same scale, for the layouts to place.
- **Layout source:** `available-forces-layouts.figcraft`, a copy of `atlas-main-view.figcraft` authored through the FigCraft MCP (CLI 0.18.1). Frames `A · Fila`, `B · Libro` and `C · Bandeja` each add nodes named `available/…`; a `crop/…` slice in each gives the 1:1 sheet. The card piles are the layout's placeholders: no private face or back is embedded.
- **Not established:** legibility on the owner's display, behaviour at a count of zero, and any native rendering.
