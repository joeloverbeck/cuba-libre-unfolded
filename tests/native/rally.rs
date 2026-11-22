use super::*;

fn rally_app() -> (tempfile::TempDir, Native) {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-120"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("free-rally-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket = tempfile::tempdir().unwrap();
    let app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        save: root.join("task.json"),
        socket: socket.path().join("seat.sock"),
        root,
        step: 0,
    };
    eprintln!("Free Rally evidence: {}", app.root.display());
    // The client must drop first so it can still quit the detached seat on failure.
    (socket, app)
}

fn ready_for_event(app: &mut Native, retain_base: bool) {
    app.launch_size("3840x2160");
    app.choose("terror");
    if !retain_base {
        app.choose("remove");
        app.remove("sierra-maestra:base", 1);
        app.choose("done-removal");
    }
    app.choose("kidnap");
    app.roll();
    app.choose("terror:santiago");
    app.choose("end-terror");
}

fn choose_rally_option(app: &mut Native, action: &str) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("option:{action}"), revision);
    app.choose(action);
}

fn rally_space(view: &Value) -> &Value {
    view["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|space| space["id"] == "sierra-maestra")
        .unwrap()
}

fn available_guerrillas(view: &Value) -> &Value {
    &view["available"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["faction"] == "M26" && group["kind"] == "Guerrilla")
        .unwrap()["count"]
}

fn assert_unshaded_card_end(before: &Value, table: &Value) {
    let view = &table["observation"];
    assert_eq!(
        view["eligibility"]["next_card"],
        json!([
            {"faction":"Government","eligible":true,"reason":"NoOperationOrEvent"},
            {"faction":"M26","eligible":false,"reason":"ExecutedOperation"},
            {"faction":"Directorio","eligible":false,"reason":"ExecutedEvent"},
            {"faction":"Syndicate","eligible":true,"reason":"NoOperationOrEvent"}
        ])
    );
    assert!(view["eligibility"]["event_restriction"].is_null());
    assert!(view["choices"].as_array().unwrap().is_empty());
    for key in [
        "draw_count",
        "played_cards",
        "preview_card",
        "current_card",
        "kidnap",
        "resources",
    ] {
        assert_eq!(view[key], before[key], "{key}");
    }
    assert_eq!(view["current_card"], 17);
    assert_eq!(view["preview_card"], 18);
    let track = &table["table"]["eligibility"]["track"];
    assert_eq!(track["adjusted"], true);
    assert_eq!(track["ineligible"], json!(["M26", "Directorio"]));
    let result = &table["table"]["choice_parts"].as_array().unwrap();
    let reasons = result
        .iter()
        .find(|part| part["part"] == "ended-value:ELIGIBILITY FOR CARD 18 · R 2.3.7")
        .unwrap()["text"]
        .as_str()
        .unwrap();
    for reason in [
        "Government · Eligible · took no Operation or Event",
        "26July · Ineligible · executed Terror",
        "Directorio · Ineligible · executed the Event",
        "Syndicate · Eligible · took no Operation or Event",
    ] {
        assert!(reasons.contains(reason), "{reasons}");
    }
    let rally = result
        .iter()
        .find(|part| part["part"] == "ended-value:UNSHADED EVENT · FREE RALLY")
        .unwrap()["text"]
        .as_str()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(rally.contains("0 Resources"), "{rally}");
    assert!(
        rally.contains("free Rally adds no eligibility penalty"),
        "{rally}"
    );
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Rally captures"]
fn native_free_rally_hands_the_decision_to_the_beneficiary() {
    let (_socket, mut app) = rally_app();
    let historical = std::env::var_os("CUBA_TEST_RALLY_SAVE").map(PathBuf::from);
    if let Some(historical) = &historical {
        std::fs::copy(historical, &app.save).unwrap();
        app.launch_size("3840x2160");
    } else {
        ready_for_event(&mut app, true);
    }
    let handoff = app.observe()["observation"].clone();
    assert_eq!(handoff["owner"], "Directorio");
    if historical.is_some() {
        assert_eq!(handoff["revision"], 9);
        assert_eq!(handoff["kidnap"]["die"], 5);
        assert_eq!(handoff["resources"], json!([1, 5, 3, 15]));
    }
    let option = handoff["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str().unwrap().ends_with(":event:unshaded"))
        .unwrap();
    if option["supported"] != true {
        app.quit();
        panic!("Directorio must be able to choose unshaded: {option}");
    }
    choose_rally_option(&mut app, "event:unshaded");
    let choose_beneficiary = app.observe()["observation"].clone();
    assert_eq!(choose_beneficiary["owner"], "Directorio");
    for (beneficiary, reason) in [
        ("26july", "Sierra Maestra"),
        ("directorio", "No qualifying Base spaces"),
    ] {
        let shown = app.point(&format!("option:rally:beneficiary:{beneficiary}"));
        assert!(tag_text(&shown["table"]["briefing"]).contains(reason));
        assert_eq!(shown["observation"], choose_beneficiary);
    }
    app.capture("02-directorio-chooses-beneficiary");
    choose_rally_option(&mut app, "rally:beneficiary:26july");
    let pending = app.observe()["observation"].clone();
    assert_eq!(pending["owner"], "M26");
    assert_eq!(pending["executing"], "M26");
    for field in ["spaces", "available", "resources", "kidnap"] {
        assert_eq!(pending[field], handoff[field], "{field}");
    }
    assert_eq!(available_guerrillas(&pending), 12);
    let target = app.point("space:sierra-maestra");
    assert_eq!(target["table"]["status"]["control"], "26July");
    let revision = pending["revision"].as_u64().unwrap();
    let options = pending["choices"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|choice| {
            choice["id"]
                .as_str()
                .unwrap()
                .split_once(':')
                .map(|(_, action)| action)
        })
        .filter(|action| action.starts_with("rally:sierra-maestra:"))
        .collect::<Vec<_>>();
    assert_eq!(
        options,
        [
            "rally:sierra-maestra:place:1",
            "rally:sierra-maestra:place:2",
            "rally:sierra-maestra:place:3",
            "rally:sierra-maestra:place:4"
        ]
    );
    let maximum = app.point("option:rally:sierra-maestra:place:4");
    let preview = tag_text(&maximum["table"]["briefing"]);
    for text in [
        "2 × 1 Base + 2 × 1 Population = 4",
        "12 Available Guerrillas",
        "up to 4",
        "0 Resources",
        "Active Opposition",
    ] {
        assert!(preview.contains(text), "{preview}");
    }
    app.capture("03-26july-rally-maximum");
    app.request(json!({"op":"activate","revision":revision,"choice":format!("{revision}:rally:sierra-maestra:place:5")}), false);
    assert_eq!(app.observe()["observation"], pending);

    let pending_bytes = std::fs::read(&app.save).unwrap();
    std::fs::write(app.root.join("pending-rally.json"), &pending_bytes).unwrap();
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(std::fs::read(&app.save).unwrap(), pending_bytes);
    app.request(json!({"op":"activate","revision":revision - 1,"choice":format!("{}:rally:beneficiary:26july", revision - 1)}), false);
    assert_eq!(app.observe()["observation"], pending);

    let place_three = "rally:sierra-maestra:place:3";
    let shown = app.point(&format!("option:{place_three}"));
    assert!(tag_text(&shown["table"]["briefing"]).contains("leaves 9 Guerrillas"));
    app.capture("04-rally-after-restart");
    choose_rally_option(&mut app, place_three);
    let complete = app.observe();
    let final_position = &complete["observation"];
    assert_unshaded_card_end(&handoff, &complete);
    assert_eq!(
        rally_space(final_position)["pieces"],
        json!([
            {"faction":"M26","kind":"Base","state":"Ordinary","count":1},
            {"faction":"M26","kind":"Guerrilla","state":"Underground","count":3}
        ])
    );
    assert_eq!(rally_space(final_position)["support"], -2);
    assert_eq!(available_guerrillas(final_position), 9);
    assert_eq!(final_position["standings"], handoff["standings"]);
    if historical.is_some() {
        assert_eq!(
            final_position["standings"]
                .as_array()
                .unwrap()
                .iter()
                .map(|standing| standing["score"].as_i64().unwrap())
                .collect::<Vec<_>>(),
            [16, 10, 1, 3]
        );
    }
    app.capture("05-unshaded-card-end");
    let target = app.point("space:sierra-maestra");
    assert_eq!(target["table"]["status"]["control"], "26July");
    let final_bytes = std::fs::read(&app.save).unwrap();
    let before_record: Value = serde_json::from_slice(&pending_bytes).unwrap();
    let final_record: Value = serde_json::from_slice(&final_bytes).unwrap();
    assert_eq!(
        final_record["payload"]["random"],
        before_record["payload"]["random"]
    );
    for submitted_revision in [revision, revision + 1] {
        app.request(json!({"op":"activate","revision":submitted_revision,"choice":format!("{submitted_revision}:{place_three}")}), false);
        assert_eq!(app.observe()["observation"], *final_position);
        assert_eq!(std::fs::read(&app.save).unwrap(), final_bytes);
    }
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], *final_position);
    assert_eq!(std::fs::read(&app.save).unwrap(), final_bytes);
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained empty Rally captures"]
fn native_free_rally_without_a_base_finishes_the_event() {
    for (beneficiary, retain_base) in [("26july", false), ("directorio", true)] {
        let (_socket, mut app) = rally_app();
        ready_for_event(&mut app, retain_base);
        let handoff = app.observe()["observation"].clone();
        assert_eq!(
            rally_space(&handoff)["pieces"].as_array().unwrap().len(),
            usize::from(retain_base)
        );
        choose_rally_option(&mut app, "event:unshaded");
        let action = format!("rally:beneficiary:{beneficiary}");
        let preview = app.point(&format!("option:{action}"));
        let reason = tag_text(&preview["table"]["briefing"]);
        assert!(reason.contains("No qualifying Base spaces"), "{reason}");
        app.capture(&format!("no-base-{beneficiary}-preview"));
        choose_rally_option(&mut app, &action);
        let complete = app.observe();
        assert_unshaded_card_end(&handoff, &complete);
        for field in ["spaces", "available", "standings"] {
            assert_eq!(complete["observation"][field], handoff[field], "{field}");
        }
        assert_eq!(
            complete["observation"]["event"]["Complete"]["Unshaded"]["resolutions"],
            json!([])
        );
        app.capture(&format!("no-base-{beneficiary}-card-end"));
        app.quit();
    }
}
