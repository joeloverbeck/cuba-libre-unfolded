//! Cuba-owned rules from the accepted deacd635 tactical handoff (R 1.3–1.8, 2.3, 7).
use serde::{Deserialize, Serialize};
use unfolded_random::SeededRandom;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionChoice {
    /// The action choice is open: Pass and Terror are playable; the Event, Rally,
    /// March and Attack are listed but not in this task.
    Open,
    /// R 2.3.3: +1 Resource, 26July stays Eligible and Directorio becomes 1st Eligible.
    /// The task ends there.
    Pass,
    /// Terror is chosen: 26July executes it until End Terror. A #8 save began here, at
    /// revision 0, and has no `action` field; it resumes in this state.
    #[default]
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
/// Card 17's Eligibility record. Nobody turns Ineligible inside the task: R 2.3.7
/// adjusts cylinders only after the card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Eligibility {
    /// Card 17's faction order, left to right (R 2.3.2).
    pub order: [Faction; 4],
    /// Eligible or Ineligible, indexed like `Faction::ALL`.
    pub eligible: [bool; 4],
    /// The 1st Eligible faction: Directorio after 26July Passes (R 2.3.3).
    pub first: Faction,
    pub second: Faction,
    /// The box 26July's cylinder occupies, once its action is recorded.
    pub m26_box: Option<SequenceBox>,
}
/// Card 17's shaded text, from the admitted card face, which Directorio may execute
/// after an Operation with Special Activity (R 2.3.4, 5.2).
pub const CARD_17_SHADED: &str =
    "All 26July Guerrillas Active. 26July Ineligible through next card.";
