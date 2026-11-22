//! #132. R 1.5.1 and 1.8 give the pending transfer to the giver and cap
//! its amount. R 1.4.1 allows Government to return its own pieces.
use cuba_libre_unfolded::{Faction, Game, Kind, Observation, State, Transfer};

const G: Faction = Faction::Government;
const M: Faction = Faction::M26;

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap_or_else(|error| panic!("{action}: {error}"));
}

fn refuse(game: &mut Game, owner: Faction, action: &str) -> String {
    let before = game.clone();
    let revision = game.observe().revision;
    let error = game
        .submit(owner, revision, &format!("{revision}:{action}"))
        .expect_err(action);
    assert_eq!(*game, before, "{action} changed the match");
    error
}

fn funded_opening() -> Game {
    let mut game = Game::prepared_with_seed(2);
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
    ] {
        choose(&mut game, action);
    }
    assert_eq!(game.observe().resources, [2, 3, 5, 15]);
    game
}

fn santiago_target(game: &mut Game) -> Observation {
    choose(game, "assault:areas:cities");
    choose(game, "assault:space:santiago");
    game.observe()
}

fn reload(game: &Game) -> Game {
    let restored: Game = serde_json::from_slice(&serde_json::to_vec(game).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.observe(), game.observe());
    restored
}

#[test]
fn only_the_giver_can_fund_the_next_assault_payment() {
    let mut game = funded_opening();
    refuse(&mut game, G, "transfer");
    choose(&mut game, "government:assault");
    choose(&mut game, "transfer");
    assert!(refuse(&mut game, G, "give:1:3").contains("more than 2"));
    choose(&mut game, "give:1:0");
    assert_eq!(game.observe().resources, [2, 3, 5, 15]);
    choose(&mut game, "transfer");
    choose(&mut game, "give:1:2");
    assert_eq!(game.observe().resources, [0, 5, 5, 15]);
    let mut zero_space = game.clone();
    let carried = zero_space.observe();
    choose(&mut zero_space, "end-assault");
    let completed = reload(&zero_space).observe();
    assert_eq!(completed.resources, [0, 5, 5, 15]);
    assert_eq!(completed.spaces, carried.spaces);
    assert_eq!(completed.available, carried.available);
    assert!(completed.assault_result.is_none());
    assert_eq!(completed.next.unwrap().role, "2nd Eligible");
    refuse(&mut zero_space, G, "transfer");

    let unfunded = santiago_target(&mut game.clone());
    let payment = unfunded
        .choices
        .iter()
        .find(|c| c.action() == "assault:santiago:26july");
    assert!(payment.is_some_and(|p| !p.supported && p.preview.is_none()));

    choose(&mut game, "transfer");
    choose(&mut game, "ask:1:1");
    let pending = game.observe();
    assert_eq!(pending.executing, Some(G));
    assert_eq!(pending.owner, M);
    assert_eq!(pending.resources, [0, 5, 5, 15]);
    assert_eq!(
        pending.transfer,
        Some(Transfer::Authorizing {
            giver: M,
            recipient: G,
            amount: 1,
            waited: false
        })
    );
    assert_eq!(pending.transfer_terms[0].limit, 5);
    let mut game = reload(&game);
    refuse(&mut game, G, "authorize:1");
    refuse(&mut game, G, "end-assault");
    assert!(refuse(&mut game, M, "authorize:6").contains("more than 5"));

    for (action, balances) in [
        ("decline", [0, 5, 5, 15]),
        ("authorize:0", [0, 5, 5, 15]),
        ("authorize:5", [5, 0, 5, 15]),
    ] {
        let mut candidate = game.clone();
        choose(&mut candidate, action);
        assert_eq!(candidate.observe().resources, balances);
        assert_eq!(candidate.observe().owner, G);
        assert_eq!(candidate.observe().transfer, None);
        assert!(
            candidate
                .submit(
                    M,
                    pending.revision,
                    &format!("{}:{action}", pending.revision)
                )
                .is_err()
        );
        refuse(&mut candidate, M, action);
        reload(&candidate);
    }
    choose(&mut game, "authorize:5");
    let next = santiago_target(&mut game);
    let payment = next
        .choices
        .iter()
        .find(|c| c.action() == "assault:santiago:26july")
        .unwrap();
    assert!(payment.supported);
    assert_eq!(payment.preview.as_ref().unwrap().resources[0].from, 5);
    assert_eq!(payment.preview.as_ref().unwrap().resources[0].to, 3);
    choose(&mut game, payment.action());
    choose(&mut game, "end-assault");
    let finished = reload(&game).observe();
    assert_eq!(finished.resources, [3, 0, 5, 15]);
    assert_eq!(finished.next.unwrap().role, "2nd Eligible");
    refuse(&mut game, G, "transfer");
}

