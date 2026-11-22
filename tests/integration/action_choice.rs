//! #61: 26July's 1st Eligible decision on card 17, Pass, going back before the first
//! effect, the executing faction and the Eligibility record.
//!
//! Expected values come from the Living Rules (March 2019) and the accepted prepared
//! position (deacd635), not from the engine:
//! - R 2.3.2: card 17's order is 26July, Directorio, Government, Syndicate; the
//!   leftmost Eligible is 1st Eligible, the next 2nd Eligible.
//! - R 2.3.3: a Passing 1st Eligible faction gains +1 Resource (26July 0 → 1), stays
//!   Eligible, and the next leftmost Eligible (Directorio) replaces it as 1st Eligible.
//! - R 2.3.4: a 1st Eligible faction may Pass, execute an Operation with or without a
//!   Special Activity, or execute the Event. After an Operation, the 2nd Eligible may
//!   execute a Limited Operation, or the Event after an Operation with Special Activity.
//! - R 2.3.7: Eligibility is adjusted only after the card, so nobody is Ineligible here.
//! - R 1.5.1: Resources transfer only to or from an executing faction.
//! - 26July's Operations are Rally, March, Attack and Terror (R 3.3.1–3.3.4); Infiltrate
//!   accompanies Rally or March, Ambush Attack, Kidnap Terror (R 4.3.1–4.3.3).
//!
//! Local policy, not publisher rulings (#58): going back to the action choice before
//! the first effect; transfers open once Terror is chosen; a pending Transfer request
//! is withdrawn by going back or by ending the Operation.
use cuba_libre_unfolded::{ActionChoice, Faction, Game, SequenceBox};

const INITIAL: [u8; 4] = [6, 0, 5, 15];

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

fn offered(game: &Game, action: &str) -> bool {
    let view = game.observe();
    view.choices
        .iter()
        .any(|c| c.id == format!("{}:{action}", view.revision) && c.supported)
}

/// Every submission is refused and changes neither the state nor the randomness.
fn refused(game: &mut Game, owner: Faction, actions: &[&str]) {
    let before = game.clone();
    let revision = game.observe().revision;
    for action in actions {
        let id = format!("{revision}:{action}");
        assert!(game.submit(owner, revision, &id).is_err(), "accepted {id}");
        assert_eq!(*game, before, "{id} changed the game");
    }
}

#[test]
fn the_task_opens_on_the_first_eligible_decision_with_every_26july_option_listed() {
    let game = Game::opening();
    let view = game.observe();
    assert_eq!(view.revision, 0);
    assert_eq!(view.owner, Faction::M26);
    assert_eq!(view.action, ActionChoice::Open);
    assert_eq!(view.executing, None);
    assert!(!view.finished);
    assert_eq!(view.resources, INITIAL);
    assert_eq!(view.spaces, Game::prepared().observe().spaces);
    assert!(!view.operation.contains("Full Terror"));
    let listed: Vec<_> = view
        .choices
        .iter()
        .map(|c| (c.id.as_str(), c.supported, c.accompanies.as_deref()))
        .collect();
    assert_eq!(
        listed,
        [
            ("0:pass", true, None),
            ("0:event:unshaded", false, None),
            ("0:event:shaded", false, None),
            ("0:rally", false, None),
            ("0:rally:infiltrate", false, Some("0:rally")),
            ("0:march", false, None),
            ("0:march:infiltrate", false, Some("0:march")),
            ("0:attack", false, None),
            ("0:attack:ambush", false, Some("0:attack")),
            ("0:terror", true, None),
            ("0:terror:kidnap", false, Some("0:terror")),
        ]
    );
    assert!(view.choices.iter().all(|c| c.owner == Faction::M26));
    // Card 17's two texts, from the admitted card face.
    let label = |id: &str| {
        let c = view.choices.iter().find(|c| c.id == id).unwrap();
        format!("{} {}", c.label, c.explanation)
    };
    assert!(label("0:event:unshaded").contains("free Rallies in each space it has a Base"));
    assert!(label("0:event:shaded").contains("All 26July Guerrillas Active"));
    for id in [
        "0:event:unshaded",
        "0:event:shaded",
        "0:rally",
        "0:rally:infiltrate",
        "0:march",
        "0:march:infiltrate",
        "0:attack",
        "0:attack:ambush",
    ] {
        assert!(label(id).contains("not in this task"), "{id}");
    }
    // Kidnap is part of this task, chosen during Terror rather than refused as absent.
    assert!(!label("0:terror:kidnap").contains("not in this task"));
    // R 3.3.4, in GLOSSARY.md's term for the marker ("Terror marker", not "Terror").
    assert!(label("0:terror").contains("place a Terror marker"));
}

