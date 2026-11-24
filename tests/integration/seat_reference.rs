//! The seat reference's Choices table (`docs/agents/seat.md`) against the choices
//! the rules authority offers (#94). Agents name choices from that table; a missing
//! or stale family can mislead them. The oracle is `Game::observe` at every explored
//! card 17 state and along checked card 18 paths.
//!
//! Not covered: states the explorer does not reach. It follows at most one Resource
//! transfer per path, of 1 or of the direction's limit, from the starts it names.
//! Card 18 visits the open Assault on the inherited board, with one paid space and
//! representative side choices. A family appearing only in another state will
//! fail the stale check if the table lists it.
use crate::explore;
use cuba_libre_unfolded::{Choice, Faction, Game, TransferChoice};
use std::collections::BTreeSet;

/// A choice's id family, as the table writes it without `<revision>:`. Only the
/// authority's own id parsers place a placeholder, so an id they do not parse stays
/// literal: `terror:kidnap`, a Special Activity listed at the action choice, has no
/// space and stays apart from `terror:<space>`.
fn family(choice: &Choice) -> String {
    if let Some(transfer) = choice.transfer() {
        return match transfer {
            TransferChoice::Give { .. } => "give:<faction>:<amount>",
            TransferChoice::Ask { .. } => "ask:<faction>:<amount>",
            TransferChoice::Authorize { .. } => "authorize:<amount>",
        }
        .into();
    }
    if choice.removal().is_some() {
        return "remove:<space>:<group>:<n>".into();
    }
    let action = choice.action();
    if action
        .strip_prefix("terror:")
        .is_some_and(|space| choice.space() == Some(space))
    {
        return "terror:<space>".into();
    }
    // An Assault plan: the authority states its space, the id's second part.
    if action
        .strip_prefix("assault:")
        .and_then(|rest| rest.split_once(':'))
        .is_some_and(|(space, _)| choice.space() == Some(space))
    {
        return "assault:<space>:<plan>".into();
    }
    action.into()
}

/// The families offered in `game`'s current state.
fn families(game: &Game) -> BTreeSet<String> {
    game.observe().choices.iter().map(family).collect()
}

/// Every family offered in every explored state.
fn offered() -> BTreeSet<String> {
    let mut offered = BTreeSet::new();
    let starts = explore::starts().into_iter().map(|(_, start)| start);
    for start in starts.chain(explore::event_starts()) {
        for node in explore::explore(start) {
            offered.extend(families(&node.game));
        }
    }
    offered.extend(card_18_families());
    offered
}

fn card_18_families() -> BTreeSet<String> {
    let [mut opening, _] = explore::event_starts();
    for action in ["resolve-shaded", "advance-card"] {
        explore::choose(&mut opening, action);
    }
    let mut passed = opening.clone();
    explore::choose(&mut passed, "government:pass");
    let mut offered = families(&opening);
    offered.extend(families(&passed));

    // An open Assault offers every unpaid space's plans at once (#169).
    let mut assault = opening;
    explore::choose(&mut assault, "government:assault");
    let root = assault.observe();
    offered.extend(families(&assault));

    for actions in [&["transfer", "ask:1:1", "wait"][..], &["remove"]] {
        if root
            .choices
            .iter()
            .any(|choice| choice.action() == actions[0])
        {
            let mut side = assault.clone();
            for action in actions {
                explore::choose(&mut side, action);
                offered.extend(families(&side));
            }
        }
    }
    explore::choose(&mut assault, "assault:santiago:26july");
    offered.extend(families(&assault));
    explore::choose(&mut assault, "end-assault");
    offered.extend(families(&assault));
    offered
}

/// The ids in the first cell of each row of the Choices table, without their
/// `<revision>:` prefix. An id listed twice fails, as a set would hide it.
fn documented() -> BTreeSet<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/agents/seat.md");
    let doc = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let table = doc
        .split_once("\n## Choices\n")
        .map(|(_, rest)| rest)
        .unwrap_or_else(|| panic!("{path} has no Choices section"));
    let table = table.split("\n#").next().unwrap_or(table);
    let mut families = BTreeSet::new();
    for row in table.lines().filter(|l| l.starts_with("| `")) {
        let first = row.split('|').nth(1).unwrap_or_default();
        for id in first.split('`').skip(1).step_by(2) {
            let family = id
                .strip_prefix("<revision>:")
                .unwrap_or_else(|| panic!("Choices table id `{id}` lacks `<revision>:`"));
            assert!(
                families.insert(family.to_owned()),
                "Choices table lists `{id}` twice"
            );
        }
    }
    assert!(
        !families.is_empty(),
        "found no choice ids in {path}'s Choices table"
    );
    families
}

#[test]
fn the_seat_reference_lists_every_offered_choice_family() {
    let offered = offered();
    let documented = documented();
    let missing: Vec<_> = offered.difference(&documented).collect();
    let stale: Vec<_> = documented.difference(&offered).collect();
    assert!(
        missing.is_empty() && stale.is_empty(),
        "docs/agents/seat.md's Choices table differs from the offered choices.\n\
         Offered but not listed: {missing:?}\nListed but never offered: {stale:?}"
    );
}

/// The reduction keeps `terror:kidnap` apart from `terror:<space>`, and places
/// Terror, transfer and authorization placeholders by the authority's own reading.
/// Removal ids are left to the table check, which reaches removal mode.
#[test]
fn choice_ids_reduce_to_their_table_families() {
    let mut game = Game::opening_with_seed(19);
    let opening = families(&game);
    assert!(opening.contains("terror:kidnap"), "{opening:?}");
    assert!(!opening.contains("terror:<space>"), "{opening:?}");
    explore::choose(&mut game, "terror");
    let terror = families(&game);
    assert!(terror.contains("terror:<space>"), "{terror:?}");
    assert!(terror.contains("kidnap"), "{terror:?}");
    explore::choose(&mut game, "transfer");
    let transfer = families(&game);
    assert!(transfer.contains("give:<faction>:<amount>"), "{transfer:?}");
    assert!(transfer.contains("ask:<faction>:<amount>"), "{transfer:?}");
    explore::choose(&mut game, &format!("ask:{}:1", Faction::Syndicate.index()));
    let deciding = families(&game);
    assert!(deciding.contains("authorize:<amount>"), "{deciding:?}");
}
