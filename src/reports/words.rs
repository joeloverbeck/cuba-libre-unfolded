//! A report's words, fixed per kind of action. They state what changed and judge nothing.

use super::{
    Action, Article, Balance, FromTo, Item, Moment, Transfers, kind_name, recorded, standings,
};
use crate::{
    Faction, Group, Kidnap, Kind, MARKER_SUPPLY, RallyAction, State, resources, support_level,
    total,
};

/// The most spaces a headline names; it counts more.
const NAMED: usize = 3;

fn article(title: impl Into<String>, lines: Vec<String>) -> Article {
    Article {
        title: title.into(),
        lines,
        die: None,
    }
}

/// "a, b and c".
pub(crate) fn listing(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn change(balance: Balance) -> String {
    format!("{} → {}", balance.from, balance.to)
}

/// "1 Active Guerrilla", "2 Troops".
fn pieces(group: &Group) -> String {
    let name = kind_name(group.kind, group.count);
    let state = match group.state {
        State::Underground => "Underground ",
        State::Active => "Active ",
        State::Ordinary | State::Open | State::Closed => "",
    };
    format!("{} {state}{name}", group.count)
}

fn verb(count: u8, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 { one } else { many }
}

/// "2 Guerrillas turn Underground → Active".
fn turned(count: u8, from: &str, to: &str) -> String {
    format!(
        "{count} {} {} {from} → {to}",
        kind_name(Kind::Guerrilla, count),
        verb(count, "turns", "turn")
    )
}

/// The spaces the Operation itself was executed in.
fn operation_spaces(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Terror { space, .. }
            | Item::Assault { space, .. }
            | Item::Construct { space, .. } => Some(space.clone()),
            _ => None,
        })
        .collect()
}

/// `again` says that an earlier report told the beginning of this same action.
pub(super) fn headline(
    faction: Faction,
    action: Action,
    again: bool,
    complete: bool,
    items: &[Item],
) -> String {
    let name = faction.name();
    let (what, operation) = match action {
        Action::Undecided => return format!("{name} goes back to its action choice"),
        Action::Pass => return format!("{name} Passes"),
        Action::EventUnshaded => ("the Event, unshaded text", false),
        Action::EventShaded => ("the Event, shaded text", false),
        Action::Terror => ("Terror", true),
        Action::Assault => ("Assault", true),
        Action::Construct => ("Limited Construct", true),
    };
    let spaces = operation_spaces(items);
    let named = match spaces.len() {
        0 => None,
        n if n > NAMED => Some(format!("{n} spaces")),
        _ => Some(listing(&spaces)),
    };
    match (named, again, complete) {
        (Some(named), false, true) => format!("{name} executes {what} in {named}"),
        (None, false, true) if operation => format!("{name} executes {what} in no space"),
        (Some(named), false, false) => format!("{name} executes {what}: {named} so far"),
        (None, false, _) => format!("{name} executes {what}"),
        (Some(named), true, false) => format!("{name} continues {what}: {named}"),
        (None, true, false) => format!("{name} continues {what}"),
        (Some(named), true, true) => format!("{name} ends {what}: {named}"),
        (None, true, true) => format!("{name} ends {what}"),
    }
}

/// What the page says under its headline: the open decision of a giver, else the Pass, the
/// Kidnap or the grant of the Rally.
pub(super) fn summary(faction: Faction, items: &[Item]) -> Option<String> {
    let asking = items.iter().find_map(|item| match item {
        Item::Transfers(told) => pending(told),
        _ => None,
    });
    asking.or_else(|| {
        items.iter().find_map(|item| match item {
            Item::Pass(balance) => Some(format!(
                "{} receives {}, {}.",
                faction.name(),
                resources(balance.to - balance.from),
                change(*balance)
            )),
            Item::Kidnap { space, kidnap } => Some(format!(
                "Kidnap in {space}: the die shows {}, and {}.",
                kidnap.die,
                taken(kidnap)
            )),
            Item::Granted(beneficiary) => Some(format!(
                "Directorio grants the free Rally to {}.",
                beneficiary.name()
            )),
            _ => None,
        })
    })
}

fn pending(told: &Transfers) -> Option<String> {
    let giver = told.giver.name();
    told.pending.map(|amount| {
        format!(
            "{} asks {giver} for {}. {giver} decides.",
            told.recipient.name(),
            resources(amount)
        )
    })
}

