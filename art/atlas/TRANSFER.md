# Resource transfer modal

The owner selected **TM-B, Two accounts** in #68 ([selection on #58](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)), from the [#65 proposal](../proposals/issue-65/README.md). #71 built it. The unselected TM-A and TM-C stay proposals.

## What it shows

- **Opening:** the execution panel's "Resource transfer" opens the modal (R 1.5.1: only while 26July executes Terror). The open transfer is saved, so a restart reopens it.
- **Composing (26July):** a dark header band, 26July's account on the left and the counterpart's on the right, chosen by the emblem tabs above it. The "Give to | Ask from" toggle drives the flow arrow, which always points from giver to recipient. Each account shows "now → after".
- **Amount:** a typed field, the quick values 1, 2 and Max, and Bevy's slider. The slider's track spans the giver's balance. A Resource kept for an owed Santiago Terror is hatched, and the slider stops at the limit. A typed amount beyond the limit is refused in red with the rules authority's reason, and the primary button is disabled. The accounts show that amount's effect in red, as the design does, a giver's balance below 0 included.
- **Limits strip:** the giver's balance, the recipient's 49 cap and the owed-Terror reserve. The binding limit is bold.
- **Give** confirms at once. **Ask** creates a Transfer request and hands the modal to the giver's decision: the header, arrow, slider, field and giver's card take the giver's colour, and the tabs and toggle go. The amount is prefilled with the request; the giver authorizes any amount from 0 to its limit, declines, or waits.
- **Wait** closes the modal and keeps the request pending, also across save and quit. A tab in the top band, right of the standings, names the request; its Open shows the decision again.
- **Reserved state:** a request another faction starts, which 26July answers, in 26July's red with a dashed "RESERVED STATE" banner. Nothing in this task creates one.
- **Input beneath:** a scrim covers the map, panels and session area. It is pickable, so no region, piece or panel control is hit through it, and the shared camera is blocked (`TableFacts.blocks_camera`). The top band and the Resources ledger stay clear, so the standings stay visible.
- **Emblems:** running text and buttons (the header note, the captions, the limits strip, the refusal, the footer, the primary button and the pending tab) name factions by #60's inline emblems, as in "Send request: 1 from [S]" (#58 story 60). Headings, caps labels and the account cards' names keep the names. On a red surface (26July's header band and the primary button while 26July composes or decides) 26July keeps its name (#60, accepted in #68); for the same reason a giver keeps its name on the primary button in its own colour ("Syndicate authorizes 1" on green), since its emblem would vanish there. The seat reads every name.
- **Keyboard:** digits and Backspace edit the amount; Tab and Shift+Tab move focus through the modal's controls; Enter activates the focused control or else the primary button; Escape cancels an open transfer, or waits in a giver's decision.

The rules authority decides everything: which transfers exist, each direction's terms (`TransferTerms`) and the refusal of an amount beyond the limit. The counterpart, direction and typed amount are presentation state and are not saved.

## Files

- **Layout source:** `atlas-transfer.figcraft`, the #65 proposal's source with TM-A and TM-C removed. It keeps the `Atlas` colour variables and the `Transfer/*` text styles, and has four pages:
  - **Transfer modal:** the 16:9 frame (6144×3456, reference ×2.7) with 26July asking Syndicate for 2, exported as `atlas-transfer-layout.png`, and the states sheet, exported at half size as `atlas-transfer-states.png`.
  - **Reference-size masters:** the modal in each state as auto layout at the 1920×1280 reference. The runtime rebuilds this auto layout with Bevy's flexbox; `transfer::fig` copies its padding, gaps and sizes.
  - **Production plates:** each frame, field, button, card, band and tab plate at reference size, exported at 3× with its layer name.
  - **VectorCraft graphics (placed).**
- **Graphics source:** `atlas-transfer.vectorcraft`, the proposal's shared and TM-B artboards, exported at 3×: `icon-balance`, `icon-cap`, `icon-reserve`, `icon-wait`, `b-flow-arrow` (`atlas-transfer-arrow.png`, flipped at runtime when the counterpart gives), `b-slider-thumb` (`atlas-transfer-thumb.png`) and `b-reserve-hatch` (`atlas-transfer-hatch.png`, tiled).
- **Plates:** `atlas-transfer-*.png`. The colour variants are `ink` (26July composing), `government`, `26july`, `directorio` and `syndicate` (the giver deciding). Plates are nine-sliced, so hugging buttons keep their corners.
- **Layout record:** `atlas-transfer.layout.json` records the 16:9 frame's nodes. The native regression `native_transfer_modal_matches_layout` compares the running app with it at 3840×2160 (the frame ×0.625) within 4 px. Plates are compared on all four edges. Parts that hug text (buttons, quick values, the toggle) are compared on their top and bottom, and the amount field on its top, bottom and width, because Bevy's text metrics differ from FigCraft's. The deltas are 1.4–3.1 px.
- **Manifest:** `transfer.sha256` admits the runtime exports at launch. A missing or changed file refuses launch.

Accepted components are reused: the faction emblems (`atlas-victory-icon-*.png`) and Lato.

## Limits

- **Shadow:** the modal's drop shadow is not drawn; the plate is the modal's own box.
- **Text widths:** Bevy's Lato advances are narrower than FigCraft's, so hugging buttons are a few pixels narrower and the field sits slightly right.
- **The pending tab's place** is the proposal's ("top band, right of the standings"); it was not tested against every top-display hover tag.
- **Seat reach:** the seat types digits through its own keys (the shared seat sends no text, so Bevy's text-input widget cannot take typed input from it) and reaches the slider by a click on its track. Dragging the thumb from the seat needs the shared seat's `drag`, which arrived after Cuba's `unfolded-tabletop` pin.
- **Display:** legibility is judged at the owner's display, as [the seat reference](../../docs/agents/seat.md#the-owners-display) describes.
