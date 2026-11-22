# Card tag

The owner selected card tag **T1 · Face and ledger** on 2026-11-22 ([decision record](../proposals/main-view-cards/README.md), sheet [`card-tags.png`](../proposals/main-view-cards/card-tags.png), left; owner's words in [#136](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/136)). #139 built it for spec #137 and retired the card reader, so there is one way to read a card.

## What it shows

- **Opening:** hovering the current or the preview card, or focusing it with Tab, opens its tag after the shared 400 ms dwell, as the other hover tags open. A click, or Enter on the focused card, pins it; a second click unpins it. Escape closes it, and a hovered or focused card's tag then stays closed until the pointer or focus moves on. Reading a card changes nothing in the match.
- **Left:** the card's face at reading size, 620 px wide on the owner's display, no smaller than the face's own 572–573 px. Its corners are rounded so the black printed outside the card's corners never shows.
- **Right, from the rules model (`observation.cards`):** the number and title, a CURRENT CARD or PREVIEW mark, the faction order as the victory emblems, the kind of Event, where the Event stands, its lasting effect (Capability or Momentum) and the rule references. The tag words nothing itself; each line is the model's `words`.
- **Placement:** the tag opens above the card panel, its left edge on the panel's and its bottom 8 reference px above it, so both cards stay visible beneath it. It fits inside the 1920×1280 reference, so it stays inside every window.
- **Not shown:** Event text. The face carries it (T1's trade-off).

## Files

- **Layout source:** `atlas-card-tag.figcraft`, authored in FigCraft CLI 0.18.1 from sheet T1. Its two frames are drawn, as T1 is, at the owner's 3840×2160 display, which is the 1920×1280 reference ×1.6875:
  - **current:** card 17 at Directorio's handoff after Terror with Kidnap, T1's own example;
  - **preview:** card 18 in the same state, with the PREVIEW mark.
  Every part is named as the runtime names it. The placeholder faces are `../proposals/main-view-cards/placeholder-face-{17,18}.png`; no private face is embedded. `atlas-card-tag-layout.png` is both frames at half size.
- **Changes from T1:** the plate is the accepted hover tag plate (`atlas-top-tag.png`), whose ink stripe is slightly narrower than T1's drawn rectangle and has no drop shadow. A value's caption follows it by a constant 50 px, where T1 used 48 or 50. The face's corner radius is 35 px (see Limits); T1's placeholder drew none.
- **Reused components:** the hover tag plate, the briefing rule (`atlas-victory-rule.png`), the victory emblems (`atlas-victory-icon-*.png`, the same files T1 placed) and Lato.
- **New exports:** the two mark plates, FigCraft exports of the frames' `mark` rectangles at 3 px per reference pixel (`atlas-card-tag-current.png` 481×82 in `ink`, `atlas-card-tag-preview.png` 338×82 in `ink-secondary`). Their text is runtime Lato Bold.
- **Layout record:** `atlas-card-tag.layout.json` records each part relative to its frame. The native regression `native_card_tag_matches_layout_and_reads_the_model` compares the running app with it at 3840×2160, within 2 px: images on all four edges, text on left, top and right. The ledger column flows, so a value that wraps moves the captions below it; the record holds the line counts of its example state, which the regression reproduces.
- **Manifest:** `card-tag.sha256` admits the two mark plates at launch. A missing or changed file refuses launch. The tag is stateless presentation, so it does not change a save's identity.
- **Palette:** `atlas-card-tag.variables.json` is the file's `Atlas` export.

| Part | Frame rectangle (x, y, w, h) | Type |
|---|---|---|
| plate | 0, 0, 1250, 1090 | `atlas-top-tag.png`, stretched |
| title | 60, 34, w 860 | Lato Regular 54, `ink` |
| mark | right edge 1210, 46, w 270 (current) or 190 (preview), h 46 | mark plate; label Lato Bold 22, tracking 2, `paper`, centred at y 55 |
| rule | 60, 120, 1150, 2 | `atlas-victory-rule.png` |
| face | 60, 150, 620, 889 | the admitted face, corner radius 35 |
| ledger column | 720, 150, w 490 | captions Lato Bold 22, tracking 2, line 26, `ink-secondary`; values Lato Regular 32, line 38, `ink` (rules `ink-secondary`) |
| emblems | 12 below FACTION ORDER, 52 square, every 64 | victory emblems in the card's printed order |
| hint | 720, 1000, w 490 | "Click pins · Esc closes", Lato Regular 26, `ink-secondary` |

In the column, the emblems start 12 below their caption and the EVENT caption 40 below them; each value starts 8 below its caption, and each caption 50 below the value before it.

## Rules ownership

The game model (`src/game/cards.rs`) owns every fact for the three admitted faces, transcribed from the supplied faces: titles; orders (17 and 18: 26July, DR, Government, Syndicate; 19: 26July, DR, Syndicate, Government); kinds (17 dual-use, R 5.2; 18 and 19 one text, R 5.1); and lasting effects (18 is an Insurgent Capability, R 5.3, not in effect until executed; 17 and 19 have none). Where the Event stands follows R 2.3.2–2.3.4 and 2.3.8: open to the 1st Eligible faction, not yet open while the 1st Eligible executes its Operation, not open this card after an Operation Only, in progress, resolved, and, for the preview, not yet open until it is played. `tests/integration/card_facts.rs` checks every stage of card 17's Event and card 18 after the advance against values worked from the rules and faces.

## Limits

- **Corner radius.** Every admitted face has black printed outside its rounded corners. Clearing every dark pixel needs a radius of 27.4 px on card 17 (572 px wide), 30.6 px on card 18 and 26.2 px on card 19 (573 px), measured on the private files; the shared card shape's 4.5% (25.8 px) leaves black on card 18. The tag and the panel's thumbnails therefore use 5.6% of the face's width: 35 px on the 620 px face.
- **Window sizes:** the frames are authored at 16:9 only, as foundation 4 requires. The tag scales with the window like every other area.
- **Italic:** none in T1.
- **Display:** legibility is judged at the owner's display, as [the seat reference](../../docs/agents/seat.md#the-owners-display) describes. The owner's native review is the final judgement.
