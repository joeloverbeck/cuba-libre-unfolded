//! The save session without a GPU: a save holds the match's seed, its decisions in
//! order and their position, and opens only by exact replay (ADR 0003). Each test
//! opens a save, submits decisions, drops the session, reopens the file and compares
//! what a player or the seat observes. Every match starts at the opening position.
use cuba_libre_unfolded::{ActionChoice, Faction, Game, Kind, State, Transfer, save::Session};
use std::path::Path;
use unfolded_record_replacement::{previous_path, save};

/// Reseal an edited record, so its checksum is intact and only what was edited
/// can be refused.
fn reseal(record: &serde_json::Value) -> Vec<u8> {
    let identity = serde_json::from_value(record["identity"].clone()).unwrap();
    save::seal(&identity, &record["payload"])
}

fn record(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// The decisions the save at `path` holds, in order.
fn decisions(path: &Path) -> Vec<String> {
    serde_json::from_value(record(path)["payload"]["decisions"].clone()).unwrap()
}

/// The refusal of a save another build wrote: it says so, says every file is
/// unchanged, and says a match can be started under an unused save name.
fn assert_another_build(refusal: &str) {
    for words in [
        "Another build wrote this save",
        "Every file was left unchanged",
        "Start a match under an unused save name",
    ] {
        assert!(refusal.contains(words), "{refusal}");
    }
}

#[test]
fn completed_alberto_bayo_advances_once_and_reloads_at_government_opening() {
    let dir = tempfile::tempdir().unwrap();
    for (text, resolution, shaded) in [
        ("shaded", "resolve-shaded", true),
        ("unshaded", "rally:beneficiary:directorio", false),
    ] {
        let path = dir.path().join(format!("{text}.json"));
        let mut session = Session::open(&path, None).unwrap();
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            &format!("event:{text}"),
            resolution,
        ] {
            act(&mut session, action);
        }
        let resolved = session.observe();
        let random = session.game().random_state();
        let card_end = std::fs::read(&path).unwrap();
        drop(session);
        let mut session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), resolved);
        assert_eq!(
            (
                resolved.current_card,
                resolved.preview_card,
                resolved.draw_count
            ),
            (17, 18, 34)
        );
        assert!(!resolved.played_cards.contains(&17));

        act(&mut session, "advance-card");
        let opening = session.observe();
        assert_eq!(opening.revision, resolved.revision + 1);
        assert_eq!(
            (
                opening.current_card,
                opening.preview_card,
                opening.draw_count
            ),
            (18, 19, 33)
        );
        let mut history = resolved.played_cards.clone();
        history.push(17);
        assert_eq!(opening.played_cards, history);
        let public = serde_json::to_value(&opening).unwrap();
        assert!(public.get("draw_deck").is_none());
        assert!(opening.capabilities.is_empty());
        assert!(opening.momentum.is_empty());
        assert_eq!(opening.owner, Faction::Government);
        assert_eq!(opening.eligibility.first, Faction::Government);
        assert_eq!(opening.eligibility.second, Some(Faction::Syndicate));
        assert_eq!(opening.executing, None);
        assert_eq!(opening.eligible, [true, false, false, true]);
        assert_eq!(
            opening
                .eligibility
                .event_restriction
                .map(|r| (r.faction, r.through_card)),
            shaded.then_some((Faction::M26, 18))
        );
        assert_eq!(opening.resources, resolved.resources);
        assert_eq!(opening.spaces, resolved.spaces);
        assert_eq!(opening.available, resolved.available);
        assert_eq!(opening.kidnap, resolved.kidnap);
        assert_eq!(opening.resolved, resolved.resolved);
        assert_eq!(opening.standings, resolved.standings);
        assert_eq!(session.game().random_state(), random);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), card_end);

        drop(session);
        let mut resumed = Session::open(&path, None).unwrap();
        assert_eq!(resumed.observe(), opening);
        assert_eq!(resumed.game().random_state(), random);
        let bytes = std::fs::read(&path).unwrap();
        // Art is not in the identity: the advance leaves it as it was.
        let identity = |bytes: &[u8]| {
            serde_json::from_slice::<serde_json::Value>(bytes).unwrap()["identity"].clone()
        };
        assert_eq!(identity(&bytes), identity(&card_end));
        let recovered = dir.path().join(format!("{text}-recovered.json"));
        assert_eq!(
            Session::open(&recovered, Some(&path)).unwrap().observe(),
            resolved
        );
        for (owner, revision) in [
            (resolved.owner, resolved.revision),
            (opening.owner, opening.revision),
        ] {
            assert!(
                resumed
                    .submit(owner, revision, &format!("{revision}:advance-card"))
                    .is_err()
            );
        }
        assert_eq!(resumed.observe(), opening);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), card_end);

        let pass = opening
            .choices
            .iter()
            .find(|choice| choice.action() == "government:pass")
            .unwrap();
        assert!(pass.supported);
        let preview = pass.preview.as_ref().unwrap();
        assert_eq!(preview.resources.len(), 1);
        assert_eq!(preview.resources[0].faction, Faction::Government);
        let before = opening.resources[Faction::Government.index()];
        assert_eq!(preview.resources[0].from, before);
        assert_eq!(preview.resources[0].to, before + 3);
        assert_eq!(preview.next.as_ref().unwrap().faction, Faction::Syndicate);
        assert!(
            preview
                .notes
                .iter()
                .any(|note| note.contains("Syndicate now decides as 1st Eligible"))
        );
        assert!(preview.notes.iter().all(|note| !note.contains("Available")));
        assert_eq!(resumed.observe(), opening);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);

        act(&mut resumed, "government:pass");
        let handoff = resumed.observe();
        assert_eq!(handoff.revision, opening.revision + 1);
        let mut balances = opening.resources;
        balances[Faction::Government.index()] += 3;
        assert_eq!(handoff.resources, balances);
        assert_eq!(handoff.owner, Faction::Syndicate);
        assert_eq!(handoff.eligibility.first, Faction::Syndicate);
        assert_eq!(handoff.eligibility.second, None);
        assert_eq!(
            handoff.eligibility.government_box,
            Some(cuba_libre_unfolded::SequenceBox::Pass)
        );
        assert_eq!(handoff.eligible, opening.eligible);
        assert_eq!(handoff.current_card, 18);
        assert_eq!(handoff.preview_card, 19);
        assert_eq!(handoff.draw_count, 33);
        assert_eq!(handoff.played_cards, opening.played_cards);
        assert_eq!(handoff.kidnap, opening.kidnap);
        assert_eq!(handoff.spaces, opening.spaces);
        assert_eq!(handoff.available, opening.available);
        assert_eq!(handoff.recorded_resources, preview.resources);
        assert!(handoff.passed[Faction::Government.index()]);
        assert_eq!(handoff.next, preview.next);
        assert_eq!(handoff.next.as_ref().unwrap().role, "1st Eligible");
        assert_eq!(
            handoff
                .choices
                .iter()
                .map(|choice| choice.action())
                .collect::<Vec<_>>(),
            [
                "syndicate:pass",
                "syndicate:event",
                "syndicate:rally",
                "syndicate:march",
                "syndicate:terror",
                "syndicate:construct",
                "syndicate:profit",
                "syndicate:muscle",
                "syndicate:bribe",
            ]
        );
        assert!(
            handoff
                .choices
                .iter()
                .all(|choice| choice.owner == Faction::Syndicate
                    && choice.supported
                        == matches!(choice.action(), "syndicate:pass" | "syndicate:construct"))
        );
        assert!(
            handoff.choices[4]
                .explanation
                .contains("No space has an Underground Syndicate Guerrilla")
        );
        assert_eq!(resumed.game().random_state(), random);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), bytes);
        let committed = std::fs::read(&path).unwrap();
        drop(resumed);

        let mut resumed = Session::open(&path, None).unwrap();
        assert_eq!(resumed.observe(), handoff);
        for (owner, revision, choice) in [
            (opening.owner, opening.revision, pass.id.clone()),
            (
                opening.owner,
                handoff.revision,
                format!("{}:government:pass", handoff.revision),
            ),
        ] {
            assert!(resumed.submit(owner, revision, &choice).is_err());
        }
        assert_eq!(resumed.observe(), handoff);
        assert_eq!(std::fs::read(&path).unwrap(), committed);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), bytes);

        let forged_path = dir.path().join(format!("{text}-extra-credit.json"));
        let mut record: serde_json::Value = serde_json::from_slice(&committed).unwrap();
        record["payload"]["position"]["resources"][0] = (balances[0] + 1).into();
        let forged = reseal(&record);
        std::fs::write(&forged_path, &forged).unwrap();
        let error = Session::open(&forged_path, None).err().unwrap();
        assert_another_build(&format!("{error:#}"));
        assert_eq!(std::fs::read(&forged_path).unwrap(), forged);
    }
}

#[test]
fn pact_of_caracas_lists_the_first_eligible_options_and_refuses_an_unfunded_assault() {
    // Seed 2 rolls 5 (see the shared RNG cases in kidnap.rs). R 4.3.3
    // leaves Government 1 of its original 6, as in both owner paths.
    let mut game = Game::opening_with_seed(2);
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
    ] {
        let view = game.observe();
        let choice = view
            .choices
            .iter()
            .find(|choice| choice.action() == action)
            .unwrap();
        game.submit(choice.owner, view.revision, &choice.id)
            .unwrap();
    }
    let opening = game.observe();
    let ids: Vec<_> = opening
        .choices
        .iter()
        .map(|choice| choice.action())
        .collect();
    assert_eq!(
        ids,
        [
            "government:pass",
            "government:event",
            "government:train",
            "government:garrison",
            "government:sweep",
            "government:assault",
            "government:transport",
            "government:air-strike",
            "government:reprisal",
        ]
    );
    assert!(
        opening
            .choices
            .iter()
            .all(|choice| choice.owner == Faction::Government)
    );
    assert_eq!(opening.resources[0], 1);
    let pass = &opening.choices[0];
    assert!(pass.explanation.contains("1 → 4"));
    let assault = &opening.choices[5];
    assert!(assault.supported);
    assert!(assault.explanation.contains("Firm"));
    assert!(assault.explanation.contains("2 Resources per space"));
    assert!(
        assault
            .explanation
            .contains("1 Resource and cannot pay for a space")
    );
    assert!(
        assault
            .explanation
            .contains("zero-space Assault earns no Resources")
    );
    for activity in &opening.choices[6..] {
        assert!(
            activity
                .explanation
                .contains("Operation in at least one space (R 4.1)")
        );
        assert!(
            activity
                .explanation
                .contains("Government has 1 Resource and cannot pay 2 Resources for an Operation.")
        );
        assert!(!activity.supported);
    }
    for action in ["government:assault", "end-assault"] {
        let view = game.observe();
        let choice = view.choices.iter().find(|c| c.action() == action).unwrap();
        if action == "end-assault" {
            let notes = &choice.preview.as_ref().unwrap().notes;
            assert!(
                notes
                    .iter()
                    .any(|note| note.contains("Records Operation Only"))
            );
            assert!(notes.iter().all(|note| !note.contains("Available")));
        }
        game.submit(view.owner, view.revision, &choice.id).unwrap();
    }
    let handoff = game.observe();
    assert_eq!(handoff.resources, opening.resources);
    assert_eq!(handoff.spaces, opening.spaces);
    assert_eq!(handoff.available, opening.available);
    assert_eq!(handoff.owner, Faction::Syndicate);
    let next = handoff.next.unwrap();
    assert_eq!(next.role, "2nd Eligible");
    assert_eq!(next.options, ["Pass", "Limited Operation"]);
}

