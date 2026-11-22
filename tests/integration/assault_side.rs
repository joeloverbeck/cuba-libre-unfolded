//! #132. R 1.5.1 and 1.8 give the pending transfer to the giver and cap
//! its amount. R 1.4.1 allows Government to return its own pieces.
use cuba_libre_unfolded::{Faction, Game, Observation, Transfer};

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
    let has = |game: &Game, action| game.observe().choices.iter().any(|c| c.action() == action);
    if has(game, "assault:areas:cities") {
        choose(game, "assault:areas:cities");
        choose(game, "assault:space:santiago");
    }
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
    let unfunded = santiago_target(&mut game.clone());
    let payment = unfunded
        .choices
        .iter()
        .find(|c| c.action() == "assault:santiago:26july");
    assert!(payment.is_none_or(|p| !p.supported && p.preview.is_none()));

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
