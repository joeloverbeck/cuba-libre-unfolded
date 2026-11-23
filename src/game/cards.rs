//! What can be read about the two cards showing (R 2.2, 2.3, 5.0–5.4): the facts
//! printed on each face, and where its Event stands in the current play. The card tag
//! and the development seat only read these.
//!
//! The faces admitted in this build are cards 17, 18 and 19. Their titles, faction
//! orders, Event layouts and lasting-effect boxes are transcribed from the supplied
//! faces. #144 extends the titles to all 52 cards.
use super::{ActionChoice, CARD_17_ORDER, EventOutcome, EventProgress, Faction, Game, SequenceBox};
use serde::Serialize;

/// What the Event's text offers its executing faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EventKind {
    /// An unshaded and a shaded text; the executing faction chooses one (R 5.2).
    DualUse,
    /// One text, carried out as written (R 5.1).
    Single,
}
impl EventKind {
    pub fn text(self) -> &'static str {
        match self {
            Self::DualUse => "Dual-use: the executing faction chooses unshaded or shaded",
            Self::Single => "Single text: the executing faction carries it out",
        }
    }
    fn rule(self) -> &'static str {
        match self {
            Self::DualUse => "R 5.2",
            Self::Single => "R 5.1",
        }
    }
}

/// A box on the face that makes the Event last beyond its execution (R 5.3, 5.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum LastingEffect {
    InsurgentCapability { in_effect: bool },
    GovernmentMomentum { in_effect: bool },
}
impl LastingEffect {
    fn text(self) -> String {
        let (name, in_effect) = match self {
            Self::InsurgentCapability { in_effect } => ("Insurgent Capability", in_effect),
            Self::GovernmentMomentum { in_effect } => ("Government Momentum", in_effect),
        };
        if in_effect {
            format!("{name} · in effect")
        } else {
            format!("{name} · not in effect until executed")
        }
    }
    fn rule(self) -> &'static str {
        match self {
            Self::InsurgentCapability { .. } => "R 5.3",
            Self::GovernmentMomentum { .. } => "R 5.4",
        }
    }
}

/// The role of an Eligible faction on the card (R 2.3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Role {
    First,
    Second,
}

/// Which of a dual-use Event's texts is being, or was, carried out (R 5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EventText {
    Unshaded,
    Shaded,
}
impl EventText {
    fn name(self) -> &'static str {
        match self {
            Self::Unshaded => "unshaded",
            Self::Shaded => "shaded",
        }
    }
}

/// Where a shown card's Event stands now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum EventStatus {
    /// The preview: the draw deck's top card, played after the current card (R 2.3.8).
    Preview {
        after: u8,
    },
    /// An Eligible faction may execute the Event now (R 2.3.4).
    Open {
        faction: Faction,
        role: Role,
    },
    /// The 1st Eligible is executing its Operation. The 2nd may execute the Event only
    /// if a Special Activity accompanies that Operation (R 2.3.4).
    AwaitingOperation {
        executing: Faction,
        second: Faction,
        special_activity: bool,
    },
    /// After the 1st Eligible's Operation Only, the 2nd may take only a Limited
    /// Operation, so nobody executes the Event on this card (R 2.3.4).
    Closed {
        executed: Faction,
        second: Option<Faction>,
    },
    InProgress {
        faction: Faction,
        text: EventText,
    },
    Resolved {
        faction: Faction,
        text: EventText,
    },
}
impl EventStatus {
    pub fn text(self) -> String {
        match self {
            Self::Preview { after } => {
                format!("Event not yet open: this card is played after card {after}")
            }
            Self::Open { faction, role } => format!(
                "Event open to {} as {} Eligible",
                faction.name(),
                match role {
                    Role::First => "1st",
                    Role::Second => "2nd",
                }
            ),
            Self::AwaitingOperation {
                executing,
                second,
                special_activity: false,
            } => format!(
                "Event not yet open: {} may execute it if {}'s Operation includes a Special Activity",
                second.name(),
                executing.name()
            ),
            Self::AwaitingOperation {
                executing, second, ..
            } => format!(
                "Event not yet open: {} may execute it once {} ends its Operation",
                second.name(),
                executing.name()
            ),
            Self::Closed {
                executed,
                second: Some(second),
            } => format!(
                "Event not open this card: after {}'s Operation Only, {} may take only a Limited Operation",
                executed.name(),
                second.name()
            ),
            Self::Closed {
                executed,
                second: None,
            } => format!(
                "Event not open this card: no Eligible faction follows {}",
                executed.name()
            ),
            Self::InProgress { faction, text } => format!(
                "Event in progress: {} is executing the {} text",
                faction.name(),
                text.name()
            ),
            Self::Resolved { faction, text } => format!(
                "Event resolved: {} executed the {} text",
                faction.name(),
                text.name()
            ),
        }
    }
    fn rule(self) -> &'static str {
        match self {
            Self::Preview { .. } => "R 2.3.8",
            _ => "R 2.3.4",
        }
    }
}

/// The card tag's words for each fact, worded once here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CardWords {
    pub event: &'static str,
    pub status: String,
    pub lasting: String,
    pub rules: String,
}

/// One shown card: its printed facts and where its Event stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CardFacts {
    pub number: u8,
    pub title: &'static str,
    /// The printed faction order, left to right (R 2.3.2).
    pub order: [Faction; 4],
    pub kind: EventKind,
    pub lasting: Option<LastingEffect>,
    pub status: EventStatus,
    /// The rules behind the status, the Event's kind and its lasting effect.
    pub rules: Vec<&'static str>,
    pub words: CardWords,
}

/// The current card, on top of the played pile, and the preview, on top of the draw
/// deck (R 2.2, 2.3.8).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ShownCards {
    pub current: CardFacts,
    pub preview: CardFacts,
}