/// Menu steps are decisions: after loops through Assault's transfer and removal
/// menus a restart shows the same match revision, equal to the number of saved
/// decisions, and the giver still decides. Looking at the map is not a decision
/// (#169), so the loops hold no browse step.
#[test]
fn assault_menu_loops_preserve_revision_and_the_live_giver_decision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("loops.json");
    // Seed 2 rolls 5 (see the shared RNG cases in kidnap.rs).
    let mut session = Session::create_seeded(&path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
    ] {
        act(&mut session, action);
    }
    let opening = session.observe();
    let loop_choices = [
        "transfer",
        "back",
        "transfer",
        "give:3:0",
        "remove",
        "done-removal",
        "transfer",
        "ask:1:1",
        "wait",
        "reopen",
        "decline",
        "transfer",
        "ask:2:1",
        "wait",
        "authorize:0",
    ];
    const LOOPS: u64 = 3;
    for _ in 0..LOOPS {
        for action in loop_choices {
            act(&mut session, action);
        }
    }
    let looped = session.observe();
    assert_eq!(
        looped.revision,
        opening.revision + LOOPS * loop_choices.len() as u64
    );
    assert_eq!(looped.resources, opening.resources);
    assert_eq!(looped.spaces, opening.spaces);
    assert_eq!(looped.available, opening.available);

    for action in ["transfer", "ask:1:1", "wait"] {
        act(&mut session, action);
    }
    let waiting = session.observe();
    drop(session);
    let mut reloaded = Session::open(&path, None).unwrap();
    assert_eq!(reloaded.observe(), waiting);
    assert_eq!(waiting.revision, decisions(&path).len() as u64);
    assert_eq!(waiting.owner, Faction::M26);
    assert_eq!(
        waiting.transfer,
        Some(Transfer::Authorizing {
            giver: Faction::M26,
            recipient: Faction::Government,
            amount: 1,
            waited: true,
        })
    );
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        reloaded
            .submit(
                Faction::Government,
                waiting.revision,
                &format!("{}:authorize:2", waiting.revision)
            )
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    for action in ["reopen", "authorize:2", "end-assault"] {
        act(&mut reloaded, action);
    }
    let finished = reloaded.observe();
    assert_eq!(finished.resources, [3, 2, 5, 15]);
    assert_eq!(finished.owner, Faction::Syndicate);
    drop(reloaded);
    assert_eq!(Session::open(&path, None).unwrap().observe(), finished);
    assert_eq!(finished.revision, decisions(&path).len() as u64);
}

#[test]
fn event_transfer_refuses_the_executors_authorization_and_unrecorded_effects() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("event.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "transfer",
        "ask:3:2",
    ] {
        act(&mut session, action);
    }
    let pending = session.observe();
    assert_eq!(pending.owner, Faction::Syndicate);
    assert_eq!(pending.executing, Some(Faction::Directorio));
    let saved = std::fs::read(&path).unwrap();
    let id = format!("{}:authorize:2", pending.revision);
    let refusal = session
        .submit(Faction::Directorio, pending.revision, &id)
        .unwrap_err();
    assert!(refusal.to_string().contains("decision owner"), "{refusal}");
    assert_eq!(session.observe(), pending);
    assert_eq!(std::fs::read(&path).unwrap(), saved);

    let mut forged: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    // Tally still balances, but Resources cannot already have moved while
    // the request awaits its giver.
    forged["payload"]["position"]["resources"][2] = (pending.resources[2] + 2).into();
    forged["payload"]["position"]["resources"][3] = (pending.resources[3] - 2).into();
    let forged_path = dir.path().join("unrecorded-transfer.json");
    let forged_bytes = reseal(&forged);
    std::fs::write(&forged_path, &forged_bytes).unwrap();
    assert_another_build(&format!(
        "{:#}",
        Session::open(&forged_path, None).err().unwrap()
    ));
    assert_eq!(std::fs::read(&forged_path).unwrap(), forged_bytes);

    session
        .submit(Faction::Syndicate, pending.revision, &id)
        .unwrap();
    let given = session.observe();
    assert_eq!(given.owner, Faction::Directorio);
    assert_eq!(given.resources[2], pending.resources[2] + 2);
    assert_eq!(given.resources[3], pending.resources[3] - 2);
    assert_eq!(Session::open(&path, None).unwrap().observe(), given);
}

#[test]
fn pending_removal_and_owed_piece_survive_restart_without_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("removal.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "remove",
        "remove:santiago:underground:1",
        "done-removal",
        "remove",
        "done-removal",
        "terror:santiago",
        "remove",
        "remove:santiago:active:1",
        "remove:sierra-maestra:base:1",
        "done-removal",
        "end-terror",
    ] {
        let v = session.observe();
        session
            .submit(v.owner, v.revision, &format!("{}:{action}", v.revision))
            .unwrap();
        let after = session.observe();
        let rng = session.game().random_state();
        let bytes = std::fs::read(&path).unwrap();
        drop(session);
        session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after);
        assert_eq!(session.game().random_state(), rng);
        assert!(
            session
                .submit(v.owner, v.revision, &format!("{}:{action}", v.revision))
                .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    assert_eq!(session.observe().map_piece_count, 26);
    // 7 + 1 for Santiago's Terror − 1 for the Base.
    assert_eq!(session.observe().standings[1].score, 7);
}

/// #73: removal mode and a removal of several pieces are saved before they are
/// published, resume exactly, and cannot be applied twice.
#[test]
fn removal_mode_and_a_quantity_removal_resume_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("quantity.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in ["terror", "remove", "remove:santiago:underground:2"] {
        let v = session.observe();
        session
            .submit(v.owner, v.revision, &format!("{}:{action}", v.revision))
            .unwrap();
        let after = session.observe();
        let bytes = std::fs::read(&path).unwrap();
        drop(session);
        session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after);
        assert_eq!(session.observe().removal.is_some(), action != "terror");
        assert!(
            session
                .submit(v.owner, v.revision, &format!("{}:{action}", v.revision))
                .is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    // Both Guerrillas left once: 29 − 2 on the map, 12 + 2 Available.
    let view = session.observe();
    assert_eq!(view.map_piece_count, 27);
    let guerrillas = view
        .available
        .iter()
        .find(|p| p.faction == Faction::M26 && p.kind == cuba_libre_unfolded::Kind::Guerrilla)
        .unwrap()
        .count;
    assert_eq!(guerrillas, 14);
}

#[test]
fn removal_admission_refuses_foreign_changes_wrong_supply_and_impossible_obligation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("admission.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "remove",
        "remove:santiago:underground:1",
        "done-removal",
    ] {
        let v = session.observe();
        session
            .submit(v.owner, v.revision, &format!("{}:{action}", v.revision))
            .unwrap();
    }
    let source: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let previous = std::fs::read(previous_path(&path)).unwrap();
    let mut cases = vec![];
    let mut altered = source.clone();
    altered["payload"]["position"]["spaces"][2]["pieces"][1]["count"] = 0.into();
    cases.push(altered); // Government Police are not removable
    let mut altered = source.clone();
    altered["payload"]["position"]["available"][3]["count"] = 12.into();
    cases.push(altered); // missing returned Guerrilla
    let mut altered = source.clone();
    altered["payload"]["position"]["spaces"][2]["pieces"]
        .as_array_mut()
        .unwrap()
        .pop();
    altered["payload"]["position"]["available"][3]["count"] = 14.into();
    cases.push(altered); // exact totals, but owed Terror lost its last Underground
    let mut altered = source.clone();
    altered["payload"]["position"]["removal"] = "Selecting".into();
    altered["payload"]["position"]["transfer"] = "Selecting".into();
    cases.push(altered); // mutually exclusive local decisions
    let mut altered = source;
    altered["payload"]["position"]["spaces"][8]["pieces"][0]["state"] = "Active".into();
    cases.push(altered); // no Terror to activate this piece
    for record in cases {
        let bytes = reseal(&record);
        std::fs::write(&path, &bytes).unwrap();
        let refusal = Session::open(&path, None).err().expect("admitted");
        assert_another_build(&format!("{refusal:#}"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
    }
}

#[test]
fn committed_roll_and_terror_resume_without_reroll_or_duplicate_effect() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("kidnap.json");
    let mut session = Session::open(&path, None).unwrap();
    act(&mut session, "terror");
    let initial_rng = session.game().random_state();
    session.submit(Faction::M26, 1, "1:kidnap").unwrap();
    session.submit(Faction::M26, 2, "2:kidnap:roll").unwrap();
    let rolled = session.observe();
    let consumed = session.game().random_state();
    assert_ne!(consumed, initial_rng);
    let kidnap = rolled.kidnap.as_ref().unwrap();
    assert!((1..=6).contains(&kidnap.die));
    assert_eq!(kidnap.proceeds, kidnap.die);
    assert_eq!(rolled.owed_terror.as_deref(), Some("santiago"));
    drop(session);
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), rolled);
    assert_eq!(session.game().random_state(), consumed);
    let bytes = std::fs::read(&path).unwrap();
    for (rev, action) in [
        (1, "1:kidnap"),
        (2, "2:kidnap:roll"),
        (3, "3:kidnap"),
        (3, "3:end-terror"),
    ] {
        assert!(session.submit(Faction::M26, rev, action).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    session
        .submit(Faction::M26, 3, "3:terror:santiago")
        .unwrap();
    let terror = session.observe();
    drop(session);
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), terror);
    assert_eq!(session.game().random_state(), consumed);
    assert_eq!(terror.standings[1].score, 8);
    assert_eq!(terror.owed_terror, None);
    assert!(
        session
            .submit(Faction::M26, 3, "3:terror:santiago")
            .is_err()
    );
    session.submit(Faction::M26, 4, "4:end-terror").unwrap();
    assert!(
        session
            .observe()
            .next
            .unwrap()
            .options
            .contains(&"Event".into())
    );
}

#[test]
fn a_record_cannot_claim_a_proclamation_for_a_future_card_or_twice_for_one_author() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::create_seeded(&path, 5).unwrap();
    session
        .publish(Faction::M26, 0, 17, "Our agreement is public.")
        .unwrap();
    drop(session);
    let valid = record(&path);
    let kept = std::fs::read(previous_path(&path)).unwrap();
    let document = &valid["payload"]["proclamations"][0];
    let mut future = document.clone();
    future["card"] = 18.into();
    let mut too_long = document.clone();
    too_long["text"] = "é".repeat(401).into();
    for (documents, reason) in [
        (vec![future], "card"),
        (
            vec![document.clone(), document.clone()],
            "already published",
        ),
        (vec![too_long], "400"),
    ] {
        let mut altered = valid.clone();
        altered["payload"]["proclamations"] = serde_json::json!(documents);
        let bytes = reseal(&altered);
        std::fs::write(&path, &bytes).unwrap();
        let error = Session::open(&path, None)
            .err()
            .expect("invalid publication record");
        assert!(error.to_string().contains(reason), "{error:#}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), kept);
    }
}

#[test]
fn new_sessions_seed_once_from_entropy_while_resume_preserves_the_exact_seed() {
    let dir = tempfile::tempdir().unwrap();
    let first_path = dir.path().join("first.json");
    let first = Session::open(&first_path, None).unwrap();
    let second = Session::open(&dir.path().join("second.json"), None).unwrap();
    assert_ne!(first.game().random_state(), second.game().random_state());
    assert_eq!(
        Session::open(&first_path, None)
            .unwrap()
            .game()
            .random_state(),
        first.game().random_state()
    );
    // The save holds the seed it drew, beside an empty decision list.
    let saved = record(&first_path);
    assert_eq!(saved["payload"]["seed"], first.game().random_state());
    assert_eq!(decisions(&first_path), Vec::<String>::new());
}

