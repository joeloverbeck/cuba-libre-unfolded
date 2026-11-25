//! The seat brief (#204): what a faction's player knows at the table, from that faction's
//! side, given to its session when it joins. Its rules are those of the March 2019 Living
//! Rules; each statement names its own. Its numbers are the rules authority's. It gives no
//! advice, and tells nothing that the faction may not know.

use super::view::{RULES, Rules};
use crate::{Faction, Group, Kind, Observation, Played, Standing, State};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Brief {
    pub faction: Faction,
    pub name: &'static str,
    pub who: &'static str,
    /// The match revision its numbers are of.
    pub revision: u64,
    pub rules: Rules,
    pub victory: Victory,
    /// When victory is checked, and how the factions rank.
    pub ranking: &'static str,
    /// Its own pieces on the map.
    pub pieces: Vec<Pieces>,
    /// Its Available Forces.
    pub available: Vec<Pieces>,
    /// What an Operation and a Special Activity are.
    pub actions: &'static str,
    pub operations: Vec<Action>,
    pub special_activities: Vec<Action>,
    pub rivals: Vec<Victory>,
}

/// A faction's victory condition and how it stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Victory {
    pub name: &'static str,
    pub condition: &'static str,
    pub margin_rule: &'static str,
    /// What contributes.
    pub counts: &'static str,
    pub standing: Standing,
}

/// Pieces of one kind and state in a space, or of one kind Available.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pieces {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<String>,
    pub kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    pub count: u8,
}

/// An Operation or a Special Activity of the rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Action {
    pub name: &'static str,
    pub rule: &'static str,
    pub purpose: &'static str,
    /// The Operations a Special Activity may accompany.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accompanies: Option<&'static str>,
    /// Where this build plays it.
    pub played: String,
}

impl Brief {
    /// What `faction`'s player is told of `seen`.
    pub fn of(seen: &Observation, faction: Faction) -> Self {
        let own = |group: &&Group| group.faction == faction;
        Self {
            faction,
            name: faction.name(),
            who: who(faction),
            revision: seen.revision,
            rules: RULES,
            victory: Victory::of(seen, faction),
            ranking: RANKING,
            pieces: seen
                .spaces
                .iter()
                .flat_map(|space| {
                    space.pieces.iter().filter(own).map(|group| Pieces {
                        space: Some(space.name.clone()),
                        kind: group.kind,
                        state: Some(group.state),
                        count: group.count,
                    })
                })
                .collect(),
            available: seen
                .available
                .iter()
                .filter(own)
                .map(|group| Pieces {
                    space: None,
                    kind: group.kind,
                    state: None,
                    count: group.count,
                })
                .collect(),
            actions: ACTIONS,
            operations: operations(faction)
                .into_iter()
                .map(|(name, rule, purpose)| Action::new(faction, name, rule, purpose, None))
                .collect(),
            special_activities: special_activities(faction)
                .into_iter()
                .map(|(name, rule, accompanies, purpose)| {
                    Action::new(faction, name, rule, purpose, Some(accompanies))
                })
                .collect(),
            rivals: Faction::ALL
                .into_iter()
                .filter(|rival| *rival != faction)
                .map(|rival| Victory::of(seen, rival))
                .collect(),
        }
    }
}

impl Victory {
    fn of(seen: &Observation, faction: Faction) -> Self {
        let (condition, margin_rule, counts) = victory(faction);
        Self {
            name: faction.name(),
            condition,
            margin_rule,
            counts,
            standing: seen
                .standings
                .iter()
                .find(|standing| standing.faction == faction)
                .expect("the authority states every faction's standing")
                .clone(),
        }
    }
}

impl Action {
    fn new(
        faction: Faction,
        name: &'static str,
        rule: &'static str,
        purpose: &'static str,
        accompanies: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            rule,
            purpose,
            accompanies,
            played: match Played::of(faction, name) {
                Played::Nowhere => "not in this build".into(),
                Played::OnCard(card) => format!("on card {card}"),
                Played::FreeRally(card) => format!(
                    "on card {card}, as the free Rally its unshaded Event grants where the faction has a Base"
                ),
            },
        }
    }
}

fn who(faction: Faction) -> &'static str {
    match faction {
        Faction::Government => {
            "You play the Government (Govt, blue), the one faction that is not Insurgent (R 1.0, 1.5). Its pieces are Troops, Police and Bases (R 1.4). Every other faction is its enemy (R 1.5)."
        }
        Faction::M26 => {
            "You play 26July (M26, red), an Insurgent faction: the leftist Movement of 26July, led by Fidel Castro (R 1.0, 1.5; Key Terms Index). Its pieces are Guerrillas and Bases (R 1.4). Every other faction is its enemy (R 1.5)."
        }
        Faction::Directorio => {
            "You play Directorio (DR, yellow), an Insurgent faction: the anti-communist, anti-Batista Revolutionary Directorate (R 1.0, 1.5). Its pieces are Guerrillas and Bases (R 1.4). Every other faction is its enemy (R 1.5)."
        }
        Faction::Syndicate => {
            "You play the Syndicate (green), an Insurgent faction: organized crime (R 1.0, 1.5). Its pieces are Guerrillas and Casinos, which are its Bases (R 1.4). Every other faction is its enemy (R 1.5)."
        }
    }
}

