//! Cuba-owned rules from the accepted deacd635 tactical handoff (R 1.3–1.8, 2.3, 7).
use serde::{Deserialize, Serialize};
use unfolded_random::SeededRandom;

mod assault;
mod cards;
mod pile_facts;
mod rally;
mod titles;
pub use assault::AssaultResolution;
pub use cards::{
    CardFacts, CardWords, DECK_CARDS, EventKind, EventStatus, EventText, LastingEffect,
    PROPAGANDA_CARDS, Piles, Role, ShownCards, is_propaganda,
};
pub use pile_facts::{
    DeckFacts, DeckWords, NextPropaganda, PlayedCard, PlayedPile, PlayedWords, next_propaganda,
    next_propaganda_words,
};
pub use rally::{Rally, RallyAction, RallyResolution};
pub use titles::title;

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
/// The US Alliance level in this build: the setup's Firm, which no Operation or Event
/// it implements shifts. Every COIN Operation's cost reads it (R 6.3.1).
const ALLIANCE: UsAlliance = UsAlliance::Firm;
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
/// The Terror and Sabotage markers the two kinds share: a marker is placed only from
/// the marker supply, and with all of them on the map Terror places none (R 3.3.4).
pub const MARKER_SUPPLY: u8 = 20;
/// The marker supply (R 3.3.4): the shared Terror and Sabotage markers not on the map,
/// and those on it by kind. `available` is [`MARKER_SUPPLY`] less the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MarkerSupply {
    pub available: u8,
    pub terror_on_map: u8,
    pub sabotage_on_map: u8,
}
impl MarkerSupply {
    /// The supply left by the markers on the map. Nothing in this build places more
    /// than the supply holds, but a count past it shows none available, never a wrap.
    pub fn left_by(terror_on_map: u8, sabotage_on_map: u8) -> Self {
        Self {
            available: MARKER_SUPPLY
                .saturating_sub(terror_on_map)
                .saturating_sub(sabotage_on_map),
            terror_on_map,
            sabotage_on_map,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Troops,
    Police,
    Guerrilla,
    Base,
    Casino,
}
/// The Available Forces chart (rules, p. 20): every piece of each kind a faction
/// has, on the map or Available, before setup. It limits the pieces in play (R 1.4.1);
/// the Syndicate's Bases are its Casinos. The opening's Available Forces are these
/// less the pieces it sets up on the map.
pub const INVENTORY: [(Faction, Kind, u8); 9] = [
    (Faction::Government, Kind::Troops, 15),
    (Faction::Government, Kind::Police, 15),
    (Faction::Government, Kind::Base, 2),
    (Faction::M26, Kind::Guerrilla, 15),
    (Faction::M26, Kind::Base, 4),
    (Faction::Directorio, Kind::Guerrilla, 15),
    (Faction::Directorio, Kind::Base, 4),
    (Faction::Syndicate, Kind::Guerrilla, 6),
    (Faction::Syndicate, Kind::Casino, 10),
];
/// How many pieces of `kind` `faction` has in all (`INVENTORY`): 0 for a kind the
/// chart does not give it.
pub fn inventory(faction: Faction, kind: Kind) -> u8 {
    INVENTORY
        .iter()
        .find(|&&(f, k, _)| f == faction && k == kind)
        .map_or(0, |&(_, _, total)| total)
}
/// How many pieces of `kind` `faction` these groups hold, in any state.
pub(crate) fn total<'a>(
    groups: impl IntoIterator<Item = &'a Group>,
    faction: Faction,
    kind: Kind,
) -> u8 {
    groups
        .into_iter()
        .filter(|group| group.faction == faction && group.kind == kind)
        .map(|group| group.count)
        .sum()
}
/// The kinds a hand-built position holds in the wrong number, each as "26July
/// Guerrilla: 12 Available + 5 on the map, of 15". Every piece of the chart is
/// Available or on the map (R 1.4.1), and no other piece exists, so a position the
/// rules reach has none. In unit tests `Game::observe` refuses a position that has
/// one; a test that edits an observation by hand asserts this itself.
#[cfg(test)]
pub(crate) fn inventory_breaches(available: &[Group], spaces: &[Space]) -> Vec<String> {
    let on_map = || spaces.iter().flat_map(|space| &space.pieces);
    let mut kinds: Vec<(Faction, Kind)> = INVENTORY.iter().map(|&(f, k, _)| (f, k)).collect();
    for group in available.iter().chain(on_map()) {
        if !kinds.contains(&(group.faction, group.kind)) {
            kinds.push((group.faction, group.kind));
        }
    }
    kinds
        .into_iter()
        .filter_map(|(faction, kind)| {
            let in_available = total(available, faction, kind);
            let on_the_map = total(on_map(), faction, kind);
            let chart = inventory(faction, kind);
            (u16::from(in_available) + u16::from(on_the_map) != u16::from(chart)).then(|| {
                format!(
                    "{} {kind:?}: {in_available} Available + {on_the_map} on the map, of {chart}",
                    faction.name()
                )
            })
        })
        .collect()
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
    /// Terror is chosen: 26July executes it until End Terror.
    Terror,
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
/// The current card's Eligibility record (R 2.3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Eligibility {
    /// The printed faction order, left to right (R 2.3.2).
    pub order: [Faction; 4],
    /// Eligibility on the track, indexed like `Faction::ALL`. After adjustment
    /// these cylinders govern the next card.
    pub eligible: [bool; 4],
    /// The 1st Eligible faction. A Pass gives this role to the next Eligible faction.
    pub first: Faction,
    pub second: Option<Faction>,
    /// The box 26July's cylinder occupies, once its action is recorded.
    pub m26_box: Option<SequenceBox>,
    /// The box Government's cylinder occupies on card 18.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub government_box: Option<SequenceBox>,
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
const GOVERNMENT_PASS_INCOME: u8 = 3;
/// What 26July pays for Terror in each City or Province it selects (R 3.3.4).
pub const TERROR_COST: u8 = 1;
/// The remedy of a choice that another faction's Resources would make supported.
const TRANSFER_REMEDY: &str = "Request a Resource transfer";
/// The remedy of Santiago's Terror that a Kidnap roll would fund.
const KIDNAP_REMEDY: &str = "Kidnap here first";
/// What would open a space blocked with `remedies`, as one line of its tag (#170).
fn remedy_line(remedies: &[String]) -> String {
    let named = |remedy: &str| remedies.iter().any(|r| r == remedy);
    match (named(KIDNAP_REMEDY), named(TRANSFER_REMEDY)) {
        (true, true) => "Kidnap or a transfer opens it",
        (true, false) => "Kidnap here first opens it",
        _ => "A Resource transfer opens it",
    }
    .into()
}
/// The prepared position's Resources, indexed like `Faction::ALL`.
const PREPARED_RESOURCES: [u8; 4] = [6, 0, 5, 15];
/// The random state of `Game::opening`.
const OPENING_SEED: u64 = 17;
/// Card 17's order: 26July, Directorio, Government, Syndicate (rules baseline).
pub const CARD_17_ORDER: [Faction; 4] = [
    Faction::M26,
    Faction::Directorio,
    Faction::Government,
    Faction::Syndicate,
];
/// Card 17's Event in progress. A completed Event carries the applied effect so its
/// result and printed eligibility restriction survive restart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventProgress {
    ExecuteShaded,
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
    pub(crate) fn pending(&self) -> bool {
        self.outcome().is_none()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventOutcome {
    Shaded { flips: Vec<Flip> },
    Unshaded(Rally),
}
/// The match's position. A saved position decodes only in this shape: every field is
/// required, an `Option` included, and an unknown field is refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub(crate) revision: u64,
    pub(crate) resources: [u8; 4],
    pub(crate) spaces: Vec<Space>,
    pub(crate) available: Vec<Group>,
    pub(crate) m26_action_ended: bool,
    pub(crate) random: u64,
    pub(crate) draw_deck: Vec<u8>,
    /// Card 18 entered play after the completed Alberto Bayo Event.
    pub(crate) card_18: bool,
    /// The Government balance before its recorded card 18 Pass, which the record of
    /// its Resources shows.
    #[serde(deserialize_with = "Option::deserialize")]
    government_pass_from: Option<u8>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) transfer: Option<Transfer>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) kidnap: Option<Kidnap>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) removal: Option<Removal>,
    pub(crate) action: ActionChoice,
    /// Set by the first real effect: 26July's Pass, an authorized nonzero transfer, a
    /// resolved Terror, a Kidnap roll or a removal. Going back to the action choice is
    /// local policy (#58) allowed only before it.
    pub(crate) effect: bool,
    /// Kidnap opened and not yet rolled or cancelled (#63). Opening consumes no
    /// randomness; only the roll does.
    pub(crate) kidnap_open: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    pub(crate) event: Option<EventProgress>,
    #[serde(deserialize_with = "Option::deserialize")]
    assault: Option<assault::Assault>,
    /// The spaces where 26July's Terror resolved, in the order resolved (#171): the
    /// execution panel lists them so, which the map's markers cannot tell.
    pub(crate) terror_resolved: Vec<String>,
}

