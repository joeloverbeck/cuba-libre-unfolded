use super::*;
use cuba_libre_unfolded::{assets, save::Session};
use std::fs;
use unfolded_record_replacement::previous_path;

const OWNER_SAVES: [(&str, &str, u64); 2] = [
    (
        "shaded",
        "9c1d99b46e5d9279ac24d9f8b0018556e58d03c48b7aa6d317fe502d49a213b7",
        11,
    ),
    (
        "unshaded",
        "57f4b367566c1ca341a107b2527ba8612e4d29b6cf0eab0049f64bb5d5a5cf17",
        16,
    ),
];

fn copy_owner_save(dir: &Path, text: &str, hash: &str, path: &Path) {
    let original = dir.join(format!("117-alberto-bayo-59e3c0f-{text}.json"));
    let bytes = fs::read(&original).unwrap();
    assert_eq!(assets::digest(&bytes), hash);
    fs::write(path, bytes).unwrap();
    fs::copy(previous_path(&original), previous_path(path)).unwrap();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained card reader and table"]
fn native_card_advance_reopens_both_event_results_and_commits_once() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-128"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    let retained = std::env::var_os("CUBA_TEST_CARD_18_SAVES").map(PathBuf::from);
    for (text, hash, revision) in OWNER_SAVES {
        let root = tempfile::Builder::new()
            .prefix(&format!("{text}-"))
            .tempdir_in(&evidence)
            .unwrap()
            .keep();
        let socket = tempfile::tempdir().unwrap();
        let mut app = Native {
            cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
            save: root.join("task.json"),
            socket: socket.path().join("seat.sock"),
            root,
            step: 0,
        };
        eprintln!("{text} card advance evidence: {}", app.root.display());
        if let Some(dir) = &retained {
            copy_owner_save(dir, text, hash, &app.save);
        } else {
            finish_event(&app.save, text);
        }
        let saved = fs::read(&app.save).unwrap();
        let prior = fs::read(previous_path(&app.save)).unwrap();
        let random = Session::open(&app.save, None)
            .unwrap()
            .game()
            .random_state();
        assert_missing_or_changed_face_is_recoverable(&app, &saved, &prior);

        app.launch_size("3840x2160");
        let end = app.observe()["observation"].clone();
        if retained.is_some() {
            assert_eq!(end["revision"], revision);
            assert_eq!(end["kidnap"]["die"], 5);
            assert_eq!(end["resources"][0], 1);
        }
        let revision = end["revision"].as_u64().unwrap();
        assert_eq!(end["current_card"], 17);
        assert_eq!(end["preview_card"], 18);
        assert_eq!(end["draw_count"], 34);
        app.capture("card-17-result");
        app.activate("reader", revision);
        app.capture("card-17-current-18-preview");
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], end);
        app.choose("advance-card");
        let opening = app.observe();
        let after = &opening["observation"];
        assert_eq!(after["revision"], revision + 1);
        assert_eq!(after["current_card"], 18);
        assert_eq!(after["preview_card"], 19);
        assert_eq!(after["draw_count"], 33);
        assert_eq!(
            after["played_cards"].as_array().unwrap().last().unwrap(),
            17
        );
        assert!(after.get("draw_deck").is_none());
        assert_eq!(after["capabilities"], json!([]));
        assert_eq!(after["owner"], "Government");
        assert_eq!(after["eligibility"]["first"], "Government");
        assert_eq!(after["eligibility"]["second"], "Syndicate");
        assert_eq!(
            choice_part(&opening, "choice-heading")["text"],
            "Government's decision"
        );
        assert_eq!(after["eligible"], json!([true, false, false, true]));
        assert_eq!(
            after["eligibility"]["event_restriction"],
            if text == "shaded" {
                json!({"faction": "M26", "through_card": 18})
            } else {
                Value::Null
            }
        );
        for key in [
            "spaces",
            "available",
            "resources",
            "resolved",
            "kidnap",
            "standings",
        ] {
            assert_eq!(after[key], end[key], "{key}");
        }
        assert_eq!(
            Session::open(&app.save, None)
                .unwrap()
                .game()
                .random_state(),
            random
        );
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        app.capture("card-18-government-opening");
        app.activate("option:government:assault", revision + 1);
        let briefing = app.observe();
        let briefing_text = tag_text(&briefing["table"]["briefing"]);
        assert!(
            briefing_text.contains("2 Resources per space"),
            "{briefing_text}"
        );
        if retained.is_some() {
            assert!(
                briefing_text.contains("1 Resource and cannot pay"),
                "{briefing_text}"
            );
            assert!(briefing_text.contains("A zero-space Assault earns no Resources"));
        }
        app.capture("card-18-assault-briefing");
        app.activate("reader", revision + 1);
        app.capture("card-18-current-19-preview");
        app.quit();

        let advanced = fs::read(&app.save).unwrap();
        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], *after);
        for request in [
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:advance-card")}),
            json!({"op":"activate","revision":revision+1,"choice":format!("{}:advance-card", revision+1)}),
        ] {
            app.request(request, false);
        }
        assert_eq!(app.observe()["observation"], *after);
        assert_eq!(fs::read(&app.save).unwrap(), advanced);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        app.capture("card-18-restarted");
        if retained.is_some() {
            for action in [
                "government:assault",
                "assault:areas:cities",
                "assault:space:santiago",
            ] {
                let current = app.observe()["observation"]["revision"].as_u64().unwrap();
                app.activate(&format!("option:{action}"), current);
                app.choose(action);
            }
            let pending = app.observe()["observation"].clone();
            let payment_revision = pending["revision"].as_u64().unwrap();
            let active = fs::read(&app.save).unwrap();
            let previous = fs::read(previous_path(&app.save)).unwrap();
            app.activate("option:assault:santiago:26july", payment_revision);
            assert!(
                tag_text(&app.observe()["table"]["briefing"])
                    .contains("1 Resource and cannot pay 2 Resources")
            );
            app.activate("choice-commit", payment_revision);
            let refused = app.observe();
            assert!(
                refused["table"]["notice"]
                    .as_str()
                    .unwrap()
                    .contains("1 Resource and cannot pay 2 Resources")
            );
            assert_eq!(refused["observation"], pending);
            assert_eq!(fs::read(&app.save).unwrap(), active);
            assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
            app.capture("unfunded-assault-payment-refused");

            for action in ["assault:back", "assault:back", "end-assault"] {
                let current = app.observe()["observation"]["revision"].as_u64().unwrap();
                app.activate(&format!("option:{action}"), current);
                if action == "end-assault" {
                    let explanation = tag_text(&app.observe()["table"]["briefing"]);
                    assert!(explanation.contains("earns no Pass Resources (local policy)"));
                    app.capture("zero-space-assault-preview");
                }
                app.choose(action);
            }
            let handoff = app.observe()["observation"].clone();
            assert_eq!(handoff["current_card"], 18);
            assert_eq!(handoff["resources"], after["resources"]);
            assert_eq!(handoff["spaces"], after["spaces"]);
            assert_eq!(handoff["available"], after["available"]);
            assert_eq!(handoff["eligibility"]["government_box"], "OperationOnly");
            assert_eq!(handoff["owner"], "Syndicate");
            assert_eq!(handoff["next"]["role"], "2nd Eligible");
            assert_eq!(
                handoff["next"]["options"],
                json!(["Pass", "Limited Operation"])
            );
            app.capture("zero-space-assault-handoff");
        }
        app.quit();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Pass and handoff"]
