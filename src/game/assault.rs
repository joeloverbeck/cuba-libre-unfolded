//! Government's Operation Only Assault on Pact of Caracas (R 2.3.4, 3.1, 3.2.4).
use super::*;

const OPEN: &str = "government:assault";
const FINISH: &str = "end-assault";
/// The plan id of the one plan that removes nothing: `assault:<space>:none`.
const NOTHING: &str = "none";
const ENEMIES: [Faction; 3] = [Faction::M26, Faction::Directorio, Faction::Syndicate];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Assault {
    /// What Government paid in each space it Assaulted and what was removed or closed
    /// there, in the order paid. A space may be selected once (R 3.1), so a paid
    /// space offers no plan. Where the player looks on the map is not recorded (#169).
    paid: Vec<AssaultResolution>,
    pub(super) finished: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssaultResolution {
    /// The paid space's id.
    pub space: String,
    pub paid: u8,
    pub returned: u8,
    detail: AssaultDetail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssaultDetail {
    space: String,
    removed: Vec<Group>,
    closed: u8,
}

impl AssaultResolution {
    pub(crate) fn space_name(&self) -> &str {
        &self.detail.space
    }

    /// The record of this space: its name, what it paid and what was removed or
    /// closed, such as "Havana · paid 2 Resources · closed 1 Casino".
    pub fn summary(&self) -> String {
        format!(
            "{} · paid {} · {}",
            self.detail.space,
            resources(self.paid),
            self.detail.record()
        )
    }
}

impl AssaultDetail {
    fn effect(&self, resolved: bool) -> String {
        let mut lines = self
            .removed
            .iter()
            .map(|group| {
                let name = match group.kind {
                    Kind::Guerrilla => plural(group.count, "Active Guerrilla", "Active Guerrillas"),
                    _ => plural(group.count, "Base", "Bases"),
                };
                let verb = if resolved { "Returned" } else { "Return" };
                format!(
                    "{verb} {} {} {name} to Available Forces",
                    group.count,
                    group.faction.name()
                )
            })
            .collect::<Vec<_>>();
        if self.closed > 0 {
            let casinos = plural(self.closed, "Casino", "Casinos");
            let verb = if resolved { "Closed" } else { "Close" };
            lines.push(format!(
                "{verb} {} {casinos} in place (R 1.4.5)",
                self.closed
            ));
        }
        if lines.is_empty() {
            return "No pieces affected".into();
        }
        lines.join(". ")
    }

    /// What a paid space's record says was removed or closed.
    fn record(&self) -> String {
        let returned = self
            .removed
            .iter()
            .map(|group| {
                let name = match group.kind {
                    Kind::Guerrilla => plural(group.count, "Guerrilla", "Guerrillas"),
                    _ => plural(group.count, "Base", "Bases"),
                };
                format!("{} {} {name}", group.count, group.faction.name())
            })
            .collect::<Vec<_>>();
        let mut parts = vec![];
        if !returned.is_empty() {
            parts.push(format!("returned {}", returned.join(", ")));
        }
        if self.closed > 0 {
            parts.push(format!(
                "closed {} {}",
                self.closed,
                plural(self.closed, "Casino", "Casinos")
            ));
        }
        if parts.is_empty() {
            return "removed nothing".into();
        }
        parts.join(", ")
    }
}

struct Target {
    faction: Faction,
    active: u8,
    underground: u8,
    bases: u8,
    closed: u8,
}

impl Target {
    fn in_space(space: &Space, faction: Faction) -> Self {
        let bases = if faction == Faction::Syndicate {
            count(space, faction, Kind::Casino, State::Open)
        } else {
            count(space, faction, Kind::Base, State::Ordinary)
        };
        Self {
            faction,
            active: count(space, faction, Kind::Guerrilla, State::Active),
            underground: count(space, faction, Kind::Guerrilla, State::Underground),
            bases,
            closed: count(space, faction, Kind::Casino, State::Closed),
        }
    }

    fn present(&self) -> bool {
        self.active + self.underground + self.bases + self.closed > 0
    }

    fn available(&self) -> u8 {
        self.active + if self.underground == 0 { self.bases } else { 0 }
    }

    fn protection(&self) -> String {
        let faction = self.faction.name();
        let mut text = format!(
            "{faction}: {} Active, {} Underground",
            self.active, self.underground
        );
        if self.bases > 0 {
            let name = if self.faction == Faction::Syndicate {
                plural(self.bases, "open Casino", "open Casinos")
            } else {
                plural(self.bases, "Base", "Bases")
            };
            text.push_str(&format!(", {} {name}", self.bases));
        }
        if self.underground > 0 {
            text.push_str(". Underground Guerrillas survive");
            if self.bases > 0 {
                text.push_str(" and protect their faction's Bases");
            }
        } else if self.bases > 0 && self.active > 0 {
            text.push_str(". Bases are affected after this faction's Guerrillas");
        }
        if self.closed > 0 {
            text.push_str(". Closed Casinos stay closed");
        }
        text.push('.');
        text
    }
}

struct Plan {
    action: String,
    label: String,
    protection: String,
    detail: AssaultDetail,
}

impl Plan {
    /// Whether the plan removes or closes at least one piece.
    fn effect(&self) -> bool {
        !self.detail.removed.is_empty() || self.detail.closed > 0
    }
}

/// What Government may do in `space` (#169): every choice of targeted factions, and
/// of a short capacity's division among them, that removes or closes at least one
/// piece (R 3.1, 3.2.4), then the one plan that removes nothing. R 3.2.4 lets
/// Government select any space, so that plan is offered in every space; where no
/// other plan exists it is the only one. Every way of removing nothing pays the
/// same cost and changes nothing else, so one plan stands for them all.
fn plans(space: &Space) -> Vec<Plan> {
    let targets: Vec<_> = ENEMIES
        .into_iter()
        .map(|faction| Target::in_space(space, faction))
        .filter(Target::present)
        .collect();
    let capacity = capacity(space).0;
    let mut plans: Vec<Plan> = (1..1 << targets.len())
        .flat_map(|mask| {
            let selected: Vec<_> = targets
                .iter()
                .enumerate()
                .filter_map(|(index, target)| (mask & (1 << index) != 0).then_some(target))
                .collect();
            let limits: Vec<_> = selected.iter().map(|target| target.available()).collect();
            let maximum = capacity.min(limits.iter().sum());
            let allocations = allocations(&limits, maximum);
            let competing = allocations.len() > 1;
            allocations
                .iter()
                .map(|allocation| plan(space, &selected, allocation, competing))
                .collect::<Vec<_>>()
        })
        .filter(Plan::effect)
        .collect();
    let why = if targets.is_empty() {
        "No enemy pieces are there.".to_owned()
    } else if plans.is_empty() {
        targets
            .iter()
            .map(Target::protection)
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        "This plan targets no faction with a piece Assault can affect here (R 3.1).".to_owned()
    };
    plans.push(Plan {
        action: format!("assault:{}:{NOTHING}", space.id),
        label: "Removes nothing".into(),
        protection: why,
        detail: AssaultDetail {
            space: space.name.clone(),
            removed: vec![],
            closed: 0,
        },
    });
    plans
}

/// R 1.4.1 and 3.1 require the maximum total across the targeted factions.
/// The player can divide a short capacity among those factions.
fn allocations(limits: &[u8], remaining: u8) -> Vec<Vec<u8>> {
    let Some((&limit, rest)) = limits.split_first() else {
        return vec![vec![]];
    };
    let least = remaining.saturating_sub(rest.iter().sum());
    let most = remaining.min(limit);
    (least..=most)
        .flat_map(|affected| {
            allocations(rest, remaining - affected)
                .into_iter()
                .map(move |mut suffix| {
                    suffix.insert(0, affected);
                    suffix
                })
        })
        .collect()
}

fn plan(space: &Space, targets: &[&Target], allocation: &[u8], competing: bool) -> Plan {
    let mut removed = vec![];
    let mut closed = 0;
    for (target, &affected) in targets.iter().zip(allocation) {
        let active = target.active.min(affected);
        if active > 0 {
            removed.push(Group {
                faction: target.faction,
                kind: Kind::Guerrilla,
                state: State::Active,
                count: active,
            });
        }
        let bases = affected - active;
        if target.faction == Faction::Syndicate {
            closed = bases;
        } else if bases > 0 {
            removed.push(Group {
                faction: target.faction,
                kind: Kind::Base,
                state: State::Ordinary,
                count: bases,
            });
        }
    }
    let target_names = targets.iter().map(|t| t.faction.name()).collect::<Vec<_>>();
    let mut id = target_names.join("+").to_ascii_lowercase();
    let mut label = format!("Target {}", target_names.join(" + "));
    if competing {
        let counts: Vec<_> = allocation.iter().map(u8::to_string).collect();
        id = format!("{id}:{}", counts.join("-"));
        label = format!("{label} · {}", counts.join(" / "));
    }
    Plan {
        action: format!("assault:{}:{id}", space.id),
        label,
        protection: targets
            .iter()
            .map(|t| t.protection())
            .collect::<Vec<_>>()
            .join(" "),
        detail: AssaultDetail {
            space: space.name.clone(),
            removed,
            closed,
        },
    }
}

fn capacity(space: &Space) -> (u8, String) {
    let troops = count(space, Faction::Government, Kind::Troops, State::Ordinary);
    let police = count(space, Faction::Government, Kind::Police, State::Ordinary);
    let (limit, arithmetic) = match space.terrain {
        Terrain::City => (
            troops + police,
            format!(
                "{troops} Troops + {police} Police = {} in a City",
                troops + police
            ),
        ),
        Terrain::EconomicCenter => (
            troops + police,
            format!(
                "{troops} Troops + {police} Police = {} in an Economic Center",
                troops + police
            ),
        ),
        Terrain::Mountain => (
            troops / 2,
            format!(
                "{troops} Troops ÷ 2 = {} in Mountain (round down). {police} Police do not count",
                troops / 2
            ),
        ),
        terrain => {
            let name = if terrain == Terrain::Forest {
                "Forest"
            } else {
                "Grassland"
            };
            (
                troops,
                format!("{troops} Troops = {troops} in {name}. {police} Police do not count"),
            )
        }
    };
    (limit, format!("{arithmetic} (R 3.2.4)."))
}

fn plural<'a>(count: u8, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

fn payment_action(action: &str) -> Option<&str> {
    let (space, _) = action.strip_prefix("assault:")?.split_once(':')?;
    Some(space)
}

impl Game {
    /// What each paid space paid and what was removed or closed there, in the order paid.
    pub(super) fn assault_paid(&self) -> Vec<AssaultResolution> {
        self.assault
            .as_ref()
            .map_or_else(Vec::new, |assault| assault.paid.clone())
    }

    /// What the preview of paying for `action` states besides its computed changes:
    /// the capacity arithmetic, whom the plan can affect, what it removes or closes,
    /// and whether Government could then pay for another space. `self` is the match
    /// before the payment and `after` the copy it was applied to.
    pub(super) fn assault_preview_notes(&self, after: &Game, action: &str) -> Vec<String> {
        let mut notes = vec![];
        let space = payment_action(action).and_then(|id| self.spaces.iter().find(|s| s.id == id));
        if let Some(space) = space {
            notes.push(capacity(space).1);
            if let Some(plan) = plans(space).into_iter().find(|plan| plan.action == action) {
                notes.push(plan.protection);
            }
        }
        let Some(assault) = &after.assault else {
            return notes;
        };
        if let Some(paid) = assault.paid.last() {
            notes.push(format!("R 3.2.4: {}.", paid.detail.effect(false)));
        }
        let balance = after.resources[Faction::Government.index()];
        let cost = UsAlliance::Firm.coin_cost();
        if balance < cost && assault.paid.len() < after.spaces.len() {
            notes.push(format!(
                "Government would have {}: another space would need a Resource transfer first (R 6.3.1).",
                resources(balance)
            ));
        }
        notes
    }

    pub(super) fn assault_finished(&self) -> bool {
        self.assault.as_ref().is_some_and(|op| op.finished)
    }

    pub(super) fn assault_opening(&self) -> Choice {
        let alliance = UsAlliance::Firm;
        let balance = self.resources[Faction::Government.index()];
        let cost = alliance.coin_cost();
        let cost_line = format!(
            "{} US Alliance: {} per space (R 6.3.1). Government has {}",
            alliance.name(),
            resources(cost),
            resources(balance)
        );
        let affordability = if balance >= cost {
            ". Choose each space on the map; its popover shows the plans before paying."
        } else {
            " and cannot pay for a space. Look at the map or finish without paying."
        };
        self.offer(
            OPEN,
            "Assault · Operation Only",
            true,
            format!("{cost_line}{affordability} A zero-space Assault earns no Resources (local policy). Syndicate then decides as 2nd Eligible (R 2.3.4)."),
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
        let mut choices = self.assault_plans();
        choices.extend(self.assault_steps());
        choices
    }

    /// The plans of every space not yet paid, in the map's space order (#169). Each
    /// states its space and whether it removes or closes a piece. A plan is supported
    /// when Government can pay the space's cost; otherwise it gives that reason and,
    /// while another faction has Resources to give, names the Resource transfer that
    /// would open it, as a Terror target does.
    fn assault_plans(&self) -> Vec<Choice> {
        let assault = self.assault.as_ref().unwrap();
        let balance = self.resources[Faction::Government.index()];
        let cost = UsAlliance::Firm.coin_cost();
        let funded = balance >= cost;
        let transfer_helps = !funded
            && ENEMIES
                .iter()
                .any(|faction| self.resources[faction.index()] > 0);
        self.spaces
            .iter()
            .filter(|space| assault.paid.iter().all(|paid| paid.space != space.id))
            .flat_map(|space| {
                let arithmetic = capacity(space).1;
                plans(space).into_iter().map(move |plan| {
                    let effect = plan.detail.effect(false);
                    let explanation = if funded {
                        format!(
                            "{arithmetic} {} Government pays {} ({balance} → {}). {effect}.",
                            plan.protection,
                            resources(cost),
                            balance - cost
                        )
                    } else {
                        format!(
                            "Government has {} and cannot pay {} (R 6.3.1). {arithmetic} {} {effect}.",
                            resources(balance),
                            resources(cost),
                            plan.protection
                        )
                    };
                    let mut choice = self
                        .offer(&plan.action, plan.label.as_str(), funded, explanation)
                        .at(&space.id, plan.effect());
                    if transfer_helps {
                        choice.remedies.push("Request a Resource transfer".to_owned());
                    }
                    choice
                })
            })
            .collect()
    }

    /// The steps that select no space: the ending, a Resource transfer and voluntary removal.
    fn assault_steps(&self) -> Vec<Choice> {
        let assault = self.assault.as_ref().unwrap();
        let balance = self.resources[Faction::Government.index()];
        let result = if assault.paid.is_empty() {
            "No spaces selected. A zero-space Assault costs nothing and earns no Pass Resources (local policy).".to_owned()
        } else {
            format!(
                "{}. Government has {} left.",
                assault
                    .paid
                    .iter()
                    .map(|paid| format!(
                        "{} · paid {}. {}",
                        paid.space_name(),
                        resources(paid.paid),
                        paid.detail.effect(true)
                    ))
                    .collect::<Vec<_>>()
                    .join(". "),
                resources(balance),
            )
        };
        vec![
            self.offer(FINISH, "Finish Assault", true, format!(
                "{result} Finish to record Operation Only and give Syndicate Pass or Limited Operation as 2nd Eligible (R 2.3.4)."
            )),
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
        ]
    }

    pub(super) fn assault_next(&self) -> Option<Next> {
        self.assault_finished().then(|| Next {
            faction: Faction::Syndicate,
            role: "2nd Eligible".into(),
            options: vec!["Pass".into(), "Limited Operation".into()],
        })
    }

    pub(crate) fn assault_payment_space(action: &str) -> Option<&str> {
        payment_action(action)
    }

    pub(super) fn apply_assault(&mut self, action: &str, next_revision: u64) -> Result<(), String> {
        match action {
            OPEN => {
                self.assault = Some(Assault {
                    paid: vec![],
                    finished: false,
                });
            }
            FINISH => self.assault.as_mut().unwrap().finished = true,
            _ if self.transfer.is_some()
                || self.removal.is_some()
                || matches!(action, "transfer" | "remove") =>
            {
                self.execute(action, next_revision)?;
            }
            _ => self.resolve_assault(action)?,
        }
        Ok(())
    }

    fn resolve_assault(&mut self, action: &str) -> Result<(), String> {
        let id = payment_action(action).ok_or("Unsupported Assault decision.")?;
        let space = self
            .spaces
            .iter_mut()
            .find(|space| space.id == id)
            .ok_or("Unknown Assault space.")?;
        let plan = plans(space)
            .into_iter()
            .find(|plan| plan.action == action)
            .ok_or("Unknown Assault targets.")?;
        let paid = UsAlliance::Firm.coin_cost();
        self.resources[Faction::Government.index()] -= paid;
        for returned in &plan.detail.removed {
            let piece = space
                .pieces
                .iter_mut()
                .find(|p| {
                    p.faction == returned.faction
                        && p.kind == returned.kind
                        && p.state == returned.state
                })
                .unwrap();
            piece.count -= returned.count;
            self.available
                .iter_mut()
                .find(|p| p.faction == returned.faction && p.kind == returned.kind)
                .unwrap()
                .count += returned.count;
        }
        let closed = plan.detail.closed;
        if closed > 0 {
            space
                .pieces
                .iter_mut()
                .find(|p| p.kind == Kind::Casino && p.state == State::Open)
                .unwrap()
                .count -= closed;
            if let Some(piece) = space
                .pieces
                .iter_mut()
                .find(|p| p.kind == Kind::Casino && p.state == State::Closed)
            {
                piece.count += closed;
            } else {
                space.pieces.push(Group {
                    faction: Faction::Syndicate,
                    kind: Kind::Casino,
                    state: State::Closed,
                    count: closed,
                });
            }
        }
        space.pieces.retain(|p| p.count > 0);
        self.assault.as_mut().unwrap().paid.push(AssaultResolution {
            space: id.to_owned(),
            paid,
            returned: plan.detail.removed.iter().map(|group| group.count).sum(),
            detail: plan.detail,
        });
        Ok(())
    }
}

fn count(space: &Space, faction: Faction, kind: Kind, state: State) -> u8 {
    space
        .pieces
        .iter()
        .find(|p| p.faction == faction && p.kind == kind && p.state == state)
        .map_or(0, |p| p.count)
}
