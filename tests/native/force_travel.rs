use super::*;

const BASES: &str = "manifesto:place:havana:base-base";

fn manifesto(app: &mut Native) {
    card_advance::finish_event(&app.save, "shaded");
    app.launch_size("3840x2160");
    app.choose("advance-card");
    for action in ["government:pass", "syndicate:pass"] {
        let revision = app.revision();
        app.activate(&format!("option:{action}"), revision);
        app.choose(action);
    }
    app.choose("advance-card");
    for action in ["m26:event", "manifesto:decline"] {
        let revision = app.revision();
        app.activate(&format!("option:{action}"), revision);
        app.choose(action);
    }
}

fn preview(app: &mut Native, action: &str) -> Value {
    let revision = app.revision();
    app.activate("space:havana", revision);
    app.activate(&format!("popover:row:{action}"), revision);
    app.observe()
}

fn flights(view: &Value) -> &[Value] {
    view["table"]["force_travel"]
        .as_array()
        .expect("the seat reports the moving forces")
}

fn activate_unsettled(app: &mut Native, component: &str) -> Value {
    let revision = app.revision();
    app.request(
        json!({"op":"activate","revision":revision,"component":component}),
        true,
    );
    app.observe()
}

fn shown_count<'a>(view: &'a Value, solid: &str) -> Option<&'a str> {
    view["table"]["counts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|count| count["name"] == format!("count:{solid}"))
        .and_then(|count| count["text"].as_str())
}

fn quantity(view: &Value, space: &str, faction: &str, kind: &str, state: &str) -> u64 {
    view["observation"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == space)
        .unwrap()["pieces"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["faction"] == faction && p["kind"] == kind && p["state"] == state)
        .map(|p| p["count"].as_u64().unwrap())
        .sum()
}

fn counter_center(view: &Value, counter: &str) -> [f32; 2] {
    let parts = view["table"]["available_forces"]["parts"]
        .as_array()
        .unwrap();
    let count = parts.iter().find(|part| part["part"] == counter).unwrap();
    let [left, top, right, bottom] = rect_of(&count["bounds"]);
    [((left + right) / 2.) as f32, ((top + bottom) / 2.) as f32]
}

fn point(value: &Value) -> [f32; 2] {
    let p = value.as_array().unwrap();
    [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32]
}

