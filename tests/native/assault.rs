use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;
use unfolded_record_replacement::previous_path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Assault frames"]
fn native_funded_santiago_assault_carries_each_event_and_restarts() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-130"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();

    for (text, returned, left_underground) in [("shaded", 2, 0), ("unshaded", 1, 1)] {
        let root = tempfile::Builder::new()
            .prefix(&format!("santiago-{text}-"))
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
        eprintln!("{text} Assault evidence: {}", app.root.display());
        fund_santiago(&app.save, 1);
        let random = Session::open(&app.save, None)
            .unwrap()
            .game()
            .random_state();
        app.launch_size("3840x2160");
        let handoff = app.observe()["observation"].clone();
        assert_eq!(handoff["owner"], "Directorio");
        assert_eq!(handoff["kidnap"]["die"], 5);
        assert_eq!(handoff["resources"][0], 2);

        pin(&mut app, &format!("event:{text}"));
        app.choose(&format!("event:{text}"));
        if text == "shaded" {
            pin(&mut app, "resolve-shaded");
            app.choose("resolve-shaded");
        } else {
            for action in ["rally:beneficiary:26july", "rally:sierra-maestra:place:0"] {
                pin(&mut app, action);
                app.choose(action);
            }
        }
        app.choose("advance-card");
        pin(&mut app, "government:assault");
        app.choose("government:assault");
        let prompt = app.observe();
        assert_eq!(
            choice_part(&prompt, "choice-heading")["text"],
            "Government · Assault"
        );
        let pending = prompt["observation"].clone();
        assert_eq!(pending["executing"], "Government");
        assert_eq!(pending["resources"], handoff["resources"]);

        // Santiago is chosen on the map (#169): its popover previews the plan.
        const ATTACK: &str = "assault:santiago:26july";
        let before = fs::read(&app.save).unwrap();
        app.quit();
        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], pending);
        let revision = pending["revision"].as_u64().unwrap();
        app.activate("space:santiago", revision);
        let opened = app.observe();
        assert_eq!(opened["observation"], pending, "a look decides nothing");
        let preview = popover_text(&opened);
        for required in [
            "2 Troops + 1 Police = 3",
            "Pay 2 Resources: Government 2 → 0.",
            &format!("{returned} Active, {left_underground} Underground"),
        ] {
            assert!(
                preview.contains(required),
                "{required} absent from {preview}"
            );
        }
        app.capture("santiago-payment-preview");
        app.choose(ATTACK);
        let paid = app.observe();
        let after = &paid["observation"];
        assert_eq!(after["resources"][0], 0);
        for index in 1..4 {
            assert_eq!(after["resources"][index], pending["resources"][index]);
        }
        assert_eq!(after["executing"], "Government");
        assert_eq!(after["owner"], "Government");
        assert_eq!(after["standings"], pending["standings"]);
        assert_eq!(
            after["available_piece_count"].as_u64().unwrap(),
            pending["available_piece_count"].as_u64().unwrap() + returned
        );
        let space = after["spaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|space| space["id"] == "santiago")
            .unwrap();
        let enemy = space["pieces"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|piece| piece["faction"] == "M26")
            .collect::<Vec<_>>();
        assert_eq!(enemy.len(), left_underground);
        if left_underground > 0 {
            assert_eq!(enemy[0]["state"], "Underground");
            assert_eq!(enemy[0]["count"], 1);
        }
        let payment_revision = pending["revision"].as_u64().unwrap();
        let paid_revision = after["revision"].as_u64().unwrap();
        assert_eq!(paid_revision, payment_revision + 1);
        let paid_bytes = fs::read(&app.save).unwrap();
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before);
        app.capture("santiago-paid");

        app.quit();
        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], *after);
        for revision in [payment_revision, paid_revision] {
            app.request(
                json!({"op":"activate", "revision":revision, "choice":format!("{revision}:{ATTACK}")}),
                false,
            );
        }
        assert_eq!(app.observe()["observation"], *after);
        assert_eq!(fs::read(&app.save).unwrap(), paid_bytes);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before);
        pin(&mut app, "end-assault");
        app.choose("end-assault");
        let syndicate = app.observe();
        let next = &syndicate["observation"];
        assert_eq!(next["owner"], "Syndicate");
        assert_eq!(next["next"]["role"], "2nd Eligible");
        assert_eq!(
            next["next"]["options"],
            json!(["Pass", "Limited Operation"])
        );
        assert_eq!(next["eligibility"]["government_box"], "OperationOnly");
        assert_eq!(next["resources"], after["resources"]);
        assert_eq!(next["spaces"], after["spaces"]);
        assert_eq!(next["current_card"], 18);
        assert_eq!(next["preview_card"], 19);
        assert_eq!(next["draw_count"], 33);
        assert_eq!(
            choice_part(&syndicate, "choice-heading")["text"],
            "Syndicate's decision"
        );
        assert_eq!(
            syndicate["table"]["eligibility"]["track"]["occupied"],
            "eligibility:operation-only"
        );
        assert_eq!(
            Session::open(&app.save, None)
                .unwrap()
                .game()
                .random_state(),
            random
        );
        pin(&mut app, "syndicate:limited-operation");
        app.capture("syndicate-second-decision");
        app.quit();
    }
}

