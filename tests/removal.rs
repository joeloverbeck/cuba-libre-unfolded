//! Voluntary removal of 26July's own pieces (R 1.4.1). #73: removal mode stays open
//! until Done, and one decision returns any quantity of one group within its bounds.
//! The owed-Terror protection is the accepted commitment policy (local, not a
//! publisher ruling). Expected values are worked from the prepared position:
//! Santiago holds 2 Government Troops, 1 Police and 2 Underground 26July Guerrillas;
//! Oriente 1 Underground Guerrilla; Sierra Maestra the 26July Base; 26July Available
//! holds 12 Guerrillas and its standing is 7 against a threshold of 15 (margin −8).
use cuba_libre_unfolded::{Faction, Game, Kind, Removal, State};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap();
}

/// One removal decision of one piece, from the Operation menu and back to it.
fn remove(game: &mut Game, target: &str) {
    choose(game, "remove");
    choose(game, &format!("remove:{target}:1"));
    choose(game, "done-removal");
}

fn offered<'a>(
    view: &'a cuba_libre_unfolded::Observation,
    action: &str,
) -> Option<&'a cuba_libre_unfolded::Choice> {
    view.choices
        .iter()
        .find(|c| c.id == format!("{}:{action}", view.revision))
}

fn m26(game: &Game, space: &str, kind: Kind, state: State) -> u8 {
    game.observe()
        .spaces
        .iter()
        .find(|s| s.id == space)
        .unwrap()
        .pieces
        .iter()
        .filter(|p| p.faction == Faction::M26 && p.kind == kind && p.state == state)
        .map(|p| p.count)
        .sum()
}

fn available(game: &Game, kind: Kind) -> u8 {
    game.observe()
        .available
        .iter()
        .find(|p| p.faction == Faction::M26 && p.kind == kind)
        .unwrap()
        .count
}

// R 1.4.1 and accepted deacd635 removal witness: no obligation yet, so
// losing strict outnumbering does not forbid removal or a later actual donation.
#[test]
fn removal_mode_removes_nothing_until_a_removal_and_defeats_kidnap_but_not_donated_terror() {
    let mut game = Game::prepared();
    let initial = game.observe();
    choose(&mut game, "remove");
    assert_eq!(game.observe().removal, Some(Removal::Selecting));
    assert_eq!(game.observe().spaces, initial.spaces);
    // Done with nothing removed is no effect and leaves the mode.
    choose(&mut game, "done-removal");
    assert_eq!(game.observe().spaces, initial.spaces);
    assert_eq!(game.observe().removal, None);
    remove(&mut game, "santiago:underground");
    let view = game.observe();
    assert_eq!((view.map_piece_count, view.available_piece_count), (28, 58));
    assert_eq!(view.standings[1].score, 7);
    assert!(!offered(&view, "kidnap").unwrap().supported);
    assert_eq!(view.spaces[2].control(), Some(Faction::Government));
    assert_eq!(available(&game, Kind::Guerrilla), 13);
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
fn a_stack_returns_several_pieces_in_one_decision_and_the_mode_stays_open() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let view = game.observe();
    // Every quantity of every own group is listed; nothing is protected yet.
    for (action, supported) in [
        ("remove:santiago:underground:1", true),
        ("remove:santiago:underground:2", true),
        ("remove:oriente:underground:1", true),
        ("remove:sierra-maestra:base:1", true),
        ("done-removal", true),
    ] {
        assert_eq!(
            offered(&view, action).map(|c| c.supported),
            Some(supported),
            "{action}"
        );
    }
    // No quantity beyond the stack, and no other faction's pieces.
    assert!(offered(&view, "remove:santiago:underground:3").is_none());
    assert!(offered(&view, "remove:oriente:underground:2").is_none());
    assert_eq!(view.choices.len(), 5);
    let revision = view.revision;
    choose(&mut game, "remove:santiago:underground:2");
    let view = game.observe();
    // One decision, one revision; both Guerrillas return to Available.
    assert_eq!(view.revision, revision + 1);
    assert_eq!(
        m26(&game, "santiago", Kind::Guerrilla, State::Underground),
        0
    );
    assert_eq!(available(&game, Kind::Guerrilla), 14);
    assert_eq!((view.map_piece_count, view.available_piece_count), (27, 59));
    // Removal mode stays open for more pieces until Done.
    assert_eq!(view.removal, Some(Removal::Selecting));
    assert!(offered(&view, "remove:santiago:underground:1").is_none());
    assert!(
        offered(&view, "remove:oriente:underground:1")
            .unwrap()
            .supported
    );
    game.validate().unwrap();
    choose(&mut game, "remove:oriente:underground:1");
    choose(&mut game, "remove:sierra-maestra:base:1");
    // Nothing left but Done; it is never forced.
    let view = game.observe();
    assert_eq!(view.choices.len(), 1);
    assert!(offered(&view, "done-removal").is_some());
    assert_eq!(view.standings[1].score, 6);
    game.validate().unwrap();
    choose(&mut game, "done-removal");
    assert_eq!(game.observe().removal, None);
    game.validate().unwrap();
}

