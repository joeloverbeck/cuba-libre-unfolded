use cuba_libre_unfolded::{Faction, save::Session};
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
