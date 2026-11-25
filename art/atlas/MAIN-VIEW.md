# Main view

The owner selected main view layout **A · Mesa** on 2026-11-22 ([decision record](../proposals/main-view-cards/README.md), sheet [`layout-a-mesa.png`](../proposals/main-view-cards/layout-a-mesa.png); owner's words in [#136](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/136)). #142 built its first part for spec #137: the map fills the window and the accepted areas lie over it. #143 built the card piles it also places.

This layout is the authority for where every area of the main view sits ([foundation 4](../../docs/foundation/04-art-and-assets.md), "Layout authority"). It replaces the VectorCraft-authored Atlas chrome around the map: the desk strips, the map's header and footer, and with them the "CUBA" title, the terrain key and the line "Borders carry adjacency; Terror needs no movement". The space hover tag names the terrain instead ([STATUS.md](STATUS.md)).

## What it shows

- **Map:** the 3D map is the whole window. The sea is the layout's `map` colour, which the authored water frames also have.
- **Areas over the map:** the top display's two groups, the victory standings, the Sequence of Play track, the decision panel and the card area, since #159 the Available Forces ledger ([AVAILABLE.md](AVAILABLE.md)), since #172 the marker supply ([TOP.md](TOP.md#the-marker-supply)), since #174 Save · quit, with the notice slip above it while a notice is drawn (["Save · quit and the notice slip"](#save--quit-and-the-notice-slip)), and since #206 the seat cards ([SEATS.md](SEATS.md)), and since #208 the pile of editions beside the Propaganda pips while it holds one ([NEWSPAPER.md](NEWSPAPER.md)). The session panel is gone. The others are the accepted designs, moved and not redesigned: each has the size and the place on the owner's display that it had in the framed view. Their own specifications still own what is inside them ([TOP.md](TOP.md), [VICTORY.md](VICTORY.md), [ELIGIBILITY.md](ELIGIBILITY.md), [CHOICE.md](CHOICE.md), [EXECUTION.md](EXECUTION.md), [KIDNAP.md](KIDNAP.md), [TRANSFER.md](TRANSFER.md), [CARD-TAG.md](CARD-TAG.md)).
- **Pointer:** an area takes the pointer over its own rectangle. A press, a scroll or a drag there neither moves the camera nor reaches the space or piece beneath. Everywhere else the window belongs to the map: orbit, zoom, pan and Home work over all of it.
- **Home:** all thirteen spaces lie inside the layout's island box, as large as the box allows and centred in it, so no space starts under an area.
- **Tags and popovers:** a space tag opens anywhere on the map and keeps off every area. A space or piece popover opens in the part of the window the areas at its edges leave clear: below the standings, left of the decision panel and above the card area.
- **Cards:** two piles lie bare on the sea at the bottom left, as the physical game lays them out (R 2.2, 2.3.8). ["The card piles"](#the-card-piles) describes them. The card panel, its frame and its line "17 current · 18 preview · 34 cards in draw deck" are gone.

## Files

- **Layout source:** `atlas-main-view.figcraft`, authored in FigCraft CLI 0.18.1 from sheet A, whose frame it keeps. Its one frame, `16:9 main view`, is 3840×2160: the owner's display, where one reference pixel is 1.6875 frame units.
  - **Accepted areas:** the top display's and the standings' graphics are the runtime exports, placed at their runtime rectangles and named as the runtime names them. The Sequence of Play track and the decision panel are locked crops of one native capture (card 17, Directorio's shaded Event pending), as on sheet A, with the parts the runtime draws at fixed places marked by named rectangles over them. No crop includes a card face.
  - **`area/<name>`:** twelve unfilled rectangles, one for each area: the rectangle that takes the pointer.
  - **Save · quit and the notice slip** (#174): `save-quit`, the plate's at-rest export, on `area/save-quit`; `notice-slip`, a hidden frame drawn with a two-line notice (the space popover's card in `paper` with a 1-unit `ink` stroke and its shadow, the `action/primary` rule `notice-rule`, and the words `notice-text`, Lato Regular 21.09, which is the popover's `Popover/Regular 12.5`, in `ink`); and `save-quit-tag`, a hidden rectangle where the plate's tag opens. Each colour is bound to its `Atlas` variable. They replace the session panel's crop, its frame, its status text, its button and its area.
  - **Marker supply** (#172): `marker-supply`, the marker's at-rest export, and the texts `marker-supply-count` and `marker-supply-of`, placed as layout A · Franja places them ([TOP.md](TOP.md#the-marker-supply)).
  - **`island`:** the unfilled box the island fills at Home, drawn round the island of sheet A.
  - **Piles:** `played-pile` and `draw-deck` (300×430 each, 60 apart) are the top cards. Under each, `played-pile-base` and `deck-pile-base` are where the pile's foot shows on card 17, with the stack drawn between. `played-label` and `deck-label`, a count disc and its number on each pile, a caption under each, and `propaganda-label`, `propaganda-pip-0`…`3` and `propaganda-caption` lie over them. The cards are sheet A's placeholders; no private face or back is embedded.
  - **`card-tray`:** the unfilled rectangle the tray's camera draws, 60, 1480, 760×660. It is larger than the piles so that a card which lifts or turns stays inside it (#145).
  - **`card-tag`:** a hidden rectangle where the card tag opens: above the piles, its left edge on the played pile's and its foot 16 above the labels ([CARD-TAG.md](CARD-TAG.md)).
  - **`played-pile-tag`, `deck-pile-tag`:** hidden rectangles where each pile's tag opens on card 17. The tags' own frames lie beside this one ([PILE-TAGS.md](PILE-TAGS.md)).
  - **`pile-shadow`:** outside the frame, a 300×430 rectangle in `ink` at 31%, blurred by 26: the soft shadow sheet A draws under each pile.
  - `atlas-main-view-layout.png` is the frame's export at full size.
- **Layout record:** `atlas-main-view.layout.json` holds FigCraft's `absoluteBounds` in frame units.
  - `areas`: each area's rectangle and its `anchor`.
  - `island`: the Home box.
  - `parts` and `text`: the accepted areas' images and text whose places do not follow the match's state.
  - `tray`: the card tray's rectangle and anchor, both top cards, and both piles' feet on card 17.
  - `card-tag`: where the card tag opens.
  - `notice-slip`: the slip's rectangle with a two-line notice and the place of its words, anchored bottom-right; `save-quit-tag`: where Save · quit's tag opens, anchored bottom-right (#174).
  - `exports`: each node exported as an image, its file and its size in pixels.
  - `tolerance`: 2 frame units, which are pixels on the owner's display.
- **Exports:** FigCraft exports of the frame's own nodes: `atlas-main-view-count-disc.png` (`played-count-disc`, at 2 px per unit), `atlas-main-view-pip-played.png` and `atlas-main-view-pip.png` (`propaganda-pip-0` and `-1`, at 2 px per unit), and `atlas-main-view-pile-shadow.png` (`pile-shadow`, 378×508 with its blur). The numbers and words are runtime Lato.
- **Manifest:** `main-view.sha256` admits the layout record, the area frame the layout places (`atlas-layout-operation.png`) and the four exports above. #174 retired the session panel's frame, `atlas-layout-session.png`; `save-quit.sha256` admits Save · quit's two exports. A missing or changed file refuses launch before any save is created or replaced, like every other manifest. For saves, see [launch admission](README.md#launch-admission). It replaces `layout.sha256`.
- **Palette:** `atlas-main-view.variables.json` is the file's `Atlas` export ([README.md, "Interface palette"](README.md#interface-palette)). Since #172 the collection holds `action/primary`, and the file has two text styles, `Supply/count` and `Supply/of`, for the marker supply's number and "of 20" ([TOP.md](TOP.md#the-marker-supply)).

The application loads the record: the areas' rectangles and the island box come from it at launch. What each area draws inside its rectangle is implemented against its own layout, and the native regression holds both to this record. The card tray's rectangles are constants in `src/tabletop/piles.rs`, which a unit test holds equal to the record.

## Areas

Frame rectangles are x, y, width, height on the owner's display. The reference rectangle is the same place in reference pixels, measured from the edge the area is anchored to.

| Area | Frame rectangle | Anchor | Reference rectangle | Holds |
|---|---|---|---|---|
| `outside-influence` | 67.5, 30.38, 624.38×108 | top left | 40, 18, 370×64 | Aid and the US Alliance track |
| `standings` | 1528.5, 13.5, 783×182.25 | top, centred | 728, 8, 464×108 of a centred 1920 | The four victory emblems and margins |
| `resources` | 3448.5, 33.75, 324×415.13 | top right | 1688, 20, 192×246 | The Resources ledger and the Cash supply |
| `available-forces` | 2928.75, 33.75, 494×320.63 | top right | 1380, 20, 292.74×190 | The Available Forces ledger (#159) |
| `marker-supply` | 964.69, 30.38, 291×108 | top, between `outside-influence` and `standings` | 172.44×64, 18 from the top, centred between the two | The marker supply (#172) |
| `sequence-of-play` | 101.25, 300, 391.5×647.94 | top left | 60, 177.8, 232×384 | The Sequence of Play track at its tallest |
| `decision` | 2928.75, 472.5, 843.75×1113.75 | top right | 1380, 280, 500×660 | The decision panel, with the Kidnap die tray |
| `save-quit` | 3516, 2021.63, 256.5×70.88 | bottom right | 1728, 1198, 152×42 | Save · quit (#174) |
| `cards` | 67.5, 1594, 734.5×536 | bottom left | 40, 944.6, 435.3×317.6 | The two piles, their count discs and captions |
| `propaganda` | 834, 1920, 226×160 | bottom left | 494.2, 1137.8, 133.9×94.8 | The Propaganda pips |
| `seats` | 1500, 1896, 1940×226 | bottom, between `propaganda` and `save-quit`: its right edge 76 units from Save · quit; narrower windows shrink it toward that corner | 1149.6×133.9 | The four seat cards (#206) |
| `editions` | 1120, 1800, 316×234 | bottom left, beside `propaganda` while that keeps 24 from `seats`, else 24 above `propaganda` on its left edge; there only with the pile | 187.3×138.7 | The pile of the card's editions ([NEWSPAPER.md](NEWSPAPER.md#the-pile), #208) |

The island box is 620, 470, 2230×1104. The transfer modal stays in the middle of the window, over its scrim. The piles' two labels stand just above the `cards` area, on open sea beside the island box.

## The card piles

The cards lie as the physical game lays them out (R 2.2, 2.3.8): the **played pile** on the left with the current card face up on top, and the **draw deck** on the right with the preview card face up on top. They are square to each other and 60 apart.

- **Solid cards.** Each pile is the shared round-cornered card solid (`unfolded_tabletop::card`), drawn in a tray of its own: a camera on its own render layer, as the Kidnap die's tray has. The tray is fixed to the screen. It draws after the map and before the interface, so the count discs and every tag lie over the cards at rest, and the map camera moves beneath it. Its camera is marked as another view (`unfolded_tabletop::label_anchor::OtherView`), so the piece counts and the popovers keep following the map's camera.
- **The view** is the layout's: a top card lies flat on its 300×430 rectangle, and its pile's side shows below it. The camera looks straight down from in front of the piles and draws only the part of its picture that holds them, as a shifted lens does. A face parallel to the table keeps its shape, and half of a pile's height shows as its near side, as it did in the frames the owner selected the pile edge from (#140), which looked from 60° above the table.
- **Height.** A pile is as tall as its cards at a real card's proportions: 0.3 mm of a 63 mm card, 300 / 210 of a unit for each card seen edge-on, half of which shows. On card 17 the played pile's 18 cards show 12.86 of side and the draw deck's 34 show 24.29; a card played moves 0.71 from one to the other.
- **Seams.** A pile's side shows a seam every six to seven cards, the owner's selection in #140: the solid is drawn in as many layers as the count divided by 7, rounded, and at least 2 (3 for 18 or 19 cards, 5 for 33 or 34, 7 for 48 or 52). The seams are the shared solid's grooves, lit from above and in front; no texture draws them. A pile of one card is drawn in one layer: a seam marks where two cards meet, and one card has none. An empty pile draws nothing.
- **Faces.** The top card is the admitted private face of the model's current or preview card, drawn unlit so that it keeps its printed colours. The cards' corner radius is 5.6% of their width, which cuts away the black printed outside the faces' corners ([CARD-TAG.md](CARD-TAG.md), "Limits"). Only cards 17, 18 and 19 are admitted. At rest no card's back shows.
- **Shadow.** `pile-shadow` lies on the table under each pile's foot, 12 toward the viewer, as sheet A draws it.
- **Numbers.** Each pile's disc gives the rules model's count (`observation.piles`): the played pile counts the current card and the draw deck the preview, so at rest the two account for all 52 cards. A filled pip stands for each Propaganda card played, of four, and the caption says "n of 4 played". The order of the cards under the preview is in neither the observation nor the tray.
- **Targets.** Each top card is the hover, focus and click target of its card's tag. The rest of each pile is its pile's target (`pile:played`, `pile:deck`): its count disc, its visible side and its caption, down to the caption's foot. Its hover and focus open the [pile tag](PILE-TAGS.md).
- **At rest.** The tray shows the saved match's cards at rest, except while a card the running app has just played is shown moving ("The card advance" below).
- **Fixed places.** The discs and captions stand where the layout draws them for card 17. Within this build's reach (18 or 19 and 33 or 34 cards) a pile's foot moves by under a unit, so they do not follow it.

## The card advance

When a card is played (R 2.3.8) the tray shows it happen, in the four steps of [the advance sheet](../proposals/main-view-cards/pile-tags-and-advance.png). Lengths are in the layout's units, as above.

1. **At rest** on the card being left.
2. **The crossing.** The preview lifts off the draw deck and crosses to the played pile in 0.8 s, level and face up. At the middle of its way it is 30 mm above the piles, which the tray's view shows 74 toward the top of the screen. The deck beneath shows the back of its next card.
3. **The turn.** When it has landed, the deck's top card turns over about its length in 0.7 s, where it lies: its back shows until it stands on its edge, and its face after. It rises half its width, so the long edge it turns on stays on the pile and never dips into it.
4. **At rest** on the new current card and preview.

- **Saved first.** The advance is committed and saved before anything moves, as [ADR 0001](../../docs/adr/0001-baked-die-tumbles-land-on-the-committed-face.md) does for the die. The motion is presentation over the saved cards and saves nothing.
- **Shared motion.** Both moves are `unfolded_tabletop::card_motion::CardMove`: the crossing a lifted move between the two piles, the turn `Turn::over` about the card's length with a lift. Each is read at the time since the commit on the shared clock, so the seat's fixed steps always find the same pose. The tray holds the numbers above and no motion of its own.
- **Drawn toward the viewer.** Upright, the turning card's upper edge stands a card's width above the pile, which the tray's view shows 160 higher than the pile's top: 30 above the tray, which leaves 130. So the turn carries the card 45 toward the viewer at its middle and back (the shared move's `Departure`), as a hand turning a deck's top card draws it. Its upper edge then comes within 16 of the tray's top. Making the tray taller in the layout would let the card turn without it.
- **The card in the air** is one card of the shared solid with its face on one side and the back on the other, both unlit. `pile-shadow` lies under it, cast straight down: as wide as the card is seen from above, and darker as the card rises, from nothing on a pile to a pile's own shadow at the crossing's full rise.
- **Discs wait.** Neither count disc, nor its number, shows while a card is in the air: the crossing card and the turning card pass where the discs lie, and looked as if they cut through them (the owner's native review, [#146](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/146#issuecomment-7099603365)). Both are back as the cards land or are landed, reading the new counts.
- **Pips follow the cards.** The Propaganda pips change as the played card lands. The piles' thickness follows the cards too: the deck is a card thinner as the played card leaves it, the played pile a card thicker as it lands, and the turning card is still one of the deck's.
- **Tags wait.** A [pile tag](PILE-TAGS.md) and a [card tag](CARD-TAG.md) state the model's piles and cards, which are a card ahead of the table's while a card is in the air. None shows then: an open one closes as the card leaves and is back as the cards land, with the numbers the discs and the pips then read and the card then lying there. A pinned card tag stays pinned.
- **Landing at once.** A click anywhere, Space or Enter lands the cards at rest, and that input does nothing else.
- **No replay.** Nothing of the motion is saved. A restart during it and a resumed save find the cards at rest, and the advance made once. Under the development seat the clock moves only with the seat's own steps, so a seat that connects during the motion finds the card where it was left, and `settle` lands it.
- **The back** is the original shared card back, `cuba-libre-card-back.png`, admitted from the private card directory by its hash ([Private card admission](README.md#private-card-admission)). The tray draws a copy made smaller once at launch, 600×860 (twice the card's size here), so that drawing it does not skip four of every five of the scan's printed dots. The scan's corners are transparent, not printed dark: 17 px in along each diagonal. The cards' 5.6% corner cuts 24 px in, so none of that shows.
- **Above the labels.** A card in the air is above the table, so a second camera with the tray's view draws it, and only it, after the interface: it passes over the pile labels and the captions, as the advance sheet's step 2 draws the lifted card above the table. That camera is on only while a card is in the air. The card's shadow stays on the table, under them.

## Save · quit and the notice slip

The owner selected button **S2 · Etiqueta** and its place in layout **A · Franja** on 2026-11-24 ([decision record](../proposals/last-ui/README.md), sheets [`graphics-save-quit.png`](../proposals/last-ui/graphics-save-quit.png) and [`layout-a-franja-corner-1x.png`](../proposals/last-ui/layout-a-franja-corner-1x.png)). #174 built them and retired the session panel, its "Saved revision" line and its marker supply line.

- **Save · quit** is a paper plate with the door-and-arrow glyph and "SAVE · QUIT" in the plates' capitals, in the `save-quit` area: 152×42 reference pixels, 40 from the right and bottom edges in every window. Under the pointer or keyboard focus it shows its hovered export, an `ink` stroke and an `action/primary` top rule. Keyboard focus also puts the [focus mark](FOCUS-MARK.md) round it. It closes the game; every decision is already saved, and the same save resumes the match.
- **Its tag** is the top display's tag ([TOP.md](TOP.md)) titled "Save · quit" with one sentence: "Every decision is saved as it is made, and this button closes the game." It opens above the plate and the seat cards, its right edge on the window's 40 margin and its foot 82.67 above the plate, where `save-quit-tag` is, after the shared dwell.
- **The notice slip** shows a refusal, an error, or the loading state before the match opens, and nothing else: a decision that succeeded is not drawn, since its result is on the map, the ledgers and the Sequence of Play track. It reuses the space popover's card plate and top rule at the popover's width, 340, with its `Popover/Regular 12.5` text and inline emblems. The card is its own export of that plate, `atlas-popover-card-slip.png` ([HIGHLIGHTS.md](HIGHLIGHTS.md#production-source-and-plates-58-review) gives its construction): on the popover's export a short card draws its border outside the card ([#178](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/178) holds the popover's own short cards). It stands 88.67 above the plate, over the seat cards, its right edge the plate's, and grows upward with its lines; in every window it clears the cards, whichever stands forward, and, with up to nine lines, the decision panel.
- **Raised for the seats:** #174 stood the slip 14 and the tag 8 above the plate, where Syndicate's card, accepted after them, came to lie beneath both. The owner ruled on 2026-11-25 that both move up 126 px at 3840×2160 (74.67 reference pixels, scaled with the window), so that neither overlaps a seat card ([ruling](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/206#issuecomment-7173989870), #206). The plate, their sizes, words and right edges are as they were.
- **Pointer:** the plate and the slip each take the pointer over their own rectangle. A click on the slip dismisses it; the next notice replaces it. The loading state and a launch error stay until the match opens.
- **Kinds:** each notice is written with its kind (success, refusal, error or loading); nothing reads it from the words. The development seat reads every notice, successes included, as before ([seat reference](../../docs/agents/seat.md#what-the-table-reports)).

**Files**

- **Graphics source:** `atlas-save-quit.vectorcraft`, authored through the VectorCraft MCP from the study's `last-ui-graphics.vectorcraft`: S2's drawings at rest and hovered, kept at reference size (the plate 150×40 with its 1.25 stroke, the glyph and the live Lato type). Artboards `save-quit` and `save-quit-hover` are 152×42 and are exported at 3 px per unit: `atlas-save-quit.png` and `atlas-save-quit-hover.png`, 456×126.
- **Manifest:** `save-quit.sha256` admits both exports. A missing or changed file refuses launch before any save is created or replaced.
- **Layout:** `atlas-main-view.figcraft` places the plate, the slip and the tag's place (["Files"](#files)).

| Part | From | Frame rectangle on the owner's display |
|---|---|---|
| `save-quit` | `atlas-save-quit.png`, or `atlas-save-quit-hover.png` | 3516, 2021.63, 256.5×70.88 |
| `notice-slip` | `atlas-popover-card-slip.png` and `atlas-popover-rule-primary.png` ([HIGHLIGHTS.md](HIGHLIGHTS.md#production-source-and-plates-58-review)), runtime Lato | 3198.75, 1779.19, 573.75×92.81 with two lines; right edge, foot and width fixed |
| `save-quit-tag` | `atlas-top-tag.png` ([TOP.md](TOP.md)) | 2982.75, 1632.38, 789.75×249.75 with two note lines; right edge and foot fixed |

**Old button plates.** `atlas-button.png` and `atlas-button-primary.png` stay admitted by `runtime.sha256`, as #174 asked. Since #174 nothing draws either: the session panel's button was the only caller of the code that drew them, and the primary plate was already drawn by nothing.

**Tools:** VectorCraft CLI 0.24.1 and FigCraft CLI 0.18.1, the builds installed on 2026-11-22 from clean checkouts of `storytold/vectorcraft@88f5ece1` and `storytold/figcraft@5c2a3784` ([TOP.md, "The marker supply"](TOP.md#the-marker-supply) gives the same reflog and timestamp evidence; it is not a recorded build).

**What this establishes:** placement and behaviour evidence from the native regressions `native_save_quit_and_the_notice_slip_match_layout_and_resume_the_save` and `native_the_slip_and_save_quits_tag_clear_the_seat_cards`, and the main view's regressions. It is not owner play acceptance.

## Other window sizes

The layout is authored and compared at 16:9 only, as foundation 4 requires. In any window:

- **Sizes** follow the window alone ([ADR 0002](../../docs/adr/0002-sizes-follow-the-window-not-the-desktop-scale.md)): one reference pixel is the smaller of the window's width over 1920 and its height over 1280.
- **Places:** each area keeps its distance, in reference pixels, from the edges its anchor names. The marker supply keeps its distance from the top and is centred between the right edge of `outside-influence` and the left edge of `standings`, which move apart as the window widens. Nothing is letterboxed: in a window taller than 3:2 Save · quit, the notice slip and the card area lie against the bottom edge, where the framed view centred everything between two strips of desk.
- **Island box:** its top-left corner keeps its distance from the window's top-left corner, and its bottom-right corner from the bottom-right corner. Home fits the island into it, so the island is smaller in a 3:2 window and no larger in a 21:9 one, and no space lies under an area in either.
- **Camera:** a camera resting at Home follows a change of the window's size. One the player has moved stays, and Home returns it to the new framing.

## Pointer and seat

Each area is a transparent interface rectangle beneath its own controls; `editions` is there while its pile is. The shared table camera leaves the pointer to any interface node it is over, and picking stops at it, so nothing else is needed to keep input from the map. The development seat names the rectangles `area:<name>` and reports the main view as `table.main_view` ([seat reference](../../docs/agents/seat.md)).

## Provenance and limits

- **Tools:** FigCraft CLI 0.18.1, rebuilt from `storytold/figcraft@559c952d`, as [art-tools.md](../../docs/agents/art-tools.md) records. Its re-saves on 2026-11-23, for #159's `available-forces` area, and on 2026-11-24, for #172's marker supply and #174's Save · quit and notice slip, and on 2026-11-25, for the areas of #206 and #208 and the raised slip and tag, used the build installed on 2026-11-22 from `storytold/figcraft@5c2a3784`; as for [AVAILABLE.md](AVAILABLE.md#provenance-and-limits), this is reflog and timestamp evidence, not a recorded build. The source is sheet A's document with the other sheets removed, the top display's `Atlas` variables imported, and the parts placed at the runtime's rectangles and named.
- **Measured before the change:** on the build before #142, at 3840×2160, the seat reported every part named in the record within 1 px of the rectangle the record now holds; the difference is the renderer's rounding to whole pixels.
- **Retired:** `atlas-layout-background.png`, `atlas-layout-map-fill.png`, `atlas-layout-map-header.png` and `atlas-layout-map-footer.png` are deleted, and `layout.sha256` with them; #143 deleted `atlas-layout-card.png`, the card panel's frame. Their artboards remain in `atlas-layout.vectorcraft` ([LAYOUT.md](LAYOUT.md)).
- **3D look:** the frame is a flat drawing. The island in it is a capture of the map at its Home pose. The piles in it are flat rectangles stepped down to each pile's foot; in production they are solid cards, and the record holds only their top cards, their feet and what lies over them.
- **Stage:** [`art/stage/preview-stage.blend`](../stage/README.md) mirrors Home on the owner's display.
- **What this establishes:** placement evidence and the native regressions `native_main_view_fills_the_window_and_matches_its_layout`, `native_main_view_keeps_areas_and_picks_in_other_windows` and `native_card_piles_match_layout_and_follow_the_match`. It is not owner play acceptance: the owner judges the main view on the display at spec #137's review.