#[test]
fn voluntary_removal_changes_santiago_control_and_assault_capacity() {
    let mut game = funded_opening();
    refuse(&mut game, G, "remove");
    choose(&mut game, "government:assault");
    let opening_capacity = santiago_target(&mut game.clone()).choices;
    let opening_payment = opening_capacity
        .iter()
        .find(|choice| choice.action() == "assault:santiago:26july")
        .unwrap();
    assert!(
        opening_payment
            .explanation
            .contains("2 Troops + 1 Police = 3")
    );
    choose(&mut game, "remove");

    let pending = game.observe();
    assert_eq!(pending.owner, G);
    let mut no_removal = game.clone();
    choose(&mut no_removal, "done-removal");
    assert_eq!(no_removal.observe().spaces, pending.spaces);
    assert_eq!(no_removal.observe().available, pending.available);
    let mut game = reload(&game);
    refuse(&mut game, M, "remove:santiago:troops:1");
    for illegal in [
        "remove:santiago:troops:0",
        "remove:santiago:troops:3",
        "remove:santiago:active:1",
        "remove:santiago:police:2",
    ] {
        refuse(&mut game, G, illegal);
    }

    let mut partial = game.clone();
    choose(&mut partial, "remove:santiago:troops:1");
    let santiago = |view: &Observation| {
        view.spaces
            .iter()
            .find(|s| s.id == "santiago")
            .unwrap()
            .control()
    };
    assert_eq!(santiago(&partial.observe()), None);

    let action = "remove:santiago:troops:2";
    let removal = pending
        .choices
        .iter()
        .find(|choice| choice.action() == action)
        .unwrap();
    let effect = removal.preview.as_ref().unwrap();
    assert_eq!(effect.removed.len(), 1);
    assert_eq!(
        (
            effect.removed[0].faction,
            effect.removed[0].kind,
            effect.removed[0].state,
            effect.removed[0].count
        ),
        (G, Kind::Troops, State::Ordinary, 2)
    );
    assert_eq!(effect.control[0].from, Some(G));
    assert_eq!(effect.control[0].to, Some(M));
    assert!(effect.standings.is_empty());
    choose(&mut game, action);
    let removed = game.observe();
    assert_eq!(santiago(&removed), Some(M));
    assert_eq!(removed.standings, pending.standings);
    assert_eq!(removed.map_piece_count, pending.map_piece_count - 2);
    assert_eq!(
        removed.available_piece_count,
        pending.available_piece_count + 2
    );
    let available_troops = |view: &Observation| {
        view.available
            .iter()
            .find(|p| p.faction == G && p.kind == Kind::Troops)
            .unwrap()
            .count
    };
    assert_eq!(available_troops(&removed), available_troops(&pending) + 2);
    assert!(game.submit(G, pending.revision, &removal.id).is_err());
    refuse(&mut game, G, action);
    let mut game = reload(&game);
    choose(&mut game, "done-removal");
    let next = santiago_target(&mut game);
    let payment = next
        .choices
        .iter()
        .find(|choice| choice.action() == "assault:santiago:26july")
        .unwrap();
    assert!(payment.explanation.contains("0 Troops + 1 Police = 1"));
    assert_eq!(payment.preview.as_ref().unwrap().removed[0].count, 1);
    choose(&mut game, payment.action());
    assert_eq!(santiago(&game.observe()), None);
    choose(&mut game, "end-assault");
    assert_eq!(reload(&game).observe().next.unwrap().role, "2nd Eligible");
    refuse(&mut game, G, "remove");
}
