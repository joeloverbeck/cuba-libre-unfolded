//! What a faction may know, as its session is told it. Every field is named here, so
//! nothing reaches a session because the rules authority came to observe it: the
//! generator's state, the one a rolled Kidnap records among it, and the order of the
//! face-down cards are not among them. The view is whole every time.

use super::Player;
use crate::{
    ActionChoice, AssaultResolution, CardFacts, Choice, DeckFacts, Eligibility, EventOutcome,
    EventProgress, Faction, Game, Group, Kidnap, KidnapStakes, MarkerSupply, Next, Observation,
    Piles, PlayedCard, Preview, Removal, Resolved, ResourceChange, Site, SpaceReason, Standing,
    Terrain, Transfer, TransferTerms, UsAlliance, support_level,
};
use serde::Serialize;

/// The rules that govern the match, and where they can be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Rules {
    pub edition: &'static str,
    pub read_at: &'static str,
}

/// The accepted source (`docs/research/rules-baseline-and-candidates.md`).
pub(super) const RULES: Rules = Rules {
    edition: "GMT Cuba Libre, Living Rules of March 2019 (third printing). \"R 3.3.4\" names its rule 3.3.4",
    read_at: "https://s3-us-west-2.amazonaws.com/gmtwebsiteassets/cuba/CL-RULES-2019-LivingRules.pdf",
};

/// Everything one faction's session is told of the match.
#[derive(Debug, Serialize)]
pub struct View {
    /// The faction this view is for, as choices and states name it.
    pub faction: Faction,
    pub name: &'static str,
    /// A choice is made at this revision.
    pub revision: u64,
    pub rules: Rules,
    /// Each faction, its number in choice ids and who plays it.
    pub factions: Vec<Seat>,
    pub decision: Decision,
    pub state: State,
    /// What the factions' actions did, apart from the state.
    pub results: Results,
    /// Every choice the rules offer this faction now, refused ones with the reason.
    pub choices: Vec<Offer>,
}

#[derive(Debug, Serialize)]
pub struct Seat {
    pub faction: Faction,
    pub name: &'static str,
    /// Its number in choice ids, such as `ask:<number>:2`.
    pub number: usize,
    pub played_by: Player,
}

/// Where the match stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stands {
    /// A faction decides.
    Open,
    /// Only the card is to be turned, which the human does.
    Card,
    /// The deciding faction has no choice this build plays: the match stops here.
    BuildEnds,
}

#[derive(Debug, Serialize)]
pub struct Decision {
    /// The deciding faction.
    pub faction: Faction,
    pub stands: Stands,
    /// Whether this view's faction decides now.
    pub yours: bool,
    pub note: String,
}

/// The position, as the rules make it public.
#[derive(Debug, Serialize)]
pub struct State {
    pub operation: &'static str,
    /// 26July's action on card 17.
    pub action: ActionChoice,
    /// Whether 26July's action on card 17 is recorded.
    pub m26_action_recorded: bool,
    pub executing: Option<Faction>,
    pub event_executor: Option<Faction>,
    pub event: Option<EventProgress>,
    pub eligibility: Eligibility,
    pub passed: [bool; 4],
    pub next: Option<Next>,
    /// Whether the open action has had a real effect (local policy, #58).
    pub effected: bool,
    pub special_activity: Option<String>,
    /// The space where a rolled Kidnap still owes Terror.
    pub owed_terror: Option<String>,
    pub kidnap_open: bool,
    pub kidnap_stakes: Option<KidnapStakes>,
    pub removal: Option<Removal>,
    pub transfer: Option<Transfer>,
    pub transfer_terms: Vec<TransferTerms>,
    /// Indexed by a faction's number.
    pub resources: [u8; 4],
    pub standings: Vec<Standing>,
    pub spaces: Vec<Space>,
    /// Why each space is in its state, while a decision acts on spaces.
    pub space_reasons: Vec<SpaceReason>,
    pub available: Vec<Group>,
    pub current_card: CardFacts,
    pub preview_card: CardFacts,
    pub piles: Piles,
    /// Every card played, most recent first.
    pub played: Vec<PlayedCard>,
    /// What is public of the draw deck; the order of its cards is not.
    pub deck: DeckFacts,
    pub aid: u8,
    pub alliance: UsAlliance,
    pub cash_available: u8,
    pub marker_supply: MarkerSupply,
    pub capabilities: Vec<String>,
    pub momentum: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub terrain: Terrain,
    pub population: u8,
    pub economy: u8,
    pub support: i8,
    /// Its name; an Economic Center has none.
    pub support_level: Option<&'static str>,
    pub control: Option<Faction>,
    pub pieces: Vec<Group>,
    pub terror: u8,
}

/// What the actions did, as the rules authority records it: the Pass, each space a
/// Terror or an Assault resolved, the Kidnap roll and the Event.
#[derive(Debug, Serialize)]
pub struct Results {
    pub passes: Vec<ResourceChange>,
    pub terror: Vec<Resolved>,
    pub kidnap: Option<KidnapResult>,
    pub event: Option<EventOutcome>,
    pub assault: Vec<AssaultResolution>,
}

/// A committed Kidnap roll, without the generator's state it was rolled from.
#[derive(Debug, Serialize)]
pub struct KidnapResult {
    pub die: u8,
    pub proceeds: u8,
    pub government_before: u8,
    pub m26_before: u8,
    /// The match revision the roll created.
    pub revision: u64,
}

