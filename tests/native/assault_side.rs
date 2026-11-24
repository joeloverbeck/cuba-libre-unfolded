use super::*;
use cuba_libre_unfolded::{Faction, save::Session};
use std::fs;
use unfolded_record_replacement::previous_path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained transfer and removal frames"]
fn native_assault_resumes_giver_and_government_side_decisions() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-132"));
    fs::create_dir_all(&evidence).unwrap();
    let socket = tempfile::tempdir().unwrap();
    let root = tempfile::Builder::new()
        .prefix("assault-sides-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        save: root.join("task.json"),
        socket: socket.path().join("seat.sock"),
        root,
        step: 0,
    };
    eprintln!("Assault side decision evidence: {}", app.root.display());
    super::assault::fund_santiago(&app.save, 1);
    let initial = Session::open(&app.save, None).unwrap().observe();
    let m26 = initial.resources[1];
    let directorio = initial.resources[2];
    assert_eq!(initial.resources, [2, m26, directorio, 15]);

    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(&mut app, action);
    }
    app.choose("advance-card");
    choose_row(&mut app, "government:assault");
    pin(&mut app, "transfer");
    let briefing = app.observe()["table"]["briefing"].clone();
    assert_eq!(briefing["rule"], "R 1.5.1, 1.8");
    assert!(tag_text(&briefing).contains("GIVER'S DECISION"));
    app.capture("government-transfer-briefing");
    app.choose("transfer");
    let composer = app.observe();
    assert!(
        composer["table"]["inline_text"]
            .to_string()
            .contains("executes Assault"),
        "the transfer must name Government's current Operation"
    );
    let parts = composer["table"]["transfer"]["parts"].as_array().unwrap();
    let limits = parts
        .iter()
        .find(|part| part["part"] == "limit:balance")
        .unwrap();
    assert_eq!(
        limits["text"],
        "Limited by the giver’s balance and the recipient’s 49-Resource cap"
    );
    app.capture("government-transfer-compose");
    app.choose("give:3:2");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([0, m26, directorio, 17])
    );
    let after_give = app.observe()["observation"].clone();
    let (gave, before_giving) = records(&app);
    choose_row(&mut app, "transfer");
    app.choose("ask:1:1");
    let authorized_before = app.observe()["observation"].clone();
    assert_eq!(authorized_before["owner"], "M26");
    assert_eq!(authorized_before["executing"], "Government");
    assert_eq!(
        authorized_before["resources"],
        json!([0, m26, directorio, 17])
    );
    app.choose("wait");
    let waiting = app.observe()["observation"].clone();
    assert_eq!(waiting["transfer"]["Authorizing"]["waited"], true);
    restart_pending(&mut app, &waiting, Faction::Government, "authorize:3");
    app.choose("reopen");
    let shown = app.observe();
    assert_eq!(shown["table"]["transfer"]["state"]["shown"], true);
    let limits = shown["table"]["transfer"]["parts"].as_array().unwrap();
    assert!(limits.iter().any(|part| part["part"] == "limit:cap"));
    assert!(limits.iter().all(|part| part["part"] != "limit:reserve"));
    app.capture("26july-authorizes-government-request");
    let giver = app.observe()["observation"].clone();
    app.choose("authorize:3");
    let receipt = app.observe()["observation"].clone();
    assert_eq!(receipt["owner"], "Government");
    assert_eq!(receipt["resources"], json!([3, m26 - 3, directorio, 17]));
    assert_eq!(receipt["transfer"], Value::Null);
    refuse_repeat(&mut app, &giver, &receipt, "authorize:3");

    pin(&mut app, "remove");
    let briefing = app.observe()["table"]["briefing"].clone();
    assert_eq!(briefing["rule"], "R 1.4.1");
    assert!(tag_text(&briefing).contains("VOLUNTARY REMOVAL"));
    app.capture("government-removal-briefing");
    app.choose("remove");
    let removal = app.observe()["observation"].clone();
    assert_eq!(removal["owner"], "Government");
    let action = "remove:santiago:troops:2";
    restart_pending(&mut app, &removal, Faction::M26, action);
    let revision = removal["revision"].as_u64().unwrap();
    app.activate("piece:santiago Government Troops Ordinary", revision);
    app.activate("popover:quantity:plus", revision);
    let preview = app.observe();
    assert_eq!(
        preview["table"]["popover"]["card"]["rows"][0]["button"],
        "Remove 2 Troops"
    );
    let explanation = popover_text(&preview);
    assert!(
        explanation.contains("2 Troops return to Available."),
        "{explanation}"
    );
    assert!(explanation.contains("Government → 26July"), "{explanation}");
    app.capture("santiago-remove-two-troops");
    app.activate("popover:commit", revision);
    app.request(json!({"op":"settle"}), true);
    let committed = app.observe()["observation"].clone();
    assert_eq!(committed["resources"], receipt["resources"]);
    assert_eq!(committed["standings"], removal["standings"]);
    assert_eq!(
        committed["available_piece_count"],
        removal["available_piece_count"].as_u64().unwrap() + 2
    );
    assert_eq!(
        committed["map_piece_count"],
        removal["map_piece_count"].as_u64().unwrap() - 2
    );
    refuse_repeat(&mut app, &removal, &committed, action);

    choose_row(&mut app, "done-removal");
    // Santiago's popover on the map shows the capacity the removal left (#169).
    let payment = "assault:santiago:26july";
    let at = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", at);
    let contents = popover_text(&app.observe());
    assert!(contents.contains("0 Troops + 1 Police = 1"), "{contents}");
    assert!(
        contents.contains("Pay 2 Resources: Government 3 → 1."),
        "{contents}"
    );
    app.capture("santiago-recalculated-payment");
    app.choose(payment);
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([1, m26 - 3, directorio, 17]));
    assert_eq!(
        paid["available_piece_count"],
        committed["available_piece_count"].as_u64().unwrap() + 1
    );
    choose_row(&mut app, "end-assault");
    let handoff = app.observe()["observation"].clone();
    assert_eq!(handoff["next"]["role"], "2nd Eligible");
    assert_eq!(
        handoff["next"]["options"],
        json!(["Pass", "Limited Operation"])
    );
    assert_eq!(handoff["resources"], paid["resources"]);
    pin(&mut app, "syndicate:limited-operation");
    app.capture("syndicate-second-decision");
    app.quit();

    // Resume a copy taken after Government gave away all its Resources.
    app.save = app.root.join("zero-space.json");
    fs::write(&app.save, gave).unwrap();
    fs::write(previous_path(&app.save), before_giving).unwrap();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], after_give);
    choose_row(&mut app, "end-assault");
    let zero_space = app.observe()["observation"].clone();
    assert_eq!(zero_space["resources"], json!([0, m26, directorio, 17]));
    assert_eq!(zero_space["spaces"], after_give["spaces"]);
    assert_eq!(zero_space["available"], after_give["available"]);
    assert_eq!(zero_space["assault_paid"], json!([]));
    assert_eq!(zero_space["owner"], "Syndicate");
    assert_eq!(zero_space["next"]["role"], "2nd Eligible");
    assert_eq!(zero_space["eligibility"]["government_box"], "OperationOnly");
    app.capture("zero-space-after-give");
    app.quit();
}

