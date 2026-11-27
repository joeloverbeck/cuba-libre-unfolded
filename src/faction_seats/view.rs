//! What a faction may know, as its session is told it. Every field is named here, so
//! nothing reaches a session because the rules authority came to observe it: the
//! generator's state, the one a rolled Kidnap records among it, and the order of the
//! face-down cards are not among them. The view is whole every time.

use super::Player;
use crate::{
    ActionChoice, CardFacts, Choice, ConstructResolution, DeckFacts, Eligibility, EventProgress,
    EventStatement, Faction, Game, Group, KidnapStakes, Manifesto, MarkerSupply, Next, Observation,
    Piles, PlayedCard, Preview, Removal, Site, SpaceReason, Standing, Terrain, Transfer,
    TransferTerms, UsAlliance, reports::Report, save::Proclamation, support_level,
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
    /// What the factions' actions did, apart from the state: a report of each action of
    /// the match so far, in order. The last may be in progress.
    pub results: Vec<Report>,
    pub proclamations: PublicProclamations,
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

/// The public words of each faction and whether this faction may publish now.
#[derive(Debug, Serialize)]
pub struct PublicProclamations {
    pub note: &'static str,
    pub limit: usize,
    pub can_publish: bool,
    pub published: Vec<Proclamation>,
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
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::game::manifesto::serialize_observation"
    )]
    pub manifesto: Option<Manifesto>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub construct_funding: Option<crate::ConstructFunding>,
    /// Indexed by a faction's number.
    pub resources: [u8; 4],
    pub standings: Vec<Standing>,
    pub spaces: Vec<Space>,
    /// Why each space is in its state, while a decision acts on spaces.
    pub space_reasons: Vec<SpaceReason>,
    pub construct_paid: Vec<ConstructResolution>,
    pub available: Vec<Group>,
    pub current_card: Card,
    pub preview_card: Card,
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

/// A face-up card: what its tag shows, and what each text of its Event does.
#[derive(Debug, Serialize)]
pub struct Card {
    #[serde(flatten)]
    pub facts: CardFacts,
    pub texts: Vec<EventStatement>,
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
    /// What it would do, as the authority computes it; for an Event's text it does not
    /// compute, what the text does.
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
    /// What `faction` may know of `seen`, which the actions of `reports` led to, with
    /// `players` playing the factions.
    pub fn of(
        seen: &Observation,
        reports: Vec<Report>,
        published: &[Proclamation],
        faction: Faction,
        players: [Player; 4],
    ) -> Self {
        let choices: Vec<Offer> = seen
            .choices
            .iter()
            .filter(|choice| choice.owner == faction && !choice.turns_the_card())
            .map(|choice| Offer::of(choice, seen.cards.current.number))
            .collect();
        let decision = Decision::of(seen, faction, &choices);
        let proclamations = PublicProclamations {
            note: "Public faction speech (R 1.5.1). Agreements are nonbinding.",
            limit: Proclamation::LIMIT,
            can_publish: decision.stands == Stands::Open
                && !Proclamation::spent(published, seen.current_card, faction),
            published: published.to_vec(),
        };
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
            decision,
            state: State::of(seen),
            results: reports,
            proclamations,
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
        let mut note = match stands {
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
        if (seen.removal.is_some() || seen.transfer.is_some())
            && let Some(placement) = seen.manifesto.as_ref().and_then(Manifesto::pending)
        {
            note.push_str(&format!(
                " The Manifesto returns to {}'s placement decision when this closes.",
                placement.name()
            ));
        }
        if let Some(funding) = &seen.construct_funding {
            note.push_str(&format!(
                " {}. {}",
                funding.summary(),
                crate::ConstructFunding::CONDITION
            ));
        }
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
            manifesto: seen.manifesto.clone(),
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
            construct_funding: seen.construct_funding.clone(),
            resources: seen.resources,
            standings: seen.standings.clone(),
            spaces: seen.spaces.iter().map(Space::of).collect(),
            space_reasons: seen.space_reasons.clone(),
            construct_paid: seen.construct_paid.clone(),
            available: seen.available.clone(),
            current_card: Card::of(&seen.cards.current),
            preview_card: Card::of(&seen.cards.preview),
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

impl Card {
    fn of(facts: &CardFacts) -> Self {
        Self {
            facts: facts.clone(),
            texts: EventStatement::of(facts.number),
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

impl Offer {
    fn of(choice: &Choice, card: u8) -> Self {
        let event = |text| {
            let told = EventStatement::of(card).into_iter();
            let notes = told
                .filter(|told| told.text == text)
                .map(|told| told.statement.to_owned());
            Preview {
                notes: notes.collect(),
                ..Preview::default()
            }
        };
        Self {
            id: choice.id.clone(),
            label: choice.label.clone(),
            supported: choice.supported,
            explanation: choice.explanation.clone(),
            accompanies: choice.accompanies.clone(),
            remedies: choice.remedies.clone(),
            site: choice.site.clone(),
            preview: choice
                .preview
                .clone()
                .or_else(|| choice.event_text().map(event)),
        }
    }
}
