use super::*;
use cuba_libre_unfolded::{Faction, assets, save::Session};
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
    let funded = std::env::var_os("CUBA_TEST_SANTIAGO_FUNDED_SAVE");
    if let Some(source) = funded.as_deref().map(Path::new) {
        let bytes = fs::read(source).unwrap();
        assert_eq!(
            assets::digest(&bytes),
            "95d929facb49d629ce582498c4cc9f9672560b6d764379f6eb4a7c2c368cd860"
        );
        fs::write(&app.save, bytes).unwrap();
        fs::copy(previous_path(source), previous_path(&app.save)).unwrap();
    } else {
        super::assault::fund_santiago(&app.save);
    }
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
    choose_row(&mut app, "transfer");
    let composer = app.observe();
    assert!(
        composer["table"]["inline_text"]
            .to_string()
            .contains("executes Assault"),
        "the transfer must name Government's current Operation"
    );
    app.capture("government-transfer-compose");
    app.choose("give:3:1");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([1, m26, directorio, 16])
    );
    choose_row(&mut app, "transfer");
    app.choose("ask:1:1");
    let authorized_before = app.observe()["observation"].clone();
    assert_eq!(authorized_before["owner"], "M26");
    assert_eq!(authorized_before["executing"], "Government");
    assert_eq!(
        authorized_before["resources"],
        json!([1, m26, directorio, 16])
    );
    app.choose("wait");
    let waiting = app.observe()["observation"].clone();
    assert_eq!(waiting["transfer"]["Authorizing"]["waited"], true);
    restart_pending(&mut app, &waiting, Faction::Government, "authorize:2");
    app.choose("reopen");
    assert_eq!(app.observe()["table"]["transfer"]["state"]["shown"], true);
    app.capture("26july-authorizes-government-request");
    let giver = app.observe()["observation"].clone();
    app.choose("authorize:2");
    let receipt = app.observe()["observation"].clone();
    assert_eq!(receipt["owner"], "Government");
    assert_eq!(receipt["resources"], json!([3, m26 - 2, directorio, 16]));
    assert_eq!(receipt["transfer"], Value::Null);
    refuse_repeat(&mut app, &giver, &receipt, "authorize:2");

    choose_row(&mut app, "remove");
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
    if app.observe()["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|choice| {
            choice["id"]
                .as_str()
                .unwrap()
                .ends_with(":assault:areas:cities")
        })
    {
        choose_row(&mut app, "assault:areas:cities");
        choose_row(&mut app, "assault:space:santiago");
    }
    let payment = "assault:santiago:26july";
    pin(&mut app, payment);
    let contents = tag_text(&app.observe()["table"]["briefing"]);
    assert!(contents.contains("0 Troops + 1 Police = 1"), "{contents}");
    assert!(contents.contains("2 Resources (3 → 1)"), "{contents}");
    app.capture("santiago-recalculated-payment");
    app.choose(payment);
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([1, m26 - 2, directorio, 16]));
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
