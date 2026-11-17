use cuba_libre_unfolded::{ActionChoice, Faction, Game, save::Session};
use unfolded_record_replacement::{previous_path, save};

/// Reseal an edited record, so its checksum is intact and only what was edited
/// can be refused.
fn reseal(record: &serde_json::Value) -> Vec<u8> {
    let identity = serde_json::from_value(record["identity"].clone()).unwrap();
    save::seal(&identity, &record["payload"])
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
    altered["payload"]["spaces"][2]["pieces"][1]["count"] = 0.into();
    cases.push(altered); // Government Police are not removable
    let mut altered = source.clone();
    altered["payload"]["available"][3]["count"] = 12.into();
    cases.push(altered); // missing returned Guerrilla
    let mut altered = source.clone();
    altered["payload"]["spaces"][2]["pieces"]
        .as_array_mut()
        .unwrap()
        .pop();
    altered["payload"]["available"][3]["count"] = 14.into();
    cases.push(altered); // exact totals, but owed Terror lost its last Underground
    let mut altered = source.clone();
    altered["payload"]["removal"] = serde_json::json!({"Confirming":{"space":"santiago","kind":"Guerrilla","state":"Underground"}});
    cases.push(altered); // impossible pending confirmation
    let mut altered = source.clone();
    altered["payload"]["removal"] = "Selecting".into();
    altered["payload"]["transfer"] = "Selecting".into();
    cases.push(altered); // mutually exclusive local decisions
    let mut altered = source;
    altered["payload"]["spaces"][8]["pieces"][0]["state"] = "Active".into();
    cases.push(altered); // no Terror to activate this piece
    for record in cases {
        let bytes = reseal(&record);
        std::fs::write(&path, &bytes).unwrap();
        assert!(Session::open(&path, None).is_err(), "admitted {record}");
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
    assert!(session.observe().next_options.contains(&"Event".into()));
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
        record["payload"]["kidnap"][field] = value.into();
        cases.push(record);
    }
    let mut record = rolled.clone();
    record["payload"]["finished"] = true.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["revision"] = 1.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["spaces"][2]["support"] = 0.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["payload"]["spaces"][2]["terror"] = 2.into();
    cases.push(record);
    let mut record = resolved;
    record["payload"]["spaces"][2]["pieces"][3]["count"] = 2.into();
    cases.push(record);
    for record in cases {
        let bytes = reseal(&record);
        std::fs::write(&path, &bytes).unwrap();
        assert!(Session::open(&path, None).is_err(), "admitted {record}");
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
    first.submit(Faction::M26, 0, "0:pass").unwrap();
    let valid = std::fs::read(&path).unwrap();
    assert!(
        stale
            .submit(Faction::M26, 0, "0:pass")
            .unwrap_err()
            .to_string()
            .contains("another session")
    );
    assert!(!stale.observe().finished);
    assert_eq!(std::fs::read(&path).unwrap(), valid);
    first.save().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), valid);
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
    let mut cases = vec![b"not JSON".to_vec(), vec![b'x'; 256 * 1024 + 1]];
    for pointer in ["/identity/source", "/identity/art", "/identity/format"] {
        let mut changed = record.clone();
        *changed.pointer_mut(pointer).unwrap() = "wrong identity".into();
        cases.push(reseal(&changed));
    }
    let mut changed = record.clone();
    changed["checksum"] = "wrong checksum".into();
    cases.push(serde_json::to_vec(&changed).unwrap());
    // The record this game wrote before the shared save record.
    cases.push(
        serde_json::to_vec(&serde_json::json!({
            "format": 1, "source": record["identity"]["source"], "art": record["identity"]["art"],
            "game": record["payload"], "checksum": "0",
        }))
        .unwrap(),
    );
    // A valid checksum does not authorize a fabricated resource balance.
    let mut wrong = record.clone();
    wrong["payload"]["resources"][1] = 9.into();
    cases.push(reseal(&wrong));
    // A payload that does not decode as a game is malformed.
    let mut undecodable = record.clone();
    undecodable["payload"]["resources"] = "not resources".into();
    cases.push(reseal(&undecodable));
    for (index, bytes) in cases.into_iter().enumerate() {
        std::fs::write(&path, &bytes).unwrap();
        let refusal = Session::open(&path, None).err().unwrap().to_string();
        let expected = match index {
            0 => "Malformed save",
            1 => "This file exceeds",
            2..=4 | 6 => "Incompatible save format",
            5 => "Save checksum mismatch",
            8 => "Malformed save",
            _ => "",
        };
        assert!(refusal.starts_with(expected), "{index}: {refusal}");
        // The game's own check keeps its own words.
        assert!(index != 7 || !refusal.starts_with("Malformed"), "{refusal}");
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
    assert!(Session::open(&path, None).is_err());
    assert!(!path.exists());
    assert_eq!(std::fs::read(&previous).unwrap(), recovery);
    assert!(Session::open(&dir.path().join("recovered.json"), Some(&path)).is_ok());
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
    let waiting = std::fs::read(&path).unwrap();
    resumed.submit(Faction::Syndicate, 3, "3:wait").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), waiting);
    let mut stale_writer = Session::open(&path, None).unwrap();
    resumed
        .submit(Faction::Syndicate, 3, "3:authorize:1")
        .unwrap();
    let accepted = std::fs::read(&path).unwrap();
    assert!(
        stale_writer
            .submit(Faction::Syndicate, 3, "3:authorize:1")
            .is_err()
    );
    assert_eq!(stale_writer.observe(), pending);
    drop(resumed);
    let mut resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe().resources, [6, 1, 5, 14]);
    assert!(
        resumed
            .submit(Faction::Syndicate, 3, "3:authorize:1")
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), accepted);
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert_eq!(recovered.observe(), pending);
}

