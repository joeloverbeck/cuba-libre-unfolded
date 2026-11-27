use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;
use unfolded_record_replacement::previous_path;

/// Both texts of the Alberto Bayo Event, played to card end by `finish_event`.
const TEXTS: [&str; 2] = ["shaded", "unshaded"];

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained card tags and table"]
fn native_card_advance_reopens_both_event_results_and_commits_once() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-128"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    for text in TEXTS {
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
        finish_event(&app.save, text);
        let saved = fs::read(&app.save).unwrap();
        let prior = fs::read(previous_path(&app.save)).unwrap();
        let random = Session::open(&app.save, None)
            .unwrap()
            .game()
            .random_state();
        assert_missing_or_changed_face_is_recoverable(&app, &saved, &prior, "card-19.png");

        app.launch_size("3840x2160");
        let end = app.observe()["observation"].clone();
        assert_eq!(end["kidnap"]["die"], 5);
        assert_eq!(end["resources"][0], 1);
        let revision = end["revision"].as_u64().unwrap();
        assert_eq!(end["current_card"], 17);
        assert_eq!(end["preview_card"], 18);
        assert_eq!(end["draw_count"], 34);
        // The piles at rest on card 17 (#143): 18 played with it, 34 in the deck.
        let resting = app.request(json!({"op":"settle"}), true);
        let resting = if resting["table"].is_null() {
            app.observe()
        } else {
            resting
        };
        card_piles::assert_piles(&resting, 18, 34, 17, 18);
        app.capture("card-17-result");
        card_piles::assert_piles_drawn(&app, "card-17-result", &resting);
        // Both cards read through their tags (#139).
        for (slot, number) in [("card:current", 17), ("card:preview", 18)] {
            app.activate(slot, revision);
            let tag = app.observe();
            assert_card_tag_placed(&tag, slot);
            assert_eq!(tag["table"]["card"]["shown"]["number"], number);
            app.capture(&format!("card-17-tag-{number}"));
            close_card_tag(&mut app);
        }
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], end);
        // A restart before the advance shows the same piles.
        app.request(json!({"op":"settle"}), true);
        let restarted = app.observe();
        card_piles::assert_piles(&restarted, 18, 34, 17, 18);
        assert_eq!(restarted["table"]["piles"], resting["table"]["piles"]);
        app.choose("advance-card");
        app.request(json!({"op":"settle"}), true);
        let opening = app.observe();
        // The advance shows the new resting state: 19 and 33.
        card_piles::assert_piles(&opening, 19, 33, 18, 19);
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
        card_piles::assert_piles_drawn(&app, "card-18-government-opening", &opening);
        app.activate("option:government:assault", revision + 1);
        let briefing = app.observe();
        let briefing_text = tag_text(&briefing["table"]["briefing"]);
        assert!(
            briefing_text.contains("2 Resources per space"),
            "{briefing_text}"
        );
        assert!(
            briefing_text.contains("1 Resource and cannot pay"),
            "{briefing_text}"
        );
        assert!(briefing_text.contains("A zero-space Assault earns no Resources"));
        app.capture("card-18-assault-briefing");
        for (slot, number) in [("card:current", 18), ("card:preview", 19)] {
            app.activate(slot, revision + 1);
            let tag = app.observe();
            assert_card_tag_placed(&tag, slot);
            assert_eq!(tag["table"]["card"]["shown"]["number"], number);
            app.capture(&format!("card-18-tag-{number}"));
            close_card_tag(&mut app);
        }
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
        // A restart after the advance shows the saved cards at rest.
        app.request(json!({"op":"settle"}), true);
        let resumed = app.observe();
        card_piles::assert_piles(&resumed, 19, 33, 18, 19);
        assert_eq!(resumed["table"]["piles"], opening["table"]["piles"]);
        app.capture("card-18-restarted");
        card_piles::assert_piles_drawn(&app, "card-18-restarted", &resumed);
        {
            let action = "government:assault";
            let current = app.observe()["observation"]["revision"].as_u64().unwrap();
            app.activate(&format!("option:{action}"), current);
            app.choose(action);
        }
        // Government has 1 Resource and a space costs 2: Santiago is blocked on the
        // map (#169). Its popover gives the reason and offers no button, and the
        // payment itself is refused with nothing changed.
        let pending = app.observe()["observation"].clone();
        let payment_revision = pending["revision"].as_u64().unwrap();
        let active = fs::read(&app.save).unwrap();
        let previous = fs::read(previous_path(&app.save)).unwrap();
        app.activate("space:santiago", payment_revision);
        let blocked = app.observe();
        let row = &blocked["table"]["popover"]["card"]["rows"][0];
        assert_eq!(row["mark"], "not-yet", "{row}");
        assert!(row["button"].is_null(), "{row}");
        assert!(
            row["lines"][0][1]
                .as_str()
                .unwrap()
                .contains("1 Resource and cannot pay 2 Resources"),
            "{row}"
        );
        app.request(
            json!({"op":"activate","revision":payment_revision,
                "choice":format!("{payment_revision}:assault:santiago:26july")}),
            false,
        );
        let refused = app.observe();
        assert_eq!(refused["observation"], pending);
        assert_eq!(fs::read(&app.save).unwrap(), active);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
        app.capture("unfunded-assault-payment-refused");
        // Close Santiago's popover, which would win over Finish Assault's briefing.
        let current = app.observe()["observation"]["revision"].as_u64().unwrap();
        app.request(json!({"op":"key","revision":current,"key":"Escape"}), true);
        app.request(json!({"op":"settle"}), true);

        let action = "end-assault";
        let current = app.observe()["observation"]["revision"].as_u64().unwrap();
        app.activate(&format!("option:{action}"), current);
        let view = app.observe();
        let explanation = tag_text(&view["table"]["briefing"]);
        assert!(explanation.contains("earns no Pass Resources (local policy)"));
        let notes = preview_notes(&view, action);
        assert!(notes.contains("Records Operation Only"), "{notes}");
        assert!(!notes.contains("Available"), "{notes}");
        app.capture("zero-space-assault-preview");
        app.choose(action);
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
    for text in TEXTS {
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
        finish_event(&app.save, text);

        app.launch_size("3840x2160");
        app.choose("advance-card");
        let opening = app.observe()["observation"].clone();
        let revision = opening["revision"].as_u64().unwrap();
        let from = opening["resources"][0].as_u64().unwrap();
        assert_eq!(opening["kidnap"]["die"], 5);
        assert_eq!(from, 1);
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
            app.activate("option:government:transport", revision);
            let briefing = tag_text(&app.observe()["table"]["briefing"]);
            assert!(
                briefing.contains("Operation in at least one space (R 4.1)"),
                "{briefing}"
            );
            assert!(
                briefing.contains("cannot pay 2 Resources for an Operation"),
                "{briefing}"
            );
            app.capture("government-special-activity-briefing");
        }
        app.activate("option:government:pass", revision);
        let preview = app.observe();
        let briefing = tag_text(&preview["table"]["briefing"]);
        assert!(briefing.contains("Pass gains 3 Resources"), "{briefing}");
        assert!(briefing.contains(&format!("{from} → {to}")), "{briefing}");
        let notes = preview_notes(&preview, "government:pass");
        assert!(
            notes.contains("Syndicate now decides as 1st Eligible"),
            "{notes}"
        );
        assert!(!notes.contains("Available"), "{notes}");
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
        app.activate("option:syndicate:terror", revision + 1);
        let briefing = tag_text(&app.observe()["table"]["briefing"]);
        assert!(
            briefing.contains("No space has an Underground Syndicate Guerrilla"),
            "{briefing}"
        );
        assert!(briefing.contains("inspection only"), "{briefing}");
        app.capture("syndicate-terror-briefing");
        app.activate("option:syndicate:construct", revision + 1);
        let inspection = app.observe();
        let briefing = tag_text(&inspection["table"]["briefing"]);
        assert!(
            briefing.contains("next Eligible faction in card order"),
            "{briefing}"
        );
        assert!(briefing.contains("5 Resources"), "{briefing}");
        app.capture("syndicate-construct-briefing");
        assert_eq!(inspection["observation"], *after);
        assert_eq!(fs::read(&app.save).unwrap(), committed);
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

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Pass preview and result"]
fn native_syndicate_pass_finishes_card_18_and_replays_after_both_handoffs() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-216"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    for (assault, agent) in [(false, false), (true, true), (true, false)] {
        let name = match (assault, agent) {
            (true, true) => "after-assault-",
            (true, false) => "after-assault-human-",
            _ => "after-pass-",
        };
        let root = tempfile::Builder::new()
            .prefix(name)
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
        eprintln!("Syndicate Pass evidence: {}", app.root.display());
        if assault {
            super::assault::fund_santiago(&app.save, 4);
        } else {
            finish_event(&app.save, "unshaded");
        }
        app.launch_size("3840x2160");
        if assault {
            for action in ["event:shaded", "resolve-shaded"] {
                let revision = app.revision();
                app.activate(&format!("option:{action}"), revision);
                app.choose(action);
            }
        }
        app.choose("advance-card");
        if assault {
            let revision = app.revision();
            app.activate("option:government:assault", revision);
            app.choose("government:assault");
            app.choose("assault:havana:syndicate");
            let revision = app.revision();
            app.activate("option:end-assault", revision);
            app.choose("end-assault");
        } else {
            let revision = app.revision();
            app.activate("option:government:pass", revision);
            app.choose("government:pass");
        }
        let before = app.observe()["observation"].clone();
        let revision = before["revision"].as_u64().unwrap();
        let pass_id = format!("{revision}:syndicate:pass");
        let saved = fs::read(&app.save).unwrap();
        let role = if assault {
            "2nd Eligible"
        } else {
            "1st Eligible"
        };
        assert_eq!(before["next"]["role"], role);
        assert_eq!(before["resources"][3], 15);
        if assault && !agent {
            app.point("eligibility:operation-only");
            let observed = app.observe();
            let parts = observed["table"]["eligibility"]["parts"]
                .as_array()
                .unwrap();
            let part = |name| parts.iter().find(|p| p["part"] == name).unwrap();
            let note = part("eligibility-tag-note");
            assert_eq!(
                note["text"],
                "Card 18 Eligibility set after card 17 (R 2.3.7)."
            );
            let [l, t, r, b] = rect_of(&note["bounds"]);
            let tag = part("eligibility-tag");
            let [tl, tt, tr, tb] = rect_of(&tag["bounds"]);
            assert!(
                l > tl && t > tt && r < tr && b < tb && b - t < 30.,
                "{note}"
            );
            app.capture("government-action");

            app.activate("option:syndicate:limited-operation", revision);
            let observed = app.observe();
            assert_briefing_lines_fit(&observed);
            let options = choice_part(&observed, "briefing-value:2ND ELIGIBLE");
            assert_eq!(
                options["text"],
                "Government recorded Operation Only.\nSyndicate may Pass or take a Limited Operation."
            );
            app.capture("construct-briefing");
            fs::write(
                app.root.join("assault-handoff-copy-bounds.json"),
                serde_json::to_vec_pretty(&json!([tag, note, options])).unwrap(),
            )
            .unwrap();
        }
        app.activate("option:syndicate:pass", revision);
        let inspected = app.observe();
        let briefing = tag_text(&inspected["table"]["briefing"]);
        assert!(briefing.contains("15 → 16"), "{briefing}");
        assert!(briefing.contains("ELIGIBILITY FOR CARD 19"), "{briefing}");
        assert_eq!(inspected["observation"], before);
        assert_eq!(fs::read(&app.save).unwrap(), saved);
        app.capture("syndicate-pass-preview");
        if agent {
            let player = app.seat_agent(super::faction_seats::SYNDICATE, "syndicate");
            let (code, told) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{told}");
            assert_eq!(told["view"]["decision"]["yours"], true);
            let pass = told["view"]["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|choice| choice["id"] == pass_id)
                .unwrap();
            assert_eq!(pass["supported"], true);
            assert_eq!(pass["preview"]["resources"][0]["to"], 16);
            app.offers_nothing();
            app.key("Escape");
            app.decide(&player, "syndicate:pass", "applied");
            let (code, told) = app.ask(&["observe", &player]);
            assert_eq!(code, 0, "{told}");
            assert_eq!(told["view"]["decision"]["stands"], "card");
            assert_eq!(told["view"]["decision"]["yours"], false);
            let report = told["view"]["results"].as_array().unwrap().last().unwrap();
            assert_eq!(report["headline"], "Syndicate Passes");
            assert_eq!(report["role"], role);
            assert_eq!(report["complete"], true);
            assert_eq!(report["summary"], "Syndicate receives 1 Resource, 15 → 16.");
            let edition = app.observe();
            assert_eq!(edition["table"]["newspaper"]["open"]["report"], *report);
            fs::write(
                app.root.join("faction-client-after-pass.json"),
                serde_json::to_vec_pretty(&told).unwrap(),
            )
            .unwrap();
            app.capture("syndicate-pass-edition");
            app.key("Escape");
            app.decide(&player, "syndicate:pass", "refused");
        } else {
            let committed = app.press_by_keyboard(&pass_id);
            assert!(committed["table"]["newspaper"]["open"].is_null());
        }
        let table = app.observe();
        let after = &table["observation"];
        assert_eq!(after["revision"], revision + 1);
        assert_eq!(after["resources"][3], 16);
        assert_eq!(after["standings"][3]["score"], if assault { 2 } else { 3 });
        assert_eq!(after["eligible"], json!([!assault, true, true, true]));
        assert!(after["eligibility"]["event_restriction"].is_null());
        assert_card_advance_is_the_only_choice(after);
        assert_eq!(after["capabilities"], json!([]));
        assert_eq!(after["cards"]["current"]["status"], "Unexecuted");
        assert_eq!(
            choice_part(&table, "execution-heading")["text"],
            "Syndicate · passed"
        );
        let result = choice_part(&table, "ended-value:RESULT");
        assert_eq!(
            result["text"],
            "Syndicate Resources: 15 → 16. Pact Capability inactive."
        );
        let [_, top, _, bottom] = rect_of(&result["bounds"]);
        assert!(bottom - top < 40., "{result}");
        if assault && !agent {
            fs::write(
                app.root.join("pass-result-copy-bounds.json"),
                serde_json::to_vec_pretty(result).unwrap(),
            )
            .unwrap();
        }
        let track = &table["table"]["eligibility"]["track"];
        assert_eq!(track["adjusted"], true);
        let rows = track["rows"].as_array().unwrap();
        let statuses: Vec<_> = rows
            .iter()
            .map(|row| {
                (
                    row["faction"].as_str().unwrap(),
                    row["eligible"].as_bool().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            statuses,
            [
                ("M26", true),
                ("Directorio", true),
                ("Syndicate", true),
                ("Government", !assault)
            ]
        );
        for key in [
            "current_card",
            "preview_card",
            "draw_count",
            "deck",
            "played_pile",
            "spaces",
            "available",
        ] {
            assert_eq!(after[key], before[key], "{key}");
        }
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        let committed = fs::read(&app.save).unwrap();
        app.capture("syndicate-pass-complete");
        app.quit();

        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], *after);
        for (at, id) in [
            (revision, pass_id),
            (revision + 1, format!("{}:syndicate:pass", revision + 1)),
        ] {
            app.request(json!({"op":"activate","revision":at,"choice":id}), false);
        }
        assert_eq!(app.observe()["observation"], *after);
        assert_eq!(fs::read(&app.save).unwrap(), committed);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
        app.capture("syndicate-pass-restarted");
        app.quit();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the card 19 table and tags"]
fn native_manifesto_advance_lands_the_cards_and_reopens_the_same_decision() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-239"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("manifesto-advance-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let prepared = root.join("prepared.json");
    finish_event(&prepared, "shaded");
    let mut session = Session::open(&prepared, None).unwrap();
    for action in ["advance-card", "government:pass"] {
        let seen = session.observe();
        session
            .submit(
                seen.owner,
                seen.revision,
                &format!("{}:{action}", seen.revision),
            )
            .unwrap();
    }
    let seen = session.observe();
    session
        .publish_as_human(
            cuba_libre_unfolded::Faction::Syndicate,
            seen.revision,
            18,
            "We will wait for the next card.",
        )
        .unwrap();
    session
        .submit(
            seen.owner,
            seen.revision,
            &format!("{}:syndicate:pass", seen.revision),
        )
        .unwrap();
    drop(session);

    let saved = fs::read(&prepared).unwrap();
    let prior = fs::read(previous_path(&prepared)).unwrap();
    let mut app = advance_run(&root, "keyboard", &saved, &prior);
    assert_missing_or_changed_face_is_recoverable(&app, &saved, &prior, "card-20.png");
    app.launch_size("3840x2160");
    let ending = app.observe();
    let revision = ending["observation"]["revision"].as_u64().unwrap();
    card_piles::assert_piles(&ending, 19, 33, 18, 19);
    assert_card_advance_is_the_only_choice(&ending["observation"]);
    let label = choice_part(&ending, "advance-card-label");
    assert_eq!(label["text"], "Advance to Sierra Maestra Manifesto");
    let [_, top, _, bottom] = rect_of(&label["bounds"]);
    assert!(bottom - top < 40., "{label}");
    app.capture("card-18-advance-control");
    app.quit();

    let mut pointer = advance_run(&root, "pointer", &saved, &prior);
    pointer.launch_size("3840x2160");
    commit_advance(&mut pointer, revision);
    pointer.request(json!({"op":"settle"}), true);
    let clicked = pointer.observe();
    card_piles::assert_piles(&clicked, 20, 32, 19, 20);
    pointer.quit();

    app.launch_size("3840x2160");
    let advanced = app.press_by_keyboard(&format!("{revision}:advance-card"));
    assert_eq!(advanced["observation"]["revision"], revision + 1);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    app.request(json!({"op":"settle"}), true);
    let landed = app.observe();
    assert_eq!(landed["observation"], clicked["observation"]);
    card_piles::assert_piles(&landed, 20, 32, 19, 20);
    assert_eq!(landed["observation"]["owner"], "M26");
    assert_eq!(
        landed["observation"]["eligible"],
        json!([true, true, true, true])
    );
    assert_eq!(
        landed["table"]["notice"],
        "Sierra Maestra Manifesto is current. 26July is 1st Eligible. Saved."
    );
    assert!(
        choice_part(&landed, "choice-subtitle")["text"]
            .as_str()
            .unwrap()
            .contains("1st Eligible on card 19")
    );
    assert_eq!(
        choice_part(&landed, "section:event")["text"],
        "EVENT · CARD 19, SIERRA MAESTRA MANIFESTO"
    );
    app.capture("manifesto-landed");
    card_piles::assert_piles_drawn(&app, "manifesto-landed", &landed);

    for (slot, number, title, kind, status) in [
        (
            "card:current",
            19,
            "19 · Sierra Maestra Manifesto",
            "Single",
            "Event open to 26July as 1st Eligible",
        ),
        (
            "card:preview",
            20,
            "20 · The Twelve",
            "DualUse",
            "Event not yet open: this card is played after card 19",
        ),
    ] {
        app.activate(slot, revision + 1);
        let tagged = app.observe();
        assert_card_tag_placed(&tagged, slot);
        let shown = &tagged["table"]["card"]["shown"];
        assert_eq!(shown["number"], number);
        assert_eq!(shown["title"], title);
        let card = if number == 19 { "current" } else { "preview" };
        assert_eq!(tagged["observation"]["cards"][card]["kind"], kind);
        assert_eq!(
            tagged["observation"]["cards"][card]["words"]["status"],
            status
        );
        app.capture(&format!("card-19-tag-{number}"));
        close_card_tag(&mut app);
    }
    let played = landed["observation"]["played_pile"]["cards"]
        .as_array()
        .unwrap();
    assert_eq!(played[0]["title"], "Sierra Maestra Manifesto");
    assert_eq!(played[1]["title"], "Pact of Caracas");
    assert_eq!(landed["observation"]["deck"]["face_down"], 31);

    app.point("pile:played");
    let shown = app.observe();
    let played = &shown["table"]["pile_tag"]["shown"];
    assert_eq!(played["ledger"][0][1], "20 · current card on top");
    let columns = played["columns"].as_array().unwrap();
    assert_eq!(
        columns
            .iter()
            .map(|c| c.as_array().unwrap().len())
            .collect::<Vec<_>>(),
        [10, 10]
    );
    assert_eq!(columns[0][0], "19 · Sierra Maestra Manifesto · current");
    assert_eq!(columns[0][1], "18 · Pact of Caracas");
    app.capture("manifesto-played-history");

    app.point("pile:deck");
    let shown = app.observe();
    let deck = &shown["table"]["pile_tag"]["shown"];
    assert_eq!(deck["ledger"][0][1], "32 · counts the preview");
    assert_eq!(deck["ledger"][1][1], "31");
    app.capture("manifesto-draw-deck");

    let saved_after = app.save_bytes();
    for action in ["m26:event", "pass", "terror"] {
        app.activate(&format!("option:{action}"), revision + 1);
        let inspect = app.observe();
        assert_eq!(inspect["table"]["choice"]["pinned"], action);
        assert_eq!(inspect["observation"], landed["observation"]);
        assert_briefing_lines_fit(&inspect);
        app.capture(&format!("manifesto-option-{}", action.replace(':', "-")));
        assert_eq!(
            inspect["table"]["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["name"] == format!("{}:m26:event", revision + 1)),
            action == "m26:event"
        );
        assert_eq!(app.save_bytes(), saved_after);
    }
    app.key("Escape");
    app.key("KeyR");
    let read = app.observe();
    let history = &read["table"]["proclamations"];
    assert_eq!(history["documents"][0]["card"], 18);
    assert_eq!(
        history["documents"][0]["text"],
        "We will wait for the next card."
    );
    assert_eq!(history["unread"], 0);
    app.capture("manifesto-carried-proclamation");
    app.key("Escape");

    app.quit();
    app.launch_size("3840x2160");
    let resumed = app.observe();
    assert_eq!(resumed["observation"], landed["observation"]);
    card_piles::assert_piles(&resumed, 20, 32, 19, 20);
    assert!(resumed["table"]["piles"]["advance"].is_null());
    assert_eq!(
        resumed["table"]["proclamations"]["documents"],
        history["documents"]
    );
    app.capture("manifesto-resumed-at-rest");
    app.activate("option:m26:event", revision + 1);
    app.choose("m26:event");
    let pending = app.observe();
    assert_eq!(pending["observation"]["owner"], "M26");
    assert_eq!(pending["observation"]["executing"], "M26");
    assert_eq!(pending["observation"]["event_executor"], "M26");
    assert_eq!(pending["observation"]["manifesto"]["declines"], json!([]));
    app.capture("manifesto-executed-by-pointer");
}

fn preview_notes(reply: &Value, action: &str) -> String {
    let view = &reply["observation"];
    let id = format!("{}:{action}", view["revision"]);
    view["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|choice| choice["id"] == id)
        .unwrap()["preview"]["notes"]
        .to_string()
}

/// Plays the Alberto Bayo Event's `text` to card end from the opening save at `path`,
/// or from a new save there with seed 2. Seed 2 rolls the Kidnap die 5, which leaves
/// Government 1 Resource (R 4.3.3), too few to pay for an Assault space on card 18.
pub(super) fn finish_event(path: &Path, text: &str) {
    let mut session = if path.exists() {
        Session::open(path, None)
    } else {
        Session::create_seeded(path, 2)
    }
    .unwrap();
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

pub(super) fn assert_card_advance_is_the_only_choice(view: &Value) {
    let choices = view["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(
        choices[0]["id"],
        format!("{}:advance-card", view["revision"])
    );
    assert_eq!(choices[0]["supported"], true);
}

fn assert_missing_or_changed_face_is_recoverable(
    app: &Native,
    saved: &[u8],
    prior: &[u8],
    face: &str,
) {
    let wrong_cards = tempfile::tempdir().unwrap();
    for name in [
        "card-17.png",
        "card-18.png",
        "card-19.png",
        "card-20.png",
        "cuba-libre-card-back.png",
    ] {
        if name != face {
            fs::copy(app.cards.join(name), wrong_cards.path().join(name)).unwrap();
        }
    }
    let fresh = app.root.join("never-created.json");
    for problem in ["missing", "changed"] {
        if problem == "changed" {
            let mut bytes = fs::read(app.cards.join(face)).unwrap();
            bytes[100] ^= 1;
            fs::write(wrong_cards.path().join(face), bytes).unwrap();
        }
        for (save, kind) in [(&app.save, "existing"), (&fresh, "fresh")] {
            let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
                .arg("--cards")
                .arg(wrong_cards.path())
                .arg("--save")
                .arg(save)
                .arg("--agent-seat")
                .arg("--socket")
                .arg(&app.socket)
                .output()
                .unwrap();
            fs::write(
                app.root.join(format!("{problem}-{face}-{kind}.stderr")),
                &output.stderr,
            )
            .unwrap();
            let reason = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "{problem} {face} was accepted");
            assert!(
                reason.contains(face) && reason.contains("--cards"),
                "{reason}"
            );
        }
        assert_eq!(fs::read(&app.save).unwrap(), saved);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), prior);
        assert!(!fresh.exists());
        assert!(!previous_path(&fresh).exists());
    }
}

