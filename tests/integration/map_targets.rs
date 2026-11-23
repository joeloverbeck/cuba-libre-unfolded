//! #72: while 26July executes Terror the map shows each space as selectable, blocked
//! with a remedy, or not involved, read from the rules authority's choices.
//!
//! Expected states come from the rules and the accepted deacd635 handoff, not from
//! the engine: Terror needs 1 Resource and an Underground 26July Guerrilla in the space,
//! at most once per space (R 3.1, 3.3.4); only Santiago and Oriente hold Underground
//! 26July Guerrillas. Kidnap needs more 26July Guerrillas than Police in Santiago, which
//! starts with 2 against 1 (R 4.3.3), and Government starts with 6 Resources. 26July
//! starts with 0 Resources, so neither Terror can be paid until a transfer or Kidnap
//! funds it.
use cuba_libre_unfolded::{
    Faction, Game, Kind, State,
    targets::{PieceState, SpaceState, removal_targets, space_targets},
};

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

fn give(game: &mut Game, giver: Faction, amount: u8) {
    choose(game, "transfer");
    choose(game, &format!("ask:{}:{amount}", giver.index()));
    choose(game, &format!("authorize:{amount}"));
}

/// The state of each named space, and that every other space is not involved.
fn states(game: &Game) -> Vec<(String, SpaceState, Vec<String>)> {
    let view = game.observe();
    space_targets(&view)
        .expect("targeting while Terror executes")
        .into_iter()
        .map(|t| {
            let actions = t
                .choices
                .iter()
                .map(|id| id.split_once(':').unwrap().1.to_owned())
                .collect();
            (t.space, t.state, actions)
        })
        .collect()
}

fn state(game: &Game, space: &str) -> (SpaceState, Vec<String>) {
    states(game)
        .into_iter()
        .find(|(s, _, _)| s == space)
        .map(|(_, state, actions)| (state, actions))
        .unwrap()
}

fn only_targets_involved(game: &Game) {
    for (space, state, actions) in states(game) {
        if space != "santiago" && space != "oriente" {
            assert_eq!(state, SpaceState::NotInvolved, "{space}");
            assert!(actions.is_empty(), "{space}: {actions:?}");
        }
    }
}

#[test]
fn no_space_is_a_target_at_the_action_choice_or_after_pass() {
    let mut game = Game::opening();
    assert_eq!(space_targets(&game.observe()), None);
    choose(&mut game, "pass");
    assert_eq!(space_targets(&game.observe()), None);
}

#[test]
fn at_0_resources_santiago_offers_kidnap_and_oriente_waits_for_a_transfer() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    assert_eq!(states(&game).len(), 13);
    assert_eq!(
        state(&game, "santiago"),
        (
            SpaceState::Selectable,
            vec!["terror:santiago".into(), "kidnap".into()]
        )
    );
    assert_eq!(
        state(&game, "oriente"),
        (SpaceState::Blocked, vec!["terror:oriente".into()])
    );
    only_targets_involved(&game);
}

#[test]
fn a_resource_transfer_makes_both_targets_selectable() {
    let mut game = crate::terror_chosen(Game::opening());
    give(&mut game, Faction::Syndicate, 2);
    assert_eq!(state(&game, "santiago").0, SpaceState::Selectable);
    assert_eq!(state(&game, "oriente").0, SpaceState::Selectable);
    only_targets_involved(&game);
}

#[test]
fn a_resolved_terror_leaves_its_space_not_involved() {
    let mut game = crate::terror_chosen(Game::opening());
    give(&mut game, Faction::Syndicate, 1);
    choose(&mut game, "terror:oriente");
    // Each space at most once (R 3.1); nothing in this task undoes that.
    assert_eq!(state(&game, "oriente").0, SpaceState::NotInvolved);
    // Santiago still offers Kidnap; its Terror waits for another Resource.
    assert_eq!(state(&game, "santiago").0, SpaceState::Selectable);
}

#[test]
fn removing_a_santiago_guerrilla_before_kidnap_leaves_only_the_transfer_remedy() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "remove");
    choose(&mut game, "remove:santiago:underground:1");
    choose(&mut game, "done-removal");
    // 1 Guerrilla does not exceed 1 Police, so Kidnap is gone; a transfer still
    // funds Terror there.
    assert_eq!(
        state(&game, "santiago"),
        (
            SpaceState::Blocked,
            vec!["terror:santiago".into(), "kidnap".into()]
        )
    );
    let view = game.observe();
    let kidnap = view
        .choices
        .iter()
        .find(|c| c.id.ends_with(":kidnap"))
        .unwrap();
    assert!(!kidnap.supported && kidnap.remedies.is_empty());
}

