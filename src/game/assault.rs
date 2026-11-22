//! Government's Operation Only Assault on Pact of Caracas (R 2.3.4, 3.1, 3.2.4).
use super::*;

const OPEN: &str = "government:assault";
const FINISH: &str = "end-assault";
const BACK: &str = "assault:back";
const LEGACY_SANTIAGO: &str = "assault:santiago:26july";
const ENEMIES: [Faction; 3] = [Faction::M26, Faction::Directorio, Faction::Syndicate];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Assault {
    start: AssaultStart,
    actions: Vec<String>,
    /// Choices in completed menu loops. Older records retain every choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skipped_choices: Option<u64>,
    resolution: Option<AssaultResolution>,
    pub(super) finished: bool,
    #[serde(default)]
    browse: Browse,
}

impl Assault {
    fn record_action(&mut self, action: &str) {
        if self.skipped_choices.is_none() {
            self.skipped_choices = Some(0);
            for earlier in std::mem::take(&mut self.actions) {
                self.retain_action(earlier);
            }
        }
        self.retain_action(action.into());
    }

    /// Drop closed loops that return to the same decision, including waits that
    /// have been reopened. Keep open menus and every committed effect in order.
    fn retain_action(&mut self, action: String) {
        let remove = match (action.as_str(), self.actions.as_slice()) {
            (BACK, [.., open])
                if open.starts_with("assault:areas:") || open.starts_with("assault:space:") =>
            {
                1
            }
            ("back", [.., open]) if open == "transfer" => 1,
            ("reopen", [.., wait]) if wait == "wait" => 1,
            ("done-removal", [.., open]) if open == "remove" => 1,
            (give, [.., open])
                if open == "transfer" && give.starts_with("give:") && give.ends_with(":0") =>
            {
                1
            }
            ("decline" | "authorize:0", [.., open, ask, wait])
                if open == "transfer" && ask.starts_with("ask:") && wait == "wait" =>
            {
                3
            }
            ("decline" | "authorize:0", [.., open, ask])
                if open == "transfer" && ask.starts_with("ask:") =>
            {
                2
            }
            _ => 0,
        };
        if remove == 0 {
            self.actions.push(action);
        } else {
            self.actions.truncate(self.actions.len() - remove);
            *self.skipped_choices.as_mut().unwrap() += remove as u64 + 1;
        }
    }
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
enum Browse {
    #[default]
    Areas,
    Spaces(Area),
    Targets(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Area {
    Cities,
    Provinces,
    EconomicCenters,
}

impl Area {
    const ALL: [Self; 3] = [Self::Cities, Self::Provinces, Self::EconomicCenters];

    fn for_space(space: &Space) -> Self {
        match space.terrain {
            Terrain::City => Self::Cities,
            Terrain::EconomicCenter => Self::EconomicCenters,
            _ => Self::Provinces,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Cities => "cities",
            Self::Provinces => "provinces",
            Self::EconomicCenters => "economic-centers",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Cities => "Cities",
            Self::Provinces => "Provinces",
            Self::EconomicCenters => "Economic Centers",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssaultResolution {
    pub paid: u8,
    pub returned: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    detail: Option<AssaultDetail>,
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
        self.detail.as_ref().map_or("Santiago", |d| &d.space)
    }

    pub(crate) fn removal_summary(&self) -> String {
        self.detail.as_ref().map_or_else(
            || {
                format!(
                    "Returned {} Active 26July {}",
                    self.returned,
                    plural(self.returned, "Guerrilla", "Guerrillas")
                )
            },
            |detail| detail.effect(true),
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

fn plans(space: &Space) -> Vec<Plan> {
    let targets: Vec<_> = ENEMIES
        .into_iter()
        .map(|faction| Target::in_space(space, faction))
        .filter(Target::present)
        .collect();
    let capacity = capacity(space).0;
    if targets.is_empty() {
        return vec![plan(space, &[], &[], false)];
    }
    (1..1 << targets.len())
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
        .collect()
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
    let (mut id, mut label) = if target_names.is_empty() {
        ("none".into(), "No enemy pieces".into())
    } else {
        (
            target_names.join("+").to_ascii_lowercase(),
            target_names.join(" + "),
        )
    };
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
    (!matches!(space, "areas" | "space")).then_some(space)
}

impl Game {
    pub(super) fn assault_result(&self) -> Option<AssaultResolution> {
        self.assault.as_ref()?.resolution.clone()
    }

    pub(super) fn assault_preview_effect(&self) -> Option<String> {
        let detail = self
            .assault
            .as_ref()?
            .resolution
            .as_ref()?
            .detail
            .as_ref()?;
        Some(format!("R 3.2.4: {}.", detail.effect(false)))
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
            ". Inspect spaces and targets before paying."
        } else {
            " and cannot pay for a space. Inspect spaces or finish without paying."
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
        self.assault_browse_choices()
    }

    fn assault_browse_choices(&self) -> Vec<Choice> {
        let assault = self.assault.as_ref().unwrap();
        let unselected = |space: &&Space| {
            !assault
                .actions
                .iter()
                .any(|action| payment_action(action) == Some(&space.id))
        };
        let balance = self.resources[Faction::Government.index()];
        let cost = UsAlliance::Firm.coin_cost();
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
        match &assault.browse {
            Browse::Areas => {
                let mut choices: Vec<_> = Area::ALL.into_iter().filter(|&area| {
                    self.spaces.iter().filter(unselected).any(|space| Area::for_space(space) == area)
                }).map(|area| self.offer(
                    &format!("assault:areas:{}", area.id()),
                    area.label(),
                    true,
                    format!("Inspect the remaining {} and choose where to Assault. {payment} A space may be selected once (R 3.1).", area.label().to_lowercase()),
                )).collect();
                let result = assault.resolution.as_ref().map_or_else(
                    || "No spaces selected. A zero-space Assault costs nothing and earns no Pass Resources (local policy).".into(),
                    |result| format!(
                        "{} · paid {}. {}. Government has {} left.",
                        result.space_name(),
                        resources(result.paid),
                        result.removal_summary(),
                        resources(balance),
                    ),
                );
                choices.push(self.offer(FINISH, "Finish Assault", true, format!(
                    "{result} Finish to record Operation Only and give Syndicate Pass or Limited Operation as 2nd Eligible (R 2.3.4)."
                )));
                choices.extend([
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
                ]);
                choices
            }
            Browse::Spaces(area) => {
                let mut choices: Vec<_> = self
                    .spaces
                    .iter()
                    .filter(unselected)
                    .filter(|space| Area::for_space(space) == *area)
                    .map(|space| {
                        self.offer(
                            &format!("assault:space:{}", space.id),
                            &space.name,
                            true,
                            format!(
                                "{} {payment} Inspect target and piece choices before paying.",
                                capacity(space).1
                            ),
                        )
                    })
                    .collect();
                choices.push(self.offer(
                    BACK,
                    "Back to all areas",
                    true,
                    "Choose another area of the board.",
                ));
                choices
            }
            Browse::Targets(id) => {
                let space = self
                    .spaces
                    .iter()
                    .filter(unselected)
                    .find(|space| space.id == *id)
                    .unwrap();
                let arithmetic = capacity(space).1;
                let mut choices: Vec<_> = plans(space)
                    .into_iter()
                    .map(|plan| {
                        self.offer(
                            &plan.action,
                            plan.label,
                            funded,
                            format!(
                                "{arithmetic} {} {payment} {}.",
                                plan.protection,
                                plan.detail.effect(false)
                            ),
                        )
                    })
                    .collect();
                choices.push(self.offer(
                    BACK,
                    "Back to spaces",
                    true,
                    "Choose a different space before paying.",
                ));
                choices
            }
        }
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
                    start: AssaultStart {
                        revision: self.revision,
                        resources: self.resources,
                        pieces: self.spaces.iter().map(|s| s.pieces.clone()).collect(),
                        available: self.available.clone(),
                    },
                    actions: vec![],
                    skipped_choices: Some(0),
                    resolution: None,
                    finished: false,
                    browse: Browse::Areas,
                });
                return Ok(());
            }
            FINISH => self.assault.as_mut().unwrap().finished = true,
            BACK => {
                let browse = &mut self.assault.as_mut().unwrap().browse;
                *browse = match browse {
                    Browse::Targets(id) => Browse::Spaces(Area::for_space(
                        self.spaces.iter().find(|s| s.id == *id).unwrap(),
                    )),
                    _ => Browse::Areas,
                };
            }
            _ if action.starts_with("assault:areas:") => {
                let area = Area::ALL
                    .into_iter()
                    .find(|area| action.ends_with(area.id()))
                    .ok_or("Unknown Assault area.")?;
                self.assault.as_mut().unwrap().browse = Browse::Spaces(area);
            }
            _ if action.starts_with("assault:space:") => {
                let id = action.strip_prefix("assault:space:").unwrap();
                self.assault.as_mut().unwrap().browse = Browse::Targets(id.into());
            }
            _ if self.transfer.is_some()
                || self.removal.is_some()
                || matches!(action, "transfer" | "remove") =>
            {
                self.execute(action, next_revision)?;
            }
            _ => self.resolve_assault(action)?,
        }
        self.assault.as_mut().unwrap().record_action(action);
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
        let assault = self.assault.as_mut().unwrap();
        assault.resolution = Some(AssaultResolution {
            paid,
            returned: plan.detail.removed.iter().map(|group| group.count).sum(),
            detail: Some(plan.detail),
        });
        assault.browse = Browse::Areas;
        Ok(())
    }

    pub(super) fn validate_assault(&self, assault: &Assault) -> Result<(), String> {
        let start = &assault.start;
        let skipped = assault.skipped_choices.unwrap_or(0);
        if !self.card_18
            || start.pieces.len() != self.spaces.len()
            || skipped == 1
            || start
                .revision
                .checked_add(1)
                .and_then(|rev| rev.checked_add(assault.actions.len() as u64))
                .and_then(|rev| rev.checked_add(skipped))
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
        for (index, action) in assault.actions.iter().enumerate() {
            // The Santiago-only build committed directly from the initial menu.
            let legacy = index == 0 && action == LEGACY_SANTIAGO;
            if legacy {
                before.assault.as_mut().unwrap().browse = Browse::Targets("santiago".into());
            }
            before.apply(action)?;
            if legacy
                && assault
                    .resolution
                    .as_ref()
                    .is_some_and(|resolution| resolution.detail.is_none())
            {
                before
                    .assault
                    .as_mut()
                    .unwrap()
                    .resolution
                    .as_mut()
                    .unwrap()
                    .detail = None;
            }
        }
        let replay = before.assault.as_mut().unwrap();
        if assault.skipped_choices.is_some() {
            // From Areas, opening and cancelling a transfer takes two choices.
            // Opening, asking for zero and declining takes three. Any skipped
            // count of two or more has a witness without changing the position.
            // Card 18 changes only the match revision for these choices.
            before.revision += skipped;
            *replay.skipped_choices.as_mut().unwrap() += skipped;
        } else {
            // Check every legacy choice without accumulating menu loops in the
            // replay. Preserve the admitted record until the player acts again.
            replay.actions.clone_from(&assault.actions);
            replay.skipped_choices = None;
        }
        if before != *self {
            return Err("Unsupported Assault state.".into());
        }
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