/// One run of the card advance regression: its own copy of the save as it stood on
/// card 17 with its recovery record, launched at the owner's display size.
fn advance_run(root: &Path, name: &str, saved: &[u8], prior: &[u8]) -> NativeRun {
    let socket = tempfile::tempdir().unwrap();
    let app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        save: root.join(format!("{name}.json")),
        socket: socket.path().join("seat.sock"),
        root: root.join(name),
        step: 0,
    };
    fs::create_dir_all(&app.root).unwrap();
    fs::write(&app.save, saved).unwrap();
    fs::write(previous_path(&app.save), prior).unwrap();
    NativeRun {
        app,
        _socket: socket,
    }
}

/// Commits the advance and returns the reply, without waiting for the cards.
fn commit_advance(app: &mut Native, revision: u64) -> Value {
    let done = app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:advance-card")}),
        true,
    );
    assert_eq!(done["status"], "activated", "{done}");
    done
}

/// Steps the seat's clock until the advance is `seconds` old, to the nearest step,
/// and returns what the table shows then. The table is still unsettled.
fn advance_to(app: &mut Native, seconds: f64) -> Value {
    let elapsed = app.observe()["table"]["piles"]["advance"]["elapsed"]
        .as_f64()
        .expect("a card is in the air");
    let steps = ((seconds - elapsed) * 60.).round() as u32;
    assert!(steps > 0, "the advance is already {elapsed} s old");
    let held = app.request(json!({"op":"settle","steps":steps}), true);
    assert_eq!(held["status"], "unsettled", "{held}");
    let view = app.observe();
    let now = view["table"]["piles"]["advance"]["elapsed"]
        .as_f64()
        .unwrap();
    assert!((now - seconds).abs() < 0.009, "{now} s, not {seconds} s");
    assert_eq!(view["table"]["settled"], false);
    assert_eq!(view["table"]["pending"], json!(["card advance"]));
    view
}