#[test]
fn unsupported_options_and_execution_choices_are_refused_without_effect() {
    let mut game = Game::opening();
    refused(
        &mut game,
        Faction::M26,
        &[
            "event:unshaded",
            "event:shaded",
            "rally",
            "rally:infiltrate",
            "march",
            "march:infiltrate",
            "attack",
            "attack:ambush",
            "terror:kidnap",
            // Execution choices exist only once Terror is chosen.
            "transfer",
            "give:0:0",
            "ask:3:1",
            "authorize:0",
            "remove",
            "end-terror",
            "kidnap",
            "terror:santiago",
            "back-to-choice",
        ],
    );
    refused(&mut game, Faction::Directorio, &["pass", "terror"]);
    assert!(game.submit(Faction::M26, 1, "1:pass").is_err());
    assert_eq!(game, Game::opening());
    for name in ["unshaded", "shaded"] {
        let refusal = game
            .submit(Faction::M26, 0, &format!("0:event:{name}"))
            .unwrap_err();
        assert!(refusal.contains("26July's opening decision"), "{refusal}");
        assert!(refusal.contains("not in this task"), "{refusal}");
    }
    assert_eq!(game, Game::opening());
    // The refusal names the reason rather than only rejecting the identity.
    let refusal = game.submit(Faction::M26, 0, "0:rally").unwrap_err();
    assert!(refusal.contains("not in this task"), "{refusal}");
}

#[test]
fn pass_gains_one_resource_keeps_26july_eligible_and_makes_directorio_first_eligible() {
    let mut game = Game::opening();
    let opening = game.clone();
    choose(&mut game, "pass");
    let view = game.observe();
    assert_eq!(view.resources, [6, 1, 5, 15]);
    assert_eq!(view.revision, 1);
    assert_eq!(view.action, ActionChoice::Pass);
    assert!(view.finished);
    assert_eq!(view.owner, Faction::Directorio);
    assert_eq!(view.executing, None);
    assert!(view.choices.is_empty());
    assert_eq!(
        view.next.as_ref().unwrap().options,
        [
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)",
            "Event"
        ]
    );
    // Nothing else changes: no piece, Terror, card, deck or randomness.
    assert_eq!(view.spaces, opening.observe().spaces);
    assert_eq!(view.available, opening.observe().available);
    assert_eq!((view.current_card, view.draw_count), (17, 34));
    assert_eq!(game.random_state(), opening.random_state());
    assert_eq!(view.eligible, [true; 4]);
    assert_eq!(view.passed, [false, true, false, false]);
    let e = &view.eligibility;
    assert_eq!(
        e.order,
        [
            Faction::M26,
            Faction::Directorio,
            Faction::Government,
            Faction::Syndicate
        ]
    );
    assert_eq!(e.eligible, [true; 4]);
    assert_eq!(
        (e.first, e.second),
        (Faction::Directorio, Faction::Government)
    );
    assert_eq!(e.m26_box, Some(SequenceBox::Pass));
    game.validate().unwrap();
    // The task has ended: nothing further, including transfers, is accepted.
    for owner in Faction::ALL {
        refused(
            &mut game,
            owner,
            &["pass", "terror", "transfer", "end-terror", "back-to-choice"],
        );
    }
}