/// The cards in a deck: 48 Events and 4 Propaganda cards (Deck Preparation).
pub const DECK_CARDS: u8 = 52;
/// The Propaganda cards in a deck; the game ends at the fourth (R 2.4.1).
pub const PROPAGANDA_CARDS: u8 = 4;

/// Whether a card is one of the four Propaganda cards, which the playbook's card list
/// numbers 49 to 52.
pub fn is_propaganda(card: u8) -> bool {
    (49..=DECK_CARDS).contains(&card)
}

/// What the two piles show without opening them (R 2.2): how many cards each holds
/// and how many Propaganda cards have been played. The order of the cards under the
/// preview is not among them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Piles {
    /// Cards in the played pile, counting the current card on top of it.
    pub played: u8,
    /// Cards in the draw deck, counting the preview on top of it.
    pub deck: u8,
    /// Propaganda cards played so far, the current card among them if it is one.
    pub propaganda_played: u8,
    /// The Propaganda cards in the deck.
    pub propaganda_cards: u8,
}

impl Piles {
    /// The piles for the cards played before the current one, the current card and
    /// the number of cards in the draw deck.
    pub(crate) fn new(played_before: &[u8], current: u8, deck: usize) -> Self {
        let played = played_before.iter().chain([&current]);
        Self {
            played: played_before.len() as u8 + 1,
            deck: deck as u8,
            propaganda_played: played.filter(|card| is_propaganda(**card)).count() as u8,
            propaganda_cards: PROPAGANDA_CARDS,
        }
    }
}

/// The printed facts of an admitted face: title, order, Event kind and lasting box.
struct Printed {
    title: &'static str,
    order: [Faction; 4],
    kind: EventKind,
    capability: bool,
}

fn printed(number: u8) -> Printed {
    use Faction::{Directorio, Government, M26, Syndicate};
    match number {
        17 => Printed {
            title: "Alberto Bayo",
            order: CARD_17_ORDER,
            kind: EventKind::DualUse,
            capability: false,
        },
        18 => Printed {
            title: "Pact of Caracas",
            order: CARD_17_ORDER,
            kind: EventKind::Single,
            capability: true,
        },
        19 => Printed {
            title: "Sierra Maestra Manifesto",
            order: [M26, Directorio, Syndicate, Government],
            kind: EventKind::Single,
            capability: false,
        },
        _ => unreachable!("card {number} is not among the admitted faces"),
    }
}

impl CardFacts {
    /// The title printed on an admitted face (cards 17, 18 and 19).
    pub fn title_of(number: u8) -> &'static str {
        printed(number).title
    }
    fn new(number: u8, status: EventStatus, capabilities: &[String]) -> Self {
        let Printed {
            title,
            order,
            kind,
            capability,
        } = printed(number);
        let lasting = capability.then(|| LastingEffect::InsurgentCapability {
            in_effect: capabilities.iter().any(|c| c == title),
        });
        let rules: Vec<_> = [
            Some(status.rule()),
            Some(kind.rule()),
            lasting.map(LastingEffect::rule),
        ]
        .into_iter()
        .flatten()
        .collect();
        Self {
            number,
            title,
            order,
            kind,
            lasting,
            status,
            words: CardWords {
                event: kind.text(),
                status: status.text(),
                lasting: lasting.map_or_else(|| "None".into(), LastingEffect::text),
                rules: rules.join(" · "),
            },
            rules,
        }
    }
}

impl Game {
    /// Where the current card's Event stands, from the Eligibility record and the
    /// Event's progress.
    fn current_event(&self) -> EventStatus {
        let eligibility = self.eligibility();
        let second = eligibility.second;
        if self.card_18 {
            return if self.assault.is_some()
                || eligibility.government_box == Some(SequenceBox::OperationOnly)
            {
                // Government's Assault is an Operation Only in this build.
                EventStatus::Closed {
                    executed: Faction::Government,
                    second: Some(Faction::Syndicate),
                }
            } else {
                EventStatus::Open {
                    faction: eligibility.first,
                    role: Role::First,
                }
            };
        }
        let text = |event: &EventProgress| match event {
            EventProgress::ExecuteShaded | EventProgress::Complete(EventOutcome::Shaded { .. }) => {
                EventText::Shaded
            }
            _ => EventText::Unshaded,
        };
        if let Some(event) = &self.event {
            let faction = Faction::Directorio;
            return if event.pending() {
                EventStatus::InProgress {
                    faction,
                    text: text(event),
                }
            } else {
                EventStatus::Resolved {
                    faction,
                    text: text(event),
                }
            };
        }
        match (self.action, eligibility.m26_box) {
            (ActionChoice::Open, _) | (ActionChoice::Pass, _) => EventStatus::Open {
                faction: eligibility.first,
                role: Role::First,
            },
            (_, Some(SequenceBox::OperationWithSpecialActivity)) => EventStatus::Open {
                faction: second.expect("Directorio is 2nd Eligible"),
                role: Role::Second,
            },
            (_, Some(SequenceBox::OperationOnly)) => EventStatus::Closed {
                executed: Faction::M26,
                second,
            },
            _ => EventStatus::AwaitingOperation {
                executing: Faction::M26,
                second: second.expect("Directorio is 2nd Eligible"),
                special_activity: self.kidnap.is_some(),
            },
        }
    }

    pub(crate) fn shown_cards(
        &self,
        current: u8,
        preview: u8,
        capabilities: &[String],
    ) -> ShownCards {
        ShownCards {
            current: CardFacts::new(current, self.current_event(), capabilities),
            preview: CardFacts::new(
                preview,
                EventStatus::Preview { after: current },
                capabilities,
            ),
        }
    }
}