/// The piles as the tray draws them and the numbers over them, while a card is in
/// the air: each pile's cards and what lies on top. The count discs are not shown then.
fn assert_cards_shown(view: &Value, played: (u64, Value), deck: (u64, Value), faces: u64) {
    let piles = &view["table"]["piles"];
    let drawn = piles["piles"].as_array().unwrap();
    assert_eq!(drawn.len(), 2, "{piles}");
    for (pile, (cards, top)) in drawn.iter().zip([played, deck]) {
        assert_eq!(pile["cards"], cards, "{pile}");
        assert_eq!(pile["top"], top, "{pile}");
    }
    assert_eq!(piles["faces"], faces, "{piles}");
    assert_eq!(piles["pips"], json!([true, false, false, false]));
    // No count disc is shown while a card is in the air: the cards pass where the discs
    // lie, and looked as if they cut through them (the owner, 2026-11-23, #146).
    let parts = card_piles::parts(view);
    for part in [
        "played-count-disc",
        "played-count",
        "deck-count-disc",
        "deck-count",
    ] {
        assert!(
            !parts.contains_key(part),
            "{part} is shown while a card is in the air"
        );
    }
    assert_eq!(piles["discs"], json!([]), "{piles}");
    assert_eq!(parts["propaganda-caption"].0, "1 of 4 played");
}