fn pin(app: &mut Native, action: &str) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("option:{action}"), revision);
}

fn choose_row(app: &mut Native, action: &str) {
    pin(app, action);
    app.choose(action);
}

fn records(app: &Native) -> (Vec<u8>, Vec<u8>) {
    (
        fs::read(&app.save).unwrap(),
        fs::read(previous_path(&app.save)).unwrap(),
    )
}

fn restart_pending(app: &mut Native, pending: &Value, wrong_owner: Faction, action: &str) {
    let bytes = records(app);
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], *pending);
    // The hot-seat socket reaches player controls. A caller with an explicit
    // faction reaches the same durable submit authority directly.
    let revision = pending["revision"].as_u64().unwrap();
    let mut session = Session::open(&app.save, None).unwrap();
    let error = session
        .submit(wrong_owner, revision, &format!("{revision}:{action}"))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("Only the current decision owner"),
        "{error}"
    );
    assert_eq!(records(app), bytes);
}

fn refuse_repeat(app: &mut Native, before: &Value, after: &Value, action: &str) {
    let bytes = records(app);
    for at in [before, after] {
        let revision = at["revision"].as_u64().unwrap();
        app.request(
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:{action}")}),
            false,
        );
    }
    assert_eq!(app.observe()["observation"], *after);
    assert_eq!(records(app), bytes);
}
