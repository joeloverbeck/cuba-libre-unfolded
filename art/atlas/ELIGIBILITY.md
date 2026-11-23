# Eligibility indicator

The owner selected **EL-A, Sequence track** in #68 ([selection on #58](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)), from the [#64 proposal](../proposals/issue-64/README.md). #70 built it. The unselected EL-B and EL-C stay proposals.

## What it shows

The track lies over the sea west of the island, 60 reference px from the window's left edge and 178 from its top, where the [main view layout](MAIN-VIEW.md) places it. It reads only the observation's Eligibility record (#61): the presentation derives nothing.

- **Header:** the ruled "CARD 17 · SEQUENCE OF PLAY" strip.
- **Eligible box:** card 17's four factions in card order (R 2.3.2), each with its number, emblem, name and an Eligible check. The 1st and 2nd Eligible factions carry the 1st and 2nd ribbons and bold names. After a Pass, Directorio is 1st and Government 2nd (R 2.3.3).
- **Action boxes:** Pass, Operation Only, and Operation + Special Activity. Once 26July's action is recorded, its emblem moves into that box, which takes the heavier ink outline. Its row then shows a dashed vacant slot and "acted: in its box below", and keeps its Eligible check. During Terror no box is occupied yet.
- **Ineligible box:** empty during card 17, with "None on card 17: Eligibility changes after the card (R 2.3.7)".
- **Hover:** each action box is a hover target (`eligibility:pass`, `eligibility:operation-only`, `eligibility:operation-special-activity`). After the shared 400 ms dwell, a tag opens 8 px right of the panel with a leader to the box. An occupied box's tag says what 26July did and what it allows Directorio (R 2.3.3, 2.3.4). An empty box's tag states that box's rule. Every tag ends with "Nobody becomes Ineligible during this card: Eligibility is adjusted after it (R 2.3.7)". The tag closes when the pointer leaves, like the other hover tags. Tags state facts and never recommend a move.
- **Not shown:** the next card's order.
- **Card-end adjustment:** after Directorio's Event, the same four rows show eligibility for card 18. The role ribbons clear, the Ineligible box lists the Ineligible factions, and 26July's earlier action remains in the recorded action section. The ended panel explains the reasons.

## Files

- **Layout source:** `atlas-eligibility.figcraft`, copied from the #64 proposal's EL-A frames (FigCraft CLI 0.18.1). It keeps the `Atlas` colour variables and the `Indicator/*` and `Hover tag/*` text styles, and has three frames:
  - **16:9 frame (6144×3456, reference ×2.7):** the track after End Terror with Kidnap, with the Operation + Special Activity box's tag. Its export is `atlas-eligibility-layout.png`.
  - **States:** the opening (unchanged all through Terror), after Pass with the Pass tag, and after Operation Only with its tag. Its export is `atlas-eligibility-states.png`, at half size.
  - **Production plates:** the two panel heights and the occupied box, with their text and graphics made transparent so the auto layout keeps its sizes.
- **Changes from the proposal:** the proposal's other alternatives and its action-choice page are removed. Its hidden layers are deleted, and every part is named as the runtime names it. The after-Pass Directorio ribbon, which showed the 1st ribbon under a "tag-2nd" name, is renamed to its role.
- **Graphics:** the six VectorCraft graphics are the proposal's exports from `../proposals/issue-64/briefing-eligibility-graphics.vectorcraft`, byte for byte (drawn at reference size, exported at 3×). `tests/integration/eligibility.rs` checks this.
- **Layout record:** `atlas-eligibility.layout.json` records each part's FigCraft bounds in the 16:9 frame, for all four states and the tag. The native regression `native_eligibility_indicator_matches_layout_in_every_state` compares the running app with it at 3840×2160, within 2 px. The record was read from the saved document by an editor-side script kept with the #70 evidence.
- **Manifest:** `eligibility.sha256` admits the nine runtime files at launch. A missing or changed file refuses launch. Like all art, the indicator is not part of a save's identity.

| Export | Source | PNG | Reference size |
|---|---|---:|---|
| `atlas-eligibility-panel-open.png` | FigCraft plate (the opening's panel) | 696×1141 | 232×380 |
| `atlas-eligibility-panel.png` | FigCraft plate (once an action is recorded) | 696×1152 | 232×384 |
| `atlas-eligibility-box-occupied.png` | FigCraft plate (occupied box) | 630×90 | 210×30 |
| `atlas-eligibility-header.png` | VectorCraft `track-header` | 600×90 | 212×30 |
| `atlas-eligibility-tag-1st.png`, `-tag-2nd.png` | VectorCraft `tag-1st`, `tag-2nd` | 156×66 | 36×15 |
| `atlas-eligibility-eligible.png`, `-ineligible.png` | VectorCraft `mark-eligible`, `mark-ineligible` | 54×54 | 14×14 |
| `atlas-eligibility-vacant.png` | VectorCraft `slot-vacant` | 120×120 | 20×20 |

Accepted components are reused: the faction emblems, the top-display tag plate (`atlas-top-tag.png`, stretched to content), the status leader, the briefing rule and Lato. `atlas-eligibility-ineligible.png` is admitted so that an Ineligible flag in the observation would show; nobody is Ineligible on card 17.

## Limits

- **Italic:** the "acted" note, the Ineligible note and the tag's last line are italic in the layout. The runtime draws them in Lato Regular because no italic face is admitted.
- **Tag text:** the layout's tag text is example content. The runtime's wording follows the observation. It adds "Government is now 2nd Eligible" to the Pass tag, and labels the zero-space reading as local policy in the Operation Only tag. Inline emblems follow #60's first-mention rule, so the body can wrap one line longer than the layout.
- **Other aspects:** foundation 4 requires only the 16:9 frame. At 3:2 (1920×1280), the camera frames Cuba wider. The track then covers Pinar del Río's pieces, and its boxes take their hover. #70 records this as an open limit with a capture; it does not invent a 3:2 layout.
- **Keyboard:** the boxes are reached by pointer through the seat (point and activate). Tab does not reach them, as it does not reach the top display's hover targets.
- **Display:** legibility is judged at the owner's display, as [the seat reference](../../docs/agents/seat.md#the-owners-display) describes.
