use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;
use unfolded_record_replacement::previous_path;

const STACKING: &str = "2 non-Casino Base slots full (R 1.4.2).";
const CUBE_MIX: &str = "manifesto:place:havana:troops-police";
const CUBE_LABEL: &str = "1 Troop + 1 Police cube";

fn inspect_mix(app: &mut Native, action: &str) -> Value {
    let revision = app.revision();
    app.activate("space:havana", revision);
    app.activate(&format!("popover:row:{action}"), revision);
    app.observe()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect Havana's stacking reason and legal mixes at 4K"]
fn native_manifesto_havana_restarts_with_both_base_slots_filled() {
    let mut app = available_forces::unlaunched("manifesto-havana-", "issue-242");
    card_advance::finish_event(&app.save, "shaded");
    app.launch_size("3840x2160");
    app.choose("advance-card");
    for action in ["government:pass", "syndicate:pass"] {
        let revision = app.revision();
        app.activate(&format!("option:{action}"), revision);
        app.choose(action);
    }
    let ready = app.observe();
    assert_eq!(
        ready["observation"]["passed"],
        json!([true, false, false, true])
    );
    app.choose("advance-card");
    for action in ["m26:event", "manifesto:decline"] {
        let revision = app.revision();
        app.activate(&format!("option:{action}"), revision);
        app.choose(action);
    }

    let bases = "manifesto:place:havana:base-base";
    let directorio = inspect_mix(&mut app, bases);
    assert_eq!(directorio["observation"]["owner"], "Directorio");
    let subtitle = directorio["table"]["popover"]["card"]["subtitle"]
        .as_str()
        .unwrap();
    assert!(
        subtitle.contains("Directorio has 2 Underground Guerrillas"),
        "{subtitle}"
    );
    let preview = popover_row(&directorio, "2 Bases")["lines"].to_string();
    assert!(preview.contains("2 Bases · Available 4 → 2"), "{preview}");
    assert!(
        preview.contains("Directorio 1 → 3 (margin -8 → -6)"),
        "{preview}"
    );
    app.capture("havana-directorio-two-bases");
    let revision = app.revision();
    app.activate("popover:commit", revision);
    let syndicate = inspect_mix(&mut app, "manifesto:place:havana:guerrilla-guerrilla");
    assert_eq!(syndicate["observation"]["owner"], "Syndicate");
    let subtitle = syndicate["table"]["popover"]["card"]["subtitle"]
        .as_str()
        .unwrap();
    assert!(
        subtitle.contains("Syndicate has 1 open Casino"),
        "{subtitle}"
    );
    let preview = popover_row(&syndicate, "2 Guerrillas")["lines"].to_string();
    assert!(
        preview.contains("2 Underground Guerrillas · Available 6 → 4"),
        "{preview}"
    );
    app.capture("havana-syndicate-intervening-placement");
    app.key("Escape");

    let government = app.seat_agent(faction_seats::GOVERNMENT, "government");
    let mut waiting = app.player("government-wait", &["wait", &government]);
    app.until_waiting("Government");
    app.pick("havana", "manifesto:place:havana:guerrilla-guerrilla");
    let (code, awake) = app.answered(&mut waiting);
    assert_eq!((code, &awake["status"]), (0, &json!("decision")), "{awake}");
    let client = &awake["view"];
    assert_eq!(client["decision"]["faction"], "Government");
    assert_eq!(client["decision"]["yours"], true);
    let pending = app.observe()["observation"].clone();
    assert_eq!(pending["owner"], "Government");
    assert_eq!(client["state"]["manifesto"], pending["manifesto"]);
    assert_eq!(client["state"]["available"], pending["available"]);
    let revision = pending["revision"].as_u64().unwrap();
    let choices = client["choices"].as_array().unwrap();
    let havana = choices
        .iter()
        .filter(|c| c["site"]["space"] == "havana")
        .collect::<Vec<_>>();
    assert_eq!(
        havana
            .iter()
            .map(|c| c["label"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["2 Troops", CUBE_LABEL, "2 Police cubes"]
    );
    let mix = havana[1];
    assert_eq!(mix["id"], format!("{revision}:{CUBE_MIX}"));
    let explanation = mix["explanation"].as_str().unwrap();
    assert!(
        explanation.contains("Government has 6 Troops, 4 Police cubes"),
        "{explanation}"
    );
    assert!(explanation.contains(STACKING), "{explanation}");
    assert!(
        choices
            .iter()
            .any(|c| c["site"]["space"] == "santiago" && c["label"] == "1 Troop + 1 Base")
    );
    let saved = app.save_bytes();
    let previous = fs::read(previous_path(&app.save)).unwrap();
    app.quit();

    app.launch_size("3840x2160");
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(app.save_bytes(), saved);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
    space_reason::hover(
        &mut app,
        "havana",
        "selectable",
        Some("Both Base slots full (R 1.4.2)."),
        "havana-government-stacking",
    );
    let map = inspect_mix(&mut app, CUBE_MIX);
    let card = &map["table"]["popover"]["card"];
    assert_eq!(
        card["subtitle"],
        "City · Government has 6 Troops, 4 Police cubes"
    );
    let rows = card["rows"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| r["choice"].as_str().unwrap())
            .collect::<Vec<_>>(),
        havana
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect::<Vec<_>>()
    );
    let row = popover_row(&map, CUBE_LABEL);
    assert_eq!(row["button"], "Place these pieces");
    let lines = row["lines"].to_string();
    for fact in [
        STACKING,
        "1 Troop · Available 3 → 2",
        "1 Police cube · Available 8 → 7",
        "Control and standings stay as shown.",
    ] {
        assert!(lines.contains(fact), "{fact}: {lines}");
    }
    let [left, top, right, bottom] = rect_of(&map["table"]["popover"]["bounds"]);
    let [map_left, map_top, map_right, map_bottom] = rect_of(&map["table"]["map_viewport"]);
    assert!(left >= map_left && top >= map_top && right <= map_right && bottom <= map_bottom);
    app.capture("havana-government-cube-mixes");
    app.key("Escape");
    let government = app.seat_agent(faction_seats::GOVERNMENT, "government");
    let (_, returned) = app.ask(&["observe", &government]);
    assert_eq!(returned["view"]["choices"], client["choices"]);
    assert_eq!(returned["view"]["state"], client["state"]);
    let held = inspect_mix(&mut app, CUBE_MIX);
    assert_eq!(popover_row(&held, CUBE_LABEL)["lines"], row["lines"]);
    app.offers_nothing();
    app.capture("havana-government-held-cube-mix");
    app.key("Escape");

    let refused = app.decide(&government, "manifesto:place:havana:troops-base", "refused");
    let old = (revision - 1).to_string();
    let (code, stale) = app.ask(&["choose", &government, &old, &format!("{old}:{CUBE_MIX}")]);
    assert_eq!((code, &stale["status"]), (1, &json!("refused")), "{stale}");
    assert_eq!(app.observe()["observation"], pending);
    assert_eq!(app.save_bytes(), saved);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);

    app.decide(&government, CUBE_MIX, "applied");
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    let final_table = app.observe();
    let finished = &final_table["observation"];
    assert_eq!(finished["revision"], revision + 1);
    assert_eq!(finished["standings"], pending["standings"]);
    assert_eq!(finished["resources"], pending["resources"]);
    assert_eq!(finished["eligible"], pending["eligible"]);
    assert_eq!(finished["owner"], "Directorio");
    assert_eq!(finished["next"]["role"], "2nd Eligible");
    assert_eq!(
        finished["next"]["options"],
        json!([
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)"
        ])
    );
    assert_eq!(
        available_forces::drawn_count(&final_table, "government-troops"),
        "2"
    );
    assert_eq!(
        available_forces::drawn_count(&final_table, "government-police"),
        "7"
    );
    assert_eq!(
        available_forces::drawn_count(&final_table, "government-base"),
        "2"
    );
    assert_eq!(
        available_forces::drawn_count(&final_table, "directorio-base"),
        "2"
    );
    assert_eq!(
        available_forces::drawn_count(&final_table, "syndicate-guerrilla"),
        "4"
    );
    app.capture("havana-government-committed-directorio-next");
    let final_bytes = app.save_bytes();
    app.quit();
    app.launch_size("3840x2160");
    assert_eq!(&app.observe()["observation"], finished);
    assert_eq!(app.save_bytes(), final_bytes);
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), saved);
    let check = Session::open(&app.save, None).unwrap();
    let view = check.observe();
    let city = view
        .spaces
        .iter()
        .find(|space| space.id == "havana")
        .unwrap();
    assert_eq!(city.control(), Some(Faction::Government));
    assert_eq!(
        city.pieces.iter().map(|p| p.count).collect::<Vec<_>>(),
        [7, 5, 2, 1, 2, 2]
    );
    let reports = check.reports();
    let report = reports.last().unwrap();
    assert_eq!(report.faction, Faction::M26);
    assert!(report.complete);
    assert_eq!(
        report
            .articles
            .iter()
            .flat_map(|article| &article.lines)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "26July declined placement.",
            "Directorio placed 2 Bases in Havana.",
            "Syndicate placed 2 Underground Guerrillas in Havana.",
            "Government placed 1 Troop + 1 Police cube in Havana."
        ]
    );
    fs::write(
        app.root.join("manifesto-havana-client.json"),
        serde_json::to_vec_pretty(&json!({"awake": awake, "resumed": returned, "illegal": refused, "stale": stale, "report": report})).unwrap(),
    ).unwrap();
    app.quit();
}