/// A save holds the seed the match opened with, every decision in order as its
/// choice's action without the match revision, and the position, under an identity
/// of only the format and the accepted rules source. A restart keeps the consumed
/// randomness, and the match revision equals the number of decisions.
#[test]
fn a_save_holds_its_seed_decisions_and_position_under_format_14() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("seeded.json");
    // Seed 5 rolls 3 (kidnap.rs's shared RNG cases).
    let mut session = Session::create_seeded(&path, 5).unwrap();
    assert_eq!(session.game(), &Game::opening_with_seed(5));
    for action in ["terror", "kidnap", "kidnap:roll", "terror:santiago"] {
        act(&mut session, action);
    }
    let played = session.observe();
    assert_eq!(played.kidnap.as_ref().unwrap().die, 3);
    let consumed = session.game().random_state();
    drop(session);

    let saved = record(&path);
    assert_eq!(
        saved["identity"],
        serde_json::json!({"format": "14", "source": cuba_libre_unfolded::assets::SOURCE})
    );
    assert_eq!(saved["payload"]["seed"], 5);
    assert_eq!(
        decisions(&path),
        ["terror", "kidnap", "kidnap:roll", "terror:santiago"]
    );
    assert_eq!(
        saved["payload"]["position"],
        serde_json::to_value(Session::open(&path, None).unwrap().game()).unwrap()
    );
    let resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe(), played);
    assert_eq!(resumed.game().random_state(), consumed);
    assert_eq!(played.revision, 4);

    // The seeded entry creates only a new save: an existing one is refused unchanged.
    let bytes = std::fs::read(&path).unwrap();
    let previous = std::fs::read(previous_path(&path)).unwrap();
    assert!(Session::create_seeded(&path, 2).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
}

/// Each refusal leaves the save and its previous record byte for byte as they were.
/// Another format, and decisions that do not replay to the position, say another
/// build wrote the save; checksum and size refusals keep their own words.
#[test]
fn saves_this_build_cannot_continue_are_refused_with_every_file_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::create_seeded(&path, 5).unwrap();
    for action in ["terror", "kidnap", "kidnap:roll"] {
        act(&mut session, action);
    }
    drop(session);
    let valid = record(&path);
    let previous = std::fs::read(previous_path(&path)).unwrap();

    let mut cases: Vec<(&str, Vec<u8>)> = vec![];
    // Another format: the identity format 1 wrote, with the art it bound.
    let mut format_1 = valid.clone();
    format_1["identity"] = serde_json::json!({
        "format": "1", "source": cuba_libre_unfolded::assets::SOURCE, "art": "0".repeat(64),
    });
    format_1["payload"] = valid["payload"]["position"].clone();
    cases.push(("another build", reseal(&format_1)));
    // A record under the previous format number.
    let mut other_format = valid.clone();
    other_format["identity"]["format"] = "3".into();
    cases.push(("another build", reseal(&other_format)));
    // The position does not follow from its decisions.
    let mut moved = valid.clone();
    moved["payload"]["position"]["resources"][1] = 9.into();
    cases.push(("another build", reseal(&moved)));
    let mut dropped = valid.clone();
    dropped["payload"]["decisions"]
        .as_array_mut()
        .unwrap()
        .pop();
    cases.push(("another build", reseal(&dropped)));
    let mut reseeded = valid.clone();
    reseeded["payload"]["seed"] = 19.into();
    cases.push(("another build", reseal(&reseeded)));
    // A decision the rules do not offer at that point.
    let mut unoffered = valid.clone();
    unoffered["payload"]["decisions"][1] = "kidnap:roll".into();
    cases.push(("another build", reseal(&unoffered)));
    // A failed checksum.
    let mut damaged = valid.clone();
    damaged["checksum"] = "0".repeat(64).into();
    cases.push((
        "Save checksum mismatch",
        serde_json::to_vec(&damaged).unwrap(),
    ));
    // Over the size limit of about 4 MiB.
    let mut oversized = valid.clone();
    oversized["payload"]["padding"] = "x".repeat(4 * 1024 * 1024).into();
    cases.push(("Save exceeds its size limit", reseal(&oversized)));

    for (expected, bytes) in cases {
        std::fs::write(&path, &bytes).unwrap();
        let refusal = format!("{:#}", Session::open(&path, None).err().unwrap());
        if expected == "another build" {
            assert_another_build(&refusal);
        } else {
            assert!(refusal.starts_with(expected), "{refusal}");
            assert!(!refusal.contains("Another build"), "{refusal}");
        }
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
    }
    let unoffered = {
        let mut unoffered = valid.clone();
        unoffered["payload"]["decisions"][1] = "kidnap:roll".into();
        reseal(&unoffered)
    };
    std::fs::write(&path, &unoffered).unwrap();
    let refusal = format!("{:#}", Session::open(&path, None).err().unwrap());
    assert!(
        refusal.contains("decision 2 of 3, kidnap:roll"),
        "{refusal}"
    );
    let moved = {
        let mut moved = valid.clone();
        moved["payload"]["position"]["resources"][1] = 9.into();
        reseal(&moved)
    };
    std::fs::write(&path, &moved).unwrap();
    let refusal = format!("{:#}", Session::open(&path, None).err().unwrap());
    assert!(
        refusal.contains("(its 3 decisions do not reproduce its position)"),
        "{refusal}"
    );

    // The previous record still recovers into an unused path.
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert_eq!(recovered.observe().revision, 2);
    assert!(recovered.observe().kidnap_open);
}

/// A save decodes only in its current shape (#152): a save, or its position, lacking
/// any field, an absent `Option` included, or carrying a field this build does not
/// know, such as a removed one, is refused as a save another build wrote, and the
/// save and its previous record stay byte for byte as they were.
#[test]
fn a_save_lacking_a_field_or_with_an_unknown_one_is_refused_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shape.json");
    // Seed 2 rolls 5, and 26July funds the Assault. The position ends inside the paid
    // Assault with a request set aside, so it holds absent and present optional fields.
    let mut session = Session::create_seeded(&path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
        "assault:santiago:26july",
        "transfer",
        "ask:1:1",
        "wait",
    ] {
        act(&mut session, action);
    }
    drop(session);
    let valid = record(&path);
    let position = &valid["payload"]["position"];
    assert!(position["removal"].is_null() && position["government_pass_from"].is_null());
    assert_eq!(position["transfer"]["Authorizing"]["waited"], true);
    assert!(position["kidnap"].is_object() && position["assault"]["paid"][0].is_object());
    let previous = std::fs::read(previous_path(&path)).unwrap();

    let mut cases = vec![];
    for field in position.as_object().unwrap().keys() {
        let mut altered = valid.clone();
        altered["payload"]["position"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        cases.push((format!("missing field `{field}`"), altered));
    }
    for (pointer, field) in [
        ("/payload/position/kidnap", "revision"),
        ("/payload/position/transfer/Authorizing", "waited"),
        ("/payload/position/assault", "paid"),
        ("/payload/position/assault", "finished"),
        ("/payload/position/assault/paid/0", "detail"),
    ] {
        let mut altered = valid.clone();
        let fields = altered
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap();
        assert!(fields.remove(field).is_some(), "{pointer}/{field}");
        cases.push((format!("missing field `{field}`"), altered));
    }
    for (pointer, field) in [
        ("/payload/position", "finished"),
        ("/payload/position", "event_side_decisions"),
        ("/payload/position/assault", "omitted"),
        // The browse position an earlier build's open Assault stored (#169).
        ("/payload/position/assault", "browse"),
        ("/payload/position/kidnap", "unknown"),
    ] {
        let mut altered = valid.clone();
        let fields = altered
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap();
        fields.insert(field.into(), 0.into());
        cases.push((format!("unknown field `{field}`"), altered));
    }
    // Outside the position: the save itself, not its position, is not in this
    // build's shape.
    let mut altered = valid.clone();
    altered["payload"]
        .as_object_mut()
        .unwrap()
        .insert("start".into(), position.clone());
    cases.push(("unknown field `start`".to_owned(), altered));
    let mut altered = valid.clone();
    altered["payload"]
        .as_object_mut()
        .unwrap()
        .remove("decisions");
    cases.push(("missing field `decisions`".to_owned(), altered));
    assert_eq!(cases.len(), position.as_object().unwrap().len() + 12);
    for (expected, altered) in cases {
        let bytes = reseal(&altered);
        std::fs::write(&path, &bytes).unwrap();
        let refusal = format!("{:#}", Session::open(&path, None).err().expect(&expected));
        assert_another_build(&refusal);
        assert!(
            refusal.contains("(the save is not in this build's shape: "),
            "{refusal}"
        );
        assert!(refusal.contains(&expected), "{expected}: {refusal}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
    }
    // The save as written still opens.
    std::fs::write(&path, reseal(&valid)).unwrap();
    assert_eq!(
        serde_json::to_value(Session::open(&path, None).unwrap().game()).unwrap(),
        *position
    );
}

/// Submits each action and restarts after it: the reopened save shows exactly what
/// was published, with the same randomness; the match revision equals the number of
/// saved decisions; and resubmitting the decision is refused with the bytes unchanged.
fn restart_after_each(path: &Path, mut session: Session, actions: &[&str]) -> Session {
    for &action in actions {
        let Submitted {
            owner,
            revision,
            id,
        } = act(&mut session, action);
        let after = session.observe();
        let random = session.game().random_state();
        let bytes = std::fs::read(path).unwrap();
        drop(session);
        session = Session::open(path, None).unwrap();
        assert_eq!(session.observe(), after, "{action}");
        assert_eq!(session.game().random_state(), random, "{action}");
        assert_eq!(after.revision, decisions(path).len() as u64, "{action}");
        assert!(session.submit(owner, revision, &id).is_err(), "{action}");
        assert_eq!(session.observe(), after, "{action}");
        assert_eq!(std::fs::read(path).unwrap(), bytes, "{action}");
    }
    session
}

/// A restart at each pending decision of the shaded Event and Government's Assault:
/// an open Kidnap before its roll, the rolled Kidnap with Santiago's Terror owed, the
/// open Transfer modal, Directorio's handoff, the shaded Event before it resolves, a
/// request awaiting its giver and set aside with Wait, the card advance, the
/// open Assault, its payment and side decisions, and Syndicate's decision after
/// the finished Assault.
#[test]
fn every_decision_of_the_shaded_event_and_the_assault_resumes_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shaded.json");
    // Seed 2 rolls 5: Government keeps 1 Resource (R 4.3.3).
    let session = Session::create_seeded(&path, 2).unwrap();
    let session = restart_after_each(
        &path,
        session,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "transfer",
            "give:0:1",
            "end-terror",
            "event:shaded",
            "transfer",
            "ask:3:2",
            "wait",
            "reopen",
            "authorize:2",
            "resolve-shaded",
            "advance-card",
            "government:assault",
            "assault:santiago:26july",
            "transfer",
            "ask:1:1",
            "wait",
            "reopen",
            "authorize:1",
            "remove",
            "remove:santiago:troops:1",
            "done-removal",
            "end-assault",
        ],
    );
    let finished = session.observe();
    assert_eq!(finished.owner, Faction::Syndicate);
    assert_eq!(finished.next.as_ref().unwrap().role, "2nd Eligible");
}

