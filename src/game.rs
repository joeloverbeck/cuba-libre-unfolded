//! Cuba-owned rules from the accepted deacd635 tactical handoff (R 1.3–1.8, 2.3, 7).
use serde::{Deserialize, Serialize};
use unfolded_random::SeededRandom;

mod rally;
#[cfg(feature = "tabletop")]
pub(crate) use rally::rally_spaces;
pub use rally::{Rally, RallyAction, RallyResolution};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Government,
    M26,
    Directorio,
    Syndicate,
}
impl Faction {
    pub const ALL: [Self; 4] = [
        Self::Government,
        Self::M26,
        Self::Directorio,
        Self::Syndicate,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Government => "Government",
            Self::M26 => "26July",
            Self::Directorio => "Directorio",
            Self::Syndicate => "Syndicate",
        }
    }
}
/// The US Alliance track, best level first (R 6.3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsAlliance {
    Firm,
    Reluctant,
    Embargoed,
}
impl UsAlliance {
    pub const ALL: [Self; 3] = [Self::Firm, Self::Reluctant, Self::Embargoed];
    pub fn name(self) -> &'static str {
        match self {
            Self::Firm => "Firm",
            Self::Reluctant => "Reluctant",
            Self::Embargoed => "Embargoed",
        }
    }
    /// Resources each Government COIN Operation costs per space; Garrison pays it
    /// once in total (R 6.3.1).
    pub fn coin_cost(self) -> u8 {
        match self {
            Self::Firm => 2,
            Self::Reluctant => 3,
            Self::Embargoed => 4,
        }
    }
    /// Embargoed forbids Air Strike (R 4.2.2, R 6.3.1).
    pub fn allows_air_strike(self) -> bool {
        self != Self::Embargoed
    }
}
/// The game's Cash markers; any others are spares (R 4.5.2).
pub const CASH_MARKERS: u8 = 4;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Troops,
    Police,
    Guerrilla,
    Base,
    Casino,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Ordinary,
    Underground,
    Active,
    Open,
    Closed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub faction: Faction,
    pub kind: Kind,
    pub state: State,
    pub count: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    City,
    Forest,
    Grassland,
    Mountain,
    EconomicCenter,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub terrain: Terrain,
    pub population: u8,
    pub economy: u8,
    pub support: i8,
    pub pieces: Vec<Group>,
    pub terror: u8,
}
impl Space {
    pub fn control(&self) -> Option<Faction> {
        if self.terrain == Terrain::EconomicCenter {
            return None;
        }
        let mut counts = [0u16; 4];
        for p in &self.pieces {
            if p.state != State::Closed {
                counts[p.faction.index()] += p.count as u16;
            }
        }
        Faction::ALL
            .into_iter()
            .find(|f| counts[f.index()] * 2 > counts.iter().sum())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Standing {
    pub faction: Faction,
    pub score: i16,
    pub margin: i16,
    pub threshold: String,
    pub extra: String,
    pub contributions: Vec<String>,
}
/// 26July's decision as 1st Eligible on card 17 (R 2.3.2–2.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionChoice {
    /// The action choice is open: Pass and Terror are playable; the Event, Rally,
    /// March and Attack are listed but not in this task.
    Open,
    /// R 2.3.3: +1 Resource, 26July stays Eligible and Directorio becomes 1st Eligible.
    /// The task ends there.
    Pass,
    /// Terror is chosen: 26July executes it until End Terror. A #8 save began here, at
    /// revision 0, and has no `action` field; it resumes in this state.
    Terror,
}
/// The #8-save admission: a #8 record has no `action` field because #8 began with
/// Terror chosen, so it resumes with Terror chosen. This is only the serde default of
/// `Game::action`; a new task opens at `ActionChoice::Open`.
fn eight_save_entry() -> ActionChoice {
    ActionChoice::Terror
}
/// The Sequence of Play box a faction's cylinder occupies once its action is recorded
/// (R 2.3, record steps; R 2.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SequenceBox {
    Pass,
    OperationOnly,
    OperationWithSpecialActivity,
}
impl SequenceBox {
    /// The box's name, as a record states it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::OperationOnly => "Operation Only",
            Self::OperationWithSpecialActivity => "Operation with Special Activity",
        }
    }
    /// The rule that records an action in this box.
    pub fn rule(self) -> &'static str {
        match self {
            Self::Pass => "R 2.3.3",
            Self::OperationOnly | Self::OperationWithSpecialActivity => "R 2.3.4",
        }
    }
}
/// Card 17's Eligibility record and its R 2.3.7 adjustment for card 18.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Eligibility {
    /// Card 17's faction order, left to right (R 2.3.2).
    pub order: [Faction; 4],
    /// Eligibility on the track, indexed like `Faction::ALL`. After adjustment
    /// these cylinders govern the next card.
    pub eligible: [bool; 4],
    /// The 1st Eligible faction: Directorio after 26July Passes (R 2.3.3).
    pub first: Faction,
    pub second: Faction,
    /// The box 26July's cylinder occupies, once its action is recorded.
    pub m26_box: Option<SequenceBox>,
    /// Whether 26July is Eligible for the next card, once its action is recorded:
    /// a Pass leaves it Eligible (R 2.3.3); an Operation makes it Ineligible when
    /// cylinders are adjusted after the card (R 2.3.7).
    pub m26_next_card: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_card: Option<[CardEndEligibility; 4]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_restriction: Option<EligibilityRestriction>,
    #[serde(skip_serializing)]
    event_started: bool,
}
impl Eligibility {
    /// The card 17 continuation opens only after 26July records Terror with Kidnap.
    pub(crate) fn directorio_handoff(&self) -> bool {
        self.m26_box == Some(SequenceBox::OperationWithSpecialActivity) && !self.event_started
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CardEndEligibility {
    pub faction: Faction,
    pub eligible: bool,
    pub reason: EligibilityReason,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EligibilityReason {
    NoOperationOrEvent,
    ExecutedOperation,
    ExecutedEvent,
}
impl EligibilityReason {
    pub fn text(self) -> &'static str {
        match self {
            Self::NoOperationOrEvent => "took no Operation or Event",
            Self::ExecutedOperation => "executed Terror",
            Self::ExecutedEvent => "executed the Event",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct EligibilityRestriction {
    pub faction: Faction,
    pub through_card: u8,
}
/// A count of Resources in words: "1 Resource", "3 Resources".
pub fn resources(n: u8) -> String {
    format!("{n} Resource{}", if n == 1 { "" } else { "s" })
}
/// Card 17's unshaded text from the admitted card face (Playbook p. 33).
pub(crate) const CARD_17_UNSHADED: &str =
    "26July or DR free Rallies in each space it has a Base (as if spaces Neutral).";
/// Card 17's shaded text from the admitted card face (Playbook p. 33).
pub const CARD_17_SHADED: &str =
    "All 26July Guerrillas Active. 26July Ineligible through next card.";
/// What Terror does in each space it resolves after paying, worded once (R 3.3.4).
const TERROR_EFFECTS: &str = "flip 1 Underground Guerrilla there to Active, place a Terror marker and shift 1 level toward Active Opposition";
/// Why End Terror is refused while a rolled Kidnap still owes Santiago's Terror.
const SANTIAGO_OWED: &str = "Santiago Terror is owed: under the accepted commitment policy, its payment and Activation come before End Terror.";
/// The space where this task offers Kidnap (R 4.3.3): a City with Government
/// Resources at stake.
pub const KIDNAP_SPACE: &str = "santiago";
/// A faction's Resources never exceed 49 (R 1.8); excess is lost.
const RESOURCE_CAP: u8 = 49;
/// What 26July pays for Terror in each City or Province it selects (R 3.3.4).
pub const TERROR_COST: u8 = 1;
/// The prepared position's Resources, indexed like `Faction::ALL`.
const PREPARED_RESOURCES: [u8; 4] = [6, 0, 5, 15];
/// Card 17's order: 26July, Directorio, Government, Syndicate (rules baseline).
pub const CARD_17_ORDER: [Faction; 4] = [
    Faction::M26,
    Faction::Directorio,
    Faction::Government,
    Faction::Syndicate,
];
/// Card 17's Event is absent in older saves. A completed Event carries the applied
/// effect so its result and printed eligibility restriction survive restart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventProgress {
    ChooseRallyBeneficiary,
    Rally(Rally),
    Complete(EventOutcome),
}
impl EventProgress {
    pub fn outcome(&self) -> Option<&EventOutcome> {
        match self {
            Self::Complete(outcome) => Some(outcome),
            _ => None,
        }
    }
    #[cfg(feature = "tabletop")]
    pub(crate) fn rally_pending(&self) -> bool {
        matches!(self, Self::ChooseRallyBeneficiary | Self::Rally(_))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventOutcome {
    Shaded { flips: Vec<Flip> },
    Unshaded(Rally),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub(crate) revision: u64,
    pub(crate) resources: [u8; 4],
    pub(crate) spaces: Vec<Space>,
    pub(crate) available: Vec<Group>,
    pub(crate) finished: bool,
    pub(crate) random: u64,
    pub(crate) draw_deck: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) transfer: Option<Transfer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) kidnap: Option<Kidnap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) removal: Option<Removal>,
    /// A #8 save lacks this field and resumes with Terror chosen.
    #[serde(default = "eight_save_entry")]
    pub(crate) action: ActionChoice,
    /// Set by the first real effect: an authorized nonzero transfer, a resolved
    /// Terror, a Kidnap roll or a removal. Going back to the action choice is local
    /// policy (#58) allowed only before it. A #8 save lacks it; `effected` also reads
    /// such a save's state.
    #[serde(default)]
    pub(crate) effect: bool,
    /// Kidnap opened and not yet rolled or cancelled (#63). Opening consumes no
    /// randomness; only the roll does.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) kidnap_open: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) event: Option<EventProgress>,
}
/// Removal mode (#73): open from "Remove own pieces" until Done. Each decision in it
/// returns a chosen quantity of one 26July group to Available (R 1.4.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Removal {
    Selecting,
    /// Only in #8 saves, which chose a group and then confirmed one piece. Such a
    /// save resumes in removal mode with the same choices as `Selecting`; new
    /// decisions never write it.
    Confirming {
        space: String,
        kind: Kind,
        state: State,
    },
}
/// A removal choice's group, as its id names it: `remove:<space>:<group>:<quantity>`.
pub fn removal_group(kind: Kind, state: State) -> &'static str {
    match (kind, state) {
        (Kind::Base, _) => "base",
        (_, State::Active) => "active",
        _ => "underground",
    }
}
/// The kind and state a removal group names, for 26July's Guerrillas and Base.
pub fn removal_piece(group: &str) -> Option<(Kind, State)> {
    match group {
        "base" => Some((Kind::Base, State::Ordinary)),
        "active" => Some((Kind::Guerrilla, State::Active)),
        "underground" => Some((Kind::Guerrilla, State::Underground)),
        _ => None,
    }
}
/// The committed die and its actual proceeds. Balances at the roll remain
/// inspectable even when subsequent Terror or Resource transfers change current balances.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kidnap {
    pub die: u8,
    pub proceeds: u8,
    pub government_before: u8,
    pub m26_before: u8,
    pub random_before: u64,
    /// The match revision this roll created (#74). The die presentation chooses its
    /// tumble from it, independently of the die. A #8 roll lacks it and reads 0.
    #[serde(default)]
    pub revision: u64,
}
impl Kidnap {
    /// Government's Resources after it paid the proceeds.
    pub fn government_after(&self) -> u8 {
        self.government_before - self.proceeds
    }
    /// 26July's Resources after it received the proceeds, which stop at 49 (R 1.8).
    pub fn m26_after(&self) -> u8 {
        self.m26_before
            .saturating_add(self.proceeds)
            .min(RESOURCE_CAP)
    }
}
/// A Resource transfer in progress (R 1.5.1). `Selecting` is the transfer modal open
/// for 26July's choice of "Give to" or "Ask from"; `Authorizing` is a Transfer request
/// awaiting its giver, whose `amount` is the amount asked, which prefills the giver's
/// decision. A request whose giver is 26July is the reserved state: a request another
/// faction started, which nothing in this task creates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Transfer {
    Selecting,
    Authorizing {
        giver: Faction,
        recipient: Faction,
        amount: u8,
        /// Wait set the giver's decision aside (#75): the request stays pending, its
        /// tab in the top band reopens the decision, and a restart shows it so. Saves
        /// written before #75 lack it and read false, the giver deciding, as they
        /// resumed before.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        waited: bool,
    },
}
impl Transfer {
    /// The same transfer, as if Wait had never set its decision aside.
    pub fn without_wait(&self) -> Self {
        match self {
            Transfer::Authorizing {
                giver,
                recipient,
                amount,
                ..
            } => Transfer::Authorizing {
                giver: *giver,
                recipient: *recipient,
                amount: *amount,
                waited: false,
            },
            Transfer::Selecting => Transfer::Selecting,
        }
    }
}
/// Which limit sets a transfer's maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Bound {
    /// The giver's balance.
    Balance,
    /// The recipient's 49 cap (R 1.8).
    Cap,
    /// The Resource 26July keeps for an owed Santiago Terror (accepted commitment
    /// policy, local).
    Reserve,
}
/// The terms of one transfer direction now: what the giver holds, what the recipient
/// can still receive, what is held back and the largest amount that may move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TransferTerms {
    pub giver: Faction,
    pub recipient: Faction,
    /// The giver's Resources.
    pub balance: u8,
    /// The recipient's Resources.
    pub held: u8,
    /// What the recipient can receive before it holds 49 (R 1.8).
    pub room: u8,
    /// Kept by 26July for an owed Santiago Terror when it gives: 0 or 1.
    pub reserved: u8,
    /// The largest amount that may move: the least of the three limits.
    pub limit: u8,
    /// The limit that binds.
    pub bound: Bound,
}
impl TransferTerms {
    /// Why `amount` cannot move, or `None` when it can. The rules authority refuses
    /// with the same words.
    pub fn refusal(&self, amount: u8) -> Option<String> {
        if amount <= self.limit {
            return None;
        }
        let why = match self.bound {
            Bound::Balance => format!("{} has {}", self.giver.name(), resources(self.balance)),
            Bound::Cap => format!(
                "{} holds {} of at most {RESOURCE_CAP} Resources",
                self.recipient.name(),
                self.held
            ),
            Bound::Reserve => format!(
                "the owed Santiago Terror needs {} of 26July's {}",
                self.reserved,
                resources(self.balance)
            ),
        };
        Some(format!("{amount} is more than {}: {why}.", self.limit))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub revision: u64,
    pub owner: Faction,
    /// 26July's action choice: open, Pass or Terror.
    pub action: ActionChoice,
    /// The faction executing the current Operation or Event. During the free Rally it
    /// is the faction directing the Operation granted by Directorio's Event (R 5.1).
    pub executing: Option<Faction>,
    pub eligibility: Eligibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventProgress>,
    pub resources: [u8; 4],
    pub spaces: Vec<Space>,
    pub available: Vec<Group>,
    pub map_piece_count: u16,
    pub available_piece_count: u16,
    pub standings: Vec<Standing>,
    pub current_card: u8,
    pub preview_card: u8,
    pub played_cards: Vec<u8>,
    pub draw_count: usize,
    pub eligible: [bool; 4],
    pub passed: [bool; 4],
    pub aid: u8,
    pub alliance: UsAlliance,
    pub cash_available: u8,
    pub terror_available: u8,
    pub capabilities: Vec<String>,
    pub momentum: Vec<String>,
    pub finished: bool,
    pub choices: Vec<Choice>,
    /// The next faction to act, its role and its options, once 26July's action is
    /// recorded (R 2.3.3–2.3.4).
    pub next: Option<Next>,
    pub operation: &'static str,
    pub special_activity: Option<String>,
    pub owed_terror: Option<String>,
    pub transfer: Option<Transfer>,
    /// The terms of each transfer 26July may make now: all six directions while the
    /// transfer modal is open, the pending request's while it awaits its giver, and
    /// none otherwise.
    pub transfer_terms: Vec<TransferTerms>,
    pub kidnap: Option<Kidnap>,
    /// Kidnap is opened: its stakes are shown and it awaits the roll or a cancel.
    pub kidnap_open: bool,
    pub removal: Option<Removal>,
    /// Whether the first effect has happened: an authorized transfer, a resolved
    /// Terror, a Kidnap roll or a removal. Until then 26July may go back to its action
    /// choice (local policy, #58), so a step that waits says whether it still can.
    pub effected: bool,
    /// What the step that recorded 26July's action itself changed in Resources, as
    /// the authority applied it: a Pass's +1, which stops at 49 (R 2.3.3, 1.8). Empty
    /// until the action is recorded, and after End Terror, which moves none itself.
    pub recorded_resources: Vec<ResourceChange>,
    /// This tactical task starts without Terror markers. Each space with a marker
    /// contributes one resolution at the fixed per-space cost [`TERROR_COST`] (R 3.3.4).
    pub resolved: Vec<Resolved>,
    /// What a Kidnap roll could take now (R 4.3.3), from current Resources, until
    /// Kidnap is rolled or the action is recorded.
    pub kidnap_stakes: Option<KidnapStakes>,
}
/// A space where 26July's Terror resolved, and the Resources it paid there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub space: String,
    pub paid: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub supported: bool,
    pub explanation: String,
    /// The faction that may submit it. Usually the observation's owner; while a
    /// Transfer request awaits its giver, 26July may still withdraw it.
    pub owner: Faction,
    /// For a Special Activity listed at the action choice, the id of the Operation
    /// it accompanies (R 4.1).
    pub accompanies: Option<String>,
    /// For an unsupported choice, the actions possible in this task that would make
    /// it supported, such as "Kidnap here first"; empty when nothing here can.
    pub remedies: Vec<String>,
    /// The consequences of an irreversible step, computed by applying it to a copy of
    /// the match: Pass, each Terror target, End Terror and each Voluntary removal.
    pub preview: Option<Preview>,
}
impl Choice {
    /// The action without its match revision: `terror:santiago` for the id
    /// `3:terror:santiago`.
    pub fn action(&self) -> &str {
        self.id.split_once(':').map_or(self.id.as_str(), |(_, a)| a)
    }
    /// The map space this choice acts on, so the map can offer it there: a Terror
    /// target, or opening Kidnap in its space. Other choices act on no space.
    pub fn space(&self) -> Option<&str> {
        // A Special Activity listed under its Operation at the action choice is
        // not an action on the map.
        if self.accompanies.is_some() {
            return None;
        }
        let action = self.action();
        match action {
            "kidnap" => Some(KIDNAP_SPACE),
            _ => action.strip_prefix("terror:"),
        }
    }
    /// For a removal in removal mode: its space, group and quantity.
    pub fn removal(&self) -> Option<(&str, &str, u8)> {
        let action = self.id.split_once(':')?.1;
        let mut parts = action.strip_prefix("remove:")?.split(':');
        let (space, group, n) = (parts.next()?, parts.next()?, parts.next()?);
        removal_piece(group)?;
        let n = n.parse().ok()?;
        parts.next().is_none().then_some((space, group, n))
    }
    /// For a transfer decision: its direction, the other faction and the amount.
    pub fn transfer(&self) -> Option<TransferChoice> {
        let action = self.action();
        let (verb, rest) = action.split_once(':')?;
        let other_and_amount = || {
            let (other, n) = rest.split_once(':')?;
            let other = *Faction::ALL.get(other.parse::<usize>().ok()?)?;
            Some((other, n.parse().ok()?))
        };
        match verb {
            "give" => other_and_amount()
                .map(|(recipient, amount)| TransferChoice::Give { recipient, amount }),
            "ask" => {
                other_and_amount().map(|(giver, amount)| TransferChoice::Ask { giver, amount })
            }
            "authorize" => rest
                .parse()
                .ok()
                .map(|amount| TransferChoice::Authorize { amount }),
            _ => None,
        }
    }
}
/// A transfer decision, as its choice id names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransferChoice {
    /// `give:<recipient>:<amount>`: 26July's own Resources move at once.
    Give { recipient: Faction, amount: u8 },
    /// `ask:<giver>:<amount>`: a Transfer request; nothing moves yet.
    Ask { giver: Faction, amount: u8 },
    /// `authorize:<amount>`: the giver's decision on the pending request.
    Authorize { amount: u8 },
}
impl TransferChoice {
    /// The amount the decision names.
    pub fn amount(self) -> u8 {
        match self {
            Self::Give { amount, .. } | Self::Ask { amount, .. } | Self::Authorize { amount } => {
                amount
            }
        }
    }
    /// The same direction and faction with no amount: what a transfer's amounts share.
    pub fn direction(self) -> Self {
        match self {
            Self::Give { recipient, .. } => Self::Give {
                recipient,
                amount: 0,
            },
            Self::Ask { giver, .. } => Self::Ask { giver, amount: 0 },
            Self::Authorize { .. } => Self::Authorize { amount: 0 },
        }
    }
}
/// What one step would change. The rules authority computes it by applying the step
/// to a copy of the match; reading it changes neither the match nor its randomness.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Preview {
    pub resources: Vec<ResourceChange>,
    /// Pieces that change state in place, such as an Underground Guerrilla flipped to
    /// Active (Activate, R 1.4.3).
    pub flips: Vec<Flip>,
    /// Own pieces returned to Available (Voluntary removal, R 1.4.1).
    pub removed: Vec<Returned>,
    /// Spaces that receive a Terror marker.
    pub terror: Vec<String>,
    pub support: Vec<SupportShift>,
    pub control: Vec<ControlChange>,
    pub standings: Vec<StandingChange>,
    /// Choices offered now that the step would leave unavailable, with the reason
    /// they would then give.
    pub unavailable: Vec<Unavailable>,
    /// The box 26July's action would record, when the step ends it.
    pub records: Option<SequenceBox>,
    /// Whether 26July would be Eligible for the next card, when the step ends its
    /// action (R 2.3.3, 2.3.7).
    pub m26_next_card: Option<bool>,
    /// Directorio's options, when the step ends 26July's action.
    pub next: Option<Next>,
    /// What a Kidnap roll could take, for opening Kidnap and for its roll. A roll's
    /// result is never previewed.
    pub kidnap: Option<KidnapStakes>,
    /// Facts with rule references, such as the zero-space warning.
    pub notes: Vec<String>,
}
/// What a Kidnap roll could take (R 4.3.3), from current Resources only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KidnapStakes {
    /// The Kidnap space, Santiago in this task.
    pub space: String,
    /// The targeted faction: Government, in a City or EC.
    pub target: Faction,
    /// The target's current Resources.
    pub government: u8,
    /// Every possible proceeds amount, ascending: `min(die, government)` for die 1–6.
    pub proceeds: Vec<u8>,
    /// The guaranteed minimum, a roll of 1.
    pub minimum: u8,
    /// Whether rolling now owes Santiago Terror (the accepted commitment policy).
    pub commits_santiago: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResourceChange {
    pub faction: Faction,
    pub from: u8,
    pub to: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flip {
    pub space: String,
    pub faction: Faction,
    pub kind: Kind,
    pub from: State,
    pub to: State,
    pub count: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Returned {
    pub space: String,
    pub faction: Faction,
    pub kind: Kind,
    pub state: State,
    pub count: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SupportShift {
    pub space: String,
    pub from: i8,
    pub to: i8,
    /// For example "Neutral → Passive Opposition".
    pub label: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ControlChange {
    pub space: String,
    pub from: Option<Faction>,
    pub to: Option<Faction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StandingChange {
    pub faction: Faction,
    pub score_from: i16,
    pub score_to: i16,
    pub margin_from: i16,
    pub margin_to: i16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Unavailable {
    /// The choice id without its revision prefix, such as "kidnap".
    pub choice: String,
    pub label: String,
    pub reason: String,
}
/// The next faction's role and options once 26July's action is recorded (R 2.3.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Next {
    pub faction: Faction,
    /// "1st Eligible" after a Pass, otherwise "2nd Eligible".
    pub role: String,
    pub options: Vec<String>,
}
/// A support level's name (R 1.6).
pub fn support_level(level: i8) -> &'static str {
    match level {
        2 => "Active Support",
        1 => "Passive Support",
        0 => "Neutral",
        -1 => "Passive Opposition",
        _ => "Active Opposition",
    }
}
fn group(faction: Faction, kind: Kind, count: u8) -> Group {
    Group {
        faction,
        kind,
        count,
        state: match kind {
            Kind::Guerrilla => State::Underground,
            Kind::Casino => State::Open,
            _ => State::Ordinary,
        },
    }
}
impl Game {
    /// #8's entry: the accepted prepared position with Terror already chosen, at
    /// revision 0, as every #8 save began. New tasks start at `opening` instead.
    pub fn prepared() -> Self {
        use Faction::*;
        use Kind::*;
        use Terrain::*;
        let rows = [
            (
                "havana",
                "Havana",
                City,
                6,
                0,
                2,
                vec![
                    group(Government, Troops, 6),
                    group(Government, Police, 4),
                    group(Directorio, Guerrilla, 2),
                    group(Syndicate, Casino, 1),
                ],
            ),
            (
                "camaguey-city",
                "Camagüey City",
                City,
                1,
                0,
                1,
                vec![group(Government, Troops, 1), group(Government, Police, 2)],
            ),
            (
                "santiago",
                "Santiago de Cuba",
                City,
                1,
                0,
                0,
                vec![
                    group(Government, Troops, 2),
                    group(Government, Police, 1),
                    group(M26, Guerrilla, 2),
                ],
            ),
            (
                "pinar-del-rio",
                "Pinar del Río",
                Forest,
                1,
                0,
                2,
                vec![group(Syndicate, Casino, 1)],
            ),
            (
                "la-habana",
                "La Habana",
                Grassland,
                1,
                0,
                1,
                vec![group(Syndicate, Casino, 1)],
            ),
            ("matanzas", "Matanzas", Grassland, 1, 0, -1, vec![]),
            (
                "las-villas",
                "Las Villas",
                Mountain,
                2,
                0,
                0,
                vec![group(Government, Troops, 3)],
            ),
            (
                "camaguey-province",
                "Camagüey Province",
                Forest,
                1,
                0,
                -1,
                vec![group(Directorio, Guerrilla, 1)],
            ),
            (
                "oriente",
                "Oriente",
                Forest,
                2,
                0,
                -1,
                vec![group(M26, Guerrilla, 1)],
            ),
            (
                "sierra-maestra",
                "Sierra Maestra",
                Mountain,
                1,
                0,
                -2,
                vec![group(M26, Base, 1)],
            ),
            (
                "ec-pinar-la-habana",
                "Pinar del Río–La Habana EC",
                EconomicCenter,
                0,
                3,
                0,
                vec![],
            ),
            (
                "ec-las-villas-camaguey",
                "Las Villas–Camagüey EC",
                EconomicCenter,
                0,
                3,
                0,
                vec![],
            ),
            (
                "ec-oriente-sierra",
                "Oriente–Sierra Maestra EC",
                EconomicCenter,
                0,
                2,
                0,
                vec![],
            ),
        ];
        let spaces = rows
            .into_iter()
            .map(
                |(id, name, terrain, population, economy, support, pieces)| Space {
                    id: id.into(),
                    name: name.into(),
                    terrain,
                    population,
                    economy,
                    support,
                    pieces,
                    terror: 0,
                },
            )
            .collect();
        let mut draw_deck: Vec<u8> = (18..=24).collect();
        draw_deck.push(50);
        draw_deck.extend(25..=36);
        draw_deck.push(51);
        draw_deck.extend(37..=48);
        draw_deck.push(52);
        Self {
            revision: 0,
            resources: PREPARED_RESOURCES,
            spaces,
            finished: false,
            random: 17,
            draw_deck,
            transfer: None,
            kidnap: None,
            removal: None,
            action: ActionChoice::Terror,
            effect: false,
            kidnap_open: false,
            event: None,
            available: vec![
                group(Government, Troops, 3),
                group(Government, Police, 8),
                group(Government, Base, 2),
                group(M26, Guerrilla, 12),
                group(M26, Base, 3),
                group(Directorio, Guerrilla, 12),
                group(Directorio, Base, 4),
                group(Syndicate, Guerrilla, 6),
                group(Syndicate, Casino, 7),
            ],
        }
    }
    pub fn observe(&self) -> Observation {
        let mut played_cards: Vec<u8> = (1..=12).collect();
        played_cards.push(49);
        played_cards.extend(13..=16);
        let eligibility = self.eligibility();
        Observation {
            revision: self.revision,
            owner: self.owner(),
            action: self.action,
            executing: self.executing(),
            eligible: eligibility.eligible,
            eligibility,
            event: self.event.clone(),
            resources: self.resources,
            spaces: self.spaces.clone(),
            available: self.available.clone(),
            map_piece_count: self
                .spaces
                .iter()
                .flat_map(|s| &s.pieces)
                .map(|p| p.count as u16)
                .sum(),
            available_piece_count: self.available.iter().map(|p| p.count as u16).sum(),
            standings: self.standings(),
            current_card: 17,
            preview_card: 18,
            played_cards,
            draw_count: self.draw_deck.len(),
            passed: Faction::ALL.map(|f| f == Faction::M26 && self.action == ActionChoice::Pass),
            aid: 15,
            alliance: UsAlliance::Firm,
            cash_available: CASH_MARKERS,
            terror_available: 20 - self.spaces.iter().map(|s| s.terror).sum::<u8>(),
            capabilities: vec![],
            momentum: vec![],
            finished: self.finished,
            choices: self
                .choices()
                .into_iter()
                .map(|mut choice| {
                    choice.preview = self.preview(&choice);
                    choice
                })
                .collect(),
            next: self.next(),
            operation: match self.action {
                _ if matches!(self.event, Some(EventProgress::ChooseRallyBeneficiary)) => {
                    "Directorio chooses the Rally beneficiary"
                }
                _ if matches!(self.event, Some(EventProgress::Rally(_))) => "executing free Rally",
                _ if self
                    .event
                    .as_ref()
                    .and_then(EventProgress::outcome)
                    .is_some() =>
                {
                    "Card 17 ended: Eligibility adjusted"
                }
                ActionChoice::Open => "1st Eligible: choose an action",
                ActionChoice::Pass => "Passed: +1 Resource, stays Eligible",
                ActionChoice::Terror if self.finished && self.kidnap.is_some() => {
                    "Terror ended: Operation with Special Activity recorded"
                }
                ActionChoice::Terror if self.finished => "Terror ended: Operation Only recorded",
                ActionChoice::Terror => "executing Terror",
            },
            special_activity: self.kidnap.as_ref().map(|_| "Kidnap".into()),
            owed_terror: self.owes_santiago().then(|| "santiago".into()),
            transfer: self.transfer.clone(),
            transfer_terms: self.transfer_terms(),
            kidnap: self.kidnap.clone(),
            kidnap_open: self.kidnap_open,
            removal: self.removal.clone(),
            effected: self.effected(),
            recorded_resources: self.recorded_resources(),
            resolved: self
                .spaces
                .iter()
                .filter(|s| s.terror > 0)
                .map(|s| Resolved {
                    space: s.id.clone(),
                    paid: TERROR_COST,
                })
                .collect(),
            kidnap_stakes: (self.kidnap.is_none() && !self.finished).then(|| self.stakes()),
        }
    }
    /// What Pass changed in 26July's Resources. A Pass is taken only at the action
    /// choice before any effect, so from the prepared Resources; a loaded save is held
    /// to exactly that (`validate`).
    fn recorded_resources(&self) -> Vec<ResourceChange> {
        if self.action != ActionChoice::Pass {
            return vec![];
        }
        let m26 = Faction::M26.index();
        vec![ResourceChange {
            faction: Faction::M26,
            from: PREPARED_RESOURCES[m26],
            to: self.resources[m26],
        }]
    }
    /// The next faction to act once 26July's action is recorded: the first Eligible
    /// faction other than 26July, in its role, with its options (R 2.3.3–2.3.4).
    fn next(&self) -> Option<Next> {
        if self.event.is_some() {
            return None;
        }
        let e = self.eligibility();
        e.m26_box?;
        let (faction, role) = if e.first != Faction::M26 {
            (e.first, "1st Eligible")
        } else {
            (e.second, "2nd Eligible")
        };
        Some(Next {
            faction,
            role: role.into(),
            options: self.next_options(),
        })
    }
    /// The next faction's options once 26July's action is recorded (R 2.3.3–2.3.4).
    fn next_options(&self) -> Vec<String> {
        if self.action == ActionChoice::Pass {
            // Directorio, now 1st Eligible, has every 1st Eligible option.
            vec![
                "Pass (+1 if executed later)".into(),
                "Operation (with or without a Special Activity)".into(),
                "Event".into(),
            ]
        } else if self.finished {
            let mut next = vec![
                "Pass (+1 if executed later)".into(),
                "Limited Operation".into(),
            ];
            if self.kidnap.is_some() {
                next.push("Event".into());
            }
            next
        } else {
            vec![]
        }
    }
    /// Submits `action` as the faction the game offers it to.
    fn apply(&mut self, action: &str) -> Result<(), String> {
        let id = format!("{}:{action}", self.revision);
        let owner = self
            .choices()
            .into_iter()
            .find(|c| c.id == id)
            .ok_or("Stale or unknown choice identity.")?
            .owner;
        self.submit(owner, self.revision, &id)
    }
    /// The consequences of an irreversible choice, from a copy of the match. Kidnap's
    /// roll is never applied to a copy, so no future die result can show: opening it
    /// and its roll preview the stakes from current Resources instead.
    fn preview(&self, choice: &Choice) -> Option<Preview> {
        if !choice.supported {
            return None;
        }
        let action = choice.id.split_once(':')?.1;
        let steps = match action {
            "kidnap" | "kidnap:roll" => return Some(self.kidnap_stakes(action)),
            "pass" | "end-terror" | "event:shaded" => vec![action],
            _ if action.starts_with("terror:")
                || matches!(self.event, Some(EventProgress::Rally(_)))
                || choice.removal().is_some() =>
            {
                vec![action]
            }
            _ => return None,
        };
        let mut after = self.clone();
        for step in &steps {
            after.apply(step).ok()?;
        }
        Some(self.consequences(&after, action))
    }
    fn consequences(&self, after: &Game, action: &str) -> Preview {
        let mut preview = Preview::default();
        for f in Faction::ALL {
            let (from, to) = (self.resources[f.index()], after.resources[f.index()]);
            if from != to {
                preview.resources.push(ResourceChange {
                    faction: f,
                    from,
                    to,
                });
            }
        }
        for (before, now) in self.spaces.iter().zip(&after.spaces) {
            let id = || before.id.clone();
            for f in Faction::ALL {
                for kind in [
                    Kind::Troops,
                    Kind::Police,
                    Kind::Guerrilla,
                    Kind::Base,
                    Kind::Casino,
                ] {
                    let count = |space: &Space, state: Option<State>| -> u8 {
                        space
                            .pieces
                            .iter()
                            .filter(|p| {
                                p.faction == f
                                    && p.kind == kind
                                    && state.is_none_or(|s| s == p.state)
                            })
                            .map(|p| p.count)
                            .sum()
                    };
                    let total = (count(before, None), count(now, None));
                    let states = [
                        State::Ordinary,
                        State::Underground,
                        State::Active,
                        State::Open,
                        State::Closed,
                    ];
                    if total.1 < total.0 {
                        for state in states {
                            let (was, is) = (count(before, Some(state)), count(now, Some(state)));
                            if is < was {
                                preview.removed.push(Returned {
                                    space: id(),
                                    faction: f,
                                    kind,
                                    state,
                                    count: was - is,
                                });
                            }
                        }
                    } else if total.0 == total.1 {
                        let lost = states
                            .into_iter()
                            .find(|&s| count(now, Some(s)) < count(before, Some(s)));
                        let gained = states
                            .into_iter()
                            .find(|&s| count(now, Some(s)) > count(before, Some(s)));
                        if let (Some(from), Some(to)) = (lost, gained) {
                            preview.flips.push(Flip {
                                space: id(),
                                faction: f,
                                kind,
                                from,
                                to,
                                count: count(before, Some(from)) - count(now, Some(from)),
                            });
                        }
                    }
                }
            }
            if now.terror > before.terror {
                preview.terror.push(id());
            }
            if now.support != before.support {
                preview.support.push(SupportShift {
                    space: id(),
                    from: before.support,
                    to: now.support,
                    label: format!(
                        "{} → {}",
                        support_level(before.support),
                        support_level(now.support)
                    ),
                });
            }
            if now.control() != before.control() {
                preview.control.push(ControlChange {
                    space: id(),
                    from: before.control(),
                    to: now.control(),
                });
            }
        }
        for (was, is) in self.standings().into_iter().zip(after.standings()) {
            if (was.score, was.margin) != (is.score, is.margin) {
                preview.standings.push(StandingChange {
                    faction: was.faction,
                    score_from: was.score,
                    score_to: is.score,
                    margin_from: was.margin,
                    margin_to: is.margin,
                });
            }
        }
        if after.finished {
            if !action.starts_with("rally:") {
                let e = after.eligibility();
                preview.records = e.m26_box;
                preview.m26_next_card = e.m26_next_card;
                preview.next = after.next();
            }
        } else {
            // Compare the Operation menus, so removal mode's own menu does not count:
            // a removal leaves the mode open, but what it changes is what the
            // Operation offers after Done.
            let mut menu = self.clone();
            menu.removal = None;
            let mut later = after.clone();
            later.removal = None;
            let later = later.choices();
            let suffix = |c: &Choice| {
                c.id.split_once(':')
                    .map_or(String::new(), |(_, a)| a.to_owned())
            };
            for choice in menu.choices().iter().filter(|c| c.supported) {
                let name = suffix(choice);
                if name == action {
                    continue;
                }
                let now = later.iter().find(|c| suffix(c) == name);
                if now.is_none_or(|c| !c.supported) {
                    preview.unavailable.push(Unavailable {
                        choice: name,
                        label: choice.label.clone(),
                        reason: now
                            .map_or_else(|| "No longer offered.".into(), |c| c.explanation.clone()),
                    });
                }
            }
        }
        preview.notes = self.notes(after, action);
        preview
    }
    /// The facts a preview states besides its computed changes, with rule references.
    fn notes(&self, after: &Game, action: &str) -> Vec<String> {
        let mut notes = vec![];
        if action == "pass" {
            notes.push("26July stays Eligible (R 2.3.3). The task ends there; Directorio's options are shown, not executed.".into());
        } else if action == "end-terror" {
            let resolved = self.spaces.iter().any(|s| s.terror > 0);
            if self.kidnap.is_some() {
                notes.push(format!("After a Special Activity, Directorio may execute card 17's Event, either text (R 2.3.4, 5.2). Shaded: \"{CARD_17_SHADED}\""));
            } else {
                notes.push("No Event for Directorio: it follows only an Operation with Special Activity (R 2.3.4).".into());
            }
            if resolved {
                notes.push("26July becomes Ineligible after this card (R 2.3.7).".into());
            } else {
                notes.push("No Terror resolved: the Operation has no effect, gives no Pass income, and 26July becomes Ineligible after this card (R 2.3.7). Zero-space Operation Only is the accepted local reading of R 3.1, 3.3.4 and 4.1, not a publisher ruling.".into());
            }
            if matches!(self.transfer, Some(Transfer::Authorizing { .. })) {
                notes.push(
                    "The pending Transfer request is withdrawn; nothing moves (local policy, #58)."
                        .into(),
                );
            }
            if self.may_go_back() {
                notes.push("Nothing has happened yet: Back to the action choice is still offered instead (local policy, #58).".into());
            }
        } else if action == "event:shaded" {
            notes.push(CARD_17_SHADED.into());
            notes.push("The Event costs 0 Resources. After eligibility is adjusted (R 2.3.7), card 18 stays in preview.".into());
        } else if action.starts_with("rally:") {
            notes.push("The free Rally adds no eligibility penalty (R 5.5).".into());
        } else if action.starts_with("terror:") {
            notes.push(format!("R 3.3.4: pay 1 Resource, {TERROR_EFFECTS}."));
        } else if action.ends_with(":1") {
            notes.push("R 1.4.1: the piece returns to Available. Earlier Terror markers, support shifts and Kidnap results stay.".into());
        } else {
            notes.push("R 1.4.1: the pieces return to Available. Earlier Terror markers, support shifts and Kidnap results stay.".into());
        }
        if !self.effected() && after.effected() && !after.finished {
            notes.push("This is the first effect: going back to the action choice ends (local policy, #58).".into());
        }
        notes
    }
    /// What a Kidnap roll in Santiago could take from current Resources (R 4.3.3).
    fn stakes(&self) -> KidnapStakes {
        let government = self.resources[Faction::Government.index()];
        let mut proceeds: Vec<u8> = (1..=6).map(|die| die.min(government)).collect();
        proceeds.dedup();
        KidnapStakes {
            space: KIDNAP_SPACE.into(),
            target: Faction::Government,
            government,
            minimum: proceeds[0],
            proceeds,
            commits_santiago: self
                .spaces
                .iter()
                .any(|s| s.id == KIDNAP_SPACE && s.terror == 0),
        }
    }
    /// Opening Kidnap and its roll: the target, the possible proceeds and their
    /// guaranteed minimum from current Resources, the owed Santiago Terror and card 17's
    /// Event for Directorio. Only facts certain before the roll are stated.
    fn kidnap_stakes(&self, action: &str) -> Preview {
        let stakes = self.stakes();
        let (government, commits_santiago) = (stakes.government, stakes.commits_santiago);
        let mut preview = Preview {
            kidnap: Some(stakes),
            ..Preview::default()
        };
        let mut notes = vec![format!(
            "R 4.3.3: roll a die and take that many Resources from Government, which has {government}; once it runs out, no more is taken."
        )];
        if action == "kidnap" {
            notes.push(
                "Opening rolls nothing: Cancel leaves the match and its randomness unchanged."
                    .into(),
            );
        }
        if commits_santiago {
            notes.push("Rolling before Santiago's Terror owes it, under the accepted commitment policy (local, not a publisher ruling): 1 Resource and Santiago's last Underground Guerrilla stay reserved, and End Terror is refused until it resolves.".into());
            preview.unavailable.push(Unavailable {
                choice: "end-terror".into(),
                label: "End Terror".into(),
                reason: SANTIAGO_OWED.into(),
            });
        }
        notes.push(format!("End Terror then records Operation with Special Activity: Directorio may execute card 17's Event, either text (R 2.3.4, 5.2). Shaded: \"{CARD_17_SHADED}\""));
        if !self.effected() {
            notes.push("The roll is the first effect: going back to the action choice ends (local policy, #58).".into());
            preview.unavailable.push(Unavailable {
                choice: "back-to-choice".into(),
                label: "Back to the action choice".into(),
                reason: "No longer offered.".into(),
            });
        }
        preview.notes = notes;
        preview
    }
    fn executing(&self) -> Option<Faction> {
        match &self.event {
            Some(EventProgress::ChooseRallyBeneficiary) => Some(Faction::Directorio),
            Some(EventProgress::Rally(rally)) => Some(rally.beneficiary),
            _ => (self.action == ActionChoice::Terror && !self.finished).then_some(Faction::M26),
        }
    }
    pub(crate) fn eligibility(&self) -> Eligibility {
        let passed = self.action == ActionChoice::Pass;
        let outcome = self.event.as_ref().and_then(EventProgress::outcome);
        let next_card = outcome.map(|_| {
            Faction::ALL.map(|faction| {
                let reason = match faction {
                    Faction::M26 => EligibilityReason::ExecutedOperation,
                    Faction::Directorio => EligibilityReason::ExecutedEvent,
                    _ => EligibilityReason::NoOperationOrEvent,
                };
                CardEndEligibility {
                    faction,
                    eligible: reason == EligibilityReason::NoOperationOrEvent,
                    reason,
                }
            })
        });
        let event_restriction = matches!(outcome, Some(EventOutcome::Shaded { .. })).then_some(
            EligibilityRestriction {
                faction: Faction::M26,
                through_card: 18,
            },
        );
        Eligibility {
            order: CARD_17_ORDER,
            eligible: next_card.map_or([true; 4], |results| results.map(|r| r.eligible)),
            next_card,
            event_restriction,
            event_started: self.event.is_some(),
            first: if passed {
                Faction::Directorio
            } else {
                Faction::M26
            },
            second: if passed {
                Faction::Government
            } else {
                Faction::Directorio
            },
            m26_box: match self.action {
                ActionChoice::Pass => Some(SequenceBox::Pass),
                ActionChoice::Terror if self.finished && self.kidnap.is_some() => {
                    Some(SequenceBox::OperationWithSpecialActivity)
                }
                ActionChoice::Terror if self.finished => Some(SequenceBox::OperationOnly),
                _ => None,
            },
            m26_next_card: match self.action {
                ActionChoice::Pass => Some(true),
                ActionChoice::Terror if self.finished => Some(false),
                _ => None,
            },
        }
    }
    /// Whether a real effect has happened. A #8 save has no `effect` flag, so its
    /// state is read too: Resources, Terror, a roll or a removed piece. (A #8 save
    /// whose transfers returned every Resource is indistinguishable from none.)
    fn effected(&self) -> bool {
        let prepared = Self::prepared();
        self.effect
            || self.kidnap.is_some()
            || self.spaces.iter().any(|s| s.terror > 0)
            || self.available != prepared.available
            || self.resources != prepared.resources
    }
    /// Local policy (#58): back to the action choice until the first effect, from the
    /// Operation menu or while a Transfer request awaits another faction.
    fn may_go_back(&self) -> bool {
        self.executing().is_some()
            && self.removal.is_none()
            && !self.kidnap_open
            && !self.effected()
            && match self.transfer {
                None => true,
                Some(Transfer::Authorizing { giver, .. }) => giver != Faction::M26,
                Some(Transfer::Selecting) => false,
            }
    }
    pub fn standings(&self) -> Vec<Standing> {
        Faction::ALL
            .into_iter()
            .map(|f| {
                let mut score = 0;
                let mut contributions = vec![];
                for s in &self.spaces {
                    let value = match f {
                        Faction::Government => s.population as i16 * s.support.max(0) as i16,
                        Faction::M26 => s.population as i16 * (-s.support).max(0) as i16,
                        Faction::Directorio => {
                            if s.control() == Some(f) {
                                s.population as i16
                            } else {
                                0
                            }
                        }
                        Faction::Syndicate => 0,
                    };
                    if value > 0 {
                        score += value;
                        contributions.push(format!(
                            "{}: {} × {} = {}",
                            s.name,
                            s.population,
                            if s.population > 0 {
                                value / s.population as i16
                            } else {
                                0
                            },
                            value
                        ));
                    }
                    for p in &s.pieces {
                        if p.faction == f
                            && ((matches!(f, Faction::M26 | Faction::Directorio)
                                && p.kind == Kind::Base)
                                || (f == Faction::Syndicate
                                    && p.kind == Kind::Casino
                                    && p.state == State::Open))
                        {
                            score += p.count as i16;
                            contributions.push(format!(
                                "{}: {} {:?} = {}",
                                s.name, p.count, p.kind, p.count
                            ));
                        }
                    }
                }
                let (threshold, margin, extra) = match f {
                    Faction::Government => {
                        let cities = self
                            .spaces
                            .iter()
                            .filter(|s| s.terrain == Terrain::City && s.support == 2)
                            .count();
                        (
                            "Support >18 and all Cities at Active Support".into(),
                            score - 18,
                            format!(
                                "Active Support cities {cities}/3 ({})",
                                if cities == 3 { "met" } else { "not met" }
                            ),
                        )
                    }
                    Faction::M26 => (
                        "Opposition + Bases >15".into(),
                        score - 15,
                        "Opposition plus Bases".into(),
                    ),
                    Faction::Directorio => (
                        "Controlled population + Bases >9".into(),
                        score - 9,
                        "Controlled population plus Bases".into(),
                    ),
                    Faction::Syndicate => {
                        let resources = self.resources[Faction::Syndicate.index()] as i16;
                        contributions.push(format!("Resources: {resources}"));
                        (
                            "Open Casinos >7 AND Resources >30".into(),
                            (score - 7).min(resources - 30),
                            format!("Resources {resources}/>30 (not met)"),
                        )
                    }
                };
                Standing {
                    faction: f,
                    score,
                    margin,
                    threshold,
                    extra,
                    contributions,
                }
            })
            .collect()
    }
    /// Who decides now: the giver of a pending authorization, the faction directing
    /// a free Rally, or the faction whose card action is open.
    fn owner(&self) -> Faction {
        if let Some(Transfer::Authorizing { giver, .. }) = self.transfer {
            giver
        } else if let Some(EventProgress::Rally(rally)) = &self.event {
            rally.beneficiary
        } else if self.finished {
            Faction::Directorio
        } else {
            Faction::M26
        }
    }
    fn offer(
        &self,
        id: &str,
        label: impl Into<String>,
        supported: bool,
        explanation: impl Into<String>,
    ) -> Choice {
        Choice {
            id: format!("{}:{id}", self.revision),
            label: label.into(),
            supported,
            explanation: explanation.into(),
            owner: self.owner(),
            accompanies: None,
            remedies: vec![],
            preview: None,
        }
    }
    /// 26July's complete option set as 1st Eligible on card 17 (R 2.3.4): Pass, the
    /// Event's two texts and its four Operations, each Special Activity under the
    /// Operations it may accompany (R 4.3.1–4.3.3).
    fn opening_choices(&self) -> Vec<Choice> {
        const ABSENT: &str =
            "Listed so the option set is complete; not in this task, so it cannot be chosen here.";
        let resources = self.resources[Faction::M26.index()];
        let absent = |id: &str, label: &str, rule: &str| {
            self.offer(
                id,
                format!("{label} · not in this task"),
                false,
                format!("{rule} {ABSENT}"),
            )
        };
        let under = |mut choice: Choice, operation: &str| {
            choice.accompanies = Some(format!("{}:{operation}", self.revision));
            choice
        };
        vec![
            self.offer(
                "pass",
                format!("Pass · +1 Resource ({resources} → {})", resources.saturating_add(1).min(RESOURCE_CAP)),
                true,
                "R 2.3.3: 26July gains 1 Resource and stays Eligible; Directorio becomes 1st Eligible. The task ends without executing Directorio.",
            ),
            absent(
                "event:unshaded",
                "Event (unshaded)",
                &format!("R 2.3.4, 5.2. Card 17, Vet trains guerrillas: {CARD_17_UNSHADED}"),
            ),
            absent(
                "event:shaded",
                "Event (shaded)",
                &format!("R 2.3.4, 5.2. Card 17, Mexico blocks training by Cuban expat: {CARD_17_SHADED}"),
            ),
            absent("rally", "Rally", "R 3.3.1: Rally places or flips 26July Guerrillas or a Base."),
            under(
                absent("rally:infiltrate", "Infiltrate · with Rally", "R 4.3.1: Infiltrate accompanies Rally or March."),
                "rally",
            ),
            absent("march", "March", "R 3.3.2: March moves 26July Guerrillas."),
            under(
                absent("march:infiltrate", "Infiltrate · with March", "R 4.3.1: Infiltrate accompanies Rally or March."),
                "march",
            ),
            absent("attack", "Attack", "R 3.3.3: Attack rolls to remove enemy pieces."),
            under(
                absent("attack:ambush", "Ambush · with Attack", "R 4.3.2: Ambush accompanies Attack."),
                "attack",
            ),
            self.offer(
                "terror",
                "Terror · Operation",
                true,
                format!("R 3.3.4: in each selected space pay 1 Resource, {TERROR_EFFECTS}. While 26July executes Terror, Resource transfers are open (R 1.5.1; offering them from this choice is local policy). Going back to this choice is allowed until the first effect (local policy)."),
            ),
            under(
                {
                    let mut kidnap = self.offer(
                        "terror:kidnap",
                        "Kidnap · with Terror · offered once chosen",
                        false,
                        "R 4.1, 4.3.3: Kidnap accompanies Terror in a space selected for that Terror, here Santiago. Choose Terror; Kidnap is then offered.",
                    );
                    kidnap.remedies.push("Choose Terror".to_owned());
                    kidnap
                },
                "terror",
            ),
        ]
    }
    fn directorio_choices(&self) -> Vec<Choice> {
        [
            (
                "directorio:pass",
                "Pass",
                "Pass is legal. Directorio would gain 1 Resource and stay Eligible (R 2.3.3). Pass execution is outside this task.",
            ),
            (
                "event:unshaded",
                "Event · unshaded",
                "Directorio chooses the beneficiary. That faction directs the Rally, at no Resource cost (R 5.1, 5.5).",
            ),
            (
                "event:shaded",
                "Event · shaded",
                "0 Resources. Already Active Guerrillas stay Active.",
            ),
            (
                "directorio:limited-operation",
                "Limited Operation",
                "A Limited Operation is legal in just one space without a Special Activity. March may bring pieces from several spaces to that one destination (R 2.3.5). Limited Operation execution is outside this task.",
            ),
        ]
        .into_iter()
        .map(|(id, label, explanation)| self.offer(id, label, id.starts_with("event:"), explanation))
        .collect()
    }
    fn choices(&self) -> Vec<Choice> {
        if self.finished {
            return match &self.event {
                Some(EventProgress::ChooseRallyBeneficiary) => self.rally_beneficiary_choices(),
                Some(EventProgress::Rally(rally)) => self.rally_choices(rally),
                None if self.eligibility().directorio_handoff() => self.directorio_choices(),
                _ => vec![],
            };
        }
        if self.action == ActionChoice::Open {
            return self.opening_choices();
        }
        if self.kidnap_open {
            let mut roll = self.kidnap_offer();
            roll.id = format!("{}:kidnap:roll", self.revision);
            roll.label = "Roll the Kidnap die · commits the result".into();
            return vec![
                roll,
                self.offer(
                    "kidnap:cancel",
                    "Cancel Kidnap · nothing rolled",
                    true,
                    "Closes Kidnap without rolling: the match and its randomness stay unchanged, and it is no effect (local policy, #58).",
                ),
            ];
        }
        // Removal mode, including a #8 save's pending confirmation.
        if self.removal.is_some() {
            let mut choices = self.removal_choices();
            choices.push(self.offer(
                "done-removal",
                "Done · leave removal mode",
                true,
                "Leaves removal mode for the Operation. Pieces already removed stay in Available; nothing is reversed.",
            ));
            return choices;
        }
        let finish = || {
            let mut end = self.offer(
                "end-terror",
                "End Terror",
                !self.owes_santiago(),
                if self.owes_santiago() {
                    SANTIAGO_OWED
                } else if self.kidnap.is_some() {
                    "Ends 26July's Terror and records Operation with Special Activity (R 2.3.4)."
                } else {
                    "Ends 26July's Terror and records Operation Only (R 2.3.4)."
                },
            );
            // The commitment policy guarantees the owed Terror's payment and its
            // Underground Guerrilla, so resolving it is always possible here.
            if self.owes_santiago() {
                end.remedies.push("Resolve Santiago's Terror".to_owned());
            }
            end
        };
        let back = |label: &str| {
            let mut back = self.offer(
                "back-to-choice",
                label,
                true,
                "Local policy (#58), not a publisher ruling: until the first effect (an authorized transfer, a resolved Terror, a Kidnap roll or a removal), 26July may return to its action choice. A pending Transfer request is withdrawn; nothing has moved.",
            );
            back.owner = Faction::M26;
            back
        };
        match self.transfer {
            // The transfer modal: "Give to" moves 26July's own Resources at once;
            // "Ask from" creates a Transfer request that moves nothing (R 1.5.1).
            Some(Transfer::Selecting) => {
                let mut choices = vec![];
                for terms in self.transfer_terms() {
                    let (verb, other) = if terms.giver == Faction::M26 {
                        ("give", terms.recipient)
                    } else {
                        ("ask", terms.giver)
                    };
                    for n in 0..=terms.limit {
                        choices.push(if verb == "give" {
                            self.offer(
                                &format!("give:{}:{n}", other.index()),
                                format!("Give {n} to {}", other.name()),
                                true,
                                "26July's own Resource transfer moves at once (R 1.5.1).",
                            )
                        } else {
                            self.offer(
                                &format!("ask:{}:{n}", other.index()),
                                format!("Ask {} for {n}", other.name()),
                                true,
                                format!("A Transfer request: nothing moves until {} decides the actual amount (R 1.5.1).", other.name()),
                            )
                        });
                    }
                }
                choices.push(self.offer(
                    "back",
                    "Cancel · nothing moves",
                    true,
                    "Closes the transfer without moving anything.",
                ));
                return choices;
            }
            Some(Transfer::Authorizing { giver, waited, .. }) => {
                let limit = self.transfer_terms()[0].limit;
                let mut choices: Vec<_> = (0..=limit)
                    .map(|n| {
                        self.offer(
                            &format!("authorize:{n}"),
                            format!("{} authorizes {n}", giver.name()),
                            true,
                            "Only the giver decides the actual amount, from 0 to its limit; it moves on authorization.",
                        )
                    })
                    .collect();
                choices.push(self.offer(
                    "decline",
                    format!("{} declines", giver.name()),
                    true,
                    "Nothing moves, and 26July decides again.",
                ));
                // Wait sets the decision aside and Open brings it back; each is
                // saved, so a restart shows the request as it was left (#75).
                choices.push(if waited {
                    self.offer(
                        "reopen",
                        format!("Open · {} decides", giver.name()),
                        true,
                        "Shows the giver's decision again; the request is unchanged.",
                    )
                } else {
                    self.offer(
                        "wait",
                        "Wait · keep this decision",
                        true,
                        "The request stays pending, also across save and quit.",
                    )
                });
                // Local policy (#58): 26July's request does not hold 26July's own
                // Operation hostage; ending it or going back withdraws the request.
                if giver != Faction::M26 {
                    if self.may_go_back() {
                        choices.push(back("26July withdraws · back to action choice"));
                    }
                    let mut end = finish();
                    end.label = format!("26July withdraws · {}", end.label);
                    end.owner = Faction::M26;
                    // Nothing else is offered until the giver decides, so a remedy
                    // waits on that decision and says so (#58 story 53).
                    for remedy in &mut end.remedies {
                        *remedy = format!("{remedy} once {} has decided", giver.name());
                    }
                    choices.push(end);
                }
                return choices;
            }
            None => {}
        }
        let mut choices = vec![
            finish(),
            self.offer(
                "transfer",
                "Resource transfer",
                true,
                "Opens the transfer: give 26July's Resources to a faction, or ask one for Resources, which it decides (R 1.5.1).",
            ),
            self.offer(
                "remove",
                "Remove own pieces",
                true,
                "Voluntary removal (R 1.4.1): removal mode marks each 26July Guerrilla and Base on the map; remove any quantity of a stack, one decision at a time, until Done. Only an already owed Terror restricts removal.",
            ),
        ];
        let kidnap = self.kidnap.is_none().then(|| self.kidnap_offer());
        for id in ["santiago", "oriente"] {
            choices.push(self.terror_target(id, kidnap.as_ref()));
        }
        choices.extend(kidnap);
        if self.may_go_back() {
            choices.push(back("Back to the action choice"));
        }
        choices
    }
    /// Opening Kidnap in Santiago (R 4.1, 4.3.3): offered while it outnumbers the
    /// Police and Santiago's Terror is resolved or guaranteed on every roll. The roll
    /// is offered on the same terms.
    fn kidnap_offer(&self) -> Choice {
        let santiago = self.spaces.iter().find(|s| s.id == KIDNAP_SPACE).unwrap();
        let count = |kind, faction, state: Option<State>| -> u8 {
            santiago
                .pieces
                .iter()
                .filter(|p| {
                    p.kind == kind && p.faction == faction && state.is_none_or(|s| p.state == s)
                })
                .map(|p| p.count)
                .sum()
        };
        let guerrillas = count(Kind::Guerrilla, Faction::M26, None);
        let active = count(Kind::Guerrilla, Faction::M26, Some(State::Active));
        let underground = count(Kind::Guerrilla, Faction::M26, Some(State::Underground));
        let police = count(Kind::Police, Faction::Government, None);
        let funded = self.resources[Faction::M26.index()] > 0
            || self.resources[Faction::Government.index()] > 0;
        let feasible = santiago.terror > 0 || (underground > 0 && funded);
        // Both states count toward outnumbering (R 4.3.3); the words give each count.
        let noun = |n: u8| if n == 1 { "Guerrilla" } else { "Guerrillas" };
        let counted = match (active, underground) {
            (0, 0) => "0 Guerrillas".to_owned(),
            (a, 0) => format!("{a} Active {}", noun(a)),
            (0, u) => format!("{u} Underground {}", noun(u)),
            (a, u) => format!("{guerrillas} Guerrillas ({a} Active, {u} Underground)"),
        };
        let mut kidnap = self.offer(
            "kidnap",
            "Kidnap · Santiago · Government",
            guerrillas > police && feasible,
            format!("{counted} vs {police} Police; must outnumber. Die 1–6: proceeds {}–{} from Government. {}", self.resources[Faction::Government.index()].min(1), self.resources[Faction::Government.index()].min(6), if feasible {"Santiago Terror was resolved or is guaranteed feasible on every roll; an early roll commits its payment and Underground Guerrilla."} else {"Santiago needs an Underground Guerrilla and guaranteed payment without a promised Resource transfer or favorable roll."}),
        );
        // Only payment can still change: nothing in this task places a Guerrilla,
        // removes Police or makes an Active Guerrilla Underground again. With
        // Government at 0, Kidnap takes nothing, so only a transfer to 26July funds it.
        if guerrillas > police
            && !feasible
            && underground > 0
            && self.executing().is_some()
            && [Faction::Directorio, Faction::Syndicate]
                .iter()
                .any(|f| self.resources[f.index()] > 0)
        {
            kidnap
                .remedies
                .push("Request a Resource transfer".to_owned());
        }
        kidnap
    }
    /// A Terror target, listed even when 26July cannot act on it now: its reason and
    /// the remedies possible in this task (R 3.1, 3.3.4).
    fn terror_target(&self, id: &str, kidnap: Option<&Choice>) -> Choice {
        let space = self.spaces.iter().find(|s| s.id == id).unwrap();
        let m26 = self.resources[Faction::M26.index()];
        let reserve = u8::from(self.owes_santiago() && id != "santiago");
        let underground = space.pieces.iter().any(|p| {
            p.faction == Faction::M26
                && p.kind == Kind::Guerrilla
                && p.state == State::Underground
                && p.count > 0
        });
        let cost = format!(
            "Cost: {} per space (26July has {m26}).",
            resources(TERROR_COST)
        );
        let mut remedies = vec![];
        let reason = if space.terror > 0 {
            Some(
                "Terror already resolved here: each space at most once per Operation (R 3.1)."
                    .to_owned(),
            )
        } else if !underground {
            // Nothing in this task places a Guerrilla, so a removed one has no remedy.
            Some(
                "There is no Underground 26July Guerrilla there to flip to Active (R 3.3.4)."
                    .to_owned(),
            )
        } else if reserve > 0 && m26 <= reserve {
            Some(format!(
                "{cost} Santiago Terror is owed: the accepted commitment policy reserves 1 Resource for it, so Oriente needs another."
            ))
        } else if m26 == 0 {
            Some(cost.clone())
        } else {
            None
        };
        if reason.is_some() && space.terror == 0 && underground {
            // Kidnap funds Santiago only when it would take at least 1 Resource.
            if id == "santiago"
                && kidnap.is_some_and(|k| k.supported)
                && self.resources[Faction::Government.index()] > 0
            {
                remedies.push("Kidnap here first".to_owned());
            }
            if self.executing().is_some()
                && [Faction::Government, Faction::Directorio, Faction::Syndicate]
                    .iter()
                    .any(|f| self.resources[f.index()] > 0)
            {
                remedies.push("Request a Resource transfer".to_owned());
            }
        }
        let mut choice = self.offer(
            &format!("terror:{id}"),
            format!("Terror · {}", space.name),
            reason.is_none(),
            match reason {
                Some(reason) => format!("R 3.3.4. {reason}"),
                None => format!(
                    "R 3.3.4. {cost} Then {TERROR_EFFECTS} ({} → {}).",
                    support_level(space.support),
                    support_level((space.support - 1).max(-2))
                ),
            },
        );
        choice.remedies = remedies;
        choice
    }
    /// Whether the owed-Terror protection holds this group's last piece: the last
    /// Santiago Underground Guerrilla while Santiago Terror is owed (accepted
    /// commitment policy, local).
    fn protects(&self, id: &str, state: State) -> bool {
        self.owes_santiago() && id == "santiago" && state == State::Underground
    }
    /// How many of a 26July group may be removed now.
    fn removable(&self, id: &str, kind: Kind, state: State) -> u8 {
        self.spaces
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| {
                s.pieces
                    .iter()
                    .find(|p| p.faction == Faction::M26 && p.kind == kind && p.state == state)
            })
            .map_or(0, |p| {
                p.count.saturating_sub(u8::from(self.protects(id, state)))
            })
    }
    /// Removal mode: every quantity of every 26July group on the map (R 1.4.1). A
    /// quantity beyond what the owed-Terror protection allows is listed, refused with
    /// its reason.
    fn removal_choices(&self) -> Vec<Choice> {
        let mut choices = vec![];
        for s in &self.spaces {
            for p in s.pieces.iter().filter(|p| p.faction == Faction::M26) {
                let group = removal_group(p.kind, p.state);
                let limit = self.removable(&s.id, p.kind, p.state);
                let (one, many) = match (p.kind, p.state) {
                    (Kind::Base, _) => ("Base", "Bases"),
                    (_, State::Active) => ("Active Guerrilla", "Active Guerrillas"),
                    _ => ("Underground Guerrilla", "Underground Guerrillas"),
                };
                for n in 1..=p.count {
                    let pieces = if n == 1 { one } else { many };
                    let explanation = if n <= limit {
                        format!(
                            "R 1.4.1: return {n} 26July {pieces} in {} to Available. Earlier Terror markers, support shifts and Kidnap results stay.",
                            s.name
                        )
                    } else if limit == 0 {
                        format!(
                            "Santiago Terror is owed. Under the accepted commitment policy (local, not a publisher ruling), {}'s last Underground Guerrilla stays until that Terror resolves. It becomes removable once Terror in {} has resolved.",
                            s.name, s.name
                        )
                    } else {
                        format!(
                            "At most {limit}: {} protected while Santiago Terror is owed (accepted commitment policy, local).",
                            if p.count == 2 {
                                "the other is"
                            } else {
                                "the last one is"
                            }
                        )
                    };
                    choices.push(self.offer(
                        &format!("remove:{}:{group}:{n}", s.id),
                        format!("{} · remove {n} {pieces}", s.name),
                        n <= limit,
                        explanation,
                    ));
                }
            }
        }
        choices
    }
}
impl Game {
    pub fn random_state(&self) -> u64 {
        self.random
    }
    /// Deterministic prepared task for a supplied random state. Ordinary saves
    /// use OS entropy once; resumed saves always keep their consumed state.
    pub fn prepared_with_seed(seed: u64) -> Self {
        let mut game = Self::prepared();
        game.random = seed;
        game
    }
    /// A new task: the prepared position at 26July's 1st Eligible action choice.
    pub fn opening() -> Self {
        Self::opening_with_seed(Self::prepared().random)
    }
    /// A new task for a supplied random state; new saves use OS entropy once.
    pub fn opening_with_seed(seed: u64) -> Self {
        let mut game = Self::prepared_with_seed(seed);
        game.action = ActionChoice::Open;
        game
    }
    fn owes_santiago(&self) -> bool {
        self.kidnap.is_some()
            && self
                .spaces
                .iter()
                .any(|s| s.id == "santiago" && s.terror == 0)
    }
    pub fn submit(&mut self, owner: Faction, revision: u64, choice: &str) -> Result<(), String> {
        if revision != self.revision {
            return Err("Stale match revision; observe again.".into());
        }
        let offered = self
            .choices()
            .into_iter()
            .find(|c| c.id == choice)
            .ok_or_else(|| {
                // An amount beyond a transfer's limit is refused with its reason.
                choice
                    .split_once(':')
                    .and_then(|(_, action)| self.transfer_refusal(action))
                    .unwrap_or_else(|| "Stale or unknown choice identity.".into())
            })?;
        if owner != offered.owner {
            return Err("Only the current decision owner may submit.".into());
        }
        if !offered.supported {
            return Err(offered.explanation);
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or("Match revision exhausted.")?;
        let action = choice.split_once(':').ok_or("Unknown choice.")?.1;
        match action {
            // The giver's decision set aside, or shown again: the request itself, and
            // whether going back is allowed, are unchanged (#75).
            "wait" | "reopen" => {
                if let Some(Transfer::Authorizing { waited, .. }) = &mut self.transfer {
                    *waited = action == "wait";
                }
            }
            // R 2.3.3. Only offered at the action choice.
            "pass" => {
                self.resources[Faction::M26.index()] = self.resources[Faction::M26.index()]
                    .saturating_add(1)
                    .min(RESOURCE_CAP);
                self.action = ActionChoice::Pass;
                self.finished = true;
            }
            "terror" if self.action == ActionChoice::Open => self.action = ActionChoice::Terror,
            // Local policy (#58): withdraws a pending request; nothing has moved.
            "back-to-choice" => {
                self.action = ActionChoice::Open;
                self.transfer = None;
            }
            // Ending closes transfers and withdraws a pending request.
            "end-terror" => {
                self.finished = true;
                self.transfer = None;
            }
            "event:shaded" => self.execute_shaded_event(),
            "event:unshaded" => self.event = Some(EventProgress::ChooseRallyBeneficiary),
            _ if matches!(self.event, Some(EventProgress::ChooseRallyBeneficiary)) => {
                self.select_rally_beneficiary(action)?;
            }
            _ if let Some(EventProgress::Rally(rally)) = &self.event => {
                self.resolve_rally(rally.clone(), action)?;
            }
            _ => return self.execute(action, next_revision),
        }
        self.revision = next_revision;
        Ok(())
    }
    /// The #8 Operation steps: removal, transfers, Terror and Kidnap.
    fn execute(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        // Removal mode: only its own choices are offered, so `submit` has already
        // bound the space, group and quantity to a supported removal.
        if self.removal.is_some() {
            if action == "done-removal" {
                self.removal = None;
            } else {
                let mut parts = action.split(':').skip(1);
                let (space, group, n) = (parts.next(), parts.next(), parts.next());
                let (kind, state) = group.and_then(removal_piece).ok_or("Unknown choice.")?;
                let n: u8 = n.and_then(|n| n.parse().ok()).ok_or("Unknown choice.")?;
                let target = self
                    .spaces
                    .iter_mut()
                    .find(|s| Some(s.id.as_str()) == space)
                    .ok_or("Unknown choice.")?;
                let piece = target
                    .pieces
                    .iter_mut()
                    .find(|p| p.faction == Faction::M26 && p.kind == kind && p.state == state)
                    .ok_or("Unknown choice.")?;
                piece.count = piece.count.checked_sub(n).ok_or("Unknown choice.")?;
                target.pieces.retain(|p| p.count > 0);
                self.effect = true;
                self.available
                    .iter_mut()
                    .find(|p| p.faction == Faction::M26 && p.kind == kind)
                    .unwrap()
                    .count += n;
                // A #8 save's pending confirmation becomes ordinary removal mode.
                self.removal = Some(Removal::Selecting);
            }
            self.revision = next_revision;
            return Ok(());
        }
        match self.transfer.clone() {
            None if action == "remove" => self.removal = Some(Removal::Selecting),
            None if action == "kidnap" => self.kidnap_open = true,
            None if action == "kidnap:cancel" => self.kidnap_open = false,
            None if action == "kidnap:roll" => {
                self.kidnap_open = false;
                self.effect = true;
                let mut random = SeededRandom::new(self.random);
                let die = (random.index(6) + 1) as u8;
                let (government, m26) = (Faction::Government.index(), Faction::M26.index());
                let roll = Kidnap {
                    die,
                    proceeds: die.min(self.resources[government]),
                    government_before: self.resources[government],
                    m26_before: self.resources[m26],
                    random_before: self.random,
                    revision: next_revision,
                };
                self.random = random.state();
                self.resources[government] = roll.government_after();
                self.resources[m26] = roll.m26_after();
                self.kidnap = Some(roll);
            }
            None if action.starts_with("terror:") => {
                let id = action.strip_prefix("terror:").unwrap();
                let space = self.spaces.iter_mut().find(|s| s.id == id).unwrap();
                Self::resolve_terror(space);
                self.resources[Faction::M26.index()] -= TERROR_COST;
                self.effect = true;
            }
            None => self.transfer = Some(Transfer::Selecting),
            Some(_) if action == "back" || action == "decline" => self.transfer = None,
            Some(Transfer::Selecting) => {
                let (verb, rest) = action.split_once(':').unwrap();
                let (other, amount) = rest.split_once(':').unwrap();
                let other = Faction::ALL[other.parse::<usize>().unwrap()];
                let amount: u8 = amount.parse().unwrap();
                if verb == "give" {
                    self.move_resources(Faction::M26, other, amount);
                    self.transfer = None;
                } else {
                    self.transfer = Some(Transfer::Authorizing {
                        giver: other,
                        recipient: Faction::M26,
                        amount,
                        waited: false,
                    });
                }
            }
            Some(Transfer::Authorizing {
                giver, recipient, ..
            }) => {
                let amount = action.strip_prefix("authorize:").unwrap().parse().unwrap();
                self.move_resources(giver, recipient, amount);
                self.transfer = None;
            }
        }
        self.revision = next_revision;
        Ok(())
    }
    /// An authorized Resource transfer, within the limits that offered it.
    fn move_resources(&mut self, giver: Faction, recipient: Faction, amount: u8) {
        self.resources[giver.index()] -= amount;
        self.resources[recipient.index()] += amount;
        // Zero is a deliberate non-transfer, not an effect.
        self.effect |= amount > 0;
    }
    /// The terms of the transfers 26July may make now (R 1.5.1, 1.8 and the accepted
    /// commitment policy's reserve).
    fn transfer_terms(&self) -> Vec<TransferTerms> {
        let pairs = match self.transfer {
            Some(Transfer::Selecting) => {
                [Faction::Government, Faction::Directorio, Faction::Syndicate]
                    .into_iter()
                    .flat_map(|other| [(other, Faction::M26), (Faction::M26, other)])
                    .collect()
            }
            Some(Transfer::Authorizing {
                giver, recipient, ..
            }) => vec![(giver, recipient)],
            None => vec![],
        };
        pairs
            .into_iter()
            .map(|(giver, recipient)| {
                let balance = self.resources[giver.index()];
                let held = self.resources[recipient.index()];
                let room = RESOURCE_CAP.saturating_sub(held);
                let reserved = u8::from(giver == Faction::M26 && self.owes_santiago());
                let spendable = balance.saturating_sub(reserved);
                let bound = if spendable > room {
                    Bound::Cap
                } else if reserved > 0 && balance > 0 {
                    Bound::Reserve
                } else {
                    Bound::Balance
                };
                TransferTerms {
                    giver,
                    recipient,
                    balance,
                    held,
                    room,
                    reserved,
                    limit: spendable.min(room),
                    bound,
                }
            })
            .collect()
    }
    /// Why an amount beyond its limit is refused, in the terms' own words: 26July's own Resource
    /// transfer or request the modal offers, or the pending request's authorization.
    fn transfer_refusal(&self, action: &str) -> Option<String> {
        let terms = self.transfer_terms();
        let parts: Vec<_> = action.split(':').collect();
        let (direction, amount) = match parts[..] {
            ["authorize", n] => (terms.first()?, n),
            [verb @ ("give" | "ask"), other, n] => {
                let other = *Faction::ALL.get(other.parse::<usize>().ok()?)?;
                let (giver, recipient) = if verb == "give" {
                    (Faction::M26, other)
                } else {
                    (other, Faction::M26)
                };
                (
                    terms
                        .iter()
                        .find(|t| t.giver == giver && t.recipient == recipient)?,
                    n,
                )
            }
            _ => return None,
        };
        direction.refusal(amount.parse().ok()?)
    }
    fn resolve_terror(space: &mut Space) {
        let underground = space
            .pieces
            .iter_mut()
            .find(|p| {
                p.faction == Faction::M26
                    && p.kind == Kind::Guerrilla
                    && p.state == State::Underground
            })
            .unwrap();
        underground.count -= 1;
        space.pieces.retain(|p| p.count > 0);
        space.pieces.push(Group {
            faction: Faction::M26,
            kind: Kind::Guerrilla,
            state: State::Active,
            count: 1,
        });
        space.terror += 1;
        space.support = (space.support - 1).max(-2);
    }
    fn execute_shaded_event(&mut self) {
        let mut flips = Vec::new();
        for space in &mut self.spaces {
            let m26_guerrilla = |p: &Group| p.faction == Faction::M26 && p.kind == Kind::Guerrilla;
            let Some(underground) = space
                .pieces
                .iter()
                .position(|p| m26_guerrilla(p) && p.state == State::Underground)
            else {
                continue;
            };
            let count = space.pieces[underground].count;
            if let Some(active) = space
                .pieces
                .iter_mut()
                .find(|p| m26_guerrilla(p) && p.state == State::Active)
            {
                active.count += count;
                space.pieces.remove(underground);
            } else {
                space.pieces[underground].state = State::Active;
            }
            flips.push(Flip {
                space: space.id.clone(),
                faction: Faction::M26,
                kind: Kind::Guerrilla,
                from: State::Underground,
                to: State::Active,
                count,
            });
        }
        self.event = Some(EventProgress::Complete(EventOutcome::Shaded { flips }));
    }
    /// Admit the prepared card-17 task after only the supported decisions and effects.
    pub fn validate(&self) -> Result<(), String> {
        let shaded = match &self.event {
            Some(EventProgress::ChooseRallyBeneficiary) => return self.validate_unshaded(None),
            Some(
                EventProgress::Rally(rally)
                | EventProgress::Complete(EventOutcome::Unshaded(rally)),
            ) => {
                return self.validate_unshaded(Some(rally));
            }
            Some(EventProgress::Complete(EventOutcome::Shaded { flips })) => {
                if !self.finished
                    || self.action != ActionChoice::Terror
                    || self
                        .kidnap
                        .as_ref()
                        .is_none_or(|r| self.revision.saturating_sub(r.revision) < 2)
                {
                    return Err("Shaded Alberto Bayo requires completed Terror with Kidnap.".into());
                }
                Some(flips.as_slice())
            }
            None => None,
        };
        let mut expected = Self::prepared_with_seed(self.random);
        let mut paid = 0;
        let mut removed = 0u64;
        for space in &mut expected.spaces {
            if let Some(actual) = self.spaces.iter().find(|s| s.id == space.id)
                && actual.terror == 1
                && matches!(space.id.as_str(), "santiago" | "oriente")
            {
                Self::resolve_terror(space);
                paid += 1;
            }
            if let Some(actual) = self.spaces.iter().find(|s| s.id == space.id) {
                let activated = shaded
                    .and_then(|flips| flips.iter().find(|f| f.space == space.id))
                    .map_or(0, |f| f.count);
                for piece in &mut space.pieces {
                    if piece.faction != Faction::M26 {
                        continue;
                    }
                    let on_map = |state| {
                        actual
                            .pieces
                            .iter()
                            .find(|p| {
                                p.faction == piece.faction
                                    && p.kind == piece.kind
                                    && p.state == state
                            })
                            .map_or(0, |p| p.count)
                    };
                    // Infer the inherited counts from the recorded flips. The
                    // exact replay below verifies every flip and final group.
                    let remaining = match (shaded, piece.kind, piece.state) {
                        (Some(_), Kind::Guerrilla, State::Underground) => activated,
                        (Some(_), Kind::Guerrilla, State::Active) => on_map(State::Active)
                            .checked_sub(activated)
                            .ok_or("Unsupported shaded Event activation.")?,
                        _ => on_map(piece.state),
                    };
                    let returned = piece
                        .count
                        .checked_sub(remaining)
                        .ok_or("Unsupported added or activated own pieces.")?;
                    removed += u64::from(returned);
                    expected
                        .available
                        .iter_mut()
                        .find(|p| p.faction == Faction::M26 && p.kind == piece.kind)
                        .unwrap()
                        .count += returned;
                    piece.count = remaining;
                }
                space.pieces.retain(|p| p.count > 0);
            }
        }
        // Pass and the choice itself have no other effect; only Terror executes.
        let untouched = !self.effect
            && paid == 0
            && removed == 0
            && self.kidnap.is_none()
            && !self.kidnap_open
            && self.transfer.is_none()
            && self.removal.is_none();
        let mut passed = PREPARED_RESOURCES;
        passed[Faction::M26.index()] += 1;
        let consistent = match self.action {
            ActionChoice::Open => {
                untouched && !self.finished && self.resources == PREPARED_RESOURCES
            }
            ActionChoice::Pass => untouched && self.finished && self.resources == passed,
            ActionChoice::Terror => true,
        };
        expected.action = self.action;
        if !consistent
            || self.resources.iter().any(|&n| n > RESOURCE_CAP)
            || self.resources.iter().map(|&n| n as u16).sum::<u16>()
                != 26 - paid + u16::from(self.action == ActionChoice::Pass)
            || self.revision
                < u64::from(paid)
                    + u64::from(self.kidnap.is_some())
                    + u64::from(self.finished)
                    + u64::from(shaded.is_some())
                    // Entering removal mode, then at least one removal decision,
                    // which may return several pieces of one group (#73).
                    + if removed > 0 { 2 } else { u64::from(self.removal.is_some()) }
            || (self.revision == 0 && *self != expected)
            || (self.finished && (self.transfer.is_some() || self.removal.is_some()))
            || (self.transfer.is_some() && self.removal.is_some())
            // An opened Kidnap is a modal step of Terror execution, still unrolled
            // and on the terms that offered it.
            || (self.kidnap_open
                && (self.action != ActionChoice::Terror
                    || self.finished
                    || self.kidnap.is_some()
                    || self.transfer.is_some()
                    || self.removal.is_some()
                    || self.revision == 0
                    || !self.kidnap_offer().supported))
        {
            return Err("Unsupported transfer balances or decision state.".into());
        }
        if let Some(kidnap) = &self.kidnap {
            let mut rng = SeededRandom::new(kidnap.random_before);
            let die = (rng.index(6) + 1) as u8;
            if kidnap.die != die
                || kidnap.revision > self.revision
                || self.random != rng.state()
                || kidnap.government_before > 26
                || kidnap.m26_before > 26
                || u16::from(kidnap.government_before) + u16::from(kidnap.m26_before) > 26
                || kidnap.proceeds != die.min(kidnap.government_before)
                || (self.owes_santiago()
                    && (self.finished
                        || self.resources[Faction::M26.index()] == 0
                        || !self.spaces.iter().any(|s| {
                            s.id == "santiago"
                                && s.pieces.iter().any(|p| {
                                    p.faction == Faction::M26
                                        && p.kind == Kind::Guerrilla
                                        && p.state == State::Underground
                                        && p.count > 0
                                })
                        })
                        || (kidnap.government_before == 0 && kidnap.m26_before == 0)))
            {
                return Err("Unsupported Kidnap result or impossible owed Santiago Terror.".into());
            }
        }
        if let Some(Removal::Confirming { space, kind, state }) = &self.removal
            && self.removable(space, *kind, *state) == 0
        {
            return Err("Unsupported pending own removal or protected owed Terror piece.".into());
        }
        if let Some(Transfer::Authorizing {
            giver,
            recipient,
            amount,
            ..
        }) = self.transfer
            && (giver == recipient
                || (giver != Faction::M26 && recipient != Faction::M26)
                || amount > self.resources[giver.index()]
                || (self.owes_santiago()
                    && giver == Faction::M26
                    && amount >= self.resources[Faction::M26.index()])
                || u16::from(self.resources[recipient.index()]) + u16::from(amount)
                    > u16::from(RESOURCE_CAP))
        {
            return Err("Unsupported pending giver authorization.".into());
        }
        expected.finished = self.finished;
        expected.revision = self.revision;
        expected.resources = self.resources;
        expected.transfer = self.transfer.clone();
        expected.kidnap = self.kidnap.clone();
        expected.removal = self.removal.clone();
        expected.effect = self.effect;
        expected.kidnap_open = self.kidnap_open;
        if shaded.is_some() {
            expected.execute_shaded_event();
        }
        if *self != expected {
            return Err(
                "State is not a supported prepared transfer/Terror/Kidnap/removal position.".into(),
            );
        }
        Ok(())
    }
}
