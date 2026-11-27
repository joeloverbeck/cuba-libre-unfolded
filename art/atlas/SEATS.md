# Seat controls

The owner selected **A1 · Tarjetas** for the seats on 2026-11-25, with the decision panel "**same panel, inert**" drawn on A2's sheets ([selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197#issuecomment-7161227757), [studies](../proposals/issue-200/README.md)), for spec [#197](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/197). [#206](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/206) built them. The sixth state, "Agent at the table", stands from the same selection. What handing a faction over does is in [the play guide](../../docs/playing.md#handing-a-faction-to-an-agent).

## The cards

Four place cards stand on the sea along the bottom, between the Propaganda pips and Save · quit: Government, 26July, Directorio, Syndicate. Each has its faction's colour on its edge, emblem and name, what its seat does, a chair that says the same without words, and its buttons.

| State | Words | Chair | Buttons |
|---|---|---|---|
| The human plays it | You play it | taken, solid | **Hand over** |
| Handed over, no session has joined | Awaiting an agent | empty, dashed | **Copy its line** (filled), **Take back** |
| A session holds it, and nothing is asked of it | Agent at the table | taken, outlined | **Take back** |
| Its session's decision is open | Thinking | taken, with three dots | **Take back** |
| Its session was not heard from for whole minutes while its decision is open | Silent · *n* min, red | taken, with a red clock | **Take back** |
| It decides, and this build plays none of its options | Build ends here | turned over on the floor | **Take back** |

