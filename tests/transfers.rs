use cuba_libre_unfolded::{Faction, Game};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap();
}

// Accepted handoff R 1.5.1/1.8: an actual giver decision, never a promised donation.
#[test]
fn government_chooses_and_authorizes_funding_before_balances_change() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:0:1");
    assert_eq!(game.observe().owner, Faction::Government);
    choose(&mut game, "amount:1");
    assert_eq!(game.observe().resources, [6, 0, 5, 15]);
    choose(&mut game, "authorize");
    assert_eq!(game.observe().resources, [5, 1, 5, 15]);
    assert_eq!(game.observe().owner, Faction::M26);
    game.validate().unwrap();
}

#[test]
fn zero_decline_and_wait_leave_resources_unchanged_and_preserve_giver_custody() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "back");
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:3:1");
    let waiting = game.clone();
    choose(&mut game, "wait");
    assert_eq!(game, waiting);
    choose(&mut game, "decline");
    assert_eq!(game.observe().owner, Faction::M26);
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:1:3");
    choose(&mut game, "authorize");
    assert_eq!(game.observe().resources, [6, 0, 5, 15]);
    choose(&mut game, "end-terror");
    game.validate().unwrap();
}

#[test]
fn repeated_all_six_directions_conserve_resources_and_update_syndicate_contributions() {
    let mut game = Game::prepared();
    for (pair, expected, margin) in [
        ("0:1", [5, 1, 5, 15], -15),
        ("1:0", [6, 0, 5, 15], -15),
        ("2:1", [6, 1, 4, 15], -15),
        ("1:2", [6, 0, 5, 15], -15),
        ("3:1", [6, 1, 5, 14], -16),
        ("1:3", [6, 0, 5, 15], -15),
        ("3:1", [6, 1, 5, 14], -16),
    ] {
        choose(&mut game, "transfer");
        choose(&mut game, &format!("transfer:{pair}"));
        choose(&mut game, "amount:1");
        choose(&mut game, "authorize");
        let view = game.observe();
        assert_eq!(view.resources, expected);
        assert_eq!(
            view.resources.iter().map(|&v| u16::from(v)).sum::<u16>(),
            26
        );
        assert_eq!(view.standings[3].margin, margin);
        assert!(
            view.standings[3]
                .contributions
                .contains(&format!("Resources: {}", expected[3]))
        );
        assert_eq!(view.spaces, Game::prepared().observe().spaces);
        assert_eq!(game.random_state(), 17);
        game.validate().unwrap();
    }
}

#[test]
fn wrong_owner_stale_nonexecuting_pairs_and_invalid_quantities_have_no_effect() {
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    let before = game.clone();
    for pair in ["0:2", "2:0", "0:3", "3:0", "2:3", "3:2", "1:1", "4:1"] {
        assert!(
            game.submit(Faction::M26, 1, &format!("1:transfer:{pair}"))
                .is_err()
        );
        assert_eq!(game, before);
    }
    choose(&mut game, "transfer:0:1");
    let before = game.clone();
    for (owner, revision, choice) in [
        (Faction::M26, 2, "2:authorize"),
        (Faction::M26, 2, "2:amount:1"),
        (Faction::Government, 1, "2:authorize"),
        (Faction::Government, 2, "1:authorize"),
        (Faction::Government, 2, "2:amount:-1"),
        (Faction::Government, 2, "2:amount:1.5"),
        (Faction::Government, 2, "2:amount:7"),
        (Faction::Government, 2, "2:amount:256"),
        (Faction::Government, 2, "2:end-terror"),
    ] {
        assert!(game.submit(owner, revision, choice).is_err());
        assert_eq!(game, before);
    }
    for amount in 1..=6 {
        choose(&mut game, &format!("amount:{amount}"));
    }
    assert!(
        !game
            .observe()
            .choices
            .iter()
            .any(|c| c.id.ends_with("amount:7"))
    );
    choose(&mut game, "amount:5");
    choose(&mut game, "authorize");
    assert_eq!(game.observe().resources, [1, 5, 5, 15]);
}

#[test]
fn recipient_cap_is_enforced_even_in_a_counterfactual_high_balance_position() {
    // The accepted source explicitly says the 49 cap cannot bind in the 26-total
    // fixture. Challenge its rule guard separately; this is not an admitted save.
    let mut game = Game::prepared();
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:0:1");
    let mut serialized = serde_json::to_value(&game).unwrap();
    serialized["resources"] = serde_json::json!([6, 49, 5, 15]);
    let mut counterfactual: Game = serde_json::from_value(serialized).unwrap();
    assert!(counterfactual.validate().is_err());
    let before = counterfactual.clone();
    assert!(
        counterfactual
            .submit(Faction::Government, 2, "2:amount:1")
            .is_err()
    );
    assert_eq!(counterfactual, before);
}
