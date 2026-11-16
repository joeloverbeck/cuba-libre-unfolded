# Top display

The owner selected these in [#46](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/46):
- **Graphics:** Cash B (banknote bundle), Aid B (supply crate), US Alliance A (eagle marker on three boxes) and Resources C (strongbox).
- **Layout:** A in general, but with C's Resources ledger and C's Cash placement.

They also asked for three changes:
- remove the "CUBA LIBRE" title and put Aid and the US Alliance in the top-left corner;
- wrap the Cash band around the bundle in perspective;
- redraw the eagle as a respectable heraldic eagle.

The selection studies remain in [`../proposals/issue-46/`](../proposals/issue-46/README.md).

## What it shows

- **Top left (outside influence):** the Aid crate with its value, and the US Alliance track, which marks the current level.
- **Top right:** a Resources ledger with one row per faction (its accepted emblem and value), and below it the Cash supply: four slots, filled while that marker is Available and an empty dashed ring while it is on the map.
- **Hover tags:** each of the four elements opens a tag with the shared emblem/region dwell. The Resources tag opens beside the ledger, so the Cash supply stays visible.

## Files

- **Graphics source:** `atlas-top-display.vectorcraft`, authored through the VectorCraft MCP.
  - Artboards are drawn at 6× the 1920×1280 reference and exported at 0.5, which is 3× the reference, for the owner's 6144×3456 display (2.7×).
  - The exception is `tag`, which is drawn at 1× and exported at 2×, like the victory tags.
  - The heraldic eagle master sits off-artboard; the three track markers are copies of it.
- **Layout source:** `atlas-top-display.figcraft`.
  - It is a FigCraft 16:9 frame at 6144×3456. The existing areas are locked context rectangles; the new parts are placed exports.
  - Each new part is named as the runtime names it.
  - It defines the `Atlas` colour variables and the `HUD/*` text styles.
  - `atlas-top-display-layout.png` is its half-size export.
- **Layout record:** `atlas-top-display.layout.json` records each image part's FigCraft `absoluteBounds`. The native regression compares the running app with it.
- **Manifest:** `top.sha256` admits the exports at launch. A missing or changed file refuses launch, like the other presentation manifests.

| Export | Artboard | PNG | Reference rect (anchor) |
|---|---|---:|---|
| `atlas-top-aid.png` | aid | 132×114 | 40,30 44×38 (left) |
| `atlas-top-alliance-{firm,reluctant,embargoed}.png` | alliance-* | 780×192 | 150,18 260×64 (left), one image per level |
| `atlas-top-ledger.png` | ledger | 576×570 | 1688,20 192×190 (right) |
| `atlas-top-resources.png` | resources | 111×111 | 1765.5,28 37×37 (right) |
| `atlas-top-cash.png`, `atlas-top-cash-empty.png` | cash, cash-empty | 108×108 | 1706+46n,226 36×36 (right) |
| `atlas-top-tag.png` | tag | 936×880 | stretched to each tag's content |

The ledger rows reuse `atlas-victory-icon-*.png` at 1704,76+31n, 27×27. Values are runtime Lato text:
- Aid: 30 px at 92,30;
- ledger: 26 px, right-aligned in 1790..1864, rows from 74.

## Drawing notes

- **Cash.** The bundle is an oblique block of bills. Its paper band crosses the top face along the bill's short side, then continues down the front edge at the same width, shaded darker. The bills are pinched beside it, following a photographed banded stack on Wikimedia Commons.
- **Eagle.** An American eagle displayed, original rather than the official seal, from heraldic references on Wikimedia Commons:
  - raised wings with scalloped coverts and five round-tipped primaries each;
  - a white head facing the viewer's left, with a hooked gold beak;
  - a shield with a navy chief and red and white pales;
  - a fanned tail and gold talons.
- **Track labels.** 11 px at the reference, about 30 px on the owner's display.

## Provenance and limits

- **Tools:**
  - VectorCraft CLI 0.24.1, checkout `ed6f22a8`;
  - FigCraft CLI 0.18.1, checkout `559c952d`.
  These are inspected checkouts, not installed-binary provenance.
- **Reader frame:** "CUBA LIBRE" was removed from `atlas-victory-reader.vectorcraft` and the reader re-exported.
- **Title export:** `atlas-layout-title.png` is retired from `layout.sha256`.
- **What this establishes:** component and placement evidence, plus the native regression `native_top_display_matches_layout_and_follows_the_economy`. It is not owner play acceptance.