/// #169: a save this build writes in the middle of an Assault is admitted by exact
/// replay and resumes with its paid spaces, in the order paid: the record of what each
/// paid and what was removed or closed, the map states and the choices that remain.
/// Seed 2 rolls 5 and 26July gives 4, so Government opens the Assault with 5
/// Resources: Santiago (2 Active Guerrillas return, R 3.2.4) and then Havana (the
/// Casino closes, R 1.4.5) leave 1.
#[test]
fn a_save_written_in_the_middle_of_an_assault_resumes_with_its_paid_spaces() {
    use cuba_libre_unfolded::targets::{SpaceState, space_targets};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("assault.json");
    let mut session = Session::create_seeded(&path, 2).unwrap();
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
        "government:assault",
        "assault:santiago:26july",
        "assault:havana:syndicate",
    ] {
        act(&mut session, action);
    }
    let middle = session.observe();
    let random = session.game().random_state();
    drop(session);
    // Only decisions are saved: no step for looking at a space.
    let saved = decisions(&path);
    assert_eq!(saved.len() as u64, middle.revision);
    assert_eq!(
        saved[saved.len() - 3..],
        [
            "government:assault",
            "assault:santiago:26july",
            "assault:havana:syndicate"
        ]
    );
    let stored = &record(&path)["payload"]["position"]["assault"];
    assert_eq!(
        stored.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["finished", "paid"]
    );

    let mut session = Session::open(&path, None).unwrap();
    let resumed = session.observe();
    assert_eq!(resumed, middle);
    assert_eq!(session.game().random_state(), random);
    assert_eq!(resumed.resources, [1, 0, 5, 15]);
    assert_eq!(
        resumed
            .assault_paid
            .iter()
            .map(|paid| paid.summary())
            .collect::<Vec<_>>(),
        [
            "Santiago de Cuba · paid 2 Resources · returned 2 26July Guerrillas",
            "Havana · paid 2 Resources · closed 1 Casino",
        ]
    );
    let targets = space_targets(&resumed).unwrap();
    for target in &targets {
        let paid = matches!(target.space.as_str(), "santiago" | "havana");
        assert_eq!(
            target.state,
            if paid {
                SpaceState::NotInvolved
            } else {
                SpaceState::NoEffect
            },
            "{}",
            target.space
        );
        assert_eq!(target.choices.is_empty(), paid);
    }
    // A paid space cannot be paid again after the restart, and the save is unchanged.
    let bytes = std::fs::read(&path).unwrap();
    for repeat in ["assault:santiago:26july", "assault:santiago:none"] {
        assert!(
            session
                .submit(
                    Faction::Government,
                    resumed.revision,
                    &format!("{}:{repeat}", resumed.revision)
                )
                .is_err()
        );
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    act(&mut session, "end-assault");
    assert_eq!(session.observe().owner, Faction::Syndicate);
    assert_eq!(session.observe().assault_paid, middle.assault_paid);
}

/// #169: the match's record of an open Assault lost the browse position, so a save an
/// earlier build wrote in an Assault is a disposable save (foundation 2, ADR 0003). It
/// is refused as another build's, with the save and its previous record byte for byte
/// as they were, whether it carries the earlier format or, to show that nothing reads
/// the earlier shape, this build's.
#[test]
fn a_save_an_earlier_build_wrote_in_an_assault_is_refused_with_every_file_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("earlier.json");
    let mut session = Session::create_seeded(&path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
    ] {
        act(&mut session, action);
    }
    drop(session);
    let valid = record(&path);
    let previous = std::fs::read(previous_path(&path)).unwrap();
    assert_eq!(
        valid["payload"]["position"]["assault"],
        serde_json::json!({ "paid": [], "finished": false })
    );

    // The earlier build's open Assault: the paid spaces' ids, the latest result and
    // the browse position, here at the list of Cities after two menu decisions.
    let mut earlier = valid.clone();
    earlier["payload"]["position"]["assault"] = serde_json::json!({
        "paid_spaces": [],
        "resolution": null,
        "finished": false,
        "browse": { "Spaces": "Cities" },
    });
    let decisions = earlier["payload"]["decisions"].as_array_mut().unwrap();
    decisions.push("assault:areas:cities".into());
    earlier["payload"]["position"]["revision"] = (decisions.len() as u64).into();
    let mut earlier_format = earlier.clone();
    earlier_format["identity"]["format"] = "2".into();

    for (case, altered) in [("earlier format", earlier_format), ("this format", earlier)] {
        let bytes = reseal(&altered);
        std::fs::write(&path, &bytes).unwrap();
        let refusal = format!("{:#}", Session::open(&path, None).err().expect(case));
        assert_another_build(&refusal);
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "{case}");
        assert_eq!(
            std::fs::read(previous_path(&path)).unwrap(),
            previous,
            "{case}"
        );
    }
    // The same decisions without the browse step, in this build's shape, still open.
    std::fs::write(&path, reseal(&valid)).unwrap();
    assert!(Session::open(&path, None).is_ok());
}

/// R 2.3.7 and 2.3.8: all four finish families carry their distinct positions
/// into the Manifesto and reach Directorio's second-Eligible decision.
#[test]
fn card_18_finishes_reach_the_manifesto_endpoint_with_their_actual_consequences() {
    let dir = tempfile::tempdir().unwrap();
    for (name, actions, resources, eligible, assault, construct) in [
        (
            "both-pass",
            &["government:pass", "syndicate:pass"][..],
            [4, 4, 5, 16],
            [true, true, true, true],
            0,
            0,
        ),
        (
            "pass-and-multi-space-construct",
            &[
                "government:pass",
                "syndicate:construct",
                "construct:camaguey-city:place",
                "construct:havana:place",
                "end-construct",
            ],
            [4, 4, 5, 5],
            [true, true, true, false],
            0,
            2,
        ),
        (
            "empty-assault-and-pass",
            &["government:assault", "end-assault", "syndicate:pass"],
            [1, 4, 5, 16],
            [false, true, true, true],
            0,
            0,
        ),
        (
            "funded-assault-and-limited-construct",
            &[
                "government:assault",
                "transfer",
                "ask:3:12",
                "authorize:12",
                "assault:havana:syndicate",
                "end-assault",
                "syndicate:limited-operation",
                "transfer",
                "ask:1:2",
                "authorize:1",
                "transfer",
                "ask:2:1",
                "authorize:1",
                "construct:havana:open",
                "end-construct",
            ],
            [11, 3, 4, 0],
            [false, true, true, false],
            1,
            1,
        ),
    ] {
        let path = dir.path().join(format!("{name}.json"));
        let mut session = Session::create_seeded(&path, 2).unwrap();
        session
            .publish_as_human(Faction::M26, 0, 17, "At Santiago.")
            .unwrap();
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ] {
            act(&mut session, action);
        }
        if name == "both-pass" {
            // The free Rally adds four Underground Guerrillas in Sierra Maestra
            // without changing Resources or the two factions eligible on card 18.
            for action in [
                "event:unshaded",
                "rally:beneficiary:26july",
                "rally:sierra-maestra:place:4",
            ] {
                act(&mut session, action);
            }
            let rallied = session.observe();
            let sierra = rallied
                .spaces
                .iter()
                .find(|space| space.id == "sierra-maestra")
                .unwrap();
            assert_eq!(
                sierra
                    .pieces
                    .iter()
                    .filter(|p| p.faction == Faction::M26
                        && p.kind == Kind::Guerrilla
                        && p.state == State::Underground)
                    .map(|p| p.count)
                    .sum::<u8>(),
                4
            );
        } else {
            act(&mut session, "event:shaded");
            act(&mut session, "resolve-shaded");
        }
        act(&mut session, "advance-card");
        let card_18 = session.observe();
        session
            .publish_as_human(
                Faction::M26,
                card_18.revision,
                18,
                "We will wait for the next card.",
            )
            .unwrap();
        for action in actions {
            act(&mut session, action);
        }
        let before = session.observe();
        assert_eq!(before.resources, resources, "{name}");
        assert_eq!(before.eligible, eligible, "{name}");
        assert_eq!(before.assault_paid.len(), assault, "{name}");
        assert_eq!(before.construct_paid.len(), construct, "{name}");
        assert_eq!((before.piles.played, before.piles.deck), (19, 33));
        let havana = before
            .spaces
            .iter()
            .find(|space| space.id == "havana")
            .unwrap();
        let casinos = |state| {
            havana
                .pieces
                .iter()
                .filter(|p| p.kind == Kind::Casino && p.state == state)
                .map(|p| p.count)
                .sum::<u8>()
        };
        assert_eq!(
            (casinos(State::Open), casinos(State::Closed)),
            (1, u8::from(construct == 2))
        );
        assert_eq!(
            before
                .available
                .iter()
                .find(|p| p.kind == Kind::Casino)
                .unwrap()
                .count,
            if construct == 2 { 5 } else { 7 }
        );

        let reports = session.reports();
        assert_eq!(reports.last().unwrap().card, 18);
        assert!(reports.last().unwrap().complete, "{name}");
        let published = session.proclamations().to_vec();
        assert_eq!(
            published.iter().map(|p| p.card).collect::<Vec<_>>(),
            [17, 18]
        );
        let pre_advance = std::fs::read(&path).unwrap();
        let random = session.game().random_state();
        let advance = act(&mut session, "advance-card");
        let after = session.observe();
        assert_eq!((after.current_card, after.preview_card), (19, 20));
        assert_eq!((after.piles.played, after.piles.deck), (20, 32));
        assert_eq!(
            (after.cards.current.title, after.cards.preview.title),
            ("Sierra Maestra Manifesto", "The Twelve")
        );
        assert_eq!(
            after.eligibility.order,
            [
                Faction::M26,
                Faction::Directorio,
                Faction::Syndicate,
                Faction::Government
            ]
        );
        assert_eq!(after.eligibility.second, Some(Faction::Directorio));
        assert_eq!(after.owner, Faction::M26);
        assert_eq!(after.eligible, before.eligible);
        assert_eq!(
            (
                after.resources,
                &after.spaces,
                &after.available,
                &after.standings
            ),
            (
                before.resources,
                &before.spaces,
                &before.available,
                &before.standings
            )
        );
        assert_eq!(
            session.reports(),
            reports,
            "advancing is not a faction action"
        );
        assert_eq!(session.proclamations(), published);
        assert_eq!(session.game().random_state(), random);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), pre_advance);
        let saved = std::fs::read(&path).unwrap();

        drop(session);
        let mut session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after);
        assert_eq!(session.proclamations(), published);
        assert!(session.proclamation_is_read(0) && session.proclamation_is_read(1));
        assert_eq!(session.reports(), reports);
        assert_eq!(session.game().random_state(), random);
        assert!(
            session
                .submit(advance.owner, advance.revision, &advance.id)
                .is_err()
        );
        assert!(
            session
                .submit(
                    after.owner,
                    after.revision,
                    &format!("{}:advance-card", after.revision)
                )
                .is_err()
        );
        let stale = session
            .publish_as_human(Faction::M26, after.revision, 18, "Late.")
            .unwrap_err();
        assert!(stale.to_string().contains("Stale proclamation view"));
        assert_eq!(std::fs::read(&path).unwrap(), saved);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), pre_advance);
        session
            .publish_as_human(Faction::M26, after.revision, 19, "On the new card.")
            .unwrap();
        let with_publication = std::fs::read(&path).unwrap();
        drop(session);
        let mut session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after);
        let documents = session.proclamations();
        assert_eq!(&documents[..2], published);
        assert_eq!((documents[2].card, documents[2].author), (19, Faction::M26));
        assert_eq!(documents[2].text, "On the new card.");
        assert!(session.proclamation_is_read(2));
        let spent = session
            .publish_as_human(Faction::M26, after.revision, 19, "Again.")
            .unwrap_err();
        assert!(spent.to_string().contains("already published"));
        assert_eq!(std::fs::read(&path).unwrap(), with_publication);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), saved);
        assert_manifesto_endpoint(session, &path, name);
    }
}

