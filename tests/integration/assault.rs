//! Matched Santiago positions worked from R 3.1, 3.2.4, 6.3.1 and #130.
use cuba_libre_unfolded::{Faction, Game, Kind, State};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap_or_else(|error| panic!("{action}: {error}"));
}

#[test]
fn funded_santiago_assault_returns_only_active_guerrillas_then_hands_off() {
    let mut funded = Game::prepared_with_seed(2);
    for action in ["kidnap", "kidnap:roll", "terror:santiago", "transfer"] {
        choose(&mut funded, action);
    }
    let transfer = funded.observe();
    assert_eq!(transfer.resources, [1, 4, 5, 15]);
    assert_eq!(transfer.owner, Faction::M26);
    choose(&mut funded, "give:0:1");
    choose(&mut funded, "end-terror");
    assert_eq!(funded.observe().resources, [2, 3, 5, 15]);
    let random = funded.random_state();

    for (event, steps, returned, underground) in [
        ("event:shaded", ["resolve-shaded", ""], 2, 0),
        (
            "event:unshaded",
            ["rally:beneficiary:26july", "rally:sierra-maestra:place:0"],
            1,
            1,
        ),
    ] {
        let mut game = funded.clone();
        choose(&mut game, event);
        for step in steps.into_iter().filter(|step| !step.is_empty()) {
            choose(&mut game, step);
        }
        choose(&mut game, "advance-card");
        choose(&mut game, "government:assault");
        choose(&mut game, "assault:areas:cities");
        choose(&mut game, "assault:space:santiago");
        let pending = game.observe();
        assert_eq!(pending.executing, Some(Faction::Government));
        assert_eq!(pending.resources, [2, 3, 5, 15]);
        let payment = pending
            .choices
            .iter()
            .find(|choice| choice.action() == "assault:santiago:26july")
            .unwrap();
        assert!(payment.explanation.contains("2 Troops + 1 Police = 3"));
        let preview = payment.preview.as_ref().unwrap();
        assert_eq!(preview.resources[0].from, 2);
        assert_eq!(preview.resources[0].to, 0);
        assert_eq!(preview.removed.len(), 1);
        assert_eq!(
            (
                preview.removed[0].kind,
                preview.removed[0].state,
                preview.removed[0].count
            ),
            (Kind::Guerrilla, State::Active, returned)
        );
        assert!(
            game.submit(
                pending.owner,
                pending.revision,
                &format!(
                    "{}:assault:santiago:26july:{}",
                    pending.revision,
                    returned + 1
                )
            )
            .is_err()
        );
        for rejected in [0, returned - 1] {
            assert!(
                game.submit(
                    pending.owner,
                    pending.revision,
                    &format!("{}:assault:santiago:26july:{rejected}", pending.revision),
                )
                .is_err()
            );
        }
        assert_eq!(game.observe(), pending);
        choose(&mut game, payment.action());
        let paid = game.observe();
        assert_eq!(paid.resources, [0, 3, 5, 15]);
        assert_eq!(paid.executing, Some(Faction::Government));
        assert_eq!(
            paid.map_piece_count,
            pending.map_piece_count - u16::from(returned)
        );
        assert_eq!(
            paid.available_piece_count,
            pending.available_piece_count + u16::from(returned)
        );
        let available_m26 = |view: &cuba_libre_unfolded::Observation| {
            view.available
                .iter()
                .find(|p| p.faction == Faction::M26 && p.kind == Kind::Guerrilla)
                .unwrap()
                .count
        };
        assert_eq!(available_m26(&paid), available_m26(&pending) + returned);
        let santiago = paid.spaces.iter().find(|s| s.id == "santiago").unwrap();
        assert_eq!(santiago.control(), Some(Faction::Government));
        assert_eq!(
            santiago
                .pieces
                .iter()
                .filter(|p| p.faction == Faction::M26)
                .map(|p| (p.kind, p.state, p.count))
                .collect::<Vec<_>>(),
            if underground == 0 {
                vec![]
            } else {
                vec![(Kind::Guerrilla, State::Underground, 1)]
            }
        );
        assert_eq!(paid.standings, pending.standings);
        assert!(
            game.submit(pending.owner, pending.revision, &payment.id)
                .is_err()
        );
        assert!(
            game.submit(
                paid.owner,
                paid.revision,
                &format!("{}:{}", paid.revision, payment.action())
            )
            .is_err()
        );
        assert_eq!(game.observe(), paid);
        game.validate().unwrap();

        choose(&mut game, "end-assault");
        let handoff = game.observe();
        assert_eq!(handoff.owner, Faction::Syndicate);
        assert_eq!(handoff.executing, None);
        let next = handoff.next.unwrap();
        assert_eq!(next.role, "2nd Eligible");
        assert_eq!(next.options, ["Pass", "Limited Operation"]);
        assert_eq!(
            handoff
                .choices
                .iter()
                .map(|c| c.action())
                .collect::<Vec<_>>(),
            ["syndicate:pass", "syndicate:limited-operation"]
        );
        assert_eq!(handoff.resources, paid.resources);
        assert_eq!(handoff.spaces, paid.spaces);
        assert_eq!(game.random_state(), random);
        game.validate().unwrap();
    }
}

