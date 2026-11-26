use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;
use unfolded_record_replacement::previous_path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained Construct previews and results"]
fn native_operation_only_construct_resumes_the_map_and_finishes_in_payment_order() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-218"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("operation-only-construct-")
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
    eprintln!("Operation Only Construct evidence: {}", app.root.display());
    super::assault::fund_santiago(&app.save, 4);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(&mut app, action);
    }
    app.choose("advance-card");
    choose_row(&mut app, "government:pass");
    let before = app.observe()["observation"].clone();
    assert_eq!(before["next"]["role"], "1st Eligible");
    assert_casinos(&before, 15, 3, 7);
    choose_row(&mut app, "syndicate:construct");
    let open = app.observe();
    assert_eq!(
        choice_part(&open, "choice-subtitle")["text"],
        "15 Resources · 5 Resources per space"
    );
    let revision = app.revision();
    let saved = app.save_bytes();
    let first_action = "construct:camaguey-city:place";
    let first_id = format!("{revision}:{first_action}");
    let inspected = app.press_by_keyboard("space:camaguey-city");
    let row = &inspected["table"]["popover"]["card"]["rows"][0];
    assert_eq!(row["choice"], first_id);
    assert_eq!(row["button"], "Pay 5 · Place 1 closed Casino");
    let lines = row["lines"].to_string();
    assert!(lines.contains("Syndicate Resources 15 → 10."), "{lines}");
    assert!(lines.contains("0 → 0 open · 0 → 1 closed"), "{lines}");
    assert_eq!(inspected["observation"], open["observation"]);
    assert_eq!(app.save_bytes(), saved);
    app.capture("camaguey-payment-preview");

    app.press_by_keyboard("popover:commit");
    let after_first = app.observe();
    assert_casinos(&after_first["observation"], 10, 3, 6);
    let line = choice_part(&after_first, "construct-record:camaguey-city");
    assert!(
        line["text"]
            .as_str()
            .unwrap()
            .contains("Camagüey City · paid 5 Resources")
    );
    let targets = after_first["table"]["map_targets"].as_array().unwrap();
    let state = |space| {
        targets.iter().find(|t| t["space"] == json!(space)).unwrap()["state"]
            .as_str()
            .unwrap()
    };
    assert_eq!(state("camaguey-city"), "not-involved");
    assert_eq!(state("havana"), "selectable");
    let first_bytes = app.save_bytes();
    let first_revision = app.revision();
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    fs::write(app.root.join("after-first-payment.json"), &first_bytes).unwrap();
    app.capture("camaguey-paid");
    app.quit();

    app.launch_size("3840x2160");
    let restarted = app.observe();
    assert_eq!(restarted["observation"], after_first["observation"]);
    assert_eq!(
        restarted["table"]["map_targets"],
        after_first["table"]["map_targets"]
    );
    assert_eq!(app.save_bytes(), first_bytes);
    app.capture("camaguey-paid-restarted");
    app.point("space:camaguey-city");
    let used = app.observe()["table"]["status"]["reason"].clone();
    assert!(used.to_string().contains("already paid"), "{used}");
    app.activate("space:havana", first_revision);
    let shown = app.observe();
    let row = &shown["table"]["popover"]["card"]["rows"][0];
    let second_action = "construct:havana:place";
    assert_eq!(row["choice"], format!("{first_revision}:{second_action}"));
    let lines = row["lines"].to_string();
    assert!(lines.contains("Syndicate Resources 10 → 5."), "{lines}");
    assert!(lines.contains("1 → 1 open · 0 → 1 closed"), "{lines}");
    assert_eq!(shown["observation"], restarted["observation"]);
    app.capture("havana-payment-preview");
    app.key("Escape");

    let player = app.seat_agent(super::faction_seats::SYNDICATE, "syndicate");
    let (code, told) = app.ask(&["observe", &player]);
    assert_eq!(code, 0, "{told}");
    assert_eq!(
        told["view"]["state"]["construct_paid"],
        after_first["observation"]["construct_paid"]
    );
    let choice = told["view"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == format!("{first_revision}:{second_action}"))
        .unwrap();
    assert_eq!(choice["preview"]["resources"][0]["to"], 5);
    let repeated = app.decide(&player, first_action, "refused");
    assert!(
        repeated["error"].as_str().unwrap().contains("already paid"),
        "{repeated}"
    );
    let (code, stale) = app.ask(&["choose", &player, &revision.to_string(), &first_id]);
    assert_eq!(code, 1, "{stale}");
    assert!(
        stale["error"].as_str().unwrap().contains("Stale"),
        "{stale}"
    );
    assert_eq!(app.save_bytes(), first_bytes);
    app.decide(&player, second_action, "applied");

    let after_second = app.observe();
    let view = &after_second["observation"];
    assert_casinos(view, 5, 3, 5);
    let paid = view["construct_paid"].as_array().unwrap();
    assert_eq!(
        paid.iter()
            .map(|p| p["space"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["camaguey-city", "havana"]
    );
    let first = choice_part(&after_second, "construct-record:camaguey-city");
    let second = choice_part(&after_second, "construct-record:havana");
    assert!(first["bounds"][1].as_f64().unwrap() < second["bounds"][1].as_f64().unwrap());
    assert!(
        second["text"]
            .as_str()
            .unwrap()
            .contains("Havana · paid 5 Resources")
    );
    let pre_finish = app.save_bytes();
    fs::write(app.root.join("before-finish.json"), &pre_finish).unwrap();
    app.capture("two-payments-in-order");
    app.decide(&player, "end-construct", "applied");

    let (code, told) = app.ask(&["observe", &player]);
    assert_eq!(code, 0, "{told}");
    let report = told["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["recorded"], "Operation Only");
    assert_eq!(
        report["headline"],
        "Syndicate executes Construct in Camagüey City and Havana"
    );
    assert_eq!(report["complete"], true);
    assert_eq!(
        report["articles"][0]["lines"][0],
        "Syndicate pays 5 Resources, 15 → 10."
    );
    assert_eq!(
        report["articles"][1]["lines"][0],
        "Syndicate pays 5 Resources, 10 → 5."
    );
    fs::write(
        app.root.join("faction-client-after-finish.json"),
        serde_json::to_vec_pretty(&told).unwrap(),
    )
    .unwrap();
    let edition = app.observe()["table"]["newspaper"]["open"].clone();
    assert_eq!(edition["report"]["recorded"], "Operation Only");
    app.capture("construct-edition");
    app.key("Escape");

    let finished = app.observe();
    let after = &finished["observation"];
    assert_casinos(after, 5, 3, 5);
    assert_eq!(after["eligible"], json!([true, true, true, false]));
    assert_eq!(after["choices"], json!([]));
    assert_eq!(after["current_card"], 18);
    assert_eq!(after["preview_card"], 19);
    assert_eq!(after["deck"], before["deck"]);
    assert_eq!(after["capabilities"], json!([]));
    assert_eq!(finished["table"]["eligibility"]["track"]["adjusted"], true);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), pre_finish);
    let terminal = app.save_bytes();
    app.capture("construct-complete");
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], *after);
    let revision = app.revision();
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:end-construct")}),
        false,
    );
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), pre_finish);
    app.quit();

    let recovered = app.root.join("recovered.json");
    let recovery = Session::open(&recovered, Some(&app.save)).unwrap();
    assert_eq!(serde_json::to_value(recovery.observe()).unwrap(), *view);
    assert_eq!(fs::read(recovered).unwrap(), pre_finish);
    assert_eq!(app.save_bytes(), terminal);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained Construct previews and results"]
