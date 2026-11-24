//! #172: the marker supply in the top band, against the main view's layout record, and
//! its number before and after each Terror (R 3.3.4). A pinned Terror preview leaves it
//! as it is. The captures of the top band are for the legibility check at the owner's
//! display.
use super::*;
use std::collections::BTreeMap;

/// The graphic as the seat reports it: (what it shows, its drawn parts by name).
fn supply(view: &Value) -> (Value, BTreeMap<String, Value>) {
    let reported = &view["table"]["marker_supply"];
    let parts = reported["parts"]
        .as_array()
        .expect("the seat reports the marker supply's parts")
        .iter()
        .map(|part| (part["part"].as_str().unwrap().to_owned(), part.clone()))
        .collect();
    (reported["shown"].clone(), parts)
}

/// The number drawn, and what the observation reports beside it.
fn number(view: &Value) -> (String, u64) {
    let (_, parts) = supply(view);
    (
        parts["marker-supply-count"]["text"]
            .as_str()
            .unwrap()
            .to_owned(),
        view["observation"]["marker_supply"]["available"]
            .as_u64()
            .unwrap(),
    )
}

/// Saves the top band of a capture beside it, for inspection at the owner's display.
fn top_band(app: &mut Native, name: &str) {
    app.capture(name);
    let capture = image::open(app.root.join(format!("{name}.png"))).unwrap();
    capture
        .crop_imm(0, 0, 2400, 220)
        .save(app.root.join(format!("{name}-top-band.png")))
        .unwrap();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_marker_supply_matches_layout_and_follows_each_terror() {
    use super::available_forces::start;
    // 3840x2160 is the owner's display and the record's frame: one unit is one pixel.
    let mut app = start("marker-supply-", "issue-172", "3840x2160");
    let layout: Value =
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap();
    assert_eq!(layout["frame"], json!([3840, 2160]));
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let rest = app.observe();
    // The prepared scenario has no marker on the map (the accepted tactical handoff).
    assert_eq!(
        rest["observation"]["marker_supply"],
        json!({"available": 20, "terror_on_map": 0, "sabotage_on_map": 0})
    );
    let (shown, parts) = supply(&rest);
    assert_eq!(
        shown,
        json!({"available": 20, "of": "of 20", "empty": false})
    );
    assert_eq!(
        parts.keys().collect::<Vec<_>>(),
        ["marker-supply", "marker-supply-count", "marker-supply-of"]
    );
    assert_eq!(parts["marker-supply-count"]["text"], "20");
    assert_eq!(parts["marker-supply-of"]["text"], "of 20");
    assert_eq!(parts["marker-supply"]["dimmed"], false);
    // The image on all four edges, the text on left, top and right.
    for (group, edges) in [("parts", 4), ("text", 3)] {
        for (name, rect) in layout[group].as_object().unwrap() {
            if !name.starts_with("marker-supply") {
                continue;
            }
            let [x, y, w, h] = rect_of(rect);
            let expected = [x, y, x + w, y + h];
            let native = rect_of(&parts[name]["bounds"]);
            for (n, e) in native.iter().zip(expected).take(edges) {
                assert!(
                    (n - e).abs() <= tolerance,
                    "{name}: native {native:?} differs from layout {expected:?} by more than {tolerance} px"
                );
            }
        }
    }
    // The graphic lies inside its area, between the US Alliance track and the
    // standings, and is centred between the outside-influence area and the standings.
    let areas = &rest["table"]["main_view"]["areas"];
    let area = rect_of(&areas["marker-supply"]);
    let union = parts.values().map(|p| rect_of(&p["bounds"])).fold(
        [f64::MAX, f64::MAX, f64::MIN, f64::MIN],
        |a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        },
    );
    assert!(
        union[0] >= area[0] && union[1] >= area[1] && union[2] <= area[2] && union[3] <= area[3],
        "the graphic {union:?} leaves its area {area:?}"
    );
    let between = (rect_of(&areas["outside-influence"])[2] + rect_of(&areas["standings"])[0]) / 2.;
    assert!(
        ((union[0] + union[2]) / 2. - between).abs() <= tolerance,
        "the graphic {union:?} is off the middle {between}"
    );
    top_band(&mut app, "at-rest");

    // 26July chooses Terror and asks the Directorio for 2 Resources, so it can pay
    // for both targets.
    app.choose("terror");
    app.choose("transfer");
    app.choose("ask:2:2");
    app.choose("authorize:2");
    let funded = app.observe();
    assert_eq!(number(&funded), ("20".into(), 20));
    let mut expected = 20;
    for space in ["santiago", "oriente"] {
        let view = app.observe();
        let revision = view["observation"]["revision"].as_u64().unwrap();
        // The space's popover pins the Terror's preview: it changes nothing.
        app.activate(&format!("space:{space}"), revision);
        let pinned = app.observe();
        assert_eq!(pinned["table"]["popover"]["pinned"], true, "{pinned}");
        assert_eq!(pinned["observation"], view["observation"]);
        assert_eq!(number(&pinned), (expected.to_string(), expected));
        top_band(&mut app, &format!("{space}-preview"));
        app.activate(&format!("popover:row:terror:{space}"), revision);
        app.activate("popover:commit", revision);
        // R 3.3.4: the Terror marker comes from the shared supply.
        expected -= 1;
        let after = app.observe();
        assert_eq!(
            after["observation"]["marker_supply"],
            json!({"available": expected, "terror_on_map": 20 - expected, "sabotage_on_map": 0})
        );
        assert_eq!(number(&after), (expected.to_string(), expected));
        let (_, parts) = supply(&after);
        assert_eq!(parts["marker-supply"]["dimmed"], false);
        top_band(&mut app, &format!("after-{space}"));
    }
    app.quit();
}
