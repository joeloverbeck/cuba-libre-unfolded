//! Government's Operation Only Assault on Pact of Caracas (R 2.3.4, 3.1, 3.2.4).
use super::*;

const OPEN: &str = "government:assault";
const SANTIAGO: &str = "assault:santiago:26july";
const FINISH: &str = "end-assault";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Assault {
    start: AssaultStart,
    actions: Vec<String>,
    resolution: Option<AssaultResolution>,
    pub(super) finished: bool,
}

/// The card 18 position before the Operation. Admission checks it against the
/// completed Event, then replays every Assault decision in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssaultStart {
    revision: u64,
    resources: [u8; 4],
    pieces: Vec<Vec<Group>>,
    available: Vec<Group>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssaultResolution {
    pub paid: u8,
    pub returned: u8,
}

impl AssaultResolution {
    pub(crate) fn removal_summary(&self) -> String {
        format!(
            "Returned {} Active 26July {}",
            self.returned,
            if self.returned == 1 {
                "Guerrilla"
            } else {
                "Guerrillas"
            },
        )
    }
}

impl Game {
    pub(super) fn assault_result(&self) -> Option<AssaultResolution> {
        self.assault.as_ref()?.resolution.clone()
    }

    pub(super) fn assault_finished(&self) -> bool {
        self.assault.as_ref().is_some_and(|op| op.finished)
    }

    pub(super) fn assault_opening(&self) -> Choice {
        let alliance = UsAlliance::Firm;
        let balance = self.resources[Faction::Government.index()];
        let cost = alliance.coin_cost();
        let text = format!(
            "{} US Alliance: {} per space (R 6.3.1). Government has {}.",
            alliance.name(),
            resources(cost),
            resources(balance)
        );
        self.offer(
            OPEN,
            "Assault · Operation Only",
            balance >= cost,
            if balance >= cost {
                format!("{text} Inspect Santiago before paying. Completing an Operation Only gives Syndicate the second choice (R 2.3.4, 3.2.4).")
            } else {
                format!("{} US Alliance: {} per space (R 6.3.1). Government has {} and cannot pay.", alliance.name(), resources(cost), resources(balance))
            },
        )
    }

    pub(super) fn assault_choices(&self) -> Vec<Choice> {
        if self.assault_finished() {
            return vec![
                self.offer("syndicate:pass", "Pass", false, "Syndicate may Pass for 1 Resource, up to 49, and stays Eligible. Available for inspection (R 1.8, 2.3.3)."),
                self.offer("syndicate:limited-operation", "Limited Operation", false, "Syndicate may choose one space and has no Special Activity. Inspect only (R 2.3.4–5)."),
            ];
        }
        if let Some(choices) = self.transfer_choices() {
            return choices;
        }
        if self.removal.is_some() {
            return self.removal_menu();
        }
        let side_choices = [
            self.offer(
                "transfer",
                "Resource transfer",
                true,
                "Government may give Resources or ask another faction for Resources. The giver decides how much to transfer (R 1.5.1, 1.8).",
            ),
            self.offer(
                "remove",
                "Remove own pieces",
                true,
                "Choose Government pieces on the map to return to Available Forces (R 1.4.1). Assault uses the remaining cubes.",
            ),
        ];
        if let Some(result) = self.assault_result() {
            return [self.offer(
                FINISH,
                "Finish Assault",
                true,
                format!("Santiago paid {}. {} to Available Forces. Finish to record Operation Only and give Syndicate Pass or Limited Operation as 2nd Eligible (R 2.3.4).", resources(result.paid), result.removal_summary()),
            )].into_iter().chain(side_choices).collect();
        }
        let santiago = self.santiago();
        let troops = count(santiago, Faction::Government, Kind::Troops, State::Ordinary);
        let police = count(santiago, Faction::Government, Kind::Police, State::Ordinary);
        let active = count(santiago, Faction::M26, Kind::Guerrilla, State::Active);
        let underground = count(santiago, Faction::M26, Kind::Guerrilla, State::Underground);
        let cost = UsAlliance::Firm.coin_cost();
        let balance = self.resources[Faction::Government.index()];
        let funded = balance >= cost;
        let payment = if funded {
            format!(
                "Government pays {} ({balance} → {}).",
                resources(cost),
                balance - cost
            )
        } else {
            format!(
                "Government has {} and cannot pay {}.",
                resources(balance),
                resources(cost)
            )
        };
        [self.offer(
            SANTIAGO,
            "Santiago · target 26July",
            funded,
            format!("{troops} Troops + {police} Police = {} removal capacity in a City (R 3.2.4). 26July has {active} Active, {underground} Underground Guerrillas. {payment} Return {} Active to Available Forces.", troops + police, active.min(troops + police)),
        )].into_iter().chain(side_choices).collect()
    }