/// Card 17's order: 26July, Directorio, Government, Syndicate (rules baseline).
pub const CARD_17_ORDER: [Faction; 4] = [
    Faction::M26,
    Faction::Directorio,
    Faction::Government,
    Faction::Syndicate,
];
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
    #[serde(default)]
    pub(crate) action: ActionChoice,
    /// Set by the first real effect: an authorized nonzero transfer, a resolved
    /// Terror, a Kidnap roll or a removal. Going back to the action choice is local
    /// policy (#58) allowed only before it. A #8 save lacks it; `effected` also reads
    /// such a save's state.
    #[serde(default)]
    pub(crate) effect: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Removal {
    Selecting,
    Confirming {
        space: String,
        kind: Kind,
        state: State,
    },
}
/// The committed die and its actual proceeds. Balances at the roll remain
/// inspectable even when subsequent Terror or donations change current balances.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kidnap {
    pub die: u8,
    pub proceeds: u8,
    pub government_before: u8,
    pub m26_before: u8,
    pub random_before: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Transfer {
    Selecting,
    Authorizing {
        giver: Faction,
        recipient: Faction,
        amount: u8,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub revision: u64,
    pub owner: Faction,
    /// 26July's action choice: open, Pass or Terror.
    pub action: ActionChoice,
    /// The faction executing an Operation now (glossary): 26July from choosing Terror
    /// until End Terror; none at the action choice or after Pass. Resource transfers are
    /// offered only while it is set (R 1.5.1).
    pub executing: Option<Faction>,
    pub eligibility: Eligibility,
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
    pub next_options: Vec<String>,
    pub operation: &'static str,
    pub special_activity: Option<String>,
    pub owed_terror: Option<String>,
    pub transfer: Option<Transfer>,
    pub kidnap: Option<Kidnap>,
    pub removal: Option<Removal>,
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
    /// Directorio's options, when the step ends 26July's action.
    pub next: Option<Next>,
    /// Facts with rule references, such as the zero-space warning.
    pub notes: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ResourceChange {
    pub faction: Faction,
    pub from: u8,
    pub to: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
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
            resources: [6, 0, 5, 15],
            spaces,
            finished: false,
            random: 17,
            draw_deck,
            transfer: None,
            kidnap: None,
            removal: None,
            action: ActionChoice::Terror,
            effect: false,
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
        Observation {
            revision: self.revision,
            owner: self.owner(),
            action: self.action,
            executing: self.executing(),
            eligibility: self.eligibility(),
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
            eligible: [true; 4],
            passed: [false, self.action == ActionChoice::Pass, false, false],
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
            next_options: self.next_options(),
            operation: match self.action {
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
            kidnap: self.kidnap.clone(),
            removal: self.removal.clone(),
        }
    }
    /// Directorio's options once 26July's action is recorded (R 2.3.3–2.3.4).
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
    /// roll is never previewed here, so no future die result can show.
    fn preview(&self, choice: &Choice) -> Option<Preview> {
        if !choice.supported {
            return None;
        }
        let action = choice.id.split_once(':')?.1;
        let steps = match action {
            "pass" | "end-terror" | "confirm-removal" => vec![action],
            _ if action.starts_with("terror:") => vec![action],
            // A removal candidate previews the removal its confirmation would make.
            _ if action.starts_with("remove:") => vec![action, "confirm-removal"],
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
            preview.records = after.eligibility().m26_box;
            preview.next = Some(Next {
                faction: Faction::Directorio,
                role: if after.action == ActionChoice::Pass {
                    "1st Eligible"
                } else {
                    "2nd Eligible"
                }
                .into(),
                options: after.next_options(),
            });
        } else {
            // Compare the Operation menus, so a removal's own menu does not count.
            let mut menu = self.clone();
            menu.removal = None;
            let later = after.choices();
            let suffix = |c: &Choice| {
                c.id.split_once(':')
                    .map_or(String::new(), |(_, a)| a.to_owned())
            };
            for choice in menu.choices().iter().filter(|c| c.supported) {
                let name = suffix(choice);
                if name == action || name == "confirm-removal" {
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
            notes.push("R 2.3.3: 26July stays Eligible, and Directorio becomes 1st Eligible. The task ends there; Directorio's options are shown, not executed.".into());
        } else if action == "end-terror" {
            let resolved = self.spaces.iter().any(|s| s.terror > 0);
            if self.kidnap.is_some() {
                notes.push(
                    "Records Operation with Special Activity: Terror with Kidnap (R 2.3.4).".into(),
                );
                notes.push(format!("Directorio, 2nd Eligible, may Pass, take a Limited Operation or execute card 17's Event, either text (R 2.3.4, 5.2). Shaded: \"{CARD_17_SHADED}\""));
            } else {
                notes.push("Records Operation Only (R 2.3.4).".into());
                notes.push("Directorio, 2nd Eligible, may Pass or take a Limited Operation; the Event follows only an Operation with Special Activity (R 2.3.4).".into());
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
        } else if action.starts_with("terror:") {
            notes.push("R 3.3.4: pay 1 Resource, flip 1 Underground Guerrilla there to Active, place a Terror marker and shift 1 level toward Active Opposition.".into());
        } else {
            notes.push("R 1.4.1: the piece returns to Available. Earlier Terror markers, support shifts and Kidnap results stay.".into());
        }
        if !self.effected() && after.effected() && !after.finished {
            notes.push("This is the first effect: going back to the action choice ends (local policy, #58).".into());
        }
        notes
    }
    fn executing(&self) -> Option<Faction> {
        (self.action == ActionChoice::Terror && !self.finished).then_some(Faction::M26)
    }
    fn eligibility(&self) -> Eligibility {
        let passed = self.action == ActionChoice::Pass;
        Eligibility {
            order: CARD_17_ORDER,
            eligible: [true; 4],
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
                        let resources = self.resources[3] as i16;
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
    /// Who decides now: the giver of a pending authorization, Directorio once 26July's
    /// action is recorded, otherwise 26July.
    fn owner(&self) -> Faction {
        if self.finished {
            Faction::Directorio
        } else if let Some(Transfer::Authorizing { giver, .. }) = self.transfer {
            giver
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
                format!("Pass · +1 Resource ({resources} → {})", resources.saturating_add(1).min(49)),
                true,
                "R 2.3.3: 26July gains 1 Resource and stays Eligible; Directorio becomes 1st Eligible. The task ends without executing Directorio.",
            ),
            absent(
                "event:unshaded",
                "Event (unshaded)",
                "R 2.3.4, 5.2. Card 17, Vet trains guerrillas: 26July or DR free Rallies in each space it has a Base (as if spaces Neutral).",
            ),
            absent(
                "event:shaded",
                "Event (shaded)",
                "R 2.3.4, 5.2. Card 17, Mexico blocks training by Cuban expat: All 26July Guerrillas Active. 26July Ineligible through next card.",
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
                "R 3.3.4: in each selected space pay 1 Resource, flip 1 Underground Guerrilla there to Active, place Terror and shift 1 level toward Active Opposition. While 26July executes Terror, Resource transfers are open (R 1.5.1; offering them from this choice is local policy). Going back to this choice is allowed until the first effect (local policy).",
            ),
            under(
                self.offer(
                    "terror:kidnap",
                    "Kidnap · with Terror · offered once chosen",
                    false,
                    "R 4.1, 4.3.3: Kidnap accompanies Terror in a space selected for that Terror, here Santiago. Choose Terror; Kidnap is then offered.",
                ),
                "terror",
            ),
        ]
    }
    fn choices(&self) -> Vec<Choice> {
        if self.finished {
            return vec![];
        }
        if self.action == ActionChoice::Open {
            return self.opening_choices();
        }
        if let Some(removal) = &self.removal {
            let mut choices = match removal {
                Removal::Selecting => self.removal_choices(),
                Removal::Confirming { space, kind, state } => vec![self.offer(
                    "confirm-removal",
                    "Confirm · return 1 to Available",
                    self.removable(space, *kind, *state),
                    "Remove only this 26July piece. Earlier Terror and Opposition remain.",
                )],
            };
            choices.push(self.offer(
                "cancel-removal",
                "Cancel removal · return to Operation",
                true,
                "No piece is removed until confirmation. Cancellation never reverses earlier effects.",
            ));
            return choices;
        }
        let choice = |id: String, label: String| {
            self.offer(
                &id,
                label,
                true,
                "Only the giver selects the amount and authorizes; no resources move before authorization.",
            )
        };
        let finish = || {
            self.offer(
                "end-terror",
                "End Terror",
                !self.owes_santiago(),
                if self.owes_santiago() {
                    "Santiago Terror is owed: under the accepted commitment policy, its payment and Activation come before End Terror."
                } else if self.kidnap.is_some() {
                    "Ends 26July's Terror and records Operation with Special Activity (R 2.3.4)."
                } else {
                    "Ends 26July's Terror and records Operation Only (R 2.3.4)."
                },
            )
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
            Some(Transfer::Selecting) => {
                let mut choices: Vec<_> =
                    [Faction::Government, Faction::Directorio, Faction::Syndicate]
                        .into_iter()
                        .flat_map(|other| [(other, Faction::M26), (Faction::M26, other)])
                        .map(|(giver, recipient)| {
                            choice(
                                format!("transfer:{}:{}", giver.index(), recipient.index()),
                                format!("{} → {}", giver.name(), recipient.name()),
                            )
                        })
                        .collect();
                choices.push(choice("back".into(), "Return to Operation".into()));
                return choices;
            }
            Some(Transfer::Authorizing {
                giver,
                recipient,
                amount,
            }) => {
                let limit = self.resources[giver.index()]
                    .saturating_sub(u8::from(self.owes_santiago() && giver == Faction::M26))
                    .min(49u8.saturating_sub(self.resources[recipient.index()]));
                let mut choices = vec![];
                if amount > 0 {
                    choices.push(choice(
                        format!("amount:{}", amount - 1),
                        format!("Amount −1 (choose {})", amount - 1),
                    ));
                }
                if amount < limit {
                    choices.push(choice(
                        format!("amount:{}", amount + 1),
                        format!("Amount +1 (choose {})", amount + 1),
                    ));
                }
                choices.push(choice(
                    "authorize".into(),
                    format!("{} authorizes {amount}", giver.name()),
                ));
                choices.push(choice(
                    "decline".into(),
                    format!("{} declines", giver.name()),
                ));
                choices.push(choice("wait".into(), "Wait · keep this decision".into()));
                // Local policy (#58): 26July's request does not hold 26July's own
                // Operation hostage; ending it or going back withdraws the request.
                if giver != Faction::M26 {
                    if self.may_go_back() {
                        choices.push(back("26July withdraws · back to action choice"));
                    }
                    let mut end = finish();
                    end.label = format!("26July withdraws · {}", end.label);
                    end.owner = Faction::M26;
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
                "Choose a direction, then hand the decision to its giver.",
            ),
            self.offer(
                "remove",
                "Voluntary removal",
                true,
                "Choose one 26July Guerrilla or Base, then confirm or cancel. Only an already owed Terror restricts removal.",
            ),
        ];
        let kidnap = self.kidnap.is_none().then(|| {
            let santiago = self.spaces.iter().find(|s| s.id == "santiago").unwrap();
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
            let police = count(Kind::Police, Faction::Government, None);
            let feasible = santiago.terror > 0
                || (count(Kind::Guerrilla, Faction::M26, Some(State::Underground)) > 0
                    && (self.resources[1] > 0 || self.resources[0] > 0));
            self.offer(
                "kidnap",
                "Kidnap · Santiago · Government",
                guerrillas > police && feasible,
                format!("{guerrillas} Guerrillas (Active + Underground) vs {police} Police; must outnumber. Die 1–6: proceeds {}–{} from Government. {}", self.resources[0].min(1), self.resources[0].min(6), if feasible {"Santiago Terror was resolved or is guaranteed feasible on every roll; an early roll commits its payment and Underground Guerrilla."} else {"Santiago needs an Underground Guerrilla and guaranteed payment without a promised Resource transfer or favorable roll."}),
            )
        });
        for id in ["santiago", "oriente"] {
            choices.push(self.terror_target(id, kidnap.as_ref()));
        }
        choices.extend(kidnap);
        if self.may_go_back() {
            choices.push(back("Back to the action choice"));
        }
        choices
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
        let cost = format!("Cost: 1 Resource per space (26July has {m26}).");
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
            if id == "santiago" && kidnap.is_some_and(|k| k.supported) && self.resources[0] > 0 {
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
                None => format!("R 3.3.4. {cost} Flip 1 Underground Guerrilla there to Active, place a Terror marker and shift 1 level toward Active Opposition ({} → {}).", support_level(space.support), support_level((space.support - 1).max(-2))),
            },
        );
        choice.remedies = remedies;
        choice
    }
    fn removable(&self, id: &str, kind: Kind, state: State) -> bool {
        self.spaces.iter().find(|s| s.id == id).is_some_and(|s| {
            s.pieces.iter().any(|p| {
                p.faction == Faction::M26
                    && p.kind == kind
                    && p.state == state
                    && p.count
                        > u8::from(
                            self.owes_santiago() && id == "santiago" && state == State::Underground,
                        )
            })
        })
    }
    fn removal_choices(&self) -> Vec<Choice> {
        self.spaces.iter().flat_map(|s| s.pieces.iter().filter(|p| p.faction == Faction::M26).map(|p| {
            let suffix = match (p.kind, p.state) {
                (Kind::Base, _) => "base",
                (_, State::Active) => "active",
                _ => "underground",
            };
            self.offer(
                &format!("remove:{}:{suffix}", s.id),
                format!("{} · {} {suffix}", s.name, p.count),
                self.removable(&s.id, p.kind, p.state),
                "Return one own piece to Available. If Santiago Terror is owed, the accepted commitment policy protects its last Underground Guerrilla.",
            )
        })).collect()
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
            .ok_or("Stale or unknown choice identity.")?;
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
        if action == "wait" {
            return Ok(());
        }
        match action {
            // R 2.3.3. Only offered at the action choice.
            "pass" => {
                self.resources[Faction::M26.index()] = self.resources[Faction::M26.index()]
                    .saturating_add(1)
                    .min(49);
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
            _ => return self.execute(action, next_revision),
        }
        self.revision = next_revision;
        Ok(())
    }
    /// The #8 Operation steps: removal, transfers, Terror and Kidnap.
    fn execute(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        if let Some(removal) = self.removal.clone() {
            if action == "cancel-removal" {
                self.removal = None;
            } else if let Removal::Confirming { space, kind, state } = removal {
                let target = self.spaces.iter_mut().find(|s| s.id == space).unwrap();
                let piece = target
                    .pieces
                    .iter_mut()
                    .find(|p| p.faction == Faction::M26 && p.kind == kind && p.state == state)
                    .unwrap();
                piece.count -= 1;
                target.pieces.retain(|p| p.count > 0);
                self.effect = true;
                self.available
                    .iter_mut()
                    .find(|p| p.faction == Faction::M26 && p.kind == kind)
                    .unwrap()
                    .count += 1;
                self.removal = None;
            } else {
                let mut parts = action.split(':');
                parts.next();
                let space = parts.next().unwrap().to_owned();
                let (kind, state) = match parts.next().unwrap() {
                    "base" => (Kind::Base, State::Ordinary),
                    "active" => (Kind::Guerrilla, State::Active),
                    _ => (Kind::Guerrilla, State::Underground),
                };
                self.removal = Some(Removal::Confirming { space, kind, state });
            }
            self.revision = next_revision;
            return Ok(());
        }
        match self.transfer.clone() {
            None if action == "remove" => self.removal = Some(Removal::Selecting),
            None if action == "kidnap" => {
                self.effect = true;
                let mut random = SeededRandom::new(self.random);
                let die = (random.index(6) + 1) as u8;
                let proceeds = die.min(self.resources[0]);
                self.kidnap = Some(Kidnap {
                    die,
                    proceeds,
                    government_before: self.resources[0],
                    m26_before: self.resources[1],
                    random_before: self.random,
                });
                self.random = random.state();
                self.resources[0] -= proceeds;
                self.resources[1] += proceeds.min(49 - self.resources[1]);
            }
            None if action.starts_with("terror:") => {
                let id = action.strip_prefix("terror:").unwrap();
                let space = self.spaces.iter_mut().find(|s| s.id == id).unwrap();
                Self::resolve_terror(space);
                self.resources[1] -= 1;
                self.effect = true;
            }
            None => self.transfer = Some(Transfer::Selecting),
            Some(_) if action == "back" || action == "decline" => self.transfer = None,
            Some(Transfer::Selecting) => {
                let endpoints: Vec<_> = action.split(':').collect();
                let giver = Faction::ALL[endpoints[1].parse::<usize>().unwrap()];
                let recipient = Faction::ALL[endpoints[2].parse::<usize>().unwrap()];
                self.transfer = Some(Transfer::Authorizing {
                    giver,
                    recipient,
                    amount: 0,
                });
            }
            Some(Transfer::Authorizing {
                giver,
                recipient,
                amount,
            }) => {
                if action == "authorize" {
                    if self.owes_santiago() && giver == Faction::M26 && amount >= self.resources[1]
                    {
                        return Err("Santiago Terror is owed: retain 1 Resource under the accepted commitment policy.".into());
                    }
                    self.resources[giver.index()] -= amount;
                    self.resources[recipient.index()] += amount;
                    self.transfer = None;
                    // Zero is a deliberate non-transfer, not an effect.
                    self.effect |= amount > 0;
                } else {
                    self.transfer = Some(Transfer::Authorizing {
                        giver,
                        recipient,
                        amount: action.strip_prefix("amount:").unwrap().parse().unwrap(),
                    });
                }
            }
        }
        self.revision = next_revision;
        Ok(())
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
    /// Admit only the prepared board at the action choice, after Pass, or with Terror
    /// chosen: transfers, two Terror effects, Kidnap and own removal.
    pub fn validate(&self) -> Result<(), String> {
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
                for piece in &mut space.pieces {
                    if piece.faction != Faction::M26 {
                        continue;
                    }
                    let remaining = actual
                        .pieces
                        .iter()
                        .find(|p| {
                            p.faction == piece.faction
                                && p.kind == piece.kind
                                && p.state == piece.state
                        })
                        .map_or(0, |p| p.count);
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
            && self.transfer.is_none()
            && self.removal.is_none();
        let consistent = match self.action {
            ActionChoice::Open => untouched && !self.finished && self.resources == [6, 0, 5, 15],
            ActionChoice::Pass => untouched && self.finished && self.resources == [6, 1, 5, 15],
            ActionChoice::Terror => true,
        };
        expected.action = self.action;
        if !consistent
            || self.resources.iter().any(|&n| n > 49)
            || self.resources.iter().map(|&n| n as u16).sum::<u16>()
                != 26 - paid + u16::from(self.action == ActionChoice::Pass)
            || self.revision
                < u64::from(paid)
                    + u64::from(self.kidnap.is_some())
                    + u64::from(self.finished)
                    + removed
                    + u64::from(self.removal.is_some())
            || (self.revision == 0 && *self != expected)
            || (self.finished && (self.transfer.is_some() || self.removal.is_some()))
            || (self.transfer.is_some() && self.removal.is_some())
        {
            return Err("Unsupported transfer balances or decision state.".into());
        }
        if let Some(kidnap) = &self.kidnap {
            let mut rng = SeededRandom::new(kidnap.random_before);
            let die = (rng.index(6) + 1) as u8;
            if kidnap.die != die
                || self.random != rng.state()
                || kidnap.government_before > 26
                || kidnap.m26_before > 26
                || u16::from(kidnap.government_before) + u16::from(kidnap.m26_before) > 26
                || kidnap.proceeds != die.min(kidnap.government_before)
                || (self.owes_santiago()
                    && (self.finished
                        || self.resources[1] == 0
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
            && !self.removable(space, *kind, *state)
        {
            return Err("Unsupported pending own removal or protected owed Terror piece.".into());
        }
        if let Some(Transfer::Authorizing {
            giver,
            recipient,
            amount,
        }) = self.transfer
            && (giver == recipient
                || (giver != Faction::M26 && recipient != Faction::M26)
                || amount > self.resources[giver.index()]
                || (self.owes_santiago() && giver == Faction::M26 && amount >= self.resources[1])
                || u16::from(self.resources[recipient.index()]) + u16::from(amount) > 49)
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
        if *self != expected {
            return Err(
                "State is not a supported prepared transfer/Terror/Kidnap/removal position.".into(),
            );
        }
        Ok(())
    }
}
