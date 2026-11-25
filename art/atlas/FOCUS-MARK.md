# Focus mark

The owner selected **A · Doble filete** on 2026-11-24 ([selection](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/179#issuecomment-7144819750), [studies](../proposals/focus-mark/README.md)), for [#179](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/179): an `ink` line outside a control, across a `paper` gap.

## What it shows

- **One meaning:** the mark is on the control that Enter would press. At most one control carries it, and it stays there wherever the pointer is. It is drawn besides a control's hover, pin, held, chosen and disabled looks, which it does not change.
- **Marked:** the decision panel's options, steps and commits, Kidnap's Cancel, a pending request's Open, the BRIEFING ribbon and its tabs, a space popover's rows and button, the transfer modal's controls, the seat controls, and Save · quit.
- **Not marked:** map spaces, cards, piles, the marker supply and the Available Forces rows. Enter commits nothing there, and focus opens their tags.
- **The transfer modal:** Enter presses the primary button while nothing has focus and while the amount field has it, so the primary carries the mark then, blocked or not (the owner, 2026-11-24). While Tab has reached the field, the field has the light form, a thin `ink-secondary` line.
- **The Kidnap die:** once landed, its Continue carries the mark, because Enter presses it (the owner, 2026-11-24). Nothing is marked while the die or a played card is in the air: Enter lands them.

## Files

| Export | Artboard (reference pixels) | Corner slice | Drawn on |
|---|---|---|---|
| `atlas-focus-mark.png` | 24×24 | 9 | every marked control but the next |
| `atlas-focus-mark-pill.png` | 40×40 | 19 | the round segments of the modal's direction toggle |
| `atlas-focus-mark-field.png` | 24×24 | 9 | the modal's amount field, the light form |

- **Source:** `atlas-focus-mark.vectorcraft`, authored through the VectorCraft MCP from the study's A, and exported at 3×. The CLI is 0.24.1, the build installed on 2026-11-22 from a clean checkout of `storytold/vectorcraft@88f5ece1` ([TOP.md, "The marker supply"](TOP.md#the-marker-supply) gives the same reflog and timestamp evidence; it is not a recorded build). The round cut is not on the study's sheets; the owner accepted it on 2026-11-24 ([#192](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/192)): on the toggle's round segments the square corners left pieces of the toggle's own outline showing.
- **Manifest:** `focus-mark.sha256` admits the three exports. A missing or changed file refuses launch before any save is created or replaced.
- **Placement:** nine-sliced, the mark reaches 4 reference pixels past the control on every side: the gap for 1.25, the line for 2, then 0.75 of paper. Two option rows are 4.1 apart. A popover's row is as wide as its card, so its mark lies inside it, 1 pixel in. The direction toggle's segments touch, so its mark lies over the end of the neighbouring segment, off its words. The mark is the control's child, so it moves, hides and goes with it.

## Limits

- **A popover moved off screen** by the camera hides its mark with it, while its row and button stay keyboard stops: Enter still presses a focused button that is out of sight. #179 left what Enter does unchanged.
- **Checks:** unit tests compare the rule with what the keyboard reads. The native regressions `native_focus_mark_*` compare the drawn marks' controls with the keyboard's at every stop they reach and find the line in captures, at the owner's display and at 3:2, 16:9 and 21:9 ([native regressions](../../docs/agents/native-tests.md)). They do not judge how clear the mark is: that is the owner's.
- **The study's sheets** are composed pictures. The captures of the build are the evidence of what is drawn.
