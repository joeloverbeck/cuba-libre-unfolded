# The newspaper

The owner selected **B1 · El Correo de la Isla** on 2026-11-25, with its headline face, Noto Serif Display ([selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7161227757), [studies](../proposals/issue-200/README.md)), for spec [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197). [#208](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/208) built it. When an edition opens, what it holds and how it is put away is in [the play guide](../../docs/playing.md#the-newspaper). The masthead's Spanish title stays; everything else on the page is in English.

## The page

A front page in the middle of the window, over a scrim that leaves the top band clear: masthead, folio line, headline across the page, a sentence under it, three ruled columns, a foot line. It prints one action report ([GLOSSARY.md](../../GLOSSARY.md)) in the report's own words, and nothing else but the fixed words below.

| Part | What it says |
|---|---|
| Folio line, left | `No. n`, `partway` while the action has not ended, the card's number and title. Editions are numbered for each card. |
| Folio line, right | The faction, its role on the card, and the box the action is recorded in; `… in progress` before it has ended. `Event` is the record for a completed Event. |
| Headline and the sentence under it | The report's headline and, when it has one, its summary |
| Articles | One for each of the report's, in its order, its sentences run on. A die roll's article has the die. |
| The standings | Every faction's Resources and margin: what changed as `from → to`, bold; what did not, quiet |
| Notes | The report's own, each sentence apart: Control, Available Forces, Markers, Victory totals, Eligibility |
| Foot line | `Written from the committed result`; on a continued edition the page; how it is put away |

- **Columns.** Articles and then notes fill the three columns in order, the third below the standings. The page is as low as holds them and never lower than the standings. A block is never divided.
- **Two pages.** A page is at most 1900 tall, to 42 from the window's foot. What does not fit continues on another page with the masthead, the folio line and the columns, turned with Left and Right. The type never shrinks.
- **Lines.** A change (`4 → 3`) stays on one line, and so does a number with the word it counts. The headline and the sentence under it are balanced. Lines are broken in the layout's own units, so an edition has the same lines and pages in every window.
- **No faction's colour is in the print.** A faction is its name, or its accepted emblem in the standings. It looks like no proclamation: a masthead, several articles in a fixed order, a table, and no seal, slogan or author ([foundation 5](../../docs/foundation/05-players-and-proclamations.md)).

## The pile

The editions of the current card that were put away lie in a pile beside the Propaganda pips, with their count on the accepted count disc and the caption "card 18's editions". A click, or Tab and Enter, opens the latest again. The pile lies over the transfer modal's scrim and over the open page's, and goes with its card; a card without an edition has none. With more than one, a reopened page has a tab for each under its folio line, the latest first, the open one filled: `No. 2 · Assault ends`, `No. 1 · partway`. More than five say only their numbers, in as many rows as they need.

## Files

- **Layout source:** `atlas-newspaper.figcraft` (FigCraft CLI 0.18.1, the build installed on 2026-11-22 from a clean checkout of `storytold/figcraft@5c2a3784`; reflog and timestamp evidence, not a recorded build). Made from the round's B1 frames in `newspaper-layouts.figcraft`: `16:9 edition · Terror with Kidnap` and `16:9 edition · Pass, reopened` (3840×2160), `Edition states, 1:1` (partway, tabs, the longest edition found on its two pages, the pile) and `Plates`. `atlas-newspaper-layout.png`, `atlas-newspaper-reopened.png` and `atlas-newspaper-states.png` are the first three's exports. The captures beneath the frames are the round's, with both piles' top cards blanked. Every report printed in the frames is #207's, from a match of this build.
- **Graphics source:** `atlas-newspaper.vectorcraft` (VectorCraft CLI 0.24.1, the build installed on 2026-11-22 from a clean checkout of `storytold/vectorcraft@88f5ece1`; reflog and timestamp evidence, not a recorded build): the masthead, the six dice and the pile, kept from the round's `seat-and-newspaper-graphics.vectorcraft`. Every export is pixel-identical to the round's.
- **Typeface:** `NotoSerifDisplay-Bold.ttf`, for the headline and the titles, and in the masthead and the pile as they were authored: Google's Noto project, SIL OFL 1.1, the unchanged file of Ubuntu's `fonts-noto-core` 20201225-2build1 (SHA-256 `8666c2488e681a31670f9fc0b5d59fa4f21f2a0664a76876fe49ebd832f673c7`). [FONT-NOTICE-NOTO-SERIF-DISPLAY.txt](FONT-NOTICE-NOTO-SERIF-DISPLAY.txt) is that package's unchanged copyright file. The page uses no other weight of it. Everything else is the accepted Lato.
- **Layout record:** `atlas-newspaper.layout.json` holds both 16:9 frames' `absoluteBounds` and words, the decisions that play each sample, the 2-unit tolerance and each export's size. A unit test sets the samples from the admitted fonts and holds lines, blocks, rules and foot equal to it, and `native_the_newspaper_matches_the_selected_layout` the drawn parts.
- **Exports,** at 2 px per unit, in `newspaper.sha256` with the typeface: `atlas-newspaper-masthead.png`, `-pile.png` and `-die-1.png` to `-die-6.png` from VectorCraft; the plates `-page.png` (its shadow reaches 42, 32, 42 and 52 past it), `-scrim.png`, `-tab.png` and `-tab-open.png` from FigCraft. The page and the tabs are nine-sliced. The rules are the accepted `atlas-popover-rule-ink.png` and `atlas-briefing-rule.png`, the emblems and the count disc the accepted ones. A missing or changed file refuses launch like every other manifest.
- **Scrim and shadow.** The application blends in linear light, where the frames' 22% ink and the shadow's 33% come out lighter. Their plates are 37% and 53%, which give the drawn result there: the sea under the scrim is within 3 of the frame's value per channel.
- **Place:** the page keeps its distance from the window's top and its middle. The pile is the main view's area `editions` ([MAIN-VIEW.md](MAIN-VIEW.md#areas)).
- **Palette:** `atlas-newspaper.variables.json` ([README.md, "Interface palette"](README.md#interface-palette)).

## Differences from the round's sheets

- **The words** are #207's reports. The sheets' "Nothing else changed" and "To be continued" are in no report and are not printed, so a short edition's note stands in the first column.
- **The space under the columns** is 24 everywhere. The sheets left 20 to 78.
- **The tabs** stand on a row of their own under the folio line, 60 tall as drawn, each as wide as its words. The sheets drew them apart from the page.
- **Not drawn in the round,** built in the same form: the second page, the page's number in the foot line, and rows of tabs.

## Limits

- **Two pages at most,** in what was found: the unshaded Event with every side decision it can have fills 1886 of the first page and 604 of a second. The longest Terror is 1898 tall, the longest Assault, in 12 spaces, 1862 and the longest shaded Event 1576, on one page each. The search behind them is #207's and proves no maximum; a longer one would continue on a second page.
- **Type at the owner's display,** where a pixel is 0.18 mm: text 27 (4.9 mm), the table and the folio line 25 (4.5 mm), the foot line 22 (4 mm), the table's headings 20 (3.6 mm).
- **A request's modal** lies beneath the edition that tells of it, and shows below a short one.
- **Other shapes of window** keep the page's size and place by the scale; none was drawn (foundation 4).