fn assert_manifesto_endpoint(mut session: Session, path: &Path, route: &str) {
    use Faction::{Directorio, Government, M26, Syndicate};

    const DECLINE: &str = "manifesto:decline";
    // Living Rules 1.4, 1.7, 2.3 and 7.2, the printed Event and the prepared
    // board determine these outcomes. Construct closes new Casinos (3.3.5);
    // only an open Casino contributes to Control or Syndicate's score.
    let (actions, lines, standings) = match route {
        "both-pass" => (
            [
                "manifesto:place:sierra-maestra:guerrilla-base",
                DECLINE,
                DECLINE,
                DECLINE,
            ],
            [
                "26July placed 1 Underground Guerrilla + 1 Base in Sierra Maestra.",
                "Directorio declined placement.",
                "Syndicate declined placement.",
                "Government declined placement.",
            ],
            [(16, -2), (9, -6), (1, -8), (3, -14)],
        ),
        "pass-and-multi-space-construct" => (
            [
                DECLINE,
                DECLINE,
                "manifesto:place:camaguey-city:guerrilla-guerrilla",
                DECLINE,
            ],
            [
                "26July declined placement.",
                "Directorio declined placement.",
                "Syndicate placed 2 Underground Guerrillas in Camagüey City.",
                "Government declined placement.",
            ],
            [(16, -2), (8, -7), (1, -8), (3, -25)],
        ),
        "empty-assault-and-pass" => (
            [
                "manifesto:place:santiago:guerrilla-base",
                DECLINE,
                DECLINE,
                "manifesto:place:camaguey-city:troops-base",
            ],
            [
                "26July placed 1 Underground Guerrilla + 1 Base in Santiago de Cuba.",
                "Directorio declined placement.",
                "Syndicate declined placement.",
                "Government placed 1 Troop + 1 Base in Camagüey City.",
            ],
            [(16, -2), (9, -6), (1, -8), (3, -14)],
        ),
        "funded-assault-and-limited-construct" => (
            [
                DECLINE,
                "manifesto:place:havana:base-base",
                "manifesto:place:havana:guerrilla-guerrilla",
                "manifesto:place:havana:troops-police",
            ],
            [
                "26July declined placement.",
                "Directorio placed 2 Bases in Havana.",
                "Syndicate placed 2 Underground Guerrillas in Havana.",
                "Government placed 1 Troop + 1 Police cube in Havana.",
            ],
            [(16, -2), (8, -7), (3, -6), (3, -30)],
        ),
        _ => panic!("Unknown card 18 finish: {route}"),
    };
    let opening = session.observe();
    let prior_reports = session.reports();
    let random = session.game().random_state();
    act(&mut session, "m26:event");
    for (faction, action) in [M26, Directorio, Syndicate, Government]
        .into_iter()
        .zip(actions)
    {
        let pending = session.observe();
        assert_eq!(pending.owner, faction, "{route}");
        assert_eq!(pending.executing, Some(M26), "{route}");
        assert_eq!(pending.eligible, opening.eligible, "{route}");
        assert_eq!(act(&mut session, action).owner, faction, "{route}");
    }
    let end = session.observe();
    assert_eq!(end.revision, opening.revision + 5, "{route}");
    assert_eq!((end.current_card, end.preview_card), (19, 20), "{route}");
    assert_eq!((end.piles.played, end.piles.deck), (20, 32), "{route}");
    assert_eq!(end.eligible, opening.eligible, "{route}");
    assert_eq!(end.resources, opening.resources, "{route}");
    assert_eq!(end.eligibility.m26_next_card, Some(true), "{route}");
    assert_eq!(end.event_executor, Some(M26), "{route}");
    assert_eq!(end.executing, None, "{route}");
    assert_eq!(end.owner, Directorio, "{route}");
    assert!(end.manifesto.as_ref().unwrap().complete(), "{route}");
    let next = end.next.as_ref().unwrap();
    assert_eq!(
        (next.faction, next.role.as_str()),
        (Directorio, "2nd Eligible"),
        "{route}"
    );
    assert_eq!(
        end.choices
            .iter()
            .map(|c| (c.action(), c.owner, c.supported))
            .collect::<Vec<_>>(),
        [
            ("directorio:pass", Directorio, false),
            ("directorio:rally", Directorio, true),
            ("directorio:operation", Directorio, false)
        ],
        "{route}"
    );
    assert_eq!(
        end.standings
            .iter()
            .map(|s| (s.score, s.margin))
            .collect::<Vec<_>>(),
        standings,
        "{route}"
    );

    let check_space = |id, control, groups: &[(Faction, Kind, State, u8, u8)]| {
        let space = end.spaces.iter().find(|space| space.id == id).unwrap();
        assert_eq!(space.control(), Some(control), "{route}: {id}");
        for &(faction, kind, state, count, available) in groups {
            let on_map = space
                .pieces
                .iter()
                .find(|p| (p.faction, p.kind, p.state) == (faction, kind, state))
                .unwrap();
            let available_group = end
                .available
                .iter()
                .find(|p| (p.faction, p.kind) == (faction, kind))
                .unwrap();
            assert_eq!(
                (on_map.count, available_group.count),
                (count, available),
                "{route}: {id} {kind:?}"
            );
        }
    };
    match route {
        "both-pass" => check_space(
            "sierra-maestra",
            M26,
            &[
                (M26, Kind::Guerrilla, State::Underground, 5, 7),
                (M26, Kind::Base, State::Ordinary, 2, 2),
            ],
        ),
        "pass-and-multi-space-construct" => check_space(
            "camaguey-city",
            Government,
            &[
                (Syndicate, Kind::Casino, State::Closed, 1, 5),
                (Syndicate, Kind::Guerrilla, State::Underground, 2, 4),
            ],
        ),
        "empty-assault-and-pass" => {
            check_space(
                "santiago",
                M26,
                &[
                    (M26, Kind::Guerrilla, State::Active, 2, 11),
                    (M26, Kind::Guerrilla, State::Underground, 1, 11),
                    (M26, Kind::Base, State::Ordinary, 1, 2),
                ],
            );
            check_space(
                "camaguey-city",
                Government,
                &[
                    (Government, Kind::Troops, State::Ordinary, 2, 2),
                    (Government, Kind::Base, State::Ordinary, 1, 1),
                ],
            );
        }
        "funded-assault-and-limited-construct" => check_space(
            "havana",
            Government,
            &[
                (Directorio, Kind::Base, State::Ordinary, 2, 2),
                (Syndicate, Kind::Casino, State::Open, 1, 7),
                (Syndicate, Kind::Guerrilla, State::Underground, 2, 4),
                (Government, Kind::Troops, State::Ordinary, 7, 2),
                (Government, Kind::Police, State::Ordinary, 5, 7),
            ],
        ),
        _ => unreachable!(),
    }

    let reports = session.reports();
    assert_eq!(reports.len(), prior_reports.len() + 1, "{route}");
    assert_eq!(&reports[..prior_reports.len()], prior_reports, "{route}");
    let report = reports.last().unwrap();
    assert_eq!(
        (report.card, report.faction, report.complete),
        (19, M26, true),
        "{route}"
    );
    assert_eq!(
        report
            .articles
            .iter()
            .flat_map(|article| &article.lines)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        lines,
        "{route}"
    );
    assert_eq!(session.game().random_state(), random, "{route}");
    drop(session);
    let reopened = Session::open(path, None).unwrap();
    assert_eq!(reopened.observe(), end, "{route}");
    assert_eq!(reopened.reports(), reports, "{route}");
}

/// A restart at each pending decision of the unshaded Event: the Rally's beneficiary
/// choice, the free Rally, the card advance, Government's Pass and Syndicate's
/// decision after it.
#[test]
fn every_decision_of_the_free_rally_and_government_s_pass_resumes_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unshaded.json");
    let session = Session::create_seeded(&path, 2).unwrap();
    let session = restart_after_each(
        &path,
        session,
        &[
            "terror",
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
            "government:pass",
        ],
    );
    let handoff = session.observe();
    assert_eq!(handoff.owner, Faction::Syndicate);
    assert!(handoff.passed[Faction::Government.index()]);
}

#[test]
fn admission_refuses_forged_roll_effects_and_impossible_revision_without_replacing_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("effects.json");
    let mut session = Session::open(&path, None).unwrap();
    act(&mut session, "terror");
    session.submit(Faction::M26, 1, "1:kidnap").unwrap();
    session.submit(Faction::M26, 2, "2:kidnap:roll").unwrap();
    let rolled: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    session
        .submit(Faction::M26, 3, "3:terror:santiago")
        .unwrap();
    let resolved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let previous = std::fs::read(previous_path(&path)).unwrap();
    let mut cases = vec![];
    for (field, value) in [
        ("die", 7u64),
        ("proceeds", 7),
        ("government_before", 49),
        ("random_before", u64::MAX),
    ] {
        let mut record = rolled.clone();
        record["payload"]["position"]["kidnap"][field] = value.into();
        cases.push(record);
    }
    let mut record = rolled.clone();
    record["payload"]["position"]["m26_action_ended"] = true.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["position"]["revision"] = 1.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["position"]["spaces"][2]["support"] = 0.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["position"]["spaces"][2]["terror"] = 2.into();
    cases.push(record);
    let mut record = resolved;
    record["payload"]["position"]["spaces"][2]["pieces"][3]["count"] = 2.into();
    cases.push(record);
    for record in cases {
        let bytes = reseal(&record);
        std::fs::write(&path, &bytes).unwrap();
        let refusal = Session::open(&path, None).err().expect("admitted");
        assert_another_build(&format!("{refusal:#}"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
    }
}

#[test]
fn failed_record_replacement_does_not_consume_the_roll_or_publish_its_effect() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roll.json");
    let mut session = Session::open(&path, None).unwrap();
    act(&mut session, "terror");
    act(&mut session, "kidnap");
    std::fs::remove_file(previous_path(&path)).unwrap();
    let before = session.game().clone();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::create_dir(previous_path(&path)).unwrap();
    assert!(session.submit(Faction::M26, 2, "2:kidnap:roll").is_err());
    assert_eq!(session.game(), &before);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn real_record_replacement_resumes_both_states_and_preserves_previous() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.json");
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe().resources, [6, 0, 5, 15]);
    act(&mut session, "terror");
    let initial = std::fs::read(&path).unwrap();
    session.submit(Faction::M26, 1, "1:end-terror").unwrap();
    let finished = std::fs::read(&path).unwrap();
    assert_ne!(initial, finished);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), initial);
    let resumed = Session::open(&path, None).unwrap();
    assert!(resumed.observe().finished);
    assert_eq!(resumed.observe().resources, [6, 0, 5, 15]);
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert!(!recovered.observe().finished);
    assert_eq!(std::fs::read(&path).unwrap(), finished);
}

#[test]
fn stale_writer_and_failed_replacement_do_not_publish_an_effect() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut first = Session::open(&path, None).unwrap();
    let mut stale = Session::open(&path, None).unwrap();
    let opening = stale.observe();
    first.submit(Faction::M26, 0, "0:terror").unwrap();
    let valid = std::fs::read(&path).unwrap();
    assert_eq!(
        stale
            .submit(Faction::M26, 0, "0:terror")
            .unwrap_err()
            .to_string(),
        "Save changed in another session; reload before continuing"
    );
    assert_eq!(stale.observe(), opening);
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    first.save().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    // Refused until it reloads: the reloaded session shows the decision the other
    // session saved, and its next decision is saved after it.
    drop(stale);
    let mut reloaded = Session::open(&path, None).unwrap();
    assert_eq!(reloaded.observe(), first.observe());
    reloaded.submit(Faction::M26, 1, "1:end-terror").unwrap();
    assert_eq!(decisions(&path), ["terror", "end-terror"]);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), valid);
    assert!(Session::open(&path, None).unwrap().observe().finished);
    let blocked = dir.path().join("blocked.json");
    let mut session = Session::open(&blocked, None).unwrap();
    let initial = std::fs::read(&blocked).unwrap();
    std::fs::create_dir(previous_path(&dir.path().join("blocked.json"))).unwrap();
    assert!(session.submit(Faction::M26, 0, "0:pass").is_err());
    assert!(!session.observe().finished);
    assert_eq!(std::fs::read(&blocked).unwrap(), initial);
}