- **The deciding faction's card stands forward,** 36 up with an ink edge, whoever plays it. No card stands forward while the card is to be turned: that is the human's and no faction's seat.
- **The controls work at any time:** over the transfer modal's scrim, while the Kidnap die or a card is in the air, and while an agent's choice waits. An open edition is put away first, by any click; F1 to F4 work over it ([NEWSPAPER.md](NEWSPAPER.md)). A click on one acts at once and lands a card in the air, as any click does.
- **No notice of a hand-over is drawn:** the card shows it. A socket that cannot be opened is an error on the [notice slip](MAIN-VIEW.md#save--quit-and-the-notice-slip).

## The line to copy

**Copy its line** puts the faction's one line on the clipboard and opens a sheet over its card: "Paste this in a terminal, in a Claude Code or Codex session", the line, "Copied. One session plays one seat." and **Copy again**. When the clipboard refuses, the sheet says why and still shows the line. The sheet starts at its own card's left or, for Syndicate, ends with the row. It keeps its foot and grows upward when the line needs more than three lines. Escape or a click on the map puts it away, and it goes when a session joins or the seat is taken back. It holds no agent's choice. The line names the faction, the game's socket and the folders whose instructions its session follows ([opening a session](../../docs/playing.md#opening-a-session)); its words are in one place (`faction_seats::line`).

## The decision panel while an agent decides

The panel stays where it is, with the same heading, sections, rows and places as when the human decides ([CHOICE.md](CHOICE.md), [EXECUTION.md](EXECUTION.md)). What the agent's faction may choose loses its paper plate and stands in regular `ink`; what it may not keeps its pill. Hover, focus and a pin open each row's briefing as always. Where the commit and Kidnap's Cancel were stands the **strip**: a mark, "*Faction*: *state*", one sentence and **Take back**, with the line "Hover an option for its briefing. Only *Faction*'s seat can choose." above it.

| State | Edge and title | Sentence |
|---|---|---|
| Thinking | `ink` | Its agent holds the choice. |
| Silent · *n* min | `action/primary` | Its choice is still open to it. |
| Awaiting an agent | `ink-secondary` | No agent has joined yet. |
| Build ends here, without Take back | `ink-secondary` | No option here is in this build. |

- **Held choices are available for inspection.** A space or piece popover opens and previews as always. Its confirmation and the transfer modal require the decision owner’s human seat. The [pending tab](TRANSFER.md) names a request to an agent’s faction.
- **The human's own withdrawal** (26July's, while another giver answers) keeps its plate. Pinned, its commit stands in the strip's place until it is unpinned, and the line above the strip says so.
- During the Manifesto, the strip’s hint names the held faction’s placement decision. Human 26July can pin its own Resource transfer or voluntary removal and confirm it in the strip’s place. During a human faction’s placement, a pinned choice of held 26July keeps its briefing and shows a disabled confirmation. A pending Resource request names the giver in the subtitle.
- **The instruction lines** that tell the human how to choose are left out of the panel's subtitle, and the line that sends the player to the map says that a click shows what a space would do.

## Files

- **Layout source:** `atlas-seats.figcraft` (FigCraft CLI 0.18.1, the build installed on 2026-11-22 from a clean checkout of `storytold/figcraft@5c2a3784`; reflog and timestamp evidence, not a recorded build). Made from the round's `seat-controls-layouts.figcraft`: the frames `16:9 seats · Directorio thinking` and `16:9 seats · Government silent` (3840×2160, A1's cards with A2's panel) and `Seat states, 1:1`. `atlas-seats-layout.png`, `atlas-seats-silent.png` and `atlas-seats-states.png` are their exports. The captures beneath the frames are the round's, with both piles' top cards blanked.
- **Graphics source:** `atlas-seats.vectorcraft` (VectorCraft CLI 0.24.1, the build installed on 2026-11-22 from a clean checkout of `storytold/vectorcraft@88f5ece1`; reflog and timestamp evidence, not a recorded build): the four cards, the six chairs and the strip's four round marks, kept from the round's `seat-and-newspaper-graphics.vectorcraft`. Every export is pixel-identical to the round's.
- **Layout record:** `atlas-seats.layout.json` holds the first frame's `absoluteBounds`, the area's corner they are measured from, the parts of the panel that keep their accepted places, the 2-unit tolerance and each export's size. A unit test holds the runtime's numbers equal to it, and `native_the_seat_controls_match_the_selected_layout` the drawn parts.
- **Exports,** at 2 px per unit, in `seats.sha256`: `atlas-seat-card-<faction>.png`, `atlas-seat-chair-<state>.png` and `atlas-seat-round-<state>.png` from VectorCraft; `atlas-seat-edge.png`, the plates `atlas-seat-button.png` and `atlas-seat-button-primary.png`, the sheet's `atlas-seat-line.png` (its shadow reaches 42, 32, 42 and 52 past it) and `atlas-seat-line-box.png`, and the strip's `atlas-seat-strip-thinking.png`, `-silent.png` and `-quiet.png` from FigCraft. The buttons, the sheet and its box are nine-sliced. The words are runtime Lato. A missing or changed file refuses launch like every other manifest.
- **Place:** the main view's area `seats` ([MAIN-VIEW.md](MAIN-VIEW.md#areas)). On the owner's display the row is 76 from Save · quit, as the layout draws it. In a window narrower than 16:9 it keeps that edge and shrinks, to stay 24 clear of the Propaganda pips. The strip keeps its place in the decision panel.
- **Palette:** `atlas-seats.variables.json` ([README.md, "Interface palette"](README.md#interface-palette)).

## Differences from the round's sheets

- **The panel's rows and header** keep AC-B's accepted places and type sizes. A2's sheets redrew them up to 21 units away, at rounded sizes; followed, the rows would move each time a decision passed between the human and an agent. The strip and the line above it are where A2 draws them. The words in the pills stay the panel's ("NOT IN THIS TASK", "INSPECT").
- **The cards' soft shadow** drew nothing on the round's sheets (its plate is 0.4% opaque), so none is built.
- **Not drawn in the round,** built in the same form: the panel during Terror, Kidnap, removal, the Event and Assault, the ended panel, and a Transfer request that an agent answers.

## Limits

- **The notice slip and Save · quit's tag** stand above the cards, clear of them whichever stands forward ([MAIN-VIEW.md](MAIN-VIEW.md#save--quit-and-the-notice-slip)). While Directorio's or Syndicate's line sheet is open, both open over the right of that sheet, in front of it, and can cover its **Copy again**: a click puts the slip away, and the tag goes with the pointer.
- **The clipboard** is the desktop's X11 one, and holds the line while the game runs. The development seat's windowless table keeps its own and never touches the desktop's (`table.faction_seats.copied`).
