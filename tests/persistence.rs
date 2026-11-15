use cuba_libre_unfolded::{Faction, save::Session};

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
    use sha2::{Digest, Sha256};
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
    let previous = std::fs::read(path.with_file_name("effects.json.previous")).unwrap();
    let mut cases = vec![];
    for (field, value) in [
        ("die", 7u64),
        ("proceeds", 7),
        ("government_before", 49),
        ("random_before", u64::MAX),
    ] {
        let mut record = rolled.clone();
        record["game"]["kidnap"][field] = value.into();
        cases.push(record);
    }
    let mut record = rolled.clone();
    record["game"]["finished"] = true.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["game"]["revision"] = 1.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["game"]["spaces"][2]["support"] = 0.into();
    cases.push(record);
    let mut record = resolved.clone();
    record["game"]["spaces"][2]["terror"] = 2.into();
    cases.push(record);
    let mut record = resolved;
    record["game"]["spaces"][2]["pieces"][3]["count"] = 2.into();
    cases.push(record);
    for mut record in cases {
        let game: cuba_libre_unfolded::Game =
            serde_json::from_value(record["game"].clone()).unwrap();
        record["checksum"] = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(1u32, &record["source"], &record["art"], game)).unwrap()
            )
        )
        .into();
        let bytes = serde_json::to_vec(&record).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        assert!(Session::open(&path, None).is_err(), "admitted {record}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            std::fs::read(path.with_file_name("effects.json.previous")).unwrap(),
            previous
        );
    }
}

#[test]
fn failed_record_replacement_does_not_consume_the_roll_or_publish_its_effect() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roll.json");
    let mut session = Session::open(&path, None).unwrap();
    let before = session.game().clone();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::create_dir(path.with_file_name("roll.json.previous")).unwrap();
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
    assert_eq!(
        std::fs::read(path.with_file_name("game.json.previous")).unwrap(),
        initial
    );
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
    std::fs::create_dir(dir.path().join("blocked.json.previous")).unwrap();
    assert!(session.submit(Faction::M26, 0, "0:finish").is_err());
    assert!(!session.observe().finished);
    assert_eq!(std::fs::read(&blocked).unwrap(), initial);
}

#[test]
fn corrupt_incompatible_and_semantically_wrong_records_preserve_recovery() {
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut session = Session::open(&path, None).unwrap();
    let initial = std::fs::read(&path).unwrap();
    session.submit(Faction::M26, 0, "0:finish").unwrap();
    let finished = std::fs::read(&path).unwrap();
    let previous = dir.path().join("task.json.previous");
    let record: serde_json::Value = serde_json::from_slice(&finished).unwrap();
    let mut cases = vec![b"not JSON".to_vec(), vec![b'x'; 256 * 1024 + 1]];
    for key in ["source", "art", "checksum"] {
        let mut changed = record.clone();
        changed[key] = "wrong identity".into();
        cases.push(serde_json::to_vec(&changed).unwrap());
    }
    // A valid checksum does not authorize a fabricated resource balance.
    let mut wrong = record.clone();
    wrong["game"]["resources"][1] = 9.into();
    let game: cuba_libre_unfolded::Game = serde_json::from_value(wrong["game"].clone()).unwrap();
    wrong["checksum"] = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&(1u32, &wrong["source"], &wrong["art"], game)).unwrap())
    )
    .into();
    cases.push(serde_json::to_vec(&wrong).unwrap());
    for bytes in cases {
        std::fs::write(&path, &bytes).unwrap();
        assert!(Session::open(&path, None).is_err());
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
    let previous = dir.path().join("task.json.previous");
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
