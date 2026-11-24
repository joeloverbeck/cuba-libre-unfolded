//! #172: the marker supply in the top band, against the main view's layout record, and
//! its number before and after each Terror (R 3.3.4). A pinned Terror preview leaves it
//! as it is. The captures of the top band are for the legibility check at the owner's
//! display. #173: its tag, opened by hover and by Tab, before and after a Terror.
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
        // The Terror row is the one chosen, with its preview of the marker.
        let row = super::popover_row(&pinned, "Terror");
        assert_eq!(row["chosen"], true, "{row}");
        assert!(
            row["lines"].to_string().contains("Place a Terror marker"),
            "{row}"
        );
        top_band(&mut app, &format!("{space}-preview"));
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

/// The marker supply's tag as the seat reports it (`table.top`): its rows as
/// (label, value).
fn tag_rows(view: &Value) -> Vec<(String, String)> {
    view["table"]["top"]["rows"]
        .as_array()
        .expect("a tag is open")
        .iter()
        .map(|row| {
            (
                row[0].as_str().unwrap().to_owned(),
                row[1].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// The rows the rules give (R 3.3.4): the markers on the map by kind, and those left
/// of the 20.
fn expected_rows(terror: u8) -> Vec<(String, String)> {
    vec![
        ("Terror on the map".into(), terror.to_string()),
        ("Sabotage on the map".into(), "0".into()),
        ("Available".into(), format!("{} of 20", 20 - terror)),
    ]
}

/// The notes, each note's drawn lines joined by spaces.
fn notes(view: &Value) -> Vec<String> {
    view["table"]["top"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().replace('\n', " "))
        .collect()
}

/// Checks the open tag: the marker supply's, titled "Marker supply" with these rows
/// and the two notes; below the graphic and inside the window; covering neither the
/// outside-influence area, where the US Alliance track is, nor the standings. Saves
/// the capture and its top left beside it, at 1:1, for the legibility check.
fn assert_tag(app: &mut Native, view: &Value, terror: u8, at: &str) {
    let top = &view["table"]["top"];
    assert_eq!(top["item"], "marker-supply", "{at}: {top}");
    assert_eq!(top["title"], "Marker supply", "{at}");
    assert_eq!(tag_rows(view), expected_rows(terror), "{at}");
    assert_eq!(
        notes(view),
        [
            "Terror and Sabotage share 20 markers. Terror places no marker when all 20 are on the map (R 3.3.4).",
            "Civic Action and Agitation return Terror markers to the supply (R 6.3.2, 6.3.3); the Reset Phase returns the Terror and Sabotage markers (R 6.5).",
        ],
        "{at}"
    );
    let plate = rect_of(&top["bounds"]);
    let areas = &view["table"]["main_view"]["areas"];
    let graphic = rect_of(&areas["marker-supply"]);
    let (width, height) = {
        let w = &view["table"]["viewport"];
        (w[0].as_f64().unwrap(), w[1].as_f64().unwrap())
    };
    assert!(
        plate[0] >= 0. && plate[1] >= 0. && plate[2] <= width && plate[3] <= height,
        "{at}: the tag {plate:?} leaves the window {width}x{height}"
    );
    assert!(
        plate[1] > graphic[3] && plate[0] <= graphic[0] && graphic[2] <= plate[2],
        "{at}: the tag {plate:?} is not below the graphic {graphic:?}"
    );
    for name in ["outside-influence", "standings"] {
        let area = rect_of(&areas[name]);
        let apart = plate[2] <= area[0]
            || area[2] <= plate[0]
            || plate[3] <= area[1]
            || area[3] <= plate[1];
        assert!(apart, "{at}: the tag {plate:?} covers {name} {area:?}");
    }
    app.capture(at);
    let capture = image::open(app.root.join(format!("{at}.png"))).unwrap();
    let scale = f64::from(capture.width()) / width;
    let right = ((plate[2] + 24.) * scale).min(f64::from(capture.width()));
    let bottom = ((plate[3] + 24.) * scale).min(f64::from(capture.height()));
    capture
        .crop_imm(0, 0, right as u32, bottom as u32)
        .save(app.root.join(format!("{at}-tag.png")))
        .unwrap();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_marker_supply_tag_opens_on_hover_and_focus_and_follows_a_terror() {
    use super::available_forces::{focus, start};
    let mut app = start("marker-supply-tag-", "issue-173", "3840x2160");
    let rest = app.observe();
    assert!(rest["table"]["top"].is_null());
    // Hover opens the tag after the shared dwell; reading it changes nothing.
    let shown = app.point("marker-supply");
    assert_eq!(shown["table"]["hover"], "marker-supply");
    assert_tag(&mut app, &shown, 0, "hover-at-rest");
    assert_eq!(shown["observation"], rest["observation"]);
    assert_eq!(number(&shown), ("20".into(), 20));
    // It closes when the pointer leaves, as the top display's tags do.
    let left = app.point("area:propaganda");
    assert!(left["table"]["top"].is_null(), "{}", left["table"]["top"]);
    // Tab reaches it after the panel's controls and before the Available Forces rows;
    // focus opens the tag with the same dwell, and the next stop closes it.
    let focused = focus(&mut app, "marker-supply");
    assert_tag(&mut app, &focused, 0, "focus-at-rest");
    assert_eq!(focused["observation"], rest["observation"]);
    let next = focus(&mut app, "available:government");
    assert_eq!(next["table"]["top"]["item"], "available:government");

    // 26July's Terror in Santiago places a Terror marker from the supply (R 3.3.4):
    // the tag then reads 1 on the map and 19 of 20, by hover and by focus.
    app.choose("terror");
    app.choose("transfer");
    app.choose("ask:2:2");
    app.choose("authorize:2");
    let view = app.observe();
    let revision = view["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", revision);
    app.activate("popover:commit", revision);
    let after = app.observe();
    assert_eq!(
        after["observation"]["marker_supply"],
        json!({"available": 19, "terror_on_map": 1, "sabotage_on_map": 0})
    );
    let shown = app.point("marker-supply");
    assert_tag(&mut app, &shown, 1, "hover-after-santiago");
    assert_eq!(shown["observation"], after["observation"]);
    app.point("area:propaganda");
    let focused = focus(&mut app, "marker-supply");
    assert_tag(&mut app, &focused, 1, "focus-after-santiago");
    app.quit();

    // At 3:2 and 21:9 the tag still opens below the graphic, inside the window and
    // clear of the track and the standings. The match resumes from its save.
    for size in ["1920x1280", "2520x1080"] {
        app.launch_size(size);
        let shown = app.point("marker-supply");
        assert_tag(&mut app, &shown, 1, &format!("hover-{size}"));
        app.quit();
    }
}