fn native_government_pass_credits_once_and_hands_first_choice_to_syndicate() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-129"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    let retained = std::env::var_os("CUBA_TEST_CARD_18_SAVES").map(PathBuf::from);
    for (text, hash, _) in OWNER_SAVES {
        let root = tempfile::Builder::new()
            .prefix(&format!("{text}-"))
            .tempdir_in(&evidence)
            .unwrap()
            .keep();
        let socket = tempfile::tempdir().unwrap();
        let mut app = Native {
            cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
            save: root.join("task.json"),
            socket: socket.path().join("seat.sock"),
            root,
            step: 0,
        };
        eprintln!("{text} Government Pass evidence: {}", app.root.display());
        if let Some(dir) = &retained {
            copy_owner_save(dir, text, hash, &app.save);
        } else {
            finish_event(&app.save, text);
        }

        app.launch_size("3840x2160");
        app.choose("advance-card");
        let opening = app.observe()["observation"].clone();
        let revision = opening["revision"].as_u64().unwrap();
        let from = opening["resources"][0].as_u64().unwrap();
        if retained.is_some() {
            assert_eq!(opening["kidnap"]["die"], 5);
            assert_eq!(from, 1);
        }
        let to = from + 3; // R 2.3.3: both worked saves are below the R 1.8 cap.
        let balances = json!([
            to,
            opening["resources"][1],
            opening["resources"][2],
            opening["resources"][3]
        ]);
        let advanced = fs::read(&app.save).unwrap();
        let previous = fs::read(previous_path(&app.save)).unwrap();
        let random = Session::open(&app.save, None)
            .unwrap()
            .game()
            .random_state();
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], opening);
        app.capture("government-opening");
        if from == 1 {
            app.activate("option:government:assault", revision);
            let refusal = tag_text(&app.observe()["table"]["briefing"]);
            assert!(refusal.contains("2 Resources per space"), "{refusal}");
            assert!(refusal.contains("1 Resource and cannot pay"), "{refusal}");
            assert!(
                refusal.contains("A zero-space Assault earns no Resources"),
                "{refusal}"
            );
        }
        app.activate("option:government:pass", revision);
        let preview = app.observe();
        let briefing = tag_text(&preview["table"]["briefing"]);
        assert!(briefing.contains("Pass pays 3 Resources"), "{briefing}");
        assert!(briefing.contains(&format!("{from} → {to}")), "{briefing}");
        assert_eq!(preview["observation"], opening);
        assert_eq!(fs::read(&app.save).unwrap(), advanced);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
        app.capture("government-pass-preview");

        let pass = format!("{revision}:government:pass");
        app.request(
            json!({"op":"activate","revision":revision-1,"choice":pass}),
            false,
        );
        assert_eq!(app.observe()["observation"], opening);
        app.choose("government:pass");
        let handoff = app.observe();
        let after = &handoff["observation"];
        assert_eq!(after["resources"], balances);
        assert_eq!(after["revision"], revision + 1);
        assert_eq!(after["owner"], "Syndicate");
        assert_eq!(after["eligibility"]["first"], "Syndicate");
        assert_eq!(after["eligibility"]["second"], Value::Null);
        assert_eq!(after["eligibility"]["government_box"], "Pass");
        assert_eq!(after["eligible"], opening["eligible"]);
        assert_eq!(
            after["recorded_resources"],
            json!([{"faction":"Government","from":from,"to":to}])
        );
        for key in [
            "current_card",
            "preview_card",
            "draw_count",
            "played_cards",
            "kidnap",
            "spaces",
            "available",
        ] {
            assert_eq!(after[key], opening[key], "{key}");
        }
        assert_eq!(
            choice_part(&handoff, "choice-heading")["text"],
            "Syndicate's decision"
        );
        let subtitle = choice_part(&handoff, "choice-subtitle")["text"]
            .as_str()
            .unwrap();
        assert!(subtitle.contains("1st Eligible on card 18"), "{subtitle}");
        assert!(
            subtitle.contains(&format!("+3 Resources ({from} → {to})")),
            "{subtitle}"
        );
        let track = &handoff["table"]["eligibility"]["track"];
        assert_eq!(track["occupied"], "eligibility:pass");
        assert_eq!(track["occupant"], "Government");
        assert!(
            track["rows"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["role"] != "2nd")
        );
        for action in [
            "pass",
            "event",
            "rally",
            "march",
            "terror",
            "construct",
            "profit",
            "muscle",
            "bribe",
        ] {
            let name = format!("option:syndicate:{action}");
            assert_eq!(choice_part(&handoff, &name)["part"], name);
        }
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), advanced);
        assert_eq!(
            Session::open(&app.save, None)
                .unwrap()
                .game()
                .random_state(),
            random
        );
        let committed = fs::read(&app.save).unwrap();
        app.capture("syndicate-opening");
        app.point("eligibility:pass");
        let result = app.observe();
        let tag = &result["table"]["eligibility"]["tag"];
        assert!(tag["body"].as_str().unwrap().contains("Government Passed"));
        app.capture("government-pass-record");
        app.activate("option:syndicate:construct", revision + 1);
        let inspection = app.observe();
        let briefing = tag_text(&inspection["table"]["briefing"]);
        assert!(
            briefing.contains("next Eligible faction in card order"),
            "{briefing}"
        );
        assert!(briefing.contains("5 Resources"), "{briefing}");
        app.capture("syndicate-construct-briefing");
        app.request(
            json!({"op":"activate","revision":revision+1,"choice":format!("{}:syndicate:construct", revision+1)}),
            false,
        );
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], *after);
        for request in [
            json!({"op":"activate","revision":revision,"choice":pass}),
            json!({"op":"activate","revision":revision+1,"choice":format!("{}:government:pass", revision+1)}),
        ] {
            app.request(request, false);
        }
        assert_eq!(app.observe()["observation"], *after);
        assert_eq!(fs::read(&app.save).unwrap(), committed);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), advanced);
        app.capture("syndicate-restarted");
        app.quit();
    }
}

