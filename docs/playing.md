# Playing this build

What you can do in the current build and how to operate it. [README](../README.md) says how to build and launch it. It names a rule effect only where you need it to operate the build; the in-game briefings and the [rules baseline](research/rules-baseline-and-candidates.md) are the rules authority. It does not describe the visuals: each selected design's spec in [`art/atlas/`](../art/atlas/) covers those. Rule interpretations this project made are in [local-rule-policies.md](local-rule-policies.md). An agent driving the build reads [the seat reference](agents/seat.md) instead.

## What the task covers

The native task begins with 26July's decision as 1st Eligible on card 17, *Alberto Bayo*. In faction order, the card shows 26July, Directorio, Government and Syndicate. [First task (#8)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8) · [Directorio continuation (#117)](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/117)

- **Playable:** Pass, and Terror with optional Kidnap in Santiago, Resource transfers and voluntary removal of 26July's own pieces.
- **At 26July's opening decision**, both Event texts, Rally, March, Attack, and the accompanying Infiltrate and Ambush are listed for inspection. Choosing one is refused without changing the match.
- **Directorio's handoff:** after Terror with Kidnap, inspect Directorio's legal options as 2nd Eligible. Directorio can execute either text of Alberto Bayo. Its Pass and Limited Operation are outside the task.

## Reading the table

