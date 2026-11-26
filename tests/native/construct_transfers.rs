use super::{construct::choose_row, faction_seats, *};
use std::fs;
use unfolded_record_replacement::previous_path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; drives held seats and same-build reopening"]
fn native_public_proclamations_preserve_the_givers_decision() {
    use cuba_libre_unfolded::save::Session;
    use faction_seats::{DIRECTORIO, GOVERNMENT, M26, SYNDICATE, seat};
    use std::time::{Duration, Instant};

    let mut app = super::available_forces::unlaunched("held-publication-", "issue-229");
    let mut seed = Session::create_seeded(&app.save, 2).unwrap();
    for action in ["terror", "kidnap", "kidnap:roll", "terror:santiago"] {
        let seen = seed.observe();
        seed.submit(
            seen.owner,
            seen.revision,
            &format!("{}:{action}", seen.revision),
        )
        .unwrap();
    }
    drop(seed);
    app.launch_size("3840x2160");
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    let mut giver = app.player("giver-17", &["wait", &syndicate]);
    let mut idle = app.player("idle-17", &["wait", &directorio]);
    app.until_waiting("Syndicate");
    app.until_waiting("Directorio");
    app.choose("transfer");
    app.choose("ask:3:2");
    let (code, wake) = app.answered(&mut giver);
    assert_eq!((code, &wake["status"]), (0, &json!("decision")));
    let government = app.seat_agent(GOVERNMENT, "government");
    let m26 = app.seat_agent(M26, "26july");
    let before = wake["view"].clone();
    assert_eq!(before["decision"]["yours"], true);
    assert_eq!(before["decision"]["faction"], "Syndicate");
    assert_eq!(before["state"]["resources"], json!([1, 4, 5, 15]));
    let revision = before["revision"].as_u64().unwrap();
    let revision_arg = revision.to_string();
    let pending = app.observe()["observation"].clone();
    let saved = app.save_bytes();
    let previous = previous_path(&app.save);
    let kept = fs::read(&previous).unwrap();
    app.capture("card-17-giver-before-publication");

    // 400 Unicode scalar values, with combining marks and UTF-8 beyond one byte.
    // The fixture includes command-like text and whitespace that must survive.
    let opening = "--  Havana, mañana.\n\nWe offer two Resources.\n";
    let wording = format!(
        "{opening}{}! ",
        "e\u{301}".repeat((398 - opening.chars().count()) / 2)
    );
    let wording = format!("{wording}{}", " ".repeat(400 - wording.chars().count()));
    assert_eq!(wording.chars().count(), 400);
    assert!(wording.len() > 400);
    for (id, at, card, text, expected) in [
        (
            &syndicate[..],
            revision,
            17,
            &format!("{wording}x")[..],
            "400",
        ),
        (&syndicate, revision - 1, 17, &wording, "Stale"),
        (&syndicate, revision, 18, &wording, "Stale"),
        ("unseated", revision, 17, &wording, "holds a seat"),
    ] {
        refused_publication(&mut app, id, at, card, text, expected);
        assert_eq!(app.save_bytes(), saved);
        assert_eq!(fs::read(&previous).unwrap(), kept);
    }

    // Refuse the write while another writer holds the ordinary save lock.
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(unfolded_record_replacement::lock_path(&app.save))
        .unwrap();
    lock.try_lock().unwrap();
    let blocked = refused_publication(&mut app, &syndicate, revision, 17, &wording, "save is busy");
    assert_eq!(blocked["view"]["proclamations"]["published"], json!([]));
    assert_eq!(blocked["view"]["proclamations"]["can_publish"], true);
    assert_eq!(app.save_bytes(), saved);
    assert_eq!(fs::read(&previous).unwrap(), kept);
    assert_eq!(app.observe()["observation"], pending);
    app.capture("save-busy-before-publication");
    lock.unlock().unwrap();

    // Discard the original command's output as soon as persistence proves it wrote.
    let lost = app.player(
        "giver-17",
        &["publish", &syndicate, &revision_arg, "17", &wording],
    );
    let end = Instant::now() + Duration::from_secs(30);
    while app.save_bytes() == saved {
        app.settle();
        assert!(Instant::now() < end, "publication never reached the save");
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(lost);
    let written = app.save_bytes();
    assert_eq!(fs::read(&previous).unwrap(), saved);
    let repeated = refused_publication(
        &mut app,
        &syndicate,
        revision,
        17,
        &wording,
        "already published",
    );
    assert_eq!(app.save_bytes(), written);
    let words = json!([{"card": 17, "author": "Syndicate", "text": wording}]);
    assert_eq!(repeated["view"]["proclamations"]["can_publish"], false);
    for field in ["revision", "decision", "choices", "state", "results"] {
        assert_eq!(repeated["view"][field], before[field], "{field}");
    }
    assert_eq!(app.observe()["observation"], pending);
    app.capture("card-17-giver-after-publication");
    for player in [&government, &m26, &directorio, &syndicate] {
        let (code, refreshed) = app.ask(&["observe", player]);
        assert_eq!(code, 0);
        assert_eq!(refreshed["view"]["proclamations"]["published"], words);
        let faction = refreshed["view"]["faction"].as_str().unwrap();
        let public_view = &refreshed["view"]["proclamations"];
        fs::write(
            app.root.join(format!("card-17-public-{faction}.json")),
            serde_json::to_vec_pretty(public_view).unwrap(),
        )
        .unwrap();
    }
    assert!(idle.ended().is_none(), "publication woke the idle seat");
    assert_eq!(seat(&app.settle(), "Directorio")["waits"], true);

    assert_eq!(app.settle()["table"]["proclamations"]["open"], 1);
    app.key("Escape");
    app.key(SYNDICATE);
    let (code, ended) = app.ask(&["publish", &syndicate, &revision_arg, "17", "Again"]);
    assert_eq!((code, &ended["status"]), (0, &json!("closed")));
    let successor = app.seat_agent(SYNDICATE, "syndicate");
    assert_ne!(successor, syndicate);
    refused_publication(
        &mut app,
        &successor,
        revision,
        17,
        "Again",
        "already published",
    );
    assert_eq!(app.save_bytes(), written);
    app.key(M26);
    app.key(GOVERNMENT);
    // The game follows the checked amount, even though the published offer said two.
    app.decide(&successor, "authorize:0", "applied");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([1, 4, 5, 15])
    );
    app.choose("end-terror");
    let (code, next) = app.answered(&mut idle);
    assert_eq!((code, &next["status"]), (0, &json!("decision")));
    assert_eq!(next["view"]["decision"]["faction"], "Directorio");
    assert_eq!(next["view"]["proclamations"]["published"], words);
    client_applies(&mut app, &directorio, "event:shaded");
    client_applies(&mut app, &directorio, "resolve-shaded");
    let (code, silent) = app.ask(&["observe", &directorio]);
    assert_eq!(code, 0);
    assert_eq!(silent["view"]["proclamations"]["published"], words);
    // The committed Event earns the ordinary edition. The received speech
    // had already been read, and the silent Event still earns its edition.
    assert!(!app.settle()["table"]["newspaper"]["open"].is_null());
    app.key("Escape");
    app.key(DIRECTORIO);
    let closed = app.revision();
    refused_publication(&mut app, &successor, closed, 17, "Again", "card is closed");
    app.choose("advance-card");
    let opened = app.revision();
    let (_, new_card) = app.ask(&["observe", &successor]);
    assert_eq!(new_card["view"]["state"]["current_card"]["number"], 18);
    assert_eq!(new_card["view"]["proclamations"]["published"], words);
    assert_eq!(new_card["view"]["proclamations"]["can_publish"], true);
    refused_publication(&mut app, &successor, opened, 17, "Old card", "Stale");
    // Government needs the human's Syndicate to answer its earlier request.
    app.key(SYNDICATE);
    close_havana_with_assault(&mut app);
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let syndicate_revision = app.revision();
    let renewed = published(
        &mut app,
        &syndicate,
        syndicate_revision,
        18,
        "We seek to reopen Havana.",
    );
    assert_eq!(renewed["view"]["decision"]["yours"], true);
    assert_eq!(
        renewed["view"]["proclamations"]["published"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(app.settle()["table"]["proclamations"]["open"], 2);
    app.key("Escape");
    app.key(SYNDICATE);

    let m26 = app.seat_agent(M26, "26july");
    let mut giver = app.player("giver-18", &["wait", &m26]);
    app.until_waiting("M26");
    choose_row(&mut app, "syndicate:limited-operation");
    choose_row(&mut app, "transfer");
    app.choose("ask:1:2");
    let (code, wake) = app.answered(&mut giver);
    assert_eq!((code, &wake["status"]), (0, &json!("decision")));
    let before = &wake["view"];
    assert_eq!(before["decision"]["faction"], "M26");
    assert_eq!(before["decision"]["yours"], true);
    assert_eq!(before["state"]["resources"], json!([11, 4, 5, 3]));
    assert_eq!(havana_casino(before)["state"], "Closed");
    let revision = before["revision"].as_u64().unwrap();
    let pending = app.observe()["observation"].clone();
    app.capture("havana-closed-with-giver-decision");
    let offered = published(
        &mut app,
        &m26,
        revision,
        18,
        "Havana may count on us for two Resources.",
    );
    let offered_view = &offered["view"];
    for field in ["revision", "decision", "choices", "state", "results"] {
        assert_eq!(offered_view[field], before[field], "{field}");
    }
    assert_eq!(
        offered_view["proclamations"]["published"][2]["author"],
        "M26"
    );
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(app.settle()["table"]["proclamations"]["open"], 3);
    app.key("Escape");
    let saved = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.save_bytes(), saved);
    assert_eq!(app.observe()["observation"], pending);
    let resumed = app.seat_agent(M26, "26july");
    let (code, after) = app.ask(&["observe", &resumed]);
    assert_eq!(code, 0);
    for field in [
        "revision",
        "decision",
        "choices",
        "state",
        "results",
        "proclamations",
    ] {
        assert_eq!(after["view"][field], offered_view[field], "{field}");
    }
    app.capture("havana-closed-with-restored-giver-decision");
    let authorized = app.decide(&resumed, "authorize:2", "applied");
    assert_eq!(
        authorized["view"]["state"]["resources"],
        json!([11, 2, 5, 5])
    );
    assert_eq!(havana_casino(&authorized["view"])["state"], "Closed");
    app.press_by_keyboard("space:havana");
    app.press_by_keyboard("popover:commit");
    let paid = app.observe();
    assert_eq!(paid["observation"]["resources"], json!([11, 2, 5, 0]));
    // The five-Resource payment is a separate choice from authorization.
    let (_, seen) = app.ask(&["observe", &resumed]);
    assert_eq!(havana_casino(&seen["view"])["state"], "Open");
    app.capture("havana-reopened-by-the-construct-payment");
    for (name, answer) in [
        ("card-17-silent-seat-decision", next),
        ("card-17-silent-seat-resolves-event", silent),
        ("card-17-retry", repeated),
        ("card-18-giver-publication", offered),
        ("card-18-giver-reopened", after),
    ] {
        fs::write(
            app.root.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&answer).unwrap(),
        )
        .unwrap();
    }
    app.quit();
}

