//! #63: Kidnap in two steps. Opening shows the stakes and consumes no randomness; the
//! roll commits the die and its proceeds.
//!
//! Expected values come from the Living Rules (March 2019) and #8's accepted
//! position, not from the engine:
//! - R 4.3.3: Kidnap may take place in 1 space selected for Terror that is a City, EC
//!   or has an open Casino and has more 26July Guerrillas than Police. 26July targets
//!   Government in a City or EC (Santiago here). Roll a die and transfer that many
//!   Resources from the target; once it runs out, no more is transferred. So with
//!   Government at G the possible proceeds are `min(d, G)` for d = 1–6: 1–6 at G = 6,
//!   1–2 at G = 2, and only 0 at G = 0. The guaranteed minimum is `min(1, G)`.
//! - R 4.1: Kidnap accompanies Terror; an early roll owes Santiago Terror under the
//!   accepted commitment policy (#8), which also gates the roll on its feasibility.
//! - R 2.3.4, 5.2: after an Operation with Special Activity Directorio may execute card
//!   17's Event, either text.
//! - Going back to the action choice before the first effect is local policy (#58);
//!   the roll is an effect, opening and cancelling are not.
//!
//! The six die faces are #8's recorded shared-RNG faces for those seeds
//! (`tests/integration/terror.rs`), so a cancel that consumed randomness would change them.
use cuba_libre_unfolded::{CARD_17_SHADED, Faction, Game, KidnapStakes, save::Session};

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

fn transfer(game: &mut Game, giver: Faction, recipient: Faction, amount: u8) {
    choose(game, "transfer");
    if giver == Faction::M26 {
        choose(game, &format!("give:{}:{amount}", recipient.index()));
    } else {
        choose(game, &format!("ask:{}:{amount}", giver.index()));
        choose(game, &format!("authorize:{amount}"));
    }
}

fn ids(game: &Game) -> Vec<String> {
    let view = game.observe();
    view.choices
        .iter()
        .map(|c| c.id.split_once(':').unwrap().1.to_owned())
        .collect()
}

fn stakes(game: &Game, action: &str) -> (KidnapStakes, Vec<String>) {
    let view = game.observe();
    let id = format!("{}:{action}", view.revision);
    let choice = view.choices.iter().find(|c| c.id == id).unwrap();
    let preview = choice.preview.clone().unwrap_or_else(|| panic!("{id}"));
    assert!(
        preview.resources.is_empty(),
        "no future result: {preview:?}"
    );
    (preview.kidnap.unwrap(), preview.notes)
}

/// Everything except the revision, which every published decision advances.
fn unrevised(game: &Game) -> serde_json::Value {
    let mut value = serde_json::to_value(game).unwrap();
    value["revision"] = 0.into();
    value
}

#[test]
fn opening_shows_the_stakes_and_cancelling_changes_neither_state_nor_randomness() {
    let mut game = crate::terror_chosen(Game::opening());
    let before = game.clone();
    let (open, notes) = stakes(&game, "kidnap");
    assert_eq!(game, before, "reading the preview changed the match");
    assert_eq!(open.target, Faction::Government);
    assert_eq!(open.space, "santiago");
    assert_eq!(open.government, 6);
    assert_eq!(open.proceeds, [1, 2, 3, 4, 5, 6]);
    assert_eq!(open.minimum, 1);
    assert!(open.commits_santiago);
    let notes = notes.join(" ");
    assert!(notes.contains("accepted commitment policy"), "{notes}");
    assert!(notes.contains(CARD_17_SHADED), "{notes}");
    assert!(notes.contains("R 2.3.4, 5.2"), "{notes}");

    choose(&mut game, "kidnap");
    let opened = game.observe();
    assert!(opened.kidnap_open);
    assert_eq!(opened.kidnap, None);
    assert_eq!(game.random_state(), before.random_state());
    assert_eq!(opened.resources, [6, 0, 5, 15]);
    // While open, only the roll and the cancel are offered.
    assert_eq!(ids(&game), ["kidnap:roll", "kidnap:cancel"]);
    let (roll, _) = stakes(&game, "kidnap:roll");
    assert_eq!(roll, open);
    assert!(opened.choices.iter().all(|c| c.owner == Faction::M26));

    choose(&mut game, "kidnap:cancel");
    assert_eq!(unrevised(&game), unrevised(&before));
    assert_eq!(game.random_state(), before.random_state());
    assert!(!game.observe().kidnap_open);
}