#[derive(Debug, Serialize)]
pub struct Offer {
    pub id: String,
    pub label: String,
    /// Whether it can be chosen now.
    pub supported: bool,
    pub explanation: String,
    /// The Operation a Special Activity accompanies.
    pub accompanies: Option<String>,
    /// What would make a refused choice supported.
    pub remedies: Vec<String>,
    pub site: Option<Site>,
    pub preview: Option<Preview>,
}

impl Stands {
    /// Where `game` stands, whoever plays the deciding faction.
    pub fn of(game: &Game) -> Self {
        Self::among(&game.choices())
    }

    fn among(choices: &[Choice]) -> Self {
        let mut supported = choices.iter().filter(|choice| choice.supported).peekable();
        if supported.peek().is_none() {
            Self::BuildEnds
        } else if supported.all(Choice::turns_the_card) {
            Self::Card
        } else {
            Self::Open
        }
    }
}

impl View {
    /// What `faction` may know of `seen`, with `players` playing the factions.
    pub fn of(seen: &Observation, faction: Faction, players: [Player; 4]) -> Self {
        let choices: Vec<Offer> = seen
            .choices
            .iter()
            .filter(|choice| choice.owner == faction && !choice.turns_the_card())
            .map(Offer::of)
            .collect();
        Self {
            faction,
            name: faction.name(),
            revision: seen.revision,
            rules: RULES,
            factions: Faction::ALL
                .into_iter()
                .map(|each| Seat {
                    faction: each,
                    name: each.name(),
                    number: each.index(),
                    played_by: players[each.index()],
                })
                .collect(),
            decision: Decision::of(seen, faction, &choices),
            state: State::of(seen),
            results: Results {
                passes: seen.recorded_resources.clone(),
                terror: seen.resolved.clone(),
                kidnap: seen.kidnap.as_ref().map(KidnapResult::of),
                event: seen
                    .event
                    .as_ref()
                    .and_then(EventProgress::outcome)
                    .cloned(),
                assault: seen.assault_paid.clone(),
            },
            choices,
        }
    }
}

impl Decision {
    fn of(seen: &Observation, faction: Faction, choices: &[Offer]) -> Self {
        let stands = Stands::among(&seen.choices);
        let decider = seen.owner.name();
        let yours = stands == Stands::Open && seen.owner == faction;
        let waited = matches!(
            &seen.transfer,
            Some(Transfer::Authorizing { waited: true, .. })
        );
        let note = match stands {
            Stands::BuildEnds => format!(
                "The build stops here: {decider} would decide next, and this build plays none of its options. No faction can act, and nothing more will come."
            ),
            Stands::Card => {
                "The human turns the card. Until then no faction has a choice to make.".to_owned()
            }
            Stands::Open if yours && waited => {
                "You set this Transfer request aside. It stays yours to answer until it is answered or withdrawn."
                    .to_owned()
            }
            Stands::Open if yours => "You decide: make one of your supported choices.".to_owned(),
            Stands::Open if choices.iter().any(|choice| choice.supported) => format!(
                "{decider} decides. Your choices below stay offered meanwhile, and nothing waits for them."
            ),
            Stands::Open => format!("{decider} decides."),
        };
        Self {
            faction: seen.owner,
            stands,
            yours,
            note,
        }
    }
}

impl State {
    fn of(seen: &Observation) -> Self {
        Self {
            operation: seen.operation,
            action: seen.action,
            m26_action_recorded: seen.finished,
            executing: seen.executing,
            event_executor: seen.event_executor,
            event: seen.event.clone(),
            eligibility: seen.eligibility.clone(),
            passed: seen.passed,
            next: seen.next.clone(),
            effected: seen.effected,
            special_activity: seen.special_activity.clone(),
            owed_terror: seen.owed_terror.clone(),
            kidnap_open: seen.kidnap_open,
            kidnap_stakes: seen.kidnap_stakes.clone(),
            removal: seen.removal.clone(),
            transfer: seen.transfer.clone(),
            transfer_terms: seen.transfer_terms.clone(),
            resources: seen.resources,
            standings: seen.standings.clone(),
            spaces: seen.spaces.iter().map(Space::of).collect(),
            space_reasons: seen.space_reasons.clone(),
            available: seen.available.clone(),
            current_card: seen.cards.current.clone(),
            preview_card: seen.cards.preview.clone(),
            piles: seen.piles,
            played: seen.played_pile.cards.clone(),
            deck: seen.deck.clone(),
            aid: seen.aid,
            alliance: seen.alliance,
            cash_available: seen.cash_available,
            marker_supply: seen.marker_supply,
            capabilities: seen.capabilities.clone(),
            momentum: seen.momentum.clone(),
        }
    }
}

impl Space {
    fn of(space: &crate::Space) -> Self {
        Self {
            id: space.id.clone(),
            name: space.name.clone(),
            terrain: space.terrain,
            population: space.population,
            economy: space.economy,
            support: space.support,
            support_level: (space.terrain != Terrain::EconomicCenter)
                .then(|| support_level(space.support)),
            control: space.control(),
            pieces: space.pieces.clone(),
            terror: space.terror,
        }
    }
}

impl KidnapResult {
    fn of(kidnap: &Kidnap) -> Self {
        Self {
            die: kidnap.die,
            proceeds: kidnap.proceeds,
            government_before: kidnap.government_before,
            m26_before: kidnap.m26_before,
            revision: kidnap.revision,
        }
    }
}

impl Offer {
    fn of(choice: &Choice) -> Self {
        Self {
            id: choice.id.clone(),
            label: choice.label.clone(),
            supported: choice.supported,
            explanation: choice.explanation.clone(),
            accompanies: choice.accompanies.clone(),
            remedies: choice.remedies.clone(),
            site: choice.site.clone(),
            preview: choice.preview.clone(),
        }
    }
}