fn published(app: &mut Native, player: &str, revision: u64, card: u8, text: &str) -> Value {
    let (code, answer) = app.ask(&[
        "publish",
        player,
        &revision.to_string(),
        &card.to_string(),
        text,
    ]);
    assert_eq!(
        (code, &answer["status"]),
        (0, &json!("published")),
        "{answer}"
    );
    answer
}

fn refused_publication(
    app: &mut Native,
    player: &str,
    revision: u64,
    card: u8,
    text: &str,
    reason: &str,
) -> Value {
    let (code, answer) = app.ask(&[
        "publish",
        player,
        &revision.to_string(),
        &card.to_string(),
        text,
    ]);
    assert_eq!(
        (code, &answer["status"]),
        (1, &json!("refused")),
        "{answer}"
    );
    assert!(
        answer["error"].as_str().unwrap().contains(reason),
        "{answer}"
    );
    answer
}

fn havana_casino(view: &Value) -> &Value {
    let havana = view["state"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|space| space["id"] == "havana")
        .unwrap();
    havana["pieces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["kind"] == "Casino")
        .unwrap()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained transfer and Construct evidence"]
fn native_construct_transfer_resumes_at_the_giver_and_finishes_after_spending() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-220"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("construct-transfer-")
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
    eprintln!("Construct transfer evidence: {}", app.root.display());
    underfunded_handoff(&mut app);
    let handoff = app.save_bytes();

    choose_row(&mut app, "syndicate:limited-operation");
    super::space_reason::hover(
        &mut app,
        "havana",
        "blocked",
        Some("A Resource transfer opens it"),
        "construct-havana-blocked",
    );
    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:ask", revision);
    app.type_amount("2");
    let before = app.observe();
    assert_eq!(
        before["table"]["transfer"]["state"]["commit"],
        format!("{revision}:ask:1:2")
    );
    app.capture("syndicate-asks-26july");
    app.choose("ask:1:2");
    let asked = app.observe();
    assert_eq!(asked["observation"]["owner"], "M26");
    assert_eq!(asked["observation"]["executing"], "Syndicate");
    assert_eq!(asked["observation"]["resources"], json!([11, 4, 5, 3]));
    app.capture("giver-previews-two");
    app.choose("authorize:2");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 2, 5, 5])
    );
    app.press_by_keyboard("space:havana");
    app.press_by_keyboard("popover:commit");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 2, 5, 0])
    );
    choose_row(&mut app, "end-construct");
    app.capture("construct-paid-with-two-authorized");
    app.quit();

    app.save = app.root.join("three-resources.json");
    fs::write(&app.save, handoff).unwrap();
    app.launch_size("3840x2160");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );

    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "syndicate:limited-operation");
    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:1:2");
    let interrupted = app.observe()["table"]["newspaper"]["open"].clone();
    let report = &interrupted["report"];
    assert_eq!(report["action"], "construct");
    assert_eq!(report["faction"], "Syndicate");
    assert_eq!(report["complete"], false);
    assert!(
        report["summary"]
            .as_str()
            .unwrap()
            .contains("asks 26July for 2 Resources")
    );
    assert_eq!(report["articles"], json!([]));
    fs::write(
        app.root.join("interrupted-edition.json"),
        serde_json::to_vec_pretty(&interrupted).unwrap(),
    )
    .unwrap();
    app.capture("edition-interrupts-at-giver");
    app.key("Escape");
    app.choose("wait");
    let pending = app.observe()["observation"].clone();
    let pending_bytes = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(app.save_bytes(), pending_bytes);
    assert_eq!(app.observe()["table"]["transfer"]["state"]["pending"], true);
    app.choose("reopen");
    app.type_amount("3");
    let preview = app.observe();
    assert_eq!(
        preview["table"]["transfer"]["state"]["label"],
        "26July authorizes 3"
    );
    assert_eq!(preview["observation"]["resources"], json!([11, 4, 5, 3]));
    app.capture("giver-previews-three");
    app.choose("wait");
    let giver = app.seat_agent(faction_seats::M26, "26july");
    let (code, observed) = app.ask(&["observe", &giver]);
    assert_eq!(code, 0, "{observed}");
    assert_eq!(observed["view"]["decision"]["faction"], "M26");
    let terms = &observed["view"]["state"]["transfer_terms"][0];
    assert_eq!(terms["giver"], "M26");
    assert_eq!(terms["recipient"], "Syndicate");
    assert_eq!(terms["limit"], 4);
    app.decide(&giver, "authorize:5", "refused");
    let stale = app.revision().to_string();
    let choice = format!("{stale}:authorize:3");
    let applied = client_applies(&mut app, &giver, "authorize:3");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 1, 5, 6])
    );
    assert_eq!(app.observe()["observation"]["owner"], "Syndicate");
    let authorized_bytes = app.save_bytes();
    let (code, refused) = app.ask(&["choose", &giver, &stale, &choice]);
    assert_eq!(code, 1, "{refused}");
    assert!(
        refused["error"].as_str().unwrap().contains("Stale"),
        "{refused}"
    );
    assert_eq!(app.save_bytes(), authorized_bytes);
    fs::write(
        app.root.join("giver-reply.json"),
        serde_json::to_vec_pretty(&applied).unwrap(),
    )
    .unwrap();

    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:give", revision);
    app.type_amount("2");
    let blocked = app.observe();
    let state = &blocked["table"]["transfer"]["state"];
    assert_eq!(state["limit"], 1);
    assert_eq!(state["commit"], Value::Null);
    assert!(state["refusal"].as_str().unwrap().contains("Pay first"));
    assert_eq!(blocked["observation"]["resources"], json!([11, 1, 5, 6]));
    app.capture("outgoing-transfer-refused-until-payment");
    app.key("Escape");

    let payment = app.press_by_keyboard("space:havana");
    let plan = &payment["table"]["popover"]["card"]["rows"][0];
    assert!(
        plan["lines"]
            .to_string()
            .contains("Syndicate Resources 6 → 1.")
    );
    app.capture("havana-payment-preview");
    app.press_by_keyboard("popover:commit");
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([11, 1, 5, 1]));
    assert_eq!(paid["construct_paid"][0]["space"], "havana");
    assert_eq!(paid["construct_paid"][0]["plan"], "Open");
    assert_eq!(paid["standings"][3]["score"], 3);
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], paid);
    choose_row(&mut app, "transfer");
    app.choose("give:1:1");
    let before_finish = app.observe()["observation"].clone();
    assert_eq!(before_finish["resources"], json!([11, 2, 5, 0]));
    let previous = app.save_bytes();
    fs::write(app.root.join("before-finish.json"), &previous).unwrap();
    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "end-construct");
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    let (code, told) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{told}");
    let report = told["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["faction"], "Syndicate");
    assert_eq!(report["recorded"], "Limited Operation");
    assert_eq!(report["complete"], true);
    let articles = report["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "26July gives Syndicate 3 Resources");
    assert_eq!(articles[0]["lines"][1], "26July 4 → 1, Syndicate 3 → 6.");
    assert_eq!(articles[1]["title"], "Construct in Havana");
    assert_eq!(
        articles[1]["lines"][0],
        "Syndicate pays 5 Resources, 6 → 1."
    );
    assert_eq!(articles[2]["title"], "Syndicate gives 26July 1 Resource");
    assert_eq!(articles[2]["lines"][0], "Syndicate 1 → 0, 26July 1 → 2.");
    fs::write(
        app.root.join("faction-client-after-finish.json"),
        serde_json::to_vec_pretty(&told).unwrap(),
    )
    .unwrap();
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["eligible"],
        json!([false, true, true, false])
    );
    assert_eq!(finished["observation"]["resources"], json!([11, 2, 5, 0]));
    assert_eq!(
        finished["table"]["newspaper"]["open"]["report"]["complete"],
        true
    );
    app.capture("construct-finish-edition");
    let terminal = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], finished["observation"]);
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    app.quit();

    let source = launch_recovered(&mut app);
    assert_eq!(app.save_bytes(), previous);
    assert_eq!(app.observe()["observation"], before_finish);
    choose_row(&mut app, "end-construct");
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    assert_eq!(fs::read(source).unwrap(), terminal);
    app.quit();
}

