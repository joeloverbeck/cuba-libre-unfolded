//! #58 story 53, found false by #75: a disabled choice's hint says what in this task
//! would make it available, and never offers a remedy that is impossible here. The
//! panel's hover card says "Nothing in this task can make it available" exactly when
//! the authority names no remedy, so the authority's remedies must be honest in both
//! directions:
//! - no remedy: no sequence of offered decisions from here makes the choice
//!   supported;
//! - a remedy: some sequence does, and each remedy named does it.
//!
//! The oracle is reachability through the authority's own offered decisions, not the
//! remedies' wording; the states explored are `support/explore.rs`'s. A disabled
//! choice is judged where that path's one transfer is still unused.
#[path = "support/explore.rs"]
mod explore;
use cuba_libre_unfolded::{Choice, Faction, Game};
use explore::{Node, action, choose, explore, reachable, starts};
use std::collections::BTreeSet;

/// Every dishonest hint among the explored states, by label.
fn dishonest(nodes: &[Node]) -> BTreeSet<String> {
    let future = reachable(nodes);
    let mut found = BTreeSet::new();
    for node in nodes.iter().filter(|n| !n.transferred) {
        let later: BTreeSet<_> = node
            .next
            .iter()
            .flat_map(|&j| future[j].iter().cloned())
            .collect();
        for (action, label, remedies) in &node.disabled {
            let possible = later.contains(action);
            if remedies.is_empty() == possible {
                found.insert(format!(
                    "{label} ({action}): remedies {remedies:?}, but it {} become available",
                    if possible { "can" } else { "cannot" }
                ));
            }
        }
    }
    found
}

#[test]
fn every_disabled_choice_s_remedies_are_exactly_what_this_task_allows() {
    for (start, game) in starts() {
        let found = dishonest(&explore(game));
        assert!(
            found.is_empty(),
            "{start}:\n{}",
            found.into_iter().collect::<Vec<_>>().join("\n")
        );
    }
}

/// Whether `action` is offered and supported in `game`.
fn supported(game: &Game, wanted: &str) -> bool {
    game.observe()
        .choices
        .iter()
        .any(|c| c.supported && action(c) == wanted)
}

fn offered(game: &Game, wanted: &str) -> bool {
    game.observe()
        .choices
        .iter()
        .any(|c: &Choice| c.supported && c.action() == wanted)
}

/// Closes the step open in `game` with no effect, so a remedy's first decision can
/// be offered: the giver declines a pending request, Cancel closes the transfer or an
/// opened Kidnap, and Done leaves removal mode. A remedy named while a step is open
/// is performed once that step has closed.
fn close_open_step(game: &mut Game) {
    let view = game.observe();
    if view.transfer.is_some() {
        for close in ["decline", "back"] {
            if offered(game, close) {
                return choose(game, close);
            }
        }
    } else if view.kidnap_open {
        choose(game, "kidnap:cancel");
    } else if view.removal.is_some() {
        choose(game, "done-removal");
    }
}

/// The decisions that perform a named remedy, as alternatives: each is tried from a
/// copy of `game`. A Resource transfer is requested from each faction that can give,
/// for its limit, which it authorizes in full.
fn performances(game: &Game, remedy: &str) -> Vec<Vec<String>> {
    match remedy {
        "Choose Terror" => vec![vec!["terror".into()]],
        "Resolve Santiago's Terror" => vec![vec!["terror:santiago".into()]],
        "Kidnap here first" => vec![vec!["kidnap".into(), "kidnap:roll".into()]],
        "Request a Resource transfer" => {
            let mut opened = game.clone();
            if !offered(&opened, "transfer") {
                close_open_step(&mut opened);
            }
            if !offered(&opened, "transfer") {
                return vec![];
            }
            choose(&mut opened, "transfer");
            [Faction::Government, Faction::Directorio, Faction::Syndicate]
                .into_iter()
                .filter_map(|giver| {
                    let limit = opened
                        .observe()
                        .choices
                        .iter()
                        .filter_map(|c| {
                            c.action()
                                .strip_prefix(&format!("ask:{}:", giver.index()))?
                                .parse::<u8>()
                                .ok()
                        })
                        .max()
                        .filter(|&n| n > 0)?;
                    Some(vec![
                        "transfer".into(),
                        format!("ask:{}:{limit}", giver.index()),
                        format!("authorize:{limit}"),
                    ])
                })
                .collect()
        }
        other => panic!("no performance is known for the remedy {other:?}"),
    }
}

/// Whether performing `remedy` from `game` makes `wanted` supported.
fn works(game: &Game, wanted: &str, remedy: &str) -> bool {
    performances(game, remedy).into_iter().any(|decisions| {
        let mut after = game.clone();
        if !offered(&after, &decisions[0]) {
            close_open_step(&mut after);
        }
        for decision in &decisions {
            if !offered(&after, decision) {
                return false;
            }
            choose(&mut after, decision);
        }
        supported(&after, wanted)
    })
}

/// Story 53's depth: not only does some remedy exist, each remedy a hint names works.
/// Performing it from the state that names it (after closing an open step with no
/// effect) reaches a state where the refused choice is supported.
#[test]
fn every_named_remedy_makes_its_choice_available() {
    for (start, game) in starts() {
        let nodes = explore(game);
        let mut failed = BTreeSet::new();
        let mut checked = 0;
        for node in &nodes {
            for (wanted, label, remedies) in &node.disabled {
                for remedy in remedies {
                    checked += 1;
                    if !works(&node.game, wanted, remedy) {
                        failed.insert(format!(
                            "{label} ({wanted}): {remedy:?} does not make it available"
                        ));
                    }
                }
            }
        }
        assert!(checked > 0, "{start}: no remedy was named");
        assert!(
            failed.is_empty(),
            "{start}:\n{}",
            failed.into_iter().collect::<Vec<_>>().join("\n")
        );
    }
}
