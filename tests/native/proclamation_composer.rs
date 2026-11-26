use super::*;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; drives the native proclamation composer"]
fn native_human_composes_a_proclamation_for_the_selected_faction() {
    let mut app = super::available_forces::start("broadsheet-compose-", "issue-230", "3840x2160");
    let pending = app.observe()["observation"].clone();
    let revision = pending["revision"].as_u64().unwrap();
    app.activate("proclamation:write", revision);
    let open = app.observe();
    assert_eq!(open["table"]["proclamation_composer"]["author"], "M26");
    assert_eq!(open["table"]["proclamation_composer"]["limit"], 400);
    assert_eq!(open["table"]["proclamation_composer"]["spent"], false);
    app.activate("proclamation:author:syndicate", revision);
    let chosen = app.observe();
    assert_eq!(
        chosen["table"]["proclamation_composer"]["author"],
        "Syndicate"
    );
    assert_eq!(chosen["observation"], pending);
    app.key("Escape");
    assert_eq!(app.observe()["observation"], pending);
    app.choose("pass");
    assert_ne!(app.revision(), revision);
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; checks human publication and the real faction client"]
fn native_human_publication_preserves_the_pending_havana_request() {
    use super::{construct::choose_row, construct_transfers::underfunded_handoff, faction_seats};
    use faction_seats::{DIRECTORIO, SYNDICATE};

    let mut app = super::available_forces::unlaunched("havana-compose-", "issue-230");
    underfunded_handoff(&mut app);
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    choose_row(&mut app, "syndicate:limited-operation");
    choose_row(&mut app, "transfer");
    app.choose("ask:1:2");
    let pending = app.observe()["observation"].clone();
    assert_eq!(pending["owner"], "M26");
    assert_eq!(pending["resources"], json!([11, 4, 5, 3]));
    let revision = app.revision();
    let before = app.save_bytes();

    app.key("KeyP");
    app.activate("proclamation:author:syndicate", revision);
    const WORDS: &str = "26July, the doors of the Havana Casino are chained. Two of your Resources will put five in our hands and let the house reopen. In return, when you ask for a contribution on a later card, we will hear you first. Our word is public. Government can read it, and so can Directorio. Give 1 if two is too dear, or give nothing. Either way, when the lights come on, Havana will remember who brought the key.";
    assert_eq!(WORDS.chars().count(), 400);
    type_text(&app, revision, WORDS);
    let full = app.observe()["table"]["proclamation_composer"].clone();
    assert_eq!(full["draft"], WORDS);
    assert_eq!(full["count"], 400);
    assert_eq!(
        part(&full, "plate")["bounds"],
        json!([760., 378., 3080., 1778.])
    );
    assert_eq!(
        part(&full, "field")["bounds"],
        json!([1092., 926., 2943., 1464.])
    );
    let height = part(&full, "draft")["text_size"][1].as_f64().unwrap();
    assert!(
        (400. ..=505.).contains(&height),
        "wrapped draft height {height}"
    );
    app.capture("400-character-syndicate-broadsheet");

    // One additional keystroke is visible, then the normal Publish button refuses.
    type_text(&app, revision, "!");
    app.activate("proclamation:publish", revision);
    let refused = app.observe()["table"]["proclamation_composer"].clone();
    assert_eq!(refused["draft"], format!("{WORDS}!"));
    assert_eq!(refused["count"], 401);
    assert!(refused["refusal"].as_str().unwrap().contains("400"));
    assert!(!refused["spent"].as_bool().unwrap());
    assert_eq!(app.save_bytes(), before);
    app.capture("401-character-refusal-retains-draft");

    app.key("Backspace");
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    app.activate("proclamation:publish", revision);
    let refused = app.observe()["table"]["proclamation_composer"].clone();
    assert_eq!(refused["draft"], WORDS);
    assert!(
        refused["refusal"]
            .as_str()
            .unwrap()
            .contains("another seat")
    );
    assert_eq!(app.save_bytes(), before);
    app.capture("held-faction-refusal-retains-draft");

    app.key(SYNDICATE);
    let (code, ended) = app.ask(&["observe", &syndicate]);
    assert_eq!((code, &ended["status"]), (0, &json!("closed")));
    // The original words were edited and corrected by the native field, then published
    // by keyboard from its explicit control.
    app.key("Tab");
    app.key("Tab");
    app.request(
        json!({
            "op":"key", "key":"Enter", "focused":"proclamation:publish", "revision":revision
        }),
        true,
    );
    assert!(app.observe()["table"]["proclamation_composer"].is_null());
    assert_eq!(app.observe()["observation"], pending);
    let (code, seen) = app.ask(&["observe", &directorio]);
    assert_eq!(code, 0, "{seen}");
    assert_eq!(
        seen["view"]["proclamations"]["published"],
        json!([{"author":"Syndicate", "card":18, "text":WORDS}])
    );
    std::fs::write(
        app.root.join("directorio-refreshed.json"),
        serde_json::to_vec_pretty(&seen).unwrap(),
    )
    .unwrap();

    // A second takeover cannot restore the author allowance. Attempting another native
    // publication leaves the new draft and the first record intact.
    let _replacement = app.seat_agent(SYNDICATE, "syndicate");
    app.key(SYNDICATE);
    app.activate("proclamation:write", revision);
    app.activate("proclamation:author:syndicate", revision);
    assert_eq!(
        app.observe()["table"]["proclamation_composer"]["spent"],
        true
    );
    type_text(&app, revision, "Keep  this record.\n");
    let saved = app.save_bytes();
    app.activate("proclamation:publish", revision);
    let duplicate = app.observe()["table"]["proclamation_composer"].clone();
    assert_eq!(duplicate["draft"], "Keep  this record.\n");
    assert!(
        duplicate["refusal"]
            .as_str()
            .unwrap()
            .contains("already published")
    );
    assert_eq!(app.save_bytes(), saved);
    app.capture("spent-faction-retains-new-draft");

    app.activate("proclamation:compose:return", revision);
    assert_eq!(app.observe()["observation"], pending);
    app.choose("authorize:2");
    let funded = app.observe()["observation"].clone();
    assert!(funded["transfer"].is_null());
    assert_eq!(funded["resources"], json!([11, 2, 5, 5]));
    let havana = funded["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|space| space["id"] == "havana")
        .unwrap();
    assert!(havana["pieces"].as_array().unwrap().iter().any(|piece| {
        piece["faction"] == "Syndicate" && piece["kind"] == "Casino" && piece["state"] == "Closed"
    }));
    app.quit();
}

fn part<'a>(view: &'a Value, name: &str) -> &'a Value {
    view["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["part"] == name)
        .unwrap()
}

fn type_text(app: &Native, revision: u64, text: &str) {
    let requests = text.chars().map(|character| {
        let key = match character {
            ' ' => "Space".into(),
            '\n' => "Enter".into(),
            printable => printable.to_string(),
        };
        json!({"op":"key", "key":key, "focused":"proclamation:field", "revision":revision})
            .to_string()
    });
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .args(["seat", "--brief", app.socket.to_str().unwrap()])
        .args(requests)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "typing failed: {}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}