#[test]
fn havana_assault_closes_the_targeted_casino_without_returning_it() {
    let mut game = Game::prepared_with_seed(2);
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:4",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
        "assault:areas:cities",
        "assault:space:havana",
    ] {
        choose(&mut game, action);
    }
    let before = game.observe();
    assert_eq!(before.resources, [5, 0, 5, 15]);
    assert_eq!(
        before
            .choices
            .iter()
            .map(|c| c.action())
            .collect::<Vec<_>>(),
        [
            "assault:havana:directorio",
            "assault:havana:syndicate",
            "assault:havana:directorio+syndicate",
            "assault:back",
        ]
    );
    let directorio = &before.choices[0];
    assert!(
        directorio
            .explanation
            .contains("Underground Guerrillas survive")
    );
    let zero = directorio.preview.as_ref().unwrap();
    assert!(zero.flips.is_empty() && zero.removed.is_empty());
    assert_eq!(zero.resources[0].to, 3);
    let target = before
        .choices
        .iter()
        .find(|choice| choice.action() == "assault:havana:syndicate")
        .unwrap();
    assert!(target.explanation.contains("6 Troops + 4 Police = 10"));
    let preview = target.preview.as_ref().unwrap();
    assert_eq!(&before.choices[2].preview, &target.preview);
    assert_eq!(preview.resources[0].from, 5);
    assert_eq!(preview.resources[0].to, 3);
    assert!(preview.removed.is_empty());
    let flip = &preview.flips[0];
    assert_eq!(
        (
            &*flip.space,
            flip.faction,
            flip.kind,
            flip.from,
            flip.to,
            flip.count
        ),
        (
            "havana",
            Faction::Syndicate,
            Kind::Casino,
            State::Open,
            State::Closed,
            1
        ),
    );

    for action in [
        "assault:havana:none",
        "assault:havana:syndicate:0",
        "assault:havana:syndicate:2",
    ] {
        assert!(
            game.submit(
                before.owner,
                before.revision,
                &format!("{}:{action}", before.revision)
            )
            .is_err()
        );
    }
    assert_eq!(game.observe(), before);
    choose(&mut game, target.action());
    let after = game.observe();
    assert_eq!(after.resources, [3, 0, 5, 15]);
    assert_eq!(after.map_piece_count, before.map_piece_count);
    assert_eq!(after.available, before.available);
    let havana = &after.spaces[0];
    assert_eq!(havana.control(), Some(Faction::Government));
    assert_eq!(
        havana
            .pieces
            .iter()
            .find(|p| p.kind == Kind::Casino)
            .unwrap()
            .state,
        State::Closed,
    );
    assert_eq!(
        havana
            .pieces
            .iter()
            .find(|p| p.faction == Faction::Directorio)
            .unwrap()
            .count,
        2,
    );
    let standing = &after.standings[Faction::Syndicate.index()];
    assert_eq!(standing.score, 2);
    assert_eq!(standing.margin, -15);
    game.validate().unwrap();
}