#[test]
fn quantities_out_of_bounds_and_stale_or_foreign_removals_change_nothing() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let before = game.clone();
    for (owner, action) in [
        (Faction::Government, "1:remove:santiago:underground:1"),
        (Faction::M26, "1:remove:santiago:underground:3"),
        (Faction::M26, "1:remove:santiago:underground:0"),
        (Faction::M26, "1:remove:santiago:police:1"),
        (Faction::M26, "1:remove:havana:underground:1"),
        (Faction::M26, "1:remove:pinar-del-rio:casino:1"),
        (Faction::M26, "1:remove:santiago:underground"),
        (Faction::M26, "1:confirm-removal"),
        (Faction::M26, "1:cancel-removal"),
    ] {
        assert!(game.submit(owner, 1, action).is_err(), "{action}");
        assert_eq!(game, before);
    }
    choose(&mut game, "remove:santiago:underground:1");
    let after = game.clone();
    assert!(
        game.submit(Faction::M26, 1, "1:remove:santiago:underground:1")
            .is_err()
    );
    assert_eq!(game, after);
    assert_eq!(game.random_state(), 17);
}

#[test]
fn owed_terror_protects_only_last_santiago_underground_and_nothing_once_it_resolves() {
    let mut game = Game::prepared_with_seed(5); // actual roll 3, source witness
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    let rng = game.random_state();
    choose(&mut game, "remove");
    let view = game.observe();
    // At most 1 of the 2: the other is protected while Santiago Terror is owed.
    assert!(
        offered(&view, "remove:santiago:underground:1")
            .unwrap()
            .supported
    );
    let two = offered(&view, "remove:santiago:underground:2").unwrap();
    assert!(!two.supported);
    assert!(two.explanation.contains("owed"), "{}", two.explanation);
    assert!(
        two.explanation.contains("accepted commitment policy"),
        "{}",
        two.explanation
    );
    let before = game.clone();
    assert!(
        game.submit(
            view.owner,
            view.revision,
            &format!("{}:remove:santiago:underground:2", view.revision)
        )
        .unwrap_err()
        .contains("owed")
    );
    assert_eq!(game, before);
    choose(&mut game, "remove:santiago:underground:1");
    // The last one is protected outright, with its reason.
    let view = game.observe();
    let last = offered(&view, "remove:santiago:underground:1").unwrap();
    assert!(!last.supported);
    assert!(last.explanation.contains("owed"), "{}", last.explanation);
    assert!(last.preview.is_none());
    let before = game.clone();
    assert!(
        game.submit(
            view.owner,
            view.revision,
            &format!("{}:remove:santiago:underground:1", view.revision)
        )
        .is_err()
    );
    assert_eq!(game, before);
    choose(&mut game, "remove:sierra-maestra:base:1");
    choose(&mut game, "remove:oriente:underground:1");
    choose(&mut game, "done-removal");
    game.validate().unwrap();
    choose(&mut game, "terror:santiago");
    assert_eq!(game.observe().standings[1].score, 7);
    // After the owed Terror resolves, nothing is protected.
    choose(&mut game, "remove");
    assert!(
        offered(&game.observe(), "remove:santiago:active:1")
            .unwrap()
            .supported
    );
    choose(&mut game, "remove:santiago:active:1");
    choose(&mut game, "done-removal");
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
fn removal_previews_control_kidnap_terror_and_the_standing() {
    let mut game = Game::prepared();
    // Syndicate gives 26July 2 Resources first, so both Terror targets are offered
    // and their loss shows; this transfer is already the first effect.
    choose(&mut game, "transfer");
    choose(&mut game, "transfer:3:1");
    choose(&mut game, "amount:1");
    choose(&mut game, "amount:2");
    choose(&mut game, "authorize");
    choose(&mut game, "remove");
    let view = game.observe();
    let preview = |action: &str| offered(&view, action).unwrap().preview.clone().unwrap();
    let unavailable = |p: &cuba_libre_unfolded::Preview| {
        let mut u: Vec<String> = p.unavailable.iter().map(|u| u.choice.clone()).collect();
        u.sort();
        u
    };
    // 1 of Santiago's 2: Government keeps Control (3 of 4 pieces); 1 Guerrilla does
    // not exceed 1 Police, so Kidnap goes (R 4.3.3); Terror stays (1 Underground left).
    let one = preview("remove:santiago:underground:1");
    assert_eq!(one.removed.len(), 1);
    assert_eq!(
        (
            one.removed[0].kind,
            one.removed[0].state,
            one.removed[0].count
        ),
        (Kind::Guerrilla, State::Underground, 1)
    );
    assert!(one.control.is_empty());
    assert!(one.standings.is_empty());
    assert_eq!(unavailable(&one), ["kidnap"]);
    // Both: no Underground Guerrilla is left, so Terror there goes too (R 3.3.4).
    let two = preview("remove:santiago:underground:2");
    assert_eq!(two.removed[0].count, 2);
    assert!(two.control.is_empty());
    assert_eq!(unavailable(&two), ["kidnap", "terror:santiago"]);
    // Oriente's only Guerrilla: Oriente's Terror goes.
    let oriente = preview("remove:oriente:underground:1");
    assert_eq!(unavailable(&oriente), ["terror:oriente"]);
    // The Base: 26July loses Sierra Maestra (1 of 1 pieces) and 1 point, 7 → 6, so
    // its margin to 15 goes −8 → −9.
    let base = preview("remove:sierra-maestra:base:1");
    assert_eq!(base.control.len(), 1);
    assert_eq!(
        (
            base.control[0].space.as_str(),
            base.control[0].from,
            base.control[0].to
        ),
        ("sierra-maestra", Some(Faction::M26), None)
    );
    assert_eq!(base.standings.len(), 1);
    let s = &base.standings[0];
    assert_eq!(
        (
            s.faction,
            s.score_from,
            s.score_to,
            s.margin_from,
            s.margin_to
        ),
        (Faction::M26, 7, 6, -8, -9)
    );
    assert!(unavailable(&base).is_empty());
    for p in [&one, &two, &oriente, &base] {
        assert!(
            p.notes.iter().any(|n| n.starts_with("R 1.4.1")),
            "{:?}",
            p.notes
        );
        assert!(p.resources.is_empty() && p.terror.is_empty() && p.support.is_empty());
    }
    // Reading previews changed nothing.
    assert_eq!(game.observe(), view);
    assert_eq!(game.random_state(), 17);
    // Before any effect, the first removal also ends going back (local policy, #58).
    let mut fresh = Game::prepared();
    choose(&mut fresh, "remove");
    let view = fresh.observe();
    let first = offered(&view, "remove:sierra-maestra:base:1")
        .unwrap()
        .preview
        .clone()
        .unwrap();
    assert_eq!(unavailable(&first), ["back-to-choice"]);
    assert!(first.notes.iter().any(|n| n.contains("first effect")));
}

#[test]
fn removal_leaves_earlier_terror_support_and_the_kidnap_result_in_place() {
    let mut game = Game::prepared_with_seed(5);
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    choose(&mut game, "terror:santiago");
    choose(&mut game, "terror:oriente");
    let before = game.observe();
    let kidnap = before.kidnap.clone();
    choose(&mut game, "remove");
    // Santiago now holds 1 Active and 1 Underground Guerrilla: separate groups.
    assert!(offered(&game.observe(), "remove:santiago:active:1").is_some());
    assert!(offered(&game.observe(), "remove:santiago:active:2").is_none());
    choose(&mut game, "remove:santiago:active:1");
    choose(&mut game, "remove:santiago:underground:1");
    choose(&mut game, "remove:oriente:active:1");
    choose(&mut game, "remove:sierra-maestra:base:1");
    let v = game.observe();
    for space in ["santiago", "oriente"] {
        let (was, is) = (
            before.spaces.iter().find(|s| s.id == space).unwrap(),
            v.spaces.iter().find(|s| s.id == space).unwrap(),
        );
        assert_eq!(
            (is.terror, is.support),
            (was.terror, was.support),
            "{space}"
        );
    }
    assert_eq!(v.kidnap, kidnap);
    assert_eq!(v.resources, before.resources);
    // Opposition still scores: 7 + 1 (Santiago) + 2 (Oriente) − 1 (the Base) = 9.
    assert_eq!(v.standings[1].score, 9);
    assert_eq!(v.terror_available, 18);
    game.validate().unwrap();
    choose(&mut game, "done-removal");
    choose(&mut game, "end-terror");
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
    assert_eq!(game.observe().choices.len(), 1); // explicit Done, no forced target
    choose(&mut game, "done-removal");
    choose(&mut game, "end-terror");
    assert_eq!(
        game.observe().next_options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    game.validate().unwrap();
}

/// A #8 save may hold a group awaiting confirmation. It resumes in removal mode with
/// the same choices, so it can be finished or left with Done.
#[test]
fn a_pending_confirmation_from_an_8_save_resumes_in_removal_mode() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let mut record = serde_json::to_value(&game).unwrap();
    record["removal"] = serde_json::json!({"Confirming":{"space":"santiago","kind":"Guerrilla","state":"Underground"}});
    let resumed: Game = serde_json::from_value(record).unwrap();
    resumed.validate().unwrap();
    let ids = |g: &Game| -> Vec<String> { g.observe().choices.into_iter().map(|c| c.id).collect() };
    assert_eq!(ids(&resumed), ids(&game));
    let mut resumed = resumed;
    choose(&mut resumed, "remove:santiago:underground:1");
    assert_eq!(resumed.observe().removal, Some(Removal::Selecting));
    resumed.validate().unwrap();
}
