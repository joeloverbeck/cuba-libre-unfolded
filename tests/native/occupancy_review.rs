use super::*;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; retain the review-only EC's Home and close views"]
fn native_economic_center_review_opens_without_a_save() {
    let mut app = available_forces::unlaunched("occupancy-review-", "issue-267");
    let case = "dense:ec-las-villas-camaguey";
    let launch = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--occupancy-review",
            case,
            "--agent-seat",
            "--seat-size",
            "3840x2160",
            "--socket",
            app.socket.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        launch.status.success(),
        "occupancy review did not open: {}",
        String::from_utf8_lossy(&launch.stderr)
    );
    let home = app.request(json!({"op":"settle"}), true);
    let table = &home["table"];
    assert_eq!(table["occupancy_review"], case);
    assert_eq!(table["viewport"], json!([3840.0, 2160.0]));
    assert!(
        table["main_view"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|part| part["part"] == "decision-frame")
    );
    let dense = home["observation"]["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|space| space["id"] == "ec-las-villas-camaguey")
        .unwrap();
    assert_eq!(
        dense["pieces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|piece| {
                (
                    piece["kind"].as_str().unwrap(),
                    piece["count"].as_u64().unwrap(),
                )
            })
            .collect::<Vec<_>>(),
        [
            ("Troops", 15),
            ("Police", 15),
            ("Guerrilla", 14),
            ("Guerrilla", 1),
            ("Guerrilla", 1),
            ("Guerrilla", 14),
            ("Guerrilla", 5),
            ("Guerrilla", 1)
        ]
    );
    assert_eq!(dense["terror"], 0);
    assert_eq!(home["observation"]["marker_supply"]["sabotage_on_map"], 1);
    assert_eq!(home["observation"]["cash_available"], 0);
    assert_eq!(home["observation"]["choices"], json!([]));
    assert!(table["counts"].as_array().unwrap().iter().any(|count| {
        count["name"] == "count:ec-las-villas-camaguey Review Sabotage" && count["text"] == "S"
    }));
    assert!(
        table["controls"]
            .as_array()
            .unwrap()
            .iter()
            .all(|control| { control["choice"] == false })
    );
    let refused = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--occupancy-review",
            case,
            "--save",
            app.save.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("Occupancy review has no save"));

    app.capture("ec-factory-home");
    app.request(
        json!({"op":"scroll","revision":0,"component":"space:ec-las-villas-camaguey","y":18}),
        true,
    );
    let close = app.request(json!({"op":"settle"}), true);
    assert!(
        close["table"]["camera"]["distance"].as_f64().unwrap()
            < table["camera"]["distance"].as_f64().unwrap()
    );
    app.capture("ec-factory-close");
    app.request(json!({"op":"key","revision":0,"key":"Home"}), true);
    assert_eq!(
        app.request(json!({"op":"settle"}), true)["table"]["camera"],
        table["camera"]
    );
    assert!(!app.save.exists());
    app.quit();
}