#[test]
fn the_preview_never_shows_a_future_result() {
    // Seeds 19 and 1 roll 1 and 6 (#8): their previews must not differ.
    let low = crate::terror_chosen(Game::opening_with_seed(19));
    let high = crate::terror_chosen(Game::opening_with_seed(1));
    let preview = |game: &Game| {
        let view = game.observe();
        view.choices
            .iter()
            .find(|c| c.id.ends_with(":kidnap"))
            .unwrap()
            .preview
            .clone()
    };
    assert_eq!(preview(&low), preview(&high));
    let (mut low, mut high) = (low, high);
    choose(&mut low, "kidnap");
    choose(&mut high, "kidnap");
    let roll = |game: &Game| {
        let view = game.observe();
        view.choices
            .iter()
            .find(|c| c.id.ends_with(":kidnap:roll"))
            .unwrap()
            .preview
            .clone()
    };
    assert_eq!(roll(&low), roll(&high));
}

#[test]
fn all_six_faces_from_8_hold_after_opening_and_cancelling_first() {
    for (seed, die) in [(19, 1), (0, 2), (5, 3), (3, 4), (2, 5), (1, 6)] {
        let mut game = crate::terror_chosen(Game::opening_with_seed(seed));
        choose(&mut game, "kidnap");
        choose(&mut game, "kidnap:cancel");
        choose(&mut game, "kidnap");
        choose(&mut game, "kidnap:roll");
        let view = game.observe();
        let rolled = view.kidnap.as_ref().unwrap();
        assert_eq!((rolled.die, rolled.proceeds), (die, die), "seed {seed}");
        assert_eq!(view.resources, [6 - die, die, 5, 15]);
        assert_eq!(view.owed_terror.as_deref(), Some("santiago"));
        assert!(!view.kidnap_open);
        // The roll is committed once: no second open or roll is offered.
        assert!(
            !ids(&game)
                .iter()
                .any(|id| id == "kidnap" || id.starts_with("kidnap:"))
        );
    }
}

#[test]
fn the_range_follows_governments_current_resources_down_to_zero() {
    // Government gives 4: 2 left, so a die takes 1 or 2 (R 4.3.3).
    let mut game = crate::terror_chosen(Game::opening_with_seed(1)); // #8's face 6
    transfer(&mut game, Faction::Government, Faction::M26, 4);
    let (two, _) = stakes(&game, "kidnap");
    assert_eq!(
        (two.government, two.proceeds.clone(), two.minimum),
        (2, vec![1, 2], 1)
    );

    // Government gives away its whole balance: nothing to take, minimum 0.
    let mut game = crate::terror_chosen(Game::opening_with_seed(1));
    transfer(&mut game, Faction::Government, Faction::M26, 6);
    let (zero, _) = stakes(&game, "kidnap");
    assert_eq!(
        (zero.government, zero.proceeds, zero.minimum),
        (0, vec![0], 0)
    );
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    let view = game.observe();
    let rolled = view.kidnap.as_ref().unwrap();
    assert_eq!((rolled.die, rolled.proceeds), (6, 0));
    assert_eq!(view.resources, [0, 6, 5, 15]);
}

#[test]
fn after_santiago_terror_the_roll_commits_nothing_further() {
    let mut game = crate::terror_chosen(Game::opening());
    transfer(&mut game, Faction::Directorio, Faction::M26, 1);
    choose(&mut game, "terror:santiago");
    let (late, notes) = stakes(&game, "kidnap");
    assert!(!late.commits_santiago);
    assert!(!notes.join(" ").contains("accepted commitment policy"));
}

/// #75: the outnumbering terms give Santiago's 26July Guerrillas by state, as they
/// stand: #8's position has 2 Underground; a removal leaves 1; Santiago's Terror flips
/// 1 to Active (R 3.3.4). Police are 1 throughout.
#[test]
fn the_outnumbering_terms_count_guerrillas_by_their_state() {
    let kidnap = |game: &Game| {
        game.observe()
            .choices
            .into_iter()
            .find(|c| c.id.ends_with(":kidnap"))
            .unwrap()
            .explanation
    };
    let game = crate::terror_chosen(Game::opening());
    assert!(
        kidnap(&game).starts_with("2 Underground Guerrillas vs 1 Police; must outnumber."),
        "{}",
        kidnap(&game)
    );
    let mut removed = crate::terror_chosen(Game::opening());
    choose(&mut removed, "remove");
    choose(&mut removed, "remove:santiago:underground:1");
    choose(&mut removed, "done-removal");
    assert!(
        kidnap(&removed).starts_with("1 Underground Guerrilla vs 1 Police; must outnumber."),
        "{}",
        kidnap(&removed)
    );
    let mut resolved = crate::terror_chosen(Game::opening());
    transfer(&mut resolved, Faction::Government, Faction::M26, 1);
    choose(&mut resolved, "terror:santiago");
    assert!(
        kidnap(&resolved)
            .starts_with("2 Guerrillas (1 Active, 1 Underground) vs 1 Police; must outnumber."),
        "{}",
        kidnap(&resolved)
    );
}

