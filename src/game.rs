//! Cuba-owned rules from the accepted deacd635 tactical handoff (R 1.3–1.8, 2.3, 7).
use serde::{Deserialize, Serialize};

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
    pub alliance: &'static str,
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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub supported: bool,
    pub explanation: String,
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
            owner: if self.finished {
                Faction::Directorio
            } else if let Some(Transfer::Authorizing { giver, .. }) = self.transfer {
                giver
            } else {
                Faction::M26
            },
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
            passed: [false; 4],
            aid: 15,
            alliance: "Firm",
            cash_available: 4,
            terror_available: 20,
            capabilities: vec![],
            momentum: vec![],
            finished: self.finished,
            choices: self.choices(),
            next_options: if self.finished {
                vec![
                    "Pass (+1 if executed later)".into(),
                    "Limited Operation".into(),
                ]
            } else {
                vec![]
            },
            operation: if self.finished {
                "Operation Only completed; no Pass income"
            } else {
                "Full Terror already selected"
            },
            special_activity: None,
            owed_terror: None,
            transfer: self.transfer.clone(),
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
    fn choices(&self) -> Vec<Choice> {
        if self.finished {
            return vec![];
        }
        let choice = |id: String, label: String| {
            Choice {
            id: format!("{}:{id}", self.revision), label, supported: true,
            explanation: "Only the giver selects the amount and authorizes; no resources move before authorization.".into(),
        }
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
                return choices;
            }
            None => {}
        }
        [("finish","Finish Operation Only",true,"Zero-space completion uses the accepted interpretation. No Pass income."),
         ("kidnap","Kidnap in Santiago",false,"2 Guerrillas > 1 Police; die 1–6 takes up to Government's current Resources. Incomplete until ticket #14."),
         ("terror","Terror in Santiago / Oriente",false,"Each costs 1 Resource and needs an Underground Guerrilla. Compare 26July's current Resources above. Incomplete until #14."),
         ("transfer","Resource transfer",true,"Choose a direction, then hand the decision to its giver."),
         ("remove","Voluntary own removal",false,"26July may remove its Guerrillas or Sierra Maestra Base. Incomplete until #15.")].into_iter().map(|(id,label,supported,explanation)|Choice{id:format!("{}:{id}",self.revision),label:label.into(),supported,explanation:explanation.into()}).collect()
    }
}
impl Game {
    pub fn random_state(&self) -> u64 {
        self.random
    }
    pub fn submit(&mut self, owner: Faction, revision: u64, choice: &str) -> Result<(), String> {
        if revision != self.revision {
            return Err("Stale match revision; observe again.".into());
        }
        if owner != self.observe().owner {
            return Err("Only the current decision owner may submit.".into());
        }
        let offered = self
            .choices()
            .into_iter()
            .find(|c| c.id == choice)
            .ok_or("Stale or unknown choice identity.")?;
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
        match self.transfer.clone() {
            None if action == "finish" => self.finished = true,
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
                    self.resources[giver.index()] -= amount;
                    self.resources[recipient.index()] += amount;
                    self.transfer = None;
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
    /// Admit only this prepared board, conserved transfers and zero-space Finish.
    pub fn validate(&self) -> Result<(), String> {
        let mut expected = Self::prepared();
        if self.resources.iter().any(|&n| n > 49)
            || self.resources.iter().map(|&n| n as u16).sum::<u16>() != 26
            || (self.revision == 0 && *self != expected)
            || (self.finished && self.transfer.is_some())
        {
            return Err("Unsupported transfer balances or decision state.".into());
        }
        if let Some(Transfer::Authorizing {
            giver,
            recipient,
            amount,
        }) = self.transfer
            && (giver == recipient
                || (giver != Faction::M26 && recipient != Faction::M26)
                || amount > self.resources[giver.index()]
                || u16::from(self.resources[recipient.index()]) + u16::from(amount) > 49)
        {
            return Err("Unsupported pending giver authorization.".into());
        }
        expected.finished = self.finished;
        expected.revision = self.revision;
        expected.resources = self.resources;
        expected.transfer = self.transfer.clone();
        if *self != expected {
            return Err(
                "State is not a supported prepared transfer or zero-space finished position."
                    .into(),
            );
        }
        Ok(())
    }
}
