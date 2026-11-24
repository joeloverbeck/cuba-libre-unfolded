//! The task's reachable states through the authority's own offered decisions, shared
//! by the integration tests (`available_forces`, `honest_hints`, `marker_supply`,
//! `seat_reference`) and the panel's discovery test (`src/tabletop/discovery.rs`). The includer brings `Choice`,
//! `Faction`, `Game` and `TransferChoice` into scope.
//!
//! From each start, offered decisions are explored with at most one Resource
//! transfer on any path (every Terror remedy here needs at most one), of 1 or of
//! the direction's limit. The Terror starts:
//! - the opening position under Kidnap roll 1 (#8's seed 19), which leaves the owed
//!   Terror's reserve exactly paid, and under roll 3 (seed 5), which funds Oriente too;
//! - Government and 26July both at 0 (Government gave 26July 6, which 26July gave
//!   Directorio): Kidnap would take nothing, so only a transfer can fund Terror or
//!   guarantee Kidnap's commitment.
//!
//! The Event's side decisions are explored from its two texts so the graph does
//! not multiply their paths by every earlier Terror and transfer permutation.
//! Card 18 opens a separate Operation. Its seat documentation uses checked
//! representative paths instead of folding that Operation into this graph.
//!
//! Exploring dominates these tests, so a test process explores each start once, in
//! parallel (`explored`), and the tests read the same graphs.
#![allow(dead_code)]
use super::{Choice, Faction, Game, TransferChoice};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::LazyLock,
    thread,
};

/// A state's identity: the digest of its decision's key, which is far shorter than the
/// key, and whether its path's one Resource transfer has been made.
type StateId = ([u8; 32], bool);

fn state_id(game: &Game, transferred: bool) -> StateId {
    (Sha256::digest(key(game)).into(), transferred)
}

/// The current decision without its revision. Reopening and cancelling a side
/// decision reaches the same choices.
pub fn key(game: &Game) -> String {
    let serialized = serde_json::to_string(game).unwrap();
    let (revision, decision) = serialized.split_once(',').unwrap();
    assert!(
        revision.starts_with("{\"revision\":"),
        "a serialized game starts with its revision, not {revision}"
    );
    decision.into()
}

