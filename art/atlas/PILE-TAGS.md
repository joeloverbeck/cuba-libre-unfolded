# Pile tags

The owner's selection for spec #137 includes a tag on each pile, drawn on sheet [`pile-tags-and-advance.png`](../proposals/main-view-cards/pile-tags-and-advance.png) and built on the accepted hover tag, so neither is a new visual design ([decision record](../proposals/main-view-cards/README.md)). #144 built them.

## What they show

- **Opening:** hovering a pile's count disc on the main view ([MAIN-VIEW.md](MAIN-VIEW.md#the-card-piles)), or focusing it with Tab, opens its tag after the shared 400 ms dwell, as the other hover tags open. The tag closes when the pointer or focus leaves. There is no pin. Reading a pile changes nothing in the match. While a played card is in the air no pile tag shows ([the card advance](MAIN-VIEW.md#the-card-advance)).
- **Played pile:** the title "Played pile"; the count with "current card on top"; Propaganda "n of 4 played"; under MOST RECENT FIRST every card in the pile, most recent first, as "number · title", the current card marked "· current" and each Propaganda card set in bold; and a note on why the pile is open (R 2.2).
- **Draw deck:** the title "Draw deck"; the count with "counts the preview"; the cards face down; the Propaganda cards left, of 4; how soon the next Propaganda card can come; and a note on Deck Preparation that states the figure's assumption, labelled as local policy ([local rule policies](../../docs/local-rule-policies.md#next-propaganda-assumes-equal-piles)). Nothing on it, or anywhere, shows the order of the face-down cards.
- **Words:** each line is the rules model's: `observation.played_pile` and `observation.deck`, with their `words`. The tags word nothing themselves.
- **Placement:** each tag opens above the piles with its foot 16 px above the piles' labels, as the card tag does, and its left edge on its own pile's: the played pile's at 110, the deck's at 470 on the owner's display (`open` in the record). A tag grows upward from its foot, so both piles, their labels and discs stay visible beneath it. The largest played tag, 18 rows in three columns, fits inside the 1920×1280 reference, so every tag stays inside every window. A pile tag lies above an open card tag.

## A long played list

Every card in the played pile is in the tag. Sheet's tag showed eight and "… 11 more"; the production frames settle the list within the same tag, type and rows:

- A column holds at most 18 rows, one every 47 px, and the columns are balanced: columns = ⌈n / 18⌉, rows = ⌈n / columns⌉, filled top to bottom, then left to right.
- Each column is 600 wide, one every 660; the plate is 100 + 660 × columns wide, and never narrower than sheet's 800.
- 18 cards on card 17 make one column, 19 after the advance two of 10 and 9, and 51 late in a game three of 17 (2080 × 1323 on the owner's display). 52 would make three of 18.

## Files

- **Layout source:** the four frames beside the main view's frame in `atlas-main-view.figcraft` (FigCraft CLI 0.18.1), drawn at the owner's 3840×2160 display, the reference ×1.6875:
  - **played-pile tag · card 17:** 18 cards, one column;
  - **played-pile tag · card 18:** 19 cards, two columns;
  - **played-pile tag · late example:** 51 cards of the prepared deck with card 48 current, three columns;
  - **draw-deck tag · card 17.**
  The main view's frame holds two hidden rectangles, `played-pile-tag` and `deck-pile-tag`, where each opens on card 17. Every part is named as the runtime names it; a played card's row is `column-<c>-row-<r>` at runtime, the frames' `column-<c>` text box holding its rows. `atlas-pile-tags-layout.png` is the four frames at half size.
- **Layout record:** `atlas-pile-tags.layout.json` holds each frame's parts relative to it, the `open` rectangles and the column rule. A unit test holds the runtime's constants to it, and the native regression `native_pile_tags_match_layout_and_read_the_model` compares the running app with it at 3840×2160 within 2 px: the plate and the rule on all four edges, text on left, top and right.
- **Reused components:** the hover tag plate (`atlas-top-tag.png`, stretched to the tag, as every tag stretches it), the briefing rule (`atlas-victory-rule.png`) and Lato. No new export or manifest: the tags are stateless presentation and change no save's identity.

| Part | Frame rectangle (x, y, w) | Type |
|---|---|---|
| plate | 0, 0, the tag's size | `atlas-top-tag.png`, stretched |
| title | 60, 34 | Lato Regular 54, `ink` |
| rule | 60, 120, width − 100, h 2 | `atlas-victory-rule.png` |
| ledger | labels at 60, values at 340 (played) or 360 (deck), from 146, a line every 47 | Lato Regular 34; labels `ink`, values `ink-secondary` |
| label-list | 60, 270 | MOST RECENT FIRST, Lato Bold 22, tracking 2, `ink-secondary` |
| column-<c> | 60 + 660c, 308, w 600 | Lato Regular 34, line 47, `ink`; Propaganda cards Lato Bold |
| note | 60, 36 below the last row (played) or 370 (deck), w 700 or 720 | Lato Regular 30, line 39, `ink-secondary` |

The plate ends 63 below the note, whose words may wrap to more lines than the frame's example: the plate grows with it.

## Rules ownership

The game model owns every fact (`src/game/pile_facts.rs`, `src/game/titles.rs`):

- **Titles:** all 52, transcribed from the supplied faces and checked against the third-printing playbook's card list (p. 36); they agree. The four Propaganda cards are titled "Propaganda!" as printed. No Event text is transcribed.
- **Next Propaganda:** with exactly equal piles of 12 Events, the n-th Propaganda card lies in the n-th pile of 13 cards. With P cards played and k Propaganda cards among them, the next lies between the max(P + 1, 13k + 1)-th and the 13(k + 1)-th card of the game; counted from the deck's top, the preview being the 1st, that is max(1, 13k + 1 − P) to 13(k + 1) − P. A Propaganda preview is the next card; an Event preview narrows the earliest to the 2nd. The figure reads "within the next N cards" when the current pile still holds its Propaganda card, and "the Nth to the Mth card" when that card is already played.
- `tests/integration/pile_tags.rs` checks positions worked by hand from Deck Preparation and R 2.4 before the model: the prepared deck on card 17 (2nd to 8th) and 18 (2nd to 7th), a current pile whose Propaganda card is played (12th to 24th), a Propaganda preview (the next card), a preview that starts a pile (2nd to 13th), the first card of a pile and of the game, and none left.

## Limits

- **Plate stripe:** the hover tag plate is stretched, as on every tag, so its ink stripe widens with the tag: about 9 px on the 800-wide tag and 22 px on the three-column one.
- **Window sizes:** the frames are authored at 16:9 only, as foundation 4 requires. The tags scale with the window like every other area.
- **Display:** legibility is judged at the owner's display. The owner's native review is the final judgement.