/// #75: with Government and 26July both at 0, a Resource transfer to 26July is the
/// one in-task step that guarantees Santiago's payment, so it is Kidnap's remedy. With
/// a Guerrilla removed, nothing here restores outnumbering, so there is none.
#[test]
fn kidnap_s_only_remedy_is_a_transfer_that_funds_its_commitment() {
    let mut game = crate::terror_chosen(Game::opening_with_seed(5));
    transfer(&mut game, Faction::Government, Faction::M26, 6);
    transfer(&mut game, Faction::M26, Faction::Directorio, 6);
    let remedies = |game: &Game| {
        let v = game.observe();
        let k = v
            .choices
            .into_iter()
            .find(|c| c.id.ends_with(":kidnap"))
            .unwrap();
        (k.supported, k.remedies)
    };
    assert_eq!(
        remedies(&game),
        (false, vec!["Request a Resource transfer".to_owned()])
    );
    transfer(&mut game, Faction::Directorio, Faction::M26, 1);
    assert_eq!(remedies(&game), (true, vec![]));
    let mut removed = crate::terror_chosen(Game::opening());
    choose(&mut removed, "remove");
    choose(&mut removed, "remove:santiago:underground:1");
    choose(&mut removed, "done-removal");
    assert_eq!(remedies(&removed), (false, vec![]));
}

#[test]
fn the_feasibility_and_outnumbering_refusals_are_unchanged_and_open_nothing() {
    // Government and 26July both at 0: an early roll could not pay Santiago.
    let mut game = crate::terror_chosen(Game::opening_with_seed(5));
    transfer(&mut game, Faction::Government, Faction::M26, 6);
    transfer(&mut game, Faction::M26, Faction::Directorio, 6);
    let before = game.clone();
    let v = game.observe();
    let kidnap = v
        .choices
        .iter()
        .find(|c| c.id.ends_with(":kidnap"))
        .unwrap();
    assert!(!kidnap.supported);
    assert!(kidnap.preview.is_none());
    assert!(
        kidnap.explanation.contains("guaranteed payment"),
        "{}",
        kidnap.explanation
    );
    let id = format!("{}:kidnap", v.revision);
    assert_eq!(
        game.submit(Faction::M26, v.revision, &id),
        Err(kidnap.explanation.clone())
    );
    assert_eq!(game, before);

    // One Santiago Guerrilla removed: 1 does not outnumber 1 Police.
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "remove");
    choose(&mut game, "remove:santiago:underground:1");
    choose(&mut game, "done-removal");
    let before = game.clone();
    let v = game.observe();
    let kidnap = v
        .choices
        .iter()
        .find(|c| c.id.ends_with(":kidnap"))
        .unwrap();
    assert!(!kidnap.supported);
    assert!(
        kidnap.explanation.contains("must outnumber"),
        "{}",
        kidnap.explanation
    );
    let id = format!("{}:kidnap", v.revision);
    assert!(game.submit(Faction::M26, v.revision, &id).is_err());
    assert_eq!(game, before);
}

#[test]
fn while_open_other_steps_and_stale_or_foreign_submissions_are_refused() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "kidnap");
    let before = game.clone();
    let r = game.observe().revision;
    for (owner, id) in [
        (Faction::M26, format!("{r}:terror:santiago")),
        (Faction::M26, format!("{r}:transfer")),
        (Faction::M26, format!("{r}:end-terror")),
        (Faction::M26, format!("{r}:back-to-choice")),
        (Faction::M26, format!("{r}:kidnap")),
        (Faction::M26, format!("{}:kidnap:roll", r - 1)),
        (Faction::Government, format!("{r}:kidnap:roll")),
    ] {
        assert!(game.submit(owner, r, &id).is_err(), "accepted {id}");
        assert_eq!(game, before, "{id}");
    }
}

