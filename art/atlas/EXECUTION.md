# Execution panel during Terror

The owner selected **EX-A, Pinned step tag** in #83's round ([selection on #58](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58)), from the [#83 proposal](../proposals/issue-83/README.md). #84 built it. It replaced the two `WORK-2` bootstraps that held the panel during Terror: the interim button stack (`operation.rs`) and the interim hover preview card (`preview.rs`). Both files and their records are gone. The unselected EX-B and EX-C stay proposals.

## What it shows

- **The steps:** a fixed list in AC-B's option rows, in three sections:
  - TERROR: Kidnap on its connector, under the section's lines (below);
  - WHILE 26JULY EXECUTES: Resource transfer and Remove own pieces;
  - END THE OPERATION: End Terror and Back to the action choice.

  Terror's two targets are map spaces, so they are no steps (#171): a space is chosen on the map and nowhere else ([foundation 2](../../docs/foundation/02-rules-and-verification.md#engine-authority)). A click on Santiago or Oriente, or Tab and Enter, opens the space popover ([HIGHLIGHTS.md](HIGHLIGHTS.md)), whose one button commits the Terror.
- **The first section's lines** sit where the two target rows were, at the section label's left edge, above Kidnap's row, which keeps its place:
  - the map line, "Choose a space on the map; its popover pays and resolves.", in a step note's quiet type (Lato Regular 36.45 frame units), top 1058.4;
  - then one record line for each resolved Terror, in the order resolved, with what the rules authority recorded it paid, such as "Terror in Oriente · paid 1 Resource". They use the ended panel's value type (Lato Regular 40.5, line height 54), the first at top 1113.2 and the next 64.8 below. The section holds both targets' lines.

  The lines wait, dimmed with the steps, while removal mode or a transfer holds the panel, as the target rows did. The owed Terror stays visible: End Terror keeps its NOT YET · SANTIAGO OWED pill, and Santiago's popover names the Terror owed.

  A step offered and possible now has the paper plate and a bold label. The others keep a quiet label. Each row's state sits at its right:
  - as a note, such as "opens the transfer" or "records Operation Only";
  - or as a pill in one of four styles: OWED (red outline, also the open step), NOT YET (dashed red: an in-task remedy exists), DONE (ink), and the rule-outlined pill for a step not offered, such as NO LONGER OFFERED or AFTER ROLL OR CANCEL.