fn pin(app: &mut Native, action: &str) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("option:{action}"), revision);
}

pub(super) fn fund_santiago(path: &Path, amount: u8) {
    // A new save with seed 2, whose shared Kidnap die rolls 5. Every decision uses
    // the same checked and durable Session submission as the native seat.
    let mut session = Session::create_seeded(path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        &format!("give:0:{amount}"),
        "end-terror",
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

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained 4K frames"]
fn native_havana_and_santiago_assault_in_either_order_restarts_and_finishes() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-131"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();

    for [first, second] in [["havana", "santiago"], ["santiago", "havana"]] {
        let root = tempfile::Builder::new()
            .prefix(&format!("{first}-first-"))
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
        eprintln!("{first} first Assault evidence: {}", app.root.display());
        fund_santiago(&app.save, 4);

        app.launch_size("3840x2160");
        pin(&mut app, "event:shaded");
        app.choose("event:shaded");
        pin(&mut app, "transfer");
        app.choose("transfer");
        app.choose("give:0:1");
        pin(&mut app, "resolve-shaded");
        app.choose("resolve-shaded");
        app.choose("advance-card");
        pin(&mut app, "government:assault");
        app.choose("government:assault");
        let opening = app.observe();
        assert_eq!(opening["observation"]["resources"][0], 6);
        app.capture("assault-opened");

        for (index, space) in [first, second].into_iter().enumerate() {
            let map = app.observe();
            if index == 1 {
                // The space paid first is not involved: no plan is offered there.
                assert_eq!(target_state(&map, first), "not-involved");
                assert!(
                    map["observation"]["choices"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|c| c["site"]["space"] != first)
                );
            }
            assert_eq!(target_state(&map, space), "selectable");
            let target = if space == "havana" {
                "syndicate"
            } else {
                "26july"
            };
            let action = format!("assault:{space}:{target}");
            let at = map["observation"]["revision"].as_u64().unwrap();
            app.activate(&format!("space:{space}"), at);
            let pinned = app.observe();
            let before = &pinned["observation"];
            let revision = before["revision"].as_u64().unwrap();
            let id = format!("{revision}:{action}");
            let choice = before["choices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == id)
                .unwrap();
            let preview = &choice["preview"];
            let balances = [6, 4, 2];
            assert_eq!(preview["resources"][0]["from"], balances[index]);
            assert_eq!(preview["resources"][0]["to"], balances[index + 1]);
            let briefing = popover_text(&pinned);
            if space == "havana" {
                assert!(briefing.contains("6 Troops + 4 Police = 10"));
                assert!(briefing.contains("Close 1 Casino in place"));
                assert!(briefing.contains("Syndicate 3 → 2"), "{briefing}");
                assert_eq!(
                    preview["flips"],
                    json!([{"space":"havana","faction":"Syndicate",
                    "kind":"Casino","from":"Open","to":"Closed","count":1}])
                );
                assert_eq!(preview["removed"], json!([]));
                let notes = preview["notes"].to_string();
                assert!(notes.contains("Close 1 Casino in place"), "{notes}");
                assert!(!notes.contains("to Available"), "{notes}");
            } else {
                assert!(briefing.contains("2 Troops + 1 Police = 3"));
                assert!(
                    preview["notes"]
                        .to_string()
                        .contains("Return 2 26July Active Guerrillas to Available Forces")
                );
                assert_eq!(
                    preview["removed"],
                    json!([{"space":"santiago","faction":"M26",
                    "kind":"Guerrilla","state":"Active","count":2}])
                );
            }
            app.capture(&format!("{space}-payment-preview"));

            app.choose(&action);
            let settled = app.observe();
            let after = &settled["observation"];
            assert_eq!(after["resources"][0], balances[index + 1]);
            assert_eq!(after["executing"], "Government");
            assert_eq!(after["current_card"], 18);
            assert_eq!(after["preview_card"], 19);
            assert_eq!(after["draw_count"], 33);
            let altered = after["spaces"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == space)
                .unwrap();
            let pieces = altered["pieces"].as_array().unwrap();
            if space == "havana" {
                assert_eq!(after["map_piece_count"], before["map_piece_count"]);
                assert_eq!(after["available"], before["available"]);
                let casino = pieces.iter().find(|p| p["kind"] == "Casino").unwrap();
                assert_eq!(casino["count"], 1);
                assert_eq!(casino["state"], "Closed");
                let directorio = pieces
                    .iter()
                    .find(|p| p["faction"] == "Directorio")
                    .unwrap();
                assert_eq!(directorio["count"], 2);
                assert_eq!(directorio["state"], "Underground");
                assert_eq!(after["standings"][3]["score"], 2);
                assert_eq!(after["standings"][3]["margin"], -15);
                assert_eq!(
                    after["standings"][3]["extra"],
                    before["standings"][3]["extra"]
                );
            } else {
                assert!(pieces.iter().all(|p| p["faction"] != "M26"));
                assert_eq!(
                    after["available_piece_count"].as_u64().unwrap(),
                    before["available_piece_count"].as_u64().unwrap() + 2
                );
            }
            app.capture(&format!("{space}-paid"));

            if index == 0 {
                let active = fs::read(&app.save).unwrap();
                let previous = fs::read(previous_path(&app.save)).unwrap();
                app.quit();
                app.launch_size("3840x2160");
                assert_eq!(app.observe()["observation"], *after);
                for rejected in [revision, revision + 1] {
                    app.request(
                        json!({"op":"activate", "revision":rejected,
                        "choice":format!("{rejected}:{action}")}),
                        false,
                    );
                }
                assert_eq!(fs::read(&app.save).unwrap(), active);
                assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
                assert_eq!(app.observe()["observation"], *after);
            }
        }
        let before_finish = app.observe()["observation"].clone();
        assert_eq!(before_finish["resources"][0], 2);
        // Every other space stays choosable: its plan that removes nothing is offered.
        assert!(
            before_finish["choices"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["site"]["space"] == "matanzas" && c["supported"] == true)
        );
        pin(&mut app, "end-assault");
        app.choose("end-assault");
        let handoff = app.observe()["observation"].clone();
        assert_eq!(handoff["owner"], "Syndicate");
        assert_eq!(handoff["next"]["role"], "2nd Eligible");
        assert_eq!(
            handoff["next"]["options"],
            json!(["Pass", "Limited Operation"])
        );
        assert_eq!(handoff["eligibility"]["government_box"], "OperationOnly");
        assert_eq!(handoff["resources"], before_finish["resources"]);
        assert_eq!(handoff["spaces"], before_finish["spaces"]);
        app.capture("syndicate-second-decision");
        app.quit();
    }
}