/// What a Kidnap took (R 4.3.3) from Government, the target in a City.
fn taken(kidnap: &Kidnap) -> String {
    if kidnap.proceeds == 0 {
        return "Government has no Resources to take".into();
    }
    format!(
        "26July takes {} from Government",
        resources(kidnap.proceeds)
    )
}

/// What happened, in order. The Pass, the grant of the Rally and a pending request are in
/// the summary alone.
pub(super) fn articles(faction: Faction, items: &[Item]) -> Vec<Article> {
    let mut articles: Vec<Article> =
        items
            .iter()
            .filter_map(|item| match item {
                Item::Pass(_) | Item::Granted(_) => None,
                Item::WentBack { request } => Some(article(
                    "Back to the action choice",
                    vec![match request {
                        Some(giver) => format!(
                            "{} withdraws its request to {}.",
                            faction.name(),
                            giver.name()
                        ),
                        None => format!("{} goes back to its action choice.", faction.name()),
                    }],
                )),
                Item::Transfers(told) => transfers(told),
                Item::Removed { faction, groups } => Some(removed(*faction, groups)),
                Item::ManagedCasinos {
                    space,
                    group,
                    closed_in_place,
                } => {
                    let state = if group.state == State::Open {
                        "open"
                    } else {
                        "closed"
                    };
                    let casinos = kind_name(Kind::Casino, group.count);
                    let effect = if *closed_in_place {
                        format!(
                            "Syndicate closes {} {state} {casinos} in place.",
                            group.count
                        )
                    } else {
                        format!(
                            "Syndicate returns {} {state} {casinos} to Available Forces.",
                            group.count
                        )
                    };
                    Some(article(
                        format!("Casino management in {space}"),
                        vec![effect, "0 Resources · no Profit.".into()],
                    ))
                }
                Item::Kidnap { space, kidnap } => Some(kidnapped(space, kidnap)),
                Item::Terror {
                    space,
                    paid,
                    balance,
                    activated,
                    markers,
                    support,
                } => {
                    let mut lines = vec![
                        pays(faction, *paid, *balance),
                        format!("{}.", turned(*activated, "Underground", "Active")),
                    ];
                    if *markers > 0 {
                        lines.push(format!(
                            "{markers} Terror {} placed.",
                            verb(*markers, "marker is", "markers are")
                        ));
                    }
                    lines.extend(support.map(|(from, to)| {
                        format!("{} → {}.", support_level(from), support_level(to))
                    }));
                    Some(article(format!("Terror in {space}"), lines))
                }
                Item::Assault {
                    removed, closed, ..
                } if removed.is_empty() && *closed == 0 => None,
                Item::Assault {
                    space,
                    paid,
                    balance,
                    removed,
                    closed,
                } => {
                    let mut lines = vec![pays(faction, *paid, *balance)];
                    lines.extend(removed.iter().map(|group| {
                        format!(
                            "{}: {} {} to Available Forces.",
                            group.faction.name(),
                            pieces(group),
                            verb(group.count, "returns", "return")
                        )
                    }));
                    if *closed > 0 {
                        lines.push(format!(
                            "Syndicate: {closed} {} in place.",
                            verb(*closed, "Casino closes", "Casinos close")
                        ));
                    }
                    Some(article(format!("Assault in {space}"), lines))
                }
                Item::Construct {
                    space,
                    paid,
                    balance,
                    plan,
                } => Some(article(
                    format!("Construct in {space}"),
                    vec![
                        pays(faction, *paid, *balance),
                        format!("{} {}.", faction.name(), plan.result()),
                    ],
                )),
                Item::Shaded { flips } => {
                    let mut lines: Vec<String> = flips
                        .iter()
                        .map(|(space, count)| {
                            format!("{} in {space}.", turned(*count, "Underground", "Active"))
                        })
                        .collect();
                    if lines.is_empty() {
                        lines.push("No Guerrilla turns Underground → Active.".into());
                    }
                    Some(article("26July's Guerrillas", lines))
                }
                Item::Rally {
                    faction,
                    space,
                    action,
                } => Some(article(
                    format!("{} Rallies in {space}", faction.name()),
                    vec![rally(*action)],
                )),
                Item::NoRally(faction) => Some(article(
                    "The free Rally",
                    vec![format!("{} Rallies in no space.", faction.name())],
                )),
            })
            .collect();
    articles.extend(assaults_that_changed_no_piece(faction, items));
    articles
}

fn pays(faction: Faction, paid: u8, balance: Balance) -> String {
    format!(
        "{} pays {}, {}.",
        faction.name(),
        resources(paid),
        change(balance)
    )
}

