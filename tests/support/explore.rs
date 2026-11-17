//! The task's reachable states through the authority's own offered decisions, shared
//! by `tests/honest_hints.rs` and the panel's discovery test
//! (`src/tabletop/discovery.rs`). The includer brings `Choice`, `Faction`, `Game`
//! and `TransferChoice` into scope.
//!
//! From each start, every state reachable through offered decisions is explored,
//! with at most one Resource transfer on any path (every remedy here needs at most
//! one), of 1 or of the direction's limit. The starts:
//! - the prepared position under Kidnap roll 1 (#8's seed 19), which leaves the owed
//!   Terror's reserve exactly paid, and under roll 3 (seed 5), which funds Oriente too;
//! - Government and 26July both at 0 (Government gave 26July 6, which 26July gave
//!   Directorio): Kidnap would take nothing, so only a transfer can fund Terror or
//!   guarantee Kidnap's commitment.
#![allow(dead_code)]
use super::{Choice, Faction, Game, TransferChoice};
use std::collections::{BTreeSet, HashMap};

/// A state without its revision, which every decision advances.
pub fn key(game: &Game) -> String {
    let mut value = serde_json::to_value(game).unwrap();
    value["revision"] = 0.into();
    value.to_string()
}

/// The action a choice stands for. Kidnap is listed at the action choice before
/// Terror is chosen, under another id (R 4.1).
pub fn action(choice: &Choice) -> String {
    match choice.action() {
        "terror:kidnap" => "kidnap".into(),
        other => other.into(),
    }
}

/// Submits the offered choice as the faction that owns it.
pub fn choose(game: &mut Game, action: &str) {
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
pub fn explored(choices: &[Choice]) -> Vec<&Choice> {
    let amount = |c: &Choice| c.transfer().map(|t| (t.direction(), t.amount()));
    let mut largest: HashMap<TransferChoice, u8> = HashMap::new();
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
pub fn transfers(choice: &Choice) -> bool {
    matches!(
        choice.transfer(),
        Some(TransferChoice::Give { amount, .. } | TransferChoice::Authorize { amount }) if amount > 0
    )
}

pub struct Node {
    pub game: Game,
    /// The disabled choices here: (action, label, remedies).
    pub disabled: Vec<(String, String, Vec<String>)>,
    pub supported: BTreeSet<String>,
    pub next: Vec<usize>,
    /// Whether this path's one Resource transfer has been made.
    pub transferred: bool,
}

pub fn explore(start: Game) -> Vec<Node> {
    let mut index = HashMap::from([((key(&start), false), 0)]);
    let mut games = vec![(start, false)];
    let mut nodes = vec![];
    while let Some((game, transferred)) = games.get(nodes.len()).cloned() {
        let view = game.observe();
        let mut node = Node {
            game: game.clone(),
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
pub fn reachable(nodes: &[Node]) -> Vec<BTreeSet<String>> {
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

pub fn drained() -> Game {
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

/// The explored starts, by name.
pub fn starts() -> [(&'static str, Game); 3] {
    [
        ("prepared, roll 1", Game::opening_with_seed(19)),
        ("prepared, roll 3", Game::opening_with_seed(5)),
        ("Government and 26July at 0", drained()),
    ]
}
