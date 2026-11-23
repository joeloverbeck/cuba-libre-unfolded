use super::*;

fn rally_app() -> NativeRun {
    event_app("free-rally-", "issue-120")
}

fn event_app(prefix: &str, issue: &str) -> NativeRun {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification").join(issue));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix(prefix)
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
    eprintln!("Alberto Bayo evidence: {}", app.root.display());
    NativeRun {
        app,
        _socket: socket,
    }
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

fn open_event_handoff(app: &mut Native) {
    if let Some(input) = std::env::var_os("CUBA_TEST_COMPLETED_KIDNAP") {
        std::fs::copy(input, &app.save).unwrap();
        app.launch_size("3840x2160");
    } else {
        app.launch_size("3840x2160");
        for action in [
            "terror",
            "transfer",
            "ask:2:2",
            "authorize:2",
            "terror:santiago",
            "terror:oriente",
            "kidnap",
        ] {
            app.choose(action);
        }
        app.roll();
        app.choose("end-terror");
    }
}

fn has_action(position: &Value, action: &str) -> bool {
    position["choices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|choice| {
            choice["supported"] == true
                && choice["id"].as_str().unwrap().split_once(':').unwrap().1 == action
        })
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_shaded_panel_previews_multiple_spaces_and_no_flips() {
    for remove_underground in [false, true] {
        let mut app = event_app("shaded-preview-", "issue-117");
        app.launch_size("3840x2160");
        app.choose("terror");
        app.choose("kidnap");
        app.roll();
        app.choose("terror:santiago");
        if remove_underground {
            app.choose("remove");
            app.remove("santiago:underground", 1);
            app.remove("oriente:underground", 1);
            app.choose("done-removal");
        }
        app.choose("end-terror");
        choose_rally_option(&mut app, "event:shaded");
        let shown = app.observe();
        let subtitle = choice_part(&shown, "choice-subtitle");
        let text = subtitle["text"].as_str().unwrap();
        if remove_underground {
            assert!(
                text.contains("No Underground 26July Guerrillas remain."),
                "{text}"
            );
        } else {
            for space in ["Santiago de Cuba", "Oriente"] {
                assert!(
                    text.contains(&format!("{space} · 1 Underground Guerrilla → Active")),
                    "{text}"
                );
            }
        }
        let resolve = choice_part(&shown, "option:resolve-shaded");
        assert!(subtitle["bounds"][3].as_f64().unwrap() <= resolve["bounds"][1].as_f64().unwrap());
        app.capture("pending");
        choose_rally_option(&mut app, "resolve-shaded");
        let ended = app.observe();
        let flips = ended["observation"]["event"]["Complete"]["Shaded"]["flips"]
            .as_array()
            .unwrap();
        assert_eq!(flips.len(), if remove_underground { 0 } else { 2 });
        assert_eq!(
            ended["observation"]["eligibility"]["event_restriction"],
            json!({"faction": "M26", "through_card": 18})
        );
        app.quit();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained side decision captures"]
fn native_shaded_execution_preserves_directorios_side_decisions() {
    let mut app = event_app("shaded-side-decisions-", "issue-121");
    open_event_handoff(&mut app);
    let before = app.observe()["observation"].clone();
    choose_rally_option(&mut app, "event:shaded");
    let shaded = app.observe()["observation"].clone();
    assert_eq!(shaded["executing"], "Directorio");
    assert_eq!(shaded["event_executor"], "Directorio");
    assert_eq!(shaded["owner"], "Directorio");
    assert_eq!(shaded["spaces"], before["spaces"]);
    assert_eq!(shaded["resources"], before["resources"]);
    assert!(shaded["eligibility"]["next_card"].is_null());
    app.capture("directorio-execution");

    choose_rally_option(&mut app, "transfer");
    assert_eq!(app.observe()["observation"]["transfer"], "Selecting");
    app.choose("ask:3:2");
    let asked = app.observe();
    assert_eq!(
        asked["table"]["notice"],
        "Transfer request saved: Directorio asks Syndicate for 2. Nothing moves until Syndicate decides."
    );
    let pending = asked["observation"].clone();
    assert_eq!(pending["owner"], "Syndicate");
    assert_eq!(pending["executing"], "Directorio");
    assert_eq!(pending["resources"], before["resources"]);
    app.choose("wait");
    let waiting = app.observe()["observation"].clone();
    let waiting_save = std::fs::read(&app.save).unwrap();
    app.capture("shaded-request-waiting");
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], waiting);
    assert_eq!(std::fs::read(&app.save).unwrap(), waiting_save);
    let at = waiting["revision"].as_u64().unwrap();
    app.request(json!({"op":"key","key":"Tab","revision":at}), true);
    app.request(json!({"op":"settle"}), true);
    app.request(
        json!({"op":"key","key":"Enter","revision":at,"focused":format!("{at}:reopen")}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    let reopened = app.observe();
    assert_eq!(reopened["observation"]["revision"], at + 1);
    assert_eq!(reopened["observation"]["resources"], waiting["resources"]);
    assert_eq!(reopened["observation"]["owner"], "Syndicate");
    assert_eq!(reopened["table"]["transfer"]["state"]["shown"], true);
    app.capture("syndicate-authorizes-directorio-request");
    app.choose("authorize:2");
    let funded = app.observe()["observation"].clone();
    let mut balances = before["resources"].clone();
    balances[2] = (balances[2].as_u64().unwrap() + 2).into();
    balances[3] = (balances[3].as_u64().unwrap() - 2).into();
    assert_eq!(funded["resources"], balances);
    assert_eq!(funded["owner"], "Directorio");
    assert!(has_action(&funded, "resolve-shaded"));
    let funded_bytes = std::fs::read(&app.save).unwrap();
    let given_at = funded["revision"].as_u64().unwrap() - 1;
    app.request(
        json!({"op":"activate","revision":given_at,"choice":format!("{given_at}:authorize:2")}),
        false,
    );
    assert_eq!(app.observe()["observation"], funded);
    assert_eq!(std::fs::read(&app.save).unwrap(), funded_bytes);

    choose_rally_option(&mut app, "remove");
    let removal = app.observe();
    assert_eq!(
        removal["table"]["notice"],
        "Removal mode: click a ringed Directorio piece on the map. Saved."
    );
    assert!(has_action(
        &removal["observation"],
        "remove:camaguey-province:underground:1"
    ));
    assert!(!has_action(
        &removal["observation"],
        "remove:sierra-maestra:base:1"
    ));
    app.capture("directorio-removal");
    let revision = removal["observation"]["revision"].as_u64().unwrap();
    app.activate(
        "piece:camaguey-province Directorio Guerrilla Underground",
        revision,
    );
    let card = app.observe()["table"]["popover"]["card"].clone();
    assert_eq!(card["title"], "Directorio Guerrilla · Underground");
    assert_eq!(
        card["rows"][0]["choice"],
        format!("{revision}:remove:camaguey-province:underground:1")
    );
    app.capture("directorio-removal-preview");
    app.activate("popover:commit", revision);
    let removed = app.point("space:camaguey-province");
    assert_eq!(removed["table"]["status"]["control"], "Uncontrolled");
    assert_eq!(removed["observation"]["standings"][2]["score"], 0);
    assert_eq!(
        removed["observation"]["available_piece_count"].as_u64(),
        before["available_piece_count"]
            .as_u64()
            .map(|count| count + 1)
    );
    choose_rally_option(&mut app, "done-removal");
    choose_rally_option(&mut app, "resolve-shaded");
    let completed = app.observe()["observation"].clone();
    assert_eq!(completed["resources"], balances);
    assert_eq!(completed["kidnap"], before["kidnap"]);
    assert_eq!(completed["eligible"], json!([true, false, false, true]));
    assert_eq!(
        completed["eligibility"]["event_restriction"],
        json!({"faction":"M26","through_card":18})
    );
    assert_eq!(
        completed["event"]["Complete"]["Shaded"]["flips"],
        json!([{"space":"santiago","faction":"M26","kind":"Guerrilla","from":"Underground","to":"Active","count":1}])
    );
    app.capture("shaded-card-end");
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], completed);
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained side decision captures"]
fn native_beneficiary_and_rally_preserve_side_decision_owners() {
    let mut app = event_app("unshaded-side-decisions-", "issue-121");
    open_event_handoff(&mut app);
    let before = app.observe()["observation"].clone();
    choose_rally_option(&mut app, "event:unshaded");
    let choosing = app.observe()["observation"].clone();
    assert_eq!(choosing["owner"], "Directorio");
    assert_eq!(choosing["event_executor"], "Directorio");
    assert_eq!(choosing["executing"], "Directorio");
    app.capture("beneficiary-selection");
    choose_rally_option(&mut app, "transfer");
    app.choose("give:1:1");
    let funded = app.observe()["observation"].clone();
    let mut balances = before["resources"].clone();
    balances[1] = (balances[1].as_u64().unwrap() + 1).into();
    balances[2] = (balances[2].as_u64().unwrap() - 1).into();
    assert_eq!(funded["resources"], balances);
    assert_eq!(funded["owner"], "Directorio");
    assert!(has_action(&funded, "rally:beneficiary:26july"));
    choose_rally_option(&mut app, "rally:beneficiary:26july");
    let rally = app.observe()["observation"].clone();
    assert_eq!(rally["owner"], "M26");
    assert_eq!(rally["executing"], "M26");
    assert_eq!(rally["event_executor"], "Directorio");
    app.capture("26july-rally");
    choose_rally_option(&mut app, "transfer");
    app.choose("ask:2:1");
    let pending = app.observe()["observation"].clone();
    assert_eq!(pending["owner"], "Directorio");
    assert_eq!(pending["executing"], "M26");
    assert_eq!(pending["event_executor"], "Directorio");
    assert!(!has_action(&pending, "rally:sierra-maestra:place:3"));
    app.capture("directorio-decides-26july-request");
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], pending);
    app.choose("decline");
    let resumed = app.observe()["observation"].clone();
    assert_eq!(resumed["resources"], balances);
    assert_eq!(resumed["owner"], "M26");
    assert!(has_action(&resumed, "rally:sierra-maestra:place:3"));

    choose_rally_option(&mut app, "remove");
    let removal = app.observe();
    assert!(!has_action(
        &removal["observation"],
        "remove:camaguey-province:underground:1"
    ));
    assert!(has_action(
        &removal["observation"],
        "remove:santiago:active:1"
    ));
    app.capture("26july-rally-removal");
    app.remove("santiago:active", 1);
    assert_eq!(available_guerrillas(&app.observe()["observation"]), 13);
    choose_rally_option(&mut app, "done-removal");
    let revised = app.observe()["observation"].clone();
    assert!(has_action(&revised, "rally:sierra-maestra:place:4"));
    choose_rally_option(&mut app, "rally:sierra-maestra:place:3");
    let card_end = app.observe();
    assert_unshaded_card_end(&revised, &card_end);
    assert_eq!(available_guerrillas(&card_end["observation"]), 10);
    assert_eq!(rally_space(&card_end["observation"])["support"], -2);
    app.capture("rally-card-end");
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], card_end["observation"]);
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained side decision captures"]
fn native_rally_removing_the_last_base_closes_the_operation() {
    let mut app = event_app("rally-removed-last-base-", "issue-121");
    if let Some(input) = std::env::var_os("CUBA_TEST_PENDING_RALLY") {
        std::fs::copy(input, &app.save).unwrap();
        app.launch_size("3840x2160");
    } else {
        open_event_handoff(&mut app);
        choose_rally_option(&mut app, "event:unshaded");
        choose_rally_option(&mut app, "rally:beneficiary:26july");
    }
    let rally = app.observe()["observation"].clone();
    assert!(has_action(&rally, "rally:sierra-maestra:place:4"));
    choose_rally_option(&mut app, "remove");
    app.remove("sierra-maestra:base", 1);
    let removal = app.observe()["observation"].clone();
    assert!(
        rally_space(&removal)["pieces"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        removal["standings"][1]["score"].as_i64().unwrap(),
        rally["standings"][1]["score"].as_i64().unwrap() - 1
    );
    assert_eq!(
        removal["available_piece_count"].as_u64().unwrap(),
        rally["available_piece_count"].as_u64().unwrap() + 1
    );
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], removal);
    choose_rally_option(&mut app, "done-removal");
    let card_end = app.observe();
    assert_unshaded_card_end(&rally, &card_end);
    assert!(
        card_end["observation"]["event"]["Complete"]["Unshaded"]["resolutions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let at = card_end["observation"]["revision"].as_u64().unwrap();
    let valid = std::fs::read(&app.save).unwrap();
    app.request(json!({"op":"activate","revision":at,"choice":format!("{at}:rally:sierra-maestra:place:4")}), false);
    assert_eq!(std::fs::read(&app.save).unwrap(), valid);
    app.capture("no-base-card-end");
    app.quit();
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
    let choices = view["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 1);
    assert_eq!(
        choices[0]["id"],
        format!("{}:advance-card", view["revision"].as_u64().unwrap())
    );
    assert_eq!(choices[0]["supported"], true);
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
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained zero-placement captures"]
fn native_free_rally_can_choose_zero_in_a_qualifying_base_space() {
    let mut app = rally_app();
    if let Some(input) = std::env::var_os("CUBA_TEST_RALLY_SAVE") {
        std::fs::copy(input, &app.save).unwrap();
        app.launch_size("3840x2160");
    } else {
        ready_for_event(&mut app, true);
    }
    choose_rally_option(&mut app, "event:unshaded");
    choose_rally_option(&mut app, "rally:beneficiary:26july");
    let shown = app.point("space:sierra-maestra");
    let pending = shown["observation"].clone();
    assert_eq!(shown["table"]["status"]["control"], "26July");
    assert_eq!(rally_space(&pending)["support"], -2);
    assert_eq!(
        rally_space(&pending)["pieces"],
        json!([{"faction":"M26","kind":"Base","state":"Ordinary","count":1}])
    );
    assert_eq!(available_guerrillas(&pending), 12);

    let mut row_tops = Vec::new();
    for count in 0..=4 {
        let action = format!("rally:sierra-maestra:place:{count}");
        assert!(has_action(&pending, &action), "{action}");
        let label = format!("option:{action}-label");
        let row = choice_part(&shown, &label);
        assert!(
            row["text"]
                .as_str()
                .unwrap()
                .contains(&format!("Place {count}"))
        );
        row_tops.push(row["bounds"][1].as_f64().unwrap());
    }
    // The selected Atlas within-section pitch is 97.2 frame units at 6144 wide.
    // Native text bounds round to whole pixels on the 3840-wide owner display.
    for pair in row_tops.windows(2) {
        assert!(((pair[1] - pair[0]) - 60.75).abs() <= 1.0, "{row_tops:?}");
    }
    let place_zero = "rally:sierra-maestra:place:0";
    let preview = app.point(&format!("option:{place_zero}"));
    let contents = tag_text(&preview["table"]["briefing"]);
    for text in [
        "No pieces move",
        "12 Guerrillas stay",
        "0 Resources",
        "Actual Active Opposition stays",
    ] {
        assert!(contents.contains(text), "{contents}");
    }
    app.capture("zero-placement-preview");
    choose_rally_option(&mut app, place_zero);

    let completed = app.observe();
    let after = &completed["observation"];
    assert_unshaded_card_end(&pending, &completed);
    for field in [
        "spaces",
        "available",
        "map_piece_count",
        "available_piece_count",
        "standings",
        "resources",
        "kidnap",
    ] {
        assert_eq!(after[field], pending[field], "{field}");
    }
    assert_eq!(
        after["event"]["Complete"]["Unshaded"]["resolutions"],
        json!([{"space":"sierra-maestra","action":{"Place":0}}])
    );
    let parts = completed["table"]["choice_parts"].as_array().unwrap();
    let text = parts
        .iter()
        .find(|part| part["part"] == "ended-value:UNSHADED EVENT · FREE RALLY")
        .unwrap()["text"]
        .as_str()
        .unwrap();
    assert!(text.contains("Chose to place 0 Guerrillas"), "{text}");
    assert!(text.contains("No pieces moved"), "{text}");
    assert!(!text.contains("No qualifying Base spaces"), "{text}");
    app.capture("zero-placement-card-end");
    assert_eq!(
        app.point("space:sierra-maestra")["table"]["status"]["control"],
        "26July"
    );

    let final_bytes = std::fs::read(&app.save).unwrap();
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], *after);
    assert_eq!(std::fs::read(&app.save).unwrap(), final_bytes);
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Rally captures"]
fn native_free_rally_hands_the_decision_to_the_beneficiary() {
    let mut app = rally_app();
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
            "rally:sierra-maestra:place:0",
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
    assert_eq!(
        final_position["map_piece_count"],
        handoff["map_piece_count"].as_u64().unwrap() + 3
    );
    assert_eq!(
        final_position["available_piece_count"],
        handoff["available_piece_count"].as_u64().unwrap() - 3
    );
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
    for field in ["random", "draw_deck"] {
        assert_eq!(
            final_record["payload"][field],
            before_record["payload"][field]
        );
    }
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
        let mut app = rally_app();
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
