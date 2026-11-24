//! #168: the keyboard reaches the spaces the open decision offers. Tab goes from the
//! panel's controls to the selectable spaces, then the blocked ones; a focused space
//! shows its tag; Enter or Space opens and closes its popover as a click does; the
//! popover's rows and button take the keyboard; Escape closes it and returns focus to
//! the space. Played at the owner's display, through the development seat's own keys.
use super::*;

/// Presses Tab until `stop` has keyboard focus, letting each focused tag's dwell end,
/// and returns every stop focus passed through, `stop` last.
fn tab_to(app: &mut Native, stop: &str) -> Vec<String> {
    let mut passed = vec![];
    for _ in 0..40 {
        let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        app.request(json!({"op":"settle"}), true);
        let focus = app.observe()["table"]["choice"]["focus"].clone();
        let focus = focus.as_str().unwrap_or_default().to_owned();
        passed.push(focus.clone());
        if focus == stop {
            return passed;
        }
    }
    panic!("Tab never reached {stop}: {passed:?}");
}

/// Presses Tab once round the whole order, from no focus back to the first stop, and
/// returns the stops in order.
fn tab_round(app: &mut Native) -> Vec<String> {
    let mut passed: Vec<String> = vec![];
    for _ in 0..60 {
        let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        app.request(json!({"op":"settle"}), true);
        let focus = app.observe()["table"]["choice"]["focus"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        if passed.first() == Some(&focus) {
            return passed;
        }
        passed.push(focus);
    }
    panic!("Tab never came round: {passed:?}");
}

/// Presses `key` (Enter or Space) on the focused control `focused`, as a player does.
fn press(app: &mut Native, key: &str, focused: &str) -> Value {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.request(
        json!({"op":"key","revision":revision,"key":key,"focused":focused}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    app.observe()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_keyboard_reaches_santiago_and_commits_terror() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-168".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("map-keys-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Keyboard map native evidence: {}", app.root.display());
    let no_space = |order: &[String], when: &str| {
        assert!(
            !order
                .iter()
                .any(|n| n.starts_with("space:") || n.starts_with("popover:")),
            "{when}: a space took focus: {order:?}"
        );
    };
    app.launch_size("3840x2160");
    // The pointer rests on an area, away from the map, for the whole run (#174: not
    // on Save · quit, whose tag the pointer opens).
    app.point("area:propaganda");

    // At the action choice no space is a stop. Terror is chosen with the keyboard.
    let order = tab_round(&mut app);
    no_space(&order, "action choice");
    std::fs::write(
        app.root.join("order-action-choice.json"),
        json!(order).to_string(),
    )
    .unwrap();
    tab_to(&mut app, "option:terror");
    press(&mut app, "Enter", "option:terror");
    tab_to(&mut app, "0:terror");
    let chosen = press(&mut app, "Enter", "0:terror");
    assert_eq!(chosen["observation"]["revision"], 1);
    assert_eq!(chosen["observation"]["action"], "Terror");

    // Terror at 0 Resources: Santiago offers Kidnap (selectable), Oriente waits for a
    // transfer (blocked). Tab passes the panel's controls, then Santiago, then Oriente,
    // then the stops after the panel.
    assert_eq!(target_state(&chosen, "santiago"), "selectable");
    assert_eq!(target_state(&chosen, "oriente"), "blocked");
    let order = tab_round(&mut app);
    std::fs::write(app.root.join("order-terror.json"), json!(order).to_string()).unwrap();
    let ribbon = order.iter().position(|n| n == "briefing-ribbon").unwrap();
    assert_eq!(
        order[ribbon..ribbon + 4],
        [
            "briefing-ribbon",
            "space:santiago",
            "space:oriente",
            "marker-supply"
        ],
        "{order:?}"
    );
    // The panel's steps come first; Terror's targets are no steps (#171), so the
    // first is Kidnap's.
    assert_eq!(order[0], "step:kidnap", "{order:?}");
    assert!(
        order.iter().all(|n| !n.starts_with("step:terror")),
        "{order:?}"
    );

    // A focused space shows its tag, as hover does; nothing is decided.
    let saved = std::fs::read(&app.save).unwrap();
    tab_to(&mut app, "space:oriente");
    let oriente = app.observe();
    assert_eq!(oriente["table"]["choice"]["focused_space"], "oriente");
    assert_eq!(oriente["table"]["status"]["space"], "oriente");
    assert_eq!(oriente["table"]["popover"], Value::Null);
    app.capture("focused-oriente-blocked");
    // Enter on the blocked space opens its popover, which offers no row to choose.
    let blocked = press(&mut app, "Enter", "space:oriente");
    assert_eq!(blocked["table"]["popover"]["space"], "oriente", "{blocked}");
    assert_eq!(popover_row(&blocked, "Terror")["mark"], "not-yet");
    assert_eq!(blocked["observation"]["revision"], 1);
    // Escape closes it and returns focus to the space.
    app.request(json!({"op":"key","revision":1,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    let escaped = app.observe();
    assert_eq!(escaped["table"]["popover"], Value::Null);
    assert_eq!(escaped["table"]["choice"]["focus"], "space:oriente");
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);

    // Santiago with the keyboard: its tag, then Enter opens its popover, Space closes
    // it, Space opens it again, as clicks do.
    tab_to(&mut app, "space:santiago");
    let focused = app.observe();
    assert_eq!(focused["table"]["choice"]["focused_space"], "santiago");
    assert_eq!(focused["table"]["status"]["space"], "santiago");
    app.capture("focused-santiago");
    let opened = press(&mut app, "Enter", "space:santiago");
    assert_eq!(opened["table"]["popover"]["space"], "santiago", "{opened}");
    assert_eq!(opened["table"]["popover"]["pinned"], true);
    let closed = press(&mut app, "Space", "space:santiago");
    assert_eq!(closed["table"]["popover"], Value::Null);
    let opened = press(&mut app, "Space", "space:santiago");
    assert_eq!(opened["table"]["popover"]["space"], "santiago");
    assert_eq!(
        popover_row(&opened, "Kidnap · from Government")["chosen"],
        true
    );
    app.capture("santiago-popover-zero-resources");
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);

    // The popover's button opens Kidnap; Kidnap pauses map targeting, so no space is
    // a stop. Its Roll is the panel's commit.
    tab_to(&mut app, "popover:row:kidnap");
    assert_eq!(
        app.observe()["table"]["choice"]["focus"],
        "popover:row:kidnap"
    );
    tab_to(&mut app, "popover:commit");
    let kidnap = press(&mut app, "Enter", "popover:commit");
    assert_eq!(kidnap["observation"]["revision"], 2);
    assert_eq!(kidnap["observation"]["kidnap_open"], true);
    assert_eq!(kidnap["table"]["map_targets"], Value::Null);
    let order = tab_round(&mut app);
    no_space(&order, "Kidnap open");
    tab_to(&mut app, "2:kidnap:roll");
    press(&mut app, "Enter", "2:kidnap:roll");
    let landed = press(&mut app, "Enter", "kidnap-die:continue");
    let die = landed["observation"]["kidnap"]["die"].as_u64().unwrap();
    assert!((1..=6).contains(&die));
    // Kidnap took min(die, 6) from Government's 6 Resources (R 4.3.3), so 26July can
    // now pay Santiago's Terror, which the early roll owes.
    assert_eq!(landed["observation"]["resources"][1], die);
    assert_eq!(landed["observation"]["owed_terror"], "santiago");
    let revision = landed["observation"]["revision"].as_u64().unwrap();
    assert_eq!(target_state(&landed, "santiago"), "selectable");

    // A popover the pointer opened, closed by Escape, leaves focus where it was.
    assert_eq!(landed["table"]["choice"]["focus"], Value::Null);
    app.activate("space:santiago", revision);
    assert_eq!(app.observe()["table"]["popover"]["space"], "santiago");
    app.request(json!({"op":"key","revision":revision,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    let escaped = app.observe();
    assert_eq!(escaped["table"]["popover"], Value::Null);
    assert_eq!(escaped["table"]["choice"]["focus"], Value::Null);
    app.point("area:propaganda");

    // Terror in Santiago with the keyboard alone: reach the space, open its popover,
    // choose Terror's row, commit with its button.
    let before = landed["observation"].clone();
    tab_to(&mut app, "space:santiago");
    let open = press(&mut app, "Enter", "space:santiago");
    assert_eq!(open["table"]["popover"]["space"], "santiago");
    tab_to(&mut app, "popover:row:terror:santiago");
    let row = press(&mut app, "Enter", "popover:row:terror:santiago");
    let terror = popover_row(&row, "Terror");
    assert_eq!(terror["chosen"], true, "{row}");
    assert_eq!(terror["choice"], format!("{revision}:terror:santiago"));
    let button = terror["button"].as_str().unwrap().to_owned();
    tab_to(&mut app, "popover:commit");
    app.capture("santiago-terror-popover-focused-button");
    let after = press(&mut app, "Enter", "popover:commit");
    // The same checked decision a click submits: one revision, Santiago resolved.
    assert_eq!(after["observation"]["revision"], revision + 1);
    assert_eq!(after["table"]["popover"], Value::Null);
    let resolved = after["observation"]["resolved"].to_string();
    assert!(resolved.contains("santiago"), "{resolved}");
    // R 3.3.4: Terror costs 1 Resource, places a Terror marker and shifts Santiago one
    // level toward Active Opposition, Neutral to Passive Opposition: 26July's score
    // (Opposition plus Bases) rises by Santiago's population of 1, from 7 to 8.
    let santiago = |view: &Value| -> Value {
        view["spaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == "santiago")
            .unwrap()
            .clone()
    };
    let (was, now) = (santiago(&before), santiago(&after["observation"]));
    assert_eq!(
        (was["support"].clone(), was["terror"].clone()),
        (json!(0), json!(0))
    );
    assert_eq!(was["population"], 1);
    assert_eq!(before["standings"][1]["score"], 7);
    assert_eq!(after["observation"]["resources"][1], die - 1);
    assert_eq!(
        (now["support"].clone(), now["terror"].clone()),
        (json!(-1), json!(1))
    );
    assert_eq!(after["observation"]["standings"][1]["score"], 8);
    std::fs::write(app.root.join("button.txt"), &button).unwrap();
    app.capture("after-santiago-terror");

    // End Terror with the keyboard; the map's targeting ends with it.
    tab_to(&mut app, "step:end-terror");
    press(&mut app, "Enter", "step:end-terror");
    let end = format!("{}:end-terror", revision + 1);
    tab_to(&mut app, &end);
    let ended = press(&mut app, "Enter", &end);
    assert_eq!(ended["observation"]["revision"], revision + 2);
    assert_eq!(ended["table"]["map_targets"], Value::Null);
    let order = tab_round(&mut app);
    no_space(&order, "after End Terror");
    app.capture("ended");
    app.quit();
}