fn native_limited_construct_pays_in_havana_and_resumes_at_finish() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-217"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    let mut handoff: Option<Vec<u8>> = None;
    for (plan, label, count_line, available, score, via_agent) in [
        (
            "open",
            "Open 1 closed Casino",
            "Casinos here · 0 → 1 open · 1 → 0 closed.",
            7,
            3,
            false,
        ),
        (
            "place",
            "Place 1 closed Casino",
            "Casinos here · 0 → 0 open · 1 → 2 closed.",
            6,
            2,
            true,
        ),
    ] {
        let root = tempfile::Builder::new()
            .prefix(&format!("{plan}-havana-"))
            .tempdir_in(&evidence)
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
        eprintln!("Limited Construct evidence: {}", app.root.display());
        if let Some(handoff) = &handoff {
            fs::write(&app.save, handoff).unwrap();
            app.launch_size("3840x2160");
        } else {
            super::assault::fund_santiago(&app.save, 4);
            app.launch_size("3840x2160");
            for action in ["event:shaded", "resolve-shaded"] {
                choose_row(&mut app, action);
            }
            app.choose("advance-card");
            choose_row(&mut app, "government:assault");
            app.choose("assault:havana:syndicate");
            choose_row(&mut app, "end-assault");
            handoff = Some(fs::read(&app.save).unwrap());
        }

        let before = app.observe()["observation"].clone();
        assert_eq!(before["next"]["role"], "2nd Eligible");
        assert_casinos(&before, 15, 2, 7);
        choose_row(&mut app, "syndicate:limited-operation");
        if !via_agent {
            // Before the first irreversible effect, Back restores the same two actions.
            choose_row(&mut app, "back-to-choice");
            let left = app.observe()["observation"].clone();
            for field in ["spaces", "available", "resources", "eligible", "next"] {
                assert_eq!(left[field], before[field], "{field}");
            }
            assert_eq!(
                left["choices"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|choice| choice["supported"] == true)
                    .map(|choice| choice["label"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                ["Pass", "Limited Operation · Construct"],
            );
            choose_row(&mut app, "syndicate:limited-operation");
        }

        let initial = app.observe();
        assert!(
            choice_part(&initial, "choice-subtitle")["text"]
                .as_str()
                .unwrap()
                .contains("5 Resources for one space")
        );
        assert_eq!(
            initial["table"]["map_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["space"] == "havana")
                .unwrap()["state"],
            "selectable",
        );
        let revision = app.revision();
        let action = format!("construct:havana:{plan}");
        let id = format!("{revision}:{action}");
        let saved = fs::read(&app.save).unwrap();
        let shown = if via_agent {
            app.activate("space:havana", revision);
            app.activate(&format!("popover:row:{action}"), revision);
            app.observe()
        } else {
            app.press_by_keyboard("space:havana")
        };
        let rows = shown["table"]["popover"]["card"]["rows"]
            .as_array()
            .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| row["head"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["Open 1 closed Casino", "Place 1 closed Casino"]
        );
        let selected = rows.iter().find(|row| row["choice"] == id).unwrap();
        assert_eq!(selected["chosen"], true);
        assert_eq!(selected["button"], format!("Pay 5 · {label}"));
        let text = selected["lines"].to_string();
        assert!(text.contains("Syndicate Resources 15 → 10."), "{text}");
        assert!(text.contains(count_line), "{text}");
        assert!(
            text.contains(&format!("Casinos in Available Forces: 7 → {available}.")),
            "{text}"
        );
        assert_eq!(shown["observation"], initial["observation"]);
        assert_eq!(fs::read(&app.save).unwrap(), saved);
        app.capture("havana-payment-preview");

        if via_agent {
            app.key("Escape");
            let player = app.seat_agent(super::faction_seats::SYNDICATE, "syndicate");
            let (code, told) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{told}");
            let choice = told["view"]["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|choice| choice["id"] == id)
                .unwrap();
            assert_eq!(choice["supported"], true);
            assert_eq!(choice["preview"]["resources"][0]["to"], 10);
            assert!(
                choice["preview"]["notes"][0]
                    .as_str()
                    .unwrap()
                    .contains(count_line)
            );
            app.offers_nothing();
            app.decide(&player, &action, "applied");
        } else {
            app.press_by_keyboard("popover:commit");
        }
        let paid = app.observe();
        assert_casinos(&paid["observation"], 10, score, available);
        assert_eq!(paid["observation"]["revision"], revision + 1);
        assert_eq!(paid["observation"]["construct_paid"][0]["space"], "havana");
        assert_eq!(paid["observation"]["construct_paid"][0]["paid"], 5);
        let paid_line = choice_part(&paid, "construct-record:havana");
        assert!(
            paid_line["text"]
                .as_str()
                .unwrap()
                .contains("Havana · paid 5 Resources")
        );
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        let paid_bytes = fs::read(&app.save).unwrap();
        fs::write(app.root.join("before-finish.json"), &paid_bytes).unwrap();
        app.capture("havana-paid");
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], paid["observation"]);
        let pre_finish = app.revision();
        app.request(
            json!({"op":"activate","revision":revision,"choice":id}),
            false,
        );
        let attempt = format!("{pre_finish}:construct:havana:{plan}");
        app.request(
            json!({"op":"activate","revision":pre_finish,"choice":attempt}),
            false,
        );
        assert_eq!(fs::read(&app.save).unwrap(), paid_bytes);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        app.capture("havana-paid-restarted");

        if via_agent {
            let player = app.seat_agent(super::faction_seats::SYNDICATE, "syndicate");
            let refused = app.decide(&player, &action, "refused");
            assert!(
                refused["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("one space")),
                "{refused}"
            );
            let stale_revision = revision.to_string();
            let (code, stale) = app.ask(&["choose", &player, &stale_revision, &id]);
            assert_eq!(code, 1, "{stale}");
            assert!(
                stale["error"]
                    .as_str()
                    .is_some_and(|error| error.contains("Stale")),
                "{stale}"
            );
            assert_eq!(fs::read(&app.save).unwrap(), paid_bytes);
            app.decide(&player, "end-construct", "applied");
            let (code, told) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{told}");
            let report = told["view"]["results"].as_array().unwrap().last().unwrap();
            assert_eq!(report["action"], "construct");
            assert_eq!(report["recorded"], "Limited Operation");
            assert_eq!(report["complete"], true);
            assert_eq!(report["articles"][0]["title"], "Construct in Havana");
            assert_eq!(
                report["articles"][0]["lines"][0],
                "Syndicate pays 5 Resources, 15 → 10."
            );
            assert_eq!(
                report["articles"][0]["lines"][1],
                "Syndicate placed 1 closed Casino."
            );
            let edition = app.observe()["table"]["newspaper"]["open"].clone();
            assert_eq!(edition["report"]["action"], "construct");
            assert_eq!(edition["report"]["from_revision"], pre_finish);
            assert_eq!(edition["report"]["recorded"], "Limited Operation");
            assert_eq!(edition["report"]["complete"], true);
            fs::write(
                app.root.join("faction-client-after-finish.json"),
                serde_json::to_vec_pretty(&told).unwrap(),
            )
            .unwrap();
            app.capture("havana-construct-edition");
            app.key("Escape");
        } else {
            choose_row(&mut app, "end-construct");
        }

        let finished = app.observe();
        let after = &finished["observation"];
        assert_casinos(after, 10, score, available);
        assert_eq!(after["revision"], pre_finish + 1);
        assert_eq!(after["eligible"], json!([false, true, true, false]));
        assert_eq!(after["choices"], json!([]));
        assert_eq!(after["current_card"], 18);
        assert_eq!(after["preview_card"], 19);
        assert_eq!(after["deck"], before["deck"]);
        assert_eq!(finished["table"]["eligibility"]["track"]["adjusted"], true);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), paid_bytes);
        let finished_bytes = fs::read(&app.save).unwrap();
        app.capture("havana-construct-complete");
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], *after);
        app.request(json!({"op":"activate","revision":pre_finish + 1,"choice":format!("{}:end-construct", pre_finish + 1)}), false);
        assert_eq!(fs::read(&app.save).unwrap(), finished_bytes);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), paid_bytes);
        app.quit();
        if via_agent {
            let original = app.save.clone();
            app.save = app.root.join("recovered.json");
            let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
                .args([
                    "--cards",
                    app.cards.to_str().unwrap(),
                    "--save",
                    app.save.to_str().unwrap(),
                    "--recover-from",
                    original.to_str().unwrap(),
                    "--agent-seat",
                    "--seat-size",
                    "3840x2160",
                    "--socket",
                    app.socket.to_str().unwrap(),
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "recovery: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            fs::write(app.root.join("recovery-launch.jsonl"), output.stdout).unwrap();
            assert_eq!(fs::read(&app.save).unwrap(), paid_bytes);
            assert_eq!(fs::read(&original).unwrap(), finished_bytes);
            assert_eq!(app.observe()["observation"], paid["observation"]);
            app.quit();
        }
    }
}