/// Where two rectangles overlap, 4 px in from the overlap's edges, if they do.
fn overlap(a: [f64; 4], b: [f64; 4]) -> Option<[f64; 4]> {
    let both = [
        a[0].max(b[0]) + 4.,
        a[1].max(b[1]) + 4.,
        a[2].min(b[2]) - 4.,
        a[3].min(b[3]) - 4.,
    ];
    (both[2] > both[0] && both[3] > both[1]).then_some(both)
}

/// How many 6 px blocks of `region` in a capture differ from the card face drawn at
/// `card` by more than a letter's ink would leave unnoticed: 40 levels in the mean of
/// the block. The face drawn alone stays well within that.
fn inked_blocks(
    capture: &image::RgbImage,
    region: [f64; 4],
    card: [f64; 4],
    face: &image::RgbImage,
) -> usize {
    let (width, height) = (card[2] - card[0], card[3] - card[1]);
    let half = (3. * f64::from(face.width()) / width).max(1.) as u32;
    let mut inked = 0;
    let mut worst: f64 = 0.;
    let mut y = region[1] + 3.;
    while y <= region[3] - 3. {
        let mut x = region[0] + 3.;
        while x <= region[2] - 3. {
            let shown = card_piles::block(capture, [x, y], 3);
            let printed = card_piles::block(
                face,
                [
                    (x - card[0]) / width * f64::from(face.width()),
                    (y - card[1]) / height * f64::from(face.height()),
                ],
                half,
            );
            let apart = shown
                .iter()
                .zip(printed)
                .map(|(s, p)| (s - p).abs())
                .sum::<f64>()
                / 3.;
            worst = worst.max(apart);
            if apart > 40. {
                inked += 1;
            }
            x += 6.;
        }
        y += 6.;
    }
    eprintln!("region {region:?}: {inked} inked blocks, the worst {worst:.1} levels apart");
    inked
}

