//! R 1.5.1, 1.8, 2.3.5 and 3.3.5. A willing giver can fund the one required payment.
use cuba_libre_unfolded::{Faction, Game, Transfer, reports::Reports};

const SYNDICATE: Faction = Faction::Syndicate;
const M26: Faction = Faction::M26;

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

fn card_18() -> Game {
    let mut game = Game::opening_with_seed(2);
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
    ] {
        choose(&mut game, action);
    }
    game
}

fn underfunded_construct() -> Game {
    let mut game = card_18();
    // Kidnap leaves Government 1 and 26July 4 after Terror. Government gets 12
    // from Syndicate and pays 2 to close Havana's Casino, leaving [11, 4, 5, 3].
    for action in [
        "government:assault",
        "transfer",
        "ask:3:12",
        "authorize:12",
        "assault:havana:syndicate",
        "end-assault",
        "syndicate:limited-operation",
    ] {
        choose(&mut game, action);
    }
    game
}

#[test]
fn operation_only_construct_can_transfer_its_last_resource_and_finish() {
    let mut game = card_18();
    for action in [
        "government:pass",
        "syndicate:construct",
        "transfer",
        "give:1:15",
        "transfer",
        "ask:2:1",
        "authorize:1",
    ] {
        choose(&mut game, action);
    }
    assert_eq!(game.observe().resources, [2, 19, 4, 1]);
    assert!(game.observe().construct_paid.is_empty());
    for action in ["transfer", "give:0:1", "end-construct"] {
        choose(&mut game, action);
    }
    let complete = game.observe();
    assert_eq!(complete.resources, [3, 19, 4, 0]);
    assert_eq!(complete.eligible, [true, true, true, false]);
    assert!(complete.construct_paid.is_empty());
}

#[test]
fn construct_uses_the_amount_the_giver_authorizes() {
    let mut game = underfunded_construct();
    assert_eq!(game.observe().resources, [11, 4, 5, 3]);
    let unfunded = game.observe();
    let payment = unfunded
        .choices
        .iter()
        .find(|choice| choice.action() == "construct:havana:open")
        .unwrap();
    assert!(!payment.supported);
    assert_eq!(payment.remedies, ["Request a Resource transfer"]);
    refuse(&mut game, SYNDICATE, "end-construct");
    choose(&mut game, "transfer");
    assert_eq!(game.observe().transfer_terms.len(), 6);
    for terms in &game.observe().transfer_terms {
        assert!(terms.giver == SYNDICATE || terms.recipient == SYNDICATE);
    }
    refuse(&mut game, Faction::Government, "give:1:1");
    assert!(refuse(&mut game, SYNDICATE, "give:1:1").contains("Pay first"));
    assert!(!game.observe().effected);
    choose(&mut game, "ask:1:2");
    let pending = game.observe();
    assert_eq!(pending.owner, M26);
    assert_eq!(pending.executing, Some(SYNDICATE));
    assert_eq!(pending.resources, [11, 4, 5, 3]);
    assert_eq!(
        pending.transfer,
        Some(Transfer::Authorizing {
            giver: M26,
            recipient: SYNDICATE,
            amount: 2,
            waited: false,
        })
    );
    assert_eq!(pending.transfer_terms[0].limit, 4);
    assert!(refuse(&mut game, M26, "authorize:5").contains("more than 4"));
    assert!(refuse(&mut game, SYNDICATE, "authorize:2").contains("decision owner"));
    let mut withdrawn = game.clone();
    withdrawn
        .submit(
            SYNDICATE,
            pending.revision,
            &format!("{}:back-to-choice", pending.revision),
        )
        .unwrap();
    assert_eq!(withdrawn.observe().transfer, None);
    assert_eq!(withdrawn.observe().resources, pending.resources);
    assert_eq!(withdrawn.observe().executing, None);

    for no_move in ["decline", "authorize:0"] {
        let mut tried = game.clone();
        choose(&mut tried, no_move);
        assert_eq!(tried.observe().resources, pending.resources);
        assert_eq!(tried.observe().owner, SYNDICATE);
        choose(&mut tried, "back-to-choice");
        assert_eq!(tried.observe().resources, pending.resources);
    }

    for (amount, authorized, paid) in [
        (2, [11, 2, 5, 5], [11, 2, 5, 0]),
        (3, [11, 1, 5, 6], [11, 1, 5, 1]),
    ] {
        let mut funded = game.clone();
        choose(&mut funded, &format!("authorize:{amount}"));
        assert_eq!(funded.observe().owner, SYNDICATE);
        assert_eq!(funded.observe().resources, authorized);
        refuse(&mut funded, SYNDICATE, "back-to-choice");
        assert!(
            funded
                .submit(
                    M26,
                    pending.revision,
                    &format!("{}:authorize:{amount}", pending.revision)
                )
                .unwrap_err()
                .contains("Stale")
        );
        refuse(&mut funded, M26, &format!("authorize:{amount}"));
        choose(&mut funded, "transfer");
        // A give that leaves exactly 5 is legal; one more would prevent payment.
        let spare = authorized[3] - 5;
        assert!(
            refuse(&mut funded, SYNDICATE, &format!("give:1:{}", spare + 1)).contains("Pay first")
        );
        let mut exact = funded.clone();
        choose(&mut exact, &format!("give:1:{spare}"));
        assert_eq!(exact.observe().resources[3], 5);
        choose(&mut exact, "construct:havana:open");
        assert_eq!(exact.observe().resources[3], 0);
        choose(&mut exact, "end-construct");

        choose(&mut funded, "back");
        choose(&mut funded, "construct:havana:open");
        assert_eq!(funded.observe().resources, paid);
        choose(&mut funded, "transfer");
        choose(&mut funded, &format!("give:1:{}", paid[3]));
        choose(&mut funded, "end-construct");
        assert_eq!(funded.observe().resources, [11, 2, 5, 0]);
        refuse(&mut funded, SYNDICATE, "transfer");
    }
}

#[test]
fn construct_reports_transfers_in_order_even_when_the_same_giver_replies_twice() {
    let mut game = underfunded_construct();
    let mut reports = Reports::new(&game.observe());
    for action in [
        "transfer",
        "ask:1:2",
        "authorize:3",
        "construct:havana:open",
        "transfer",
        "ask:1:1",
        "authorize:1",
        "transfer",
        "give:1:2",
        "end-construct",
    ] {
        choose(&mut game, action);
        reports.record(&game.observe());
    }
    let whole = reports.all();
    let report = whole.last().unwrap();
    assert_eq!(report.faction, SYNDICATE);
    assert_eq!(report.recorded, Some("Limited Operation"));
    assert_eq!(
        report
            .articles
            .iter()
            .map(|a| a.title.as_str())
            .collect::<Vec<_>>(),
        [
            "26July gives Syndicate 3 Resources",
            "Construct in Havana",
            "26July gives Syndicate 1 Resource",
            "Syndicate gives 26July 2 Resources",
        ]
    );
    assert_eq!(
        report.articles[0].lines,
        [
            "Syndicate asked for 2 Resources.",
            "26July 4 → 1, Syndicate 3 → 6.",
        ]
    );
    assert_eq!(
        report.articles[1].lines[0],
        "Syndicate pays 5 Resources, 6 → 1."
    );
    assert_eq!(
        report.articles[2].lines,
        [
            "Syndicate asked for 1 Resource.",
            "26July 1 → 0, Syndicate 1 → 2.",
        ]
    );
    assert_eq!(report.articles[3].lines, ["Syndicate 2 → 0, 26July 0 → 2."]);
}
