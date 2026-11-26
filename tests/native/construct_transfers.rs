use super::{construct::choose_row, faction_seats, *};
use std::fs;
use unfolded_record_replacement::previous_path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained transfer and Construct evidence"]
fn native_construct_transfer_resumes_at_the_giver_and_finishes_after_spending() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-220"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("construct-transfer-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let socket = tempfile::tempdir().unwrap();
    let mut app = NativeRun {
        app: Native {
            cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
            save: root.join("task.json"),
            socket: socket.path().join("seat.sock"),
            root,
            step: 0,
        },
        _socket: socket,
    };
    eprintln!("Construct transfer evidence: {}", app.root.display());
    underfunded_handoff(&mut app);
    let handoff = app.save_bytes();

    choose_row(&mut app, "syndicate:limited-operation");
    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:ask", revision);
    app.type_amount("2");
    let before = app.observe();
    assert_eq!(
        before["table"]["transfer"]["state"]["commit"],
        format!("{revision}:ask:1:2")
    );
    app.capture("syndicate-asks-26july");
    app.choose("ask:1:2");
    let asked = app.observe();
    assert_eq!(asked["observation"]["owner"], "M26");
    assert_eq!(asked["observation"]["executing"], "Syndicate");
    assert_eq!(asked["observation"]["resources"], json!([11, 4, 5, 3]));
    app.capture("giver-previews-two");
    app.choose("authorize:2");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 2, 5, 5])
    );
    app.press_by_keyboard("space:havana");
    app.press_by_keyboard("popover:commit");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 2, 5, 0])
    );
    choose_row(&mut app, "end-construct");
    app.capture("construct-paid-with-two-authorized");
    app.quit();

    app.save = app.root.join("three-resources.json");
    fs::write(&app.save, handoff).unwrap();
    app.launch_size("3840x2160");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );

    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "syndicate:limited-operation");
    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:1:2");
    let interrupted = app.observe()["table"]["newspaper"]["open"].clone();
    let report = &interrupted["report"];
    assert_eq!(report["action"], "construct");
    assert_eq!(report["faction"], "Syndicate");
    assert_eq!(report["complete"], false);
    assert!(
        report["summary"]
            .as_str()
            .unwrap()
            .contains("asks 26July for 2 Resources")
    );
    assert_eq!(report["articles"], json!([]));
    fs::write(
        app.root.join("interrupted-edition.json"),
        serde_json::to_vec_pretty(&interrupted).unwrap(),
    )
    .unwrap();
    app.capture("edition-interrupts-at-giver");
    app.key("Escape");
    app.choose("wait");
    let pending = app.observe()["observation"].clone();
    let pending_bytes = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(app.save_bytes(), pending_bytes);
    assert_eq!(app.observe()["table"]["transfer"]["state"]["pending"], true);
    app.choose("reopen");
    app.type_amount("3");
    let preview = app.observe();
    assert_eq!(
        preview["table"]["transfer"]["state"]["label"],
        "26July authorizes 3"
    );
    assert_eq!(preview["observation"]["resources"], json!([11, 4, 5, 3]));
    app.capture("giver-previews-three");
    app.choose("wait");
    let giver = app.seat_agent(faction_seats::M26, "26july");
    let (code, observed) = app.ask(&["observe", &giver]);
    assert_eq!(code, 0, "{observed}");
    assert_eq!(observed["view"]["decision"]["faction"], "M26");
    let terms = &observed["view"]["state"]["transfer_terms"][0];
    assert_eq!(terms["giver"], "M26");
    assert_eq!(terms["recipient"], "Syndicate");
    assert_eq!(terms["limit"], 4);
    app.decide(&giver, "authorize:5", "refused");
    let stale = app.revision().to_string();
    let choice = format!("{stale}:authorize:3");
    let applied = client_applies(&mut app, &giver, "authorize:3");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 1, 5, 6])
    );
    assert_eq!(app.observe()["observation"]["owner"], "Syndicate");
    let authorized_bytes = app.save_bytes();
    let (code, refused) = app.ask(&["choose", &giver, &stale, &choice]);
    assert_eq!(code, 1, "{refused}");
    assert!(
        refused["error"].as_str().unwrap().contains("Stale"),
        "{refused}"
    );
    assert_eq!(app.save_bytes(), authorized_bytes);
    fs::write(
        app.root.join("giver-reply.json"),
        serde_json::to_vec_pretty(&applied).unwrap(),
    )
    .unwrap();

    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:give", revision);
    app.type_amount("2");
    let blocked = app.observe();
    let state = &blocked["table"]["transfer"]["state"];
    assert_eq!(state["limit"], 1);
    assert_eq!(state["commit"], Value::Null);
    assert!(state["refusal"].as_str().unwrap().contains("Pay first"));
    assert_eq!(blocked["observation"]["resources"], json!([11, 1, 5, 6]));
    app.capture("outgoing-transfer-refused-until-payment");
    app.key("Escape");

    let payment = app.press_by_keyboard("space:havana");
    let plan = &payment["table"]["popover"]["card"]["rows"][0];
    assert!(
        plan["lines"]
            .to_string()
            .contains("Syndicate Resources 6 → 1.")
    );
    app.capture("havana-payment-preview");
    app.press_by_keyboard("popover:commit");
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([11, 1, 5, 1]));
    assert_eq!(paid["construct_paid"][0]["space"], "havana");
    assert_eq!(paid["construct_paid"][0]["plan"], "Open");
    assert_eq!(paid["standings"][3]["score"], 3);
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], paid);
    choose_row(&mut app, "transfer");
    app.choose("give:1:1");
    let before_finish = app.observe()["observation"].clone();
    assert_eq!(before_finish["resources"], json!([11, 2, 5, 0]));
    let previous = app.save_bytes();
    fs::write(app.root.join("before-finish.json"), &previous).unwrap();
    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "end-construct");
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    let (code, told) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{told}");
    let report = told["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["faction"], "Syndicate");
    assert_eq!(report["recorded"], "Limited Operation");
    assert_eq!(report["complete"], true);
    let articles = report["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "26July gives Syndicate 3 Resources");
    assert_eq!(articles[0]["lines"][1], "26July 4 → 1, Syndicate 3 → 6.");
    assert_eq!(articles[1]["title"], "Construct in Havana");
    assert_eq!(
        articles[1]["lines"][0],
        "Syndicate pays 5 Resources, 6 → 1."
    );
    assert_eq!(articles[2]["title"], "Syndicate gives 26July 1 Resource");
    assert_eq!(articles[2]["lines"][0], "Syndicate 1 → 0, 26July 1 → 2.");
    fs::write(
        app.root.join("faction-client-after-finish.json"),
        serde_json::to_vec_pretty(&told).unwrap(),
    )
    .unwrap();
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["eligible"],
        json!([false, true, true, false])
    );
    assert_eq!(finished["observation"]["resources"], json!([11, 2, 5, 0]));
    assert_eq!(
        finished["table"]["newspaper"]["open"]["report"]["complete"],
        true
    );
    app.capture("construct-finish-edition");
    let terminal = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], finished["observation"]);
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    app.quit();

    let source = launch_recovered(&mut app);
    assert_eq!(app.save_bytes(), previous);
    assert_eq!(app.observe()["observation"], before_finish);
    choose_row(&mut app, "end-construct");
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    assert_eq!(fs::read(source).unwrap(), terminal);
    app.quit();
}

