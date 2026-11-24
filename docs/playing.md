# Playing this build

What you can do in the current build and how to operate it. [README](../README.md) says how to build and launch it. It names a rule effect only where you need it to operate the build; the in-game briefings and the [rules baseline](research/rules-baseline-and-candidates.md) are the rules authority. It does not describe the visuals: each selected design's spec in [`art/atlas/`](../art/atlas/) covers those. Rule interpretations this project made are in [local-rule-policies.md](local-rule-policies.md). An agent driving the build reads [the seat reference](agents/seat.md) instead.

## What the task covers

The native task begins with 26July's decision as 1st Eligible on card 17, *Alberto Bayo*. In faction order, the card shows 26July, Directorio, Government and Syndicate. [First task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8) · [Directorio continuation (#117)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/117)

- **Playable:** Pass, and Terror with optional Kidnap in Santiago, Resource transfers and voluntary removal of 26July's own pieces.
- **At 26July's opening decision**, both Event texts, Rally, March, Attack, and the accompanying Infiltrate and Ambush are listed for inspection. Choosing one is refused without changing the match.
- **Directorio's handoff:** after Terror with Kidnap, inspect Directorio's legal options as 2nd Eligible. Directorio can execute either text of Alberto Bayo. Its Pass and Limited Operation are outside the task.

## Reading the table

