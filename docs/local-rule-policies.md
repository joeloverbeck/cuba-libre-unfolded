# Local rule policies

This record lists each [local policy](../GLOSSARY.md) the app applies. The app labels each one as local policy where it explains a choice or refusal (foundation 2, `AUTH-3`).

Each entry gives the rule or case it interprets, the policy, and where it was decided. Add a policy here when a change adopts one; README and the [play guide](playing.md) link here rather than restating them.

## Accepted by the owner

The owner's [acceptance of the rules baseline and tactical scenario](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/2#issuecomment-6885871005) names these two as accepted interpretations. The reasoning is in the [tactical handoff](research/tactical-slice-handoff.md), under "Commitment policy proposed for acceptance".

### Owed Terror after an early Kidnap

**Rules:** R 4.1 and 4.3.3 require Kidnap's accompanying Terror and allow flexible timing. They don't say what happens when a player tries to destroy that future obligation.

**Policy:**
- A Kidnap before Santiago's Terror commits 26July to that Terror. Its 1-Resource payment and one Santiago Underground Guerrilla must be guaranteed before rolling. Feasibility is judged without assuming a donation or a favourable die.
- Until Santiago's Terror resolves, spending or transfers cannot consume the reserved Resource, and End Terror is refused.
- While that Terror is owed, only the last Santiago Underground Guerrilla is protected from removal. Unrelated removal remains legal.
- Before any Terror is owed, nothing is protected: removal can defeat strict outnumbering or remove every piece.
- After the Terror resolves, End Terror is always possible, even after every remaining piece is removed or every Resource is given away.

### Zero-space Operation Only

**Rules:** R 3.1 and 3.3.4 set no minimum number of Terror spaces, whereas R 4.1 requires at least one space for a Special Activity.

**Policy:** End Terror with no resolved space and no Kidnap is an Operation Only, and earns no Pass income.

**Rules consequences, not policy:** as after any Operation Only, Directorio may only Pass or take a Limited Operation, and 26July is Ineligible after the card. The preview warns of all of this.

### Zero-space Government Assault on card 18

**Rules:** R 3.1 and 3.2.4 allow selection of any spaces and require payment for each selected space. They state no explicit minimum. R 2.3.3 gives Government income for Pass.

**Policy:** In this continuation Government may open Assault and deliberately finish without selecting a space, even with fewer Resources than the space cost. Finish records Operation Only, earns no Pass income, and preserves any side decisions already committed. Each selected space still requires its full payment. The owner accepted this interpretation for card 18. [Owner ruling on #127](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/127#issuecomment-7080815790)

Syndicate becomes 2nd Eligible and may Pass or choose a Limited Operation under R 2.3.4.

### Zero-space Syndicate Construct on card 18

**Rules:** R 3.1 and 3.3.5 allow selection of qualifying spaces and require payment in each selected space. They state no minimum for Operation Only.

**Policy:** After Government Passes, Syndicate may finish Construct with zero payments. It records Operation Only, earns no Pass income, and keeps any side effects already committed. Syndicate becomes Ineligible and Government stays Eligible for card 19 (R 2.3.7). [Spec #215](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/215)

### Leaving Limited Construct on card 18

**Rules:** R 2.3.5 requires a Limited Operation to take place in one space. R 3.3.5 requires full payment in every selected space, and R 2.3.7 makes Syndicate Ineligible after it executes Construct.

**Policy:** Before the first committed effect, Syndicate may choose **Back to the action choice** and reconsider. A nonzero Resource transfer commits the attempt and stands. A request, a decline or an answer of zero leaves Syndicate free to go back if it has already committed no effect. Going back withdraws any pending request. Before payment, a side decision that commits the attempt or changes the committed position must leave at least one legal plan and 5 Resources to pay for it. This applies to outgoing transfers and Casino management. After payment, the ordinary transfer limits apply. [Spec #215](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/215)

### Partial funding for Limited Construct

**Rules:** R 1.5.1 gives the giver control of the actual amount. R 1.8 caps the recipient at 49 Resources, and R 2.3.5 and 3.3.5 require one paid space in Limited Construct.

**Policy:** While an uncommitted Limited Construct needs Resources for its first legal payment, an insufficient authorization holds its amount conditionally. It uses the giver's uncommitted balance and reserves that much room under Syndicate's cap. As soon as the actual authorizations can fund a legal plan, every held amount and the final reply move together. Syndicate then chooses where to pay and keeps any excess. Leaving Limited Construct releases every unspent authorization. [Spec #215](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/215)

### Next Propaganda assumes equal piles

**Rules:** Deck Preparation splits the Event cards "into 4 roughly (or, if desired, exactly) equal piles" and shuffles one Propaganda card into each. R 2.2 opens the played cards and the deck's number to inspection, not the order of the deck.

**Policy:** The draw deck's tag says how soon the next Propaganda card can come assuming the deck was prepared with exactly equal piles of 12 Events, as this build's prepared deck was. The n-th Propaganda card then lies in the n-th pile of 13 cards, and the figure follows from the cards played, the Propaganda cards among them and the face-up preview alone. The tag states the assumption. With roughly equal piles the true place can lie outside the figure.

**Decided:** the owner accepted the line "Next Propaganda within the next 8 cards" with this assumption stated when selecting the main view and card tags ([decision record](../art/proposals/main-view-cards/README.md), [#136](https://github.com/joeloverbeck/cuba-libre-unfolded/pull/136)); spec #137 and ticket #144 record it.

## Specified in a ticket, without a separate owner ruling

These three were specified in [#61](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/61), part of #58, and merged in PR #76. #61's labelling criterion names the first two; the withdrawal is specified in its body. The app labels all three as local policy. No owner ruling on them as rules interpretations is recorded.

### Going back from Terror

**Case:** 26July chooses Terror and wants to reconsider before anything has happened.

**Policy:** 26July may go back from Terror to the action choice until the first real effect. A real effect is an authorized nonzero transfer, a resolved Terror, a Kidnap roll or a removal. A declined or zero request, a Wait, and a cancelled removal or Kidnap are not effects.

### Transfers open once Terror is chosen

**Rules:** R 1.5.1 permits Resource transfers only while a faction is executing.

**Policy:** 26July is executing from the moment it chooses Terror until End Terror, so transfers are offered only then. There are none at the action choice and none after Pass.

### A pending Transfer request is withdrawn

**Case:** 26July stops executing while a Transfer request it made is still unanswered.

**Policy:** Going back or ending Terror withdraws a pending Transfer request. 26July may do either while the giver is still deciding.
