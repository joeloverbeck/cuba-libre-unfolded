//! #71: Resource transfers through the transfer modal's two decisions, "Give to" and
//! "Ask from", and the giver's own decision on a Transfer request.
//!
//! Expected values come from the Living Rules (March 2019) and #8's prepared position
//! (Government 6, 26July 0, Directorio 5, Syndicate 15), worked by hand:
//! - R 1.5.1: Resources may be transferred only while the giving or receiving faction
//!   executes an Operation; here 26July executes Terror, so 26July is one endpoint.
//!   Agreements are not binding: a request moves nothing, and only the giver decides.
//! - R 1.8: a faction holds at most 49 Resources.
//! - The owed-Terror reserve is the accepted commitment policy (local, #8): while a
//!   rolled Kidnap owes Santiago's Terror, 26July keeps 1 Resource for it.
//! - Zero and repeated transfers are allowed; a zero or declined request is no effect
//!   (local policy, #58).
use cuba_libre_unfolded::{Bound, Faction, Game, Transfer};

const G: Faction = Faction::Government;
const M26: Faction = Faction::M26;
const D: Faction = Faction::Directorio;
const S: Faction = Faction::Syndicate;

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

/// Submits `action` as `owner` and expects a refusal that changes nothing.
fn refused(game: &mut Game, owner: Faction, action: &str) -> String {
    let before = game.clone();
    let revision = game.observe().revision;
    let error = game
        .submit(owner, revision, &format!("{revision}:{action}"))
        .expect_err(action);
    assert_eq!(*game, before, "{action} changed the match");
    error
}

fn actions(game: &Game) -> Vec<String> {
    game.observe()
        .choices
        .iter()
        .map(|c| c.id.split_once(':').unwrap().1.to_owned())
        .collect()
}

/// 26July gives a faction Resources: the gift moves on confirmation.
#[test]
fn twenty_six_july_s_own_gift_moves_at_once() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:3");
    choose(&mut game, "authorize:3");
    assert_eq!(game.observe().resources, [6, 3, 5, 12]);
    choose(&mut game, "transfer");
    choose(&mut game, "give:2:2");
    let view = game.observe();
    assert_eq!(view.resources, [6, 1, 7, 12]);
    assert_eq!(view.owner, M26);
    assert_eq!(view.transfer, None);
    game.validate().unwrap();
}

/// A request moves nothing; the giver sets the actual amount, here more than asked.
#[test]
fn ask_hands_the_amount_to_the_giver_who_may_choose_another() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:2");
    let view = game.observe();
    assert_eq!(view.resources, [6, 0, 5, 15], "a request moved Resources");
    assert_eq!(view.owner, S);
    assert_eq!(
        view.transfer,
        Some(Transfer::Authorizing {
            giver: S,
            recipient: M26,
            amount: 2
        })
    );
    // Syndicate may authorize anything from 0 to its balance of 15.
    let authorize: Vec<_> = view
        .choices
        .iter()
        .filter(|c| c.id.contains(":authorize:"))
        .map(|c| {
            (
                c.id.rsplit_once(':').unwrap().1.parse::<u8>().unwrap(),
                c.owner,
            )
        })
        .collect();
    assert_eq!(authorize, (0..=15).map(|n| (n, S)).collect::<Vec<_>>());
    let terms = &view.transfer_terms;
    assert_eq!(terms.len(), 1);
    assert_eq!(
        (terms[0].giver, terms[0].recipient, terms[0].limit),
        (S, M26, 15)
    );
    // 26July cannot answer for Syndicate.
    refused(&mut game, M26, "authorize:2");
    choose(&mut game, "authorize:5");
    let view = game.observe();
    assert_eq!(view.resources, [6, 5, 5, 10]);
    assert_eq!((view.owner, view.transfer.clone()), (M26, None));
    // Syndicate's victory margin counts its Resources (R 7.3: Resources − 30), so
    // giving 5 moves that term from 15 − 30 = −15 to 10 − 30 = −20, which is worse.
    assert!(
        view.standings[3]
            .contributions
            .contains(&"Resources: 10".to_owned())
    );
    game.validate().unwrap();
}

#[test]
fn decline_returns_to_26july_with_nothing_moved_and_no_effect() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:0:4");
    assert_eq!(game.observe().owner, G);
    choose(&mut game, "decline");
    let view = game.observe();
    assert_eq!(view.resources, [6, 0, 5, 15]);
    assert_eq!((view.owner, view.transfer.clone()), (M26, None));
    assert!(
        actions(&game).contains(&"back-to-choice".to_owned()),
        "a declined request is no effect"
    );
    game.validate().unwrap();
}

/// Wait keeps the request exactly: the same state and revision, and the same record.
#[test]
fn wait_keeps_the_pending_request_unchanged_and_saveable() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:2:3");
    let pending = game.clone();
    choose(&mut game, "wait");
    assert_eq!(game, pending);
    let record = serde_json::to_vec(&game).unwrap();
    let restored: Game = serde_json::from_slice(&record).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.observe(), pending.observe());
    assert_eq!(restored.observe().owner, D);
}

