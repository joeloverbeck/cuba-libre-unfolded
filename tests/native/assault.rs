use super::*;
use cuba_libre_unfolded::{assets, save::Session};
use std::fs;
use unfolded_record_replacement::{previous_path, save};

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Assault frames"]
fn native_funded_santiago_assault_carries_each_event_and_restarts() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-130"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = evidence.canonicalize().unwrap();
    let funded = std::env::var_os("CUBA_TEST_SANTIAGO_FUNDED_SAVE").map(PathBuf::from);

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
        if let Some(source) = &funded {
            let bytes = fs::read(source).unwrap();
            assert_eq!(
                assets::digest(&bytes),
                "95d929facb49d629ce582498c4cc9f9672560b6d764379f6eb4a7c2c368cd860"
            );
            fs::write(&app.save, bytes).unwrap();
            fs::copy(previous_path(source), previous_path(&app.save)).unwrap();
        } else {
            fund_santiago(&app.save);
        }
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

        const ATTACK: &str = "assault:santiago:26july";
        let before = fs::read(&app.save).unwrap();
        app.quit();
        app.launch_size("3840x2160");
        assert_eq!(app.observe()["observation"], pending);
        pin(&mut app, ATTACK);
        let pinned = app.observe();
        let preview = tag_text(&pinned["table"]["briefing"]);
        for required in [
            "2 Troops + 1 Police = 3",
            "2 Resources (2 → 0)",
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

pub(super) fn fund_santiago(path: &Path) {
    // Seed 2 makes the shared Kidnap die roll 5. Every decision after the seed
    // uses the same checked and durable Session submission as the native seat.
    drop(Session::open(path, None).unwrap());
    let mut record: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    record["payload"]["random"] = 2.into();
    let identity = serde_json::from_value(record["identity"].clone()).unwrap();
    fs::write(path, save::seal(&identity, &record["payload"])).unwrap();
    let mut session = Session::open(path, None).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
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