#[test]
fn all_thirteen_spaces_allow_inspection_and_pay_once_even_when_no_pieces_can_be_affected() {
    let mut game = Game::prepared_with_seed(2);
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:4",
        "end-terror",
        "event:unshaded",
        "rally:beneficiary:26july",
        "rally:sierra-maestra:place:4",
        "advance-card",
        "government:assault",
    ] {
        choose(&mut game, action);
    }
    let original = game.observe();
    assert!(
        original
            .choices
            .iter()
            .any(|c| c.action() == "end-assault" && c.supported)
    );

    for (area, expected) in [
        ("cities", &["havana", "camaguey-city", "santiago"][..]),
        (
            "provinces",
            &[
                "pinar-del-rio",
                "la-habana",
                "matanzas",
                "las-villas",
                "camaguey-province",
                "oriente",
                "sierra-maestra",
            ][..],
        ),
        (
            "economic-centers",
            &[
                "ec-pinar-la-habana",
                "ec-las-villas-camaguey",
                "ec-oriente-sierra",
            ][..],
        ),
    ] {
        choose(&mut game, &format!("assault:areas:{area}"));
        let spaces = game.observe();
        let offered = spaces
            .choices
            .iter()
            .filter_map(|c| c.action().strip_prefix("assault:space:"))
            .collect::<Vec<_>>();
        assert_eq!(offered, expected);
        for space in offered {
            choose(&mut game, &format!("assault:space:{space}"));
            let targets = game.observe();
            assert!(targets.choices.iter().any(|c| c.preview.is_some()));
            match space {
                "las-villas" => {
                    assert!(targets.choices[0].explanation.contains("3 Troops ÷ 2 = 1"))
                }
                "pinar-del-rio" => {
                    assert!(targets.choices[0].explanation.contains("= 0 in Forest"))
                }
                "la-habana" => assert!(targets.choices[0].explanation.contains("= 0 in Grassland")),
                "ec-pinar-la-habana" => assert!(
                    targets.choices[0]
                        .explanation
                        .contains("= 0 in an Economic Center")
                ),
                _ => {}
            }
            choose(&mut game, "assault:back");
        }
        choose(&mut game, "assault:back");
        assert_eq!(game.observe().resources, original.resources);
        assert_eq!(game.observe().spaces, original.spaces);
    }

    for (area, space, target, balance) in [
        ("provinces", "sierra-maestra", "26july", 3),
        ("economic-centers", "ec-oriente-sierra", "none", 1),
    ] {
        choose(&mut game, &format!("assault:areas:{area}"));
        choose(&mut game, &format!("assault:space:{space}"));
        let pending = game.observe();
        let action = format!("assault:{space}:{target}");
        let choice = pending
            .choices
            .iter()
            .find(|c| c.action() == action)
            .unwrap();
        let preview = choice.preview.as_ref().unwrap();
        assert_eq!(preview.resources[0].to, balance);
        assert!(preview.removed.is_empty() && preview.flips.is_empty());
        if space == "sierra-maestra" {
            assert!(choice.explanation.contains("4 Underground"));
            assert!(choice.explanation.contains("protect their faction's Bases"));
        }
        let illegal = format!("{}:{action}:1", pending.revision);
        assert!(
            game.submit(pending.owner, pending.revision, &illegal)
                .is_err()
        );
        choose(&mut game, &action);
        let settled = game.observe();
        assert_eq!(settled.resources[0], balance);
        assert_eq!(settled.spaces, original.spaces);
        assert_eq!(settled.available, original.available);
        game.validate().unwrap();

        choose(&mut game, &format!("assault:areas:{area}"));
        let updated = game.observe();
        assert!(
            updated
                .choices
                .iter()
                .all(|c| c.action() != format!("assault:space:{space}"))
        );
        choose(&mut game, "assault:back");
    }

    choose(&mut game, "assault:areas:cities");
    choose(&mut game, "assault:space:havana");
    let pending = game.observe();
    let payment = pending
        .choices
        .iter()
        .find(|c| c.action() == "assault:havana:syndicate")
        .unwrap();
    assert!(!payment.supported);
    assert!(payment.preview.is_none());
    let error = game
        .submit(pending.owner, pending.revision, &payment.id)
        .unwrap_err();
    assert!(error.contains("Government has 1 Resource and cannot pay 2 Resources"));
    assert_eq!(game.observe(), pending);
    for action in ["assault:back", "assault:back", "end-assault"] {
        choose(&mut game, action);
    }
    assert_eq!(game.observe().resources[0], 1);
    assert_eq!(game.observe().owner, Faction::Syndicate);
    game.validate().unwrap();
}
