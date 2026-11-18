//! The seat reference's Choices table (`docs/agents/seat.md`) against the choices
//! the rules authority offers (#94). Agents name choices from that table; one it
//! lacks gets refused or missed, and one it lists that the game no longer offers
//! misleads them. The game is the oracle: every choice `Game::observe` offers,
//! supported or not, in every state the shared explorer reaches from its starts.
//!
//! Not covered: states the explorer does not reach. It follows at most one Resource
//! transfer per path, of 1 or of the direction's limit, from the starts it names. A
//! family offered only elsewhere is invisible to the missing check, and fails the
//! stale check only if the table lists it. `Game::prepared()`, the state every #8
//! save began in, is not a start: #94's triage probe explored from it too and found
//! no family the starts lack.
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
    action.into()
}

/// Every family offered in every explored state.
fn offered() -> BTreeSet<String> {
    let mut families = BTreeSet::new();
    for (_, start) in explore::starts() {
        for node in explore::explore(start) {
            families.extend(node.game.observe().choices.iter().map(family));
        }
    }
    families
}

/// The ids in the first cell of each row of the Choices table, without their
/// `<revision>:` prefix.
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
            families.insert(family.to_owned());
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

/// The reduction keeps the families an agent must tell apart, and gives each
/// parameterised id its placeholders by the authority's own reading.
#[test]
fn choice_ids_reduce_to_their_table_families() {
    let mut game = Game::opening_with_seed(19);
    let opening: BTreeSet<_> = game.observe().choices.iter().map(family).collect();
    assert!(opening.contains("terror:kidnap"), "{opening:?}");
    assert!(!opening.contains("terror:<space>"), "{opening:?}");
    explore::choose(&mut game, "terror");
    let terror: BTreeSet<_> = game.observe().choices.iter().map(family).collect();
    assert!(terror.contains("terror:<space>"), "{terror:?}");
    assert!(terror.contains("kidnap"), "{terror:?}");
    explore::choose(&mut game, "transfer");
    let transfer: BTreeSet<_> = game.observe().choices.iter().map(family).collect();
    assert!(transfer.contains("give:<faction>:<amount>"), "{transfer:?}");
    assert!(transfer.contains("ask:<faction>:<amount>"), "{transfer:?}");
    explore::choose(&mut game, &format!("ask:{}:1", Faction::Syndicate.index()));
    let deciding: BTreeSet<_> = game.observe().choices.iter().map(family).collect();
    assert!(deciding.contains("authorize:<amount>"), "{deciding:?}");
}
