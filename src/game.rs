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
        [("finish","Finish Operation Only",true,"Zero-space completion uses the accepted interpretation. No Pass income."),
         ("kidnap","Kidnap in Santiago",false,"Legal funding route: 2 Guerrillas > 1 Police; die 1–6 from Government 6. Incomplete until ticket #14."),
         ("terror","Terror in Santiago / Oriente",false,"Each costs 1 Resource and needs an Underground Guerrilla. Currently unaffordable at 0; legal after funding. Incomplete until #14."),
         ("transfer","Resource transfer",false,"Giver-authorized transfers between 26July and each other faction are legal. Incomplete until #13."),
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
        self.finished = true;
        self.revision += 1;
        Ok(())
    }
    /// #12 admits exactly its two implemented states; later tickets extend this boundary.
    pub fn validate(&self) -> Result<(), String> {
        let mut expected = Self::prepared();
        if self.finished {
            expected.finished = true;
            expected.revision = 1;
        }
        if *self != expected {
            return Err("State is not a supported initial or zero-space finished position.".into());
        }
        Ok(())
    }
}
