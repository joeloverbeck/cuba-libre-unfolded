use cuba_libre_unfolded::{Faction, Game, Kind};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap();
}

fn remove(game: &mut Game, target: &str) {
    choose(game, "remove");
    choose(game, &format!("remove:{target}"));
    choose(game, "confirm-removal");
}

// R 1.4.1 and accepted deacd635 removal witness: no obligation yet, so
// losing strict outnumbering does not forbid removal or a later actual donation.
#[test]
fn removal_requires_confirmation_and_defeats_kidnap_but_not_donated_terror() {
    let mut game = Game::prepared();
    let initial = game.observe();
    choose(&mut game, "remove");
    choose(&mut game, "remove:santiago:underground");
    assert_eq!(game.observe().spaces, initial.spaces);
    choose(&mut game, "cancel-removal");
    assert_eq!(game.observe().spaces, initial.spaces);
    remove(&mut game, "santiago:underground");
    let view = game.observe();
    assert_eq!((view.map_piece_count, view.available_piece_count), (28, 58));
    assert_eq!(view.standings[1].score, 7);
    assert!(
        !view
            .choices
            .iter()
            .find(|c| c.id.ends_with(":kidnap"))
            .unwrap()
            .supported
    );
    assert_eq!(view.spaces[2].control(), Some(Faction::Government));
    assert_eq!(
        view.available
            .iter()
            .find(|p| p.faction == Faction::M26 && p.kind == Kind::Guerrilla)
            .unwrap()
            .count,
        13
    );
    assert_eq!(game.random_state(), 17);
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:2:1");
    choose(&mut game, "amount:1");
    choose(&mut game, "authorize");
    choose(&mut game, "terror:santiago");
    assert_eq!(game.observe().resources, [6, 0, 4, 15]);
    assert_eq!(game.observe().standings[1].score, 8);
    choose(&mut game, "end-terror");
    assert_eq!(
        game.observe().next_options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    game.validate().unwrap();
}

#[test]
fn all_own_pieces_can_return_to_supply_and_zero_space_finish_remains_legal() {
    let mut game = Game::prepared();
    remove(&mut game, "sierra-maestra:base");
    assert_eq!(game.observe().standings[1].score, 6);
    assert_eq!(game.observe().spaces[9].control(), None);
    for target in [
        "santiago:underground",
        "santiago:underground",
        "oriente:underground",
    ] {
        remove(&mut game, target);
        game.validate().unwrap();
    }
    let v = game.observe();
    assert_eq!((v.map_piece_count, v.available_piece_count), (25, 61));
    assert_eq!(v.standings[1].score, 6);
    assert_eq!(v.spaces[8].control(), None);
    assert_eq!(v.resources, [6, 0, 5, 15]);
    assert_eq!(v.terror_available, 20);
    // #62 lists lost targets disabled, with no remedy, instead of hiding them.
    assert!(
        v.choices
            .iter()
            .filter(|c| c.id.contains(":terror:"))
            .all(|c| !c.supported && c.remedies.is_empty())
    );
    choose(&mut game, "remove");
    assert_eq!(game.observe().choices.len(), 1); // explicit cancel, no forced target
    choose(&mut game, "cancel-removal");
    choose(&mut game, "end-terror");
    assert_eq!(
        game.observe().next_options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    game.validate().unwrap();
}

#[test]
fn owed_terror_protects_only_last_santiago_underground_and_effects_survive_removal() {
    let mut game = Game::prepared_with_seed(5); // actual roll 3, source witness
    choose(&mut game, "kidnap");
    let rng = game.random_state();
    remove(&mut game, "santiago:underground");
    choose(&mut game, "remove");
    let v = game.observe();
    let before = game.clone();
    assert!(
        game.submit(
            v.owner,
            v.revision,
            &format!("{}:remove:santiago:underground", v.revision)
        )
        .unwrap_err()
        .contains("owed")
    );
    assert_eq!(game, before);
    choose(&mut game, "cancel-removal");
    remove(&mut game, "sierra-maestra:base");
    remove(&mut game, "oriente:underground");
    game.validate().unwrap();
    choose(&mut game, "terror:santiago");
    assert_eq!(game.observe().standings[1].score, 7);
    remove(&mut game, "santiago:active");
    let v = game.observe();
    assert_eq!((v.map_piece_count, v.available_piece_count), (25, 61));
    assert_eq!((v.spaces[2].support, v.spaces[2].terror), (-1, 1));
    assert_eq!(v.standings[1].score, 7);
    assert_eq!(v.resources, [3, 2, 5, 15]);
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:1:2");
    choose(&mut game, "amount:1");
    choose(&mut game, "amount:2");
    choose(&mut game, "authorize");
    choose(&mut game, "end-terror");
    assert_eq!(game.observe().resources, [3, 0, 7, 15]);
    assert!(game.observe().next_options.contains(&"Event".into()));
    assert_eq!(game.random_state(), rng);
    game.validate().unwrap();
}

#[test]
fn foreign_removal_wrong_owner_and_stale_duplicate_leave_state_and_rng_unchanged() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let before = game.clone();
    for (owner, revision, action) in [
        (Faction::Government, 1, "1:remove:santiago:underground"),
        (Faction::M26, 1, "1:remove:santiago:police"),
        (Faction::M26, 1, "1:remove:havana:underground"),
        (Faction::M26, 1, "1:remove:pinar-del-rio:casino"),
        (Faction::M26, 1, "1:confirm-removal"),
    ] {
        assert!(game.submit(owner, revision, action).is_err());
        assert_eq!(game, before);
    }
    choose(&mut game, "remove:santiago:underground");
    let rev = game.observe().revision;
    choose(&mut game, "confirm-removal");
    let before = game.clone();
    assert!(
        game.submit(Faction::M26, rev, &format!("{rev}:confirm-removal"))
            .is_err()
    );
    assert_eq!(game, before);
    assert_eq!(game.random_state(), 17);
}

#[test]
fn active_removal_preserves_other_group_and_both_terror_effects() {
    let mut game = Game::prepared_with_seed(5);
    choose(&mut game, "kidnap");
    choose(&mut game, "terror:santiago");
    choose(&mut game, "terror:oriente");
    remove(&mut game, "santiago:active");
    let v = game.observe();
    assert!(v.spaces[2].pieces.iter().any(|p| p.faction == Faction::M26
        && p.state == cuba_libre_unfolded::State::Underground
        && p.count == 1));
    assert_eq!(v.standings[1].score, 10);
    for target in [
        "santiago:underground",
        "oriente:active",
        "sierra-maestra:base",
    ] {
        remove(&mut game, target);
        game.validate().unwrap();
    }
    assert_eq!(game.observe().standings[1].score, 9);
    assert_eq!(game.observe().terror_available, 18);
    choose(&mut game, "end-terror");
    game.validate().unwrap();
}