- **Map:** the map fills the window, and every panel lies over it ([MAIN-VIEW.md](../art/atlas/MAIN-VIEW.md)). **Home** shows all thirteen spaces clear of every panel.
- **Spaces:** hover a space, its pieces or a count for a tag that names what kind of space it is: a City, a Province with its terrain (Forest, Grassland or Mountain), or an Economic center. A city's or province's tag adds its Control, political opinion and population contribution ([STATUS.md](../art/atlas/STATUS.md)). While Terror or Assault offers spaces, the tag ends with a line saying why the space is in its state: what would open it, why nothing can act there, that Terror is owed there, or what the space already paid. Clicking a space does nothing except while Terror or Assault offers spaces.
- **Victory standings:** the four emblems at the top show official victory margins. Hover one for its calculation, contributions and conditions ([VICTORY.md](../art/atlas/VICTORY.md)). Zero is the threshold, and a positive Government margin still requires every City at Active Support. No Propaganda victory check runs in this build.
- **Top display:** Aid and the US Alliance at top left; each faction's Resources and the Cash supply at top right. Hover any of them for what it does and what it costs ([TOP.md](../art/atlas/TOP.md)).
- **Marker supply:** in the top band between the US Alliance track and the standings, the number of Terror and Sabotage markers available, of 20 (R 3.3.4). Each Terror places one, so the number falls by one; a hovered or pinned preview leaves it as it is ([TOP.md](../art/atlas/TOP.md#the-marker-supply) shows how it is drawn, at 0 too). It is the one place the count is shown. Hover it, or focus it with Tab, for its tag: the Terror and the Sabotage markers on the map and the number available of 20, that Terror places no marker when all 20 are on the map, and that Civic Action, Agitation and the Reset Phase return markers. It closes when the pointer or focus leaves.
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

Terror's spaces are chosen on the map, and only there: Santiago and Oriente are no rows in the panel. The panel lists the other steps: Kidnap in Santiago; Resource transfer and Remove own pieces; End Terror and Back to the action choice ([EXECUTION.md](../art/atlas/EXECUTION.md)). Above Kidnap, a line sends you to the map, and each Terror you resolve is listed with what it paid, in the order resolved.

- **Previews:** hover or focus a step for its preview, and click to pin it; the foot button commits the pinned step. It never commits a Terror. A preview is computed by the rules authority on a copy of the match. It shows Resources, flipped Guerrillas, returned pieces, Terror markers, support and Control shifts, standings, choices that would become unavailable, the recorded box and Directorio's options. Reading a preview changes nothing, including randomness.
- **The map:** each space is marked selectable, blocked (an in-task action would allow it), unpayable (only its cost stops it, and nothing here can pay) or not involved ([HIGHLIGHTS.md](../art/atlas/HIGHLIGHTS.md)). Hovering a space shows its tag; click any of them but a not-involved one to open its popover. The popover lists every action there with its preview and the one button that acts: to resolve a Terror, click Santiago or Oriente, choose the Terror row, and press its button.
- **Choosing a space with the keyboard:** Tab past the panel's controls reaches the selectable spaces, then the blocked ones, then the unpayable ones, each focused space showing its tag. Enter or Space opens its popover, or closes it, as a click does. Tab then reaches the popover's rows and its button: Enter on a row shows its preview, and Enter on the button commits. Escape closes the popover and returns to its space.
- **Unavailable targets:** a target 26July cannot act on now, but an in-task action would allow, is marked blocked on the map, and its popover gives the reason and those remedies: "Request a Resource transfer", and for Santiago "Kidnap here first" while Kidnap would take at least 1 Resource. A target already resolved, or lost to removal, has none: the map veils the space when nothing else is offered there, and a resolved one stays in the panel's list with what it paid. While Santiago's Terror is owed, End Terror says so, and Santiago's popover names it.
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

Repeated transfers are allowed, and zero is a deliberate non-transfer. A transfer can fund either Terror target without Kidnap, or make Kidnap possible during or after Terror. 26July can transfer Resources once it chooses Terror. Going back or ending Terror withdraws a pending request. During the Event and the free Rally, the executing faction can transfer Resources. They are made by whoever plays the faction. [Transfer timing](local-rule-policies.md#transfers-open-once-terror-is-chosen) · [Withdrawing pending requests](local-rule-policies.md#a-pending-transfer-request-is-withdrawn)

Syndicate can transfer Resources while Construct is open. If Limited Construct needs funding, the transfer surfaces show the amount held and how much more authorization will move the Resources. Ask the same or another giver, or use **Back to the action choice** before committing. With a request pending, select **Wait** if the giver's modal is open, then select **Syndicate withdraws · back to action choice** in the decision panel to leave and release any held authorizations. After the transfers move, choose where to pay. Pay for your chosen space before giving away the Resources needed for that payment. [Partial funding](local-rule-policies.md#partial-funding-for-limited-construct) · [Limited Construct commitment](local-rule-policies.md#leaving-limited-construct-on-card-18)

## Writing a proclamation

While a card is open, choose **Write a proclamation** or press **P**. Select your faction under **Writing as**. The sheet shows that faction's allowance for the card and counts the draft as you type, up to 400 characters for publication.

Choose **Publish proclamation** to send the exact wording to all four factions. A refusal appears below the field and keeps the draft there. If a new proclamation interrupts the Publish press, the composer asks you to press Publish again after reading. **Return to choice** or **Escape** keeps the draft for this card. Publication leaves the pending decision in place, so a Resource promise still needs the giver's authorization and a separate legal payment.

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

Syndicate takes over as 1st Eligible on Pact of Caracas. It can Pass, choose Construct as Operation Only, or inspect its other options.

Choose **Assault · Operation Only**. The space is then chosen on the map and nowhere else. The map marks every space for the open Assault ([HIGHLIGHTS.md](../art/atlas/HIGHLIGHTS.md) shows how each state is drawn):

- **Selectable:** Assault would remove or close a piece there and Government can pay.
- **Blocked:** it would, but Government cannot pay. Resource transfers open the space: together the other factions can give it enough.
- **Veiled:** Assault would remove nothing there, the space was already assaulted in this Operation, or it would act there but Government cannot pay and cannot be given enough. The last still opens, to show what Assault would do and why it cannot be paid; only a space already assaulted does not.

A selectable space's tag says what the best Assault there would do, such as "Can close 1 Casino".

Click a space to open its popover, or reach it with Tab and press Enter ([Camera and keys](#camera-and-keys)). Tab reaches the selectable spaces first, then the blocked ones, then the veiled ones that can still be chosen. The popover lists the space's plans, one row each: the factions a plan targets, with the split when the capacity is short, and last the plan that removes nothing. Choose a row to see how many pieces the local Troops, Police and terrain let you affect, what remains protected, and what the payment will do. Looking decides nothing and saves nothing. The one button pays and resolves the space at once: **Pay 2 · Assault in Havana**, or **Pay 2 · Assault removes nothing**.

A veiled space where Assault would remove nothing stays choosable: its popover holds the one plan that removes nothing, which pays the space's cost and changes nothing else (R 3.2.4). A space already assaulted opens nothing (R 3.1), nor does one nothing can pay for; its tag says why.

With the inherited cubes still in place, targeting Syndicate in Havana closes the open Casino in place. In Santiago, the Event determines whether one or two 26July Guerrillas return to Available Forces. Voluntary removal of Government cubes can reduce either city's capacity to zero, and the map follows at once.

The panel keeps what is not a place: Government's Resources and the cost per space, one line for each space already assaulted, in the order paid, with what it paid and what was removed or closed, and the rows **Finish Assault**, **Resource transfer** and **Remove own pieces**.

Under Firm US Alliance, each payment costs 2 Resources and resolves that space once. Continue in any order you can afford, or choose **Finish Assault**. Government may finish without paying for a space under the [card 18 local policy](local-rule-policies.md#zero-space-government-assault-on-card-18). Government earns no Pass income. Syndicate can Pass or choose Limited Construct as 2nd Eligible.

Once Government opens Assault, **Resource transfer** lets Government give or request Resources. Pending requests keep their giver's decision across a restart. The authorized amount changes the balance available for the next space, and a blocked space turns selectable as soon as Government can pay. While a transfer is under way or removal mode is open, the map marks no space.

**Remove own pieces** opens the map's voluntary removal controls for Government. Click a marked Troop or Police stack, select a quantity and confirm to return it to Available Forces. Choose **Done** to return to Assault and inspect its removal capacity with the remaining cubes.

### Syndicate passes

Pin **Pass** to preview the balance after gaining 1 Resource, up to 49, and see the four factions’ Eligibility for card 19. Select the **Pass** button to commit. Syndicate, Directorio and 26July are Eligible. Government is Eligible if it Passed and Ineligible if it executed Assault. Pact of Caracas stays current and Sierra Maestra Manifesto stays in preview. The saved result is ready to inspect again after restarting the game.

### Syndicate chooses Construct

After Government Passes, choose **Construct · Operation Only**. After Government finishes Assault, choose **Limited Operation · Construct**. Click a marked city or province, or reach it with Tab and press Enter. Select **Open 1 closed Casino** or **Place 1 closed Casino** when offered, then use the **Pay 5** button. Check the payment and Casino counts before committing. The preview explains when the other plan is unavailable.

The panel shows the remaining balance and records each payment in order. In Operation Only, select another unpaid space or choose **Finish Construct**, [including with zero payments](local-rule-policies.md#zero-space-syndicate-construct-on-card-18). Government stays Eligible. In Limited Construct, pay in one space, then choose **Finish Construct**. Government becomes Ineligible. **Back to the action choice** is available before the first committed effect in Limited Construct.

Finishing makes Syndicate Ineligible for card 19. A restarted game restores the paid spaces, balance and remaining choices.

**Manage own pieces** opens the Syndicate's piece controls. Select an open Casino on the map to compare **Close 1 Casino in place** with **Return 1 Casino to Available**. Closing leaves a closed Casino in that space. Returning frees its slot and increases the Available Forces count. The popover shows changes to Control, open Casinos and the remaining Construct plans. Select a closed Casino to return it, and use the stepper when the selected group holds more than one piece. Tab reaches the group and its controls, and Enter or Space activates each control. **Cancel** or Escape closes the popover without committing. Select **Done** to return to Construct.

An unpaid Limited Construct can refuse a piece action that would leave no affordable legal payment. The popover explains when to pay first. Management stays available after payment, and a paid space remains used even when its Casino changes.

### Continue to Sierra Maestra Manifesto

After Syndicate Passes or finishes Construct, the panel shows the result and Eligibility for card 19. Select **Advance to Sierra Maestra Manifesto** to continue from that position. The Manifesto crosses to the played pile, **The Twelve** turns face up on the draw deck, and the pile counts settle at 20 and 32. 26July is 1st Eligible. Pin an option to see what the rules allow and what this build supports. Reopen the same save to resume at that decision, with the cards at rest and earlier proclamations in the public record.

Pin **Event** and select **Choose Event**. 26July executes it, and the placement decision passes through 26July, Directorio, Syndicate and Government in that order. A faction decides even if it is Ineligible for the card action.

Choose a highlighted map space where the deciding faction already has a piece. The popover names its pieces there, including a closed Casino if that is what qualifies the space. Select a mix to see what comes from Available Forces and how Control and standings would change. Select **Place these pieces** to commit. The Event places two non-Casino pieces together at no Resource cost. If the faction can place only one, the popover offers that mix with **Place this piece**. Tab reaches the same spaces, mixes and confirmation control, and Enter activates them.

Pin **Decline placement** to see who goes next, then select **Decline placement** to commit. Restarting between decisions restores the pending faction.

While 26July executes the Event, it can also pin **26July · Remove own pieces** or **26July · Resource transfer** and select the matching confirmation. In removal mode, click a marked 26July piece, preview the removal and confirm. If it was the faction’s only piece in a space, that space no longer qualifies for its placement. Select **Done · leave removal mode** to resume the pending placement. On a transfer request, the giver decides the amount or declines. Once the request is answered, the same faction resumes its placement.

When the Event ends, the panel records each faction’s placement or decline in order and shows that 26July stays Eligible. Directorio is next as 2nd Eligible and can inspect Pass or Operation with optional Special Activity. Restarting at this menu restores the record and the choices.

## Handing a faction to an agent

An external Claude Code or Codex session can play a faction for you. The four cards along the bottom say who plays each faction ([SEATS.md](../art/atlas/SEATS.md)). **Hand over** gives a faction to an agent, **Copy its line** copies the one line to [paste in its session](#opening-a-session), and **Take back** makes it yours again. F1 to F4 hand Government, 26July, Directorio and Syndicate over and take them back too. A session joins on the game's socket, by default `cuba-libre-players.sock` in `$XDG_RUNTIME_DIR`, or where you launched with `--player-socket`. The game starts no session.

- **One session plays one faction.** A faction nobody joined as waits and never acts. A silent agent's card and panel say for how many minutes it was not heard from; nothing moves for it until it does, or you take its faction back. As it joins, a session is given its faction's seat brief: its objective and standing, its forces, its actions under the rules and which of them this build plays, and its rivals' standings. The brief gives no advice and tells nothing that faction may not know.
- **While an agent's faction decides,** the panel says whose decision it is and its options can be read and not chosen; the map, with its popovers, the cards and the standings too. You turn the card yourself, whichever factions you play.
- **The line to copy** stays on its sheet until a session joins, you take the faction back, you press Escape or you click the map.
- **An agent's choice waits** for a card or the Kidnap die to come to rest and for a landed die to show its result for 2 seconds (Continue ends that sooner). A popover, a pinned or opened briefing, a pinned card tag, a due or open [edition](#the-newspaper), the transfer modal while it asks you to decide, an open proclamation composer and an open Broadsheet also hold the choice. If you decide first and the match moves on, the agent chooses again.
- **Transfer requests** go to whoever plays the giver. An agent that plays 26July keeps its withdrawal while you, as the giver, decide, and you keep yours while an agent does.
- **The socket's path:** `$CUBA_LIBRE_PLAYER_SOCKET`, when it is set, replaces the default for the game and for `cuba-player` alike, and without `$XDG_RUNTIME_DIR` the default is in the temporary directory. `--player-socket` overrides both for the game.
- **Taking a faction back** ends its session and cancels its pending choice. Use F1 to F4 when a page covers the seat card.
- **Save · quit** ends every session. After a relaunch you play all four factions. The same decisions produce the same game position whoever made them.

### Opening a session

1. Once for each computer, and again when you update the checkout, run `cargo build --locked && cargo run --locked --bin cuba-tools -- player-folders`. It installs the session's program, `cuba-player`, and writes two folders, `claude-code` and `codex`, in `${XDG_DATA_HOME:-$HOME/.local/share}/cuba-libre-unfolded/player`. Each holds the [player instructions](player-instructions.md) and that tool's permission for that program alone. The paths of the data directory, the checkout and your home directory may hold only ASCII letters, digits, `/`, `.`, `_` and `-`. A file the command did not write is refused and named, notes aside: look at it, then remove it.
2. Open a fresh terminal for each faction you want an agent to play. Run either launch command printed by `player-folders`. It changes to the installed player folder and starts Claude Code or Codex there. Either tool can play any faction. It asks once whether it may trust the folder.
3. In the game, hand the faction over, copy its line and paste it there. One session plays one faction; open another for the next, in the same folder.

- **Waiting costs nothing.** A session waits inside one command until its faction decides ([ADR 0004](adr/0004-agent-sessions-are-woken-and-do-not-poll.md)). Keep a Codex window open and its turn running: an interrupted turn is not woken until you type.
- **What a session may read.** The game tells it what its faction may know, and its instructions tell it to read nothing else. Claude Code's file tools are also denied the checkout, the saves directory beside the player folder, and the saves directory in the README launch command. A save elsewhere can still be read. Codex cannot be denied a file without losing the permission for its program, so there the instructions are all that keeps a session from the save, which holds the dice and the deck. Your own instructions for every session (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`) and your MCP servers reach a player too.
- **What a session may do.** Codex's sandbox writes only its own folder, and nothing is escalated; an option on its command line can undo that. Claude Code runs no command but `cuba-player` where its sandbox cannot start, as on an Ubuntu that restricts user namespaces, and needs `bubblewrap` and `socat`; in `auto` mode its file tools may write elsewhere if its approval model agrees. In the measured sessions the tools refused writes to the protected files that were tried, including the client program. The checks did not cover `.agents`, `.claude.json` or `AGENTS.md`. [The shared library](https://github.com/joeloverbeck/unfolded-tabletop/blob/cb5c74a184339c39dc555608c92468a1193026b7/agent-players/src/folder.rs) says what was measured.

## The newspaper

When a faction you do not play completes an action, an edition of *El Correo de la Isla* opens over the table and says what changed ([NEWSPAPER.md](../art/atlas/NEWSPAPER.md)): the faction, the action and the spaces, the pieces, Resources, Support and Opposition, Control, markers, standings, eligibility and a die roll. The game writes it from the committed result. One opens partway when you must decide inside another faction's action, such as on a Transfer request or a Rally, and the next one tells only what followed. There is none for a faction you play. A faction that changes hands during its action is told from where it became an agent's, and one you take back is yours from your first decision for it. Until then nothing its agent did is lost: the next edition tells it if you hand the faction over again, or if agents end the action.

- **It opens** once the pieces and the die have come to rest; a hover or the camera does not delay it. While it is open, your controls, the camera and the keys beneath it do nothing, and an agent's next choice waits. F1 to F4 still hand factions over and take them back. A decision you make before it opens is refused with a notice, and nothing is decided.
- **Putting it away** takes a click anywhere, Escape, Enter or Space, and decides nothing. A long edition continues on a second page: Left and Right turn it.
- **Reading it again:** the pile beside the Propaganda pips holds the current card's editions. Click it, or Tab to it and press Enter, for the latest. With more than one, each has a tab under the folio line: click one, or press Tab for the next. The pile goes when you turn the card.
- **Editions are not saved.** One you had not read when you quit is not shown again.

## Read faction proclamations

A faction player's publication opens as a public Broadsheet. Read the issuer, card and the faction's own words. **Return to choice**, Escape, Enter or Space acknowledges it. If another proclamation is waiting, **Next proclamation** opens it in publication order. You can return to the table with later proclamations still unread. While the page is open, agent choices wait. Your pending decision and keyboard focus resume when you return to the table.

Select **Take back** on a faction's seat card while reading to reclaim it and cancel any choice that its agent submitted while the reader held moves.

Select **Proclamations** or press R to reread the public record. Use the previous and next buttons or Left and Right to browse it. If a proclamation extends below the reading area, use the wheel or Up and Down. The factual newspaper reports committed actions under its own masthead.

Reopening a match restores the public record and its unread count. **Proclamations** opens the first unread document, or the first in the record if all have been read. A proclamation you publish is already read. If the table cannot save an acknowledgement, the document stays open so you can retry.

## Camera and keys

| Input | Effect |
|---|---|
| Right-drag | Orbit over the map |
| Wheel or `+`/`-` | Zoom |
| `W` `A` `S` `D` | Pan |
| `Home` | Restore the map framing; also brings back hover tags that extreme framing hid |
| `Tab`, `Shift+Tab` | Move focus through options, steps and controls, then the spaces the open decision offers on the map (selectable, then blocked, then no effect, then those nothing can pay for) and an open popover's rows and button, then the marker supply, the four Available Forces rows, the current and the preview card, the two piles, the card's editions, the seat cards' buttons and Save · quit. A reopened edition turns to the card's next one |
| `P`, `R` | Open the proclamation composer or public archive |
| `Enter`, `Space` | Acknowledge an open Broadsheet, put an edition away, land a moving card, or activate the focused control. A dark line marks the button, row or option that Enter would press. In the transfer modal that is the main button until Tab reaches another control. A focused space, card, pile, marker supply or Available Forces row shows its tag. [Focus mark](../art/atlas/FOCUS-MARK.md) |
| `Left`, `Right` | Browse proclamations in an open Broadsheet, turn an open edition's pages, or orbit the map |
| `Up`, `Down` | Scroll an open Broadsheet or orbit the map |
| `F1`, `F2`, `F3`, `F4` | Hand Government, 26July, Directorio or Syndicate to an agent, or take it back ([above](#handing-a-faction-to-an-agent)) |
| `Escape` | Acknowledge an open Broadsheet, keep the current draft and close the composer, put an edition away, close a card tag or the sheet with a seat's line, unpin a briefing, cancel an open transfer, or close a tag or popover. A space popover closed from the keyboard leaves focus on its space |

The camera answers wherever the pointer is over the map. A scroll, a drag or a click on a panel stays with that panel: it neither moves the camera nor reaches the space beneath. The transfer modal, an open edition and an open Broadsheet take camera input while open.

## Saving and quitting

Every decision is saved as it is made. Reading, previews, camera moves and cancelling have no effect on the match. Reading cards never undoes an effect or rerolls.

**Save · quit**, in the bottom-right corner, closes the game. Hover it, or focus it with Tab, for its tag; click it, or press Enter on it, to quit. Relaunching with the same `--save` resumes that record, including an open Kidnap, a landed die and a pending request. A card that was still moving when you quit is already played: the cards lie at rest and nothing is replayed. README covers recovering a damaged save.

**Notices.** A slip above the seat cards and Save · quit says why a decision or a control was refused, shows an error such as a save changed in another session (reload before continuing), and before the match opens says that the assets are loading and no save has been created, or why launch stopped. A decision that succeeded shows no slip: its result is on the map, the ledgers and the Sequence of Play track. Click the slip to dismiss it; the next notice replaces it. The loading state and a launch error stay until the match opens or you quit ([MAIN-VIEW.md, "Save · quit and the notice slip"](../art/atlas/MAIN-VIEW.md#save--quit-and-the-notice-slip)).