/// The card advance is saved once and then shown (#145; ADR 0001; R 2.3.8). From the
/// save as it stands on card 17, with 18 cards played and 34 in the deck:
///
/// - the commit writes the save before a card has moved, and nothing writes it again;
/// - the preview, card 18, lifts off the deck and crosses to the played pile while
///   the deck shows the original back; the deck lies at 33 cards from the moment it
///   leaves, and at 32 under the card that then turns over, and the played pile at 19
///   from the moment it lands; neither count disc is shown until the cards are at rest;
/// - card 19 then turns over where it lies, its back showing until it is on edge;
/// - settled, the piles are 19 with card 18 and 33 with card 19, in every run;
/// - a click, Space and Enter each land that state at once and do nothing else;
/// - a restart during the motion shows the cards at rest with the advance made once,
///   and the recovery record is the exact save from before it;
/// - a missing or altered card back refuses launch, naming the file, and changes
///   neither the save nor its recovery record.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained frames of the four steps"]
fn native_card_advance_moves_the_cards_and_lands_saved() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-145"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("card-advance-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    eprintln!("card advance evidence: {}", root.display());
    // The position: the shaded Event played to card end.
    let source = root.join("card-17.json");
    finish_event(&source, "shaded");
    let saved = fs::read(&source).unwrap();
    let prior = fs::read(previous_path(&source)).unwrap();
    let back = image::open(
        PathBuf::from(std::env::var_os("CUBA_TEST_CARDS").unwrap())
            .join("cuba-libre-card-back.png"),
    )
    .unwrap()
    .to_rgb8();

    let mut app = advance_run(&root, "shown", &saved, &prior);
    assert_missing_or_changed_back_refuses(&app, &saved, &prior);

    // Step 1: at rest on card 17.
    app.launch_size("3840x2160");
    app.request(json!({"op":"settle"}), true);
    let before = app.observe();
    let revision = before["observation"]["revision"].as_u64().unwrap();
    card_piles::assert_piles(&before, 18, 34, 17, 18);
    assert_eq!(before["table"]["piles"]["advance"], Value::Null);
    app.capture("1-rest-card-17");
    card_piles::assert_piles_drawn(&app, "1-rest-card-17", &before);
    let faces: Vec<[f64; 4]> = before["table"]["piles"]["piles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pile| rect_of(&pile["face"]))
        .collect();
    let (played_face, deck_face) = (faces[0], faces[1]);

    // The commit: saved before anything has moved.
    commit_advance(&mut app, revision);
    let advanced = fs::read(&app.save).unwrap();
    assert_ne!(advanced, saved, "the advance is saved as it is applied");
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    let committed = Session::open(&app.save, None).unwrap().observe();
    assert_eq!(committed.revision, revision + 1);
    assert_eq!((committed.current_card, committed.preview_card), (18, 19));
    let leaving = app.observe();
    let after = leaving["observation"].clone();
    assert_eq!(after, serde_json::to_value(&committed).unwrap());
    let flying = &leaving["table"]["piles"]["advance"];
    assert_eq!(flying["card"], 18, "{flying}");
    assert_eq!(flying["step"], "crossing", "{flying}");
    assert_eq!(flying["showing"], "face", "{flying}");
    let started = flying["elapsed"].as_f64().unwrap();
    assert!(started < 0.1, "the motion starts at the commit: {flying}");
    // Still over the deck: it has moved less than a tenth of the way in that time.
    let at = rect_of(&flying["bounds"]);
    assert!(
        at[0] <= deck_face[0] + 1. && at[0] > deck_face[0] - 36. && at[2] > deck_face[2] - 36.,
        "{at:?}"
    );
    assert_eq!(leaving["table"]["pending"], json!(["card advance"]));
    // The deck is a card thinner as the card leaves it; the played pile waits.
    assert_cards_shown(&leaving, (18, json!(17)), (33, json!("back")), 1);
    // A seat that connects now finds the same moment, not a new start: every request
    // here is its own connection, and only the seat's steps move the clock.
    let again = app.observe();
    assert_eq!(again["table"]["piles"], leaving["table"]["piles"]);
    // Anything but the landing is refused while the card is in the air.
    let tab = app.request(json!({"op":"key","revision":revision+1,"key":"Tab"}), false);
    assert!(
        tab["error"]
            .as_str()
            .unwrap()
            .starts_with("The table is busy (card advance)"),
        "{tab}"
    );
    let choice = format!("{}:government:pass", revision + 1);
    let early = app.request(
        json!({"op":"activate","revision":revision+1,"choice":choice}),
        false,
    );
    assert!(
        early["error"].as_str().unwrap().contains("card advance"),
        "{early}"
    );

    // Step 2, halfway: the card is between the piles and 30 mm up, which the tray's
    // view shows 73.6 px toward the top of the screen and 0.76% larger (#145's
    // worked view: 4 m above the cards, the lens 2 m in front of their near edge).
    let crossing = advance_to(&mut app, 0.4);
    let flying = &crossing["table"]["piles"]["advance"];
    assert_eq!(flying["card"], 18, "{flying}");
    assert_eq!(flying["step"], "crossing", "{flying}");
    let at = rect_of(&flying["bounds"]);
    let middle = (played_face[0] + deck_face[2]) / 2.;
    assert!(((at[0] + at[2]) / 2. - middle).abs() < 3., "{at:?}");
    let rise = (played_face[1] + played_face[3]) / 2. - (at[1] + at[3]) / 2.;
    assert!((rise - 73.6).abs() < 3., "the card shows {rise} px up");
    assert!((at[2] - at[0] - 300. * 1.0076).abs() < 2.5, "{at:?}");
    assert_cards_shown(&crossing, (18, json!(17)), (33, json!("back")), 1);
    app.capture("2-crossing-halfway");
    // The card in the air is above the table, so it draws over the labels it passes,
    // as the advance sheet draws it: where it covers "CURRENT CARD" and "PREVIEW" the
    // frame shows card 18's face and no letter (#137's review).
    let shot = image::open(app.root.join("2-crossing-halfway.png"))
        .unwrap()
        .to_rgb8();
    let face_18 = image::open(app.cards.join("card-18.png"))
        .unwrap()
        .to_rgb8();
    let parts = card_piles::parts(&crossing);
    let mut covered = 0;
    for label in ["played-label", "deck-label"] {
        let under = overlap(parts[label].1, at);
        if let Some(under) = under {
            covered += 1;
            let inked = inked_blocks(&shot, under, at, &face_18);
            assert_eq!(
                inked, 0,
                "{label} shows over the card in the air ({under:?})"
            );
        }
    }
    assert_eq!(covered, 2, "the card at halfway covers both labels: {at:?}");
    // Step 2, nearly there: the card is clear of the deck, which shows the back.
    let arriving = advance_to(&mut app, 0.7);
    app.capture("2-crossing-arriving");
    let shot = image::open(app.root.join("2-crossing-arriving.png"))
        .unwrap()
        .to_rgb8();
    let own = card_piles::face_distance(&shot, deck_face, &back);
    assert!(own < 8., "the deck is {own} levels from the card back");
    assert_eq!(arriving["table"]["piles"]["advance"]["card"], 18);

    // Step 3: card 18 has landed and counts; card 19 turns over, back first.
    let turning = advance_to(&mut app, 1.0);
    let flying = &turning["table"]["piles"]["advance"];
    assert_eq!(flying["card"], 19, "{flying}");
    assert_eq!(flying["step"], "turning", "{flying}");
    assert_eq!(flying["showing"], "back", "{flying}");
    assert_cards_shown(&turning, (19, json!(18)), (32, json!("back")), 1);
    // It turns where it lies: its middle is the deck's, and it is narrower than a
    // card lying flat.
    let at = rect_of(&flying["bounds"]);
    assert!(
        ((at[0] + at[2]) / 2. - (deck_face[0] + deck_face[2]) / 2.).abs() < 4.,
        "{at:?}"
    );
    assert!(at[2] - at[0] < 290., "{at:?}");
    let tray = rect_of(&turning["table"]["piles"]["tray"]);
    app.capture("3-turning-back");
    let shot = image::open(app.root.join("3-turning-back.png"))
        .unwrap()
        .to_rgb8();
    let own = card_piles::face_distance(&shot, played_face, &{
        image::open(app.cards.join("card-18.png"))
            .unwrap()
            .to_rgb8()
    });
    assert!(own < 8., "the played pile is {own} levels from card 18");
    // On its edge, and then past it with the face showing; never outside the tray.
    let upright = advance_to(&mut app, 0.8 + 0.35);
    let at = rect_of(&upright["table"]["piles"]["advance"]["bounds"]);
    assert!(
        at[1] > tray[1] + 8. && at[3] < tray[3],
        "{at:?} in {tray:?}"
    );
    app.capture("3-turning-on-edge");
    let facing = advance_to(&mut app, 1.3);
    let flying = &facing["table"]["piles"]["advance"];
    assert_eq!(flying["card"], 19, "{flying}");
    assert_eq!(flying["showing"], "face", "{flying}");
    app.capture("3-turning-face");

    // Step 4: at rest on card 18. Nothing was saved after the commit.
    app.request(json!({"op":"settle"}), true);
    let rest = app.observe();
    card_piles::assert_piles(&rest, 19, 33, 18, 19);
    assert_eq!(rest["table"]["piles"]["advance"], Value::Null);
    assert_eq!(rest["observation"], after);
    app.capture("4-rest-card-18");
    card_piles::assert_piles_drawn(&app, "4-rest-card-18", &rest);
    assert_eq!(fs::read(&app.save).unwrap(), advanced);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    app.quit();

    // A click, Space and Enter each land the same state at once and act on nothing.
    // The owner's click lands the cards wherever it falls: on the card area, on a
    // space of the map, which it does not pick, and on a decision's control, which it
    // does not pin.
    for (by, at) in [
        ("click", 0.3),
        ("map", 0.5),
        ("decision", 1.2),
        ("space", 1.1),
        ("enter", 0.3),
    ] {
        let mut app = advance_run(&root, &format!("skip-{by}"), &saved, &prior);
        app.launch_size("3840x2160");
        app.request(json!({"op":"settle"}), true);
        commit_advance(&mut app, revision);
        let held = advance_to(&mut app, at);
        assert!(held["table"]["piles"]["advance"].is_object());
        app.capture("in-the-air");
        let click = |component: &str| {
            (
                json!({"op":"activate","revision":revision+1,"component":component}),
                "activated",
            )
        };
        let (skip, status) = match by {
            "click" => click("area:cards"),
            "map" => click("space:havana"),
            "decision" => click("option:government:pass"),
            "space" => (
                json!({"op":"key","revision":revision+1,"key":"Space","focused":"card-advance"}),
                "pressed",
            ),
            _ => (
                json!({"op":"key","revision":revision+1,"key":"Enter","focused":"card-advance"}),
                "pressed",
            ),
        };
        let skipped = app.request(skip, true);
        assert_eq!(skipped["status"], status, "{by}: {skipped}");
        // At once: the reply's own table has the cards down, before any settling.
        let landed = app.observe();
        assert_eq!(landed["table"]["piles"]["advance"], Value::Null, "{by}");
        assert_eq!(
            landed["table"]["piles"]["piles"], rest["table"]["piles"]["piles"],
            "{by}"
        );
        // And nothing else: no space picked, no option pinned, no tag opened or closed.
        for what in ["choice", "popover", "briefing", "card", "pile_tag"] {
            assert_eq!(
                landed["table"][what], held["table"][what],
                "{by} changed table.{what}"
            );
        }
        assert_eq!(landed["observation"], after, "{by}");
        app.request(json!({"op":"settle"}), true);
        let landed = app.observe();
        card_piles::assert_piles(&landed, 19, 33, 18, 19);
        assert_eq!(landed["table"]["piles"], rest["table"]["piles"], "{by}");
        assert_eq!(landed["observation"], after, "{by}: a skip decides nothing");
        assert_eq!(landed["table"]["choice"]["pinned"], Value::Null, "{by}");
        assert_eq!(landed["table"]["card"]["shown"], Value::Null, "{by}");
        assert_eq!(fs::read(&app.save).unwrap(), advanced, "{by}");
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved, "{by}");
        app.capture("landed");
        card_piles::assert_piles_drawn(&app, "landed", &landed);
        app.quit();
    }

    // A card's tag follows the cards as the pile tags do (#137's review): the model is
    // on card 18 from the commit, while the table shows card 17 current and the
    // deck's back until card 19 has turned. A tag pinned before the advance closes for
    // the motion and is back, still pinned, once the cards have landed, on the card
    // that then lies there.
    let mut app = advance_run(&root, "card-tags", &saved, &prior);
    app.launch_size("3840x2160");
    app.request(json!({"op":"settle"}), true);
    app.activate("card:preview", revision);
    app.point("area:propaganda");
    let pinned = app.observe();
    let tag = &pinned["table"]["card"]["shown"];
    assert_eq!(
        (tag["card"].clone(), tag["pinned"].clone()),
        (json!("card:preview"), json!(true)),
        "{tag}"
    );
    assert_eq!(tag["number"], 18, "{tag}");
    commit_advance(&mut app, revision);
    assert_eq!(app.observe()["table"]["card"]["shown"], Value::Null);
    for at in [0.4, 1.0, 1.3] {
        let held = advance_to(&mut app, at);
        assert_eq!(held["table"]["card"]["shown"], Value::Null, "at {at} s");
    }
    app.capture("no-card-tag-in-the-air");
    app.request(json!({"op":"settle"}), true);
    let landed = app.observe();
    card_piles::assert_piles(&landed, 19, 33, 18, 19);
    let tag = &landed["table"]["card"]["shown"];
    assert_eq!(
        (tag["card"].clone(), tag["pinned"].clone()),
        (json!("card:preview"), json!(true)),
        "{tag}"
    );
    assert_eq!(tag["number"], 19, "{tag}");
    assert_eq!(tag["title"], "19 · Sierra Maestra Manifesto", "{tag}");
    app.capture("card-tag-landed");
    assert_eq!(fs::read(&app.save).unwrap(), advanced);
    app.quit();

    // A pile's tag states the model's piles, which are a card ahead of the piles on
    // the table while a card is in the air. An open tag closes for the motion and is
    // back as the cards land, with the numbers their discs and the pips then read.
    let mut app = advance_run(&root, "pile-tags", &saved, &prior);
    app.launch_size("3840x2160");
    app.request(json!({"op":"settle"}), true);
    app.point("pile:deck");
    let open = app.observe();
    let tag = &open["table"]["pile_tag"]["shown"];
    assert_eq!(tag["pile"], "pile:deck", "{tag}");
    assert_eq!(
        tag["ledger"][0][1],
        open["observation"]["deck"]["words"]["cards"]
    );
    assert!(
        tag["ledger"][0][1].as_str().unwrap().starts_with("34"),
        "{tag}"
    );
    // The keyboard commits the advance while the pointer stays on the deck's disc.
    let advance = format!("{revision}:advance-card");
    app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
    app.request(json!({"op":"settle"}), true);
    assert_eq!(app.observe()["table"]["choice"]["focus"], advance);
    let pressed = app.request(
        json!({"op":"key","revision":revision,"key":"Enter","focused":advance}),
        true,
    );
    assert_eq!(pressed["status"], "pressed", "{pressed}");
    let flying = app.observe();
    assert_eq!(flying["observation"], after);
    assert_eq!(flying["table"]["piles"]["advance"]["card"], 18);
    assert_eq!(flying["table"]["pile_tag"]["shown"], Value::Null);
    let held = advance_to(&mut app, 0.5);
    assert_eq!(held["table"]["pile_tag"]["shown"], Value::Null);
    app.capture("no-tag-in-the-air");
    // Landed by Space: the deck's tag is back in the reply, counting 33.
    app.request(
        json!({"op":"key","revision":revision+1,"key":"Space","focused":"card-advance"}),
        true,
    );
    let landed = app.observe();
    let tag = &landed["table"]["pile_tag"]["shown"];
    assert_eq!(tag["pile"], "pile:deck", "{tag}");
    assert_eq!(tag["ledger"][0][1], after["deck"]["words"]["cards"]);
    assert!(
        tag["ledger"][0][1].as_str().unwrap().starts_with("33"),
        "{tag}"
    );
    app.request(json!({"op":"settle"}), true);
    let landed = app.observe();
    card_piles::assert_piles(&landed, 19, 33, 18, 19);
    assert_eq!(landed["observation"], after);
    app.capture("deck-tag-landed");
    // And the played pile's: 19 cards and one of four Propaganda cards played.
    app.point("pile:played");
    let shown = app.observe();
    let tag = &shown["table"]["pile_tag"]["shown"];
    assert_eq!(tag["pile"], "pile:played", "{tag}");
    assert!(
        tag["ledger"][0][1].as_str().unwrap().starts_with("19"),
        "{tag}"
    );
    assert_eq!(tag["ledger"][1][1], "1 of 4 played", "{tag}");
    assert_eq!(fs::read(&app.save).unwrap(), advanced);
    app.quit();

    // A restart while the card is in the air: the cards at rest, one advance, and
    // the exact save from before it as the recovery record.
    let mut app = advance_run(&root, "restart", &saved, &prior);
    app.launch_size("3840x2160");
    app.request(json!({"op":"settle"}), true);
    commit_advance(&mut app, revision);
    advance_to(&mut app, 0.35);
    app.capture("in-the-air");
    app.quit();
    assert_eq!(fs::read(&app.save).unwrap(), advanced);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    app.launch_size("3840x2160");
    let resumed = app.observe();
    assert_eq!(resumed["observation"], after);
    assert_eq!(resumed["table"]["piles"]["advance"], Value::Null);
    assert!(
        !resumed["table"]["pending"]
            .as_array()
            .unwrap()
            .contains(&json!("card advance")),
        "{}",
        resumed["table"]["pending"]
    );
    app.request(json!({"op":"settle"}), true);
    let resumed = app.observe();
    card_piles::assert_piles(&resumed, 19, 33, 18, 19);
    assert_eq!(resumed["table"]["piles"], rest["table"]["piles"]);
    assert!(
        resumed["observation"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|choice| !choice["id"].as_str().unwrap().ends_with(":advance-card")),
        "no second advance is offered"
    );
    for request in [
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:advance-card")}),
        json!({"op":"activate","revision":revision+1,"choice":format!("{}:advance-card", revision+1)}),
    ] {
        app.request(request, false);
    }
    assert_eq!(app.observe()["observation"], after);
    assert_eq!(fs::read(&app.save).unwrap(), advanced);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    app.capture("resumed-at-rest");
    card_piles::assert_piles_drawn(&app, "resumed-at-rest", &resumed);
    app.quit();
}

