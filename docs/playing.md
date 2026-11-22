# Playing this build

What you can do in the current build and how to operate it. [README](../README.md) says how to build and launch it. It names a rule effect only where you need it to operate the build; the in-game briefings and the [rules baseline](research/rules-baseline-and-candidates.md) are the rules authority. It does not describe the visuals: each selected design's spec in [`art/atlas/`](../art/atlas/) covers those. Rule interpretations this project made are in [local-rule-policies.md](local-rule-policies.md). An agent driving the build reads [the seat reference](agents/seat.md) instead.

## What the task covers

The build plays one task from [#8](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8): 26July's decision as 1st Eligible on card 17, whose faction order is 26July, Directorio, Government, Syndicate.

- **Playable:** Pass, and Terror with optional Kidnap in Santiago, Resource transfers and voluntary removal of 26July's own pieces.
- **Listed but refused** as "not in this task", without changing the match: card 17's Event (both texts), Rally, March and Attack, with Infiltrate and Ambush under the Operations they accompany.
- **Directorio's handoff:** after Terror with Kidnap, inspect Directorio's legal options as 2nd Eligible. The two texts of Alberto Bayo are available for preview. Directorio's Pass and Limited Operation are outside the task.

## Reading the table

- **Spaces:** hover a city or province, its pieces or a count for a tag with its Control, political opinion and population contribution ([STATUS.md](../art/atlas/STATUS.md)). Economic centers have no tag. Clicking a space does nothing except during Terror targeting.
- **Victory standings:** the four emblems at the top show official victory margins. Hover one for its calculation, contributions, conditions and available pieces ([VICTORY.md](../art/atlas/VICTORY.md)). Zero is the threshold, and a positive Government margin still requires every City at Active Support. No Propaganda victory check runs in this build.
- **Top display:** Aid and the US Alliance at top left; each faction's Resources and the Cash supply at top right. Hover any of them for what it does and what it costs ([TOP.md](../art/atlas/TOP.md)).
- **Sequence of Play:** the track in the map's western margin shows card 17's Eligibility record and the box 26July's action lands in ([ELIGIBILITY.md](../art/atlas/ELIGIBILITY.md)). Hover a box for what it allows the next faction. Nobody turns Ineligible inside the task, because Eligibility changes only after the card (R 2.3.7). The next card's order is not shown.
- **Cards:** "Read both cards" opens the current and preview cards' complete faces.
- **Faction names:** running text names factions by their emblems, as in "[S] gave 1 to [26]. Saved." ([VICTORY.md, "Inline text emblems"](../art/atlas/VICTORY.md#inline-text-emblems)).

## Choosing an action

Hover an option, or focus it with Tab, to open its briefing beside the panel: what it does, why a faction takes it, its cost, what Directorio may do next and the rule reference ([CHOICE.md](../art/atlas/CHOICE.md)). Click an option (or press Enter on it) to pin its briefing. The button at the panel's foot commits only the pinned option.

- **Pass** gives 26July +1 Resource and keeps it Eligible; Directorio becomes 1st Eligible (R 2.3.3). The task then ends.
- **Terror** makes 26July the executing faction until End Terror. You can go back to the action choice until the first real effect ([local policy](local-rule-policies.md#going-back-from-terror)).

## Executing Terror

During Terror the panel lists the steps: Terror in Santiago and Oriente, with Kidnap beneath Santiago; Resource transfer and Remove own pieces; End Terror and Back to the action choice ([EXECUTION.md](../art/atlas/EXECUTION.md)).

- **Previews:** hover or focus a step for its preview, and click to pin it; the foot button commits the pinned step. A preview is computed by the rules authority on a copy of the match. It shows Resources, flipped Guerrillas, returned pieces, Terror markers, support and Control shifts, standings, choices that would become unavailable, the recorded box and Directorio's options. Reading a preview changes nothing, including randomness.
- **The map:** each space is marked selectable, blocked (an in-task action would allow it) or not involved ([HIGHLIGHTS.md](../art/atlas/HIGHLIGHTS.md)). Hover a selectable or blocked space for its popover, and click to keep it open. The popover lists every action there with its preview and the one button that acts.
- **Unavailable targets:** a target 26July cannot act on now stays listed, disabled, with its reason and in-task remedies: "Request a Resource transfer", and for Santiago "Kidnap here first" while Kidnap would take at least 1 Resource. A target already resolved, or lost to removal, has none.
- **Each Terror** costs 1 Resource, activates one Underground Guerrilla, places a Terror marker and shifts toward Active Opposition. Santiago and Oriente can each resolve once, in either order you can afford.
- **End Terror** records Operation Only without Kidnap, which shows Directorio's Pass and Limited Operation. With Kidnap it records Operation with Special Activity and opens Directorio's options, including the Event (R 2.3.4, 5.2). Ending with no space resolved is allowed ([local policy](local-rule-policies.md#zero-space-operation-only)).
- The [BRIEFING ribbon](../art/atlas/CHOICE.md) on the panel's top edge reopens the Terror and Kidnap briefings.

## Inspecting the handoff to Directorio

After Kidnap and End Terror, hover or focus Pass, Limited Operation or either Event text for its briefing. Click or press Enter to pin it. The two Event briefings include the recorded Terror targets, payments and Kidnap roll. The board and saved position stay at the handoff while you read.

## Kidnap in Santiago

Kidnap takes two steps (R 4.1, 4.3.3).

1. **Open** shows the stakes without rolling: Government as the target, the possible proceeds (`min(die, Government's Resources)` for each face), the guaranteed minimum, any Santiago Terror it will owe, and card 17's Event for Directorio. While it is open, only Roll and Cancel are offered. Cancel changes nothing.
2. **Roll** commits the die and transfers up to its result from Government, zero if Government is depleted. The die then tumbles in the panel's tray and lands on the committed face ([KIDNAP.md](../art/atlas/KIDNAP.md)). Click the tray, or press Space or Enter, to land it at once. **Continue Terror** (or Enter) returns to the steps.

Rolling before Santiago's Terror commits 26July to that Terror ([local policy](local-rule-policies.md#owed-terror-after-an-early-kidnap)). Outnumbering and that commitment gate opening and rolling alike.

## Resource transfers

The Resource transfer step opens the transfer modal ([TRANSFER.md](../art/atlas/TRANSFER.md)). Pick the other faction by its emblem tab, then Give to or Ask from, then the amount: type it, or use 1, 2, Max or the slider. Both balances and the limits update as you go, and an amount beyond a limit is refused with its reason.

- **Give** moves 26July's Resources on confirmation.
- **Ask** creates a Transfer request that moves nothing yet. The modal turns to the giver's colours. The giver authorizes any amount from 0 to its limit, declines, or waits.
- **Wait** closes the modal and keeps the request pending, also across Save / quit; its tab in the top band reopens it.

Repeated transfers are allowed, and zero is a deliberate non-transfer. A transfer can fund either Terror target without Kidnap, or make Kidnap possible during or after Terror. These are local faction decisions, not external agent players or binding promises. Transfers are offered only while 26July executes ([local policy](local-rule-policies.md#transfers-open-once-terror-is-chosen)), and going back or ending Terror withdraws a pending request ([local policy](local-rule-policies.md#a-pending-transfer-request-is-withdrawn)).

## Removing your own pieces

"Remove own pieces" opens removal mode (#73). The map marks 26July's removable Guerrillas and Base and dims every other space ([HIGHLIGHTS.md, "Removal mode"](../art/atlas/HIGHLIGHTS.md#removal-mode-73)). Click a piece to keep its popover open, with a quantity for a stack, the preview, Remove and Cancel. Each Remove returns that many pieces to Available, and the mode stays open until Done. The mode survives Save / quit; an open popover does not.

Supply, Control, standings and legal choices update at once; earlier Terror, support shifts and Kidnap results stay. While Santiago Terror is owed, its last Underground Guerrilla is locked ([local policy](local-rule-policies.md#owed-terror-after-an-early-kidnap)). Other factions cannot remove their pieces in this Operation.

## Camera and keys

| Input | Effect |
|---|---|
| Right-drag or arrow keys | Orbit |
| Wheel or `+`/`-` | Zoom |
| `W` `A` `S` `D` | Pan |
| `Home` | Restore the map framing; also brings back hover tags that extreme framing hid |
| `Tab`, `Shift+Tab` | Move focus through options, steps and controls |
| `Enter`, `Space` | Activate the focused control |
| `Escape` | Unpin a briefing, cancel an open transfer, or close a tag or popover |

The card reader and the transfer modal take camera input while open.

## Saving and quitting

Reading, previews, camera moves and cancelling have no effect on the match. Reading cards never undoes an effect or rerolls. **Save / quit** exits, and relaunching with the same `--save` resumes that record, including an open Kidnap, a landed die and a pending request. README covers recovering a damaged save.
