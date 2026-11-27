use super::*;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; retain the Santiago flip frames"]
fn native_santiago_terror_moves_the_committed_guerrilla() {
    let mut app = super::available_forces::start("santiago-flip-", "issue-270", "1920x1080");
    for action in ["terror", "transfer", "ask:2:2", "authorize:2"] {
        app.choose(action);
    }
    let revision = app.revision();
    let before = app.observe();
    let count = |view: &Value, face: &str| {
        let name = format!("count:santiago M26 Guerrilla {face}");
        view["table"]["counts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == name)
            .map(|row| row["text"].as_str().unwrap().to_owned())
    };
    assert_eq!(count(&before, "Underground").as_deref(), Some("2"));
    assert_eq!(count(&before, "Active"), None);
    app.activate("space:santiago", revision);
    let preview = app.observe();
    assert_eq!(super::popover_row(&preview, "Terror")["chosen"], true);
    assert_eq!(preview["observation"], before["observation"]);
    assert_eq!(preview["table"]["counts"], before["table"]["counts"]);
    app.capture("preview");
    app.request(
        json!({"op":"activate","revision":revision,"component":"popover:commit"}),
        true,
    );
    app.request(json!({"op":"advance","steps":12}), true);
    let transit = app.observe();
    assert_eq!(transit["observation"]["revision"], revision + 1);
    assert_eq!(
        transit["observation"]["resolved"],
        json!([{"space":"santiago","paid":1}])
    );
    assert_eq!(transit["observation"]["marker_supply"]["available"], 19);
    assert!(
        transit["table"]["pending"]
            .as_array()
            .unwrap()
            .contains(&json!("piece flip")),
        "the committed Guerrilla should still be moving"
    );
    assert_eq!(count(&transit, "Underground").as_deref(), Some("1"));
    assert_eq!(count(&transit, "Active"), None);
    app.capture("in-transit");
    app.request(json!({"op":"advance","steps":20}), true);
    app.capture("turn");
    app.settle();
    let settled = app.observe();
    assert_eq!(settled["observation"], transit["observation"]);
    assert_eq!(count(&settled, "Underground").as_deref(), Some("1"));
    assert_eq!(count(&settled, "Active").as_deref(), Some("1"));
    app.capture("settled");
    let saved = app.save_bytes();
    app.quit();
    app.launch_size("1920x1080");
    let reopened = app.observe();
    assert_eq!(reopened["observation"], settled["observation"]);
    assert_eq!(reopened["table"]["counts"], settled["table"]["counts"]);
    assert_eq!(app.save_bytes(), saved);
    app.quit();
}

fn drawn_count(view: &Value, name: &str) -> Option<String> {
    view["table"]["counts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .map(|row| row["text"].as_str().unwrap().to_owned())
}

fn motion_pending(view: &Value) -> bool {
    view["table"]["pending"]
        .as_array()
        .unwrap()
        .contains(&json!("piece flip"))
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; retains the two simultaneous Guerrilla flips"]
fn native_shaded_event_flips_into_empty_and_occupied_slots() {
    const SANTIAGO_ACTIVE: &str = "count:santiago M26 Guerrilla Active";
    const SANTIAGO_UNDERGROUND: &str = "count:santiago M26 Guerrilla Underground";
    const ORIENTE_ACTIVE: &str = "count:oriente M26 Guerrilla Active";
    const ORIENTE_UNDERGROUND: &str = "count:oriente M26 Guerrilla Underground";

    let mut app = super::available_forces::unlaunched("shaded-flips-", "issue-270");
    super::assault::fund_santiago(&app.save, 4);
    app.launch_size("3840x2160");
    let before = app.observe();
    assert_eq!(drawn_count(&before, SANTIAGO_ACTIVE).as_deref(), Some("1"));
    assert_eq!(
        drawn_count(&before, SANTIAGO_UNDERGROUND).as_deref(),
        Some("1")
    );
    assert_eq!(drawn_count(&before, ORIENTE_ACTIVE), None);
    assert_eq!(
        drawn_count(&before, ORIENTE_UNDERGROUND).as_deref(),
        Some("1")
    );
    assert!(!motion_pending(&before));

    super::rally::choose_rally_option(&mut app, "event:shaded");
    let awaiting = app.observe();
    assert_eq!(
        awaiting["observation"]["spaces"],
        before["observation"]["spaces"]
    );
    assert_eq!(awaiting["table"]["counts"], before["table"]["counts"]);
    assert!(!motion_pending(&awaiting));

    let revision = app.revision();
    app.activate("option:resolve-shaded", revision);
    app.capture("awaiting-shaded-resolution");
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:resolve-shaded")}),
        true,
    );
    app.request(json!({"op":"advance","steps":12}), true);
    let transit = app.observe();
    let committed = &transit["observation"];
    assert_eq!(committed["revision"], revision + 1);
    assert_eq!(
        committed["event"]["Complete"]["Shaded"]["flips"],
        json!([
            {"space":"santiago","faction":"M26","kind":"Guerrilla","from":"Underground","to":"Active","count":1},
            {"space":"oriente","faction":"M26","kind":"Guerrilla","from":"Underground","to":"Active","count":1}
        ])
    );
    assert_eq!(
        committed["map_piece_count"],
        before["observation"]["map_piece_count"]
    );
    assert_eq!(committed["available"], before["observation"]["available"]);
    assert!(
        motion_pending(&transit),
        "both committed Guerrillas should still be moving"
    );
    assert_eq!(drawn_count(&transit, SANTIAGO_UNDERGROUND), None);
    assert_eq!(drawn_count(&transit, ORIENTE_UNDERGROUND), None);
    assert_eq!(drawn_count(&transit, SANTIAGO_ACTIVE).as_deref(), Some("1"));
    assert_eq!(drawn_count(&transit, ORIENTE_ACTIVE), None);
    app.capture("shaded-in-transit");

    app.request(json!({"op":"advance","steps":30}), true);
    let turning = app.observe();
    assert!(motion_pending(&turning));
    assert_eq!(drawn_count(&turning, SANTIAGO_ACTIVE).as_deref(), Some("1"));
    assert_eq!(drawn_count(&turning, ORIENTE_ACTIVE), None);
    app.capture("shaded-turn");

    app.settle();
    let settled = app.observe();
    assert_eq!(settled["observation"], *committed);
    assert_eq!(drawn_count(&settled, SANTIAGO_ACTIVE).as_deref(), Some("2"));
    assert_eq!(drawn_count(&settled, ORIENTE_ACTIVE).as_deref(), Some("1"));
    assert!(!motion_pending(&settled));
    app.capture("shaded-settled");
    app.quit();
}

fn zoom_to(app: &mut Native, space: &str) {
    app.settle();
    app.point(&format!("space:{space}"));
    let before = app.observe();
    let revision = before["observation"]["revision"].as_u64().unwrap();
    app.request(
        json!({"op":"scroll","revision":revision,"component":format!("space:{space}"),"y":16}),
        true,
    );
    let close = app.settle();
    assert!(
        close["table"]["camera"]["distance"].as_f64().unwrap()
            < before["table"]["camera"]["distance"].as_f64().unwrap() * 0.65
    );
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; retains both Casino turns"]
fn native_havana_casino_closes_and_opens_with_both_faces() {
    const OPEN: &str = "count:havana Syndicate Casino Open";
    const CLOSED: &str = "count:havana Syndicate Casino Closed";
    let mut app = super::available_forces::unlaunched("havana-casino-flips-", "issue-270");
    super::assault::fund_santiago(&app.save, 4);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        super::construct::choose_row(&mut app, action);
    }
    app.choose("advance-card");
    super::construct::choose_row(&mut app, "government:assault");
    let revision = app.revision();
    // The Home camera frames the island. Pan west before narrowing the view,
    // using the same Havana recentering as the Manifesto native test.
    for (key, steps) in [("KeyA", 35), ("KeyW", 25)] {
        for _ in 0..steps {
            app.request(json!({"op":"key","key":key,"revision":revision}), true);
        }
    }
    zoom_to(&mut app, "havana");

    for (label, action, source, destination, old_face, new_face) in [
        (
            "closing",
            "assault:havana:syndicate",
            OPEN,
            CLOSED,
            json!("Open"),
            Value::Null,
        ),
        (
            "opening",
            "construct:havana:open",
            CLOSED,
            OPEN,
            Value::Null,
            json!("Open"),
        ),
    ] {
        if label == "opening" {
            super::construct::choose_row(&mut app, "end-assault");
            super::construct::choose_row(&mut app, "syndicate:limited-operation");
        }
        let before = app.observe();
        let revision = app.revision();
        assert_eq!(drawn_count(&before, source).as_deref(), Some("1"));
        assert_eq!(drawn_count(&before, destination), None);
        app.activate("space:havana", revision);
        let choice = format!("{revision}:{action}");
        let chosen = |view: &Value| {
            view["table"]["popover"]["card"]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["choice"] == choice && row["chosen"] == true)
        };
        if !chosen(&app.observe()) {
            app.activate(&format!("popover:row:{action}"), revision);
        }
        let preview = app.observe();
        assert!(chosen(&preview));
        assert_eq!(preview["observation"], before["observation"]);
        assert_eq!(drawn_count(&preview, source).as_deref(), Some("1"));
        assert_eq!(drawn_count(&preview, destination), None);
        assert!(!motion_pending(&preview));
        app.capture(&format!("{label}-preview"));
        app.request(
            json!({"op":"activate","revision":revision,"component":"popover:commit"}),
            true,
        );
        app.request(json!({"op":"advance","steps":5}), true);
        let lifted = app.observe();
        assert_eq!(lifted["observation"]["revision"], revision + 1);
        assert_eq!(
            lifted["observation"]["available"],
            before["observation"]["available"]
        );
        assert_eq!(
            lifted["observation"]["map_piece_count"],
            before["observation"]["map_piece_count"]
        );
        let casino = lifted["observation"]["spaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|space| space["id"] == "havana")
            .unwrap()["pieces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|piece| piece["kind"] == "Casino")
            .unwrap();
        assert_eq!(casino["count"], 1);
        assert_eq!(
            casino["state"],
            if label == "opening" { "Open" } else { "Closed" }
        );
        assert!(
            motion_pending(&lifted),
            "the Casino should be turning on the table"
        );
        assert_eq!(drawn_count(&lifted, source), None);
        assert_eq!(drawn_count(&lifted, destination), None);
        assert_eq!(lifted["table"]["piece_flips"][0]["face_mark"], old_face);
        app.capture(&format!("{label}-lifted"));

        app.request(json!({"op":"advance","steps":45}), true);
        let turning = app.observe();
        assert!(motion_pending(&turning));
        assert_eq!(turning["table"]["piece_flips"][0]["face_mark"], new_face);
        assert_eq!(drawn_count(&turning, destination), None);
        app.capture(&format!("{label}-turn"));
        app.settle();
        let settled = app.observe();
        assert_eq!(settled["observation"], lifted["observation"]);
        assert_eq!(drawn_count(&settled, source), None);
        assert_eq!(drawn_count(&settled, destination).as_deref(), Some("1"));
        assert!(!motion_pending(&settled));
        app.capture(&format!("{label}-settled"));
    }
    let before_reopen = app.observe();
    let saved = app.save_bytes();
    app.quit();
    app.launch_size("3840x2160");
    let reopened = app.observe();
    assert_eq!(reopened["observation"], before_reopen["observation"]);
    assert_eq!(drawn_count(&reopened, OPEN).as_deref(), Some("1"));
    assert_eq!(drawn_count(&reopened, CLOSED), None);
    assert!(!motion_pending(&reopened));
    assert_eq!(app.save_bytes(), saved);
    app.quit();
}