/// A card directory with every face but no card back, and then with a back one bit
/// off, refuses launch. The refusal names the file and how to supply it; no save is
/// created, and an existing save and its recovery record keep their bytes.
fn assert_missing_or_changed_back_refuses(app: &Native, saved: &[u8], prior: &[u8]) {
    const BACK: &str = "cuba-libre-card-back.png";
    let wrong_cards = tempfile::tempdir().unwrap();
    for name in ["card-17.png", "card-18.png", "card-19.png", "card-20.png"] {
        fs::copy(app.cards.join(name), wrong_cards.path().join(name)).unwrap();
    }
    let fresh = app.root.join("never-created.json");
    for problem in ["missing", "changed"] {
        if problem == "changed" {
            let mut bytes = fs::read(app.cards.join(BACK)).unwrap();
            bytes[100] ^= 1;
            fs::write(wrong_cards.path().join(BACK), bytes).unwrap();
        }
        for save in [&app.save, &fresh] {
            let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
                .arg("--cards")
                .arg(wrong_cards.path())
                .arg("--save")
                .arg(save)
                .arg("--agent-seat")
                .arg("--socket")
                .arg(&app.socket)
                .output()
                .unwrap();
            fs::write(
                app.root.join(format!("{problem}-back.stderr")),
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
            assert!(
                !output.status.success(),
                "a {problem} card back was accepted"
            );
            assert!(
                reason.contains(BACK) && reason.contains("--cards") && reason.contains("c905d7a7"),
                "{reason}"
            );
        }
        assert_eq!(fs::read(&app.save).unwrap(), saved);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), prior);
        assert!(!fresh.exists() && !previous_path(&fresh).exists());
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the Manifesto handoffs, editions and next menu"]
fn native_manifesto_declines_wake_held_syndicate_and_resume_in_order() {
    use super::faction_seats::{M26, SYNDICATE};
    use cuba_libre_unfolded::Faction;

    let mut app = super::available_forces::unlaunched("manifesto-declines-", "issue-240");
    finish_event(&app.save, "shaded");
    let mut session = Session::open(&app.save, None).unwrap();
    for action in [
        "advance-card",
        "government:pass",
        "syndicate:construct",
        "construct:camaguey-city:place",
        "end-construct",
    ] {
        let seen = session.observe();
        session
            .submit(
                seen.owner,
                seen.revision,
                &format!("{}:{action}", seen.revision),
            )
            .unwrap();
        if action == "advance-card" {
            session
                .publish_as_human(
                    Faction::M26,
                    seen.revision + 1,
                    18,
                    "We will wait for the next card.",
                )
                .unwrap();
        }
    }
    drop(session);
    app.launch_size("3840x2160");
    app.choose("advance-card");
    let m26 = app.seat_agent(M26, "26july");
    let before = app.observe()["observation"].clone();
    let at = before["revision"].as_u64().unwrap().to_string();
    let (code, fresh) = app.ask(&["observe", &m26]);
    assert_eq!(
        (code, &fresh["view"]["proclamations"]["can_publish"]),
        (0, &json!(true))
    );
    assert_eq!(fresh["view"]["proclamations"]["published"][0]["card"], 18);
    let (code, published) = app.ask(&["publish", &m26, &at, "19", "The Manifesto is before us."]);
    assert_eq!(
        (code, &published["status"]),
        (0, &json!("published")),
        "{published}"
    );
    assert_eq!(published["view"]["proclamations"]["can_publish"], false);
    assert_eq!(app.observe()["observation"], before);
    let words = published["view"]["proclamations"]["published"].clone();
    assert_eq!(words[1]["card"], 19);
    assert_eq!(words[1]["author"], "M26");
    assert_eq!(app.observe()["table"]["proclamations"]["open"], 2);
    app.capture("manifesto-publication-on-card-19");
    app.key("Escape");

    app.decide(&m26, "m26:event", "applied");
    let m26_turn = app.observe();
    assert_eq!(m26_turn["observation"]["owner"], "M26");
    assert_eq!(m26_turn["observation"]["event_executor"], "M26");
    assert_eq!(
        choice_part(&m26_turn, "choice-heading")["text"],
        "Sierra Maestra Manifesto"
    );
    assert_eq!(
        choice_part(&m26_turn, "choice-subtitle")["text"],
        "26July executes the Event.\n26July decides whether to place."
    );
    assert!(m26_turn["table"]["newspaper"]["open"].is_null());
    app.offers_nothing();
    app.capture("manifesto-26july-decision");
    app.decide(&m26, "manifesto:decline", "applied");

    let directorio_turn = app.observe();
    assert_eq!(directorio_turn["observation"]["owner"], "Directorio");
    let first_edition = &directorio_turn["table"]["newspaper"]["open"]["report"];
    assert_eq!(first_edition["faction"], "M26");
    assert_eq!(first_edition["complete"], false);
    assert_eq!(
        manifesto_articles(first_edition),
        ["26July declined placement."]
    );
    app.capture("manifesto-edition-for-directorio");
    app.key("Escape");
    let pending = app.observe()["observation"].clone();
    let pending_bytes = app.save_bytes();
    let previous_bytes = fs::read(previous_path(&app.save)).unwrap();
    app.quit();

    app.launch_size("3840x2160");
    let returned = app.observe();
    assert_eq!(returned["observation"], pending);
    assert_eq!(app.save_bytes(), pending_bytes);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous_bytes);
    assert!(returned["table"]["newspaper"]["open"].is_null());
    let documents = returned["table"]["proclamations"]["documents"]
        .as_array()
        .unwrap();
    assert_eq!(documents.len(), 2);
    for (document, published) in documents.iter().zip(words.as_array().unwrap()) {
        for key in ["card", "author", "text"] {
            assert_eq!(document[key], published[key]);
        }
        assert_eq!(document["read"], true);
    }
    app.capture("manifesto-restarted-for-directorio");

    let m26 = app.seat_agent(M26, "26july");
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let prior_revision = app.revision();
    let (code, spent) = app.ask(&["observe", &m26]);
    assert_eq!(code, 0);
    assert_eq!(spent["view"]["proclamations"]["published"], words);
    assert_eq!(spent["view"]["proclamations"]["can_publish"], false);
    let (code, refused) = app.ask(&["publish", &m26, &prior_revision.to_string(), "19", "Again."]);
    assert_eq!((code, &refused["status"]), (1, &json!("refused")));
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("already published")
    );
    assert_eq!(app.save_bytes(), pending_bytes);

    assert_eq!(spent["view"]["state"]["event_executor"], "M26");
    let mut waiting = app.player("syndicate-wait", &["wait", &syndicate]);
    app.until_waiting("Syndicate");
    assert!(waiting.ended().is_none());
    assert_eq!(returned["observation"]["eligible"][3], false);
    app.activate("space:camaguey-province", prior_revision);
    let inspect = app.observe();
    assert!(
        inspect["table"]["popover"]["card"]["subtitle"]
            .as_str()
            .unwrap()
            .contains("Directorio has 1 Underground Guerrilla")
    );
    app.capture("manifesto-placement-inspection");
    assert_eq!(app.save_bytes(), pending_bytes);
    assert!(waiting.ended().is_none());
    app.key("Escape");
    app.activate("option:manifesto:decline", prior_revision);
    let preview = app.observe();
    assert_eq!(
        choice_part(&preview, "choice-subtitle")["text"],
        "26July executes the Event.\nDirectorio decides whether to place."
    );
    assert!(
        tag_text(&preview["table"]["briefing"]).contains(
            "Directorio places no pieces and spends no Resources. Syndicate decides next."
        )
    );
    assert_briefing_lines_fit(&preview);
    app.capture("manifesto-directorio-decline-preview");
    app.choose("manifesto:decline");

    let (code, awake) = app.answered(&mut waiting);
    assert_eq!((code, &awake["status"]), (0, &json!("decision")));
    let seen = &awake["view"];
    assert_eq!(seen["decision"]["faction"], "Syndicate");
    assert_eq!(seen["decision"]["yours"], true);
    assert_eq!(seen["state"]["eligibility"]["eligible"][3], false);
    assert_eq!(seen["state"]["event_executor"], "M26");
    assert_eq!(
        seen["state"]["manifesto"]["declines"],
        json!(["M26", "Directorio"])
    );
    let at = seen["revision"].as_u64().unwrap().to_string();
    let id = format!("{at}:manifesto:decline");
    let decline = seen["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap();
    assert_eq!(decline["supported"], true);
    assert_eq!(
        decline["explanation"],
        "Syndicate places no pieces and spends no Resources. Government decides next."
    );
    app.offers_nothing();
    app.capture("manifesto-syndicate-woke-ineligible");
    let committed = app.save_bytes();
    let prior = fs::read(previous_path(&app.save)).unwrap();
    let (code, wrong_seat) = app.ask(&["choose", &m26, &at, &id]);
    assert_eq!((code, &wrong_seat["status"]), (1, &json!("refused")));
    assert!(
        wrong_seat["error"]
            .as_str()
            .unwrap()
            .contains("decision owner")
    );
    let (code, stale) = app.ask(&[
        "choose",
        &syndicate,
        &prior_revision.to_string(),
        &format!("{prior_revision}:manifesto:decline"),
    ]);
    assert_eq!((code, &stale["status"]), (1, &json!("refused")));
    assert!(
        stale["error"]
            .as_str()
            .unwrap()
            .contains("Stale: the match is at revision")
    );
    assert_eq!(app.save_bytes(), committed);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), prior);
    app.decide(&syndicate, "manifesto:decline", "applied");

    let government_turn = app.observe();
    assert_eq!(government_turn["observation"]["owner"], "Government");
    let middle_edition = &government_turn["table"]["newspaper"]["open"]["report"];
    assert_eq!(middle_edition["faction"], "M26");
    assert_eq!(middle_edition["complete"], false);
    assert_eq!(
        manifesto_articles(middle_edition),
        [
            "Directorio declined placement.",
            "Syndicate declined placement.",
        ]
    );
    app.capture("manifesto-edition-for-government");
    app.key("Escape");
    let at = app.revision();
    app.activate("option:manifesto:decline", at);
    let preview = app.observe();
    assert_eq!(
        choice_part(&preview, "choice-subtitle")["text"],
        "26July executes the Event.\nGovernment decides whether to place."
    );
    assert!(tag_text(&preview["table"]["briefing"]).contains(
        "The Event ends. 26July stays Eligible. Directorio has the 2nd Eligible choices."
    ));
    assert_briefing_lines_fit(&preview);
    app.capture("manifesto-government-decline-preview");
    app.choose("manifesto:decline");

    let finish = app.observe();
    let last_edition = &finish["table"]["newspaper"]["open"]["report"];
    assert_eq!(last_edition["complete"], true);
    assert_eq!(last_edition["faction"], "M26");
    assert_eq!(last_edition["recorded"], "Event");
    assert_eq!(
        manifesto_articles(last_edition),
        ["Government declined placement."]
    );
    app.capture("manifesto-edition-at-completion");
    app.key("Escape");
    let final_view = app.observe();
    assert_eq!(
        choice_part(&final_view, "choice-heading")["text"],
        "Directorio · 2nd Eligible"
    );
    assert_eq!(
        choice_part(&final_view, "choice-subtitle")["text"],
        "26July stays Eligible after executing the Event.\n26July, Directorio, Syndicate and Government declined."
    );
    assert_eq!(
        choice_part(&final_view, "option:directorio:pass-label")["text"],
        "Pass"
    );
    assert_eq!(
        choice_part(&final_view, "option:directorio:operation-label")["text"],
        "Operation with optional Special Activity"
    );
    let label = choice_part(&final_view, "option:directorio:operation-label");
    let status = choice_part(&final_view, "pill:directorio:operation-label");
    let [_, top, right, bottom] = rect_of(&label["bounds"]);
    let [left, ..] = rect_of(&status["bounds"]);
    assert!(
        right < left && bottom - top < 40.,
        "{label} beside {status}"
    );
    app.capture("manifesto-directorio-second-eligible-menu");
    let observation = &final_view["observation"];
    assert_eq!(
        observation["next"]["options"],
        json!([
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)"
        ])
    );
    assert_eq!(observation["owner"], "Directorio");
    assert_eq!(observation["eligible"], before["eligible"]);
    assert_eq!(observation["eligibility"]["m26_next_card"], true);
    assert_eq!(observation["executing"], Value::Null);
    for same in [
        "available",
        "spaces",
        "resources",
        "standings",
        "piles",
        "passed",
    ] {
        assert_eq!(observation[same], before[same], "{same}");
    }
    let at = observation["revision"].as_u64().unwrap();
    let saved = app.save_bytes();
    for (action, title, rule, effect) in [
        (
            "directorio:pass",
            "Pass",
            "R 2.3.3",
            "stay Eligible and gain 1 Resource",
        ),
        (
            "directorio:operation",
            "Operation",
            "R 2.3.4",
            "may include a Special Activity",
        ),
    ] {
        app.activate(&format!("option:{action}"), at);
        let inspected = app.observe();
        let offered = inspected["observation"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == format!("{at}:{action}"))
            .unwrap();
        assert_eq!(offered["supported"], false);
        assert_eq!(choice_part(&inspected, "briefing-title")["text"], title);
        assert_eq!(choice_part(&inspected, "briefing-rule-ref")["text"], rule);
        let body = choice_part(&inspected, "briefing-value:OPTION")["text"]
            .as_str()
            .unwrap();
        assert!(!body.contains(rule));
        assert_eq!(body.lines().count(), 2, "{body}");
        let briefing = tag_text(&inspected["table"]["briefing"]);
        assert!(briefing.contains(effect), "{briefing}");
        assert!(briefing.contains("outside this continuation"), "{briefing}");
        assert_briefing_lines_fit(&inspected);
        app.capture(&format!("manifesto-next-{}", action.replace(':', "-")));
        assert_eq!(app.save_bytes(), saved);
    }
    let (_, final_client) = app.ask(&["observe", &syndicate]);
    let report = final_client["view"]["results"]
        .as_array()
        .unwrap()
        .last()
        .unwrap();
    assert_eq!(
        manifesto_articles(report),
        [
            "26July declined placement.",
            "Directorio declined placement.",
            "Syndicate declined placement.",
            "Government declined placement.",
        ]
    );
    assert_eq!(final_client["view"]["proclamations"]["published"], words);
    fs::write(app.root.join("manifesto-faction-client-evidence.json"), serde_json::to_vec_pretty(&json!({
        "awake": awake, "wrong_seat": wrong_seat, "stale": stale, "spent": spent, "complete": final_client,
    })).unwrap()).unwrap();
    app.quit();
}

