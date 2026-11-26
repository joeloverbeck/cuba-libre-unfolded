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
    super::assault::fund_santiago(&app.save, 0);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(&mut app, action);
    }
    app.choose("advance-card");
    choose_row(&mut app, "government:assault");
    choose_row(&mut app, "transfer");
    app.choose("ask:3:12");
    app.choose("authorize:12");
    app.choose("assault:havana:syndicate");
    choose_row(&mut app, "end-assault");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );
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
    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate-finish");
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
