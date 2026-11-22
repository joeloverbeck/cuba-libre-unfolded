# Main view

The owner selected main view layout **A · Mesa** on 2026-11-22 ([decision record](../proposals/main-view-cards/README.md), sheet [`layout-a-mesa.png`](../proposals/main-view-cards/layout-a-mesa.png); owner's words in [#136](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/136)). #142 built its first part for spec #137: the map fills the window and the accepted areas lie over it. #143 builds the card piles it also places.

This layout is the authority for where every area of the main view sits ([foundation 4](../../docs/foundation/04-art-and-assets.md), "Layout authority"). It replaces the VectorCraft-authored Atlas chrome around the map: the desk strips, the map's header and footer, and with them the "CUBA" title, the terrain key and the line "Borders carry adjacency; Terror needs no movement". The space hover tag names the terrain instead ([STATUS.md](STATUS.md)).

## What it shows

- **Map:** the 3D map is the whole window. The sea is the layout's `map` colour, which the authored water frames also have.
- **Areas over the map:** the top display's two groups, the victory standings, the Sequence of Play track, the decision panel, the session panel and the card area. They are the accepted designs, moved and not redesigned: each has the size and the place on the owner's display that it had in the framed view. Their own specifications still own what is inside them ([TOP.md](TOP.md), [VICTORY.md](VICTORY.md), [ELIGIBILITY.md](ELIGIBILITY.md), [CHOICE.md](CHOICE.md), [EXECUTION.md](EXECUTION.md), [KIDNAP.md](KIDNAP.md), [TRANSFER.md](TRANSFER.md), [CARD-TAG.md](CARD-TAG.md)).
- **Pointer:** an area takes the pointer over its own rectangle. A press, a scroll or a drag there neither moves the camera nor reaches the space or piece beneath. Everywhere else the window belongs to the map: orbit, zoom, pan and Home work over all of it.
- **Home:** all thirteen spaces lie inside the layout's island box, as large as the box allows and centred in it, so no space starts under an area.
- **Tags and popovers:** a space tag opens anywhere on the map and keeps off every area. A space or piece popover opens in the part of the window the areas at its edges leave clear: below the standings, left of the decision panel and above the card area.
- **Cards:** until #143, the card panel stays where it was, over the sea at the bottom left. The layout already draws what replaces it: the played pile and the draw deck with their labels, count discs and captions, and four Propaganda pips.

## Files

- **Layout source:** `atlas-main-view.figcraft`, authored in FigCraft CLI 0.18.1 from sheet A, whose frame it keeps. Its one frame, `16:9 main view`, is 3840×2160: the owner's display, where one reference pixel is 1.6875 frame units.
  - **Accepted areas:** the top display's and the standings' graphics are the runtime exports, placed at their runtime rectangles and named as the runtime names them. The Sequence of Play track, the decision panel and the session panel are locked crops of one native capture (card 17, Directorio's shaded Event pending), as on sheet A, with the parts the runtime draws at fixed places marked by named rectangles over them. No crop includes a card face.
  - **`area/<name>`:** seven unfilled rectangles, one for each area: the rectangle that takes the pointer.
  - **`island`:** the unfilled box the island fills at Home, drawn round the island of sheet A.
  - **Piles:** `played-pile` and `draw-deck` (300×430 each, 60 apart), `played-label` and `deck-label`, a count disc and its number on each pile, a caption under each, and `propaganda-label`, `propaganda-pip-0`…`3` and `propaganda-caption`. The cards are sheet A's placeholders; no private face or back is embedded.
  - **Hidden layers:** `card-frame`, the interim card panel, and `card-tag`, where the card tag opens once the piles replace the panel: above the piles, its left edge on the played pile's and its foot 16 above the labels. #143 places the tag and may move it.
  - `atlas-main-view-layout.png` is the frame's export at full size.
- **Layout record:** `atlas-main-view.layout.json` holds FigCraft's `absoluteBounds` in frame units.
  - `areas`: each area's rectangle and its `anchor`.
  - `island`: the Home box.
  - `parts` and `text`: the accepted areas' images and text whose places do not follow the match's state.
  - `interim`: the card panel's frame.
  - `planned`: the piles, labels, discs, captions and pips, and the card tag. Nothing draws them yet.
  - `tolerance`: 2 frame units, which are pixels on the owner's display.
- **Manifest:** `main-view.sha256` admits the layout record and the three area frames the layout places (`atlas-layout-operation.png`, `atlas-layout-session.png` and, until #143, `atlas-layout-card.png`). A missing or changed file refuses launch before any save is created or replaced, like the other presentation manifests. The layout is stateless presentation, so it does not change a save's identity. It replaces `layout.sha256`.
- **Palette:** `atlas-main-view.variables.json` is the file's `Atlas` export ([README.md, "Interface palette"](README.md#interface-palette)).

The application loads the record: the areas' rectangles and the island box come from it at launch. What each area draws inside its rectangle is implemented against its own layout, and the native regression holds both to this record.

## Areas

Frame rectangles are x, y, width, height on the owner's display. The reference rectangle is the same place in reference pixels, measured from the edge the area is anchored to.

| Area | Frame rectangle | Anchor | Reference rectangle | Holds |
|---|---|---|---|---|
| `outside-influence` | 67.5, 30.38, 624.38×108 | top left | 40, 18, 370×64 | Aid and the US Alliance track |
| `standings` | 1528.5, 13.5, 783×182.25 | top, centred | 728, 8, 464×108 of a centred 1920 | The four victory emblems and margins |
| `resources` | 3448.5, 33.75, 324×415.13 | top right | 1688, 20, 192×246 | The Resources ledger and the Cash supply |
| `sequence-of-play` | 101.25, 300, 391.5×647.94 | top left | 60, 177.8, 232×384 | The Sequence of Play track at its tallest |
| `decision` | 2928.75, 472.5, 843.75×1113.75 | top right | 1380, 280, 500×660 | The decision panel, with the Kidnap die tray |
| `session` | 2928.75, 1626.75, 843.75×445.5 | bottom right | 1380, 964, 500×264 | The session panel |
| `cards` | 67.5, 1626.75, 1012.5×445.5 | bottom left | 40, 964, 600×264 | The card panel until #143, then the piles |

The island box is 620, 470, 2230×1104. The transfer modal stays in the middle of the window, over its scrim.

## Other window sizes

The layout is authored and compared at 16:9 only, as foundation 4 requires. In any window:

- **Sizes** follow the window alone ([ADR 0002](../../docs/adr/0002-sizes-follow-the-window-not-the-desktop-scale.md)): one reference pixel is the smaller of the window's width over 1920 and its height over 1280.
- **Places:** each area keeps its distance, in reference pixels, from the edges its anchor names. Nothing is letterboxed: in a window taller than 3:2 the session panel and the card area lie against the bottom edge, where the framed view centred everything between two strips of desk.
- **Island box:** its top-left corner keeps its distance from the window's top-left corner, and its bottom-right corner from the bottom-right corner. Home fits the island into it, so the island is smaller in a 3:2 window and no larger in a 21:9 one, and no space lies under an area in either.
- **Camera:** a camera resting at Home follows a change of the window's size. One the player has moved stays, and Home returns it to the new framing.

## Pointer and seat

Each area is a transparent interface rectangle beneath its own controls. The shared table camera leaves the pointer to any interface node it is over, and picking stops at it, so nothing else is needed to keep input from the map. The development seat names the rectangles `area:<name>` and reports the main view as `table.main_view` ([seat reference](../../docs/agents/seat.md)).

## Provenance and limits

- **Tools:** FigCraft CLI 0.18.1, rebuilt from `storytold/figcraft@559c952d`, as [art-tools.md](../../docs/agents/art-tools.md) records. The source is sheet A's document with the other sheets removed, the top display's `Atlas` variables imported, and the parts placed at the runtime's rectangles and named.
- **Measured before the change:** on the build before #142, at 3840×2160, the seat reported every part named in the record within 1 px of the rectangle the record now holds; the difference is the renderer's rounding to whole pixels.
- **Retired:** `atlas-layout-background.png`, `atlas-layout-map-fill.png`, `atlas-layout-map-header.png` and `atlas-layout-map-footer.png` are deleted, and `layout.sha256` with them. Their artboards remain in `atlas-layout.vectorcraft` ([LAYOUT.md](LAYOUT.md)).
- **3D look:** the frame is a flat drawing. The island in it is a capture of the map at its Home pose; the piles' perspective, thickness and shadow are 3D in production (#143).
- **Stage:** [`art/stage/preview-stage.blend`](../stage/README.md) mirrors Home on the owner's display.
- **What this establishes:** placement evidence and the native regressions `native_main_view_fills_the_window_and_matches_its_layout` and `native_main_view_keeps_areas_and_picks_in_other_windows`. It is not owner play acceptance: the owner judges the main view on the display at spec #137's review.