/// Removal mode (#73): open from "Remove own pieces" until Done. Each decision in it
/// returns a chosen quantity of the executing faction's group to Available (R 1.4.1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Removal {
    Selecting,
}
/// A removal choice's group, as its id names it: `remove:<space>:<group>:<quantity>`.
pub fn removal_group(kind: Kind, state: State) -> &'static str {
    match (kind, state) {
        (Kind::Troops, _) => "troops",
        (Kind::Police, _) => "police",
        (Kind::Base, _) => "base",
        (_, State::Active) => "active",
        _ => "underground",
    }
}
/// The kind and state named by a supported voluntary removal group.
pub fn removal_piece(group: &str) -> Option<(Kind, State)> {
    match group {
        "troops" => Some((Kind::Troops, State::Ordinary)),
        "police" => Some((Kind::Police, State::Ordinary)),
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
    /// tumble from it, independently of the die.
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
        /// tab in the top band reopens the decision, and a restart shows it so.
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
    /// Directorio owns the Event even while another faction directs the free Rally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_executor: Option<Faction>,
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
    /// The current and the preview card: their printed facts and where each Event
    /// stands, as the card tag shows them.
    pub cards: ShownCards,
    /// The count on each pile and the Propaganda cards played (R 2.2, 2.4.1). The
    /// played pile counts the current card and the draw deck the preview, so the two
    /// account for every card in the deck.
    pub piles: Piles,
    /// Every card in the played pile, most recent first, as its tag lists them (R 2.2).
    pub played_pile: PlayedPile,
    /// The draw deck's public facts, among them how soon the next Propaganda card can
    /// come, as its tag shows them. Derived without the face-down order.
    pub deck: DeckFacts,
    pub eligible: [bool; 4],
    pub passed: [bool; 4],
    pub aid: u8,
    pub alliance: UsAlliance,
    pub cash_available: u8,
    /// The marker supply and the markers on the map by kind (R 3.3.4).
    pub marker_supply: MarkerSupply,
    pub capabilities: Vec<String>,
    pub momentum: Vec<String>,
    /// True once 26July's first action on card 17 is recorded. Directorio's Event
    /// can still be pending. Card-end eligibility is in `eligibility.next_card`.
    pub finished: bool,
    pub choices: Vec<Choice>,
    /// The next faction to act, its role, and its options after a recorded action
    /// (R 2.3.3–2.3.4).
    pub next: Option<Next>,
    pub operation: &'static str,
    /// What Government paid in each space it Assaulted on this card and what was
    /// removed or closed there, in the order paid (#169). Empty outside an Assault.
    pub assault_paid: Vec<AssaultResolution>,
    pub special_activity: Option<String>,
    pub owed_terror: Option<String>,
    pub transfer: Option<Transfer>,
    /// The terms of each transfer the executing faction may make now: all six directions while the
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
    /// The balance before and after a recorded Pass on this card (R 2.3.3, 1.8).
    pub recorded_resources: Vec<ResourceChange>,
    /// Each space where 26July's Terror resolved, in the order resolved (#171), at the
    /// fixed per-space cost [`TERROR_COST`] it paid (R 3.3.4).
    pub resolved: Vec<Resolved>,
    /// What a Kidnap roll could take now (R 4.3.3), from current Resources, until
    /// Kidnap is rolled or the action is recorded.
    pub kidnap_stakes: Option<KidnapStakes>,
    /// While a decision offers choices that act on map spaces, why each space that
    /// needs one is in its state, in the map's space order (#170): what a paid or
    /// resolved space paid, an owed Terror, what would open a blocked space, or why
    /// nothing acts there. A space where a supported choice acts has none, and outside
    /// such a decision the list is empty. The space tag draws these words as they are.
    pub space_reasons: Vec<SpaceReason>,
}
/// The rules authority's reason for one space's state in the open decision (#170).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpaceReason {
    pub space: String,
    pub reason: String,
}
/// A space where 26July's Terror resolved, and the Resources it paid there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Resolved {
    pub space: String,
    pub paid: u8,
}
/// The rules authority's statement of where a choice acts on the map (#169).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Site {
    /// The id of the space the choice acts on.
    pub space: String,
    /// Whether the choice changes anything on the map there: an Assault plan that
    /// removes or closes at least one piece (R 3.2.4, 1.4.5), a Terror (R 3.3.4) or
    /// opening Kidnap (R 4.3.3). False for the Assault plan that removes nothing.
    pub effect: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub supported: bool,
    pub explanation: String,
    /// The faction that may submit it. Usually the observation's owner; while a
    /// Transfer request awaits its giver during Terror, 26July may still withdraw it.
    pub owner: Faction,
    /// For a Special Activity listed at the action choice, the id of the Operation
    /// it accompanies (R 4.1).
    pub accompanies: Option<String>,
    /// For an unsupported choice, the actions possible in this task that would make
    /// it supported, such as "Kidnap here first"; empty when nothing here can.
    pub remedies: Vec<String>,
    /// Where on the map the choice acts and whether it changes anything there, for a
    /// choice that acts on one space. The map and the space popover read this
    /// statement, and never a label, an id or an explanation (#169).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<Site>,
    /// The consequences of an irreversible step, computed by applying it to a copy of
    /// the match, including Pass, Terror and Assault effects, and voluntary removal.
    pub preview: Option<Preview>,
}
impl Choice {
    /// The action without its match revision: `terror:santiago` for the id
    /// `3:terror:santiago`.
    pub fn action(&self) -> &str {
        Self::action_of(&self.id)
    }
    /// The action in the choice id `id`, without its match revision.
    pub(crate) fn action_of(id: &str) -> &str {
        id.split_once(':').map_or(id, |(_, action)| action)
    }
    /// The map space this choice acts on, as the authority states it in `site`, so
    /// the map can offer it there: a Terror target, opening Kidnap in its space, or an
    /// Assault plan. Other choices act on no space.
    pub fn space(&self) -> Option<&str> {
        self.site.as_ref().map(|site| site.space.as_str())
    }
    /// States the space this choice acts on and whether it changes anything there.
    fn at(mut self, space: &str, effect: bool) -> Self {
        self.site = Some(Site {
            space: space.to_owned(),
            effect,
        });
        self
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
    /// `give:<recipient>:<amount>`: the executing faction's Resources move at once.
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
    /// Pieces returned to Available Forces by this decision.
    pub removed: Vec<Returned>,
    /// Spaces that receive a Terror marker.
    pub terror: Vec<String>,
    pub support: Vec<SupportShift>,
    pub control: Vec<ControlChange>,
    pub standings: Vec<StandingChange>,
    /// Choices offered now that the step would leave unavailable, with the reason
    /// they would then give.
    pub unavailable: Vec<Unavailable>,
    /// The box the active faction's action would record, when the step ends it.
    pub records: Option<SequenceBox>,
    /// Whether 26July would be Eligible for the next card, when the step ends its
    /// action (R 2.3.3, 2.3.7).
    pub m26_next_card: Option<bool>,
    /// The next decision's owner, role and options, when the step hands over play.
    pub next: Option<Next>,
    /// What a Kidnap roll could take, for opening Kidnap and for its roll. A roll's
    /// result is never previewed.
    pub kidnap: Option<KidnapStakes>,
    /// What paying for an Assault space states besides its computed changes, for an
    /// Assault payment only. The payment itself is in `resources`.
    pub assault: Option<AssaultPreview>,
    /// Facts with rule references, such as the zero-space warning.
    pub notes: Vec<String>,
}
/// The parts of an Assault payment's preview that its computed changes do not show,
/// each stated by name (R 3.2.4).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AssaultPreview {
    /// The capacity arithmetic: the cubes that count in the space's terrain.
    pub arithmetic: String,
    /// Whom the plan can affect, and what protects the rest.
    pub affects: String,
    /// What the plan removes or closes, or that it affects no piece.
    pub effect: String,
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
    /// The opening position: the accepted prepared position at 26July's 1st Eligible
    /// action choice, at revision 0, with the given random state. Every match and
    /// every save starts here, from a seed drawn once from the OS or a given one.
    pub fn opening_with_seed(seed: u64) -> Self {
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
        let spaces: Vec<Space> = rows
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
        // Every piece the opening does not set up on the map is Available (R 1.4.1).
        let available = INVENTORY
            .into_iter()
            .map(|(faction, kind, inventory)| {
                let on_map = total(spaces.iter().flat_map(|space| &space.pieces), faction, kind);
                let left = inventory
                    .checked_sub(on_map)
                    .expect("the opening sets up no more pieces than the chart holds");
                group(faction, kind, left)
            })
            .collect();
        Self {
            revision: 0,
            resources: PREPARED_RESOURCES,
            spaces,
            m26_action_ended: false,
            random: seed,
            draw_deck,
            card_18: false,
            government_pass_from: None,
            transfer: None,
            kidnap: None,
            removal: None,
            action: ActionChoice::Open,
            effect: false,
            kidnap_open: false,
            event: None,
            assault: None,
            terror_resolved: Vec::new(),
            available,
        }
    }
    pub fn observe(&self) -> Observation {
        // A unit test that builds its position by hand keeps the chart (p. 20).
        #[cfg(test)]
        {
            let breaches = inventory_breaches(&self.available, &self.spaces);
            assert!(
                breaches.is_empty(),
                "this position breaks the Available Forces chart: {breaches:?}"
            );
        }
        let mut played_cards: Vec<u8> = (1..=12).collect();
        played_cards.push(49);
        played_cards.extend(13..=16);
        if self.card_18 {
            played_cards.push(17);
        }
        let eligibility = self.eligibility();
        let current_card = if self.card_18 { 18 } else { 17 };
        let piles = Piles::new(&played_cards, current_card, self.draw_deck.len());
        let played_pile = pile_facts::played_pile(&played_cards, current_card);
        // Only the deck's number and its face-up top card are public (R 2.2, 2.3.8).
        let deck = pile_facts::deck_facts(&played_pile, piles.deck, self.draw_deck[0]);
        // No Capability or Momentum is in effect in this build.
        let capabilities: Vec<String> = vec![];
        let choices: Vec<Choice> = self
            .choices()
            .into_iter()
            .map(|mut choice| {
                choice.preview = self.preview(&choice);
                choice
            })
            .collect();
        let space_reasons = self.space_reasons(&choices);
        Observation {
            revision: self.revision,
            owner: self.owner(),
            action: self.action,
            executing: self.executing(),
            event_executor: self.event.as_ref().map(|_| Faction::Directorio),
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
            current_card,
            preview_card: self.draw_deck[0],
            played_cards,
            draw_count: self.draw_deck.len(),
            cards: self.shown_cards(current_card, self.draw_deck[0], &capabilities),
            piles,
            played_pile,
            deck,
            passed: Faction::ALL.map(|f| match f {
                Faction::M26 => self.action == ActionChoice::Pass,
                Faction::Government => self.government_pass_from.is_some(),
                _ => false,
            }),
            aid: 15,
            alliance: ALLIANCE,
            cash_available: CASH_MARKERS,
            // No Operation, Event or phase this build implements places a Sabotage
            // marker (R 3.3.4 in an economic center, R 4.2.3, R 6.2.1), so the
            // model holds none.
            marker_supply: MarkerSupply::left_by(
                self.spaces.iter().map(|s| s.terror).sum::<u8>(),
                0,
            ),
            capabilities,
            momentum: vec![],
            finished: self.m26_action_ended,
            choices,
            next: self.next(),
            operation: match self.action {
                _ if self.government_pass_from.is_some() => {
                    "Government Passed: Syndicate is 1st Eligible on card 18"
                }
                _ if self.assault_finished() => {
                    "Operation Only recorded. Syndicate is 2nd Eligible"
                }
                _ if self.assault.is_some() => "Government executes Assault",
                _ if self.card_18 => "Government is 1st Eligible on card 18",
                _ if matches!(self.event, Some(EventProgress::ExecuteShaded)) => {
                    "Directorio executes the shaded Event"
                }
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
                ActionChoice::Terror if self.m26_action_ended && self.kidnap.is_some() => {
                    "Terror ended: Operation with Special Activity recorded"
                }
                ActionChoice::Terror if self.m26_action_ended => {
                    "Terror ended: Operation Only recorded"
                }
                ActionChoice::Terror => "executing Terror",
            },
            assault_paid: self.assault_paid(),
            special_activity: self.kidnap.as_ref().map(|_| "Kidnap".into()),
            owed_terror: self.owes_santiago().then(|| "santiago".into()),
            transfer: self.transfer.clone(),
            transfer_terms: self.transfer_terms(),
            kidnap: self.kidnap.clone(),
            kidnap_open: self.kidnap_open,
            removal: self.removal.clone(),
            effected: self.effect,
            recorded_resources: self.recorded_resources(),
            resolved: self
                .terror_resolved
                .iter()
                .map(|space| Resolved {
                    space: space.clone(),
                    paid: TERROR_COST,
                })
                .collect(),
            kidnap_stakes: (self.kidnap.is_none() && !self.m26_action_ended).then(|| self.stakes()),
            space_reasons,
        }
    }
    /// Why each space is in its map state while `choices`, the open decision's, act on
    /// map spaces (#170); empty when none does. Each Operation states its own reasons.
    fn space_reasons(&self, choices: &[Choice]) -> Vec<SpaceReason> {
        if choices.iter().all(|c| c.site.is_none()) {
            return vec![];
        }
        self.spaces
            .iter()
            .filter_map(|space| {
                let here: Vec<&Choice> = choices
                    .iter()
                    .filter(|c| c.space() == Some(space.id.as_str()))
                    .collect();
                let reason = if self.assault.is_some() {
                    self.assault_reason(space, &here)
                } else {
                    self.terror_reason(space, &here)
                };
                reason.map(|reason| SpaceReason {
                    space: space.id.clone(),
                    reason,
                })
            })
            .collect()
    }
    /// Why `space` is in its map state while 26July executes Terror (#170), from the
    /// Terror and Kidnap choices there (`here`): what a resolved Terror paid (R 3.3.4),
    /// that Kidnap owes Terror there (R 4.3.3), what would open it, or why Terror
    /// cannot act there. A space where a supported choice acts and nothing is owed
    /// needs no reason.
    fn terror_reason(&self, space: &Space, here: &[&Choice]) -> Option<String> {
        if self.terror_resolved.contains(&space.id) {
            return Some(format!("Terror here paid {}", resources(TERROR_COST)));
        }
        if self.owes_santiago() && space.id == KIDNAP_SPACE {
            return Some("Terror is owed here (R 4.3.3)".into());
        }
        if here.iter().any(|c| c.supported) {
            return None;
        }
        if let Some(blocked) = here.iter().find(|c| !c.remedies.is_empty()) {
            return Some(remedy_line(&blocked.remedies));
        }
        let underground = space.pieces.iter().any(|p| {
            p.faction == Faction::M26
                && p.kind == Kind::Guerrilla
                && p.state == State::Underground
                && p.count > 0
        });
        if !underground {
            return Some("No Underground 26July Guerrilla".into());
        }
        if here.is_empty() {
            return None;
        }
        // Refused with no remedy though an Underground Guerrilla is there: nothing in
        // this task can pay for it.
        let cost = resources(TERROR_COST);
        Some(
            if self.owes_santiago() && self.resources[Faction::M26.index()] > 0 {
                format!("{cost} is kept for Santiago")
            } else {
                format!("26July cannot pay {cost}")
            },
        )
    }
    /// The balance change from the Pass recorded on the current card.
    fn recorded_resources(&self) -> Vec<ResourceChange> {
        if let Some(from) = self.government_pass_from {
            return vec![ResourceChange {
                faction: Faction::Government,
                from,
                to: self.resources[Faction::Government.index()],
            }];
        }
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
    /// The next faction's role and options after the acting faction records a
    /// decision (R 2.3.3–2.3.4).
    fn next(&self) -> Option<Next> {
        if self.government_pass_from.is_some() {
            return Some(Next {
                faction: self.eligibility().first,
                role: "1st Eligible".into(),
                options: Self::first_eligible_options(),
            });
        }
        if self.card_18 {
            return self.assault_next();
        }
        if self.event.is_some() {
            return None;
        }
        let e = self.eligibility();
        e.m26_box?;
        let (faction, role) = if e.first != Faction::M26 {
            (e.first, "1st Eligible")
        } else {
            (e.second?, "2nd Eligible")
        };
        Some(Next {
            faction,
            role: role.into(),
            options: self.next_options(),
        })
    }
    fn first_eligible_options() -> Vec<String> {
        vec![
            "Pass (+1 if executed later)".into(),
            "Operation (with or without a Special Activity)".into(),
            "Event".into(),
        ]
    }
    /// The next faction's options once 26July's action is recorded (R 2.3.3–2.3.4).
    fn next_options(&self) -> Vec<String> {
        if self.action == ActionChoice::Pass {
            // Directorio, now 1st Eligible, has every 1st Eligible option.
            Self::first_eligible_options()
        } else if self.m26_action_ended {
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
    /// Submits `action` as the faction the game offers it to, at the current match
    /// revision, through the checked submission play uses. A save replays its
    /// decisions this way (`crate::save`).
    pub(crate) fn apply(&mut self, action: &str) -> Result<(), String> {
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
        let steps: &[&str] = match action {
            "kidnap" | "kidnap:roll" => return Some(self.kidnap_stakes(action)),
            "event:shaded" => &[action, "resolve-shaded"],
            "pass" | "government:pass" | "end-terror" | "resolve-shaded" => &[action],
            _ if action.starts_with("terror:")
                || (action.starts_with("rally:")
                    && matches!(self.event, Some(EventProgress::Rally(_))))
                || Self::assault_payment_space(action).is_some()
                || action == "end-assault"
                || choice.removal().is_some() =>
            {
                &[action]
            }
            _ => return None,
        };
        let mut after = self.clone();
        for step in steps {
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
        if matches!(action, "government:pass" | "end-assault") {
            preview.records = after.eligibility().government_box;
            preview.next = after.next();
        } else if after.m26_action_ended && !action.starts_with("remove:") {
            if matches!(
                action,
                "end-terror" | "pass" | "event:shaded" | "resolve-shaded"
            ) {
                let e = after.eligibility();
                preview.records = e.m26_box;
                preview.m26_next_card = e.m26_next_card;
                preview.next = after.next();
            }
        } else if self.card_18 && Self::assault_payment_space(action).is_some() {
            // Paying for a space takes that space's other plans with it (R 3.1), and a
            // balance short of the cost closes every other space's. Listing each plan
            // would bury what the payment does, so the preview's notes state the
            // shortfall once instead.
        } else {
            // Compare the Operation menus, so removal mode's own menu does not count:
            // a removal leaves the mode open, but what it changes is what the
            // Operation offers after Done.
            let mut menu = self.clone();
            menu.removal = None;
            let mut later = after.clone();
            if later.removal.is_some() {
                later.apply("done-removal").ok();
            }
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
                        label: self.unavailable_label(choice),
                        reason: now
                            .map_or_else(|| "No longer offered.".into(), |c| c.explanation.clone()),
                    });
                }
            }
        }
        preview.assault = self.assault_preview(after, action);
        preview.notes = self.notes(after, action);
        preview
    }
    /// How a preview names a choice it would leave unavailable. An Assault plan's
    /// label names only whom it targets, so its space is added.
    fn unavailable_label(&self, choice: &Choice) -> String {
        let space = choice
            .site
            .as_ref()
            .filter(|_| Self::assault_payment_space(choice.action()).is_some())
            .and_then(|site| self.spaces.iter().find(|s| s.id == site.space));
        match space {
            Some(space) => format!("Assault in {} · {}", space.name, choice.label),
            None => choice.label.clone(),
        }
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
        } else if matches!(action, "event:shaded" | "resolve-shaded") {
            notes.push(CARD_17_SHADED.into());
            notes.push("The Event costs 0 Resources. After eligibility is adjusted (R 2.3.7), card 18 stays in preview.".into());
        } else if action.starts_with("rally:") {
            notes.push("The free Rally adds no eligibility penalty (R 5.5).".into());
        } else if action.starts_with("terror:") {
            notes.push(format!("R 3.3.4: pay 1 Resource, {TERROR_EFFECTS}."));
        } else if self.card_18 {
            match action {
                "government:pass" => notes.push("Government stays Eligible for the next card. Syndicate now decides as 1st Eligible (R 2.3.3).".into()),
                "end-assault" => notes.push("Records Operation Only. Syndicate may Pass or choose a Limited Operation as 2nd Eligible (R 2.3.4–5).".into()),
                _ if action.starts_with("remove:") => notes.push("R 1.4.1: after Done, Assault uses the remaining cubes.".into()),
                _ if Self::assault_payment_space(action).is_some() => {
                    notes.extend(Self::assault_preview_notes(after));
                }
                _ => {}
            }
        } else if action.ends_with(":1") {
            notes.push("R 1.4.1: the piece returns to Available. Earlier Terror markers, support shifts and Kidnap results stay.".into());
        } else {
            notes.push("R 1.4.1: the pieces return to Available. Earlier Terror markers, support shifts and Kidnap results stay.".into());
        }
        if !self.effect && after.effect && !after.m26_action_ended {
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
        if !self.effect {
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
    pub(crate) fn executing(&self) -> Option<Faction> {
        if self.card_18 {
            return self
                .assault
                .as_ref()
                .is_some_and(|op| !op.finished)
                .then_some(Faction::Government);
        }
        match &self.event {
            Some(EventProgress::ExecuteShaded | EventProgress::ChooseRallyBeneficiary) => {
                Some(Faction::Directorio)
            }
            Some(EventProgress::Rally(rally)) => Some(rally.beneficiary),
            _ => (self.action == ActionChoice::Terror && !self.m26_action_ended)
                .then_some(Faction::M26),
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
        let eligible = next_card.map_or([true; 4], |results| results.map(|r| r.eligible));
        if self.card_18 {
            // Pact of Caracas has the same printed order as Alberto Bayo
            // (P p. 33). Skip the factions made Ineligible by R 2.3.7.
            let government_passed = self.government_pass_from.is_some();
            let government_box = if government_passed {
                Some(SequenceBox::Pass)
            } else {
                self.assault_finished()
                    .then_some(SequenceBox::OperationOnly)
            };
            let mut in_order = CARD_17_ORDER.into_iter().filter(|&f| {
                eligible[f.index()] && !(f == Faction::Government && government_passed)
            });
            return Eligibility {
                order: CARD_17_ORDER,
                eligible,
                first: in_order.next().expect("Syndicate remains Eligible"),
                second: in_order.next(),
                m26_box: None,
                m26_next_card: None,
                government_box,
                next_card: None,
                event_restriction,
                event_started: false,
            };
        }
        Eligibility {
            order: CARD_17_ORDER,
            eligible,
            next_card,
            event_restriction,
            event_started: self.event.is_some(),
            first: if passed {
                Faction::Directorio
            } else {
                Faction::M26
            },
            second: Some(if passed {
                Faction::Government
            } else {
                Faction::Directorio
            }),
            government_box: None,
            m26_box: match self.action {
                ActionChoice::Pass => Some(SequenceBox::Pass),
                ActionChoice::Terror if self.m26_action_ended && self.kidnap.is_some() => {
                    Some(SequenceBox::OperationWithSpecialActivity)
                }
                ActionChoice::Terror if self.m26_action_ended => Some(SequenceBox::OperationOnly),
                _ => None,
            },
            m26_next_card: match self.action {
                ActionChoice::Pass => Some(true),
                ActionChoice::Terror if self.m26_action_ended => Some(false),
                _ => None,
            },
        }
    }
    /// Local policy (#58): back to the action choice until the first effect, from the
    /// Operation menu or while a Transfer request awaits another faction.
    fn may_go_back(&self) -> bool {
        !self.m26_action_ended
            && self.executing().is_some()
            && self.removal.is_none()
            && !self.kidnap_open
            && !self.effect
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
        } else if self.assault_finished() {
            Faction::Syndicate
        } else if self.card_18 {
            self.eligibility().first
        } else if let Some(EventProgress::Rally(rally)) = &self.event {
            rally.beneficiary
        } else if self.m26_action_ended {
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
            site: None,
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
                &format!("26July's opening decision (R 2.3.4, 5.2). Card 17, Vet trains guerrillas: {CARD_17_UNSHADED}"),
            ),
            absent(
                "event:shaded",
                "Event (shaded)",
                &format!("26July's opening decision (R 2.3.4, 5.2). Card 17, Mexico blocks training by Cuban expat: {CARD_17_SHADED}"),
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
        if self.card_18 {
            if self.government_pass_from.is_some() {
                return self.syndicate_choices();
            }
            return if self.assault.is_some() {
                self.assault_choices()
            } else {
                self.government_choices()
            };
        }
        if self.m26_action_ended && self.executing().is_none() {
            return if self.eligibility().directorio_handoff() {
                self.directorio_choices()
            } else if self
                .event
                .as_ref()
                .and_then(EventProgress::outcome)
                .is_some()
            {
                vec![self.offer(
                    "advance-card",
                    "Advance to Pact of Caracas",
                    true,
                    "Make Pact of Caracas current and reveal the draw deck's top card as the new preview (R 2.3.8).",
                )]
            } else {
                vec![]
            };
        }
        if self.action == ActionChoice::Open {
            return self.opening_choices();
        }
        if self.kidnap_open {
            let mut roll = self.kidnap_offer();
            roll.id = format!("{}:kidnap:roll", self.revision);
            roll.label = "Roll the Kidnap die · commits the result".into();
            // The roll is taken in the panel, where its stakes are: while Kidnap is
            // open no choice acts on a map space, so the map targets nothing.
            roll.site = None;
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
        if self.removal.is_some() {
            return self.removal_menu();
        }
        if let Some(choices) = self.transfer_choices() {
            return choices;
        }
        let mut choices = match &self.event {
            Some(EventProgress::ExecuteShaded) => vec![self.offer(
                "resolve-shaded",
                "Resolve shaded Event",
                true,
                "Activate the remaining Underground 26July Guerrillas and record 26July's restriction through card 18 (R 5.1).",
            )],
            Some(EventProgress::ChooseRallyBeneficiary) => self.rally_beneficiary_choices(),
            Some(EventProgress::Rally(rally)) => self.rally_choices(rally),
            _ => self.terror_choices(),
        };
        let executor = self.executing().unwrap();
        let side_choices = [
            self.offer(
                "transfer",
                "Resource transfer",
                true,
                format!("Give {}'s Resources to another faction, or request Resources that only their giver may authorize (R 1.5.1).", executor.name()),
            ),
            self.offer(
                "remove",
                "Remove own pieces",
                true,
                if executor == Faction::M26 && !self.m26_action_ended {
                    "Voluntary removal (R 1.4.1): removal mode marks each 26July Guerrilla and Base on the map; remove any quantity of a stack, one decision at a time, until Done. Only an already owed Terror restricts removal.".to_owned()
                } else {
                    format!("Voluntary removal (R 1.4.1): choose from {}'s Guerrillas and Bases on the map.", executor.name())
                },
            ),
        ];
        let at = if self.m26_action_ended {
            choices.len()
        } else {
            1
        };
        choices.splice(at..at, side_choices);
        choices
    }

    /// The Government balance after Pass income (R 1.8, 2.3.3).
    fn government_pass_balance(balance: u8) -> u8 {
        balance
            .saturating_add(GOVERNMENT_PASS_INCOME)
            .min(RESOURCE_CAP)
    }

    /// The options of the 1st Eligible Government on card 18 (R 2.3.3–4).
    fn government_choices(&self) -> Vec<Choice> {
        let balance = self.resources[Faction::Government.index()];
        let after_pass = Self::government_pass_balance(balance);
        let alliance = ALLIANCE;
        let cost = alliance.coin_cost();
        let offer = |id: &str, label: &str, rule: String| {
            self.offer(
                &format!("government:{id}"),
                label,
                false,
                format!("{rule} This build offers inspection only."),
            )
        };
        let operation = |id, label, rule, per_space| {
            let charge = if per_space { "per space" } else { "total" };
            let refusal = if balance < cost {
                format!(" Government has {} and cannot pay.", resources(balance))
            } else {
                String::new()
            };
            offer(
                id,
                label,
                format!(
                    "{rule} {} US Alliance: {} {charge} (R 6.3.1).{refusal}",
                    alliance.name(),
                    resources(cost),
                ),
            )
        };
        let activity = |id, label, rule| {
            let balance_status = if balance < cost {
                format!(
                    " Government has {} and cannot pay {} for an Operation.",
                    resources(balance),
                    resources(cost)
                )
            } else {
                String::new()
            };
            offer(
                id,
                label,
                format!(
                    "Requires an Operation in at least one space (R 4.1). {rule}{balance_status}"
                ),
            )
        };
        vec![
            self.offer(
                "government:pass",
                format!("Pass · +{} ({balance} → {after_pass})", resources(after_pass - balance)),
                true,
                format!(
                    "Government has {}. Pass gains {GOVERNMENT_PASS_INCOME} Resources up to the cap of {RESOURCE_CAP}: {balance} → {after_pass}. Government stays Eligible for the next card, and Syndicate becomes 1st Eligible (R 1.8, 2.3.3).",
                    resources(balance)
                ),
            ),
            offer(
                "event",
                "Event · Pact of Caracas",
                "Government may execute the Event as 1st Eligible (R 2.3.4). Its Capability is inactive until the Event executes (R 5.3).".into(),
            ),
            operation("train", "Train · Operation", "R 3.2.1.", true),
            operation("garrison", "Garrison · Operation", "R 3.2.2.", false),
            operation("sweep", "Sweep · Operation", "R 3.2.3.", true),
            self.assault_opening(),
            activity(
                "transport",
                "Transport · Special Activity",
                "Transport may accompany any Government Operation (R 4.2.1).",
            ),
            activity(
                "air-strike",
                "Air Strike · Special Activity",
                "Accompanies Garrison, Sweep or Assault. Air Strike occurs in a Province or economic center and requires an Alliance above Embargoed (R 4.2.2).",
            ),
            activity(
                "reprisal",
                "Reprisal · Special Activity",
                "Accompanies Garrison, Sweep or Assault. Reprisal occurs in one Government-controlled space (R 4.2.3).",
            ),
        ]
    }

    /// Syndicate's 1st Eligible options after Government's Pass (R 2.3.3–4).
    fn syndicate_choices(&self) -> Vec<Choice> {
        let balance = self.resources[Faction::Syndicate.index()];
        let after_pass = balance.saturating_add(1).min(RESOURCE_CAP);
        let terror_refusal = if self.spaces.iter().any(|space| {
            space.pieces.iter().any(|piece| {
                piece.faction == Faction::Syndicate
                    && piece.kind == Kind::Guerrilla
                    && piece.state == State::Underground
            })
        }) {
            ""
        } else {
            " No space has an Underground Syndicate Guerrilla, so Syndicate cannot select a Terror space."
        };
        let offer = |id: &str, label: &str, rule: &str| {
            self.offer(
                &format!("syndicate:{id}"),
                label,
                false,
                format!("{rule} This build offers inspection only."),
            )
        };
        vec![
            offer(
                "pass",
                "Pass",
                &format!(
                    "Syndicate gains 1 Resource up to the cap of {RESOURCE_CAP}: {balance} → {after_pass}. With both Eligible factions Passed, resolve eligibility and advance to the next card (R 1.8, 2.3.3, 2.3.7–8)."
                ),
            ),
            offer(
                "event",
                "Event · Pact of Caracas",
                "Syndicate may execute the Event as 1st Eligible (R 2.3.4). Its Capability is inactive until the Event executes (R 5.3).",
            ),
            offer(
                "rally",
                "Rally · Operation",
                "Pay 1 Resource per City or Province. Place a Syndicate Guerrilla, or flip all its Guerrillas Underground where it has an open Casino (R 3.3.1).",
            ),
            offer(
                "march",
                "March · Operation",
                "Move Syndicate Guerrillas into adjacent spaces. Pay 1 Resource per final City or Province destination, or 0 for an economic center (R 3.3.2).",
            ),
            offer(
                "terror",
                "Terror · Operation",
                &format!(
                    "Activate a Syndicate Underground Guerrilla at each target. Pay 1 Resource per City or Province to place Terror and shift toward Neutral, or 0 per economic center to place Sabotage (R 3.3.4).{terror_refusal}"
                ),
            ),
            offer(
                "construct",
                "Construct · Operation",
                "Pay 5 Resources per Government- or Syndicate-controlled City or Province to open or place a Casino within stacking limits (R 3.3.5).",
            ),
            offer(
                "profit",
                "Profit · Special Activity",
                "With Rally, March or Construct, stash Cash at open Casinos or gain Resources by closing Casinos or removing Syndicate Cash (R 4.5.1).",
            ),
            offer(
                "muscle",
                "Muscle · Special Activity",
                "With Rally, March or Construct, move Police to a City with an open Casino, or Troops to a Province with an open Casino or to an economic center (R 4.5.3).",
            ),
            offer(
                "bribe",
                "Bribe · Special Activity",
                "Pay 3 Resources in one space to remove cubes, remove or flip Guerrillas, or remove an enemy Base (R 4.5.4).",
            ),
        ]
    }

    fn end_terror_choice(&self) -> Choice {
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
    }

    fn back_to_choice(&self, label: &str) -> Choice {
        let mut back = self.offer(
                "back-to-choice",
                label,
                true,
                "Local policy (#58), not a publisher ruling: until the first effect (an authorized transfer, a resolved Terror, a Kidnap roll or a removal), 26July may return to its action choice. A pending Transfer request is withdrawn; nothing has moved.",
            );
        back.owner = Faction::M26;
        back
    }

    fn transfer_choices(&self) -> Option<Vec<Choice>> {
        let executor = self.executing()?;
        match self.transfer {
            // "Give to" moves the executor's own Resources at once;
            // "Ask from" creates a Transfer request that moves nothing (R 1.5.1).
            Some(Transfer::Selecting) => {
                let mut choices = vec![];
                for terms in self.transfer_terms() {
                    let (verb, other) = if terms.giver == executor {
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
                                format!("{}'s own Resource transfer moves at once (R 1.5.1).", executor.name()),
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
                Some(choices)
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
                    format!("Nothing moves, and {} decides again.", executor.name()),
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
                if !self.m26_action_ended && giver != Faction::M26 {
                    if self.may_go_back() {
                        choices
                            .push(self.back_to_choice("26July withdraws · back to action choice"));
                    }
                    let mut end = self.end_terror_choice();
                    end.label = format!("26July withdraws · {}", end.label);
                    end.owner = Faction::M26;
                    // Nothing else is offered until the giver decides, so a remedy
                    // waits on that decision and says so (#58 story 53).
                    for remedy in &mut end.remedies {
                        *remedy = format!("{remedy} once {} has decided", giver.name());
                    }
                    choices.push(end);
                }
                Some(choices)
            }
            None => None,
        }
    }

    fn terror_choices(&self) -> Vec<Choice> {
        let mut choices = vec![self.end_terror_choice()];
        let kidnap = self.kidnap.is_none().then(|| self.kidnap_offer());
        for id in ["santiago", "oriente"] {
            choices.push(self.terror_target(id, kidnap.as_ref()));
        }
        choices.extend(kidnap);
        if self.may_go_back() {
            choices.push(self.back_to_choice("Back to the action choice"));
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
        )
        .at(KIDNAP_SPACE, true);
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
            kidnap.remedies.push(TRANSFER_REMEDY.to_owned());
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
                remedies.push(KIDNAP_REMEDY.to_owned());
            }
            if self.executing().is_some()
                && [Faction::Government, Faction::Directorio, Faction::Syndicate]
                    .iter()
                    .any(|f| self.resources[f.index()] > 0)
            {
                remedies.push(TRANSFER_REMEDY.to_owned());
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
        choice.at(id, true)
    }
    /// Whether the owed-Terror protection holds this group's last piece: the last
    /// Santiago Underground Guerrilla while Santiago Terror is owed (accepted
    /// commitment policy, local).
    fn protects(&self, id: &str, state: State) -> bool {
        self.owes_santiago() && id == "santiago" && state == State::Underground
    }
    /// How many of the executing faction's group may be removed now.
    fn removable(&self, id: &str, kind: Kind, state: State) -> u8 {
        let Some(executor) = self.executing() else {
            return 0;
        };
        self.spaces
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| {
                s.pieces
                    .iter()
                    .find(|p| p.faction == executor && p.kind == kind && p.state == state)
            })
            .map_or(0, |p| {
                p.count.saturating_sub(u8::from(
                    executor == Faction::M26 && self.protects(id, state),
                ))
            })
    }
    /// Removal mode: every quantity of the executor's groups on the map (R 1.4.1). A
    /// quantity beyond what the owed-Terror protection allows is listed, refused with
    /// its reason.
    fn removal_choices(&self) -> Vec<Choice> {
        let executor = self.executing().unwrap();
        let mut choices = vec![];
        for s in &self.spaces {
            for p in s.pieces.iter().filter(|p| p.faction == executor) {
                let group = removal_group(p.kind, p.state);
                let limit = self.removable(&s.id, p.kind, p.state);
                let (one, many) = match (p.kind, p.state) {
                    (Kind::Troops, _) => ("Troop", "Troops"),
                    (Kind::Police, _) => ("Police cube", "Police cubes"),
                    (Kind::Base, _) => ("Base", "Bases"),
                    (_, State::Active) => ("Active Guerrilla", "Active Guerrillas"),
                    _ => ("Underground Guerrilla", "Underground Guerrillas"),
                };
                for n in 1..=p.count {
                    let pieces = if n == 1 { one } else { many };
                    let explanation = if self.card_18 {
                        format!(
                            "R 1.4.1: return {n} {pieces} in {} to Available Forces.",
                            s.name
                        )
                    } else if n <= limit {
                        format!(
                            "R 1.4.1: return {n} {} {pieces} in {} to Available. Earlier Terror markers, support shifts and Kidnap results stay.",
                            executor.name(),
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

    /// Removal mode.
    fn removal_menu(&self) -> Vec<Choice> {
        let mut choices = self.removal_choices();
        let rally_exhausted = matches!(&self.event, Some(EventProgress::Rally(rally))
            if self.rally_choices(rally).is_empty());
        choices.push(self.offer(
            "done-removal",
            if rally_exhausted {
                "Done · finish free Rally"
            } else {
                "Done · leave removal mode"
            },
            true,
            if rally_exhausted {
                "No legal Rally choices remain. The removed pieces stay in Available and card 18's eligibility is recorded (R 2.3.7, 5.1.3)."
            } else {
                "Returns to execution. The removed pieces stay in Available."
            },
        ));
        choices
    }
}
impl Game {
    pub fn random_state(&self) -> u64 {
        self.random
    }
    /// The opening position with a fixed random state, for tests and for reading
    /// fixed data such as terrain.
    pub fn opening() -> Self {
        Self::opening_with_seed(OPENING_SEED)
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
        self.apply_decision(offered.action(), next_revision)?;
        self.revision = next_revision;
        Ok(())
    }

    fn apply_decision(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        match action {
            // R 2.3.3. Only offered at the action choice.
            "pass" => {
                self.resources[Faction::M26.index()] = self.resources[Faction::M26.index()]
                    .saturating_add(1)
                    .min(RESOURCE_CAP);
                self.action = ActionChoice::Pass;
                self.m26_action_ended = true;
                // Its Resource is the first effect.
                self.effect = true;
            }
            "government:pass" => {
                let balance = &mut self.resources[Faction::Government.index()];
                self.government_pass_from = Some(*balance);
                *balance = Self::government_pass_balance(*balance);
            }
            _ if self.card_18 => return self.apply_assault(action, next_revision),
            "terror" if self.action == ActionChoice::Open => self.action = ActionChoice::Terror,
            // Local policy (#58): withdraws a pending request; nothing has moved.
            "back-to-choice" => {
                self.action = ActionChoice::Open;
                self.transfer = None;
            }
            // Ending closes transfers and withdraws a pending request.
            "end-terror" => {
                self.m26_action_ended = true;
                self.transfer = None;
            }
            "event:shaded" => self.event = Some(EventProgress::ExecuteShaded),
            "resolve-shaded" => self.execute_shaded_event(),
            "advance-card" => {
                if !self.draw_deck.starts_with(&[18, 19]) {
                    return Err("The prepared deck must begin with cards 18 and 19.".into());
                }
                self.draw_deck.remove(0);
                self.card_18 = true;
            }
            "event:unshaded" => self.event = Some(EventProgress::ChooseRallyBeneficiary),
            _ if self.transfer.is_some()
                || self.removal.is_some()
                || matches!(action, "transfer" | "remove") =>
            {
                return self.execute(action, next_revision);
            }
            _ if matches!(self.event, Some(EventProgress::ChooseRallyBeneficiary)) => {
                self.select_rally_beneficiary(action)?;
            }
            _ if let Some(EventProgress::Rally(rally)) = &self.event => {
                self.resolve_rally(rally.clone(), action)?;
            }
            _ => return self.execute(action, next_revision),
        }
        Ok(())
    }
    /// Side decisions during execution, plus the original Terror and Kidnap steps.
    fn execute(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        // Removal mode: only its own choices are offered, so `submit` has already
        // bound the space, group and quantity to a supported removal.
        let executor = self.executing().ok_or("No faction is executing.")?;
        if self.removal.is_some() {
            if action == "done-removal" {
                self.removal = None;
                if let Some(EventProgress::Rally(rally)) = &self.event {
                    self.continue_rally(rally.clone());
                }
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
                    .find(|p| p.faction == executor && p.kind == kind && p.state == state)
                    .ok_or("Unknown choice.")?;
                piece.count = piece.count.checked_sub(n).ok_or("Unknown choice.")?;
                target.pieces.retain(|p| p.count > 0);
                self.effect |= !self.card_18;
                self.available
                    .iter_mut()
                    .find(|p| p.faction == executor && p.kind == kind)
                    .unwrap()
                    .count += n;
            }
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
                self.terror_resolved.push(id.to_owned());
                self.resources[Faction::M26.index()] -= TERROR_COST;
                self.effect = true;
            }
            None if action == "transfer" => self.transfer = Some(Transfer::Selecting),
            None => return Err("Unknown execution choice.".into()),
            Some(_) if action == "back" || action == "decline" => self.transfer = None,
            // The request's giver can set it aside or reopen it without moving
            // Resources.
            Some(Transfer::Authorizing {
                giver,
                recipient,
                amount,
                ..
            }) if matches!(action, "wait" | "reopen") => {
                self.transfer = Some(Transfer::Authorizing {
                    giver,
                    recipient,
                    amount,
                    waited: action == "wait",
                });
            }
            Some(Transfer::Selecting) => {
                let (verb, rest) = action.split_once(':').unwrap();
                let (other, amount) = rest.split_once(':').unwrap();
                let other = Faction::ALL[other.parse::<usize>().unwrap()];
                let amount: u8 = amount.parse().unwrap();
                if verb == "give" {
                    self.move_resources(executor, other, amount);
                    self.transfer = None;
                } else {
                    self.transfer = Some(Transfer::Authorizing {
                        giver: other,
                        recipient: executor,
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
        Ok(())
    }
    /// An authorized Resource transfer, within the limits that offered it.
    fn move_resources(&mut self, giver: Faction, recipient: Faction, amount: u8) {
        self.resources[giver.index()] -= amount;
        self.resources[recipient.index()] += amount;
        // Zero is a deliberate non-transfer, not an effect.
        self.effect |= amount > 0 && !self.card_18;
    }
    /// The terms of the transfers the executing faction may make now (R 1.5.1, 1.8 and the accepted
    /// commitment policy's reserve).
    fn transfer_terms(&self) -> Vec<TransferTerms> {
        let Some(executor) = self.executing() else {
            return vec![];
        };
        let pairs = match self.transfer {
            Some(Transfer::Selecting) => Faction::ALL
                .into_iter()
                .filter(|&other| other != executor)
                .flat_map(|other| [(other, executor), (executor, other)])
                .collect(),
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
    /// Why an amount beyond its limit is refused, in the terms' own words: the executor's Resource
    /// transfer or request the modal offers, or the pending request's authorization.
    fn transfer_refusal(&self, action: &str) -> Option<String> {
        let executor = self.executing()?;
        let terms = self.transfer_terms();
        let parts: Vec<_> = action.split(':').collect();
        let (direction, amount) = match parts[..] {
            ["authorize", n] if matches!(self.transfer, Some(Transfer::Authorizing { .. })) => {
                (terms.first()?, n)
            }
            [verb @ ("give" | "ask"), other, n] => {
                let other = *Faction::ALL.get(other.parse::<usize>().ok()?)?;
                let (giver, recipient) = if verb == "give" {
                    (executor, other)
                } else {
                    (other, executor)
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
}

#[cfg(test)]
mod government_pass_tests {
    use super::{Faction, Game, SequenceBox};

    #[test]
    fn government_pass_applies_income_only_up_to_the_resource_cap() {
        let mut opening = crate::terror_chosen(Game::opening_with_seed(2));
        for action in [
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
        ] {
            opening.apply(action).unwrap();
        }

        // R 2.3.3 grants 3 and R 1.8 caps balances at 49. The prepared match
        // cannot accumulate these high balances, so set them only for this rule test.
        for (from, to) in [(1, 4), (46, 49), (47, 49), (48, 49), (49, 49)] {
            let mut game = opening.clone();
            let government = Faction::Government.index();
            game.resources[government] = from;
            let before = game.observe();
            let pass = before
                .choices
                .iter()
                .find(|choice| choice.action() == "government:pass")
                .unwrap();
            let projected = pass.preview.as_ref().unwrap();
            assert_eq!(projected.records, Some(SequenceBox::Pass));
            if from == to {
                assert!(projected.resources.is_empty());
            } else {
                assert_eq!(projected.resources.len(), 1);
                assert_eq!(
                    (projected.resources[0].from, projected.resources[0].to),
                    (from, to)
                );
            }
            assert!(pass.label.contains(&format!("+{}", to - from)));
            assert_eq!(game.resources[government], from);
            game.apply("government:pass").unwrap();
            let after = game.observe();
            assert_eq!(after.resources[government], to);
            let recorded = &after.recorded_resources[0];
            assert_eq!((recorded.from, recorded.to), (from, to));
            assert_eq!(after.owner, Faction::Syndicate);
            assert!(game.apply("government:pass").is_err());
            assert_eq!(game.observe(), after);
        }
    }
}
#[cfg(test)]
mod inventory_tests {
    use super::{Faction, Game, Group, Kind, State, inventory_breaches};

    /// 2 more 26July Guerrillas on the map, with or without leaving Available.
    fn with_two_more_guerrillas(leave_available: bool) -> Game {
        let mut game = Game::opening();
        let space = game
            .spaces
            .iter_mut()
            .find(|space| space.id == "sierra-maestra")
            .unwrap();
        space.pieces.push(Group {
            faction: Faction::M26,
            kind: Kind::Guerrilla,
            state: State::Active,
            count: 2,
        });
        if leave_available {
            game.available
                .iter_mut()
                .find(|group| group.faction == Faction::M26 && group.kind == Kind::Guerrilla)
                .unwrap()
                .count -= 2;
        }
        game
    }

    /// The opening has 12 of 26July's 15 Guerrillas Available and 3 on the map
    /// (p. 20 and the accepted handoff), and every other kind adds up too.
    #[test]
    fn a_position_that_keeps_the_chart_has_no_breach() {
        let opening = Game::opening();
        assert_eq!(
            inventory_breaches(&opening.available, &opening.spaces),
            Vec::<String>::new()
        );
        let moved = with_two_more_guerrillas(true);
        assert_eq!(
            inventory_breaches(&moved.available, &moved.spaces),
            Vec::<String>::new()
        );
    }

    /// 2 Guerrillas placed without leaving Available make 12 + 5 = 17 of 15.
    #[test]
    fn a_piece_placed_without_leaving_available_is_named() {
        let added = with_two_more_guerrillas(false);
        assert_eq!(
            inventory_breaches(&added.available, &added.spaces),
            ["26July Guerrilla: 12 Available + 5 on the map, of 15"]
        );
    }

    /// A kind the chart does not give a faction is a breach wherever it lies.
    #[test]
    fn a_piece_outside_the_chart_is_named() {
        let mut game = Game::opening();
        game.available.push(Group {
            faction: Faction::Government,
            kind: Kind::Guerrilla,
            state: State::Underground,
            count: 1,
        });
        assert_eq!(
            inventory_breaches(&game.available, &game.spaces),
            ["Government Guerrilla: 1 Available + 0 on the map, of 0"]
        );
    }

    /// A unit test cannot observe a position that breaks the chart.
    #[test]
    #[should_panic(expected = "breaks the Available Forces chart")]
    fn observing_a_position_that_breaks_the_chart_panics_in_unit_tests() {
        with_two_more_guerrillas(false).observe();
    }
}

#[cfg(test)]
mod space_reason_tests {
    use super::{Faction, Game, KIDNAP_REMEDY, TRANSFER_REMEDY, remedy_line};

    fn choose(game: &mut Game, action: &str) {
        let view = game.observe();
        game.submit(
            view.owner,
            view.revision,
            &format!("{}:{action}", view.revision),
        )
        .unwrap_or_else(|e| panic!("{action}: {e}"));
    }

    fn reason(game: &Game, space: &str) -> Option<String> {
        game.observe()
            .space_reasons
            .into_iter()
            .find(|r| r.space == space)
            .map(|r| r.reason)
    }

    /// #170: a blocked space's line names what would open it, from the remedies the
    /// authority gives its choice.
    #[test]
    fn a_blocked_space_s_line_names_its_remedies() {
        let line = |remedies: &[&str]| {
            remedy_line(&remedies.iter().map(|r| r.to_string()).collect::<Vec<_>>())
        };
        assert_eq!(line(&[TRANSFER_REMEDY]), "A Resource transfer opens it");
        assert_eq!(line(&[KIDNAP_REMEDY]), "Kidnap here first opens it");
        assert_eq!(
            line(&[KIDNAP_REMEDY, TRANSFER_REMEDY]),
            "Kidnap or a transfer opens it"
        );
    }

    /// #170: a Terror refused with no remedy while an Underground Guerrilla is there.
    /// No faction can give a Resource, so nothing in this task pays for it (R 3.3.4);
    /// once Kidnap owes Santiago's Terror, 26July's last Resource is kept for it
    /// (accepted commitment policy).
    #[test]
    fn a_terror_nothing_can_pay_for_says_why() {
        let mut game = Game::opening_with_seed(2);
        choose(&mut game, "terror");
        game.resources = [0; 4];
        // Kidnap takes nothing from Government at 0, so it is not offered either.
        assert_eq!(
            reason(&game, "santiago").as_deref(),
            Some("26July cannot pay 1 Resource")
        );
        assert_eq!(
            reason(&game, "oriente").as_deref(),
            Some("26July cannot pay 1 Resource")
        );

        let mut owed = Game::opening_with_seed(2);
        choose(&mut owed, "terror");
        choose(&mut owed, "kidnap");
        choose(&mut owed, "kidnap:roll");
        owed.resources = [0; 4];
        owed.resources[Faction::M26.index()] = 1;
        assert_eq!(
            reason(&owed, "santiago").as_deref(),
            Some("Terror is owed here (R 4.3.3)")
        );
        assert_eq!(
            reason(&owed, "oriente").as_deref(),
            Some("1 Resource is kept for Santiago")
        );
    }

    /// An Assault space whose plans would remove or close a piece, with none supported
    /// and no remedy, is not involved, as a Terror with no remedy is: veiled, nothing
    /// opens on a click and Tab passes it, and its line keeps the authority's reason.
    /// A space whose only plan removes nothing stays no effect though Government cannot
    /// pay for it (R 3.2.4, 6.3.1).
    #[test]
    fn an_assault_space_nobody_can_fund_is_not_involved_and_keeps_its_reason() {
        use crate::targets::{SpaceState, focus_order, space_targets};
        let mut game = Game::opening_with_seed(2);
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
        ] {
            choose(&mut game, action);
        }
        // Government has 1 Resource and a space costs 2 at a Firm US Alliance; no other
        // faction has a Resource to give, so no transfer can open a space.
        game.resources = [0; 4];
        game.resources[Faction::Government.index()] = 1;
        let view = game.observe();
        let targets = space_targets(&view).expect("an open Assault targets the map");
        let state = |space: &str| targets.iter().find(|t| t.space == space).unwrap().state;
        // Havana's open Casino would close and Santiago's 2 Active Guerrillas would
        // return, but nothing can pay for either.
        for space in ["havana", "santiago"] {
            let effective: Vec<_> = view
                .choices
                .iter()
                .filter(|c| c.space() == Some(space) && c.site.as_ref().unwrap().effect)
                .collect();
            assert!(!effective.is_empty(), "{space}");
            assert!(
                effective
                    .iter()
                    .all(|c| !c.supported && c.remedies.is_empty()),
                "{space}"
            );
            assert_eq!(state(space), SpaceState::NotInvolved, "{space}");
            assert!(!state(space).choosable(), "{space}");
            assert_eq!(
                reason(&game, space).as_deref(),
                Some("Government cannot pay 2 Resources"),
                "{space}"
            );
        }
        // The other 11 spaces' only plan removes nothing: still no effect.
        for target in targets
            .iter()
            .filter(|t| !["havana", "santiago"].contains(&t.space.as_str()))
        {
            assert_eq!(target.state, SpaceState::NoEffect, "{}", target.space);
        }
        assert_eq!(
            reason(&game, "matanzas").as_deref(),
            Some("No enemy pieces (R 3.2.4)")
        );
        let order = focus_order(&targets);
        assert_eq!(order.len(), 11);
        assert!(!order.contains(&"havana") && !order.contains(&"santiago"));
    }

    /// The same state reached by legal decisions alone (spec #166's review, D1).
    /// After Kidnap, Santiago's Terror and the shaded Event, the four factions hold
    /// 25 Resources: 26 at setup, less the Terror's 1 (R 3.3.4). Government, executing
    /// the Assault, asks each other faction for its whole balance and each gives it
    /// (R 1.5.1): Government holds 25, the others 0. Twelve spaces paid at 2 each
    /// (R 6.3.1) leave 1. Havana is left unpaid, its open Casino still a target, and
    /// nothing can pay for it: no faction has a Resource to give.
    #[test]
    fn legal_play_reaches_an_assault_space_nobody_can_fund() {
        use crate::targets::{SpaceState, focus_order, space_targets};
        let mut game = Game::opening_with_seed(2);
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
        ] {
            choose(&mut game, action);
        }
        assert_eq!(game.observe().resources, [1, 4, 5, 15]);
        for (giver, amount) in [(1, 4), (2, 5), (3, 15)] {
            choose(&mut game, "transfer");
            choose(&mut game, &format!("ask:{giver}:{amount}"));
            choose(&mut game, &format!("authorize:{amount}"));
        }
        assert_eq!(game.observe().resources, [25, 0, 0, 0]);
        let unpaid: Vec<String> = game
            .observe()
            .spaces
            .iter()
            .map(|space| space.id.clone())
            .filter(|id| id != "havana")
            .collect();
        for space in &unpaid {
            choose(&mut game, &format!("assault:{space}:none"));
        }
        let view = game.observe();
        assert_eq!(view.resources, [1, 0, 0, 0]);
        assert_eq!(view.assault_paid.len(), 12);
        let targets = space_targets(&view).expect("the Assault is still open");
        let havana = targets.iter().find(|t| t.space == "havana").unwrap();
        // Havana's plans would close the Casino, but none is supported and none names
        // a remedy.
        let plans: Vec<_> = view
            .choices
            .iter()
            .filter(|c| c.space() == Some("havana"))
            .collect();
        assert!(plans.iter().any(|c| c.site.as_ref().unwrap().effect));
        assert!(plans.iter().all(|c| !c.supported && c.remedies.is_empty()));
        assert_eq!(havana.state, SpaceState::NotInvolved);
        assert!(focus_order(&targets).is_empty());
        assert_eq!(
            reason(&game, "havana").as_deref(),
            Some("Government cannot pay 2 Resources")
        );
    }
}