fn solid_center(view: &Value, name: &str) -> Option<[f32; 2]> {
    view["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == format!("piece:{name}"))
        .map(|c| point(&c["position"]))
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; view the ledger and map at successive force travel frames"]
fn native_force_travel_follows_the_committed_manifesto_placement() {
    let mut app = available_forces::unlaunched("manifesto-force-travel-", "issue-271");
    manifesto(&mut app);
    let proposed = preview(&mut app, BASES);
    assert_eq!(proposed["observation"]["owner"], "Directorio");
    assert!(flights(&proposed).is_empty());
    assert_eq!(proposed["table"]["force_travel_started"], false);

    assert_eq!(
        quantity(&proposed, "havana", "Directorio", "Base", "Ordinary"),
        0
    );
    assert_eq!(
        available_forces::drawn_count(&proposed, "directorio-base"),
        "4"
    );
    assert!(
        popover_row(&proposed, "2 Bases")["lines"]
            .to_string()
            .contains("2 Bases · Available 4 → 2")
    );

    let revision = app.revision();
    let first = activate_unsettled(&mut app, "popover:commit");
    assert_eq!(first["observation"]["revision"], revision + 1);
    assert_eq!(
        quantity(&first, "havana", "Directorio", "Base", "Ordinary"),
        2
    );
    assert_eq!(
        available_forces::drawn_count(&first, "directorio-base"),
        "2"
    );
    let [bases] = flights(&first) else {
        panic!(
            "one group of Directorio Bases should travel after commit: {:?}",
            flights(&first)
        );
    };
    assert_eq!(
        bases["group"],
        json!({"faction":"Directorio","kind":"Base","state":"Ordinary","count":2})
    );
    assert_eq!(first["table"]["force_travel_started"], true);
    assert_eq!(bases["roundel"], "2");
    assert_eq!(bases["from"], "directorio-base-count");
    assert_eq!(bases["to"], "havana");
    let start = point(&bases["start"]);
    let destination = point(&bases["end"]);
    assert!(distance(start, counter_center(&first, "directorio-base-count")) < 4.);
    assert!(distance(start, destination) > 100.);
    assert_eq!(shown_count(&first, "havana Directorio Base Ordinary"), None);

    let middle = app.request(json!({"op":"advance","steps":15}), true);
    let [middle_base] = flights(&middle) else {
        panic!("the placement must still be moving: {:?}", flights(&middle));
    };
    let from = point(&bases["position"]);
    let moved = point(&middle_base["position"]);
    assert!(distance(moved, destination) < distance(from, destination));
    assert!(distance(moved, start) > distance(from, start));
    let early_alpha = bases["opacity"].as_f64().unwrap();
    let middle_alpha = middle_base["opacity"].as_f64().unwrap();
    assert!(early_alpha >= 0. && middle_alpha > early_alpha && middle_alpha < 1.);
    app.capture("manifesto-bases-travelling");

    let at_rest = app.advance(60);
    assert!(flights(&at_rest).is_empty());
    assert_eq!(at_rest["observation"], first["observation"]);
    assert_eq!(
        available_forces::drawn_count(&at_rest, "directorio-base"),
        "2"
    );
    assert_eq!(
        shown_count(&at_rest, "havana Directorio Base Ordinary"),
        Some("2")
    );
    assert!(
        distance(
            destination,
            solid_center(&at_rest, "havana Directorio Base Ordinary").unwrap()
        ) < 1.
    );
    app.capture("manifesto-bases-landed");

    let proposed = preview(&mut app, "manifesto:place:havana:guerrilla-guerrilla");
    assert_eq!(proposed["observation"]["owner"], "Syndicate");
    assert!(flights(&proposed).is_empty());
    let placed = activate_unsettled(&mut app, "popover:commit");
    let [guerrillas] = flights(&placed) else {
        panic!(
            "one group of Guerrillas should travel: {:?}",
            flights(&placed)
        );
    };
    assert_eq!(
        guerrillas["group"],
        json!({"faction":"Syndicate","kind":"Guerrilla","state":"Underground","count":2})
    );
    assert_eq!(guerrillas["from"], "syndicate-guerrilla-count");
    assert_eq!(guerrillas["to"], "havana");
    assert_eq!(
        available_forces::drawn_count(&placed, "syndicate-guerrilla"),
        "4"
    );
    assert_eq!(
        shown_count(&placed, "havana Syndicate Guerrilla Underground"),
        None
    );
    let landed = app.advance(60);
    assert_eq!(
        shown_count(&landed, "havana Syndicate Guerrilla Underground"),
        Some("2")
    );

    let proposed = preview(&mut app, "manifesto:place:havana:troops-police");
    assert!(flights(&proposed).is_empty());
    assert_eq!(proposed["observation"]["owner"], "Government");
    let placed = activate_unsettled(&mut app, "popover:commit");
    let [troops, police] = flights(&placed) else {
        panic!(
            "two distinct Government groups should travel: {:?}",
            flights(&placed)
        );
    };
    let travelling = app.request(json!({"op":"advance","steps":25}), true);
    let [middle_troops, middle_police] = flights(&travelling) else {
        panic!("both Government groups should remain in flight");
    };
    assert!(
        distance(
            point(&middle_troops["position"]),
            point(&middle_police["position"])
        ) > 30.
    );
    app.capture("manifesto-cubes-travelling");
    let landed = app.advance(60);
    assert_eq!(landed["observation"], placed["observation"]);
    assert!(flights(&landed).is_empty());
    for (kind, flight, base, available) in [("Troops", troops, 6, 3), ("Police", police, 4, 8)] {
        let solid = format!("havana Government {kind} Ordinary");
        let ledger = format!("government-{}", kind.to_lowercase());
        let counter = format!("{ledger}-count");
        assert_eq!(
            flight["group"],
            json!({"faction":"Government","kind":kind,"state":"Ordinary","count":1})
        );
        assert_eq!(flight["from"], counter);
        assert_eq!(flight["to"], "havana");
        assert_eq!(
            quantity(&placed, "havana", "Government", kind, "Ordinary"),
            base + 1
        );
        assert_eq!(
            shown_count(&placed, &solid),
            Some(base.to_string().as_str())
        );
        assert_eq!(
            shown_count(&landed, &solid),
            Some((base + 1).to_string().as_str())
        );
        assert_eq!(
            available_forces::drawn_count(&proposed, &ledger),
            available.to_string()
        );
        assert_eq!(
            available_forces::drawn_count(&placed, &ledger),
            (available - 1).to_string()
        );
        assert!(distance(point(&flight["start"]), counter_center(&placed, &counter)) < 1.);
        assert!(
            distance(
                point(&flight["end"]),
                solid_center(&landed, &solid).unwrap()
            ) < 1.
        );
    }
    app.capture("manifesto-cubes-landed");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect partial and final force returns and reopen"]
fn native_force_travel_draws_partial_and_final_returns_once() {
    use std::fs;
    use unfolded_record_replacement::previous_path;

    const GUERRILLA: &str = "santiago M26 Guerrilla Underground";
    const COUNTER: &str = "26july-guerrilla-count";

    let mut app = available_forces::start("force-return-", "issue-271", "3840x2160");
    app.choose("terror");
    app.choose("remove");
    let at_rest = app.point(&format!("piece:{GUERRILLA}"));
    assert_eq!(
        quantity(&at_rest, "santiago", "M26", "Guerrilla", "Underground"),
        2
    );
    assert_eq!(
        available_forces::drawn_count(&at_rest, "26july-guerrilla"),
        "12"
    );
    assert_eq!(shown_count(&at_rest, GUERRILLA), Some("2"));
    assert!(flights(&at_rest).is_empty());

    let revision = app.revision();
    let saved = app.save_bytes();
    app.activate(&format!("piece:{GUERRILLA}"), revision);
    let proposed = app.observe();
    let row = &proposed["table"]["popover"]["card"]["rows"][0];
    assert_eq!(
        row["choice"],
        format!("{revision}:remove:santiago:underground:1")
    );
    assert_eq!(proposed["observation"], at_rest["observation"]);
    assert!(flights(&proposed).is_empty());
    app.activate("popover:cancel", revision);
    assert!(flights(&app.observe()).is_empty());
    app.capture("return-cancelled");
    let refreshed = app.advance(24);
    assert!(flights(&refreshed).is_empty());
    assert_eq!(refreshed["table"]["force_travel_started"], false);
    assert_eq!(refreshed["observation"], at_rest["observation"]);
    assert_eq!(app.save_bytes(), saved);

    for left in [1, 0] {
        let before = app.observe();
        let map = solid_center(&before, GUERRILLA).unwrap();
        let rev = app.revision();
        let before_bytes = app.save_bytes();
        app.activate(&format!("piece:{GUERRILLA}"), rev);
        let after = activate_unsettled(&mut app, "popover:commit");
        assert_eq!(after["observation"]["revision"], rev + 1);
        assert_eq!(
            quantity(&after, "santiago", "M26", "Guerrilla", "Underground"),
            left
        );
        assert_eq!(
            available_forces::drawn_count(&after, "26july-guerrilla"),
            (14 - left).to_string()
        );
        assert_eq!(shown_count(&after, GUERRILLA), (left != 0).then_some("1"));
        let [moving] = flights(&after) else {
            panic!("a single Guerrilla should move: {:?}", flights(&after));
        };
        assert_eq!(
            moving["group"],
            json!({"faction":"M26","kind":"Guerrilla","state":"Underground","count":1})
        );
        assert_eq!(moving["from"], "santiago");
        assert_eq!(moving["to"], COUNTER);
        assert!(distance(point(&moving["start"]), map) < 1.);
        let end = point(&moving["end"]);
        assert!(distance(end, counter_center(&after, COUNTER)) < 2.);
        let middle = app.request(json!({"op":"advance","steps":15}), true);
        let [in_flight] = flights(&middle) else {
            panic!("the return must still be moving: {:?}", flights(&middle));
        };
        let alpha = in_flight["opacity"].as_f64().unwrap();
        assert!(alpha > 0. && alpha < moving["opacity"].as_f64().unwrap());
        let early = point(&moving["position"]);
        let between = point(&in_flight["position"]);
        assert!(distance(between, end) < distance(early, end));
        assert!(distance(between, map) > distance(early, map));
        app.capture(&format!("santiago-{left}-left-returning"));
        let committed_bytes = app.save_bytes();
        assert_ne!(committed_bytes, before_bytes);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before_bytes);

        if left == 1 {
            // The first trip is still active when the process exits.
            assert_eq!(middle["table"]["force_travel_started"], true);
            app.quit();
            app.launch_size("3840x2160");
            let reopened = app.observe();
            assert_eq!(reopened["observation"], after["observation"]);
            assert!(flights(&reopened).is_empty());
            assert_eq!(reopened["table"]["force_travel_started"], false);
        }
        let settled = app.advance(60);
        assert!(flights(&settled).is_empty());
        assert_eq!(settled["observation"], after["observation"]);
        assert_eq!(shown_count(&settled, GUERRILLA), (left != 0).then_some("1"));
        assert_eq!(app.save_bytes(), committed_bytes);
        assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before_bytes);
        app.capture(&format!("santiago-{left}-left-settled"));
    }
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; observe a Casino placement and the voluntary return of its open face"]
fn native_force_travel_draws_the_two_casino_faces_at_their_actual_slots() {
    use super::construct::choose_row;

    const OPEN: &str = "pinar-del-rio Syndicate Casino Open";
    const CLOSED: &str = "pinar-del-rio Syndicate Casino Closed";
    const COUNTER: &str = "syndicate-casino-count";

    let mut app = available_forces::unlaunched("casino-force-travel-", "issue-271");
    super::assault::fund_santiago(&app.save, 4);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(&mut app, action);
    }
    app.choose("advance-card");
    choose_row(&mut app, "government:assault");
    app.choose("assault:havana:syndicate");
    choose_row(&mut app, "end-assault");
    choose_row(&mut app, "syndicate:limited-operation");

    let revision = app.revision();
    app.activate("space:pinar-del-rio", revision);
    app.activate("popover:row:construct:pinar-del-rio:place", revision);
    let proposed = app.observe();
    assert!(flights(&proposed).is_empty());
    assert_eq!(
        quantity(&proposed, "pinar-del-rio", "Syndicate", "Casino", "Open"),
        1
    );
    assert_eq!(
        quantity(&proposed, "pinar-del-rio", "Syndicate", "Casino", "Closed"),
        0
    );
    assert_eq!(
        available_forces::drawn_count(&proposed, "syndicate-casino"),
        "7"
    );
    let placed = activate_unsettled(&mut app, "popover:commit");
    let [arriving] = flights(&placed) else {
        panic!("one closed Casino should move: {:?}", flights(&placed));
    };
    assert_eq!(arriving["marked_face"], false);
    assert_eq!(
        arriving["group"],
        json!({"faction":"Syndicate","kind":"Casino","state":"Closed","count":1})
    );
    assert_eq!(
        (arriving["from"].as_str(), arriving["to"].as_str()),
        (Some(COUNTER), Some("pinar-del-rio"))
    );
    assert_eq!(
        quantity(&placed, "pinar-del-rio", "Syndicate", "Casino", "Closed"),
        1
    );
    assert_eq!(
        available_forces::drawn_count(&placed, "syndicate-casino"),
        "6"
    );
    assert_eq!(shown_count(&placed, CLOSED), None);
    assert_eq!(shown_count(&placed, OPEN), Some("1"));
    assert!(distance(point(&arriving["start"]), counter_center(&placed, COUNTER)) < 2.);
    app.request(json!({"op":"advance","steps":20}), true);
    app.capture("closed-casino-placement");
    let settled = app.advance(60);
    assert!(flights(&settled).is_empty());
    assert_eq!(shown_count(&settled, CLOSED), Some("1"));
    assert!(
        distance(
            point(&arriving["end"]),
            solid_center(&settled, CLOSED).unwrap()
        ) < 1.
    );
    assert_ne!(solid_center(&settled, OPEN), solid_center(&settled, CLOSED));

    choose_row(&mut app, "remove");
    let rev = app.revision();
    app.activate(&format!("piece:{OPEN}"), rev);
    let removed = activate_unsettled(&mut app, "popover:casino-return");
    let [returning] = flights(&removed) else {
        panic!("one open Casino should move: {:?}", flights(&removed));
    };
    assert_eq!(returning["marked_face"], true);
    assert_eq!(
        returning["group"],
        json!({"faction":"Syndicate","kind":"Casino","state":"Open","count":1})
    );
    assert_eq!(
        (returning["from"].as_str(), returning["to"].as_str()),
        (Some("pinar-del-rio"), Some(COUNTER))
    );
    assert_eq!(
        quantity(&removed, "pinar-del-rio", "Syndicate", "Casino", "Open"),
        0
    );
    assert_eq!(shown_count(&removed, OPEN), None);
    assert_eq!(shown_count(&removed, CLOSED), Some("1"));
    assert_eq!(
        available_forces::drawn_count(&removed, "syndicate-casino"),
        "7"
    );
    assert!(
        distance(
            point(&returning["start"]),
            solid_center(&settled, OPEN).unwrap()
        ) < 1.
    );
    assert!(distance(point(&returning["end"]), counter_center(&removed, COUNTER)) < 2.);
    app.request(json!({"op":"advance","steps":20}), true);
    app.capture("open-casino-return");
    let settled = app.advance(60);
    assert!(flights(&settled).is_empty());
    assert_eq!(settled["observation"], removed["observation"]);
    assert_eq!(shown_count(&settled, CLOSED), Some("1"));
    app.quit();
}