/// `work` on every item, shared among this machine's CPUs: the results in item order.
pub fn parallel<T: Sync, R: Send>(items: &[T], work: impl Fn(usize, &T) -> R + Sync) -> Vec<R> {
    let threads = thread::available_parallelism().map_or(1, usize::from);
    let share = items.len().div_ceil(threads).max(1);
    let work = &work;
    thread::scope(|scope| {
        let running: Vec<_> = items
            .chunks(share)
            .enumerate()
            .map(|(nth, chunk)| {
                scope.spawn(move || -> Vec<R> {
                    (nth * share..)
                        .zip(chunk)
                        .map(|(i, item)| work(i, item))
                        .collect()
                })
            })
            .collect();
        running
            .into_iter()
            .flat_map(|thread| {
                thread
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
}

pub type Action = (Faction, String);

/// The action and the faction entitled to take it. Both factions can be shown
/// the same Event text, and each has its own decision. Kidnap is listed under
/// another id before Terror is chosen (R 4.1).
pub fn action(choice: &Choice) -> Action {
    let name = match choice.action() {
        "terror:kidnap" => "kidnap",
        other => other,
    };
    (choice.owner, name.into())
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

/// The choices explored. Of the transfer amounts: 1 and the largest offered in each
/// direction.
fn followed(choices: &[Choice]) -> Vec<&Choice> {
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
    pub disabled: Vec<(Action, String, Vec<String>)>,
    pub supported: HashSet<Action>,
    pub next: Vec<usize>,
    /// Whether this path's one Resource transfer has been made.
    pub transferred: bool,
}

/// The states explored from `start`, in the order found, breadth first.
fn explore(start: Game) -> Vec<Node> {
    /// States expanded at a time, which bounds the successors held before merging.
    const BATCH: usize = 1024;
    /// A successor: its state's index where it was already found, else the state.
    enum Next {
        Known(usize),
        Found(StateId, Box<Game>),
    }
    let starts_during_event = start.observe().event_executor.is_some();
    let mut index = HashMap::from([(state_id(&start, false), 0)]);
    let mut waiting = VecDeque::from([(start, false)]);
    let mut nodes = vec![];
    while !waiting.is_empty() {
        let batch: Vec<_> = waiting.drain(..waiting.len().min(BATCH)).collect();
        let expanded = parallel(&batch, |_, (game, transferred)| {
            let view = game.observe();
            let disabled = view
                .choices
                .iter()
                .filter(|c| !c.supported)
                .map(|c| (action(c), c.label.clone(), c.remedies.clone()))
                .collect();
            let supported = view
                .choices
                .iter()
                .filter(|c| c.supported)
                .map(action)
                .collect();
            let mut next = vec![];
            if view.current_card == 18 {
                return (disabled, supported, next);
            }
            for choice in followed(&view.choices) {
                if !starts_during_event
                    && view.event_executor.is_some()
                    && matches!(choice.action(), "transfer" | "remove")
                {
                    continue;
                }
                if *transferred && transfers(choice) {
                    continue;
                }
                let mut after = game.clone();
                after
                    .submit(choice.owner, view.revision, &choice.id)
                    .unwrap_or_else(|e| panic!("{}: {e}", choice.id));
                let id = state_id(&after, *transferred || transfers(choice));
                next.push(match index.get(&id) {
                    Some(&at) => Next::Known(at),
                    None => Next::Found(id, Box::new(after)),
                });
            }
            (disabled, supported, next)
        });
        for ((game, transferred), (disabled, supported, next)) in batch.into_iter().zip(expanded) {
            let next = next
                .into_iter()
                .map(|next| match next {
                    Next::Known(at) => at,
                    Next::Found(id, after) => {
                        let at = index.len();
                        *index.entry(id).or_insert_with(|| {
                            waiting.push_back((*after, id.1));
                            at
                        })
                    }
                })
                .collect();
            nodes.push(Node {
                game,
                disabled,
                supported,
                next,
                transferred,
            });
        }
        assert!(nodes.len() < 100_000, "the explored task grew unexpectedly");
    }
    nodes
}

/// For each state, the states whose choices lead to it.
pub fn predecessors(nodes: &[Node]) -> Vec<Vec<usize>> {
    let mut before = vec![vec![]; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for &j in &node.next {
            before[j].push(i);
        }
    }
    before
}

/// For each state, whether `action` is supported there or in any state reachable from
/// it. `before` is the states' `predecessors`.
pub fn reaching(nodes: &[Node], before: &[Vec<usize>], action: &Action) -> Vec<bool> {
    let mut reaching = vec![false; nodes.len()];
    let mut found: Vec<usize> = (0..nodes.len())
        .filter(|&i| nodes[i].supported.contains(action))
        .collect();
    while let Some(i) = found.pop() {
        if !std::mem::replace(&mut reaching[i], true) {
            found.extend(&before[i]);
        }
    }
    reaching
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
fn starts() -> [(&'static str, Game); 3] {
    [
        ("opening, roll 1", Game::opening_with_seed(19)),
        ("opening, roll 3", Game::opening_with_seed(5)),
        ("Government and 26July at 0", drained()),
    ]
}

pub fn event_starts() -> [Game; 2] {
    let mut handoff = Game::opening_with_seed(5);
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
    ] {
        choose(&mut handoff, action);
    }
    let mut shaded = handoff.clone();
    choose(&mut shaded, "event:shaded");
    choose(&mut handoff, "event:unshaded");
    [shaded, handoff]
}

/// The states explored from one start.
pub struct Explored {
    pub start: &'static str,
    pub nodes: Vec<Node>,
}
impl Explored {
    fn new((start, game): (&'static str, Game)) -> Self {
        let nodes = explore(game);
        Self { start, nodes }
    }
}

/// What is explored from each of `starts`, in their order.
pub fn explored() -> &'static [Explored] {
    static EXPLORED: LazyLock<Vec<Explored>> = LazyLock::new(|| starts().map(Explored::new).into());
    &EXPLORED
}

/// What is explored from each of `event_starts`, in their order.
fn event_explored() -> &'static [Explored] {
    static EXPLORED: LazyLock<Vec<Explored>> = LazyLock::new(|| {
        let [shaded, unshaded] = event_starts();
        [("shaded Event", shaded), ("unshaded Event", unshaded)]
            .map(Explored::new)
            .into()
    });
    &EXPLORED
}

/// What is explored from each of `starts` and `event_starts`, in their order.
pub fn all_explored() -> impl Iterator<Item = &'static Explored> {
    explored().iter().chain(event_explored())
}
