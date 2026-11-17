//! #58 story 53, found false by #75: a disabled choice's hint says what in this task
//! would make it available, and never offers a remedy that is impossible here. The
//! panel's hover card says "Nothing in this task can make it available" exactly when
//! the authority names no remedy, so the authority's remedies must be honest in both
//! directions:
//! - no remedy: no sequence of offered decisions from here makes the choice
//!   supported;
//! - a remedy: some sequence does.
//!
//! The oracle is reachability through the authority's own offered decisions, not the
//! remedies' wording. From each start, every state reachable through offered
//! decisions is explored, with at most one Resource transfer on any path (every
//! remedy here needs at most one), of 1 or of the direction's limit; a disabled
//! choice is judged where that transfer is still unused. The starts:
//! - the prepared position under Kidnap roll 1 (#8's seed 19), which leaves the owed
//!   Terror's reserve exactly paid, and under roll 3 (seed 5), which funds Oriente too;
//! - Government and 26July both at 0 (Government gave 26July 6, which 26July gave
//!   Directorio): Kidnap would take nothing, so only a transfer can fund Terror or
//!   guarantee Kidnap's commitment.
use cuba_libre_unfolded::{Choice, Faction, Game};
use std::collections::{BTreeSet, HashMap};

/// A state without its revision, which every decision advances.
fn key(game: &Game) -> String {
    let mut value = serde_json::to_value(game).unwrap();
    value["revision"] = 0.into();
    value.to_string()
}

/// The action a choice stands for. Kidnap is listed at the action choice before
/// Terror is chosen, under another id (R 4.1).
fn action(choice: &Choice) -> String {
    match choice.action() {
        "terror:kidnap" => "kidnap".into(),
        other => other.into(),
    }
}

/// Submits the offered choice as the faction that owns it.
fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    let id = format!("{}:{action}", view.revision);
    let owner = view
        .choices
        .iter()
        .find(|c| c.id == id)
        .map_or(view.owner, |c| c.owner);
    game.submit(owner, view.revision, &id)
        .unwrap_or_else(|e| panic!("{id}: {e}"));
}

/// The transfer amounts explored: 1 and the largest offered in each direction.
fn explored(choices: &[Choice]) -> Vec<&Choice> {
    let amount = |c: &Choice| {
        let a = c.action();
        (a.starts_with("give:") || a.starts_with("ask:") || a.starts_with("authorize:")).then(
            || {
                let (direction, n) = a.rsplit_once(':').unwrap();
                (direction.to_owned(), n.parse::<u8>().unwrap())
            },
        )
    };
    let mut largest: HashMap<String, u8> = HashMap::new();
    for (direction, n) in choices.iter().filter_map(amount) {
        let top = largest.entry(direction).or_default();
        *top = (*top).max(n);
    }
    choices
        .iter()
        .filter(|c| c.supported)
        .filter(|c| amount(c).is_none_or(|(d, n)| n == 1 || (n > 0 && n == largest[&d])))
        .collect()
}

/// Whether `choice` makes a Resource transfer: a nonzero give or authorization.
fn transfers(choice: &Choice) -> bool {
    let a = choice.action();
    (a.starts_with("give:") || a.starts_with("authorize:")) && !a.ends_with(":0")
}

struct Node {
    /// The disabled choices here: (action, label, remedies).
    disabled: Vec<(String, String, Vec<String>)>,
    supported: BTreeSet<String>,
    next: Vec<usize>,
    /// Whether this path's one Resource transfer has been made.
    transferred: bool,
}

fn explore(start: Game) -> Vec<Node> {
    let mut index = HashMap::from([((key(&start), false), 0)]);
    let mut games = vec![(start, false)];
    let mut nodes = vec![];
    while let Some((game, transferred)) = games.get(nodes.len()).cloned() {
        let view = game.observe();
        let mut node = Node {
            disabled: view
                .choices
                .iter()
                .filter(|c| !c.supported)
                .map(|c| (action(c), c.label.clone(), c.remedies.clone()))
                .collect(),
            supported: view
                .choices
                .iter()
                .filter(|c| c.supported)
                .map(action)
                .collect(),
            next: vec![],
            transferred,
        };
        for choice in explored(&view.choices) {
            if transferred && transfers(choice) {
                continue;
            }
            let mut after = game.clone();
            after
                .submit(choice.owner, view.revision, &choice.id)
                .unwrap_or_else(|e| panic!("{}: {e}", choice.id));
            let next = (key(&after), transferred || transfers(choice));
            let at = *index.entry(next).or_insert_with(|| {
                games.push((after, transferred || transfers(choice)));
                games.len() - 1
            });
            node.next.push(at);
        }
        nodes.push(node);
        assert!(nodes.len() < 100_000, "the explored task grew unexpectedly");
    }
    nodes
}

/// For each state, every action supported there or in any state reachable from it.
fn reachable(nodes: &[Node]) -> Vec<BTreeSet<String>> {
    let mut future: Vec<_> = nodes.iter().map(|n| n.supported.clone()).collect();
    loop {
        let mut changed = false;
        for i in (0..nodes.len()).rev() {
            let mut union = future[i].clone();
            for &j in &nodes[i].next {
                union.extend(future[j].iter().cloned());
            }
            if union.len() != future[i].len() {
                future[i] = union;
                changed = true;
            }
        }
        if !changed {
            return future;
        }
    }
}

/// Every dishonest hint among the explored states, by label.
fn dishonest(start: Game) -> BTreeSet<String> {
    let nodes = explore(start);
    let future = reachable(&nodes);
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

fn drained() -> Game {
    let mut game = Game::opening_with_seed(19);
    choose(&mut game, "terror");
    choose(&mut game, "transfer");
    choose(&mut game, &format!("ask:{}:6", Faction::Government.index()));
    choose(&mut game, "authorize:6");
    choose(&mut game, "transfer");
    choose(
        &mut game,
        &format!("give:{}:6", Faction::Directorio.index()),
    );
    assert_eq!(game.observe().resources, [0, 0, 11, 15]);
    game
}

#[test]
fn every_disabled_choice_s_remedies_are_exactly_what_this_task_allows() {
    for (start, game) in [
        ("prepared, roll 1", Game::opening_with_seed(19)),
        ("prepared, roll 3", Game::opening_with_seed(5)),
        ("Government and 26July at 0", drained()),
    ] {
        let found = dishonest(game);
        assert!(
            found.is_empty(),
            "{start}:\n{}",
            found.into_iter().collect::<Vec<_>>().join("\n")
        );
    }
}