/// Submits the offered choice as the faction that owns it.
fn act(session: &mut Session, action: &str) {
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
    assert_eq!(record["payload"]["action"], "Open");
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
}

/// The payload a #8 build wrote: the same game record without the two new fields,
/// sealed with this build's unchanged identity.
fn legacy_record(path: &std::path::Path, game: &Game) -> Vec<u8> {
    let identity: serde_json::Value = serde_json::from_slice::<serde_json::Value>(
        &std::fs::read(path).unwrap(),
    )
    .unwrap()["identity"]
        .clone();
    let mut payload = serde_json::to_value(game).unwrap();
    let fields = payload.as_object_mut().unwrap();
    assert!(fields.remove("action").is_some() && fields.remove("effect").is_some());
    // #8's roll did not record the revision it created (#74 added it).
    if let Some(kidnap) = fields.get_mut("kidnap").and_then(|k| k.as_object_mut()) {
        assert!(kidnap.remove("revision").is_some());
    }
    save::seal(&serde_json::from_value(identity).unwrap(), &payload)
}

#[test]
fn a_record_from_8_resumes_with_terror_chosen_and_is_not_rewritten_until_a_decision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    drop(Session::open(&path, None).unwrap());
    // Untouched: Terror chosen at revision 0, so going back is offered.
    let untouched = legacy_record(&path, &Game::prepared());
    std::fs::write(&path, &untouched).unwrap();
    let session = Session::open(&path, None).unwrap();
    let view = session.observe();
    assert_eq!(view.action, ActionChoice::Terror);
    assert_eq!(view.executing, Some(Faction::M26));
    assert!(view.choices.iter().any(|c| c.id == "0:back-to-choice"));
    drop(session);
    assert_eq!(std::fs::read(&path).unwrap(), untouched);
    // After a Kidnap roll: resumed, but no going back.
    let mut rolled = Game::prepared_with_seed(5);
    rolled.submit(Faction::M26, 0, "0:kidnap").unwrap();
    rolled.submit(Faction::M26, 1, "1:kidnap:roll").unwrap();
    let after_effect = legacy_record(&path, &rolled);
    std::fs::write(&path, &after_effect).unwrap();
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe().action, ActionChoice::Terror);
    assert!(
        !session
            .observe()
            .choices
            .iter()
            .any(|c| c.id.ends_with(":back-to-choice"))
    );
    assert_eq!(std::fs::read(&path).unwrap(), after_effect);
    // The next decision writes the current record, keeping the old one as previous.
    act(&mut session, "terror:santiago");
    assert_eq!(std::fs::read(previous_path(&path)).unwrap(), after_effect);
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(record["payload"]["action"], "Terror");
}

/// A record retained from #8 play, named by `CUBA_COMPAT_SAVE` and used only as a copy.
#[test]
#[ignore = "needs a retained #8 save named by CUBA_COMPAT_SAVE"]
fn a_retained_8_save_copy_resumes_with_terror_chosen_without_being_rewritten() {
    let original = std::fs::read(std::env::var_os("CUBA_COMPAT_SAVE").unwrap()).unwrap();
    let record: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert!(record["payload"].get("action").is_none(), "not a #8 record");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("copy.json");
    std::fs::write(&path, &original).unwrap();
    let session = Session::open(&path, None).unwrap();
    let view = session.observe();
    assert_eq!(view.action, ActionChoice::Terror);
    assert_eq!(
        view.revision,
        record["payload"]["revision"].as_u64().unwrap()
    );
    assert_eq!(
        serde_json::to_value(view.resources).unwrap(),
        record["payload"]["resources"]
    );
    let back = view
        .choices
        .iter()
        .any(|c| c.id.ends_with(":back-to-choice"));
    eprintln!(
        "revision {} · finished {} · executing {:?} · back offered {back}",
        view.revision, view.finished, view.executing
    );
    drop(session);
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
