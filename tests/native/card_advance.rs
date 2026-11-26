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
        assert_missing_or_changed_face_is_recoverable(&app, &saved, &prior);

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
            assert_eq!(told["view"]["decision"]["stands"], "build-ends");
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
        assert_eq!(after["choices"], json!([]));
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
    for name in ["card-17.png", "card-18.png", "card-19.png"] {
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