#[test]
fn eligibility_shows_order_and_roles_at_the_choice_during_terror_and_after_each_ending() {
    let order = [
        Faction::M26,
        Faction::Directorio,
        Faction::Government,
        Faction::Syndicate,
    ];
    let mut game = Game::opening();
    let e = game.observe().eligibility;
    assert_eq!((e.order, e.eligible), (order, [true; 4]));
    assert_eq!(
        (e.first, e.second, e.m26_box),
        (Faction::M26, Faction::Directorio, None)
    );
    choose(&mut game, "terror");
    // The cylinder is placed when the action is completed (R 2.3, record steps).
    let e = game.observe().eligibility;
    assert_eq!(
        (e.first, e.second, e.m26_box),
        (Faction::M26, Faction::Directorio, None)
    );
    let mut only = game.clone();
    choose(&mut only, "end-terror");
    let view = only.observe();
    assert_eq!(
        (view.eligibility.first, view.eligibility.second),
        (Faction::M26, Faction::Directorio)
    );
    assert_eq!(view.eligibility.m26_box, Some(SequenceBox::OperationOnly));
    assert_eq!(
        (view.eligible, view.eligibility.eligible),
        ([true; 4], [true; 4])
    );
    assert_eq!(view.passed, [false; 4]);
    assert_eq!(
        view.next.as_ref().unwrap().options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    // Seed 5 is a diagnostic input whose actual shared roll is 3 (tests/integration/terror.rs).
    let mut special = Game::opening_with_seed(5);
    choose(&mut special, "terror");
    choose(&mut special, "kidnap");
    choose(&mut special, "kidnap:roll");
    choose(&mut special, "terror:santiago");
    choose(&mut special, "end-terror");
    let view = special.observe();
    assert_eq!(view.resources, [3, 2, 5, 15]);
    assert_eq!(
        view.eligibility.m26_box,
        Some(SequenceBox::OperationWithSpecialActivity)
    );
    assert_eq!(view.eligibility.eligible, [true; 4]);
    assert_eq!(
        view.next.as_ref().unwrap().options,
        ["Pass (+1 if executed later)", "Limited Operation", "Event"]
    );
    special.validate().unwrap();
}

#[test]
fn transfers_are_offered_only_while_26july_executes_terror() {
    let mut game = Game::opening();
    assert!(!offered(&game, "transfer"));
    choose(&mut game, "terror");
    let view = game.observe();
    assert_eq!(view.action, ActionChoice::Terror);
    assert_eq!(view.executing, Some(Faction::M26));
    assert_eq!(view.revision, 1);
    assert!(offered(&game, "transfer"));
    choose(&mut game, "transfer");
    assert_eq!(game.observe().executing, Some(Faction::M26));
    choose(&mut game, "ask:3:0");
    let view = game.observe();
    assert_eq!(view.owner, Faction::Syndicate);
    assert_eq!(view.executing, Some(Faction::M26));
    choose(&mut game, "decline");
    let mut finished = game.clone();
    choose(&mut finished, "end-terror");
    assert_eq!(finished.observe().executing, None);
    assert!(finished.observe().choices.is_empty());
    refused(&mut finished, Faction::M26, &["transfer"]);
    let mut passed = Game::opening();
    choose(&mut passed, "pass");
    assert_eq!(passed.observe().executing, None);
    refused(&mut passed, Faction::M26, &["transfer"]);
}

#[test]
fn going_back_is_allowed_before_the_first_effect_and_restores_the_choice() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    assert!(offered(&game, "back-to-choice"));
    // None of these is an effect: a menu opened and left, a declined request, an
    // authorized zero, a Wait, and a removal opened and cancelled.
    choose(&mut game, "transfer");
    choose(&mut game, "back");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:0:0");
    choose(&mut game, "decline");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:2:0");
    choose(&mut game, "authorize:0");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:0");
    choose(&mut game, "wait");
    assert!(offered(&game, "back-to-choice"), "a Wait is no effect");
    choose(&mut game, "reopen");
    choose(&mut game, "decline");
    choose(&mut game, "remove");
    choose(&mut game, "done-removal");
    assert!(offered(&game, "back-to-choice"));
    let revision = game.observe().revision;
    choose(&mut game, "back-to-choice");
    let view = game.observe();
    assert_eq!(view.revision, revision + 1);
    assert_eq!(view.action, ActionChoice::Open);
    assert_eq!(view.executing, None);
    assert_eq!(view.owner, Faction::M26);
    assert_eq!(view.resources, INITIAL);
    let mut expected = Game::opening().observe();
    expected.revision = view.revision;
    expected.choices = view.choices.clone();
    assert_eq!(view, expected);
    assert!(
        view.choices
            .iter()
            .all(|c| c.id.starts_with(&format!("{}:", view.revision)))
    );
    game.validate().unwrap();
    // The choice is open again: Pass is still possible after exploring Terror.
    choose(&mut game, "pass");
    assert_eq!(game.observe().resources, [6, 1, 5, 15]);
    game.validate().unwrap();
}

