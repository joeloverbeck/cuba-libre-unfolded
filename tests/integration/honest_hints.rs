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
use crate::explore::{Node, action, choose, explore, reachable, starts};
use cuba_libre_unfolded::{Choice, Faction, Game, TransferChoice};
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

/// Whether `wanted`, as an explored action, is supported in `game`: Kidnap's listing
/// at the action choice (`terror:kidnap`) counts as Kidnap.
fn action_supported(game: &Game, wanted: &str) -> bool {
    game.observe()
        .choices
        .iter()
        .any(|c| c.supported && action(c) == wanted)
}

/// Whether a supported choice with exactly this action, as its id names it, is
/// offered in `game`: a decision that can be submitted now.
fn choice_offered(game: &Game, wanted: &str) -> bool {
    game.observe()
        .choices
        .iter()
        .any(|c: &Choice| c.supported && c.action() == wanted)
}

/// A remedy and its timing: "Resolve Santiago's Terror once Directorio has decided"
/// is the remedy "Resolve Santiago's Terror", which waits on Directorio's decision on
/// the pending request. A remedy without timing can be started at once.
fn timing(remedy: &str) -> (&str, Option<&str>) {
    match remedy.rsplit_once(" once ") {
        Some((base, rest)) => match rest.strip_suffix(" has decided") {
            Some(giver) => (base, Some(giver)),
            None => (remedy, None),
        },
        None => (remedy, None),
    }
}

/// The decisions that perform a remedy, as alternatives: each is tried from a copy of
/// `game`. A Resource transfer is requested from each faction that can give, for its
/// limit, which it authorizes in full.
fn performances(game: &Game, remedy: &str) -> Vec<Vec<String>> {
    match remedy {
        "Choose Terror" => vec![vec!["terror".into()]],
        "Resolve Santiago's Terror" => vec![vec!["terror:santiago".into()]],
        "Kidnap here first" => vec![vec!["kidnap".into(), "kidnap:roll".into()]],
        "Request a Resource transfer" => {
            if !choice_offered(game, "transfer") {
                return vec![];
            }
            let mut opened = game.clone();
            choose(&mut opened, "transfer");
            [Faction::Government, Faction::Directorio, Faction::Syndicate]
                .into_iter()
                .filter_map(|giver| {
                    let limit = opened
                        .observe()
                        .choices
                        .iter()
                        .filter_map(|c| match c.transfer()? {
                            TransferChoice::Ask { giver: g, amount } if g == giver => Some(amount),
                            _ => None,
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

/// Whether some performance of `remedy` makes `wanted` supported, each decision
/// offered when it is made.
fn performs(game: &Game, wanted: &str, remedy: &str) -> bool {
    performances(game, remedy).into_iter().any(|decisions| {
        let mut after = game.clone();
        for decision in &decisions {
            if !choice_offered(&after, decision) {
                return false;
            }
            choose(&mut after, decision);
        }
        action_supported(&after, wanted)
    })
}

/// Whether `remedy`, named in `game`, makes `wanted` available at the time it states.
/// A remedy without timing must start now. One timed "once {giver} has decided" is
/// named while a request awaits that giver: it cannot start before the decision,
/// and works after each decision the giver may make (decline, nothing or its limit).
fn works(game: &Game, wanted: &str, remedy: &str) -> Result<(), String> {
    let (base, giver) = timing(remedy);
    let Some(giver) = giver else {
        return if performs(game, wanted, base) {
            Ok(())
        } else {
            Err("does not make it available from here, now".into())
        };
    };
    let view = game.observe();
    match view.transfer {
        Some(cuba_libre_unfolded::Transfer::Authorizing { giver: g, .. }) if g.name() == giver => {}
        _ => {
            return Err(format!(
                "names {giver}'s decision, but no request awaits {giver}"
            ));
        }
    }
    if performs(game, wanted, base) {
        return Err(format!("can start before {giver} has decided"));
    }
    let mut decisions = vec!["decline".to_owned()];
    decisions.extend(
        view.choices
            .iter()
            .filter_map(|c| match c.transfer()? {
                TransferChoice::Authorize { amount } => Some(amount),
                _ => None,
            })
            .filter(|&n| n == 0 || Some(n) == view.transfer_terms.first().map(|t| t.limit))
            .map(|n| format!("authorize:{n}")),
    );
    for decision in decisions {
        let mut decided = game.clone();
        if !choice_offered(&decided, &decision) {
            return Err(format!("{giver} cannot {decision}"));
        }
        choose(&mut decided, &decision);
        if !performs(&decided, wanted, base) {
            return Err(format!(
                "does not make it available after {giver}'s {decision}"
            ));
        }
    }
    Ok(())
}

/// Story 53's depth: not only does some remedy exist, each remedy a hint names works
/// when it says. Performing it from the state that names it reaches a state where the
/// refused choice is supported: at once, or, for a remedy timed "once {giver} has
/// decided", only after that giver's decision, whichever it is.
#[test]
fn every_named_remedy_makes_its_choice_available() {
    let mut timed = BTreeSet::new();
    for (start, game) in starts() {
        let nodes = explore(game);
        let mut failed = BTreeSet::new();
        let mut checked = 0;
        for node in &nodes {
            for (wanted, label, remedies) in &node.disabled {
                for remedy in remedies {
                    checked += 1;
                    if let (_, Some(giver)) = timing(remedy) {
                        timed.insert(giver.to_owned());
                    }
                    if let Err(why) = works(&node.game, wanted, remedy) {
                        failed.insert(format!("{label} ({wanted}): {remedy:?} {why}"));
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
    // A request can await each faction that may give, and the timed remedy was
    // checked for each, so the timing check is not vacuous.
    assert_eq!(
        timed.into_iter().collect::<Vec<_>>(),
        ["Directorio", "Government", "Syndicate"]
    );
}