- **Map:** the map fills the window, and every panel lies over it ([MAIN-VIEW.md](../art/atlas/MAIN-VIEW.md)). **Home** shows all thirteen spaces clear of every panel.
- **Spaces:** hover a space, its pieces or a count for a tag that names what kind of space it is: a City, a Province with its terrain (Forest, Grassland or Mountain), or an Economic center. A city's or province's tag adds its Control, political opinion and population contribution ([STATUS.md](../art/atlas/STATUS.md)). Clicking a space does nothing except during Terror targeting.
- **Victory standings:** the four emblems at the top show official victory margins. Hover one for its calculation, contributions and conditions ([VICTORY.md](../art/atlas/VICTORY.md)). Zero is the threshold, and a positive Government margin still requires every City at Active Support. No Propaganda victory check runs in this build.
- **Top display:** Aid and the US Alliance at top left; each faction's Resources and the Cash supply at top right. Hover any of them for what it does and what it costs ([TOP.md](../art/atlas/TOP.md)).
- **Marker supply:** in the top band between the US Alliance track and the standings, the two-faced Terror and Sabotage marker beside the number of markers available and "of 20" (R 3.3.4). Each Terror places one, so the number falls by one; a hovered or pinned preview leaves it as it is. At 0 the number turns red and the marker dims ([TOP.md](../art/atlas/TOP.md#the-marker-supply)). It is the one place the count is shown.
- **Available Forces:** the ledger left of the Resources ledger gives, on each faction's row, the pieces it can still place (R 1.4.1): Troops, Police and Bases for the Government, Guerrillas and Bases for 26July and Directorio, Guerrillas and Casinos for the Syndicate, each kind drawn as its piece with its count. A kind with none left keeps its piece, dimmed, beside "0". The counts are the committed match's: a hovered or pinned preview leaves them as they are ([AVAILABLE.md](../art/atlas/AVAILABLE.md)). Hover a faction's row, or focus it with Tab, for its tag: each kind as "3 of 15 · 12 on the map" (Available, of the faction's pieces of that kind, and how many are on the map, a closed Casino included), then the rule that pieces are placed only from Available Forces and that a faction executing may return its own pieces. It opens left of both ledgers and closes when the pointer or focus leaves.
- **Sequence of Play.** The track at the left, over the sea west of the island, records 26July's action on card 17. Hover a box to see what it allows the next faction. When the Event ends, the track shows each faction's adjusted Eligibility for card 18 (R 2.3.7). [Eligibility layout](../art/atlas/ELIGIBILITY.md)
- **Cards:** the two piles at the bottom left are the played pile, with the current card face up on top, and the draw deck, with the preview card face up on top (R 2.2). Each pile is as thick as its cards, and the disc on it gives their number: the played pile's counts the current card and the draw deck's the preview, so at rest the two make 52; a played card crossing between them is in neither. The pips beside them show how many of the four Propaganda cards have been played; the game ends at the fourth. The piles stay in place while the map moves ([main view](../art/atlas/MAIN-VIEW.md#the-card-piles)).
- **Card tags:** hover the current or the preview card on its pile, or focus it with Tab, for its tag: the face at reading size, its number and title, whether it is the current card or the preview, the faction order, the kind of Event, where the Event stands, any lasting Capability, and the rule references ([CARD-TAG.md](../art/atlas/CARD-TAG.md)). Click the card (or press Enter on it) to keep the tag open; Escape closes it.
- **Piles:** hover a pile (its count disc, its side or its caption), or focus it with Tab, for the pile's tag ([PILE-TAGS.md](../art/atlas/PILE-TAGS.md)). The played pile's tag lists every card played, most recent first, by number and title, with the current card marked and the Propaganda cards in bold, and counts the Propaganda cards played; a long list runs into columns. The draw deck's tag gives its count with the preview, the cards face down, the Propaganda cards left and how soon the next can come. That last figure assumes the deck was prepared with exactly equal piles of 12 Events ([local rule policies](local-rule-policies.md#next-propaganda-assumes-equal-piles)). The order of the face-down cards is never shown.
- **Faction names:** running text, such as a briefing or a refusal on the notice slip, names factions by their emblems ([VICTORY.md, "Inline text emblems"](../art/atlas/VICTORY.md#inline-text-emblems)).

## Choosing an action

Hover an option, or focus it with Tab, to open its briefing beside the panel: what it does, why a faction takes it, its cost, what Directorio may do next and the rule reference ([CHOICE.md](../art/atlas/CHOICE.md)). Click an option (or press Enter on it) to pin its briefing. The button at the panel's foot commits only the pinned option.

- **Pass** gives 26July +1 Resource and keeps it Eligible; Directorio becomes 1st Eligible (R 2.3.3). The task then ends.
- **Terror** makes 26July the executing faction until End Terror. You can go back to the action choice until the first real effect ([local policy](local-rule-policies.md#going-back-from-terror)).

## Executing Terror

During Terror the panel lists the steps: Terror in Santiago and Oriente, with Kidnap beneath Santiago; Resource transfer and Remove own pieces; End Terror and Back to the action choice ([EXECUTION.md](../art/atlas/EXECUTION.md)).

- **Previews:** hover or focus a step for its preview, and click to pin it; the foot button commits the pinned step. A preview is computed by the rules authority on a copy of the match. It shows Resources, flipped Guerrillas, returned pieces, Terror markers, support and Control shifts, standings, choices that would become unavailable, the recorded box and Directorio's options. Reading a preview changes nothing, including randomness.
- **The map:** each space is marked selectable, blocked (an in-task action would allow it) or not involved ([HIGHLIGHTS.md](../art/atlas/HIGHLIGHTS.md)). Hover a selectable or blocked space for its popover, and click to keep it open. The popover lists every action there with its preview and the one button that acts.
- **Choosing a space with the keyboard:** Tab past the panel's controls reaches the selectable spaces, then the blocked ones, each focused space showing its tag. Enter or Space opens its popover, or closes it, as a click does. Tab then reaches the popover's rows and its button: Enter on a row shows its preview, and Enter on the button commits. Escape closes the popover and returns to its space.
- **Unavailable targets:** a target 26July cannot act on now stays listed, disabled, with its reason and in-task remedies: "Request a Resource transfer", and for Santiago "Kidnap here first" while Kidnap would take at least 1 Resource. A target already resolved, or lost to removal, has none.
- **Each Terror** costs 1 Resource, activates one Underground Guerrilla, places a Terror marker and shifts toward Active Opposition. Santiago and Oriente can each resolve once, in either order you can afford.
- **End Terror** records Operation Only without Kidnap, which shows Directorio's Pass and Limited Operation. With Kidnap it records Operation with Special Activity and opens Directorio's options, including the Event (R 2.3.4, 5.2). Ending with no space resolved is allowed ([local policy](local-rule-policies.md#zero-space-operation-only)).
- The [BRIEFING ribbon](../art/atlas/CHOICE.md) on the panel's top edge reopens the Terror and Kidnap briefings.

## Inspecting the handoff to Directorio

After Kidnap and End Terror, hover or focus Pass, Limited Operation or either Event text for its briefing. Click or press Enter to pin it. The board and saved position stay at the handoff while you read.

Pin the shaded Event to see the recorded Terror and Kidnap result, the pending activations and the restriction on 26July. Commit
**Choose shaded Event** to open Directorio's execution. You can transfer Resources or
remove Directorio's pieces before you commit **Resolve shaded Event**. Then inspect
the flips and each faction's eligibility for card 18. Card 18 stays in preview until you advance.

## Kidnap in Santiago

Kidnap takes two steps (R 4.1, 4.3.3).

1. **Open** shows the stakes without rolling: Government as the target, the possible proceeds (`min(die, Government's Resources)` for each face), the guaranteed minimum, any Santiago Terror it will owe, and card 17's Event for Directorio. While it is open, only Roll and Cancel are offered. Cancel changes nothing.
2. **Roll** commits the die and transfers up to its result from Government, zero if Government is depleted. The die then tumbles in the panel's tray and lands on the committed face ([KIDNAP.md](../art/atlas/KIDNAP.md)). Click the tray, or press Space or Enter, to land it at once. **Continue Terror** (or Enter) returns to the steps.

Rolling before Santiago's Terror commits 26July to that Terror ([local policy](local-rule-policies.md#owed-terror-after-an-early-kidnap)). Outnumbering and that commitment gate opening and rolling alike.

## Resource transfers

The Resource transfer step opens the transfer modal ([TRANSFER.md](../art/atlas/TRANSFER.md)). Pick the other faction by its emblem tab, then Give to or Ask from, then the amount: type it, or use 1, 2, Max or the slider. Both balances and the limits update as you go, and an amount beyond a limit is refused with its reason.

- **Give** moves Resources from the executing faction on confirmation.
- **Ask** creates a Transfer request that moves nothing yet. The modal turns to the giver's colours. The giver authorizes any amount from 0 to its limit, declines, or waits.
- **Wait** closes the modal and keeps the request pending, also across Save · quit; its tab in the top band reopens it.

Repeated transfers are allowed, and zero is a deliberate non-transfer. A transfer can fund either Terror target without Kidnap, or make Kidnap possible during or after Terror. 26July can transfer Resources once it chooses Terror. Going back or ending Terror withdraws a pending request. During the Event and the free Rally, the executing faction can transfer Resources. These decisions belong to the local player. [Transfer timing](local-rule-policies.md#transfers-open-once-terror-is-chosen) · [Withdrawing pending requests](local-rule-policies.md#a-pending-transfer-request-is-withdrawn)

## Removing your own pieces

"Remove own pieces" opens removal mode (#73). The map marks the executing faction's removable pieces and dims spaces holding none of its pieces ([HIGHLIGHTS.md, "Removal mode"](../art/atlas/HIGHLIGHTS.md#removal-mode-73)). Click a marked piece for the preview, Remove and Cancel, and a quantity selector when it is in a stack. Each Remove returns that many pieces to Available. The mode stays open until Done and survives Save · quit. After resuming, click a piece to reopen its popover.

Supply, Control, standings and legal choices update at once. Earlier Terror, support shifts and Kidnap results stay. While Santiago Terror is owed, its last Underground Guerrilla is locked ([local policy](local-rule-policies.md#owed-terror-after-an-early-kidnap)). During the Event, Directorio can remove its pieces before resolving shaded or choosing the Rally beneficiary, then the beneficiary can remove its own pieces while Rallying.

## Directorio’s unshaded Event

Select **Event · unshaded** and commit it to open Directorio’s beneficiary choice.
Inspect **Grant Rally to 26July** and **Grant Rally to Directorio** to see each
faction’s Base spaces. Directorio can transfer Resources or remove its own pieces
before choosing. The chosen faction then directs the Rally and can use those
controls for its own faction. A Resource request moves nothing until the giver
authorizes it.

Inspect and pin a Rally option before committing it. Its preview shows the placement ceiling from Bases and population, the Available Guerrillas only where fewer remain than the ceiling, the change the choice makes to Available Forces, and the zero Resource cost. **Place 0 Underground Guerrillas** resolves that qualifying space with its pieces and Available Forces unchanged. A qualifying space counts as Neutral for the Rally. Its actual Support or Opposition stays in place. If the beneficiary has no qualifying Base, the Event ends without a Rally decision.

You can save and resume while the Rally is pending. After the last required effect, the panel reports the result and each faction’s eligibility for card 18.

## Advancing to Pact of Caracas

At the completed Alberto Bayo result, inspect the board, the Event effects and each faction's eligibility. Choose **Advance to Pact of Caracas** to play card 18. The advance is saved at once, and then you see it: Pact of Caracas lifts off the draw deck and crosses to the played pile, and the card under it, Sierra Maestra Manifesto, turns over where it lies, its back showing as it turns ([the card advance](../art/atlas/MAIN-VIEW.md#the-card-advance)). The count discs are not shown while a card is in the air; they are back as the cards land, reading 19 and 33. A click anywhere, Space or Enter lands the cards at once and does nothing else. No card or pile tag shows while a card is in the air; a pinned one is back as the cards land. Pact of Caracas then lies on the played pile as the current card and Sierra Maestra Manifesto on the draw deck as the preview, and each card's tag reads it. The Pact's Capability takes effect only if its Event executes.

Government is 1st Eligible and Syndicate is 2nd Eligible. Both skipped an Operation or Event on card 17. Terror and the Event made 26July and Directorio Ineligible. On the shaded branch, Alberto Bayo expressly keeps 26July Ineligible through card 18.

Hover or focus Government's options to see what the rules allow and why a choice is unavailable. Pin **Pass** to review the current balance, the 3-Resource income and the balance after the 49-Resource cap. Select the **Pass** button to commit. Government's marker moves to the Pass box, and the next decision shows the credited income.

Syndicate takes over as 1st Eligible on Pact of Caracas. Inspect its Pass, Event, Operations and Special Activities.

Choose **Assault · Operation Only**, then choose **Cities**, **Provinces**, or **Economic Centers** and a space. Pin a target to see how many pieces the local Troops, Police and terrain let you affect, what remains protected, and what the payment will do. With the inherited cubes still in place, targeting Syndicate in Havana closes the open Casino in place. In Santiago, the Event determines whether one or two 26July Guerrillas return to Available Forces. Voluntary removal of Government cubes can reduce either city's capacity to zero.

Under Firm US Alliance, each payment costs 2 Resources and resolves that space once. Continue in any order you can afford, or choose **Finish Assault**. Government may finish without paying for a space under the [card 18 local policy](local-rule-policies.md#zero-space-government-assault-on-card-18). Government earns no Pass income and Syndicate can inspect Pass and Limited Operation as 2nd Eligible.

Once Government opens Assault, **Resource transfer** lets Government give or request Resources. Pending requests keep their giver's decision across a restart. The authorized amount changes the balance available for the next space.

**Remove own pieces** opens the map's voluntary removal controls for Government. Click a marked Troop or Police stack, select a quantity and confirm to return it to Available Forces. Choose **Done** to return to Assault and inspect its removal capacity with the remaining cubes.

## Camera and keys

| Input | Effect |
|---|---|
| Right-drag or arrow keys | Orbit |
| Wheel or `+`/`-` | Zoom |
| `W` `A` `S` `D` | Pan |
| `Home` | Restore the map framing; also brings back hover tags that extreme framing hid |
| `Tab`, `Shift+Tab` | Move focus through options, steps and controls, then the spaces the open decision offers on the map (selectable, then blocked) and an open popover's rows and button, then the four Available Forces rows, the current and the preview card, the two piles and Save · quit |
| `Enter`, `Space` | Activate the focused control. While a played card is moving, land the cards instead |
| `Escape` | Close a card's tag, unpin a briefing, cancel an open transfer, or close a tag or popover; a space popover closed from the keyboard leaves focus on its space |

The camera answers wherever the pointer is over the map. A scroll, a drag or a click on a panel stays with that panel: it neither moves the camera nor reaches the space beneath. The transfer modal takes camera input while open.

## Saving and quitting

Every decision is saved as it is made. Reading, previews, camera moves and cancelling have no effect on the match. Reading cards never undoes an effect or rerolls.

**Save · quit**, the paper plate in the bottom-right corner, closes the game. Hover it, or focus it with Tab, for its tag; click it, or press Enter on it, to quit. Relaunching with the same `--save` resumes that record, including an open Kidnap, a landed die and a pending request. A card that was still moving when you quit is already played: the cards lie at rest and nothing is replayed. README covers recovering a damaged save.

**Notices.** A slip above Save · quit says why a decision or a control was refused, shows an error such as a save changed in another session (reload before continuing), and before the match opens says that the assets are loading and no save has been created, or why launch stopped. A decision that succeeded shows no slip: its result is on the map, the ledgers and the Sequence of Play track. Click the slip to dismiss it; the next notice replaces it. The loading state and a launch error stay until the match opens or you quit ([MAIN-VIEW.md, "Save · quit and the notice slip"](../art/atlas/MAIN-VIEW.md#save--quit-and-the-notice-slip)).
