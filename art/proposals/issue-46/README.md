# Top display studies

> **Note (2026-11-18, #50):** "the owner's display" here means the 6144×3456 authoring frame. The owner's display is 3840×2160 ([foundation 4](../../../docs/foundation/04-art-and-assets.md)), so sizes given for it here are 1.6 times what it shows.

These are decision-only alternatives for [#46](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/46), within #8. They are the owner's one selection round for this issue:
- one graphic each for Cash, Aid, the US Alliance and Resources;
- one layout for placing them.

The baseline is `771aabf7e887d16ea30dc36b0325fa7a9b9654a7`. No runtime code or admission manifest consumes these files. Selection is not native play acceptance.

**Selected on 2026-11-16:**
- **Graphics:** Cash B, Aid B, US Alliance A, Resources C.
- **Layout:** A in general, with C's Resources ledger and Cash placement.
- **Owner changes:**
  - remove the "CUBA LIBRE" title and put Aid and the US Alliance in the top-left corner;
  - make the Cash band wrap around the bundle in perspective;
  - redraw the eagle from heraldic references.

The production components and layout are recorded in [`../../atlas/TOP.md`](../../atlas/TOP.md). These studies remain as comparison evidence.

## What to choose

Pick one letter per row. Graphics and layout are chosen independently: any graphic works in any layout.

| Decision | Sheet | Recommended |
|---|---|---|
| Cash marker and four-slot supply | [graphics-cash.png](graphics-cash.png) | **B** |
| Aid token | [graphics-aid.png](graphics-aid.png) | **B** |
| US Alliance track | [graphics-us-alliance.png](graphics-us-alliance.png) | **A** |
| Resources icon | [graphics-resources.png](graphics-resources.png) | **C** |
| Layout | [A](layout-a-owner.png) · [B](layout-b-chips.png) · [C](layout-c-ledger.png) | **A** |

Each sheet shows the three alternatives large, then at game size on the desk colour. The layout frames are 16:9 for the owner's 6144×3456 display, shown at half size. The `*-top-band-1x.png` crops show the top of each frame at full size, which is the scale at which to judge legibility.

## Graphics

All are new VectorCraft artwork in `top-display-graphics.vectorcraft`, with one artboard per concept.

- **Palette:** the Atlas palette (ink `#203238`, paper `#f8f4e8`, desk `#eee9dc`).
- **Font:** Lato.
- **Physical tokens** (looked at, not copied): Aid is a teal star square, US Alliance is the State Department seal, Cash is a sage disc with coins. Resources have no token; the board marks them with cylinders on its edge track.

### Cash

Cash is a marker held under a piece on the map, not a balance (glossary, R 4.5.2). Its supply has four slots: a filled slot means the marker is available, an empty dashed ring means it is on the map. In this slice all four stay available, because nothing places Cash. The same graphic will mark Cash under pieces once an action can place it.

| | Meaning | Readability and density | Atmosphere |
|---|---|---|---|
| A · Faithful disc | The physical counter, vectorized | The coin stacks become a gold smear at 36 px | Familiar to owners of the game |
| **B · Banknote bundle** | Banded bills on a cream marker | The bill and band stay distinct at 36 px; no coins | Period cash, faction-neutral |
| C · Attaché case | Casino skim on felt green | The case reads at 36 px; the bills at the lid do not | Strong Syndicate flavour, but every faction can hold Cash |

### Aid

Aid is one value, 15 here, that the app draws beside the token at runtime.

| | Meaning | Readability and density | Atmosphere |
|---|---|---|---|
| A · Faithful star square | The physical counter | Clear, but its star repeats 26July's emblem and the Active Guerrilla star, a few centimetres away | Faithful |
| **B · Supply crate** | A shipment of foreign assistance | "AID" reads at 44 px; nothing shares its shape | Practical, period shipping |
| C · Freighter | Shipments arriving by sea | The silhouette collapses below about 60 px | Evocative, but needs size |

### US Alliance track

These alternatives show the three ordered levels with Firm marked. The official seal is not reproduced: it is a real agency's insignia and illegible at this size. Green, amber and red appear only in B's dial.

| | Meaning | Readability and density | Atmosphere |
|---|---|---|---|
| **A · Eagle marker on three boxes** | Mirrors the board's track | The only option that names the level without hovering. Its box labels must be re-authored at 12 reference px or more (32 px, about 3.7 mm on the owner's display); the study's labels are too small | Official, archival |
| B · Diplomatic dial | A needle in one of three sectors; worse is further right | Severity reads at a glance. The names become unreadable, and it is the widest and tallest option | Instrument-like |
| C · Wordless pictures | Arms crate, a door ajar, a chained padlock; the current one lit | Compact. The meaning must be learned from the hover tag first | Story-like, least text |

### Resources

Resources show as one icon with a value per faction. Coins are kept for Cash, so no Resources icon uses them.

| | Meaning | Readability and density | Atmosphere |
|---|---|---|---|
| A · Sugar sack | Cuba's sugar economy | The silhouette is distinct, but the cane mark vanishes below 40 px | Most thematic |
| B · Ledger | A tally that can be spent | Dark and small: it reads as "a book" at 28 px | Bookish |
| **C · Strongbox** | A faction's reserves | The clearest at 28 and 40 px; it means stored wealth, beside the Cash bundle's cash in hand | Generic, sturdy |

## Layouts

`top-display-layouts.figcraft` has three 6144×3456 frames. All three keep:
- Aid and the US Alliance between the title and the victory emblems, as outside influence on Cuba;
- the emblems centred and everything else unchanged.

The frames use the recommended graphics only so that the layouts can be compared on equal terms. The document defines the `Atlas` colour variables and the `HUD/value` and `HUD/chip` text styles. Values are the prepared position's, from development-seat observation revision 0: Resources Government 6, 26July 0, Directorio 5, Syndicate 15; Aid 15; Firm; 4 Cash available.

| | Arrangement | Benefit | Trade-off |
|---|---|---|---|
| **A · Owner's placement** | A Resources row (icon, then emblem and value per faction) centred between the emblems and the right edge. The Cash supply is a separate cluster below it at the right | One glance across the top shows every quantity, and the groups are separated by meaning | Adds a second emblem row; the faction marks appear twice |
| B · Attached chips | A small Resources chip under each victory margin; the Cash supply alone at the top right | Faction identity appears once, and each faction's standing and wealth sit together | The chips crowd the emblem column and nearly touch the map header. 54 px values are the smallest type here |
| C · Ledger column | A paper ledger of four rows in the top-right corner, with the Cash supply beneath | A tidy accounting look with large values | Adds a panel. Cash ends 40 px above the operation panel, and the top-right corner becomes busy |

## How they were made, and limits

- **Context.** The background of each frame is a development-seat capture of the current application at 3840×2160. It is 16:9; the seat's maximum is 4096×2160. The frame scales it by 1.6.
  - It shows the real accepted layout and the text that #46 later removes.
  - The two card faces were painted out in PhotoCraft before export; no card art is committed.
  - The capture, observation and logs are in `local/verification/issue-46/seat-context/`.
- **Tools** (checkouts inspected; not build provenance for the installed binaries):
  - VectorCraft CLI 0.24.1, checkout `ed6f22a8`;
  - FigCraft CLI 0.18.1, checkout `559c952d`;
  - PhotoCraft CLI 0.23.1, checkout `5b27ff6f`.
- **Exports.** The `layout-*.png` graphics are VectorCraft exports placed by FigCraft.
- **Not covered:**
  - Hover tags: their content and behaviour are fixed by #46.
  - Production sizes: the selected graphics will be re-authored as components at their final sizes.
  - Native fit, interaction, and owner acceptance.

The issue owns selection, implementation and acceptance; this README is not a work queue.
