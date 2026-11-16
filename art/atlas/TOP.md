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
- **Top right:** a Resources ledger with one row per faction (its accepted emblem and value), and below it the Cash supply: four slots, filled while that marker is Available and an empty dashed ring while it is on the map.
- **Hover tags:** each of the four elements opens a tag with the shared emblem/region dwell. The Resources tag opens beside the ledger, so the Cash supply stays visible.
- **Rules ownership:** the game model owns the track (`UsAlliance`: levels, COIN cost per space, Air Strike) and the four Cash markers (`CASH_MARKERS`). The display only reads them.

## Files

- **Graphics source:** `atlas-top-display.vectorcraft`, authored through the VectorCraft MCP.
  - Artboards are drawn at 6× the 1920×1280 reference and exported at 0.5, which is 3× the reference, for the owner's 6144×3456 display (2.7×).
  - The exception is `tag`, which is drawn at 1× and exported at 2×, like the victory tags.
  - The heraldic eagle master sits off-artboard; the three track markers are copies of it.
- **Layout source:** `atlas-top-display.figcraft`. It defines the `Atlas` colour variables and the `HUD/*` text styles, and has two frames:
  - **16:9 frame (6144×3456):** the existing areas are locked context rectangles; the new parts are placed exports, each named as the runtime names it. Its full-size export is `atlas-top-display-layout.png`.
  - **`hover tag` frame:** the tag geometry the runtime implements: width 468, title, rule, rows from 76 in steps of 28, a value column at 230, notes after a 12 px gap at 23 px per line, and the placement rule. Its export is `atlas-top-display-tag.png`.
- **Layout record:** `atlas-top-display.layout.json` records each part's FigCraft `absoluteBounds`. The native regression compares the running app with it: images on all four edges, text on left, top and right.
- **Manifest:** `top.sha256` admits the exports at launch. A missing or changed file refuses launch, like the other presentation manifests.

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

## Drawing notes

- **Cash.** The bundle is an oblique block of bills. Its paper band crosses the top face along the bill's short side, then continues down the front edge at the same width, shaded darker. The bills are pinched beside it.
  - Reference: the banded euro and peso bundles found by the Wikimedia Commons media search for "banknote bundle" (`commons.wikimedia.org/w/index.php?search=banknote+bundle&type=image`).
- **Eagle.** An American eagle displayed, original rather than the official seal:
  - raised wings with scalloped coverts and five round-tipped primaries each;
  - a white head facing the viewer's left, with a hooked gold beak;
  - a shield with a navy chief and red and white pales;
  - a fanned tail and gold talons.

  Reference: the displayed eagles found by the Commons media search for "american eagle displayed heraldry svg".
- **Track labels.** 12 px at the reference with −30/1000 em tracking, as the proposal required; about 32 px on the owner's display.

## Provenance and limits

- **Tools:**
  - VectorCraft CLI 0.24.1. Checkout `ed6f22a8` was inspected; this is not build provenance for the installed binary.
  - FigCraft CLI 0.18.1, rebuilt from `storytold/figcraft@559c952d`, as [art-tools.md](../../docs/agents/art-tools.md) records.
- **Reader frame:** "CUBA LIBRE" was removed from `atlas-victory-reader.vectorcraft` and the reader re-exported. When the reader is open, the top display is hidden, as it was before.
- **Title export:** `atlas-layout-title.png` is retired from `layout.sha256`.
- **Native placement evidence:** 3840×2160 is the development seat's largest 16:9 surface. Legibility at the owner's 6144×3456 is shown by the full-size FigCraft export, and remains the owner's judgement.
- **What this establishes:** component and placement evidence, plus the native regression `native_top_display_matches_layout_and_follows_the_economy`. It is not owner play acceptance.
