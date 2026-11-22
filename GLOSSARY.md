# Cuba Libre Unfolded

Terms for the game and its players, following the accepted [mission](docs/foundation/01-mission-and-scope.md), [rules source](docs/foundation/02-rules-and-verification.md), [development playtesting boundary](docs/foundation/03-engineering-and-reuse.md), and [player and proclamation policy](docs/foundation/05-players-and-proclamations.md). Rule terms follow the March 2019 Living Rules pinned by the accepted [rules baseline](docs/research/rules-baseline-and-candidates.md).

## Language

**Faction**:
One of the game's four sides: Government, 26July, Directorio, or Syndicate, each with its own victory objective.
_Avoid_: Team (when referring collectively to factions opposing the human)

**Dual-use Event**:
An Event with an unshaded text and a shaded text. The faction executing the Event chooses one of the two, regardless of its faction identity.
_Avoid_: Two-part Event

**Current card**:
The card being played: the top card of the played pile. An Event card's faction order and Event govern the factions' choices on it; a Propaganda card brings a Propaganda Round instead.
_Avoid_: Active card, card in play

**Preview card**:
The draw deck's top card, turned face up, which is played next. It still belongs to the draw deck and counts in its number.
_Avoid_: Next card (R 2.3.8's name for the step that plays it), upcoming card

**Played pile**:
The face-up pile of cards played so far, with the current card on top. Its contents are open to inspection. A Government Momentum card whose Event is in effect stays beside the board, and Reset places it on the pile.
_Avoid_: Discard pile, card history, played cards (when meaning only the cards beneath the current card)

**Draw deck**:
The cards not yet played, face down except for the preview card on top. Of its contents, only the number of cards and the preview card are open to inspection.
_Avoid_: Draw pile, stock

**Faction seat**:
The player role responsible for one faction's decisions, held by a human or an assigned external agent.
_Avoid_: Development seat, built-in bot

**Development seat**:
The agent-operated playtesting role for inspecting and exercising the actual application, distinct from playing an assigned faction.
_Avoid_: Faction seat, opponent

**Proclamation**:
A public written communication attributed to its issuing faction, including rhetoric, offers, or nonbinding promises. Its claims are distinct from actual game consequences.
_Avoid_: Private dispatch, authoritative game record

**Victory standing**:
A faction's current position against its victory requirements, including contributing totals, thresholds and any additional conditions.
_Avoid_: Victory margin (when referring to the complete set of requirements)

**Victory margin**:
The faction-specific numerical measure defined by the rules for ranking factions. It is distinct from satisfying every condition for a Propaganda victory check.
_Avoid_: Victory standing, automatic win

**Operation**:
A faction action such as Terror, resolved through the choices and effects allowed for that action. Completing an Operation does not necessarily complete the current Event card.
_Avoid_: Turn, complete card (when referring only to an Operation), full Operation, Full Terror

**Limited Operation**:
An Operation with no Special Activity that selects a single space; a Garrison, Sweep or March may still draw pieces from several spaces into its one destination. It counts as an Operation.
_Avoid_: Partial Operation, cheap Operation

**Operation Only**:
The record that the 1st Eligible faction executed an Operation without a Special Activity, which limits the 2nd Eligible faction to Pass or a Limited Operation.
_Avoid_: Plain Operation

**Eligible faction**:
A faction that started the current card in the Eligible box. In the card's faction order the leftmost is the 1st Eligible and the next is the 2nd Eligible; when one Passes, the next Eligible faction takes its place.
_Avoid_: Active faction, current player

**Pass**:
An Eligible faction's choice not to act on the card, gaining +1 Resource (+3 for the Government) and remaining Eligible.
_Avoid_: Skip, end turn, zero-space Operation

**Underground Guerrilla**:
A Guerrilla in its concealed state. Terror requires one in each selected space.
_Avoid_: Hidden Guerrilla, inactive Guerrilla

**Active Guerrilla**:
A Guerrilla in its exposed state. It still counts for Control and for outnumbering Police.
_Avoid_: Revealed Guerrilla, used Guerrilla

**Activate**:
To flip an Underground Guerrilla to Active; unless the text specifies an Underground Guerrilla, an already Active one also counts and stays Active. It concerns a piece, never Resources.
_Avoid_: Spend, use (when meaning flipping a Guerrilla)

**Executing faction**:
The faction currently carrying out an Operation, Special Activity or Event. Passing is not executing.
_Avoid_: Active player, current turn (when meaning execution)

**Resource transfer**:
A voluntary movement of Resources from a giver to a recipient, decided by the giver, permitted only while the giver or the recipient is the executing faction. Movements the rules require, such as Kidnap proceeds, are not Resource transfers.
_Avoid_: Donation, gift, payment

**Transfer request**:
One faction's nonbinding ask that another give it Resources. It moves nothing; only the giver's authorization makes a Resource transfer.
_Avoid_: Petition, demand, pending transfer

**Voluntary removal**:
The executing faction's return of its own on-map pieces, Casinos included, to Available Forces during its Operation, Special Activity or Event. The rules set no count limit, and the Syndicate may instead close a Casino.
_Avoid_: Retreat, casualty, elimination

**Special Activity**:
A faction-specific activity that may accompany an eligible Operation, subject to its own timing and location restrictions. Kidnap is a 26July Special Activity that accompanies Terror, in a space that was or will be selected for that Terror.
_Avoid_: Separate turn, mandatory continuation

**Resources**:
A faction's tracked spendable amount, used to pay costs such as Terror and transferable when the rules permit.
_Avoid_: Cash

**Cash**:
A marker carried by a cube or Guerrilla, with specific transfer and use rules. Possessing Cash is distinct from having Resources available to pay a cost.
_Avoid_: Resources, spendable balance

**Aid**:
Foreign assistance: a single level from 0 to 49 that adds to the Government's Resources in Propaganda Round earnings and when an Event says so. Aid itself is not Resources and cannot pay a cost.
_Avoid_: Government Resources, US Alliance

**US Alliance**:
The United States' stance toward the Government, at one of three ordered levels: Firm, Reluctant or Embargoed. A worse level makes the Government's COIN Operations cost more and, at Embargoed, forbids Air Strike.
_Avoid_: Aid, Alliance (on its own)

**Terror marker**:
A marker in a City or Province that hinders later efforts to build Support or Opposition there; Civic Action or Agitation must remove the space's Terror markers before shifting it. Placed by the Terror Operation and by Government Reprisal. Several may occupy one space, and all are removed at Reset.
_Avoid_: Terror (when meaning the marker rather than the Operation), Sabotage marker

**Sabotage marker**:
A marker on an economic center that blocks the Government's earnings from it. Placed by Terror in an economic center or during Government Earnings; at most one may occupy an economic center, and all are removed at Reset. It shares a single supply of 20 with Terror markers.
_Avoid_: Terror marker

**Local policy**:
An interpretation this project applies where the rules are silent, or do not define how software should handle a case. It is not a publisher ruling, and is owner-accepted only where that acceptance is recorded.
_Avoid_: House rule, local ruling

**Layout**:
The arrangement of interface areas on a screen or written document: which areas exist, where they sit, their sizes, grouping and spacing, and how they adapt to the display.
_Avoid_: Graphic, mockup (when referring to the selected arrangement rather than a proposal)

**Graphic**:
A single drawn visual element, such as an icon, marker, region outline, frame or ornament, that a layout places.
_Avoid_: Layout, asset (when referring only to drawn elements)

**Reference pixels**:
The units of the 1920×1280 space in which interface sizes and positions are authored.
_Avoid_: Reference units, design pixels

**Owner's display**:
The physical monitor on which the owner plays and judges layouts, measured in its own pixels and real size.
_Avoid_: Owner's screen, owner's monitor