fn finish_event(path: &Path, text: &str) {
    let mut session = Session::open(path, None).unwrap();
    let resolution = if text == "shaded" {
        "resolve-shaded"
    } else {
        "rally:beneficiary:directorio"
    };
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        &format!("event:{text}"),
        resolution,
    ] {
        let view = session.observe();
        session
            .submit(
                view.owner,
                view.revision,
                &format!("{}:{action}", view.revision),
            )
            .unwrap();
    }
}

fn assert_missing_or_changed_face_is_recoverable(app: &Native, saved: &[u8], prior: &[u8]) {
    let wrong_cards = tempfile::tempdir().unwrap();
    for name in ["card-17.png", "card-18.png"] {
        fs::copy(app.cards.join(name), wrong_cards.path().join(name)).unwrap();
    }
    for problem in ["missing", "changed"] {
        if problem == "changed" {
            let mut bytes = fs::read(app.cards.join("card-19.png")).unwrap();
            bytes[100] ^= 1;
            fs::write(wrong_cards.path().join("card-19.png"), bytes).unwrap();
        }
        let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
            .arg("--cards")
            .arg(wrong_cards.path())
            .arg("--save")
            .arg(&app.save)
            .arg("--agent-seat")
            .arg("--socket")
            .arg(&app.socket)
            .output()
            .unwrap();
        fs::write(
            app.root.join(format!("{problem}-19.stderr")),
            &output.stderr,
        )
        .unwrap();
        let reason = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            let _ = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
                .arg("seat")
                .arg(&app.socket)
                .arg(r#"{"op":"quit"}"#)
                .output();
        }
        assert!(!output.status.success(), "{problem} card 19 was accepted");
        assert!(
            reason.contains("card-19.png") && reason.contains("--cards"),
            "{reason}"
        );
        assert_eq!(fs::read(&app.save).unwrap(), saved);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), prior);
    }
}