    pub(super) fn assault_next(&self) -> Option<Next> {
        self.assault_finished().then(|| Next {
            faction: Faction::Syndicate,
            role: "2nd Eligible".into(),
            options: vec!["Pass".into(), "Limited Operation".into()],
        })
    }

    pub(super) fn apply_assault(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        match action {
            OPEN => {
                self.assault = Some(Assault {
                    start: AssaultStart {
                        revision: self.revision,
                        resources: self.resources,
                        pieces: self.spaces.iter().map(|s| s.pieces.clone()).collect(),
                        available: self.available.clone(),
                    },
                    actions: vec![],
                    resolution: None,
                    finished: false,
                });
                return Ok(());
            }
            SANTIAGO => {
                let space = self.santiago();
                let capacity = count(space, Faction::Government, Kind::Troops, State::Ordinary)
                    + count(space, Faction::Government, Kind::Police, State::Ordinary);
                let returned =
                    capacity.min(count(space, Faction::M26, Kind::Guerrilla, State::Active));
                let paid = UsAlliance::Firm.coin_cost();
                self.resources[Faction::Government.index()] -= paid;
                if returned > 0 {
                    let pieces = &mut self
                        .spaces
                        .iter_mut()
                        .find(|s| s.id == "santiago")
                        .unwrap()
                        .pieces;
                    let active = pieces
                        .iter_mut()
                        .find(|p| {
                            p.faction == Faction::M26
                                && p.kind == Kind::Guerrilla
                                && p.state == State::Active
                        })
                        .unwrap();
                    active.count -= returned;
                    pieces.retain(|p| p.count > 0);
                    self.available
                        .iter_mut()
                        .find(|p| p.faction == Faction::M26 && p.kind == Kind::Guerrilla)
                        .unwrap()
                        .count += returned;
                }
                self.assault.as_mut().unwrap().resolution =
                    Some(AssaultResolution { paid, returned });
            }
            FINISH => self.assault.as_mut().unwrap().finished = true,
            _ if self.transfer.is_some()
                || self.removal.is_some()
                || matches!(action, "transfer" | "remove") =>
            {
                self.execute(action, next_revision)?;
            }
            _ => return Err("Unsupported Assault decision.".into()),
        }
        self.assault.as_mut().unwrap().actions.push(action.into());
        Ok(())
    }

    pub(super) fn validate_assault(&self, assault: &Assault) -> Result<(), String> {
        let start = &assault.start;
        if !self.card_18
            || start.pieces.len() != self.spaces.len()
            || start
                .revision
                .checked_add(1)
                .and_then(|rev| rev.checked_add(assault.actions.len() as u64))
                != Some(self.revision)
        {
            return Err("Unsupported Assault history.".into());
        }
        let mut before = self.clone();
        before.assault = None;
        before.transfer = None;
        before.removal = None;
        before.revision = start.revision;
        before.resources = start.resources;
        before.available.clone_from(&start.available);
        for (space, pieces) in before.spaces.iter_mut().zip(&start.pieces) {
            space.pieces.clone_from(pieces);
        }
        before.validate()?;
        before.apply(OPEN)?;
        for action in &assault.actions {
            before.apply(action)?;
        }
        if before != *self {
            return Err("Unsupported Assault state.".into());
        }
        Ok(())
    }

    fn santiago(&self) -> &Space {
        self.spaces.iter().find(|s| s.id == "santiago").unwrap()
    }
}

fn count(space: &Space, faction: Faction, kind: Kind, state: State) -> u8 {
    space
        .pieces
        .iter()
        .find(|p| p.faction == faction && p.kind == kind && p.state == state)
        .map_or(0, |p| p.count)
}