fn client_applies(app: &mut Native, player: &str, action: &str) -> Value {
    let before = app.revision();
    let (code, reply) = app.ask(&[
        "choose",
        player,
        &before.to_string(),
        &format!("{before}:{action}"),
    ]);
    assert_eq!(code, 0, "{action}: {reply}");
    // A submission may arrive while the table still has a frame to settle.
    assert!(
        matches!(reply["status"].as_str(), Some("applied" | "held")),
        "{action}: {reply}"
    );
    assert_eq!(
        app.settle()["observation"]["revision"],
        before + 1,
        "{action}"
    );
    reply
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; exercise withdrawal from the pending native panel"]
fn native_syndicate_withdraws_its_pending_limited_construct_request() {
    for held in [false, true] {
        let mut app = super::available_forces::unlaunched("withdraw-construct-", "issue-215");
        underfunded_handoff(&mut app);
        choose_row(&mut app, "syndicate:limited-operation");
        if held {
            choose_row(&mut app, "transfer");
            app.choose("ask:1:2");
            app.choose("authorize:1");
        }

        let directorio = held.then(|| app.seat_agent(faction_seats::DIRECTORIO, "directorio"));
        choose_row(&mut app, "transfer");
        app.choose("ask:2:5");
        if directorio.is_none() {
            app.choose("wait");
        }
        let pending = app.observe();
        assert_eq!(pending["table"]["transfer"]["state"]["pending"], true);
        assert_eq!(pending["observation"]["owner"], "Directorio");
        assert_eq!(pending["observation"]["resources"], json!([11, 4, 5, 3]));
        let pending_revision = app.revision();
        if let Some(player) = &directorio {
            let (code, seen) = app.ask(&["observe", player]);
            assert_eq!(code, 0, "{seen}");
            let choices = seen["view"]["choices"].to_string();
            assert!(choices.contains(&format!("{pending_revision}:authorize:2")));
            assert!(!choices.contains("back-to-choice"));
        }
        let withdraw = format!("{pending_revision}:back-to-choice");
        assert_eq!(
            choice_part(&pending, "option:back-to-choice-label")["text"],
            "Syndicate withdraws · back to action choice"
        );
        assert!(
            pending["table"]["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["name"] == withdraw),
            "Syndicate needs a native withdrawal control"
        );
        app.capture(if held {
            "held-pending-withdrawal"
        } else {
            "pending-withdrawal"
        });
        if held {
            app.press_by_keyboard(&withdraw);
        } else {
            app.request(
                json!({"op":"activate", "choice":withdraw, "revision":pending_revision}),
                true,
            );
            app.settle();
        }
        let withdrew = app.observe();
        assert_eq!(withdrew["observation"]["owner"], "Syndicate");
        assert!(withdrew["observation"]["executing"].is_null());
        assert!(withdrew["observation"]["transfer"].is_null());
        assert!(withdrew["observation"]["construct_funding"].is_null());
        assert_eq!(withdrew["observation"]["resources"], json!([11, 4, 5, 3]));
        let saved = app.save_bytes();
        if let Some(player) = &directorio {
            let (code, reply) = app.ask(&[
                "choose",
                player,
                &pending_revision.to_string(),
                &format!("{pending_revision}:authorize:2"),
            ]);
            assert_eq!(code, 1, "{reply}");
            assert!(reply["error"].as_str().unwrap().contains("Stale"));
        }
        assert_eq!(app.save_bytes(), saved);
        app.capture("syndicate-choice-restored");
        choose_row(&mut app, "syndicate:limited-operation");
        choose_row(&mut app, "transfer");
        app.choose("ask:1:4");
        assert_eq!(
            app.observe()["observation"]["transfer_terms"][0]["limit"],
            4
        );
        app.quit();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect pending funding and settlement"]
fn native_partial_construct_funding_survives_a_pending_reply_and_finishes_from_recovery() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-221"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("partial-funding-")
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
    eprintln!("Partial Construct funding evidence: {}", app.root.display());
    underfunded_handoff(&mut app);
    choose_row(&mut app, "syndicate:limited-operation");
    choose_row(&mut app, "transfer");
    let revision = app.revision();
    app.activate("transfer:tab:26july", revision);
    app.activate("transfer:direction:ask", revision);
    app.type_amount("2");
    let draft = app.observe();
    let condition = "Resources move only when authorizations fund a legal payment.";
    let modal_condition = "Resources move if authorizations fund a legal payment.";
    assert!(transfer_part(&draft, "construct-funding").contains(modal_condition));
    assert!(
        transfer_part(&draft, "construct-funding")
            .contains("0 conditionally authorized · 2 still needed")
    );
    app.capture("syndicate-conditional-request");
    app.choose("ask:1:2");
    app.type_amount("1");
    let first = app.observe();
    assert_eq!(first["observation"]["owner"], "M26");
    assert_eq!(
        first["table"]["transfer"]["state"]["label"],
        "26July authorizes 1"
    );
    assert_eq!(
        transfer_part(&first, "construct-funding"),
        transfer_part(&draft, "construct-funding")
    );
    app.capture("26july-previews-one");
    app.choose("authorize:1");
    let held = app.observe();
    assert_eq!(held["observation"]["resources"], json!([11, 4, 5, 3]));
    assert_eq!(held["observation"]["owner"], "Syndicate");
    assert_eq!(
        held["table"]["notice"],
        "26July conditionally authorized 1 Resource for Syndicate. Syndicate still needs 1 Resource. Saved."
    );
    assert_eq!(
        held["observation"]["construct_funding"],
        json!({
            "authorizations": [{"giver": "M26", "asked": 2, "amount": 1}],
            "shortfall": 1
        })
    );
    assert!(
        held["table"]["choice_parts"]
            .to_string()
            .contains("1 conditionally authorized · 1 still needed")
    );
    app.capture("unspent-authorization");

    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    let (code, seen) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{seen}");
    assert_eq!(
        seen["view"]["state"]["construct_funding"],
        held["observation"]["construct_funding"]
    );
    let held_report = seen["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(held_report["articles"], json!([]));
    assert_eq!(
        held_report["summary"],
        format!("1 conditionally authorized · 1 still needed. {condition}")
    );

    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:1:3");
    let refusing = app.observe();
    assert_eq!(refusing["observation"]["transfer_terms"][0]["limit"], 3);
    app.key("Escape"); // Close the interrupted edition and answer as 26July.
    app.type_amount("5");
    let refused = app.observe();
    let transfer = &refused["table"]["transfer"]["state"];
    assert!(transfer["commit"].is_null());
    assert_eq!(refused["observation"]["owner"], "M26");
    assert!(
        transfer["refusal"]
            .as_str()
            .unwrap()
            .contains("already conditionally authorized 1 Resource")
    );
    let title = drawn_transfer_part(&refused, "title");
    assert_eq!(title["text"], "26JULY DECIDES");
    let label = drawn_transfer_part(&refused, "primary-label");
    assert_eq!(label["text"], "Authorize 5");
    assert_eq!(label["lines"], 1);
    let status = drawn_transfer_part(&refused, "construct-funding");
    assert_eq!(
        status["text"],
        format!("1 conditionally authorized · 1 still needed\n{modal_condition}")
    );
    assert_eq!(status["lines"], 2);
    let primary = drawn_transfer_part(&refused, "button:primary");
    let wait = drawn_transfer_part(&refused, "button:wait");
    let modal = drawn_transfer_part(&refused, "transfer-modal");
    let [l, t, r, b] = rect_of(&label["bounds"]);
    let [pl, pt, pr, pb] = rect_of(&primary["bounds"]);
    assert!(l > pl && t > pt && r < pr && b < pb, "{label} in {primary}");
    let [l, _, r, b] = rect_of(&status["bounds"]);
    let [ml, _, _, mb] = rect_of(&modal["bounds"]);
    let [wl, ..] = rect_of(&wait["bounds"]);
    assert!(
        l > ml && r < wl && b < mb,
        "{status} beside {wait} in {modal}"
    );
    fs::write(
        app.root.join("same-giver-copy-bounds.json"),
        serde_json::to_vec_pretty(&json!([title, modal, wait, primary, label, status])).unwrap(),
    )
    .unwrap();
    app.capture("same-giver-signed-refusal");
    app.choose("decline");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );
    app.key("Escape");

    client_applies(&mut app, &syndicate, "transfer");
    client_applies(&mut app, &syndicate, "ask:2:5");
    let edition = app.observe()["table"]["newspaper"].clone();
    let interrupted = &edition["open"];
    assert_eq!(interrupted["report"]["faction"], "Syndicate");
    let summary = interrupted["report"]["summary"].as_str().unwrap();
    assert!(summary.contains("Directorio decides"));
    assert!(summary.contains("1 conditionally authorized · 1 still needed"));
    assert!(summary.contains(condition));
    assert!(
        !interrupted["report"]["articles"]
            .to_string()
            .contains("gives Syndicate 1 Resource")
    );
    let parts = edition["parts"].as_array().unwrap();
    let part = |name| parts.iter().find(|p| p["part"] == name).unwrap();
    let title = part("edition-block-0-title");
    let count = part("edition-block-0-text-0");
    fs::write(
        app.root.join("interruption-copy-bounds.json"),
        serde_json::to_vec_pretty(&json!([title, count])).unwrap(),
    )
    .unwrap();
    app.capture("edition-pending-directorio");
    assert_eq!(title["text"], "No Resources moved");
    assert_eq!(count["text"], "26July answered 1 request.");
    for part in [title, count] {
        assert_eq!(part["lines"], 1, "{part}");
        let [l, _, r, _] = rect_of(&part["ink"]);
        let [bl, _, br, _] = rect_of(&part["bounds"]);
        assert!(l >= bl - 0.5 && r <= br + 0.5, "{part}");
    }
    app.key("Escape");
    app.choose("wait");
    let pending = app.observe();
    assert_eq!(pending["observation"]["owner"], "Directorio");
    let withdraw = format!("{}:back-to-choice", app.revision());
    assert!(
        !pending["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["name"] == withdraw),
        "the human giver cannot withdraw on behalf of the Syndicate agent"
    );
    assert_eq!(
        transfer_part(&pending, "pending-construct-funding"),
        "1 held · +1 moves all"
    );
    assert_eq!(
        transfer_part(&pending, "pending-construct-giver"),
        "Directorio decides · 5 requested"
    );
    let bounds = |name| {
        let part = drawn_transfer_part(&pending, name);
        let limits = part["bounds"].as_array().unwrap();
        std::array::from_fn::<_, 4, _>(|i| limits[i].as_f64().unwrap())
    };
    let [left, top, right, bottom] = bounds("pending-tab");
    for name in ["pending-construct-funding", "pending-construct-giver"] {
        let [l, t, r, b] = bounds(name);
        assert!(l > left && t > top && r < right && b < bottom, "{name}");
    }
    app.capture("wait-on-directorio");
    let pending_bytes = app.save_bytes();
    fs::write(app.root.join("pending-before-restart.json"), &pending_bytes).unwrap();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.save_bytes(), pending_bytes);
    assert_eq!(app.observe()["observation"], pending["observation"]);
    let directorio = app.seat_agent(faction_seats::DIRECTORIO, "directorio");
    let (code, seen) = app.ask(&["observe", &directorio]);
    assert_eq!(code, 0, "{seen}");
    let decision = &seen["view"]["decision"];
    assert_eq!(decision["faction"], "Directorio");
    assert_eq!(decision["yours"], true);
    assert!(decision["note"].as_str().unwrap().contains(condition));
    assert_eq!(
        seen["view"]["state"]["construct_funding"],
        held["observation"]["construct_funding"]
    );
    let revision = app.revision().to_string();
    let previous_choice = format!("{revision}:authorize:2");
    let reply = client_applies(&mut app, &directorio, "authorize:2");
    fs::write(
        app.root.join("directorio-client-settlement.json"),
        serde_json::to_vec_pretty(&reply).unwrap(),
    )
    .unwrap();
    let settled = app.observe();
    assert_eq!(settled["observation"]["resources"], json!([11, 3, 3, 6]));
    assert_eq!(
        settled["table"]["notice"],
        "Directorio authorized 2 Resources. Syndicate received 3 Resources from the combined authorizations. Saved."
    );
    assert_eq!(settled["observation"]["owner"], "Syndicate");
    assert_eq!(settled["observation"]["construct_funding"], Value::Null);
    assert_eq!(settled["observation"]["construct_paid"], json!([]));
    let settled_bytes = app.save_bytes();
    let (code, stale) = app.ask(&["choose", &directorio, &revision, &previous_choice]);
    assert_eq!(code, 1, "{stale}");
    assert!(stale["error"].as_str().unwrap().contains("Stale"));
    app.decide(&directorio, "authorize:2", "refused");
    assert_eq!(app.save_bytes(), settled_bytes);
    app.key("Escape");

    let payment = app.press_by_keyboard("space:havana");
    assert!(
        payment["table"]["popover"]["card"]["rows"][0]["lines"]
            .to_string()
            .contains("Syndicate Resources 6 → 1.")
    );
    app.capture("settled-funding-havana-preview");
    app.press_by_keyboard("popover:commit");
    let paid = app.observe()["observation"].clone();
    assert_eq!(paid["resources"], json!([11, 3, 3, 1]));
    assert_eq!(paid["construct_paid"][0]["space"], "havana");
    assert_eq!(paid["construct_paid"][0]["plan"], "Open");
    let paid_bytes = app.save_bytes();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], paid);
    assert_eq!(app.save_bytes(), paid_bytes);
    choose_row(&mut app, "transfer");
    app.choose("give:1:1");
    let before_finish = app.observe()["observation"].clone();
    assert_eq!(before_finish["resources"], json!([11, 4, 3, 0]));
    let previous = app.save_bytes();
    fs::write(app.root.join("before-finish.json"), &previous).unwrap();
    let syndicate = app.seat_agent(faction_seats::SYNDICATE, "syndicate");
    client_applies(&mut app, &syndicate, "end-construct");
    let (code, told) = app.ask(&["observe", &syndicate]);
    assert_eq!(code, 0, "{told}");
    let report = told["view"]["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["recorded"], "Limited Operation");
    let articles = report["articles"].as_array().unwrap();
    assert_eq!(articles[0]["title"], "No Resources moved");
    assert_eq!(articles[1]["title"], "26July gives Syndicate 1 Resource");
    assert_eq!(articles[1]["lines"][1], "26July 4 → 3, Syndicate 3 → 4.");
    assert_eq!(
        articles[2]["title"],
        "Directorio gives Syndicate 2 Resources"
    );
    assert_eq!(
        articles[2]["lines"][1],
        "Directorio 5 → 3, Syndicate 4 → 6."
    );
    assert_eq!(articles[3]["title"], "Construct in Havana");
    assert_eq!(
        articles[3]["lines"][0],
        "Syndicate pays 5 Resources, 6 → 1."
    );
    assert_eq!(articles[4]["title"], "Syndicate gives 26July 1 Resource");
    fs::write(
        app.root.join("syndicate-client-after-finish.json"),
        serde_json::to_vec_pretty(&told).unwrap(),
    )
    .unwrap();
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["eligible"],
        json!([false, true, true, false])
    );
    assert_eq!(finished["observation"]["resources"], json!([11, 4, 3, 0]));
    assert_eq!(
        finished["table"]["newspaper"]["open"]["report"]["complete"],
        true
    );
    app.capture("partial-funding-finish-edition");
    let terminal = app.save_bytes();
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], finished["observation"]);
    assert_eq!(app.save_bytes(), terminal);
    app.quit();
    let source = launch_recovered(&mut app);
    assert_eq!(app.save_bytes(), previous);
    assert_eq!(app.observe()["observation"], before_finish);
    choose_row(&mut app, "end-construct");
    assert_eq!(app.save_bytes(), terminal);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    assert_eq!(fs::read(previous_path(&source)).unwrap(), previous);
    assert_eq!(fs::read(source).unwrap(), terminal);
    app.quit();
}