#[test]
fn corrupt_incompatible_and_semantically_wrong_records_preserve_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    let initial = std::fs::read(&path).unwrap();
    session.submit(Faction::M26, 0, "0:pass").unwrap();
    let finished = std::fs::read(&path).unwrap();
    let previous = previous_path(&dir.path().join("task.json"));
    let record: serde_json::Value = serde_json::from_slice(&finished).unwrap();
    let mut cases = vec![b"not JSON".to_vec(), vec![b'x'; 4 * 1024 * 1024 + 1]];
    for pointer in ["/identity/source", "/identity/format"] {
        let mut changed = record.clone();
        *changed.pointer_mut(pointer).unwrap() = "wrong identity".into();
        cases.push(reseal(&changed));
    }
    let mut changed = record.clone();
    changed["checksum"] = "wrong checksum".into();
    cases.push(serde_json::to_vec(&changed).unwrap());
    // Art is in no identity: an identity naming it is not this build's.
    let mut with_art = record.clone();
    with_art["identity"]["art"] = "0".repeat(64).into();
    cases.push(reseal(&with_art));
    // A valid checksum does not authorize a fabricated resource balance.
    let mut wrong = record.clone();
    wrong["payload"]["position"]["resources"][1] = 9.into();
    cases.push(reseal(&wrong));
    // An intact record whose position does not decode in this build's shape was
    // written by another build.
    let mut undecodable = record.clone();
    undecodable["payload"]["position"]["resources"] = "not resources".into();
    cases.push(reseal(&undecodable));
    for (index, bytes) in cases.into_iter().enumerate() {
        std::fs::write(&path, &bytes).unwrap();
        let refusal = Session::open(&path, None).err().unwrap().to_string();
        let expected = match index {
            0 => "Malformed save",
            1 => "Save exceeds its size limit",
            2..=3 | 5..=7 => "Another build wrote this save",
            4 => "Save checksum mismatch",
            _ => unreachable!(),
        };
        assert!(refusal.starts_with(expected), "{index}: {refusal}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read(&previous).unwrap(), initial);
        assert!(Session::open(&path, Some(&path)).is_err());
    }
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert_eq!(recovered.observe().resources, [6, 0, 5, 15]);
    assert!(!recovered.observe().finished);
}

#[test]
fn missing_main_with_recovery_history_is_not_a_fresh_task() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    session.submit(Faction::M26, 0, "0:pass").unwrap();
    let previous = previous_path(&dir.path().join("task.json"));
    let recovery = std::fs::read(&previous).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        Session::open(&path, None).err().unwrap().to_string(),
        "Main save is missing but recovery history exists; recover explicitly to an unused path"
    );
    assert!(!path.exists());
    assert_eq!(std::fs::read(&previous).unwrap(), recovery);
    assert!(Session::open(&dir.path().join("recovered.json"), Some(&path)).is_ok());
}

/// A write first admits the kept `.previous` before rotating it. When that record's
/// decisions do not replay to its position, the write is refused in a save that
/// opened, with the words for a save another build wrote, and neither record nor
/// the published position changes.
#[test]
fn a_previous_record_that_does_not_replay_refuses_the_next_write_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    act(&mut session, "terror");
    let main = std::fs::read(&path).unwrap();
    // The kept opening position, now claiming a decision it does not hold.
    let mut previous = record(&previous_path(&path));
    assert_eq!(previous["payload"]["decisions"], serde_json::json!([]));
    previous["payload"]["decisions"] = serde_json::json!(["terror"]);
    let previous = reseal(&previous);
    std::fs::write(previous_path(&path), &previous).unwrap();
    let published = session.observe();

    let refusal = session
        .submit(Faction::M26, 1, "1:end-terror")
        .unwrap_err()
        .to_string();
    assert_another_build(&refusal);
    assert!(
        refusal.contains("(its 1 decision does not reproduce its position)"),
        "{refusal}"
    );
    assert_eq!(session.observe(), published);
    assert_eq!(std::fs::read(&path).unwrap(), main);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
}

#[test]
fn pending_giver_choice_and_accepted_transfer_resume_without_duplicate_or_failed_effects() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("transfer.json");
    let mut session = Session::open(&path, None).unwrap();
    for (owner, rev, action) in [
        (Faction::M26, 0, "terror"),
        (Faction::M26, 1, "transfer"),
        (Faction::M26, 2, "ask:3:1"),
    ] {
        session
            .submit(owner, rev, &format!("{rev}:{action}"))
            .unwrap();
    }
    let pending = session.observe();
    drop(session);
    let mut resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe(), pending);
    assert_eq!(resumed.observe().owner, Faction::Syndicate);
    // Wait is saved (#75): a restart finds the request set aside, exactly once.
    resumed.submit(Faction::Syndicate, 3, "3:wait").unwrap();
    let waited = resumed.observe();
    assert!(matches!(
        waited.transfer,
        Some(Transfer::Authorizing { waited: true, .. })
    ));
    drop(resumed);
    let mut resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe(), waited);
    let mut stale_writer = Session::open(&path, None).unwrap();
    resumed
        .submit(Faction::Syndicate, 4, "4:authorize:1")
        .unwrap();
    let accepted = std::fs::read(&path).unwrap();
    assert!(
        stale_writer
            .submit(Faction::Syndicate, 4, "4:authorize:1")
            .is_err()
    );
    assert_eq!(stale_writer.observe(), waited);
    drop(resumed);
    let mut resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe().resources, [6, 1, 5, 14]);
    assert!(
        resumed
            .submit(Faction::Syndicate, 4, "4:authorize:1")
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), accepted);
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert_eq!(recovered.observe(), waited);
}

/// A submitted decision, as the session received it.
struct Submitted {
    owner: Faction,
    revision: u64,
    id: String,
}

/// Submits the offered choice as the faction that owns it.
fn act(session: &mut Session, action: &str) -> Submitted {
    let view = session.observe();
    let id = format!("{}:{action}", view.revision);
    let owner = view
        .choices
        .iter()
        .find(|c| c.id == id)
        .map_or(view.owner, |c| c.owner);
    session
        .submit(owner, view.revision, &id)
        .unwrap_or_else(|e| panic!("{id}: {e}"));
    Submitted {
        owner,
        revision: view.revision,
        id,
    }
}

#[test]
fn a_new_save_starts_at_the_action_choice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let session = Session::open(&path, None).unwrap();
    let view = session.observe();
    assert_eq!(view.action, ActionChoice::Open);
    assert_eq!((view.revision, view.owner), (0, Faction::M26));
    assert_eq!(view.executing, None);
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(record["payload"]["position"]["action"], "Open");
}

/// Each new decision state saves before it is published and resumes exactly once:
/// a stale resubmission after restart is refused and leaves the bytes unchanged.
#[test]
fn action_choice_pending_request_and_pass_survive_restart_exactly_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in [
        "terror",
        "transfer",
        "ask:3:1",
        "back-to-choice",
        "terror",
        "back-to-choice",
        "pass",
    ] {
        let before = session.observe();
        let id = format!("{}:{action}", before.revision);
        let owner = before
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(before.owner, |c| c.owner);
        act(&mut session, action);
        let after = session.observe();
        let bytes = std::fs::read(&path).unwrap();
        drop(session);
        session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after, "{action}");
        assert!(session.submit(owner, before.revision, &id).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let view = session.observe();
    assert_eq!(view.action, ActionChoice::Pass);
    assert_eq!(view.resources, [6, 1, 5, 15]);
    assert_eq!(view.revision, 7);
    assert_eq!(
        decisions(&path),
        [
            "terror",
            "transfer",
            "ask:3:1",
            "back-to-choice",
            "terror",
            "back-to-choice",
            "pass"
        ]
    );
}

#[test]
fn manifesto_declines_replay_in_printed_order_without_taking_card_actions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifesto.json");
    let mut session = Session::create_seeded(&path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:pass",
        "syndicate:construct",
        "construct:camaguey-city:place",
        "end-construct",
        "advance-card",
        "m26:event",
    ] {
        act(&mut session, action);
    }

    let before = session.observe();
    let random = session.game().random_state();
    let mut stale: Option<Submitted> = None;
    let owners = [
        Faction::M26,
        Faction::Directorio,
        Faction::Syndicate,
        Faction::Government,
    ];
    for owner in owners {
        let seen = session.observe();
        assert_eq!(seen.owner, owner);
        assert_eq!(seen.executing, Some(Faction::M26));
        assert_eq!(seen.event_executor, Some(Faction::M26));
        assert_eq!(seen.eligible, [true, true, true, false]);
        assert_eq!(seen.next, None);
        let decline = seen
            .choices
            .iter()
            .find(|c| c.action() == "manifesto:decline")
            .unwrap();
        let initial_save = std::fs::read(&path).unwrap();
        let previous_save = std::fs::read(previous_path(&path)).unwrap();
        let wrong_owner = if owner == Faction::M26 {
            Faction::Directorio
        } else {
            Faction::M26
        };
        assert!(
            session
                .submit(wrong_owner, seen.revision, &decline.id)
                .unwrap_err()
                .to_string()
                .contains("decision owner")
        );
        if let Some(stale) = stale {
            assert!(
                session
                    .submit(stale.owner, stale.revision, &stale.id)
                    .unwrap_err()
                    .to_string()
                    .contains("Stale match revision")
            );
            assert!(
                session
                    .submit(owner, seen.revision, &stale.id)
                    .unwrap_err()
                    .to_string()
                    .contains("Stale or unknown choice identity")
            );
        }
        assert_eq!(session.observe(), seen);
        assert_eq!(std::fs::read(&path).unwrap(), initial_save);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous_save);
        stale = Some(act(&mut session, "manifesto:decline"));
        let after = session.observe();
        assert_eq!(after.revision, seen.revision + 1);
        assert_eq!(after.spaces, before.spaces);
        assert_eq!(after.available, before.available);
        assert_eq!(after.resources, before.resources);
        assert_eq!(after.standings, before.standings);
        assert_eq!(after.eligible, before.eligible);
        assert_eq!(after.passed, [false; 4]);
        assert_eq!(session.game().random_state(), random);
        assert_eq!(std::fs::read(previous_path(&path)).unwrap(), initial_save);
        let reports = session.reports();
        drop(session);
        session = Session::open(&path, None).unwrap();
        assert_eq!(session.observe(), after);
        assert_eq!(session.reports(), reports);
    }
    let after = session.observe();
    assert_eq!(after.owner, Faction::Directorio);
    assert_eq!(after.executing, None);
    assert_eq!(after.event_executor, Some(Faction::M26));
    assert_eq!(
        after.cards.current.status,
        cuba_libre_unfolded::EventStatus::SingleResolved {
            faction: Faction::M26
        }
    );
    assert_eq!(after.eligibility.m26_next_card, Some(true));
    assert_eq!(
        serde_json::to_value(&after).unwrap()["manifesto"]["declines"],
        serde_json::json!(owners)
    );
    let next = after.next.unwrap();
    assert_eq!(
        (next.faction, next.role.as_str()),
        (Faction::Directorio, "2nd Eligible")
    );
    assert_eq!(
        next.options,
        [
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)"
        ]
    );
    assert_eq!(
        after
            .choices
            .iter()
            .map(|choice| (choice.action(), choice.owner, choice.supported))
            .collect::<Vec<_>>(),
        [
            ("directorio:pass", Faction::Directorio, false),
            ("directorio:rally", Faction::Directorio, true),
            ("directorio:operation", Faction::Directorio, false),
        ],
    );
    let reports = session.reports();
    let report = reports.last().unwrap();
    assert_eq!(
        (report.card, report.faction, report.complete),
        (19, Faction::M26, true)
    );
    assert_eq!(report.recorded, Some("Event"));
    assert_eq!(
        report
            .articles
            .iter()
            .flat_map(|article| &article.lines)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "26July declined placement.",
            "Directorio declined placement.",
            "Syndicate declined placement.",
            "Government declined placement.",
        ]
    );

    let finished = std::fs::read(&path).unwrap();
    let previous = std::fs::read(previous_path(&path)).unwrap();
    let recovery_path = dir.path().join("recovered.json");
    let mut recovered = Session::open(&recovery_path, Some(&path)).unwrap();
    let pending = recovered.observe();
    assert_eq!(pending.revision, session.observe().revision - 1);
    assert_eq!(pending.owner, Faction::Government);
    assert_eq!(
        serde_json::to_value(&pending).unwrap()["manifesto"]["declines"],
        serde_json::json!(owners[..3])
    );
    assert_eq!(std::fs::read(&recovery_path).unwrap(), previous);
    act(&mut recovered, "manifesto:decline");
    assert_eq!(recovered.observe(), session.observe());
    assert_eq!(recovered.reports(), reports);
    assert_eq!(std::fs::read(&path).unwrap(), finished);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
}

