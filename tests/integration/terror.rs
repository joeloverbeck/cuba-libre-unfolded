use cuba_libre_unfolded::{Faction, Game, Kind, State};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap();
}

fn transfer_resources(game: &mut Game, giver: usize, amount: u8) {
    choose(game, "transfer");
    choose(game, &format!("ask:{giver}:{amount}"));
    choose(game, &format!("authorize:{amount}"));
}

// Accepted deacd635 handoff witness: DR gives 1 by Resource transfer, Oriente Terror,
// no Kidnap.
// R 3.3.4: cost 1, activate 1 Underground, Terror, toward Active Opposition.
#[test]
fn oriente_terror_after_a_resource_transfer_updates_pieces_resources_standing_and_next_permission()
{
    let mut game = Game::prepared();
    transfer_resources(&mut game, 2, 1);
    choose(&mut game, "terror:oriente");
    let view = game.observe();
    assert_eq!(view.resources, [6, 0, 4, 15]);
    let oriente = view.spaces.iter().find(|s| s.id == "oriente").unwrap();
    assert_eq!((oriente.support, oriente.terror), (-2, 1));
    assert!(oriente.pieces.iter().any(|p| p.faction == Faction::M26
        && p.kind == Kind::Guerrilla
        && p.state == State::Active
        && p.count == 1));
    assert_eq!(view.standings[1].score, 9);
    assert_eq!(view.terror_available, 19);
    assert_eq!((view.map_piece_count, view.available_piece_count), (29, 57));
    choose(&mut game, "end-terror");
    assert_eq!(
        game.observe().next.unwrap().options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    assert_eq!(game.observe().eligible, [true; 4]);
}

// Six independent source-table rows; seeds only select actual shared RNG faces.
#[test]
fn all_six_rolls_fund_the_owed_terror_and_change_affordable_continuations() {
    for (seed, die, government, remaining, both) in [
        (19, 1, 5, 0, None),
        (0, 2, 4, 1, Some(0)),
        (5, 3, 3, 2, Some(1)),
        (3, 4, 2, 3, Some(2)),
        (2, 5, 1, 4, Some(3)),
        (1, 6, 0, 5, Some(4)),
    ] {
        // Diagnostic seed input, not a selected outcome or alternate roll API.
        let mut fixture = serde_json::to_value(Game::prepared()).unwrap();
        fixture["random"] = seed.into();
        let mut game: Game = serde_json::from_value(fixture).unwrap();
        choose(&mut game, "kidnap");
        choose(&mut game, "kidnap:roll");
        let view = game.observe();
        assert_eq!(view.resources, [government, die, 5, 15]);
        assert_eq!(view.owed_terror.as_deref(), Some("santiago"));
        assert_eq!(view.special_activity.as_deref(), Some("Kidnap"));
        assert_ne!(game.random_state(), seed);
        let rolled = game.clone();
        assert!(
            game.submit(
                Faction::M26,
                view.revision,
                &format!("{}:end-terror", view.revision)
            )
            .is_err()
        );
        assert_eq!(game, rolled);
        choose(&mut game, "terror:santiago");
        let view = game.observe();
        assert_eq!(view.resources, [government, remaining, 5, 15]);
        assert_eq!(view.standings[1].score, 8);
        assert_eq!(view.owed_terror, None);
        assert_eq!(
            view.choices
                .iter()
                .any(|c| c.id.ends_with("terror:oriente") && c.supported),
            both.is_some()
        );
        if let Some(remaining) = both {
            choose(&mut game, "terror:oriente");
            assert_eq!(game.observe().resources, [government, remaining, 5, 15]);
            assert_eq!(game.observe().standings[1].score, 10);
        }
        choose(&mut game, "end-terror");
        assert_eq!(
            game.observe().next.unwrap().options,
            ["Pass (+1 if executed later)", "Limited Operation", "Event"]
        );
    }
}

#[test]
fn owed_santiago_limits_spending_only_until_its_terror_is_resolved() {
    let mut game = Game::prepared_with_seed(0); // actual shared roll 2
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    choose(&mut game, "transfer");
    let before = game.clone();
    let v = game.observe();
    assert!(
        game.submit(v.owner, v.revision, &format!("{}:give:2:2", v.revision))
            .is_err()
    );
    assert_eq!(game, before);
    choose(&mut game, "give:2:1");
    assert_eq!(game.observe().resources, [4, 1, 6, 15]);
    let v = game.observe();
    assert!(
        !v.choices
            .iter()
            .any(|c| c.id.ends_with("terror:oriente") && c.supported)
    );
    choose(&mut game, "terror:santiago");
    transfer_resources(&mut game, 3, 1);
    choose(&mut game, "transfer");
    choose(&mut game, "give:2:1");
    choose(&mut game, "end-terror");
    assert_eq!(game.observe().resources, [4, 0, 7, 14]);
}

#[test]
fn actual_resource_transfers_allow_both_target_orders_and_early_during_late_or_no_kidnap() {
    for order in [["santiago", "oriente"], ["oriente", "santiago"]] {
        for timing in [Some(0), Some(1), Some(2), None] {
            let mut game = Game::prepared_with_seed(5); // shared roll 3
            transfer_resources(&mut game, 2, 2);
            for (index, target) in order.into_iter().enumerate() {
                if timing == Some(index) {
                    choose(&mut game, "kidnap");
                    choose(&mut game, "kidnap:roll");
                }
                choose(&mut game, &format!("terror:{target}"));
            }
            if timing == Some(2) {
                choose(&mut game, "kidnap");
                choose(&mut game, "kidnap:roll");
            }
            assert_eq!(
                game.observe().resources,
                if timing.is_some() {
                    [3, 3, 3, 15]
                } else {
                    [6, 0, 3, 15]
                }
            );
            assert_eq!(game.observe().standings[1].score, 10);
            assert_eq!(game.observe().terror_available, 18);
            assert_eq!(
                game.observe()
                    .spaces
                    .iter()
                    .filter(|s| s.terror > 0)
                    .count(),
                2
            );
            choose(&mut game, "end-terror");
            assert_eq!(
                game.observe()
                    .next
                    .unwrap()
                    .options
                    .contains(&"Event".into()),
                timing.is_some()
            );
            assert_eq!(
                (
                    game.observe().current_card,
                    game.observe().preview_card,
                    game.observe().draw_count
                ),
                (17, 18, 34)
            );
        }
    }
    // The result-1 source witness: a real later Resource transfer restores Oriente.
    let mut game = Game::prepared_with_seed(19);
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    choose(&mut game, "terror:santiago");
    transfer_resources(&mut game, 3, 1);
    choose(&mut game, "terror:oriente");
    assert_eq!(game.observe().resources, [5, 0, 5, 14]);
    assert_eq!(game.observe().standings[1].score, 10);
}

#[test]
fn depleted_government_needs_real_funding_before_early_zero_proceeds_kidnap() {
    let mut game = Game::prepared_with_seed(5);
    transfer_resources(&mut game, 0, 6);
    choose(&mut game, "transfer");
    choose(&mut game, "give:2:6");
    let before = game.clone();
    let v = game.observe();
    assert!(
        game.submit(v.owner, v.revision, &format!("{}:kidnap", v.revision))
            .is_err()
    );
    assert_eq!(game, before);
    transfer_resources(&mut game, 2, 1);
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    assert_eq!(game.observe().kidnap.as_ref().unwrap().proceeds, 0);
    assert_eq!(game.observe().kidnap.as_ref().unwrap().die, 3);
    assert_eq!(game.observe().resources, [0, 1, 10, 15]);
    choose(&mut game, "terror:santiago");
    choose(&mut game, "end-terror");
    assert!(
        game.observe()
            .next
            .unwrap()
            .options
            .contains(&"Event".into())
    );
}

#[test]
fn refused_targets_activities_funds_duplicates_and_stale_input_preserve_all_state() {
    let mut game = Game::prepared();
    for action in [
        "terror:santiago",
        "terror:oriente",
        "terror:havana",
        "kidnap:oriente",
        "kidnap:syndicate",
        "infiltrate",
        "ambush",
        "ransom",
        "launder",
        "cancel",
    ] {
        let before = game.clone();
        assert!(
            game.submit(Faction::M26, 0, &format!("0:{action}"))
                .is_err()
        );
        assert_eq!(game, before);
    }
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    let before = game.clone();
    for (owner, revision, action) in [
        (Faction::Government, 1, "1:terror:santiago"),
        (Faction::M26, 0, "0:kidnap"),
        (Faction::M26, 1, "1:kidnap"),
        (Faction::M26, 1, "1:end-terror"),
    ] {
        assert!(game.submit(owner, revision, action).is_err());
        assert_eq!(game, before);
    }
    choose(&mut game, "terror:santiago");
    let before = game.clone();
    assert!(game.submit(Faction::M26, 2, "2:terror:santiago").is_err());
    assert_eq!(game, before);
    choose(&mut game, "terror:oriente");
    let before = game.clone();
    assert!(game.submit(Faction::M26, 3, "3:terror:oriente").is_err());
    assert_eq!(game, before);
}

#[test]
fn active_only_missing_guerrillas_and_failed_outnumbering_refuse_without_rng_consumption() {
    // Deliberately malformed counterfactual states test refusal. Actual supported
    // removals are covered in removal.rs.
    for variant in 0..3 {
        let mut source = serde_json::to_value(Game::prepared()).unwrap();
        source["resources"] = serde_json::json!([5, 1, 5, 15]);
        let guerrilla = &mut source["spaces"][2]["pieces"][2];
        if variant == 0 {
            guerrilla["state"] = "Active".into();
        } else {
            guerrilla["count"] = (variant - 1).into();
        }
        let mut game: Game = serde_json::from_value(source).unwrap();
        let before = game.clone();
        assert!(game.submit(Faction::M26, 0, "0:kidnap").is_err());
        assert_eq!(game, before);
        if variant != 2 {
            assert!(game.submit(Faction::M26, 0, "0:terror:santiago").is_err());
            assert_eq!(game, before);
        }
    }
}