fn client_applies(app: &mut Native, player: &str, action: &str) -> Value {
    let before = app.revision();
    let (code, reply) = app.ask(&[
        "choose",
        player,
        &before.to_string(),
        &format!("{before}:{action}"),
    ]);
    assert_eq!(code, 0, "{action}: {reply}");
    // A submission may arrive while the table still has a frame to settle.
    assert!(
        matches!(reply["status"].as_str(), Some("applied" | "held")),
        "{action}: {reply}"
    );
    assert_eq!(
        app.settle()["observation"]["revision"],
        before + 1,
        "{action}"
    );
    reply
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect pending funding and settlement"]
fn native_partial_construct_funding_survives_a_pending_reply_and_finishes_from_recovery() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-221"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("partial-funding-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let socket = tempfile::tempdir().unwrap();
    let mut app = NativeRun {
        app: Native {
            cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
            save: root.join("task.json"),
            socket: socket.path().join("seat.sock"),
            root,
            step: 0,
        },
        _socket: socket,
    };
    eprintln!("Partial Construct funding evidence: {}", app.root.display());
    underfunded_handoff(&mut app);
    choose_row(&mut app, "syndicate:limited-operation");
    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:ask", revision);
    app.type_amount("2");
    let draft = app.observe();
    let condition = "Resources move only when authorizations fund a legal payment.";
    assert!(transfer_part(&draft, "construct-funding").contains(condition));
    assert!(
        transfer_part(&draft, "construct-funding")
            .contains("0 conditionally authorized · 2 still needed")
    );
    app.capture("syndicate-conditional-request");
    app.choose("ask:1:2");
    app.type_amount("1");
    let first = app.observe();
    assert_eq!(first["observation"]["owner"], "M26");
    assert_eq!(
        first["table"]["transfer"]["state"]["label"],
        "26July authorizes 1"
    );
    assert_eq!(
        transfer_part(&first, "construct-funding"),
        transfer_part(&draft, "construct-funding")
    );
    app.capture("26july-previews-one");
    app.choose("authorize:1");
    let held = app.observe();
    assert_eq!(held["observation"]["resources"], json!([11, 4, 5, 3]));
    assert_eq!(held["observation"]["owner"], "Syndicate");
    assert_eq!(
        held["table"]["notice"],
        "26July conditionally authorized 1 Resource for Syndicate. Syndicate still needs 1 Resource. Saved."
    );
    assert_eq!(
        held["observation"]["construct_funding"],
        json!({
            "authorizations": [{"giver": "M26", "asked": 2, "amount": 1}],
            "shortfall": 1
        })
    );
    assert!(
        held["table"]["choice_parts"]
            .to_string()
            .contains("1 conditionally authorized · 1 still needed")
    );
    app.capture("unspent-authorization");

    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    let (code, seen) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{seen}");
    assert_eq!(
        seen["view"]["state"]["construct_funding"],
        held["observation"]["construct_funding"]
    );
    assert!(
        seen["view"]["results"].as_array().unwrap().last().unwrap()["articles"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:1:3");
    let refusing = app.observe();
    assert_eq!(refusing["observation"]["transfer_terms"][0]["limit"], 3);
    app.key("Escape"); // Close the interrupted edition and answer as 26July.
    app.choose("decline");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );
    app.key("Escape");

    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:2:5");
    let interrupted = app.observe()["table"]["newspaper"]["open"].clone();
    assert_eq!(interrupted["report"]["faction"], "Syndicate");
    let summary = interrupted["report"]["summary"].as_str().unwrap();
    assert!(summary.contains("Directorio decides"));
    assert!(summary.contains("1 conditionally authorized · 1 still needed"));
    assert!(summary.contains(condition));
    assert!(
        !interrupted["report"]["articles"]
            .to_string()
            .contains("gives Syndicate 1 Resource")
    );
    app.capture("edition-pending-directorio");
    app.key("Escape");
    app.choose("wait");
    let pending = app.observe();
    assert_eq!(pending["observation"]["owner"], "Directorio");
    assert_eq!(
        transfer_part(&pending, "pending-construct-funding"),
        "1 authorized · need 1"
    );
    assert_eq!(
        transfer_part(&pending, "pending-construct-giver"),
        "Directorio decides · 5 requested"
    );
    let bounds = |name| {
        let part = drawn_transfer_part(&pending, name);
        let limits = part["bounds"].as_array().unwrap();
        std::array::from_fn::<_, 4, _>(|i| limits[i].as_f64().unwrap())
    };
    let [left, top, right, bottom] = bounds("pending-tab");
    for name in ["pending-construct-funding", "pending-construct-giver"] {
        let [l, t, r, b] = bounds(name);
        assert!(l > left && t > top && r < right && b < bottom, "{name}");
    }
    app.capture("wait-on-directorio");
    let pending_bytes = app.save_bytes();
    fs::write(app.root.join("pending-before-restart.json"), &pending_bytes).unwrap();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.save_bytes(), pending_bytes);
    assert_eq!(app.observe()["observation"], pending["observation"]);
    let directorio = app.seat_agent(faction_seats::DIRECTORIO, "directorio");
    let (code, seen) = app.ask(&["observe", &directorio]);
    assert_eq!(code, 0, "{seen}");
    let decision = &seen["view"]["decision"];
    assert_eq!(decision["faction"], "Directorio");
    assert_eq!(decision["yours"], true);
    assert!(decision["note"].as_str().unwrap().contains(condition));
    assert_eq!(
        seen["view"]["state"]["construct_funding"],
        held["observation"]["construct_funding"]
    );
    let revision = app.revision().to_string();
    let previous_choice = format!("{revision}:authorize:2");
    let reply = client_applies(&mut app, &directorio, "authorize:2");
    fs::write(
        app.root.join("directorio-client-settlement.json"),
        serde_json::to_vec_pretty(&reply).unwrap(),
    )
    .unwrap();
    let settled = app.observe();
    assert_eq!(settled["observation"]["resources"], json!([11, 3, 3, 6]));
    assert_eq!(
        settled["table"]["notice"],
        "Directorio authorized 2 Resources. Syndicate received 3 Resources from the combined authorizations. Saved."
    );
    assert_eq!(settled["observation"]["owner"], "Syndicate");
    assert_eq!(settled["observation"]["construct_funding"], Value::Null);
    assert_eq!(settled["observation"]["construct_paid"], json!([]));
    let settled_bytes = app.save_bytes();
    let (code, stale) = app.ask(&["choose", &directorio, &revision, &previous_choice]);
    assert_eq!(code, 1, "{stale}");
    assert!(stale["error"].as_str().unwrap().contains("Stale"));
    app.decide(&directorio, "authorize:2", "refused");
    assert_eq!(app.save_bytes(), settled_bytes);
    app.key("Escape");

    let payment = app.press_by_keyboard("space:havana");
    assert!(
        payment["table"]["popover"]["card"]["rows"][0]["lines"]
            .to_string()
            .contains("Syndicate Resources 6 → 1.")
    );
    app.capture("settled-funding-havana-preview");
    app.press_by_keyboard("popover:commit");
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([11, 3, 3, 1]));
    assert_eq!(paid["construct_paid"][0]["space"], "havana");
    assert_eq!(paid["construct_paid"][0]["plan"], "Open");
    let paid_bytes = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], paid);
    assert_eq!(app.save_bytes(), paid_bytes);
    choose_row(&mut app, "transfer");
    app.choose("give:1:1");
    let before_finish = app.observe()["observation"].clone();
    assert_eq!(before_finish["resources"], json!([11, 4, 3, 0]));
    let previous = app.save_bytes();
    fs::write(app.root.join("before-finish.json"), &previous).unwrap();
    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "end-construct");
    let (code, told) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{told}");
    let report = told["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["recorded"], "Limited Operation");
    let articles = report["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "26July gives Syndicate nothing");
    assert_eq!(articles[1]["title"], "26July gives Syndicate 1 Resource");
    assert_eq!(articles[1]["lines"][1], "26July 4 → 3, Syndicate 3 → 4.");
    assert_eq!(
        articles[2]["title"],
        "Directorio gives Syndicate 2 Resources"
    );
    assert_eq!(
        articles[2]["lines"][1],
        "Directorio 5 → 3, Syndicate 4 → 6."
    );
    assert_eq!(articles[3]["title"], "Construct in Havana");
    assert_eq!(
        articles[3]["lines"][0],
        "Syndicate pays 5 Resources, 6 → 1."
    );
    assert_eq!(articles[4]["title"], "Syndicate gives 26July 1 Resource");
    fs::write(
        app.root.join("syndicate-client-after-finish.json"),
        serde_json::to_vec_pretty(&told).unwrap(),
    )
    .unwrap();
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["eligible"],
        json!([false, true, true, false])
    );
    assert_eq!(finished["observation"]["resources"], json!([11, 4, 3, 0]));
    assert_eq!(
        finished["table"]["newspaper"]["open"]["report"]["complete"],
        true
    );
    app.capture("partial-funding-finish-edition");
    let terminal = app.save_bytes();
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], finished["observation"]);
    assert_eq!(app.save_bytes(), terminal);
    app.quit();
    let source = launch_recovered(&mut app);
    assert_eq!(app.save_bytes(), previous);
    assert_eq!(app.observe()["observation"], before_finish);
    choose_row(&mut app, "end-construct");
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    assert_eq!(fs::read(previous_path(&source)).unwrap(), previous);
    assert_eq!(fs::read(source).unwrap(), terminal);
    app.quit();
}

