//! R 1.5.1, 1.8, 2.3.5 and 3.3.5. A willing giver can fund the one required payment.
use cuba_libre_unfolded::{
    ConstructFundingReply, Faction, Game, Transfer, reports::Reports, save::Session,
};

const SYNDICATE: Faction = Faction::Syndicate;
const M26: Faction = Faction::M26;
const CARD_17: &[&str] = &[
    "terror",
    "kidnap",
    "kidnap:roll",
    "terror:santiago",
    "end-terror",
    "event:shaded",
    "resolve-shaded",
    "advance-card",
];
// Kidnap leaves Government 1 and 26July 4 after Terror. Government gets 12
// from Syndicate and pays 2 to close Havana's Casino, leaving [11, 4, 5, 3].
const UNDERFUNDED: &[&str] = &[
    "government:assault",
    "transfer",
    "ask:3:12",
    "authorize:12",
    "assault:havana:syndicate",
    "end-assault",
    "syndicate:limited-operation",
];

fn choose(game: &mut Game, action: &str) -> Option<ConstructFundingReply> {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap_or_else(|error| panic!("{action}: {error}"))
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
    for action in CARD_17 {
        choose(&mut game, action);
    }
    game
}

fn underfunded_construct() -> Game {
    let mut game = card_18();
    for action in UNDERFUNDED {
        choose(&mut game, action);
    }
    game
}

#[test]
fn government_pass_and_pact_status_remain_accurate_during_construct() {
    let mut game = Game::opening_with_seed(2);
    // Bring Government from 1 to 5 before Pass. A later gift must not rewrite
    // that 5 → 8 Pass or make it look like a loss of Resources.
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:4",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:pass",
    ] {
        choose(&mut game, action);
    }
    let handoff = game.observe();
    assert!(
        handoff
            .cards
            .current
            .words
            .status
            .contains("Event open to Syndicate")
    );
    assert!(
        handoff
            .choices
            .iter()
            .any(|choice| { choice.label.contains("Pact of Caracas") && !choice.supported })
    );
    for action in ["syndicate:construct", "transfer", "ask:0:4", "authorize:4"] {
        choose(&mut game, action);
    }
    let constructing = game.observe();
    assert_eq!(constructing.resources[0], 4);
    let pass = &constructing.recorded_resources[0];
    assert_eq!(
        (pass.faction, pass.from, pass.to),
        (Faction::Government, 5, 8)
    );
    assert_eq!(
        constructing.cards.current.words.status,
        "Event closed: Syndicate chose Operation Only"
    );
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
    // Government's Pass adds 3 (R 2.3.3).
    assert_eq!(game.observe().resources, [4, 19, 4, 1]);
    assert!(game.observe().construct_paid.is_empty());
    for action in ["transfer", "give:0:1", "end-construct"] {
        choose(&mut game, action);
    }
    let complete = game.observe();
    assert_eq!(complete.resources, [5, 19, 4, 0]);
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
    for action in ["transfer", "ask:1:2"] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    let after_ask = game.observe().revision;
    for action in [
        "authorize:3",
        "construct:havana:open",
        "transfer",
        "ask:1:1",
        "authorize:1",
        "transfer",
        "give:1:2",
        "end-construct",
    ] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
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
    let continued = reports.since(after_ask);
    assert_eq!(
        continued.last().unwrap().articles[0].lines,
        ["26July 4 → 1, Syndicate 3 → 6."]
    );
}

#[test]
fn construct_reports_each_open_casino_change_even_when_the_total_returns_to_two() {
    let mut game = underfunded_construct();
    let start = game.observe();
    assert_eq!(start.standings[3].score, 2);
    let mut reports = Reports::new(&start);
    for action in ["transfer", "ask:1:2", "authorize:2"] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    let before_payment = game.observe().revision;
    choose(&mut game, "construct:havana:open");
    assert_eq!(game.observe().standings[3].score, 3);
    reports.record(&game.observe(), None);
    for action in [
        "remove",
        "close-casino:pinar-del-rio:open-casino:1",
        "done-removal",
        "end-construct",
    ] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    let end = game.observe();
    assert_eq!(end.standings[3].score, 2);
    assert_eq!(
        end.spaces
            .iter()
            .find(|space| space.id == "pinar-del-rio")
            .unwrap()
            .control(),
        None
    );
    for (pages, first_article) in [(reports.all(), 1), (reports.since(before_payment), 0)] {
        let report = pages.last().unwrap();
        let opened = &report.articles[first_article];
        let closed = &report.articles[first_article + 1];
        assert_eq!(opened.title, "Construct in Havana");
        assert!(
            opened
                .lines
                .iter()
                .any(|line| line == "Syndicate open Casinos: 2 → 3.")
        );
        assert_eq!(closed.title, "Casino management in Pinar del Río");
        assert!(
            closed
                .lines
                .iter()
                .any(|line| line == "Syndicate open Casinos: 3 → 2.")
        );
        assert!(
            closed
                .lines
                .iter()
                .any(|line| line == "Pinar del Río Control: Syndicate → none.")
        );
    }
}