#[test]
fn opening_and_cancelling_is_no_effect_for_going_back_but_the_roll_is() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:cancel");
    assert!(ids(&game).contains(&"back-to-choice".to_owned()));
    let (_, notes) = stakes(&game, "kidnap");
    assert!(notes.join(" ").contains("first effect"), "{notes:?}");
    choose(&mut game, "back-to-choice");
    choose(&mut game, "terror");
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    assert!(!ids(&game).contains(&"back-to-choice".to_owned()));
}

#[test]
fn an_opened_kidnap_and_a_committed_roll_survive_restart_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kidnap.json");
    let mut session = Session::open(&path, None).unwrap();
    let act = |session: &mut Session, action: &str| {
        let v = session.observe();
        session
            .submit(
                Faction::M26,
                v.revision,
                &format!("{}:{action}", v.revision),
            )
            .unwrap();
    };
    act(&mut session, "terror");
    let random = session.game().random_state();
    act(&mut session, "kidnap");
    let opened = session.observe();
    assert!(opened.kidnap_open);
    drop(session);
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), opened);
    assert_eq!(session.game().random_state(), random);
    act(&mut session, "kidnap:roll");
    let rolled = session.observe();
    let consumed = session.game().random_state();
    assert_ne!(consumed, random);
    drop(session);
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), rolled);
    assert_eq!(session.game().random_state(), consumed);
    let bytes = std::fs::read(&path).unwrap();
    let r = opened.revision;
    assert!(
        session
            .submit(Faction::M26, r, &format!("{r}:kidnap:roll"))
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(session.observe(), rolled);
}

#[test]
fn admission_refuses_an_impossible_opened_kidnap() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("open.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in ["terror", "kidnap"] {
        let v = session.observe();
        session
            .submit(
                Faction::M26,
                v.revision,
                &format!("{}:{action}", v.revision),
            )
            .unwrap();
    }
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(record["payload"]["position"]["kidnap_open"], true);
    let reseal = |record: &serde_json::Value| {
        let identity = serde_json::from_value(record["identity"].clone()).unwrap();
        unfolded_record_replacement::save::seal(&identity, &record["payload"])
    };
    let mut cases = vec![];
    let mut altered = record.clone();
    altered["payload"]["position"]["action"] = "Open".into();
    cases.push(altered); // open before Terror is chosen
    let mut altered = record.clone();
    altered["payload"]["position"]["transfer"] = "Selecting".into();
    cases.push(altered); // open beside a transfer
    let mut altered = record.clone();
    altered["payload"]["position"]["removal"] = "Selecting".into();
    cases.push(altered); // open beside a removal
    let mut altered = record;
    altered["payload"]["position"]["m26_action_ended"] = true.into();
    cases.push(altered); // open after End Terror
    for altered in cases {
        let bytes = reseal(&altered);
        std::fs::write(&path, &bytes).unwrap();
        assert!(Session::open(&path, None).is_err(), "admitted {altered}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

/// #74: the die presentation chooses its tumble from the revision the roll created,
/// so the roll saves it. Opening and cancelling advance the revision like any decision;
/// the roll's own revision is the one after the submitted `kidnap:roll`.
#[test]
fn the_roll_saves_the_revision_it_created_and_a_restart_keeps_it() {
    let mut game = crate::terror_chosen(Game::opening_with_seed(5));
    choose(&mut game, "kidnap"); // revision 1 -> 2
    choose(&mut game, "kidnap:cancel"); // 2 -> 3
    choose(&mut game, "kidnap"); // 3 -> 4
    choose(&mut game, "kidnap:roll"); // 4 -> 5
    let view = game.observe();
    assert_eq!(view.revision, 5);
    assert_eq!(view.kidnap.as_ref().unwrap().revision, 5);
    choose(&mut game, "terror:santiago");
    assert_eq!(game.observe().kidnap.as_ref().unwrap().revision, 5);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    for (revision, action) in [(0, "0:terror"), (1, "1:kidnap"), (2, "2:kidnap:roll")] {
        session.submit(Faction::M26, revision, action).unwrap();
    }
    let rolled = session.observe();
    assert_eq!(rolled.kidnap.as_ref().unwrap().revision, 3);
    drop(session);
    assert_eq!(Session::open(&path, None).unwrap().observe(), rolled);
}