/// The Assaulted spaces where no piece changed, told together where there are several:
/// the US Alliance sets one cost for every space of the Operation (R 6.3.1).
fn assaults_that_changed_no_piece(faction: Faction, items: &[Item]) -> Option<Article> {
    const NOTHING: &str = "No piece is removed or closed.";
    let here: Vec<(&String, u8, Balance)> = items
        .iter()
        .filter_map(|item| match item {
            Item::Assault {
                space,
                paid,
                balance,
                removed,
                closed: 0,
            } if removed.is_empty() => Some((space, *paid, *balance)),
            _ => None,
        })
        .collect();
    match here[..] {
        [] => None,
        [(space, paid, balance)] => Some(article(
            format!("Assault in {space}"),
            vec![pays(faction, paid, balance), NOTHING.into()],
        )),
        [(_, paid, _), ..] => {
            let names: Vec<String> = here.iter().map(|(space, ..)| (*space).clone()).collect();
            Some(article(
                format!("Assault in {} spaces", names.len()),
                vec![
                    NOTHING.into(),
                    format!(
                        "{} pays {} in each of {}.",
                        faction.name(),
                        resources(paid),
                        listing(&names)
                    ),
                ],
            ))
        }
    }
}

/// A faction's own pieces, returned to its Available Forces (R 1.4.1), space by space.
fn removed(faction: Faction, groups: &[(String, Group)]) -> Article {
    let mut spaces: Vec<(&String, Vec<String>)> = vec![];
    for (space, group) in groups {
        match spaces.iter_mut().find(|(known, _)| *known == space) {
            Some((_, here)) => here.push(pieces(group)),
            None => spaces.push((space, vec![pieces(group)])),
        }
    }
    article(
        format!("{} removes its own pieces", faction.name()),
        spaces
            .into_iter()
            .map(|(space, here)| format!("{space}: {}.", listing(&here)))
            .collect(),
    )
}

fn kidnapped(space: &str, kidnap: &Kidnap) -> Article {
    let balances = if kidnap.proceeds == 0 {
        String::new()
    } else {
        format!(
            ": Government {} → {}, 26July {} → {}",
            kidnap.government_before,
            kidnap.government_after(),
            kidnap.m26_before,
            kidnap.m26_after()
        )
    };
    Article {
        die: Some(kidnap.die),
        ..article(
            format!("Kidnap in {space}"),
            vec![
                format!("The die shows {}.", kidnap.die),
                format!("{}{balances}.", taken(kidnap)),
            ],
        )
    }
}

fn rally(action: RallyAction) -> String {
    match action {
        RallyAction::Place(0) => "No piece moves.".into(),
        RallyAction::Place(count) => format!(
            "{count} Underground {} placed from Available Forces.",
            verb(count, "Guerrilla is", "Guerrillas are")
        ),
        RallyAction::FlipUnderground(count) => {
            format!("{}.", turned(count, "Active", "Underground"))
        }
        RallyAction::ReplaceWithBase {
            underground,
            active,
        } => format!(
            "{underground} Underground and {active} Active Guerrillas are replaced with 1 Base."
        ),
    }
}

/// What was asked and given, less a request still pending, which is in the summary.
fn transfers(told: &Transfers) -> Option<Article> {
    let (giver, recipient) = (told.giver.name(), told.recipient.name());
    let title = match told.given {
        0 if told.unmet > 0 => format!("{giver} gives {recipient} nothing"),
        0 if told.withdrawn > 0 => format!("{recipient} asks {giver} for Resources"),
        0 => return None,
        1 => format!("{giver} gives {recipient} 1 Resource"),
        given => format!("{giver} gives {recipient} {given} Resources"),
    };
    let mut lines = vec![];
    match (told.asks - u16::from(told.pending.is_some()), told.asked) {
        (0, _) | (_, None) => {}
        (1, Some(amount)) => {
            lines.push(format!("{recipient} asked for {}.", resources(amount)));
        }
        (asks, _) => lines.push(format!("{recipient} asked {asks} times.")),
    }
    match (told.moves, told.balances) {
        (0, _) | (_, None) => {}
        (1, Some((from, to))) => lines.push(format!(
            "{giver} {}, {recipient} {}.",
            change(from),
            change(to)
        )),
        (moves, _) => lines.push(format!("They move in {moves} transfers.")),
    }
    let requests = |n: u16| format!("{n} request{}", if n == 1 { "" } else { "s" });
    if told.unmet > 0 {
        lines.push(format!(
            "{giver} answers {} with nothing.",
            requests(told.unmet)
        ));
    }
    if told.withdrawn > 0 {
        lines.push(format!(
            "{recipient} withdraws {}.",
            requests(told.withdrawn)
        ));
    }
    Some(article(title, lines))
}