fn manifesto_articles(report: &Value) -> Vec<&str> {
    report["articles"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|article| {
            article["lines"]
                .as_array()
                .unwrap()
                .iter()
                .map(|line| line.as_str().unwrap())
        })
        .collect()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the Manifesto placement and largest legal menu at 4K"]
fn native_manifesto_placements_use_the_map_and_held_faction() {
    use super::faction_seats::SYNDICATE;

    let mut app = super::available_forces::unlaunched("manifesto-placement-", "issue-241");
    finish_event(&app.save, "shaded");
    let mut session = Session::open(&app.save, None).unwrap();
    for action in [
        "advance-card",
        "government:pass",
        "syndicate:construct",
        "construct:camaguey-city:place",
        "end-construct",
        "advance-card",
    ] {
        let seen = session.observe();
        session
            .submit(
                seen.owner,
                seen.revision,
                &format!("{}:{action}", seen.revision),
            )
            .unwrap();
    }
    drop(session);
    app.launch_size("3840x2160");
    let revision = app.revision();
    app.activate("option:m26:event", revision);
    app.choose("m26:event");
    let owner = app.observe();
    let revision = owner["observation"]["revision"].as_u64().unwrap();
    let before = app.save_bytes();
    let mixed = "manifesto:place:santiago:guerrilla-base";
    app.activate("space:santiago", revision);
    app.activate(&format!("popover:row:{mixed}"), revision);
    let inspection = app.observe();
    let popover = &inspection["table"]["popover"];
    assert!(
        popover["card"]["subtitle"]
            .as_str()
            .unwrap()
            .contains("26July has 2 Active Guerrillas")
    );
    let row = popover_row(&inspection, "1 Guerrilla + 1 Base");
    let text = row["lines"].to_string();
    for fact in [
        "1 Underground Guerrilla · Available 12 → 11",
        "1 Base · Available 3 → 2",
        "Control of Santiago de Cuba: Government → 26July",
        "26July 8 → 9",
    ] {
        assert!(text.contains(fact), "{fact}: {text}");
    }
    assert_eq!(row["button"], "Place these pieces");
    assert!(text.contains("0 Resources · Card 19, R 1.4, 5.1"), "{text}");
    assert!(!text.contains("return to Available"), "{text}");
    assert_eq!(
        choice_part(&inspection, "manifesto-map-line")["text"],
        "Choose a highlighted space to inspect its legal mixes."
    );
    assert_eq!(app.save_bytes(), before);
    assert_eq!(inspection["observation"], owner["observation"]);
    app.capture("manifesto-mixed-preview");
    app.activate("popover:commit", revision);
    let placed = app.observe();
    assert_eq!(placed["observation"]["owner"], "Directorio");
    assert_eq!(placed["observation"]["standings"][1]["score"], 9);
    assert_eq!(
        placed["observation"]["manifesto"]["placements"][0],
        json!({
            "space":"santiago", "faction":"M26",
            "pieces":[
                {"faction":"M26","kind":"Guerrilla","state":"Underground","count":1},
                {"faction":"M26","kind":"Base","state":"Ordinary","count":1}
            ]
        })
    );
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before);
    // The Event is the human's, so the factual report is available without an edition.
    assert!(placed["table"]["newspaper"]["open"].is_null());
    let session = Session::open(&app.save, None).unwrap();
    let reports = session.reports();
    let report = reports.last().unwrap();
    assert_eq!(report.faction, cuba_libre_unfolded::Faction::M26);
    assert_eq!(
        report.articles[0].lines,
        ["26July placed 1 Underground Guerrilla + 1 Base in Santiago de Cuba."]
    );
    drop(session);
    app.capture("manifesto-mixed-committed");
    app.key("Escape");
    let after_mixed = app.save_bytes();
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(app.save_bytes(), after_mixed);
    assert_eq!(app.observe()["observation"], placed["observation"]);
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let mut waiting = app.player("syndicate-placement", &["wait", &syndicate]);
    app.until_waiting("Syndicate");
    let revision = app.revision();
    app.activate("option:manifesto:decline", revision);
    app.choose("manifesto:decline");

    let (code, awake) = app.answered(&mut waiting);
    assert_eq!((code, &awake["status"]), (0, &json!("decision")));
    let seen = &awake["view"];
    assert_eq!(seen["decision"]["faction"], "Syndicate");
    assert_eq!(seen["state"]["eligibility"]["eligible"][3], false);
    let at = seen["revision"].as_u64().unwrap();
    let casino_placement = "manifesto:place:camaguey-city:guerrilla-guerrilla";
    let choices = seen["choices"].as_array().unwrap();
    let offered = choices
        .iter()
        .find(|choice| choice["id"] == format!("{at}:{casino_placement}"))
        .unwrap();
    assert!(
        offered["explanation"]
            .as_str()
            .unwrap()
            .contains("Syndicate has 1 closed Casino.")
    );
    assert_eq!(offered["preview"]["placement_supply"][0]["from"], 6);
    assert_eq!(offered["preview"]["placement_supply"][0]["to"], 4);
    assert_eq!(offered["preview"]["control"], json!([]));
    assert_eq!(offered["preview"]["standings"], json!([]));
    let human = app.observe();
    assert!(
        human["observation"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| { c["id"] == offered["id"] && c["preview"] == offered["preview"] })
    );
    app.activate("space:camaguey-city", at);
    let inspection = app.observe();
    assert!(
        inspection["table"]["popover"]["card"]["subtitle"]
            .as_str()
            .unwrap()
            .contains("Syndicate has 1 closed Casino")
    );
    assert!(popover_row(&inspection, "2 Guerrillas")["button"].is_null());
    app.offers_nothing();
    app.capture("manifesto-casino-only-preview-held-seat");
    app.key("Escape");
    let before = app.save_bytes();
    let old = (at - 1).to_string();
    let (code, stale) = app.ask(&[
        "choose",
        &syndicate,
        &old,
        &format!("{old}:{casino_placement}"),
    ]);
    assert_eq!((code, &stale["status"]), (1, &json!("refused")));
    assert_eq!(app.save_bytes(), before);
    app.decide(&syndicate, casino_placement, "applied");
    let government = app.observe();
    assert_eq!(government["observation"]["owner"], "Government");
    assert_eq!(government["observation"]["standings"][3]["score"], 3);
    assert_eq!(
        government["observation"]["manifesto"]["declines"],
        json!(["Directorio"])
    );
    let city = government["observation"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "camaguey-city")
        .unwrap();
    assert!(city["pieces"].as_array().unwrap().contains(&json!({
        "faction":"Syndicate","kind":"Casino","state":"Closed","count":1
    })));
    assert!(city["pieces"].as_array().unwrap().contains(&json!({
        "faction":"Syndicate","kind":"Guerrilla","state":"Underground","count":2
    })));
    let saved = Session::open(&app.save, None).unwrap();
    assert_eq!(
        saved
            .observe()
            .spaces
            .iter()
            .find(|s| s.id == "camaguey-city")
            .unwrap()
            .control(),
        Some(cuba_libre_unfolded::Faction::Government)
    );
    app.capture("manifesto-casino-only-committed");
    app.key("Escape");
    let before = app.save_bytes();

    let key = |app: &mut Native, key: &str, focused: Option<&str>| {
        let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
        let mut request = json!({"op":"key","revision":revision,"key":key});
        if let Some(focused) = focused {
            request["focused"] = json!(focused);
        }
        app.request(request, true);
        app.request(json!({"op":"settle"}), true);
        app.observe()
    };
    let tab_to = |app: &mut Native, wanted: &str| {
        for _ in 0..20 {
            let seen = key(app, "Tab", None);
            if seen["table"]["choice"]["focus"] == wanted {
                return;
            }
        }
        panic!("Tab did not reach {wanted}");
    };
    tab_to(&mut app, "space:camaguey-city");
    key(&mut app, "Enter", Some("space:camaguey-city"));
    let row = "popover:row:manifesto:place:camaguey-city:troops-base";
    tab_to(&mut app, row);
    let full = key(&mut app, "Enter", Some(row));
    let popover = &full["table"]["popover"];
    assert_eq!(popover["space"], "camaguey-city");
    assert_eq!(
        popover["card"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["head"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "2 Troops",
            "1 Troop + 1 Police cube",
            "1 Troop + 1 Base",
            "2 Police cubes",
            "1 Police cube + 1 Base",
            "2 Bases",
        ]
    );
    let row = popover_row(&full, "1 Troop + 1 Base");
    assert_eq!(row["chosen"], true);
    let text = row["lines"].to_string();
    for fact in [
        "1 Troop · Available 3 → 2",
        "1 Base · Available 2 → 1",
        "Control and standings stay as shown.",
    ] {
        assert!(text.contains(fact), "{fact}: {text}");
    }
    let [left, top, right, bottom] = rect_of(&popover["bounds"]);
    let [map_left, map_top, map_right, map_bottom] = rect_of(&full["table"]["map_viewport"]);
    assert!(
        left >= map_left && top >= map_top && right <= map_right && bottom <= map_bottom,
        "{popover}"
    );
    assert_eq!(app.save_bytes(), before);
    app.capture("manifesto-largest-menu");
    tab_to(&mut app, "popover:commit");
    let done = key(&mut app, "Enter", Some("popover:commit"));
    assert_eq!(done["observation"]["owner"], "Directorio");
    assert_eq!(done["observation"]["next"]["role"], "2nd Eligible");
    assert_eq!(
        done["observation"]["manifesto"]["placements"][2]["space"],
        "camaguey-city"
    );
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before);
    let saved = Session::open(&app.save, None).unwrap();
    let government = cuba_libre_unfolded::Faction::Government;
    let city = saved
        .observe()
        .spaces
        .into_iter()
        .find(|s| s.id == "camaguey-city")
        .unwrap();
    assert_eq!(city.control(), Some(government));
    assert!(city.pieces.iter().any(|p| p.faction == government
        && p.kind == cuba_libre_unfolded::Kind::Troops
        && p.count == 2));
    assert!(city.pieces.iter().any(|p| p.faction == government
        && p.kind == cuba_libre_unfolded::Kind::Base
        && p.count == 1));
    app.capture("manifesto-keyboard-committed");
    app.quit();
}