/// A faction's condition (R 7.2), its margin (R 7.3) and what contributes.
fn victory(faction: Faction) -> (&'static str, &'static str, &'static str) {
    match faction {
        Faction::Government => (
            "All Cities are at Active Support and Total Support exceeds 18 (R 7.2).",
            "Total Support − 18 (R 7.3).",
            "Total Support is 2 × the Population in Active Support + 1 × the Population in Passive Support (R 1.6.3).",
        ),
        Faction::M26 => (
            "Total Opposition plus the number of 26July Bases exceeds 15 (R 7.2).",
            "Total Opposition + 26July Bases − 15 (R 7.3).",
            "Total Opposition is 2 × the Population in Active Opposition + 1 × the Population in Passive Opposition; its Bases on the map count (R 1.6.3).",
        ),
        Faction::Directorio => (
            "Total Population in spaces Controlled by Directorio plus the number of Directorio Bases exceeds 9 (R 7.2).",
            "Directorio-Controlled Population + Directorio Bases − 9 (R 7.3).",
            "It Controls a Province or City if its pieces there exceed those of all other factions combined; a closed Casino is no piece for this (R 1.7). Its Bases on the map count (R 1.6.3).",
        ),
        Faction::Syndicate => (
            "The Syndicate has more than 7 open Casinos and Resources above 30 (R 7.2).",
            "The lower of open Casinos − 7 and Syndicate Resources − 30 (R 7.3).",
            "Open Casinos on the map (R 1.4.4, 1.6.3) and its Resources (R 1.8).",
        ),
    }
}

const RANKING: &str = "Victory is checked at the start of each Propaganda Round: if a faction meets its condition, the game ends (R 6.1, 7.2). Otherwise it ends with the final Propaganda card (R 2.4.1, 6.3.5). Either way the faction with the highest victory margin comes first (R 7.1, 7.3), and ties go to the Syndicate, then Directorio, then 26July (R 7.1).";

const ACTIONS: &str = "A faction that executes an Operation chooses 1 of these 4 (R 3.1). With an Operation in at least 1 space it may execute 1 type of Special Activity, at any one time immediately before, during or immediately after it; a Limited Operation, in 1 space, has none (R 2.3.5, 4.1). \"played\" says where this build plays each. Pass (R 2.3.3) and a card's Event (R 5.1) are neither: your view lists them when they are yours to choose.";

