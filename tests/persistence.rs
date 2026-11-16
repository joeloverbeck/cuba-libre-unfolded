use cuba_libre_unfolded::{Faction, save::Session};
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
        "kidnap",
        "remove",
        "remove:santiago:underground",
        "confirm-removal",
        "remove",
        "cancel-removal",
        "terror:santiago",
        "remove",
        "remove:santiago:active",
        "confirm-removal",
        "finish",
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
    assert_eq!(session.observe().map_piece_count, 27);
    assert_eq!(session.observe().standings[1].score, 8);
}

#[test]
fn removal_admission_refuses_foreign_changes_wrong_supply_and_impossible_obligation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("admission.json");
    let mut session = Session::open(&path, None).unwrap();
    for action in [
        "kidnap",
        "remove",
        "remove:santiago:underground",
        "confirm-removal",
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
    let initial_rng = session.game().random_state();
    session.submit(Faction::M26, 0, "0:kidnap").unwrap();
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
    for (rev, action) in [(0, "0:kidnap"), (1, "1:kidnap"), (1, "1:finish")] {
        assert!(session.submit(Faction::M26, rev, action).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    session
        .submit(Faction::M26, 1, "1:terror:santiago")
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
            .submit(Faction::M26, 2, "2:terror:santiago")
            .is_err()
    );
    session.submit(Faction::M26, 2, "2:finish").unwrap();
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
    session.submit(Faction::M26, 0, "0:kidnap").unwrap();
    let rolled: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    session
        .submit(Faction::M26, 1, "1:terror:santiago")
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
    let before = session.game().clone();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::create_dir(previous_path(&path)).unwrap();
    assert!(session.submit(Faction::M26, 0, "0:kidnap").is_err());
    assert_eq!(session.game(), &before);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn real_record_replacement_resumes_both_states_and_preserves_previous() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("game.json");
    let mut session = Session::open(&path, None).unwrap();
    assert_eq!(session.observe().resources, [6, 0, 5, 15]);
    let initial = std::fs::read(&path).unwrap();
    session.submit(Faction::M26, 0, "0:finish").unwrap();
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
    first.submit(Faction::M26, 0, "0:finish").unwrap();
    let valid = std::fs::read(&path).unwrap();
    assert!(
        stale
            .submit(Faction::M26, 0, "0:finish")
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
    assert!(session.submit(Faction::M26, 0, "0:finish").is_err());
    assert!(!session.observe().finished);
    assert_eq!(std::fs::read(&blocked).unwrap(), initial);
}

#[test]
fn corrupt_incompatible_and_semantically_wrong_records_preserve_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    let initial = std::fs::read(&path).unwrap();
    session.submit(Faction::M26, 0, "0:finish").unwrap();
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
    session.submit(Faction::M26, 0, "0:finish").unwrap();
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
        (Faction::M26, 0, "transfer"),
        (Faction::M26, 1, "transfer:3:1"),
        (Faction::Syndicate, 2, "amount:1"),
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
        .submit(Faction::Syndicate, 3, "3:authorize")
        .unwrap();
    let accepted = std::fs::read(&path).unwrap();
    assert!(
        stale_writer
            .submit(Faction::Syndicate, 3, "3:authorize")
            .is_err()
    );
    assert_eq!(stale_writer.observe(), pending);
    drop(resumed);
    let mut resumed = Session::open(&path, None).unwrap();
    assert_eq!(resumed.observe().resources, [6, 1, 5, 14]);
    assert!(
        resumed
            .submit(Faction::Syndicate, 3, "3:authorize")
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), accepted);
    let recovered = Session::open(&dir.path().join("recovered.json"), Some(&path)).unwrap();
    assert_eq!(recovered.observe(), pending);
}