pub(super) fn choose_row(app: &mut Native, action: &str) {
    let revision = app.revision();
    app.activate(&format!("option:{action}"), revision);
    app.choose(action);
}

fn assert_casinos(view: &Value, resources: u8, open: u8, available: u8) {
    assert_eq!(view["resources"][3], resources);
    assert_eq!(view["standings"][3]["score"], open);
    let inventory = view["available"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["kind"] == "Casino")
        .unwrap();
    assert_eq!(inventory["count"], available);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; Casino artwork, layout, keyboard, agent and replay"]
fn native_casino_management_compares_pinar_and_keeps_the_payment() {
    use super::{focus_mark::assert_marked, map_keys};
    const OPEN: &str = "own-piece:pinar-del-rio:open-casino";
    const OPEN_PIECE: &str = "piece:pinar-del-rio Syndicate Casino Open";
    const CLOSED_PIECE: &str = "piece:pinar-del-rio Syndicate Casino Closed";
    const CLOSE: &str = "popover:casino-close";
    const RETURN: &str = "popover:casino-return";

    let mut starting: Option<Vec<u8>> = None;
    for branch in ["close", "return", "stack"] {
        let mut app = super::available_forces::unlaunched(&format!("pinar-{branch}-"), "issue-219");
        if let Some(bytes) = &starting {
            fs::write(&app.save, bytes).unwrap();
            app.launch_size("3840x2160");
        } else {
            // Use the existing checked funding route, in a fresh disposable file.
            super::assault::fund_santiago(&app.save, 4);
            app.launch_size("3840x2160");
            for action in ["event:shaded", "resolve-shaded"] {
                choose_row(&mut app, action);
            }
            app.choose("advance-card");
            choose_row(&mut app, "government:assault");
            app.choose("assault:havana:syndicate");
            choose_row(&mut app, "end-assault");
            starting = Some(app.save_bytes());
        }
        choose_row(&mut app, "syndicate:limited-operation");
        let (space, plan) = if branch == "stack" {
            ("pinar-del-rio", "place")
        } else {
            ("havana", "open")
        };
        let revision = app.revision();
        app.activate(&format!("space:{space}"), revision);
        app.activate(&format!("popover:row:construct:{space}:{plan}"), revision);
        app.activate("popover:commit", revision);
        choose_row(&mut app, "remove");
        let before = app.observe();
        let revision = app.revision();
        let saved = app.save_bytes();

        if branch == "stack" {
            assert_casinos(&before["observation"], 10, 2, 6);
            app.capture("pinar-adjacent-home");
            for (name, scroll) in [("near", 3), ("far", -5)] {
                app.key("Home");
                app.point("space:las-villas");
                app.request(json!({"op":"scroll","revision":revision,"component":"space:las-villas","y":scroll}), true);
                let zoomed = app.settle();
                assert_ne!(zoomed["table"]["camera"], before["table"]["camera"]);
                app.capture(&format!("pinar-adjacent-{name}"));
            }
            app.key("Home");
            app.activate(CLOSED_PIECE, revision);
            let closed = app.observe()["table"]["popover"].clone();
            assert_eq!(closed["kind"], "closed-casino");
            assert_eq!(closed["card"]["rows"].as_array().unwrap().len(), 2);
            assert_eq!(
                closed["card"]["rows"][1]["head"],
                "Return 1 Casino to Available"
            );
            app.capture("pinar-closed-preview");
            app.key("Escape");
            app.activate(OPEN_PIECE, revision);
            assert!(
                app.observe()["table"]["popover"]["card"]["rows"][0]["lines"]
                    .to_string()
                    .contains("stays a paid space")
            );
            app.activate(CLOSE, revision);
            assert_casino_pinar(&app.observe(), 2);
            let revision = app.revision();
            app.activate(CLOSED_PIECE, revision);
            map_keys::tab_to(&mut app, "popover:quantity:plus");
            map_keys::press(&mut app, "Space", "popover:quantity:plus");
            let stack = app.observe()["table"]["popover"]["card"].clone();
            assert_eq!(stack["stepper"]["value"], 2);
            assert_eq!(stack["rows"][1]["head"], "Return 2 Casinos to Available");
            assert!(
                stack["rows"][1]["lines"]
                    .to_string()
                    .contains("goes from 6 to 8")
            );
            app.capture("pinar-closed-stack-preview");
            app.activate(RETURN, revision);
            let after = app.observe();
            assert_casino_pinar(&after, 0);
            assert_casinos(&after["observation"], 10, 1, 8);
            assert_eq!(
                after["observation"]["construct_paid"][0]["space"],
                "pinar-del-rio"
            );
            choose_row(&mut app, "done-removal");
            let revision = app.revision();
            let refused = format!("{revision}:construct:pinar-del-rio:place");
            app.request(
                json!({"op":"activate","revision":revision,"choice":refused}),
                false,
            );
            choose_row(&mut app, "end-construct");
            app.quit();
            continue;
        }

        assert_casinos(&before["observation"], 10, 3, 7);
        map_keys::tab_to(&mut app, OPEN);
        let preview = map_keys::press(&mut app, "Enter", OPEN);
        let card = &preview["table"]["popover"]["card"];
        assert_eq!(card["kind"], "open-casino");
        assert!(
            card["rows"][0]["lines"]
                .to_string()
                .contains("Both choices end Syndicate Control here")
        );
        assert!(card["rows"][0]["lines"].to_string().contains("3 → 2"));
        assert_eq!(card["rows"][1]["button"], "Close 1 Casino in place");
        assert_eq!(card["rows"][2]["button"], "Return 1 Casino to Available");
        assert!(
            card["rows"][1]["lines"]
                .to_string()
                .contains("Available stays at 7")
        );
        assert!(
            card["rows"][2]["lines"]
                .to_string()
                .contains("goes from 7 to 8")
        );
        assert_eq!(card["footer"], "0 Resources · no Profit");
        compare_casino_layout(&preview);

        map_keys::tab_to(&mut app, CLOSE);
        assert_marked(&mut app, CLOSE, "pinar-close-focus");
        map_keys::tab_to(&mut app, RETURN);
        assert_marked(&mut app, RETURN, "pinar-return-focus");
        let cancelled = app.key("Escape");
        assert!(cancelled["table"]["popover"].is_null());
        assert_eq!(cancelled["table"]["choice"]["focus"], OPEN);
        app.activate(OPEN_PIECE, revision);
        app.activate("popover:cancel", revision);
        assert!(app.observe()["table"]["popover"].is_null());
        assert_eq!(app.save_bytes(), saved);
        assert_eq!(app.observe()["observation"], before["observation"]);
        app.activate(OPEN_PIECE, revision);

        let player = if branch == "return" {
            let player = app.seat_agent(super::faction_seats::SYNDICATE, "syndicate");
            let (code, seen) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{seen}");
            let actions = seen["view"]["choices"].as_array().unwrap();
            let open = actions
                .iter()
                .find(|c| c["id"] == format!("{revision}:remove:pinar-del-rio:open-casino:1"))
                .unwrap();
            assert_eq!(open["preview"]["removed"][0]["count"], 1);
            assert_eq!(
                open["preview"]["management"]["casinos_available"],
                json!([7, 8])
            );
            assert_eq!(open["preview"]["resources"], json!([]));
            let close = actions
                .iter()
                .find(|c| c["id"] == format!("{revision}:close-casino:pinar-del-rio:open-casino:1"))
                .unwrap();
            assert_eq!(close["preview"]["flips"][0]["to"], "Closed");
            assert_eq!(
                close["preview"]["management"]["casinos_available"],
                json!([7, 7])
            );
            for control in app.observe()["table"]["controls"].as_array().unwrap() {
                assert!(control["name"] != CLOSE && control["name"] != RETURN);
            }
            app.key("Escape");
            app.decide(&player, "remove:pinar-del-rio:open-casino:1", "applied");
            Some(player)
        } else {
            map_keys::tab_to(&mut app, CLOSE);
            map_keys::press(&mut app, "Space", CLOSE);
            None
        };
        let after = app.observe();
        let closed = branch == "close";
        assert_casinos(&after["observation"], 10, 2, if closed { 7 } else { 8 });
        assert_casino_pinar(&after, if closed { 1 } else { 0 });
        assert_eq!(
            after["observation"]["construct_paid"],
            before["observation"]["construct_paid"]
        );
        assert!(after["observation"]["removal"].is_string());
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        app.capture("pinar-after");
        let after_bytes = app.save_bytes();
        fs::write(app.root.join("after-management.json"), &after_bytes).unwrap();
        let pre_finish = if let Some(player) = player {
            app.decide(&player, "done-removal", "applied");
            let pre_finish = app.save_bytes();
            app.decide(&player, "end-construct", "applied");
            let (code, seen) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{seen}");
            let return_written = "Syndicate returns 1 open Casino to Available Forces.";
            let report = &seen["view"]["results"].as_array().unwrap().last().unwrap();
            assert!(
                report["articles"].to_string().contains(return_written),
                "{report}"
            );
            let edition = &app.observe()["table"]["newspaper"]["open"]["report"];
            assert!(
                edition["articles"].to_string().contains(return_written),
                "{edition}"
            );
            fs::write(
                app.root.join("faction-and-report.json"),
                serde_json::to_vec_pretty(&seen).unwrap(),
            )
            .unwrap();
            app.capture("pinar-return-edition");
            app.key("Escape");
            pre_finish
        } else {
            app.quit();
            app.launch_size("3840x2160");
            assert_eq!(app.observe()["observation"], after["observation"]);
            assert_eq!(app.save_bytes(), after_bytes);
            let stale = format!("{revision}:close-casino:pinar-del-rio:open-casino:1");
            app.request(
                json!({"op":"activate","revision":revision,"choice":stale}),
                false,
            );
            assert_eq!(app.save_bytes(), after_bytes);
            choose_row(&mut app, "done-removal");
            let pre_finish = app.save_bytes();
            choose_row(&mut app, "end-construct");
            pre_finish
        };
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), pre_finish);
        fs::write(app.root.join("before-finish.json"), &pre_finish).unwrap();
        let complete = app.observe()["observation"].clone();
        assert_eq!(complete["eligible"], json!([false, true, true, false]));
        // fund_santiago creates the seed-2 opening and gives Government 4 Resources.
        super::faction_seats::assert_by_hand_of(&app.save, 2);
        app.quit();
        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], complete);
        app.quit();
    }
}