#[test]
fn limited_construct_holds_insufficient_authorizations_until_they_can_fund_a_payment() {
    let mut game = underfunded_construct();
    let mut reports = Reports::new(&game.observe());
    for action in ["transfer", "ask:1:2"] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    let pending = game.observe();
    let funding = pending.construct_funding.as_ref().unwrap();
    assert_eq!(funding.shortfall, 2);
    assert!(funding.authorizations.is_empty());
    let reply = choose(&mut game, "authorize:1");
    reports.record(&game.observe(), reply);
    assert_eq!(game.observe().resources, [11, 4, 5, 3]);
    assert_eq!(game.observe().owner, SYNDICATE);
    let funding = game.observe().construct_funding.unwrap();
    assert_eq!(funding.shortfall, 1);
    assert_eq!(funding.authorizations[0].giver, M26);
    assert_eq!(funding.authorizations[0].amount, 1);
    for pages in [reports.all(), reports.since(pending.revision)] {
        let report = pages.last().expect("the held reply has a report");
        assert!(report.articles.is_empty());
        assert_eq!(
            report.summary.as_deref(),
            Some(
                "1 conditionally authorized · 1 still needed. Resources move only when authorizations fund a legal payment."
            )
        );
        assert!(
            report
                .standings
                .iter()
                .all(|row| row.resources.from == row.resources.to)
        );
    }
    refuse(&mut game, SYNDICATE, "construct:havana:open");
    let mut abandoned = game.clone();
    let mut abandoned_reports = reports.clone();
    let reply = choose(&mut abandoned, "back-to-choice");
    abandoned_reports.record(&abandoned.observe(), reply);
    assert!(abandoned_reports.all().is_empty());

    let mut management = game.clone();
    choose(&mut management, "remove");
    let close = "close-casino:pinar-del-rio:open-casino:1";
    assert!(refuse(&mut management, SYNDICATE, close).contains("Fund and pay for Construct first"));
    choose(&mut management, "done-removal");
    assert_eq!(
        management.observe().construct_funding,
        game.observe().construct_funding
    );
    for action in [
        "transfer",
        "ask:2:1",
        "authorize:1",
        "remove",
        close,
        "done-removal",
    ] {
        choose(&mut management, action);
    }
    refuse(&mut management, SYNDICATE, "back-to-choice");
    assert_eq!(management.observe().resources, [11, 3, 4, 5]);
    choose(&mut management, "construct:havana:open");
    choose(&mut management, "end-construct");
    assert!(
        game.submit(
            M26,
            pending.revision,
            &format!("{}:authorize:1", pending.revision)
        )
        .unwrap_err()
        .contains("Stale")
    );
    refuse(&mut game, M26, "authorize:1");

    let mut same_giver = game.clone();
    let mut same_giver_reports = reports.clone();
    let reply = choose(&mut same_giver, "transfer");
    same_giver_reports.record(&same_giver.observe(), reply);
    assert!(refuse(&mut same_giver, SYNDICATE, "ask:1:4").contains("more than 3"));
    let reply = choose(&mut same_giver, "ask:1:3");
    same_giver_reports.record(&same_giver.observe(), reply);
    assert_eq!(same_giver.observe().transfer_terms[0].limit, 3);
    refuse(&mut same_giver, M26, "authorize:4");
    let mut maximum = same_giver.clone();
    choose(&mut maximum, "authorize:3");
    assert_eq!(maximum.observe().resources, [11, 0, 5, 7]);
    let reply = choose(&mut same_giver, "authorize:1");
    same_giver_reports.record(&same_giver.observe(), reply);
    assert_eq!(same_giver.observe().resources, [11, 2, 5, 5]);
    let same_giver_report = same_giver_reports.all().pop().unwrap();
    assert_eq!(same_giver_report.articles.len(), 2);
    assert_eq!(
        same_giver_report.articles[0].lines,
        [
            "Syndicate asked for 2 Resources.",
            "26July 4 → 3, Syndicate 3 → 4.",
        ]
    );
    assert_eq!(
        same_giver_report.articles[1].lines,
        [
            "Syndicate asked for 3 Resources.",
            "26July 3 → 2, Syndicate 4 → 5.",
        ]
    );

    let mut withdrawn = game.clone();
    for action in ["transfer", "ask:2:5"] {
        choose(&mut withdrawn, action);
    }
    let pending = withdrawn.observe();
    withdrawn
        .submit(
            SYNDICATE,
            pending.revision,
            &format!("{}:back-to-choice", pending.revision),
        )
        .unwrap();
    assert_eq!(withdrawn.observe().executing, None);
    assert_eq!(withdrawn.observe().resources, [11, 4, 5, 3]);
    assert!(withdrawn.observe().construct_funding.is_none());
    assert!(
        withdrawn
            .submit(
                Faction::Directorio,
                pending.revision,
                &format!("{}:authorize:2", pending.revision)
            )
            .unwrap_err()
            .contains("Stale")
    );

    for answer in ["decline", "authorize:0"] {
        let mut abandoned = game.clone();
        for action in ["transfer", "ask:2:5", answer] {
            choose(&mut abandoned, action);
        }
        assert_eq!(abandoned.observe().resources, [11, 4, 5, 3]);
        choose(&mut abandoned, "back-to-choice");
        assert_eq!(abandoned.observe().executing, None);
        assert_eq!(abandoned.observe().resources, [11, 4, 5, 3]);
        choose(&mut abandoned, "syndicate:limited-operation");
        choose(&mut abandoned, "transfer");
        choose(&mut abandoned, "ask:1:4");
        assert_eq!(abandoned.observe().transfer_terms[0].limit, 4);
    }

    for action in [
        "transfer",
        "ask:2:5",
        "decline",
        "transfer",
        "ask:2:5",
        "authorize:2",
    ] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    assert_eq!(game.observe().resources, [11, 3, 3, 6]);
    assert!(game.observe().construct_paid.is_empty());
    refuse(&mut game, SYNDICATE, "back-to-choice");
    for action in ["construct:havana:open", "end-construct"] {
        let reply = choose(&mut game, action);
        reports.record(&game.observe(), reply);
    }
    assert_eq!(game.observe().resources, [11, 3, 3, 1]);
    let whole = reports.all();
    let articles = &whole.last().unwrap().articles;
    assert_eq!(articles[0].title, "No Resources moved");
    assert!(
        articles[0]
            .lines
            .contains(&"Directorio answered 1 request.".to_owned())
    );
    assert_eq!(articles[1].title, "26July gives Syndicate 1 Resource");
    assert_eq!(
        articles[1].lines,
        [
            "Syndicate asked for 2 Resources.",
            "26July 4 → 3, Syndicate 3 → 4."
        ]
    );
    assert_eq!(articles[2].title, "Directorio gives Syndicate 2 Resources");
    assert_eq!(
        articles[2].lines,
        [
            "Syndicate asked for 5 Resources.",
            "Directorio 5 → 3, Syndicate 4 → 6."
        ]
    );
    assert_eq!(articles[3].title, "Construct in Havana");
}