#[test]
fn zero_and_repeated_transfers_are_allowed_and_zero_is_no_effect() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "give:0:0");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:0");
    choose(&mut game, "authorize:0");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:2");
    choose(&mut game, "authorize:0");
    assert_eq!(game.observe().resources, [6, 0, 5, 15]);
    assert!(actions(&game).contains(&"back-to-choice".to_owned()));
    for (step, expected) in [
        ("ask:0:1", [5, 1, 5, 15]),
        ("give:0:1", [6, 0, 5, 15]),
        ("ask:2:1", [6, 1, 4, 15]),
        ("give:2:1", [6, 0, 5, 15]),
        ("ask:3:1", [6, 1, 5, 14]),
        ("give:3:1", [6, 0, 5, 15]),
        ("ask:3:1", [6, 1, 5, 14]),
    ] {
        choose(&mut game, "transfer");
        choose(&mut game, step);
        if step.starts_with("ask") {
            choose(&mut game, "authorize:1");
        }
        let view = game.observe();
        assert_eq!(view.resources, expected, "{step}");
        assert_eq!(
            view.resources.iter().map(|&n| u16::from(n)).sum::<u16>(),
            26
        );
        assert_eq!(game.random_state(), 17, "a transfer consumed randomness");
        game.validate().unwrap();
    }
    assert!(
        !actions(&game).contains(&"back-to-choice".to_owned()),
        "an authorized transfer is the first effect"
    );
}

/// The limits, worked from the position: the giver's balance binds every request in
/// the prepared position (the richest holds 15, so the 49 cap has room).
#[test]
fn the_giver_s_balance_bounds_gifts_requests_and_authorizations() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    let view = game.observe();
    let terms = |giver, recipient| {
        view.transfer_terms
            .iter()
            .find(|t| t.giver == giver && t.recipient == recipient)
            .unwrap()
            .clone()
    };
    assert_eq!(view.transfer_terms.len(), 6);
    for (giver, recipient, balance, room, limit) in [
        (G, M26, 6, 49, 6),
        (D, M26, 5, 49, 5),
        (S, M26, 15, 49, 15),
        (M26, G, 0, 43, 0),
        (M26, D, 0, 44, 0),
        (M26, S, 0, 34, 0),
    ] {
        let t = terms(giver, recipient);
        assert_eq!(
            (t.balance, t.room, t.reserved, t.limit, t.bound),
            (balance, room, 0, limit, Bound::Balance),
            "{giver:?} → {recipient:?}"
        );
    }
    let offered = actions(&game);
    for limit in [
        "ask:0:6", "ask:2:5", "ask:3:15", "give:0:0", "give:2:0", "give:3:0",
    ] {
        assert!(offered.contains(&limit.to_owned()), "{limit}");
    }
    for over in ["ask:0:7", "ask:2:6", "ask:3:16", "give:0:1", "give:3:1"] {
        assert!(!offered.contains(&over.to_owned()), "{over}");
    }
    assert_eq!(
        refused(&mut game, M26, "ask:0:7"),
        "7 is more than 6: Government has 6 Resources."
    );
    assert_eq!(
        refused(&mut game, M26, "give:2:1"),
        "1 is more than 0: 26July has 0 Resources."
    );
    choose(&mut game, "ask:2:5");
    assert_eq!(
        refused(&mut game, D, "authorize:6"),
        "6 is more than 5: Directorio has 5 Resources."
    );
}

/// After a Kidnap roll of 3 in Santiago (R 4.3.3: Government 6 → 3, 26July 0 → 3),
/// Santiago's Terror is owed, so 26July can give at most 2 (accepted commitment
/// policy). Requests from others are not reserved.
#[test]
fn the_owed_terror_reserve_bounds_26july_s_gifts_until_it_resolves() {
    let mut game = Game::prepared_with_seed(5); // shared roll 3
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    assert_eq!(game.observe().resources, [3, 3, 5, 15]);
    choose(&mut game, "transfer");
    let view = game.observe();
    let gift = view
        .transfer_terms
        .iter()
        .find(|t| t.giver == M26 && t.recipient == D)
        .unwrap();
    assert_eq!(
        (gift.balance, gift.reserved, gift.limit, gift.bound),
        (3, 1, 2, Bound::Reserve)
    );
    let ask = view
        .transfer_terms
        .iter()
        .find(|t| t.giver == G && t.recipient == M26)
        .unwrap();
    assert_eq!((ask.reserved, ask.limit, ask.bound), (0, 3, Bound::Balance));
    assert_eq!(
        refused(&mut game, M26, "give:2:3"),
        "3 is more than 2: the owed Santiago Terror needs 1 of 26July's 3 Resources."
    );
    choose(&mut game, "give:2:2");
    assert_eq!(game.observe().resources, [3, 1, 7, 15]);
    choose(&mut game, "terror:santiago");
    choose(&mut game, "transfer");
    let view = game.observe();
    let gift = view
        .transfer_terms
        .iter()
        .find(|t| t.giver == M26 && t.recipient == D)
        .unwrap();
    assert_eq!((gift.balance, gift.reserved, gift.limit), (0, 0, 0));
    game.validate().unwrap();
}

