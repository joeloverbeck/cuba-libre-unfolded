# Top display

The owner selected these in [#46](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/46#issuecomment-6926405423):
- **Graphics:** Cash B (banknote bundle), Aid B (supply crate), US Alliance A (eagle marker on three boxes) and Resources C (strongbox).
- **Layout:** A in general, but with C's Resources ledger and C's Cash placement.

They also asked for three changes:
- remove the "CUBA LIBRE" title and put Aid and the US Alliance in the top-left corner;
- wrap the Cash band around the bundle in perspective;
- redraw the eagle as a respectable heraldic eagle.

The selection studies remain in [`../proposals/issue-46/`](../proposals/issue-46/README.md).

## What it shows

- **Top left (outside influence):** the Aid crate with its value, and the US Alliance track, with the eagle marker on the current level.
- **Between it and the standings:** the marker supply (#172, ["The marker supply"](#the-marker-supply)).
- **Top right:** a Resources ledger with one row per faction (its accepted emblem and value), and below it the Cash supply: four slots, filled while that marker is Available and an empty dashed ring while it is on the map.
- **Hover tags:** each of the four elements opens a tag with the shared emblem/region dwell. The Resources tag opens beside the ledger, so the Cash supply stays visible. The Available Forces ledger's rows open the same tag, at their own place ([AVAILABLE.md](AVAILABLE.md#the-tag)), and so does the marker supply, below it (["Its tag"](#its-tag)). So does Save · quit in the bottom-right corner, with no rows and one sentence, above the plate ([MAIN-VIEW.md](MAIN-VIEW.md#save--quit-and-the-notice-slip)).
- **Rules ownership:** the game model owns the track (`UsAlliance`: levels, COIN cost per space, Air Strike) and the four Cash markers (`CASH_MARKERS`). The display only reads them.

## Files

- **Graphics source:** `atlas-top-display.vectorcraft`, authored through the VectorCraft MCP.
  - Artboards are drawn at 6× the 1920×1280 reference and exported at 0.5, which is 3× the reference. That covers both the 6144×3456 authoring frame (2.7×) and the owner's 3840×2160 display (1.69×).
  - The exception is `tag`, which is drawn at 1× and exported at 2×, like the victory tags.
  - The heraldic eagle master sits off-artboard; the three track markers are copies of it.
- **Layout source:** `atlas-top-display.figcraft`. It defines the `Atlas` colour variables (checked against the runtime as [README.md, "Interface palette"](README.md#interface-palette) describes) and the `HUD/*` text styles, and has two frames:
  - **16:9 frame (6144×3456):** the existing areas are locked context rectangles; the new parts are placed exports, each named as the runtime names it. Its full-size export is `atlas-top-display-layout.png`.
  - **`hover tag` frame:** the tag geometry the runtime implements: width 468, title, rule, rows from 76 in steps of 28, a value column at 230, notes after a 12 px gap at 23 px per line, and the placement rule. Its export is `atlas-top-display-tag.png`.
- **Layout record:** `atlas-top-display.layout.json` records each part's FigCraft `absoluteBounds`. The native regression compares the running app with it: images on all four edges, text on left, top and right.
- **Manifest:** `top.sha256` admits the exports at launch. A missing or changed file refuses launch, like every other manifest.

| Export | Artboard | PNG | Reference rect (anchor) |
|---|---|---:|---|
| `atlas-top-aid.png` | aid | 132×114 | 40,30 44×38 (left) |
| `atlas-top-us-alliance-{firm,reluctant,embargoed}.png` | us-alliance-* | 780×192 | 150,18 260×64 (left), one image per level |
| `atlas-top-ledger.png` | ledger | 576×570 | 1688,20 192×190 (right) |
| `atlas-top-resources.png` | resources | 111×111 | 1765.5,28 37×37 (right) |
| `atlas-top-cash.png`, `atlas-top-cash-empty.png` | cash, cash-empty | 108×108 | 1706+46n,226 36×36 (right) |
| `atlas-top-tag.png` | tag | 936×880 | stretched to each tag's content |

The ledger rows reuse `atlas-victory-icon-*.png` at 1704,76+31n, 27×27. Values are runtime Lato text in the layout's `ink` (#203238):
- Aid: 30 px at 92,30;
- ledger: 26 px, right-aligned in 1790..1864, rows from 74.

## The marker supply

The owner selected graphic **M1 · Ficha** and its place in layout **A · Franja** on 2026-11-24 ([decision record](../proposals/last-ui/README.md), sheets [`graphics-marker-supply.png`](../proposals/last-ui/graphics-marker-supply.png) and [`layout-a-franja.png`](../proposals/last-ui/layout-a-franja.png)). #172 built it at rest; #173 added its hover tag (["Its tag"](#its-tag)).

- **What it shows:** the marker supply ([glossary](../../GLOSSARY.md)): the Terror and Sabotage markers not on the map, of the 20 the two kinds share (R 3.3.4). It is the two-faced marker, a bell on the Terror face and a flame on the Sabotage face, beside the number available and "of 20". The flame is provisional: Sabotage has no on-map design yet.
- **Counts:** the rules authority's `observation.marker_supply`; the size of the supply is `MARKER_SUPPLY` in the rules model. The graphic draws the committed position, so a hovered or pinned preview leaves it unchanged.
- **None left:** the number takes `action/primary` and the marker is its dimmed export, as the study's 0 draws them. No position this build reaches places 20 markers, so the native app has not shown it; a unit test builds it from an observation.
- **Place:** the top band, between the US Alliance track and the standings: the main view's area `marker-supply` ([MAIN-VIEW.md](MAIN-VIEW.md#areas)). In every window it is centred between the right edge of the `outside-influence` area and the left edge of the `standings` area, and keeps its distance from the top.
- **Pointer:** the area takes the pointer: a press, a scroll or a drag on it neither moves the camera nor reaches a space or piece. Space tags and popovers keep off it, as they keep off every area.

**Files**

- **Graphics source:** `atlas-marker-supply.vectorcraft`, authored through the VectorCraft MCP from the study's `last-ui-graphics.vectorcraft`: M1's at-rest and 0 drawings, copied and scaled 6× with their strokes. Artboards `marker` and `marker-dimmed` are 54 reference pixels square, drawn at 6× and exported at 0.5, which is 3× the reference, as the top display's other artboards are. The dimmed marker is the study's: its faces and ring at 40% opacity, its split and glyphs in `paper`.
- **Layout:** the main view's `atlas-main-view.figcraft` places them, with the text styles `Supply/count` (Lato Regular 64.125, 38 at the reference, `ink`) and `Supply/of` (Lato Regular 23.625, 14 at the reference, `ink-secondary`), and the `Atlas` variable `action/primary` for the number at 0. "of 20" stands 13.5 (8 reference pixels) after the number, on its baseline, so it follows the number's width as the study sets it.
- **Manifest:** `marker-supply.sha256` admits both exports. A missing or changed file refuses launch before any save is created or replaced.

| Part | From | PNG | Frame rectangle on the owner's display |
|---|---|---:|---|
| `marker-supply` | `atlas-marker-supply.png`, or `atlas-marker-supply-dimmed.png` at 0 | 162×162 | 983.6, 38.82, 91.13×91.13 |
| `marker-supply-count` | runtime text, `Supply/count` | | 1093.29, 44.72 (75×77 for "20") |
| `marker-supply-of` | runtime text, `Supply/of` | | 1181.79, 84.89 (55×28) for "20" |

The area is 964.69, 30.38, 291×108, at the outside-influence area's top and height. For "20" the graphic is centred in it, 253.19 wide.

**Tools:** VectorCraft CLI 0.24.1 and FigCraft CLI 0.18.1, the builds installed on 2026-11-22 from clean checkouts of `storytold/vectorcraft@88f5ece1` and `storytold/figcraft@5c2a3784`, which had not moved on 2026-11-24 ([AVAILABLE.md](AVAILABLE.md#provenance-and-limits) gives the same reflog and timestamp evidence; it is not a recorded build).

**Seat:** the development seat names the area `area:marker-supply` and reports the graphic as `table.marker_supply` ([seat reference](../../docs/agents/seat.md#what-the-table-reports)).

**What this establishes:** placement and count evidence from the native regression `native_marker_supply_matches_layout_and_follows_each_terror` and the main view's regressions in other windows. It is not owner play acceptance: the owner judges its legibility on the display in #166's review.

### Its tag

The marker supply opens the accepted top-display tag, as the Available Forces ledger's rows do: the same plate, title, rule, rows and notes, at the `hover tag` frame's sizes. It is not a new design.

- **Opening:** hovering the graphic, or focusing it with Tab, opens the tag after the shared 400 ms dwell. It closes when the pointer or focus leaves, as the other top-band tags do. There is no pin; reading it changes nothing.
- **Target:** the graphic's extent, the marker's height across the area less the marker's margin on each side (frame 983.6, 38.82, 253.19×91.13 on the owner's display). The area's corners stay bare, so the area still takes the pointer there.
- **Tab order:** the panel's controls, then the marker supply, then the four Available Forces rows, the cards and the piles ([seat reference](../../docs/agents/seat.md#keys) names the stops). It stands left of the ledger, so it comes before it. Aid, US Alliance, Resources and Cash stay hover-only.
- **Words:** the title "Marker supply"; the rows "Terror on the map", "Sabotage on the map" and "Available", whose value is "N of 20", all from `observation.marker_supply`; and two notes:
  - "Terror and Sabotage share 20 markers. Terror places no marker when all 20 are on the map (R 3.3.4)."
  - "Civic Action and Agitation return Terror markers to the supply (R 6.3.2, 6.3.3); the Reset Phase returns the Terror and Sabotage markers (R 6.5)."

  They state nothing the rules leave open: nothing about Terror's other effects with no marker left, and nothing about whether the cap binds Reprisal or Government Earnings.
- **Place:** just below the marker supply's area, at the tags' 8-pixel gap, which is the line the Aid and US Alliance tags open on. It is centred under the graphic, but its right edge keeps the same gap left of the standings area. So it covers neither the US Alliance track nor the standings, and stays inside the window, at 3:2, 16:9 and 21:9 (unit test `the_tag_opens_below_the_graphic_clear_of_the_track_and_the_standings`). At 3:2, where the standings stand closest to the track, its left 40 reference pixels lie over the Sequence of Play track's right edge while it is open, as the Aid and US Alliance tags cover that track. At 16:9 it clears the track, and at 21:9 it is centred.
- **Seat:** the development seat names the target `marker-supply` and reports the open tag as `table.top` ([seat reference](../../docs/agents/seat.md#what-the-table-reports)).
- **What this establishes:** the native regression `native_marker_supply_tag_opens_on_hover_and_focus_and_follows_a_terror` opens it by hover and by Tab, reads its rows before and after a Terror, and checks its place in all three windows. It is not owner play acceptance.

## Drawing notes

- **Cash.** The bundle is an oblique block of bills. Its paper band crosses the top face along the bill's short side, then continues down the front edge at the same width, shaded darker. The bills are pinched beside it.
  - Reference: the banded euro and peso bundles found by the Wikimedia Commons media search for "banknote bundle" (`commons.wikimedia.org/w/index.php?search=banknote+bundle&type=image`).
- **Eagle.** An American eagle displayed, original rather than the official seal:
  - raised wings with scalloped coverts and five round-tipped primaries each;
  - a white head facing the viewer's left, with a hooked gold beak;
  - a shield with a navy chief and red and white pales;
  - a fanned tail and gold talons.

  Reference: the displayed eagles found by the Commons media search for "american eagle displayed heraldry svg".
- **Track labels.** 12 px at the reference with −30/1000 em tracking, as the proposal required; about 20 px on the owner's display.

## Provenance and limits

- **Tools:**
  - VectorCraft CLI 0.24.1. Checkout `ed6f22a8` was inspected; this is not build provenance for the installed binary.
  - FigCraft CLI 0.18.1, rebuilt from `storytold/figcraft@559c952d`, as [art-tools.md](../../docs/agents/art-tools.md) records.
- **Always shown:** the card reader, whose frame once hid the top display, was removed in #139. The top display stays shown while a [card tag](CARD-TAG.md) is open.
- **Title export:** `atlas-layout-title.png` was retired from `layout.sha256`, which `main-view.sha256` has since replaced ([MAIN-VIEW.md](MAIN-VIEW.md)).
- **Native placement evidence:** at 3840×2160, the owner's display. The owner accepted the top display's legibility on that display in #46; [the seat reference](../../docs/agents/seat.md#the-owners-display) says how to judge it from a capture.
- **What this establishes:** component and placement evidence, plus the native regression `native_top_display_matches_layout_and_follows_the_economy`. It is not owner play acceptance.
