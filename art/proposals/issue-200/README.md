# Seat controls and the newspaper: three alternatives each

Decision-only studies for [#200](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/200), part of spec [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197). The owner selects in one round; #206 builds the seat controls and #208 the newspaper from what is selected. Nothing here is adopted: no runtime code, save or admission manifest reads these files. [Foundation 4](../../../docs/foundation/04-art-and-assets.md) governs the round, and [foundation 5](../../../docs/foundation/05-players-and-proclamations.md) reserves the newspaper for the app's reports. The baseline is `38497ac`.

**Selection:** not made yet.

## The two subjects

The seat layout, the decision panel's form and the newspaper are three independent choices. Each seat alternative is drawn with a different panel so that every panel is seen once in place, and any of them can be combined.

### A. Seat controls

| | Seats | Decision panel while an agent decides | 16:9 at the owner's display | States, 1:1 |
|---|---|---|---|---|
| **A1 · Tarjetas** | Four place cards on the sea along the bottom, one per faction | **Dossier:** the state, what the rules offer it, what happened on this card | [Directorio](a1-tarjetas-16x9.jpg), [Government](a1-tarjetas-government-16x9.jpg) | [states](a1-tarjetas-states.png) |
| **A2 · Libro** | A ledger of four rows under the decision panel | **Same panel, inert:** the faction's own rows without plates, and a state strip where the commit was | [Directorio](a2-libro-16x9.jpg), [Government](a2-libro-government-16x9.jpg) | [states](a2-libro-states.png) |
| **A3 · Cintas** | A ribbon under each victory emblem; a click opens its sheet | **Folded slip:** heading, state and a button that unfolds the options | [Directorio](a3-cintas-16x9.jpg), [Government](a3-cintas-government-16x9.jpg) | [states](a3-cintas-states.png) |

**A1 · Tarjetas**
- **Why it suits the task:** a seat is a place at a table, and the cards say who sits where. The deciding faction's card stands forward. Every control is on the card, so handing over and taking back are one click each and the line to copy is one more.
- **Readability:** the best of the three: names at 34 pixels, states at 25, a chair mark per state that reads without the words.
- **Density:** the lowest. The cards take 1,940 pixels of the empty bottom sea, below the island box.
- **Atmosphere:** the most physical.
- **Trade-off:** the cards are far from the panel they describe, and the bottom of the table is no longer open sea. The dossier shows the options as lines with no briefings to read.

**A2 · Libro**
- **Why it suits the task:** the seats stand with the decision. The eye that asks "what are we waiting for" finds the panel, its state strip and the ledger together, in the idiom of the Resources and Available Forces ledgers.
- **Readability:** good: names at 30 pixels, states at 23 to 25; the smallest type of the round is the 17-pixel pill in its panel, which sets as wide per letter as the accepted pills in the capture.
- **Density:** the highest. All four seats and every button fit 844×396 pixels, and the rows keep the briefings readable on hover.
- **Atmosphere:** the most consistent with what is there, and the least evocative.
- **Trade-off:** the right side becomes one full column from the ledgers to Save · quit, and Take back appears twice for the deciding faction.

**A3 · Cintas**
- **Why it suits the task:** the four factions already have one place at the top, so no new area is needed. The lamp lights for the faction that thinks.
- **Readability:** the state is one or two words at 24 pixels; the sentence needs a click.
- **Density:** the least space at rest, but the standings gain a second line, and every control is one click further away.
- **Atmosphere:** the lit lamp gives a sense of someone at work; the folded slip gives the east of the window back to the map while nothing is asked of the player.
- **Trade-off:** the sheet covers the north of the map while open, and the panel changes size each time a decision passes between the player and an agent. The victory emblems' own hover tags open over the same place, which is not drawn.

**States** in every alternative: human, awaiting an agent, thinking, silent, build ends here. The ticket's list has no word for a seat an agent holds while it has nothing to decide, so each also draws that resting form ("Agent at the table"). Silent shows the minutes since the agent was last heard, as Arkham's partners do; 12 is an example. The line to copy is an example too: #205 settles its words and paths. The study writes "not in this build" where the app today writes "NOT IN THIS TASK" or "INSPECT"; the words stay the rules authority's and the panel's.

### B. The newspaper

| | Form | Place | Longest | Shortest | Partway, 12 spaces, reopening |
|---|---|---|---|---|---|
| **B1 · Correo** | A front page with three columns: *El Correo de la Isla* | Centre, 1560 wide, over a scrim | [longest](b1-correo-longest-16x9.jpg) | [shortest](b1-correo-shortest-16x9.jpg) | [sheet](b1-correo-partway-and-12-spaces.png) |
| **B2 · Isla** | An illustrated weekly opened at a spread, with a map of where it happened | Centre, 2400 wide, over a scrim | [longest](b2-isla-longest-16x9.jpg) | [shortest](b2-isla-shortest-16x9.jpg) | [sheet](b2-isla-partway-and-12-spaces.png) |
| **B3 · Tarde** | A narrow evening paper on a café reading stick: *La Tarde* | Right, 900 wide, over the decision panel, no scrim | [longest](b3-tarde-longest-16x9.jpg) | [shortest](b3-tarde-shortest-16x9.jpg) | [sheet](b3-tarde-partway-and-12-spaces.png) |

**B1 · Correo**
- **Why it suits the task:** it is a newspaper at first sight: masthead, folio line, headline across the page, ruled columns. The headline alone says who did what and where.
- **Readability:** headline 84 pixels, article heads 38, text 27; columns of about 35 characters, so a change sometimes breaks across two lines.
- **Density:** middling. A short edition is half a page.
- **Atmosphere:** the strongest period print.
- **Trade-off:** it covers the map, so finding a space means putting the page away. Its headline face is new: Noto Serif Display Bold (below).

**B2 · Isla**
- **Why it suits the task:** the spread answers "where" with a drawing. The spaces are tinted on a map made from the accepted region profiles, and each article's number is on its space.
- **Readability:** headline 70 pixels, heads 32, text 27; the four factions' figures in their own boxes at 38.
- **Density:** the lowest per pixel: the largest page, and a short edition leaves most of it empty.
- **Atmosphere:** a 1950s illustrated weekly, less a newspaper than the others.
- **Trade-off:** it hides the real map to show a smaller one. It needs one tinted export for each space and a place for each number.

**B3 · Tarde**
- **Why it suits the task:** the island, the standings, the ledgers and the Sequence of Play stay in view beside the report, at full brightness, so a space that is named can be seen in the same glance. It opens where the player last read a decision, over controls that are inert anyway.
- **Readability:** headline 58 pixels, heads 30, text 27 in lines of about 66 characters, the most even setting of the three.
- **Density:** the highest. The longest edition fills the window's height to 62 pixels of its foot.
- **Atmosphere:** a café paper on its stick, in the accepted typeface.
- **Trade-off:** with no scrim, only the page and the inert panel say the table is waiting. It hides Save · quit and any control at the right while open. A longer edition than the two drawn needs a second page soonest here.

**Not a proclamation.** Each has a masthead, a folio line, several articles in a fixed order, tables of figures and no faction colour in its print; a faction appears only as a name or its accepted emblem. Notice, broadsheet and circular forms (one text, one author, a seal or a slogan) stay free. "Written from the committed result" stands at the foot of every page.

**Language.** The app is in English (foundation 1), and so is every article. The mastheads are invented Spanish titles, the one exception, for atmosphere; B2 also has the line "Semanario de noticias". A title can be put into English without touching a layout.

**Putting away** is a click, Escape, Enter or Space in all three. **Reopening** lies beside the Propaganda pips in all three, clear of every seat layout: a pile of folded papers (B1), a small cover per edition (B2), the stick with a count (B3). **Too long for a page:** the type does not shrink; the edition continues on a second page, turned with Left and Right. None of the editions below needs it.

## Sample content and its positions

Every number is from the rules authority on build `38497ac`, played through the development seat from a new save at 3840×2160. Each changed value was then worked by hand from the March 2019 Living Rules (SHA-256 `8152cb00…6792`, the pinned PDF) 1.5.1, 1.6, 1.7, 2.3.3, 2.3.4, 2.3.7, 3.2.4, 3.3.4, 4.1, 4.3.3 and 7.3, and agreed. Captures, observations and saves are kept locally under `local/verification/issue-200/`. The wording is illustrative: #207 owns the fixed wording.

| Sheet | Decisions | What it reports |
|---|---|---|
| **Longest** | Card 17, revisions 0 to 12. 26July chooses Terror; asks Syndicate for 2, authorized; rolls Kidnap in Santiago (2); Terror in Oriente, then Santiago; removes its Guerrilla in Oriente; ends Terror | Every kind of change: a transfer, a die roll, pieces flipped and removed, Opposition in two spaces, Control lost, 2 markers, Resources, standings and eligibility |
| **Shortest** | Card 18, revisions 15 to 16, after Directorio's shaded Event. Government Passes | Resources 4 → 7, the Pass box, Syndicate 1st Eligible |
| **Partway** | Card 18, revisions 15 to 19. Government Assaults Havana, then asks 26July for 2 | The Assault so far, and that 26July decides |
| **12 spaces** | Card 18, revisions 15 to 38. Syndicate, Directorio and 26July give all they have; Government Assaults 12 spaces and finishes | The most spaces this build can reach, as reckoned rather than searched: 26 Resources at the start, at least 1 spent on the Terror that Kidnap needs, 2 for each space (R 3.2.4) |

Which faction the human plays does not change a page. The removal in the longest edition is legal and poor play: it is the simplest way for this match to change Control (R 1.7).

## Custody, type and files

- **Captures:** `ctx-*.jpg` are seat captures at 3840×2160 with both piles' top cards blanked before anything was embedded; the count discs over them are redrawn from the accepted export. Both FigCraft sources embed these same files (compared by hash), and every tray in the twelve 16:9 sheets was inspected enlarged. No card tag is open and no card is in the air.
- **Palette:** the `Atlas` variables. Additions, all in the graphics: a wood near the Aid crate's for B3's stick, a lamp yellow for A3's lit lamp and ribbon, and a tint between `ink-secondary` and `rule` (`#7f9496`) for B2's marked spaces. The mastheads are ink, since red is 26July's.
- **Type:** Lato, the accepted face, everywhere, in Regular and Bold; B2's and B3's headlines and mastheads use Lato Black, which is in the same family and licence (SIL OFL 1.1) and is not admitted yet. **B1 adds Noto Serif Display Bold:** Google's Noto project, SIL OFL 1.1, from Ubuntu's `fonts-noto-core` 20201225-2build1 (`NotoSerifDisplay-Bold.ttf`, SHA-256 `8666c2488e681a31670f9fc0b5d59fa4f21f2a0664a76876fe49ebd832f673c7`). Choosing B1 means admitting that file with its notice; no font file is committed here.
- **Type sizes** are pixels of the owner's display, where one pixel is 0.18 mm: newspaper text 27 (4.9 mm), its table and notes 25 (4.5 mm), its smallest caps 20 (3.6 mm) and foot line 22; seat states 23 to 25 (4.1 to 4.5 mm); pills 17 (3.1 mm).
- **Layouts:** `seat-controls-layouts.figcraft` and `newspaper-layouts.figcraft` (FigCraft CLI 0.18.1). Sheets are exports at 1 pixel per unit.
- **Graphics:** `seat-and-newspaper-graphics.vectorcraft` (41 artboards: three sets of six seat marks, place cards, ribbons, dice, mastheads, reopening pieces) and `newspaper-locator.vectorcraft`, made from `atlas-map.vectorcraft`'s thirteen profiles. Their PNG exports are in `graphics/` at 2 pixels per unit.
- **Tools:** VectorCraft CLI 0.24.1 and FigCraft CLI 0.18.1, the builds installed on 2026-11-22 from clean checkouts of `storytold/vectorcraft@88f5ece1` and `storytold/figcraft@5c2a3784` (reflog and timestamp evidence, not a recorded build; both checkouts and binaries unmoved on 2026-11-25). Both were driven through their MCP servers' own tool calls with `.mcp.json`'s arguments, so the read and write roots were the registered ones; the call lists are kept with the local evidence.
- **What this establishes:** competent alternatives at the display's own pixels. It does not establish native behaviour, agent play or the owner's acceptance.