#[test]
fn removing_oriente_s_only_guerrilla_leaves_it_not_involved() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "remove");
    choose(&mut game, "remove:oriente:underground:1");
    choose(&mut game, "done-removal");
    // No Underground Guerrilla to flip, and nothing in this task places one.
    assert_eq!(state(&game, "oriente").0, SpaceState::NotInvolved);
}

#[test]
fn targeting_pauses_while_kidnap_is_open_a_transfer_is_under_way_or_removal_is_open() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "kidnap");
    assert_eq!(space_targets(&game.observe()), None);
    choose(&mut game, "kidnap:cancel");
    assert!(space_targets(&game.observe()).is_some());
    choose(&mut game, "transfer");
    assert_eq!(space_targets(&game.observe()), None);
    choose(&mut game, "ask:3:0");
    assert_eq!(space_targets(&game.observe()), None);
    choose(&mut game, "decline");
    choose(&mut game, "remove");
    assert_eq!(space_targets(&game.observe()), None);
}

#[test]
fn after_the_kidnap_roll_santiago_offers_its_owed_terror() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    // The roll takes at least 1 of Government's 6 Resources, which pays Santiago.
    assert_eq!(
        state(&game, "santiago"),
        (SpaceState::Selectable, vec!["terror:santiago".into()])
    );
    only_targets_involved(&game);
}

#[test]
fn ending_terror_ends_targeting() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "end-terror");
    assert_eq!(space_targets(&game.observe()), None);
}

/// #73: removal mode marks every 26July group (both Santiago Guerrillas, the Oriente
/// Guerrilla and the Sierra Maestra Base, R 1.4.1) and dims every other space.
#[test]
fn removal_mode_marks_each_own_group_and_dims_the_spaces_without_one() {
    let mut game = crate::terror_chosen(Game::opening());
    assert_eq!(removal_targets(&game.observe()), None);
    choose(&mut game, "remove");
    let targets = removal_targets(&game.observe()).unwrap();
    let pieces: Vec<_> = targets
        .pieces
        .iter()
        .map(|p| {
            (
                p.space.as_str(),
                p.kind,
                p.state,
                p.count,
                p.removable,
                p.mark,
            )
        })
        .collect();
    assert_eq!(
        pieces,
        [
            (
                "santiago",
                Kind::Guerrilla,
                State::Underground,
                2,
                2,
                PieceState::Removable
            ),
            (
                "oriente",
                Kind::Guerrilla,
                State::Underground,
                1,
                1,
                PieceState::Removable
            ),
            (
                "sierra-maestra",
                Kind::Base,
                State::Ordinary,
                1,
                1,
                PieceState::Removable
            ),
        ]
    );
    assert_eq!(
        targets.pieces[0].choices,
        [
            "2:remove:santiago:underground:1",
            "2:remove:santiago:underground:2"
        ]
    );
    let view = game.observe();
    let mut dimmed: Vec<&str> = view
        .spaces
        .iter()
        .map(|s| s.id.as_str())
        .filter(|id| !matches!(*id, "santiago" | "oriente" | "sierra-maestra"))
        .collect();
    dimmed.sort_unstable();
    let mut shown = targets
        .dimmed
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    shown.sort_unstable();
    assert_eq!(shown, dimmed);
    // Space targeting pauses: region clicks do not open the space popover.
    assert_eq!(space_targets(&view), None);
}

#[test]
fn the_owed_terror_marks_santiago_s_last_underground_guerrilla_protected() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    choose(&mut game, "remove");
    let santiago = |game: &Game| {
        removal_targets(&game.observe())
            .unwrap()
            .pieces
            .into_iter()
            .find(|p| p.space == "santiago")
            .unwrap()
    };
    // Two, of which one may go.
    let stack = santiago(&game);
    assert_eq!(
        (stack.count, stack.removable, stack.mark),
        (2, 1, PieceState::Removable)
    );
    choose(&mut game, "remove:santiago:underground:1");
    let last = santiago(&game);
    assert_eq!(
        (last.count, last.removable, last.mark),
        (1, 0, PieceState::Protected)
    );
    // Oriente's Guerrilla and the Base go; Santiago still holds its protected one, so
    // only Oriente and Sierra Maestra are then dimmed besides the never-involved.
    choose(&mut game, "remove:oriente:underground:1");
    choose(&mut game, "remove:sierra-maestra:base:1");
    let targets = removal_targets(&game.observe()).unwrap();
    assert_eq!(targets.pieces.len(), 1);
    assert!(targets.dimmed.iter().any(|s| s == "oriente"));
    assert!(targets.dimmed.iter().any(|s| s == "sierra-maestra"));
    assert!(!targets.dimmed.iter().any(|s| s == "santiago"));
    choose(&mut game, "done-removal");
    assert_eq!(removal_targets(&game.observe()), None);
}
