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