/// The recipient's 49 cap cannot bind in the 26-total position, so challenge its rule
/// guard in a counterfactual high balance; this is not an admitted save.
#[test]
fn the_recipient_s_49_cap_bounds_a_transfer_in_a_counterfactual_position() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    let mut serialized = serde_json::to_value(&game).unwrap();
    serialized["resources"] = serde_json::json!([6, 47, 5, 15]);
    let mut counterfactual: Game = serde_json::from_value(serialized).unwrap();
    assert!(counterfactual.validate().is_err());
    let view = counterfactual.observe();
    let ask = view
        .transfer_terms
        .iter()
        .find(|t| t.giver == S && t.recipient == M26)
        .unwrap();
    assert_eq!(
        (ask.balance, ask.room, ask.limit, ask.bound),
        (15, 2, 2, Bound::Cap)
    );
    assert_eq!(
        refused(&mut counterfactual, M26, "ask:3:3"),
        "3 is more than 2: 26July holds 47 of at most 49 Resources."
    );
    choose(&mut counterfactual, "ask:3:2");
    assert!(
        !actions(&counterfactual).contains(&"authorize:3".to_owned()),
        "the giver cannot exceed the recipient's room"
    );
}

/// R 1.5.1: 26July is always one endpoint, and transfers exist only while it executes.
#[test]
fn endpoints_without_26july_and_transfers_outside_execution_are_refused() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    for action in [
        "give:1:1",
        "ask:1:0",
        "give:4:0",
        "ask:9:1",
        "transfer:0:2",
        "transfer:3:2",
        "amount:1",
        "authorize:0",
    ] {
        refused(&mut game, M26, action);
    }
    choose(&mut game, "back");
    // Outside the modal neither decision is offered.
    for action in ["give:0:0", "ask:3:1", "authorize:0", "decline"] {
        refused(&mut game, M26, action);
    }
    // At the action choice, after Pass and after End Terror nothing transfers.
    let mut opening = Game::opening();
    refused(&mut opening, M26, "transfer");
    choose(&mut opening, "pass");
    assert!(opening.observe().transfer_terms.is_empty());
    refused(&mut opening, M26, "transfer");
    choose(&mut game, "end-terror");
    refused(&mut game, M26, "transfer");
    assert!(game.observe().transfer_terms.is_empty());
}

/// Local policy (#58): 26July's request does not hold its Operation hostage. Going back
/// to the action choice or ending Terror withdraws it; nothing has moved.
#[test]
fn a_pending_request_is_withdrawn_by_going_back_or_ending_terror() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:2");
    choose(&mut game, "back-to-choice");
    let view = game.observe();
    assert_eq!(view.transfer, None);
    assert_eq!(view.resources, [6, 0, 5, 15]);
    assert_eq!(view.owner, M26);
    game.validate().unwrap();

    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:0:1");
    choose(&mut game, "authorize:1");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:2");
    assert!(
        !actions(&game).contains(&"back-to-choice".to_owned()),
        "no going back after the first effect"
    );
    choose(&mut game, "end-terror");
    let view = game.observe();
    assert!(view.finished);
    assert_eq!(view.transfer, None);
    assert_eq!(view.resources, [5, 1, 5, 15]);
    game.validate().unwrap();
}

/// The reserved state: a request another faction starts, which 26July answers. Nothing
/// in this task creates one; the same pending state is admitted and decided by 26July,
/// bounded by its reserve.
#[test]
fn the_reserved_state_is_decided_by_26july_within_its_reserve() {
    let mut game = Game::prepared_with_seed(5); // shared roll 3
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    let mut record = serde_json::to_value(&game).unwrap();
    record["transfer"] =
        serde_json::json!({"Authorizing":{"giver":"M26","recipient":"Directorio","amount":2}});
    record["revision"] = (game.observe().revision + 1).into();
    let mut reserved: Game = serde_json::from_value(record).unwrap();
    reserved.validate().unwrap();
    let view = reserved.observe();
    assert_eq!(view.owner, M26);
    assert_eq!(view.transfer_terms[0].limit, 2);
    let authorize: Vec<_> = actions(&reserved)
        .into_iter()
        .filter(|a| a.starts_with("authorize:"))
        .collect();
    assert_eq!(authorize, ["authorize:0", "authorize:1", "authorize:2"]);
    choose(&mut reserved, "authorize:2");
    assert_eq!(reserved.observe().resources, [3, 1, 7, 15]);
}