#[test]
fn manifesto_own_removal_recalculates_placement_and_control() {
    use Faction::M26;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifesto-removal.json");
    let mut session = open_manifesto(&path);

    // Oriente has one active 26July Guerrilla and no other piece after shaded
    // Alberto Bayo. Its lone Guerrilla qualifies the space and gives Control.
    let before = session.observe();
    let oriente = before.spaces.iter().find(|s| s.id == "oriente").unwrap();
    assert_eq!(oriente.control(), Some(M26));
    let placement = before
        .choices
        .iter()
        .find(|c| c.action() == "manifesto:place:oriente:guerrilla-guerrilla")
        .unwrap();
    assert_eq!(
        before
            .available
            .iter()
            .find(|g| g.faction == M26 && g.kind == Kind::Guerrilla)
            .unwrap()
            .count,
        12
    );

    act(&mut session, "remove");
    let removal = session.observe();
    assert_eq!(removal.owner, M26);
    assert_eq!(
        removal
            .choices
            .iter()
            .filter(|c| c.action().starts_with("remove:oriente:"))
            .map(|c| c.action())
            .collect::<Vec<_>>(),
        ["remove:oriente:active:1"]
    );
    for (owner, action) in [
        (M26, "remove:oriente:active:0"),
        (M26, "remove:oriente:active:2"),
        (M26, "remove:havana:active:1"),
        (Faction::Directorio, "remove:oriente:active:1"),
    ] {
        assert!(
            session
                .submit(
                    owner,
                    removal.revision,
                    &format!("{}:{action}", removal.revision)
                )
                .is_err()
        );
    }
    assert_eq!(session.observe(), removal);

    act(&mut session, "remove:oriente:active:1");
    let removed = session.observe();
    let oriente = removed.spaces.iter().find(|s| s.id == "oriente").unwrap();
    assert!(oriente.pieces.is_empty());
    assert_eq!(oriente.control(), None);
    assert_eq!(
        removed
            .available
            .iter()
            .find(|g| g.faction == M26 && g.kind == Kind::Guerrilla)
            .unwrap()
            .count,
        13
    );
    assert_eq!(removed.resources, before.resources);
    assert_eq!(removed.manifesto, before.manifesto);

    let pre_done = std::fs::read(&path).unwrap();
    drop(session);
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), removed);
    act(&mut session, "done-removal");
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), pre_done);
    let returned = session.observe();
    assert_eq!(returned.owner, M26);
    assert_eq!(returned.removal, None);
    assert_eq!(
        returned
            .space_reasons
            .iter()
            .find(|r| r.space == "oriente")
            .unwrap()
            .reason,
        "26July has no piece here."
    );
    assert!(
        !returned
            .choices
            .iter()
            .any(|c| c.space() == Some("oriente"))
    );
    for (revision, id) in [
        (before.revision, placement.id.clone()),
        (
            returned.revision,
            format!("{}:{}", returned.revision, placement.action()),
        ),
    ] {
        assert!(session.submit(M26, revision, &id).is_err());
    }
    assert_eq!(session.observe(), returned);
    act(&mut session, "manifesto:place:santiago:guerrilla-base");
    assert_eq!(session.observe().owner, Faction::Directorio);

    // During Directorio's placement, 26July still executes the Event. It may
    // remove its own piece and then return the placement choice to Directorio.
    let directorio = session.observe();
    assert!(
        session
            .submit(
                Faction::Directorio,
                directorio.revision,
                &format!("{}:remove", directorio.revision)
            )
            .is_err()
    );
    assert_eq!(act(&mut session, "remove").owner, M26);
    assert_eq!(session.observe().owner, M26);
    act(&mut session, "remove:sierra-maestra:base:1");
    act(&mut session, "done-removal");
    let handed_back = session.observe();
    assert_eq!(handed_back.owner, Faction::Directorio);
    assert_eq!(handed_back.manifesto, directorio.manifesto);
    assert!(
        handed_back
            .choices
            .iter()
            .filter(|c| c.space().is_some())
            .all(|c| c.owner == Faction::Directorio)
    );
}

fn open_manifesto(path: &Path) -> Session {
    let mut session = Session::create_seeded(path, Session::KIDNAP_ROLLS_1).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:pass",
        "syndicate:pass",
        "advance-card",
        "m26:event",
    ] {
        act(&mut session, action);
    }
    session
}

#[test]
fn mixed_manifesto_reports_keep_the_transfer_between_placements_after_replay() {
    use Faction::{Directorio, Government, M26, Syndicate};
    use cuba_libre_unfolded::reports::Action;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifesto-mixed.json");
    let mut session = open_manifesto(&path);
    assert_eq!(act(&mut session, "manifesto:decline").owner, M26);
    assert_eq!(session.observe().owner, Directorio);
    act(&mut session, "transfer");
    act(&mut session, "ask:0:2");
    let pending = session.observe();
    assert_eq!(pending.owner, Government);
    assert_eq!(pending.executing, Some(M26));
    let revision = pending.revision;
    drop(session);

    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), pending);
    assert_eq!(act(&mut session, "authorize:2").owner, Government);
    assert_eq!(session.observe().owner, Directorio);
    assert_eq!(
        act(&mut session, "manifesto:place:havana:base-base").owner,
        Directorio
    );
    assert_eq!(act(&mut session, "manifesto:decline").owner, Syndicate);
    assert_eq!(
        act(&mut session, "manifesto:place:havana:troops-police").owner,
        Government
    );
    let finished = session.observe();
    assert_eq!(finished.owner, Directorio);
    assert_eq!(finished.executing, None);
    let next = finished.next.as_ref().unwrap();
    assert_eq!(
        (next.faction, next.role.as_str()),
        (Directorio, "2nd Eligible")
    );

    let reports = session.reports();
    let whole = reports.last().unwrap();
    let since = session.reports_since(revision);
    let [continuation] = since.as_slice() else {
        panic!("expected one report after the request: {since:?}");
    };
    for report in [whole, continuation] {
        assert_eq!(
            (report.card, report.faction, report.action),
            (19, M26, Action::Manifesto)
        );
        assert_eq!(report.recorded, Some("Event"));
        assert!(report.complete);
        assert_eq!(report.to_revision, finished.revision);
    }
    assert!(!whole.continues);
    assert!(continuation.continues);
    assert_eq!(continuation.from_revision, revision);
    let [declined, transfer, directorio, syndicate, government] = whole.articles.as_slice() else {
        panic!(
            "expected four placement outcomes and a transfer: {:?}",
            whole.articles
        );
    };
    assert_eq!(declined.lines, ["26July declined placement."]);
    assert_eq!(transfer.title, "Government gives 26July 2 Resources");
    assert_eq!(directorio.lines, ["Directorio placed 2 Bases in Havana."]);
    assert_eq!(syndicate.lines, ["Syndicate declined placement."]);
    assert_eq!(
        government.lines,
        ["Government placed 1 Troop + 1 Police cube in Havana."]
    );
    assert_eq!(continuation.articles.len(), 4);
    assert_eq!(continuation.articles[0].title, transfer.title);
    assert_eq!(continuation.articles[1..], whole.articles[2..]);

    drop(session);
    let replayed = Session::open(&path, None).unwrap();
    assert_eq!(replayed.observe(), finished);
    assert_eq!(replayed.reports(), reports);
    assert_eq!(replayed.reports_since(revision), since);
}

