# Action choice and briefings

The owner selected **AC-B, Briefing tag** in #68 ([selection on #58](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/58#issuecomment-6940684089)), from the [#64 proposal](../proposals/issue-64/README.md). #69 built it. The unselected AC-A and AC-C stay proposals.

## What it shows

- **The panel at the action choice:** the 26July emblem and "Choose 26July's action", then every option in the game's order. Pass and Terror are playable rows on a paper plate. The others carry a "NOT IN THIS TASK" pill, and Kidnap carries "OFFERED ONCE TERROR IS CHOSEN".
- **The briefing tag:** it opens on the accepted 26July tag plate, 8 px left of the panel and top-aligned with it. Hover (after the shared 400 ms dwell) or keyboard focus shows an option's briefing. A click pins it, and the pushpin shows that it is pinned. The active row is outlined, with the red marker.
- **The commit:** the full-width action at the panel's foot. It commits only the pinned option ("Pass · +1 Resource", "Choose Terror"). For an option not in this task it is disabled and gives the reason. Hovering a row never changes it, so moving the pointer to the button cannot change what it commits.
- **During Terror:** the BRIEFING ribbon hangs from the panel's top edge. Clicking it opens the tag with Kidnap and Terror tabs, and clicking it again closes the tag. A space popover on the map wins over the tag while it is open (the owner's clash rule). The tag reopens from the ribbon.
- **Keyboard:** Tab and Shift+Tab move focus through the options and the commit, or the ribbon and its tabs. Enter or Space activates the focused control as a click would, and Escape unpins the briefing or closes the ribbon's tag.

## Briefing content

The owner narrowed the briefings for #69. A briefing describes its option in general:
- what is done in eligible spaces, and why a faction takes it;
- its cost against current Resources;
- what Directorio may do next;
- the rule reference.

It names no province and computes no space's result. The map popover previews those (#72). Pass keeps its exact numbers from the rules authority's preview, because it involves no space. Briefings state facts and never recommend a move. The content lives in `src/tabletop/briefing.rs`.

## Files

- **Layout source:** `atlas-action-choice.figcraft`, copied from the #64 proposal's AC-B frames. It keeps the `Atlas` colour variables and the `Choice/*`, `Briefing/*` and `Tag/*` text styles, and has three frames:
  - **16:9 frame (6144×3456, reference ×2.7):** the action choice with Terror pinned. Its parts are named as the runtime names them. Its export is `atlas-action-choice-layout.png`.
  - **States:** Pass hovered, Rally pinned, and during Terror with the ribbon and tabs. Its export is `atlas-action-choice-states.png`, at half size.
  - **Production plates:** the exported plates, with their text removed.
  - **Kidnap die presented (16:9, added at #74's merge):** tray C composed into the AC-B panel while the die is shown. Its export is `atlas-action-choice-die.png`; `KIDNAP.md` describes it and `atlas-kidnap-tray.layout.json` records it.
- **Changes from the proposal:**
  - **Pin:** the tag frames no longer clip their content, so the pushpin is drawn whole above the tag's top edge.
  - **Example text:** the Terror and Kidnap briefings follow the owner's refinement. The Terror tag has no RESULT entry, and the example text names no province.
- **Graphics source:** `atlas-briefing.vectorcraft`, with the `pin` and `reminder-ribbon` artboards from the proposal's `briefing-eligibility-graphics.vectorcraft`. They are drawn at the reference size and exported at 3×.
- **Layout record:** `atlas-action-choice.layout.json` records each part's FigCraft `absoluteBounds` in the 16:9 frame. The native regression `native_action_choice_matches_layout_and_briefs_every_option` compares the running app with it at 3840×2160 within 2 px.
- **Manifest:** `choice.sha256` admits the exports at launch. A missing or changed file refuses launch.

| Export | Source | PNG | Reference size |
|---|---|---:|---|
| `atlas-choice-row.png` | FigCraft plate (Pass row) | 1356×96 | 452×32 |
| `atlas-choice-row-active.png` | FigCraft plate (Terror row, active) | 1356×96 | 452×32 |
| `atlas-choice-pill.png`, `atlas-choice-pill-wide.png` | FigCraft status pills | 320×60, 587×60 | 106.4×20, 195.6×20 |
| `atlas-choice-commit.png`, `atlas-choice-commit-disabled.png` | FigCraft primary action | 1356×156 | 452×52 |
| `atlas-briefing-tab-{kidnap,terror}[-active].png` | FigCraft tabs | 190×78, 172×78 | 63.1×26, 57.2×26 |
| `atlas-briefing-rule.png` | FigCraft separator (also the Special Activity connector) | 1260×3 | 420×1 |
| `atlas-briefing-pin.png` | VectorCraft `pin` | 72×72 | 30×30 in the tag |
| `atlas-briefing-ribbon.png` | VectorCraft `reminder-ribbon` | 108×336 | 36×112 |

Accepted components are reused: the 26July tag plate (`atlas-victory-tag-26july.png`, stretched to content), the emblems, the panel frame and Lato.

## Limits

- **Italic:** the rules line is italic in the layout. The runtime draws it in Lato Regular because no italic face is admitted.
- **Focus mark:** keyboard focus on an option shows its outline and briefing. Focus on the commit, ribbon or a tab has no visible mark of its own, so the seat reports focus.
- **Execution panel:** during Terror the panel is the selected EX-A design (#84, `EXECUTION.md`), which reuses AC-B's rows, pills, tag, pin and commit. The die-presented frame holds the panel while the Kidnap die is shown.
- **Display:** legibility is judged at the owner's display, as [the seat reference](../../docs/agents/seat.md#the-owners-display) describes (#16).