- **The step tag:** hovering a step (after the shared 400 ms dwell) or focusing it opens its preview on the accepted 26July tag plate, 8 px left of the panel and top-aligned (AC-B's briefing-tag placement). A click pins it, and the pushpin shows that it is pinned. The tag shows the rules authority's own preview of that choice: COST, ON THE MAP, STANDING, RECORDS, NEXT, THEN UNAVAILABLE, OWED, and the authority's notes under their labels. Notes that cite a rule's procedure form the rules line. For a step that cannot be taken now, the tag shows its reason (NOT YET, UNAVAILABLE or DONE) and its in-task remedies as HF-A remedy chips.
- **The commit:** the full-width primary action at the panel's foot commits only the pinned step, for example "End Terror · records Operation Only". It never commits a Terror: that is the space popover's button. For a step that cannot be taken now, it is disabled and gives the reason and remedies. Hovering another row never changes it.
- **Open steps:** a decision can open a step that then holds the panel. Its row is outlined, and the steps that wait are dimmed to 40 %.
  - **Kidnap opened:** the stakes stay pinned. Cancel (HF-A's outlined secondary) sits above Roll at the foot.
  - **Removal mode:** Done is at the foot. The tag does not open by itself, because the pieces it removes are on the map's east, which the tag covers. The row's tag explains the mode.
  - **The transfer:** TM-B's modal decides. After Wait, the pending request holds the Resource transfer step, and the commit opens it in the giver's colour (TM-B's primary plate). End Terror and Back each withdraw the request when pinned.
- **The die-presented frame** (#74) is unchanged: the tray takes the panel while the die is shown, and Continue Terror returns to the steps.
- **After the Operation:** Operation Only keeps the ended panel with RESULT, ELIGIBILITY, NEXT and NO EVENT. With Kidnap, the panel shows Directorio's options using [AC-B](CHOICE.md). After Pass, the ended panel lists RESULT, ELIGIBILITY and NEXT.
- **After Alberto Bayo:** the ended panel uses the same heading and text slots for the Event result, eligibility for all four factions, and the printed restriction when the shaded text was executed.
- **The clash rule** (#58, extended to step tags in #83's selection): a space or piece popover wins over an open step tag. The tag reopens from its row, and reopening it closes the popover. Escape and a click on the map outside the tag close it, as they do for AC-B's tag. Pinning a step takes the tag's place from the BRIEFING ribbon's tag; the ribbon reopens its own.
- **Keyboard:** Tab and Shift+Tab move through the steps that can be pinned, Kidnap's Cancel, the commit, then the ribbon and its tabs, and then the map's spaces ([the playing guide](../../docs/playing.md)). Enter or Space activates the focused control as a click would.

## Seat names

| Control or part | Seat name |
|---|---|
| A step row (activating it pins or unpins its tag) | `step:kidnap`, `step:transfer`, `step:remove`, `step:end-terror`, `step:back-to-choice` |
| The commit, enabled | the choice it commits, `<revision>:<action>` |
| The commit, disabled | `execution-commit` (activating it shows the reason) |
| The commit opening a pending request | `execution-commit` (TM-B's pending tab already carries the choice `<revision>:reopen`) |
| Kidnap's Cancel | the choice `<revision>:kidnap:cancel` |
| Drawn parts (`table.choice_parts`) | `execution-heading`, `execution-subtitle`, `section:<targets\|during\|end>`, the map line and each resolved space's record ([seat reference](../../docs/agents/seat.md#what-the-table-reports) names them), `step:<action>-label`, `-note`, `-pill`, `-pill-label`, `step:kidnap-connector`, `execution-commit-label`, `execution-cancel`, `ended-label:<LABEL>`, `ended-value:<LABEL>`, and the tag's `briefing-*` parts with `briefing-chips:<LABEL>` and `briefing-chip:<remedy>` |

`table.briefing` reports the shown tag: its `option` (the step), `pinned`, `entries` with their `chips`, and `rules`. `table.choice.pinned` reports the pinned step.

## Files

- **Layout source:** `atlas-execution.figcraft`, a sibling of `atlas-action-choice.figcraft`. It began as a copy of the #83 proposal source, without EX-B and EX-C, so it keeps the `Atlas` variables and the `Choice/*`, `Briefing/*`, `Tag/*` and `Execution/*` text styles. Page "Execution panel (EX-A)" holds:
  - **16:9 frame (6144×3456, reference ×2.7):** revision 14 of #75's run as #171 revised it: Oriente's Terror resolved and recorded, Kidnap rolled, Santiago's Terror owed, and End Terror pinned and refused, with its tag and the disabled commit. Its parts are named as the runtime names them. Export: `atlas-execution-layout.png`. The panel and its tag are the design; the frame's surroundings are a capture of the retired framed view, with its desk, title, terrain key and session panel ([MAIN-VIEW.md](MAIN-VIEW.md)), kept as context that the layout record does not check. Re-cropping the frame to the panel would move every recorded part.
  - **States:** the eight states of the proposal, without the target rows since #171, and with each state's map line and record lines. State 1's blocked-target tag is gone: a blocked target's reason and remedies are in its space popover. Export: `atlas-execution-states.png`, at 0.4×.
  - **Ended panels** (added in #84): Terror ended and passed, in EX-A's grammar. Export: `atlas-execution-ended.png`, at half size.
  - **Production plates:** the three new pills at reference size.
- **Layout record:** `atlas-execution.layout.json` records each part's FigCraft `absoluteBounds` in the 16:9 frame. It includes the Kidnap-opened foot from the states frame and the ended panel, each translated to the 16:9 panel. The native regression `native_execution_panel_matches_layout_and_previews_every_step` compares the running app with it at 3840×2160 within 2 px.
- **Manifest:** `execution.sha256` admits the new pills at launch. A missing or changed file refuses launch.

| Export | Source | PNG | Reference size |
|---|---|---:|---|
| `atlas-execution-pill-owed.png` | FigCraft plate, `action/primary` 1 px outline | 320×60 | 106.4×20 |
| `atlas-execution-pill-not-yet.png` | FigCraft plate, dashed 3/2.5 | 320×60 | 106.4×20 |
| `atlas-execution-pill-done.png` | FigCraft plate, `ink` fill | 320×60 | 106.4×20 |

Accepted plates are reused, not redrawn:
- AC-B's row, active row, rule-outlined pill (`atlas-choice-pill.png`), commit and connector rule;
- HF-A's Cancel and remedy chip;
- TM-B's giver-coloured primary;
- the 26July tag plate, the pushpin, the ribbon, the emblems, the panel frame and Lato.

The runtime nine-slices the pills, chips and Cancel. A pill's rounded ends scale with its height and its middle tiles between them, so the dashes keep their length.

## Changes from the proposal

- **Removal mode's tag** opens on hover or pin rather than by itself, so the pieces it would cover stay clickable. The proposal's state 6 shows it pinned.
- **Kidnap while a request is pending** shows the "AFTER THE TRANSFER" pill instead of a dimmed plate, because Kidnap is not offered then.
- **The ended panels** are new: the proposal covered execution only, and #84 removes every layout invented in code.
- **Previews** name a space in full ("Santiago de Cuba: …") and show margins with a hyphen-minus, as the rules authority writes them.
- **Terror's targets** (#171) left the step list for the map, which foundation 2 makes the one place to choose a space. The first section keeps Kidnap and gains the map line and the record lines, which carry what the DONE · PAID pills said. The layout source and its record were revised with FigCraft CLI 0.18.1. The map line's words are the agent's, under the proposal #166's activation accepted; the owner judges them in the native review.

## Limits

- **Italic:** the rules line is drawn in Lato Regular, because no italic face is admitted (as in AC-B).
- **Dimming** is runtime opacity (40 %), as the layout sets it on the waiting rows.
- **Pill words** are short wordings of observed facts, such as "SANTIAGO OWED". The rules authority's full reason is in the tag.
- **Display:** legibility is judged at the owner's display, as [the seat reference](../../docs/agents/seat.md#the-owners-display) describes.