#[test]
fn manifesto_transfer_retains_giver_authority_and_resumes_the_placement() {
    use Faction::{Directorio, Government, M26, Syndicate};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("manifesto-transfer.json");
    let mut session = open_manifesto(&path);
    act(&mut session, "remove");
    act(&mut session, "remove:oriente:active:1");
    act(&mut session, "done-removal");
    act(&mut session, "manifesto:place:santiago:guerrilla-base");
    let placement = session.observe();
    assert_eq!(placement.owner, Directorio);
    assert_eq!(placement.resources, [8, 0, 5, 16]);

    let transfer = format!("{}:transfer", placement.revision);
    assert!(
        session
            .submit(Directorio, placement.revision, &transfer)
            .is_err()
    );
    assert_eq!(act(&mut session, "transfer").owner, M26);
    let transfer = session.observe();
    assert_eq!(transfer.owner, M26);
    assert_eq!(transfer.transfer_terms.len(), 6);
    assert!(
        transfer
            .transfer_terms
            .iter()
            .all(|t| t.giver == M26 || t.recipient == M26)
    );
    let government = transfer
        .transfer_terms
        .iter()
        .find(|t| t.giver == Government)
        .unwrap();
    assert_eq!(
        (
            government.balance,
            government.held,
            government.room,
            government.reserved,
            government.limit
        ),
        (8, 0, 49, 0, 8)
    );
    assert!(
        session
            .submit(
                M26,
                transfer.revision,
                &format!("{}:ask:0:9", transfer.revision)
            )
            .is_err()
    );
    assert!(
        session
            .submit(
                M26,
                transfer.revision,
                &format!("{}:give:3:1", transfer.revision)
            )
            .is_err()
    );
    act(&mut session, "give:3:0");
    assert_eq!(session.observe().resources, [8, 0, 5, 16]);
    assert_eq!(session.observe().owner, Directorio);

    act(&mut session, "transfer");
    let request = act(&mut session, "ask:0:2");
    let pending = session.observe();
    assert_eq!(pending.owner, Government);
    assert_eq!(pending.executing, Some(M26));
    assert_eq!(
        pending.manifesto.as_ref().unwrap().pending(),
        Some(Directorio)
    );
    assert_eq!(pending.resources, [8, 0, 5, 16]);
    assert_eq!(pending.transfer_terms.len(), 1);
    let authorize = |revision| format!("{revision}:authorize:8");
    assert!(
        session
            .submit(
                Government,
                pending.revision,
                &format!("{}:authorize:9", pending.revision)
            )
            .is_err()
    );
    act(&mut session, "wait");
    let aside = session.observe();
    assert_eq!(aside.owner, Government);
    assert!(matches!(
        aside.transfer,
        Some(Transfer::Authorizing {
            amount: 2,
            waited: true,
            ..
        })
    ));
    assert!(
        !aside
            .choices
            .iter()
            .any(|c| matches!(c.action(), "end-terror" | "back-to-choice"))
    );
    let bytes = std::fs::read(&path).unwrap();
    let previous = std::fs::read(previous_path(&path)).unwrap();
    drop(session);

    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe(), aside);
    for (owner, revision, id) in [
        (M26, aside.revision, authorize(aside.revision)),
        (
            Directorio,
            aside.revision,
            format!("{}:manifesto:decline", aside.revision),
        ),
        (Government, pending.revision, authorize(pending.revision)),
        (request.owner, request.revision, request.id),
    ] {
        assert!(session.submit(owner, revision, &id).is_err());
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), previous);
    act(&mut session, "reopen");
    let reopened = session.observe();
    assert_eq!(reopened.owner, Government);
    assert_eq!(
        reopened
            .choices
            .iter()
            .filter_map(|c| c.action().strip_prefix("authorize:"))
            .collect::<Vec<_>>(),
        ["0", "1", "2", "3", "4", "5", "6", "7", "8"]
    );
    let before_authorization = std::fs::read(&path).unwrap();
    act(&mut session, "authorize:8");
    assert_eq!(
        std::fs::read(previous_path(&path)).unwrap(),
        before_authorization
    );
    let returned = session.observe();
    assert_eq!(returned.owner, Directorio);
    assert_eq!(returned.resources, [0, 8, 5, 16]);
    assert_eq!(returned.manifesto, placement.manifesto);
    assert_eq!(returned.spaces, placement.spaces);

    for answer in ["decline", "authorize:0"] {
        act(&mut session, "transfer");
        act(&mut session, "ask:3:1");
        let pending = session.observe();
        assert_eq!(pending.owner, Syndicate);
        assert_eq!(pending.resources, [0, 8, 5, 16]);
        act(&mut session, answer);
        assert_eq!(session.observe().owner, Directorio);
        assert_eq!(session.observe().resources, [0, 8, 5, 16]);
    }

    act(&mut session, "transfer");
    act(&mut session, "give:3:8");
    assert_eq!(session.observe().owner, Directorio);
    assert_eq!(session.observe().resources, [0, 0, 5, 24]);
    act(&mut session, "manifesto:decline");
    act(&mut session, "manifesto:decline");
    let pre_finish = std::fs::read(&path).unwrap();
    act(&mut session, "manifesto:decline");
    let finished = session.observe();
    let reports = session.reports();
    let observation = serde_json::to_value(&finished).unwrap();
    let manifesto = &observation["manifesto"];
    assert_eq!(finished.owner, Directorio);
    assert_eq!(finished.executing, None);
    assert_eq!(finished.resources, [0, 0, 5, 24]);
    assert_eq!(manifesto["placements"].as_array().unwrap().len(), 1);
    assert_eq!(
        manifesto["declines"],
        serde_json::json!([Directorio, Syndicate, Government])
    );
    assert_eq!(finished.spaces, placement.spaces);
    assert_eq!(finished.available, placement.available);
    let supported = finished
        .choices
        .iter()
        .filter(|choice| choice.supported)
        .map(|choice| choice.action())
        .collect::<Vec<_>>();
    assert_eq!(supported, ["directorio:rally"]);
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), pre_finish);
    drop(session);
    let resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe(), finished);
    assert_eq!(resumed.reports(), reports);
    let recovery_path = dir.path().join("manifesto-transfer-recovered.json");
    let mut recovered = Session::open(&recovery_path, Some(&path)).unwrap();
    assert_eq!(std::fs::read(&recovery_path).unwrap(), pre_finish);
    assert_eq!(recovered.observe().owner, Government);
    act(&mut recovered, "manifesto:decline");
    assert_eq!(recovered.observe(), finished);
    assert_eq!(recovered.reports(), reports);
}

#[test]
fn directorio_recruits_two_in_camaguey_after_its_manifesto_placement() {
    use Faction::Directorio;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paid-rally.json");
    let mut session = open_manifesto(&path);
    for action in [
        "manifesto:decline",
        "manifesto:place:camaguey-province:guerrilla-base",
        "manifesto:decline",
        "manifesto:decline",
        "directorio:rally",
    ] {
        act(&mut session, action);
    }
    let before = session.observe();
    assert_eq!(before.owner, Directorio);
    assert_eq!(before.executing, Some(Directorio));
    assert_eq!((before.current_card, before.preview_card), (19, 20));
    let position = |view: &cuba_libre_unfolded::Observation| {
        let province = view
            .spaces
            .iter()
            .find(|s| s.id == "camaguey-province")
            .unwrap();
        assert_eq!(province.population, 1);
        assert!(province.pieces.iter().any(|piece| {
            piece.faction == Directorio && piece.kind == Kind::Base && piece.count == 1
        }));
        let quantity = |pieces: &[cuba_libre_unfolded::Group]| {
            pieces
                .iter()
                .filter(|g| {
                    g.faction == Directorio
                        && g.kind == Kind::Guerrilla
                        && g.state == State::Underground
                })
                .map(|g| g.count)
                .sum::<u8>()
        };
        (
            quantity(&province.pieces),
            quantity(&view.available),
            view.resources[Directorio.index()],
        )
    };
    assert_eq!(position(&before), (2, 11, 5));
    assert_eq!(
        before
            .choices
            .iter()
            .filter(|choice| {
                choice.supported
                    && choice
                        .action()
                        .starts_with("paid-rally:camaguey-province:place:")
            })
            .map(|choice| choice.action())
            .collect::<Vec<_>>(),
        [
            "paid-rally:camaguey-province:place:0",
            "paid-rally:camaguey-province:place:1",
            "paid-rally:camaguey-province:place:2",
        ],
    );
    let action = "paid-rally:camaguey-province:place:2";
    let choice = before
        .choices
        .iter()
        .find(|c| c.action() == action)
        .unwrap();
    assert!(choice.supported);
    assert_eq!(choice.space(), Some("camaguey-province"));
    assert_eq!(choice.owner, Directorio);
    let payment = act(&mut session, action);

    let after = session.observe();
    assert_eq!(position(&after), (4, 9, 4));
    assert_eq!(after.manifesto, before.manifesto);
    assert_eq!(after.standings, before.standings);
    assert!(
        !after
            .choices
            .iter()
            .any(|c| c.space() == Some("camaguey-province"))
    );
    assert!(
        after
            .choices
            .iter()
            .any(|c| c.supported && c.space().is_some())
    );
    drop(session);

    let mut reopened = Session::open(&path, None).unwrap();
    assert_eq!(reopened.observe(), after);
    assert!(
        reopened
            .submit(payment.owner, payment.revision, &payment.id)
            .is_err()
    );
    assert_eq!(reopened.observe(), after);
}

#[test]
fn directorio_exchanges_two_in_camaguey_after_the_manifesto() {
    use Faction::Directorio;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("paid-exchange.json");
    let mut session = open_manifesto(&path);
    for action in [
        "manifesto:decline",
        "manifesto:place:camaguey-province:guerrilla-base",
        "manifesto:decline",
        "manifesto:decline",
        "directorio:rally",
    ] {
        act(&mut session, action);
    }
    let before = session.observe();
    let action = "paid-rally:camaguey-province:base:2";
    let offered = before
        .choices
        .iter()
        .find(|c| c.action() == action)
        .unwrap();
    let preview = offered.preview.as_ref().unwrap();
    let [returned] = preview.removed.as_slice() else {
        panic!("Expected a Guerrilla return: {preview:?}");
    };
    assert_eq!(
        (
            returned.faction,
            returned.kind,
            returned.state,
            returned.count
        ),
        (Directorio, Kind::Guerrilla, State::Underground, 2)
    );
    let [base] = preview.placement_available_forces.as_slice() else {
        panic!("Expected a Base from Available Forces: {preview:?}");
    };
    assert_eq!((base.piece.kind, base.from, base.to), (Kind::Base, 3, 2));
    assert!(preview.control.is_empty());
    let payment = act(&mut session, action);
    let after = session.observe();
    let counts = |view: &cuba_libre_unfolded::Observation| {
        let space = view
            .spaces
            .iter()
            .find(|s| s.id == "camaguey-province")
            .unwrap();
        let count = |pieces: &[cuba_libre_unfolded::Group], kind| {
            pieces
                .iter()
                .filter(|p| p.faction == Directorio && p.kind == kind)
                .map(|p| p.count)
                .sum::<u8>()
        };
        let standing = view
            .standings
            .iter()
            .find(|s| s.faction == Directorio)
            .unwrap();
        assert_eq!(space.control(), Some(Directorio));
        (
            count(&space.pieces, Kind::Guerrilla),
            count(&view.available, Kind::Guerrilla),
            count(&space.pieces, Kind::Base),
            count(&view.available, Kind::Base),
            view.resources[Directorio.index()],
            standing.score,
        )
    };
    assert_eq!(counts(&before), (2, 11, 1, 3, 5, 2));
    assert_eq!(counts(&after), (0, 13, 2, 2, 4, 3));
    assert!(
        !after
            .choices
            .iter()
            .any(|c| c.space() == Some("camaguey-province"))
    );
    drop(session);
    let mut reopened = Session::open(&path, None).unwrap();
    assert_eq!(reopened.observe(), after);
    assert!(
        reopened
            .submit(payment.owner, payment.revision, &payment.id)
            .is_err()
    );
    assert_eq!(reopened.observe(), after);
}

#[test]
fn directorio_can_recruit_one_in_camaguey_after_declining_the_manifesto() {
    use Faction::Directorio;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("no-base.json");
    let mut session = open_manifesto(&path);
    for _ in Faction::ALL {
        act(&mut session, "manifesto:decline");
    }
    let menu = session.observe();
    act(&mut session, "directorio:rally");
    let before = session.observe();
    assert_eq!(before.resources, menu.resources);
    assert_eq!(before.spaces, menu.spaces);
    assert_eq!(before.available, menu.available);
    let camaguey = "camaguey-province";
    let province = before
        .spaces
        .iter()
        .find(|space| space.id == camaguey)
        .unwrap();
    assert_eq!(province.population, 1);
    assert_eq!(province.support, -1);
    assert!(
        !province
            .pieces
            .iter()
            .any(|piece| piece.faction == Directorio && piece.kind == Kind::Base)
    );
    let offers = before
        .choices
        .iter()
        .filter(|choice| choice.space() == Some(camaguey))
        .collect::<Vec<_>>();
    let [single] = offers.as_slice() else {
        panic!("Expected exactly one recruitment choice: {offers:?}")
    };
    assert!(single.supported);
    assert_eq!(single.action(), "paid-rally:camaguey-province:place:1");
    let preview = single.preview.as_ref().unwrap();
    let [change] = preview.placement_available_forces.as_slice() else {
        panic!("Expected a single placement: {preview:?}")
    };
    assert_eq!(
        (
            change.piece.state,
            change.piece.count,
            change.from,
            change.to
        ),
        (State::Underground, 1, 12, 11)
    );
    let checkpoint = std::fs::read(&path).unwrap();
    assert!(
        session
            .submit(Faction::M26, before.revision, &single.id)
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), checkpoint);
    assert_eq!(session.observe(), before);
    act(&mut session, single.action());
    let after = session.observe();
    let province = after
        .spaces
        .iter()
        .find(|space| space.id == camaguey)
        .unwrap();
    assert!(
        province
            .pieces
            .iter()
            .any(|piece| piece.faction == Directorio
                && piece.kind == Kind::Guerrilla
                && piece.state == State::Underground
                && piece.count == 2)
    );
    assert!(
        after
            .available
            .iter()
            .any(|piece| piece.faction == Directorio
                && piece.kind == Kind::Guerrilla
                && piece.count == 11)
    );
    assert_eq!(after.resources[Directorio.index()], 4);
    assert_eq!(after.manifesto, before.manifesto);
}