#[test]
fn going_back_withdraws_a_pending_transfer_request_without_moving_resources() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:2");
    let view = game.observe();
    assert_eq!(view.owner, Faction::Syndicate);
    let back = view
        .choices
        .iter()
        .find(|c| c.id == format!("{}:back-to-choice", view.revision))
        .unwrap();
    // The requester withdraws its own request; the giver keeps its own decision.
    assert_eq!(back.owner, Faction::M26);
    assert!(
        view.choices
            .iter()
            .filter(|c| [
                "authorize:0",
                "authorize:2",
                "authorize:15",
                "decline",
                "wait"
            ]
            .iter()
            .any(|a| c.id == format!("{}:{a}", view.revision)))
            .all(|c| c.owner == Faction::Syndicate)
    );
    refused(&mut game, Faction::Syndicate, &["back-to-choice"]);
    refused(
        &mut game,
        Faction::M26,
        &["authorize:2", "authorize:0", "decline"],
    );
    choose(&mut game, "back-to-choice");
    let view = game.observe();
    assert_eq!(view.action, ActionChoice::Open);
    assert_eq!(view.transfer, None);
    assert_eq!(view.resources, INITIAL);
    assert_eq!(view.owner, Faction::M26);
    game.validate().unwrap();
}

#[test]
fn ending_the_operation_closes_transfers_and_withdraws_a_pending_request() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    choose(&mut game, "transfer");
    choose(&mut game, "ask:0:1");
    let finish = game
        .observe()
        .choices
        .into_iter()
        .find(|c| c.id.ends_with(":end-terror"))
        .unwrap();
    assert_eq!(finish.owner, Faction::M26);
    choose(&mut game, "end-terror");
    let view = game.observe();
    assert!(view.finished);
    assert_eq!(view.transfer, None);
    assert_eq!(view.resources, INITIAL);
    assert_eq!(view.executing, None);
    assert_eq!(view.eligibility.m26_box, Some(SequenceBox::OperationOnly));
    game.validate().unwrap();
}

#[test]
fn going_back_is_refused_after_each_kind_of_effect() {
    let terror = || {
        let mut game = Game::opening_with_seed(5);
        choose(&mut game, "terror");
        game
    };
    let back_refused = |game: &mut Game| {
        assert!(!offered(game, "back-to-choice"));
        refused(game, Faction::M26, &["back-to-choice"]);
    };
    // An authorized transfer, even one later returned in full.
    let mut game = terror();
    choose(&mut game, "transfer");
    choose(&mut game, "ask:0:1");
    choose(&mut game, "authorize:1");
    back_refused(&mut game);
    choose(&mut game, "transfer");
    choose(&mut game, "give:0:1");
    assert_eq!(game.observe().resources, INITIAL);
    back_refused(&mut game);
    // A resolved Terror (its payment needs Resources, so it follows a transfer here)
    // keeps going back refused once the transfer is returned.
    choose(&mut game, "transfer");
    choose(&mut game, "ask:2:1");
    choose(&mut game, "authorize:1");
    choose(&mut game, "terror:oriente");
    back_refused(&mut game);
    // A Kidnap roll.
    let mut game = terror();
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    back_refused(&mut game);
    // A removal.
    let mut game = terror();
    choose(&mut game, "remove");
    choose(&mut game, "remove:sierra-maestra:base:1");
    choose(&mut game, "done-removal");
    back_refused(&mut game);
    // While a request is pending after an effect, the requester cannot go back either.
    choose(&mut game, "transfer");
    choose(&mut game, "ask:3:0");
    assert!(
        !game
            .observe()
            .choices
            .iter()
            .any(|c| c.id.ends_with(":back-to-choice"))
    );
}