fn transfer_part<'a>(seen: &'a Value, name: &str) -> &'a str {
    drawn_transfer_part(seen, name)["text"].as_str().unwrap()
}

fn drawn_transfer_part<'a>(seen: &'a Value, name: &str) -> &'a Value {
    seen["table"]["transfer"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["part"] == name)
        .unwrap_or_else(|| panic!("{name} not drawn"))
}

pub(super) fn underfunded_handoff(app: &mut Native) {
    super::assault::fund_santiago(&app.save, 0);
    app.launch_size("3840x2160");
    for action in ["event:shaded", "resolve-shaded"] {
        choose_row(app, action);
    }
    app.choose("advance-card");
    close_havana_with_assault(app);
}

fn close_havana_with_assault(app: &mut Native) {
    choose_row(app, "government:assault");
    choose_row(app, "transfer");
    app.choose("ask:3:12");
    app.choose("authorize:12");
    app.choose("assault:havana:syndicate");
    choose_row(app, "end-assault");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([11, 4, 5, 3])
    );
}

pub(super) fn launch_recovered(app: &mut Native) -> PathBuf {
    let source = app.save.clone();
    app.save = app.root.join("recovered.json");
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--save",
            app.save.to_str().unwrap(),
            "--recover-from",
            source.to_str().unwrap(),
            "--agent-seat",
            "--seat-size",
            "3840x2160",
            "--socket",
            app.socket.to_str().unwrap(),
            "--player-socket",
            app.player_socket().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(app.root.join("recovery-launch.jsonl"), output.stdout).unwrap();
    source
}