/// What else differs between `first` and `last`, each kind only if it changed.
pub(super) fn notes(first: &Moment, last: &Moment, faction: Faction) -> Vec<Article> {
    [
        ("Control", control(first, last)),
        ("Available Forces", available(first, last)),
        ("Markers", markers(first, last)),
        ("Victory totals", totals(first, last)),
        ("Eligibility", eligibility(first, last, faction)),
    ]
    .into_iter()
    .filter(|(_, lines)| !lines.is_empty())
    .map(|(title, lines)| article(title, lines))
    .collect()
}

fn control(first: &Moment, last: &Moment) -> Vec<String> {
    let name = |control: Option<Faction>| control.map_or("none", Faction::name);
    first
        .spaces
        .iter()
        .zip(&last.spaces)
        .filter(|(was, is)| was.control() != is.control())
        .map(|(was, is)| {
            format!(
                "{}: {} → {}.",
                is.name,
                name(was.control()),
                name(is.control())
            )
        })
        .collect()
}

fn available(first: &Moment, last: &Moment) -> Vec<String> {
    last.available
        .iter()
        .filter_map(|group| {
            let from = total(&first.available, group.faction, group.kind);
            (from != group.count).then(|| {
                format!(
                    "{} {}: {from} → {}.",
                    group.faction.name(),
                    kind_name(group.kind, 2),
                    group.count
                )
            })
        })
        .collect()
}

fn markers(first: &Moment, last: &Moment) -> Vec<String> {
    let (from, to) = (first.markers_available, last.markers_available);
    (from != to)
        .then(|| {
            format!("Terror and Sabotage markers available: {from} → {to} of {MARKER_SUPPLY}.")
        })
        .into_iter()
        .collect()
}

/// The totals the factions' victories count, where one changed (R 7.2).
fn totals(first: &Moment, last: &Moment) -> Vec<String> {
    standings(first, last)
        .into_iter()
        .filter(|row| row.score.from != row.score.to)
        .map(|row| {
            let FromTo { from, to } = row.score;
            let counts = match row.faction {
                Faction::Government => "Total Support",
                Faction::M26 => "Total Opposition + Bases",
                Faction::Directorio => "Controlled Population + Bases",
                Faction::Syndicate => "open Casinos",
            };
            format!("{} {counts}: {from} → {to}.", row.faction.name())
        })
        .collect()
}

/// What changed on the Sequence of Play (R 2.3).
fn eligibility(first: &Moment, last: &Moment, faction: Faction) -> Vec<String> {
    let (was, is) = (&first.eligibility, &last.eligibility);
    let mut lines = vec![];
    if let Some(recorded) = recorded(last, faction)
        && !(last.card == 18 && is.next_card.is_some())
    {
        lines.push(format!("{} is in the {recorded} box.", faction.name()));
    }
    if was.m26_next_card != is.m26_next_card
        && let Some(eligible) = is.m26_next_card
    {
        lines.push(format!(
            "26July is {} for the next card.",
            if eligible { "Eligible" } else { "Ineligible" }
        ));
    }
    if was.eligible != is.eligible {
        let those = |eligible: bool| {
            let names: Vec<String> = Faction::ALL
                .into_iter()
                .filter(|f| is.eligible[f.index()] == eligible)
                .map(|f| f.name().to_owned())
                .collect();
            listing(&names)
        };
        let ineligible = those(false);
        if ineligible.is_empty() {
            lines.push(format!(
                "All four factions are Eligible for card {}.",
                last.next_card
            ));
        } else {
            lines.push(format!(
                "Card {}: {} Eligible, {ineligible} Ineligible.",
                last.next_card,
                those(true),
            ));
        }
    }
    if was.event_restriction != is.event_restriction
        && let Some(kept) = is.event_restriction
    {
        lines.push(format!(
            "The Event keeps {} Ineligible through card {}.",
            kept.faction.name(),
            kept.through_card
        ));
    }
    if first.next != last.next
        && let Some(next) = &last.next
    {
        lines.push(format!("Next: {}, {}.", next.faction.name(), next.role));
    }
    lines
}