/// A #8 save began with Terror already chosen at revision 0 and has neither new field.
#[test]
fn a_record_without_the_new_fields_resumes_with_terror_chosen() {
    let mut legacy = serde_json::to_value(Game::prepared()).unwrap();
    let fields = legacy.as_object_mut().unwrap();
    fields.remove("action");
    fields.remove("effect");
    let game: Game = serde_json::from_value(legacy.clone()).unwrap();
    game.validate().unwrap();
    let view = game.observe();
    assert_eq!(view.action, ActionChoice::Terror);
    assert_eq!(view.executing, Some(Faction::M26));
    assert_eq!(view.revision, 0);
    assert!(offered(&game, "back-to-choice"));
    assert!(offered(&game, "transfer"));
    let mut back = game.clone();
    choose(&mut back, "back-to-choice");
    assert_eq!(back.observe().action, ActionChoice::Open);
    back.validate().unwrap();
    // A #8 record after an effect resumes without going back.
    for steps in [
        &["kidnap", "kidnap:roll"][..],
        &["remove", "remove:oriente:underground:1", "done-removal"],
        &["transfer", "ask:3:1", "authorize:1"],
    ] {
        let mut played = Game::prepared_with_seed(5);
        for step in steps {
            choose(&mut played, step);
        }
        let mut record = serde_json::to_value(&played).unwrap();
        let fields = record.as_object_mut().unwrap();
        fields.remove("action");
        fields.remove("effect");
        let resumed: Game = serde_json::from_value(record).unwrap();
        resumed.validate().unwrap();
        assert_eq!(resumed.observe().action, ActionChoice::Terror);
        assert!(!offered(&resumed, "back-to-choice"), "{steps:?}");
    }
}

#[test]
fn admission_refuses_impossible_action_choice_records() {
    let opening = serde_json::to_value(Game::opening()).unwrap();
    let mut passed = Game::opening();
    choose(&mut passed, "pass");
    let passed = serde_json::to_value(passed).unwrap();
    let mut cases = vec![];
    let mut case = opening.clone();
    case["resources"] = serde_json::json!([5, 1, 5, 15]);
    cases.push(case); // a transfer before Terror was chosen
    let mut case = opening.clone();
    case["effect"] = true.into();
    cases.push(case); // an effect at the action choice
    let mut case = opening.clone();
    case["transfer"] = "Selecting".into();
    cases.push(case); // a transfer menu before execution
    let mut case = opening.clone();
    case["finished"] = true.into();
    cases.push(case); // ended without an action
    let mut case = opening;
    case["action"] = "Pass".into();
    cases.push(case); // Pass without its Resource or revision
    let mut case = passed.clone();
    case["resources"] = serde_json::json!([6, 0, 5, 15]);
    cases.push(case); // Pass without its Resource
    let mut case = passed.clone();
    case["finished"] = false.into();
    cases.push(case); // Pass still deciding
    let mut case = passed;
    case["resources"] = serde_json::json!([5, 2, 5, 15]);
    cases.push(case); // a transfer around a Pass
    for case in cases {
        let game: Game = serde_json::from_value(case.clone()).unwrap();
        assert!(game.validate().is_err(), "admitted {case}");
    }
}