fn assert_casino_pinar(view: &Value, closed: u8) {
    let pinar = view["observation"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|space| space["id"] == "pinar-del-rio")
        .unwrap();
    if closed == 0 {
        assert_eq!(pinar["pieces"], json!([]));
    } else {
        assert_eq!(pinar["pieces"].as_array().unwrap().len(), 1);
        assert_eq!(pinar["pieces"][0]["state"], "Closed");
        assert_eq!(pinar["pieces"][0]["count"], closed);
    }
}

fn compare_casino_layout(native: &Value) {
    let source: Value = serde_json::from_str(include_str!(
        "../../art/atlas/atlas-casino-actions.layout.json"
    ))
    .unwrap();
    let tolerance = source["tolerance"].as_f64().unwrap();
    let popover = &native["table"]["popover"];
    let [x, y, ..] = rect_of(&popover["bounds"]);
    let anchor_y = popover["anchor"][1].as_f64().unwrap();
    let anchor_from_top = source["anchor_from_top"].as_f64().unwrap();
    assert!((anchor_y - y - anchor_from_top).abs() <= tolerance);
    for (name, reference) in source["parts"].as_object().unwrap() {
        let [left, top, width, height] = rect_of(reference);
        let expected = [x + left, y + top, x + left + width, y + top + height];
        let part = popover["parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|part| part["part"] == name.as_str())
            .unwrap();
        let measured = rect_of(&part["bounds"]);
        for (a, b) in measured.into_iter().zip(expected) {
            assert!(
                (a - b).abs() <= tolerance,
                "{name}: {measured:?} vs {expected:?}"
            );
        }
    }
}