#[test]
fn construct_settlement_reports_follow_only_saved_replies_in_live_play_and_replay() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("match.json");
    let mut session = Session::create_seeded(&path, 2).unwrap();
    let play = |session: &mut Session, action: &str| {
        let seen = session.observe();
        let offered = seen
            .choices
            .iter()
            .find(|choice| choice.action() == action)
            .unwrap();
        session
            .submit(offered.owner, seen.revision, &offered.id)
            .unwrap();
    };
    for action in CARD_17.iter().chain(UNDERFUNDED).chain(&[
        "transfer",
        "ask:1:2",
        "authorize:1",
        "transfer",
        "ask:2:5",
        "decline",
        "transfer",
        "ask:2:5",
    ]) {
        play(&mut session, action);
    }

    let mut stale_writer = Session::open(&path, None).unwrap();
    let before = stale_writer.observe();
    let reports_before = stale_writer.reports();
    let reply = format!("{}:authorize:2", before.revision);
    play(&mut session, "authorize:2");
    let save_bytes = std::fs::read(&path).unwrap();
    let refusal = stale_writer
        .submit(Faction::Directorio, before.revision, &reply)
        .unwrap_err();
    assert!(matches!(
        refusal,
        cuba_libre_unfolded::save::Submit::Save(_)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), save_bytes);
    assert_eq!(stale_writer.observe(), before);
    assert_eq!(stale_writer.reports(), reports_before);
    assert!(stale_writer.reports_since(before.revision).is_empty());

    let since = session.reports_since(before.revision);
    let report = since.last().unwrap();
    assert_eq!(report.from_revision, before.revision);
    assert_eq!(report.articles.len(), 2);
    assert_eq!(
        report.articles[0].lines,
        [
            "Syndicate asked for 2 Resources.",
            "26July 4 → 3, Syndicate 3 → 4.",
        ]
    );
    assert_eq!(
        report.articles[1].lines,
        [
            "Syndicate asked for 5 Resources.",
            "Directorio 5 → 3, Syndicate 4 → 6.",
        ]
    );
    let reopened = Session::open(&path, None).unwrap();
    assert_eq!(reopened.reports(), session.reports());
    assert_eq!(reopened.reports_since(before.revision), since);
}
