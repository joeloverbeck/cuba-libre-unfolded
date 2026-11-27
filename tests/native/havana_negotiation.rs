use super::{
    construct::choose_row,
    construct_transfers::{havana_casino, launch_recovered, published, underfunded_handoff},
    faction_seats::{DIRECTORIO, GOVERNMENT, M26, SYNDICATE, holding, seat},
    proclamation_composer::type_text,
    *,
};
use std::fs;
use unfolded_record_replacement::previous_path;

const START: [u8; 4] = [11, 4, 5, 3];
const FUNDED: [u8; 4] = [11, 2, 5, 5];
const PAID: [u8; 4] = [11, 2, 5, 0];
const OFFER: &str = "26July, lend us two Resources to reopen Havana.\nWhen you next ask for aid, we will hear you first.";
const ANSWER: &str = "We promise two Resources. Let Havana remember who helped turn the key.";
const WITNESS: &str = "Directorio heard the promise. The money is still with 26July.";
const RETURN: &str = "proclamation:return";
const FIELD: &str = "proclamation:field";

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; compares Havana negotiations through the real client"]
fn native_havana_numeric_request_and_public_bargain() {
    let mut app = super::available_forces::unlaunched("havana-negotiation-", "issue-233");
    underfunded_handoff(&mut app);
    retain_save(&app, "owner-start.json");
    fs::copy(
        previous_path(&app.save),
        app.root.join("owner-start.json.previous"),
    )
    .unwrap();
    app.capture("owner-start");

    ask_for_two(&mut app);
    let numeric = app.observe()["observation"].clone();
    let revision = app.revision();
    assert_eq!(numeric["owner"], "M26");
    assert_eq!(numeric["transfer"]["Authorizing"]["amount"], 2);
    retain_save(&app, "original-numeric-request.json");
    app.capture("numeric-request");

    let quiet = all_views(&mut app, "numeric", &json!([]), START, "Closed");
    let m26 = app.seat_agent(M26, "26july");
    app.decide(&m26, "authorize:2", "applied");
    let funded = app.observe()["observation"].clone();
    commit_payment(&mut app, None);
    let paid = app.observe()["observation"].clone();
    let silent_report = finish_and_read_report(&mut app, &m26, "silent-construct");
    assert_eq!(
        silent_report["articles"][0]["title"],
        "26July gives Syndicate 2 Resources"
    );
    assert_eq!(
        silent_report["articles"][0]["lines"][1],
        "26July 4 → 2, Syndicate 3 → 5."
    );
    assert_eq!(silent_report["articles"][1]["title"], "Construct in Havana");
    assert_eq!(
        silent_report["articles"][1]["lines"][0],
        "Syndicate pays 5 Resources, 5 → 0."
    );
    assert_eq!(
        observe_client(&mut app, &m26)["proclamations"]["published"],
        json!([])
    );
    app.quit();

    app.save = app.root.join("public-negotiation.json");
    fs::copy(app.root.join("original-numeric-request.json"), &app.save).unwrap();
    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], numeric);
    app.key("KeyP");
    app.activate("proclamation:author:syndicate", revision);
    type_text(&app, revision, OFFER);
    app.capture("own-explanation-in-composer");
    app.activate("proclamation:publish", revision);
    let m26 = app.seat_agent(M26, "26july");
    published(&mut app, &m26, revision, 18, ANSWER);
    assert_eq!(app.settle()["observation"], numeric);
    app.capture("public-answer-before-resources-move");
    app.key(M26);
    app.activate(RETURN, revision);
    let documents = json!([
        {"card":18,"author":"Syndicate","text":OFFER},
        {"card":18,"author":"M26","text":ANSWER}
    ]);
    let explained = all_views(&mut app, "explained", &documents, START, "Closed");
    for (bare, with_words) in quiet.iter().zip(&explained) {
        for facts in ["revision", "decision", "choices", "state", "results"] {
            assert_eq!(bare[facts], with_words[facts], "{facts}");
        }
    }
    assert_eq!(app.observe()["observation"], numeric);

    let before_answer = app.save_bytes();
    let m26 = app.seat_agent(M26, "26july");
    app.decide(&m26, "authorize:0", "applied");
    let refused = observe_client(&mut app, &m26);
    assert_eq!(refused["state"]["resources"], json!(START));
    assert_eq!(havana_casino(&refused)["state"], "Closed");
    assert_eq!(refused["proclamations"]["published"], documents);
    let refusal_record = app.save.clone();
    let refusal_save = app.save_bytes();
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), before_answer);
    let refused_report = refused["results"].as_array().unwrap().last().unwrap();
    assert_eq!(refused_report["articles"][0]["title"], "No Resources moved");
    assert_eq!(refused_report["articles"].as_array().unwrap().len(), 1);
    let blocked = app.observe()["observation"].clone();
    assert_eq!(blocked["construct_funding"]["shortfall"], 2);
    for action in ["construct:havana:open", "end-construct"] {
        let choice = blocked["choices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|choice| choice["id"] == format!("{}:{action}", revision + 1))
            .unwrap();
        assert_eq!(choice["supported"], false, "{choice}");
    }
    fs::write(
        app.root.join("bargain-refused.json"),
        serde_json::to_vec_pretty(&refused).unwrap(),
    )
    .unwrap();
    app.capture("promised-funding-refused");
    app.quit();

    // Recover from the exact prior record of the refused authorization. The two
    // alternative outcomes share the same public words and pending numeric request.
    let recovery_source = launch_recovered(&mut app, "recovered-request.json");
    assert_eq!(recovery_source, refusal_record);
    assert_eq!(app.save_bytes(), before_answer);
    assert_eq!(fs::read(&recovery_source).unwrap(), refusal_save);
    assert_eq!(
        fs::read(previous_path(&recovery_source)).unwrap(),
        before_answer
    );
    assert_eq!(app.observe()["observation"], numeric);
    app.activate("proclamations", revision);
    app.capture("recovered-original-syndicate-offer");
    app.key("Escape");

    let m26 = app.seat_agent(M26, "26july");
    app.decide(&m26, "authorize:2", "applied");
    assert_eq!(app.observe()["observation"], funded);
    app.key(M26);
    all_views(&mut app, "authorized", &documents, FUNDED, "Closed");
    app.capture("authorized-but-casino-still-closed");
    commit_payment(&mut app, Some("havana-payment-preview"));
    assert_eq!(app.observe()["observation"], paid);
    all_views(&mut app, "paid", &documents, PAID, "Open");
    app.capture("casino-open-after-separate-payment");
    let m26 = app.seat_agent(M26, "26july");
    let committed = finish_and_read_report(&mut app, &m26, "bargain-funded");
    assert_eq!(committed, silent_report);
    let before_quit = app.observe();
    let resumed = super::save_quit::save_quit_and_resume(&mut app, false);
    assert_eq!(
        resumed["table"]["proclamations"],
        before_quit["table"]["proclamations"]
    );
    let m26 = app.seat_agent(M26, "26july");
    assert_client_position(&mut app, &m26, &documents, PAID, "Open");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; exercises a live editor and held Havana choice"]
fn native_havana_draft_resumes_after_consecutive_proclamations() {
    let mut app = super::available_forces::unlaunched("havana-draft-", "issue-233");
    underfunded_handoff(&mut app);
    ask_for_two(&mut app);
    let m26 = app.seat_agent(M26, "26july");
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    let government = app.seat_agent(GOVERNMENT, "government");
    let mut quiet = app.player("quiet-government", &["wait", &government]);
    app.until_waiting("Government");

    let pending = app.observe()["observation"].clone();
    let revision = app.revision();
    app.key("KeyP");
    assert_eq!(
        app.observe()["table"]["proclamation_composer"]["author"],
        "Syndicate"
    );
    const DRAFT: &str = "Havana needs two.";
    type_text(&app, revision, DRAFT);
    // The short line ends above and to the left of the field's center. Drag
    // from that blank area to the leading edge of the line to select all its words.
    app.request(
        json!({"op":"drag","revision":revision,"component":FIELD,"button":"left","x":-920.,"y":-250.}),
        true,
    );
    app.capture("live-selected-syndicate-draft");

    published(&mut app, &m26, revision, 18, ANSWER);
    app.decide(&m26, "authorize:2", "held");
    published(&mut app, &directorio, revision, 18, WITNESS);
    let first = app.settle();
    assert_eq!(first["observation"], pending);
    assert_eq!(first["table"]["proclamations"]["open"], 1);
    assert_eq!(first["table"]["proclamations"]["unread"], 2);
    assert_eq!(
        first["table"]["proclamations"]["documents"][0]["text"],
        ANSWER
    );
    assert!(holding(&first).contains(&"a proclamation".to_string()));
    assert_eq!(seat(&first, "M26")["holds_choice"], true);
    assert!(quiet.ended().is_none());
    app.capture("first-proclamation-interrupts-draft");

    app.activate("proclamation:next", revision);
    let second = app.observe();
    assert_eq!(second["observation"], pending);
    assert_eq!(second["table"]["proclamations"]["open"], 2);
    assert_eq!(
        second["table"]["proclamations"]["documents"][0]["read"],
        true
    );
    assert_eq!(
        second["table"]["proclamations"]["documents"][1]["text"],
        WITNESS
    );
    assert_eq!(seat(&second, "M26")["holds_choice"], true);
    app.capture("second-proclamation-over-same-draft");

    // Reclaiming a seat invalidates its queued authorization. The new controller
    // has the same decision but can submit it only under its own identity.
    app.key(M26);
    assert_eq!(seat(&app.observe(), "M26")["holds_choice"], false);
    let (_, ended) = app.ask(&["observe", &m26]);
    assert_eq!(ended["status"], "closed");
    let replacement = app.seat_agent(M26, "26july");
    app.decide(&replacement, "authorize:2", "held");
    let restored = app.key("Escape");
    assert_eq!(restored["table"]["proclamations"]["unread"], 0);
    assert_eq!(restored["observation"], pending);
    assert_eq!(restored["table"]["proclamation_composer"]["draft"], DRAFT);
    assert_eq!(restored["table"]["proclamation_composer"]["focus"], FIELD);
    app.capture("restored-syndicate-draft");
    type_text(&app, revision, "Y");
    assert_eq!(
        app.observe()["table"]["proclamation_composer"]["draft"],
        "Y"
    );
    let refreshed = observe_client(&mut app, &government);
    assert_eq!(
        refreshed["proclamations"]["published"],
        json!([
            {"card":18,"author":"M26","text":ANSWER},
            {"card":18,"author":"Directorio","text":WITNESS}
        ])
    );
    assert!(quiet.ended().is_none());

    app.activate("proclamation:compose:return", revision);
    let funded = app.settle();
    assert_eq!(funded["observation"]["revision"], revision + 1);
    assert_eq!(funded["observation"]["resources"], json!(FUNDED));
    assert_eq!(seat(&funded, "M26")["holds_choice"], false);
    app.decide(&replacement, "authorize:2", "refused");
    assert_eq!(app.observe()["observation"], funded["observation"]);
    app.key(GOVERNMENT);
    let (_, closed) = app.answered(&mut quiet);
    assert_eq!(closed["status"], "closed");
    app.quit();
}

fn ask_for_two(app: &mut Native) {
    choose_row(app, "syndicate:limited-operation");
    choose_row(app, "transfer");
    app.choose("ask:1:2");
}

fn retain_save(app: &Native, name: &str) {
    fs::write(app.root.join(name), app.save_bytes()).unwrap();
}

fn observe_client(app: &mut Native, player: &str) -> Value {
    let (code, answer) = app.ask(&["observe", player]);
    assert_eq!(code, 0, "{answer}");
    answer["view"].clone()
}

fn all_views(
    app: &mut Native,
    label: &str,
    documents: &Value,
    resources: [u8; 4],
    casino: &str,
) -> Vec<Value> {
    [
        (GOVERNMENT, "government"),
        (M26, "26july"),
        (DIRECTORIO, "directorio"),
        (SYNDICATE, "syndicate"),
    ]
    .into_iter()
    .map(|(key, faction)| {
        let player = app.seat_agent(key, faction);
        let view = assert_client_position(app, &player, documents, resources, casino);
        fs::write(
            app.root.join(format!("{label}-{faction}.json")),
            serde_json::to_vec_pretty(&view).unwrap(),
        )
        .unwrap();
        app.key(key);
        view
    })
    .collect()
}

fn assert_client_position(
    app: &mut Native,
    player: &str,
    documents: &Value,
    resources: [u8; 4],
    casino: &str,
) -> Value {
    let view = observe_client(app, player);
    assert_eq!(view["proclamations"]["published"], *documents);
    assert_eq!(view["state"]["resources"], json!(resources));
    assert_eq!(havana_casino(&view)["state"], casino);
    view
}

fn commit_payment(app: &mut Native, capture: Option<&str>) {
    let revision = app.revision();
    app.press_by_keyboard("space:havana");
    let popup = app.observe();
    let row = &popup["table"]["popover"]["card"]["rows"][0];
    assert!(row["button"].as_str().unwrap().contains("Pay 5"), "{row}");
    if let Some(name) = capture {
        app.capture(name);
    }
    app.press_by_keyboard("popover:commit");
    assert_eq!(app.revision(), revision + 1);
}

fn finish_and_read_report(app: &mut Native, m26: &str, evidence: &str) -> Value {
    choose_row(app, "end-construct");
    let view = observe_client(app, m26);
    let report = view["results"].as_array().unwrap().last().unwrap();
    assert_eq!(report["recorded"], "Limited Operation");
    assert_eq!(report["complete"], true);
    fs::write(
        app.root.join(format!("{evidence}.json")),
        serde_json::to_vec_pretty(&view).unwrap(),
    )
    .unwrap();
    report.clone()
}