/// A name, its section and its purpose.
type Operation = (&'static str, &'static str, &'static str);

const MARCH: Operation = (
    "March",
    "R 3.3.2",
    "Moves its Guerrillas into adjacent spaces, where a moving group may become Active.",
);
const ATTACK: Operation = (
    "Attack",
    "R 3.3.3",
    "Where it has a Guerrilla and there is an enemy piece: Activates all its Guerrillas there and rolls a die to remove up to 2 enemy pieces; Casinos close instead.",
);
const TERROR_TO_NEUTRAL: Operation = (
    "Terror",
    "R 3.3.4",
    "Activates 1 of its Underground Guerrillas in each selected space: in a City or Province, places a Terror marker and shifts any Support or Opposition 1 level toward Neutral; in an Economic Center, places Sabotage.",
);

/// The 4 Operations of a faction (R 3.2, 3.3), by section.
fn operations(faction: Faction) -> [Operation; 4] {
    match faction {
        Faction::Government => [
            (
                "Train",
                "R 3.2.1",
                "Places Troops and Police in Cities and in Provinces with a Government Base; then, in up to 1 selected space, replaces 2 cubes with a Base or builds Support by Civic Action.",
            ),
            (
                "Garrison",
                "R 3.2.2",
                "Moves Troops and Police to Economic Centers and Cities and Activates Guerrillas in Economic Centers; may then Assault in 1 of them.",
            ),
            (
                "Sweep",
                "R 3.2.3",
                "Moves adjacent Troops into selected Cities and Provinces, then Activates Guerrillas there.",
            ),
            (
                "Assault",
                "R 3.2.4",
                "Removes Insurgents' Active Guerrillas in selected spaces and, once a targeted faction has no Guerrillas there, its Bases; Casinos close instead.",
            ),
        ],
        Faction::M26 => [
            (
                "Rally",
                "R 3.3.1",
                "In Provinces and Cities at Neutral or Opposition: places Guerrillas or replaces 2 with a Base; where it has a Base, may instead flip all its Guerrillas there Underground.",
            ),
            MARCH,
            ATTACK,
            (
                "Terror",
                "R 3.3.4",
                "Activates 1 of its Underground Guerrillas in each selected space: in a City or Province, places a Terror marker and shifts the space 1 level toward Active Opposition; in an Economic Center, places Sabotage.",
            ),
        ],
        Faction::Directorio => [
            (
                "Rally",
                "R 3.3.1",
                "In Provinces and Cities at Neutral, Passive Support or Passive Opposition: places Guerrillas or replaces 2 with a Base; where it has a Base, may instead flip all its Guerrillas there Underground.",
            ),
            MARCH,
            ATTACK,
            TERROR_TO_NEUTRAL,
        ],
        Faction::Syndicate => [
            (
                "Rally",
                "R 3.3.1",
                "In Provinces and Cities: places a Guerrilla; where it has an open Casino, may instead flip all its Guerrillas there Underground.",
            ),
            MARCH,
            TERROR_TO_NEUTRAL,
            (
                "Construct",
                "R 3.3.5",
                "In Cities and Provinces that the Government or the Syndicate Controls: places a closed Casino or opens a closed one.",
            ),
        ],
    }
}

/// A name, its section, the Operations it may accompany and its purpose.
type SpecialActivity = (&'static str, &'static str, &'static str, &'static str);

const ANY_OPERATION: &str = "any Operation (R 4.1.1)";
const AMBUSH: &str = "In 1 space selected for Attack where it has an Underground Guerrilla: the Attack Activates only 1 such Guerrilla, succeeds without a roll and places 1 of its Guerrillas.";

/// The 3 Special Activities of a faction (R 4.2 to 4.5), by section.
fn special_activities(faction: Faction) -> [SpecialActivity; 3] {
    match faction {
        Faction::Government => [
            (
                "Transport",
                "R 4.2.1",
                ANY_OPERATION,
                "Moves up to 3 Troops from a City or Base to any 1 other space.",
            ),
            (
                "Air Strike",
                "R 4.2.2",
                "Garrison, Sweep or Assault",
                "In a Province or Economic Center, unless US Alliance is Embargoed: removes 1 Active Guerrilla or, if the targeted faction has no Guerrillas there, removes or closes 1 of its Bases.",
            ),
            (
                "Reprisal",
                "R 4.2.3",
                "Garrison, Sweep or Assault",
                "In 1 Government-Controlled space: places a Terror marker, shifts any Opposition 1 level toward Neutral and relocates 1 Guerrilla to an adjacent space.",
            ),
        ],
        Faction::M26 => [
            (
                "Infiltrate",
                "R 4.3.1",
                "Rally or March",
                "In 1 space without Support that has or is adjacent to an Underground 26July Guerrilla: removes 1 Police, or 1 Troops if no Police are there, or replaces it with a 26July Guerrilla.",
            ),
            ("Ambush", "R 4.3.2", "Attack", AMBUSH),
            (
                "Kidnap",
                "R 4.3.3",
                "Terror",
                "In 1 space of this Terror that is a City or an Economic Center or has an open Casino, with more 26July Guerrillas than Police: takes a die roll of Resources, or 1 Cash marker if the target has Cash there, from the Government (City or Economic Center) or the Syndicate (open Casino), then closes 1 open Casino there, if any.",
            ),
        ],
        Faction::Directorio => [
            (
                "Subvert",
                "R 4.4.1",
                "Rally or March",
                "In 1 Directorio-Controlled Province: adds its Population to Directorio Resources and sets it to Neutral.",
            ),
            ("Ambush", "R 4.4.2", "Attack", AMBUSH),
            (
                "Assassinate",
                "R 4.4.3",
                "Terror",
                "In 1 space selected for this Terror where Directorio Guerrillas outnumber Police: removes any 1 enemy piece; a Casino closes instead.",
            ),
        ],
        Faction::Syndicate => [
            (
                "Profit",
                "R 4.5.1",
                "Rally, March or Construct",
                "Places 1 Cash marker in each of 1 or 2 spaces with an open Casino; or closes Casinos and removes Syndicate Cash for 3 Resources each.",
            ),
            (
                "Muscle",
                "R 4.5.3",
                "Rally, March or Construct",
                "Relocates 1 or 2 Police to a City with an open Casino, or 1 or 2 Troops to an Economic Center or to a Province with an open Casino.",
            ),
            (
                "Bribe",
                "R 4.5.4",
                ANY_OPERATION,
                "In 1 space, for 3 Resources: removes up to 2 cubes, removes or flips up to 2 Guerrillas, or removes an enemy Base.",
            ),
        ],
    }
}