fn transfer_part<'a>(seen: &'a Value, name: &str) -> &'a str {
    drawn_transfer_part(seen, name)["text"].as_str().unwrap()
}

fn drawn_transfer_part<'a>(seen: &'a Value, name: &str) -> &'a Value {
    seen["table"]["transfer"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["part"] == name)
        .unwrap_or_else(|| panic!("{name} not drawn"))
}

fn underfunded_handoff(app: &mut Native) {
    super::assault::fund_santiago(&app.save, 0);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(app, action);
    }
    app.choose("advance-card");
    choose_row(app, "government:assault");
    choose_row(app, "transfer");
    app.choose("ask:3:12");
    app.choose("authorize:12");
    app.choose("assault:havana:syndicate");
    choose_row(app, "end-assault");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );
}

fn launch_recovered(app: &mut Native) -> PathBuf {
    let source = app.save.clone();
    app.save = app.root.join("recovered.json");
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--save",
            app.save.to_str().unwrap(),
            "--recover-from",
            source.to_str().unwrap(),
            "--agent-seat",
            "--seat-size",
            "3840x2160",
            "--socket",
            app.socket.to_str().unwrap(),
            "--player-socket",
            app.player_socket().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(app.root.join("recovery-launch.jsonl"), output.stdout).unwrap();
    source
}
