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

    /// The space and what it paid, such as "Havana · paid 2 Resources": the one wording
    /// of a paid space that its record and Finish Assault's explanation both begin with.
    fn paid_line(&self) -> String {
        format!("{} · paid {}", self.space_name(), resources(self.paid))
    }

    /// The record of this space: its name, what it paid and what was removed or
    /// closed, such as "Havana · paid 2 Resources · closed 1 Casino".
    pub fn summary(&self) -> String {
        format!("{} · {}", self.paid_line(), self.detail.record())
    }

    /// The space as Finish Assault's explanation states it, with what was removed or
    /// closed and its rule, such as "Havana · paid 2 Resources. Closed 1 Casino in
    /// place (R 1.4.5)".
    fn finished(&self) -> String {
        format!("{}. {}", self.paid_line(), self.detail.effect(true))
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
            text.push_str(
                ". Assault cannot remove its Underground Guerrillas until they are Activated",
            );
            if self.bases > 0 {
                text.push_str(", and they protect their faction's Bases");
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
/// piece (R 3.1, 3.2.4), then the one plan that removes nothing. A plan targets only
/// factions with a piece Assault can affect there: naming another changes nothing and
/// reads as if it would (the owner's decision in #166's playtest). Each plan says why
/// such a faction's pieces stay. R 3.2.4 lets Government select any space, so the plan
/// that removes nothing is offered in every space; where no other plan exists it is the
/// only one. Every way of removing nothing pays the same cost and changes nothing else,
/// so one plan stands for them all.
fn plans(space: &Space) -> Vec<Plan> {
    let targets: Vec<_> = ENEMIES
        .into_iter()
        .map(|faction| Target::in_space(space, faction))
        .filter(Target::present)
        .collect();
    let (reachable, sheltered): (Vec<_>, Vec<_>) =
        targets.iter().partition(|target| target.available() > 0);
    let sheltered: String = sheltered
        .iter()
        .map(|target| format!(" {}", target.protection()))
        .collect();
    let capacity = capacity(space).0;
    let mut plans: Vec<Plan> = (1..1 << reachable.len())
        .flat_map(|mask| {
            let selected: Vec<_> = reachable
                .iter()
                .enumerate()
                .filter_map(|(index, target)| (mask & (1 << index) != 0).then_some(*target))
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
        .map(|mut plan| {
            plan.protection.push_str(&sheltered);
            plan
        })
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
        format!(
            "This plan targets no faction with a piece Assault can affect here (R 3.1).{sheltered}"
        )
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

/// Why Assault would remove or close nothing in `space`, as one line of the space's
/// tag (#170), or `None` where some plan would affect a piece. In order: no enemy
/// piece to target; no cube that counts (R 3.2.4: Troops; Police only in a City or
/// EC; in Mountain 1 piece for every 2 Troops); no piece the cubes can reach (an
/// Underground Guerrilla survives and keeps its faction's Bases, "Bases last", and a
/// closed Casino stays closed, R 1.4.5). The words are shortened from the plan's
/// explanation to fit one line of the tag.
fn removes_nothing(space: &Space) -> Option<String> {
    let targets: Vec<_> = ENEMIES
        .into_iter()
        .map(|faction| Target::in_space(space, faction))
        .filter(Target::present)
        .collect();
    if targets.is_empty() {
        return Some("No enemy pieces (R 3.2.4)".into());
    }
    if capacity(space).0 == 0 {
        let troops = count(space, Faction::Government, Kind::Troops, State::Ordinary);
        let police = count(space, Faction::Government, Kind::Police, State::Ordinary);
        return Some(
            match space.terrain {
                Terrain::City | Terrain::EconomicCenter => "No Troops or Police (R 3.2.4)",
                Terrain::Mountain if troops == 1 => "1 Troop: Mountain needs 2",
                _ if police > 0 => "Police count only in Cities, ECs",
                _ => "No Troops (R 3.2.4)",
            }
            .into(),
        );
    }
    if targets.iter().any(|t| t.available() > 0) {
        return None;
    }
    Some(
        if targets.iter().any(|t| t.underground > 0 && t.bases > 0) {
            "No Active Guerrilla · Bases last"
        } else if targets.iter().all(|t| t.underground == 0) {
            "Casinos stay closed (R 1.4.5)"
        } else {
            "No Active Guerrilla (R 3.2.4)"
        }
        .into(),
    )
}

fn plural<'a>(count: u8, one: &'a str, many: &'a str) -> &'a str {
    if count == 1 { one } else { many }
}

/// The most characters a description takes in the tag's line. Without an emblem words
/// run wider than the reason lines (`art/atlas/STATUS.md`, "The open decision's line"),
/// so the limit is lower than theirs.
const OUTCOME_CHARS: usize = 31;

/// What the most effective Assault in `space` would do (R 3.2.4, 1.4.5), as the space
/// tag's line: the plan that affects the most pieces, or "up to" that many where several
/// do, as when a short capacity can be divided. Where those plans differ in what they
/// affect, or the words would not fit the line, it gives the number of pieces alone.
fn best_outcome(space: &Space) -> String {
    let outcome = |plan: &Plan| {
        let of = |kind| -> u8 {
            plan.detail
                .removed
                .iter()
                .filter(|group| group.kind == kind)
                .map(|group| group.count)
                .sum()
        };
        (of(Kind::Guerrilla), of(Kind::Base), plan.detail.closed)
    };
    let total = |(guerrillas, bases, casinos): (u8, u8, u8)| guerrillas + bases + casinos;
    let all: Vec<_> = plans(space).iter().map(outcome).collect();
    let most = all.iter().map(|one| total(*one)).max().unwrap_or_default();
    let best: Vec<_> = all.into_iter().filter(|one| total(*one) == most).collect();
    let up_to = if best.len() > 1 { "up to " } else { "" };
    let first = best[0];
    let guerrillas = format!("{} {}", first.0, plural(first.0, "Guerrilla", "Guerrillas"));
    let bases = format!("{} {}", first.1, plural(first.1, "Base", "Bases"));
    let described = match first {
        _ if best.iter().any(|other| *other != first) => None,
        (_, 0, 0) => Some(format!("Can remove {up_to}{guerrillas}")),
        (0, _, 0) => Some(format!("Can remove {up_to}{bases}")),
        (0, 0, n) => Some(format!(
            "Can close {up_to}{n} {}",
            plural(n, "Casino", "Casinos")
        )),
        (_, _, 0) => Some(format!("Can remove {up_to}{guerrillas}, {bases}")),
        _ => None,
    };
    described
        .filter(|line| line.chars().count() <= OUTCOME_CHARS)
        .unwrap_or_else(|| {
            format!(
                "Can affect {up_to}{most} {}",
                plural(most, "piece", "pieces")
            )
        })
}

fn payment_action(action: &str) -> Option<&str> {
    let (space, _) = action.strip_prefix("assault:")?.split_once(':')?;
    Some(space)
}

/// What Assault costs Government in each space: the US Alliance level's COIN cost
/// (R 6.3.1).
fn assault_cost() -> u8 {
    ALLIANCE.coin_cost()
}

impl Game {
    /// What each paid space paid and what was removed or closed there, in the order paid.
    pub(super) fn assault_paid(&self) -> Vec<AssaultResolution> {
        self.assault
            .as_ref()
            .map_or_else(Vec::new, |assault| assault.paid.clone())
    }

    /// Why `space` is in its map state while the Assault is open (#170), from the
    /// plans offered there (`here`): what a paid space paid (R 3.1), what would open a
    /// space Government cannot pay for now, that nothing can pay for a space where a plan
    /// would act (unpayable), or why Assault would remove nothing there.
    /// A space where a supported plan affects a piece says what its best plan does.
    pub(super) fn assault_reason(&self, space: &Space, here: &[&Choice]) -> Option<String> {
        let assault = self.assault.as_ref()?;
        if let Some(paid) = assault.paid.iter().find(|paid| paid.space == space.id) {
            return Some(format!("Assaulted · paid {}", resources(paid.paid)));
        }
        let effective: Vec<_> = here
            .iter()
            .filter(|c| c.site.as_ref().is_some_and(|site| site.effect))
            .collect();
        if effective.iter().any(|c| c.supported) {
            return Some(best_outcome(space));
        }
        if let Some(blocked) = effective.iter().find(|c| !c.remedies.is_empty()) {
            return Some(remedy_line(&blocked.remedies));
        }
        if !effective.is_empty() {
            return Some(format!(
                "Government cannot pay {}",
                resources(assault_cost())
            ));
        }
        removes_nothing(space)
    }

    /// What the preview of paying for `action` states besides its computed changes, or
    /// `None` when `action` pays for no Assault plan: the capacity arithmetic, whom the
    /// plan can affect, and what it removes or closes. `self` is the match before the
    /// payment and `after` the copy it was applied to.
    pub(super) fn assault_preview(&self, after: &Game, action: &str) -> Option<AssaultPreview> {
        let space =
            payment_action(action).and_then(|id| self.spaces.iter().find(|s| s.id == id))?;
        let plan = plans(space)
            .into_iter()
            .find(|plan| plan.action == action)?;
        let paid = after.assault.as_ref()?.paid.last()?;
        Some(AssaultPreview {
            arithmetic: capacity(space).1,
            affects: plan.protection,
            effect: format!("R 3.2.4: {}.", paid.detail.effect(false)),
        })
    }

    /// The facts an Assault payment's preview states besides [`Self::assault_preview`]:
    /// whether Government could then pay for another space, and when it could not,
    /// whether a Resource transfer could fund one (R 6.3.1).
    pub(super) fn assault_preview_notes(after: &Game) -> Vec<String> {
        let mut notes = vec![];
        let Some(assault) = &after.assault else {
            return notes;
        };
        let balance = after.resources[Faction::Government.index()];
        let cost = assault_cost();
        if balance < cost && assault.paid.len() < after.spaces.len() {
            let rest = if after.balance_after_transfers(Faction::Government) >= cost {
                "another space would need a Resource transfer first"
            } else {
                "it could not pay for another space"
            };
            notes.push(format!(
                "Government would have {}: {rest} (R 6.3.1).",
                resources(balance)
            ));
        }
        notes
    }

    pub(super) fn assault_finished(&self) -> bool {
        self.assault.as_ref().is_some_and(|op| op.finished)
    }

    pub(super) fn assault_opening(&self) -> Choice {
        let alliance = ALLIANCE;
        let balance = self.resources[Faction::Government.index()];
        let cost = assault_cost();
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
    /// while the other factions could give Government enough to pay it, names the
    /// Resource transfer that would open it, as a Terror target does.
    fn assault_plans(&self) -> Vec<Choice> {
        let assault = self.assault.as_ref().unwrap();
        let balance = self.resources[Faction::Government.index()];
        let cost = assault_cost();
        let funded = balance >= cost;
        let transfer_helps = !funded && self.balance_after_transfers(Faction::Government) >= cost;
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
                        .at(&space.id, plan.effect())
                        .stopped_by_cost(!funded && !transfer_helps);
                    if transfer_helps {
                        choice.remedies.push(TRANSFER_REMEDY.to_owned());
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
                    .map(AssaultResolution::finished)
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
        let paid = assault_cost();
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

#[cfg(test)]
mod reason_tests {
    use super::*;

    /// The most characters a space tag's line has held. Every wording measured at most
    /// 232 of the line's 248 reference pixels in Lato Regular at 17 (`art/atlas/STATUS.md`,
    /// "The open decision's line"); this keeps a longer wording from slipping in unmeasured.
    const REASON_CHARS: usize = 33;

    fn space(terrain: Terrain, pieces: &[(Faction, Kind, State, u8)]) -> Space {
        Space {
            id: "probe".into(),
            name: "Probe".into(),
            terrain,
            population: 1,
            economy: 0,
            support: 0,
            pieces: pieces
                .iter()
                .map(|&(faction, kind, state, count)| Group {
                    faction,
                    kind,
                    state,
                    count,
                })
                .collect(),
            terror: 0,
        }
    }

    const TROOPS: (Faction, Kind, State) = (Faction::Government, Kind::Troops, State::Ordinary);
    const POLICE: (Faction, Kind, State) = (Faction::Government, Kind::Police, State::Ordinary);

    fn with(group: (Faction, Kind, State), count: u8) -> (Faction, Kind, State, u8) {
        (group.0, group.1, group.2, count)
    }

    /// R 3.1, 3.2.4 worked by hand: Grassland, 2 Troops = capacity 2. 26July has 3 Active
    /// Guerrillas and Directorio 2; either alone, or both with the capacity split. The
    /// Syndicate's Underground Guerrilla shelters it and its Casino, so no plan targets
    /// it and every plan says why its pieces stay.
    #[test]
    fn plans_target_and_split_only_the_factions_assault_can_affect() {
        let active = |faction, count| (faction, Kind::Guerrilla, State::Active, count);
        let probe = space(
            Terrain::Grassland,
            &[
                with(TROOPS, 2),
                active(Faction::M26, 3),
                active(Faction::Directorio, 2),
                (Faction::Syndicate, Kind::Guerrilla, State::Underground, 1),
                (Faction::Syndicate, Kind::Casino, State::Open, 1),
            ],
        );
        let offered = plans(&probe);
        assert_eq!(
            offered
                .iter()
                .map(|p| p.action.as_str())
                .collect::<Vec<_>>(),
            [
                "assault:probe:26july",
                "assault:probe:directorio",
                "assault:probe:26july+directorio:0-2",
                "assault:probe:26july+directorio:1-1",
                "assault:probe:26july+directorio:2-0",
                "assault:probe:none",
            ]
        );
        for plan in &offered {
            assert!(
                plan.protection.ends_with(
                    "Syndicate: 0 Active, 1 Underground, 1 open Casino. Assault cannot remove its Underground Guerrillas until they are Activated, and they protect their faction's Bases."
                ),
                "{}",
                plan.protection
            );
            assert_eq!(plan.effect(), plan.action != "assault:probe:none");
            assert_eq!(plan.detail.closed, 0);
        }
    }

    /// What the best Assault would do, worked by hand from R 3.2.4 (1 piece per Troop in
    /// Grassland, plus 1 per Police in a City; a faction's Bases only once it has no
    /// Guerrillas there; a Casino closes, R 1.4.5).
    #[test]
    fn a_selectable_space_says_what_its_best_assault_would_do() {
        let active = |faction, count| (faction, Kind::Guerrilla, State::Active, count);
        let base = |faction, count| (faction, Kind::Base, State::Ordinary, count);
        let casino = |count| (Faction::Syndicate, Kind::Casino, State::Open, count);
        for (terrain, pieces, expected) in [
            // Capacity 3, 2 Active: both.
            (
                Terrain::City,
                vec![with(TROOPS, 2), with(POLICE, 1), active(Faction::M26, 2)],
                "Can remove 2 Guerrillas",
            ),
            // Capacity 1: one of the two.
            (
                Terrain::Grassland,
                vec![with(TROOPS, 1), active(Faction::M26, 2)],
                "Can remove 1 Guerrilla",
            ),
            // No Guerrillas: the Base.
            (
                Terrain::Grassland,
                vec![with(TROOPS, 4), base(Faction::M26, 1)],
                "Can remove 1 Base",
            ),
            // Guerrillas first, then the Base: 2 + 1 of capacity 3.
            (
                Terrain::Grassland,
                vec![
                    with(TROOPS, 3),
                    active(Faction::M26, 2),
                    base(Faction::M26, 1),
                ],
                "Can remove 2 Guerrillas, 1 Base",
            ),
            (
                Terrain::City,
                vec![with(TROOPS, 6), casino(1)],
                "Can close 1 Casino",
            ),
            // Two factions and room for both: Guerrillas of either, 4 in all.
            (
                Terrain::Grassland,
                vec![
                    with(TROOPS, 4),
                    active(Faction::M26, 3),
                    active(Faction::Directorio, 1),
                ],
                "Can remove 4 Guerrillas",
            ),
            // Capacity 2 among 5 of two factions: whichever are chosen, 2 Guerrillas.
            (
                Terrain::Grassland,
                vec![
                    with(TROOPS, 2),
                    active(Faction::M26, 3),
                    active(Faction::Directorio, 2),
                ],
                "Can remove up to 2 Guerrillas",
            ),
            // Capacity 1: a Guerrilla, or the Casino.
            (
                Terrain::Grassland,
                vec![with(TROOPS, 1), active(Faction::M26, 1), casino(1)],
                "Can affect up to 1 piece",
            ),
            // Room for a Guerrilla and the Casino: two kinds, one plan.
            (
                Terrain::Grassland,
                vec![with(TROOPS, 2), active(Faction::M26, 1), casino(1)],
                "Can affect 2 pieces",
            ),
            // Directorio's Underground Guerrillas are out of reach and not counted.
            (
                Terrain::City,
                vec![
                    with(TROOPS, 6),
                    with(POLICE, 4),
                    (Faction::Directorio, Kind::Guerrilla, State::Underground, 2),
                    casino(1),
                ],
                "Can close 1 Casino",
            ),
        ] {
            let line = best_outcome(&space(terrain, &pieces));
            assert_eq!(line, expected, "{pieces:?}");
            assert!(line.chars().count() <= OUTCOME_CHARS, "{line}");
        }
        // The longest words the pieces allow are replaced by the number.
        let crowded = space(
            Terrain::Grassland,
            &[
                with(TROOPS, 20),
                active(Faction::M26, 15),
                active(Faction::Directorio, 12),
                base(Faction::M26, 2),
                base(Faction::Directorio, 2),
            ],
        );
        assert_eq!(best_outcome(&crowded), "Can affect up to 20 pieces");
        let ten = space(
            Terrain::Grassland,
            &[
                with(TROOPS, 20),
                active(Faction::M26, 10),
                base(Faction::M26, 2),
            ],
        );
        assert_eq!(best_outcome(&ten), "Can affect 12 pieces");
    }

    /// Each kind of reason R 3.2.4 gives for an Assault that removes nothing, worked
    /// by hand from the rule on a position built for it.
    #[test]
    fn assault_says_why_it_would_remove_nothing_for_each_kind_of_reason() {
        let m26 = |state, count| (Faction::M26, Kind::Guerrilla, state, count);
        let base = (Faction::M26, Kind::Base, State::Ordinary, 1);
        let closed = (Faction::Syndicate, Kind::Casino, State::Closed, 1);
        for (terrain, pieces, expected) in [
            // No enemy piece: nothing to target, whatever the cubes.
            (
                Terrain::City,
                vec![with(TROOPS, 3)],
                Some("No enemy pieces (R 3.2.4)"),
            ),
            // A City or EC counts Troops and Police; with neither, 0.
            (
                Terrain::EconomicCenter,
                vec![m26(State::Active, 1)],
                Some("No Troops or Police (R 3.2.4)"),
            ),
            // A Forest counts Troops only: 0 Troops, and its Police do not count.
            (
                Terrain::Forest,
                vec![with(POLICE, 2), m26(State::Active, 1)],
                Some("Police count only in Cities, ECs"),
            ),
            (
                Terrain::Grassland,
                vec![m26(State::Active, 1)],
                Some("No Troops (R 3.2.4)"),
            ),
            // Mountain: 1 Troop ÷ 2 = 0.
            (
                Terrain::Mountain,
                vec![with(TROOPS, 1), m26(State::Active, 1)],
                Some("1 Troop: Mountain needs 2"),
            ),
            // Capacity 2, but the only Guerrilla is Underground.
            (
                Terrain::Forest,
                vec![with(TROOPS, 2), m26(State::Underground, 1)],
                Some("No Active Guerrilla (R 3.2.4)"),
            ),
            // Capacity 2; the Underground Guerrilla keeps its Base ("Bases last").
            (
                Terrain::Forest,
                vec![with(TROOPS, 2), m26(State::Underground, 1), base],
                Some("No Active Guerrilla · Bases last"),
            ),
            // Capacity 1; a closed Casino stays closed (R 1.4.5).
            (
                Terrain::City,
                vec![with(POLICE, 1), closed],
                Some("Casinos stay closed (R 1.4.5)"),
            ),
            // Mountain: 2 Troops ÷ 2 = 1, and an Active Guerrilla: a plan removes it.
            (
                Terrain::Mountain,
                vec![with(TROOPS, 2), m26(State::Active, 1)],
                None,
            ),
            // With no Guerrilla left, the Base is removed.
            (Terrain::Forest, vec![with(TROOPS, 1), base], None),
        ] {
            let probe = space(terrain, &pieces);
            assert_eq!(
                removes_nothing(&probe).as_deref(),
                expected,
                "{terrain:?} {pieces:?}"
            );
            // The reason and the plans agree: a reason exactly where no plan affects a piece.
            assert_eq!(
                expected.is_some(),
                plans(&probe).iter().all(|plan| !plan.effect()),
                "{terrain:?} {pieces:?}"
            );
            if let Some(line) = expected {
                assert!(line.chars().count() <= REASON_CHARS, "{line}");
            }
        }
    }
}

#[cfg(test)]
mod record_tests {
    use super::*;

    fn paid(removed: Vec<Group>, closed: u8) -> AssaultResolution {
        AssaultResolution {
            space: "havana".into(),
            paid: 2,
            returned: removed.iter().map(|group| group.count).sum(),
            detail: AssaultDetail {
                space: "Havana".into(),
                removed,
                closed,
            },
        }
    }

    /// A paid space's record, in the panel's summary and in Finish Assault's
    /// explanation: one wording for the space and its payment, then each sentence's
    /// own words for what it removed or closed (R 3.2.4, 1.4.5).
    #[test]
    fn a_paid_space_is_worded_once_for_the_summary_and_the_finish() {
        let casino = paid(vec![], 1);
        assert_eq!(
            casino.summary(),
            "Havana · paid 2 Resources · closed 1 Casino"
        );
        assert_eq!(
            casino.finished(),
            "Havana · paid 2 Resources. Closed 1 Casino in place (R 1.4.5)"
        );
        let guerrillas = paid(
            vec![Group {
                faction: Faction::Directorio,
                kind: Kind::Guerrilla,
                state: State::Active,
                count: 2,
            }],
            0,
        );
        assert_eq!(
            guerrillas.summary(),
            "Havana · paid 2 Resources · returned 2 Directorio Guerrillas"
        );
        assert_eq!(
            guerrillas.finished(),
            "Havana · paid 2 Resources. Returned 2 Directorio Active Guerrillas to Available Forces"
        );
        let nothing = paid(vec![], 0);
        assert_eq!(
            nothing.summary(),
            "Havana · paid 2 Resources · removed nothing"
        );
        assert_eq!(
            nothing.finished(),
            "Havana · paid 2 Resources. No pieces affected"
        );
    }
}
