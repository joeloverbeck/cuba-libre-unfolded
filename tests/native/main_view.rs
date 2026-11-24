//! #142: the map fills the window and every interface area lies over it, against the
//! main view's FigCraft layout record, through the development seat's own pointer,
//! wheel and keys.
use super::*;
use std::collections::BTreeMap;

/// The map's opening before #142, at 3840×2160: [left, top, right, bottom]. Outside it
/// the window was desk, and the camera took no input.
const OLD_OPENING: [f64; 4] = [67.5, 297., 2861.25, 1451.25];

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
}

/// A record rectangle [x, y, width, height] as [left, top, right, bottom].
fn edges(rect: &Value) -> [f64; 4] {
    let [x, y, w, h] = rect_of(rect);
    [x, y, x + w, y + h]
}

fn launch(prefix: &str, size: &str) -> Native {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-142".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    // The socket's directory outlives the test, like the evidence beside it.
    let socket = tempfile::tempdir().unwrap().keep().join("seat.sock");
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into(),
        socket,
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Main view evidence: {}", app.root.display());
    app.launch_size(size);
    app
}

fn rects(value: &Value) -> BTreeMap<String, [f64; 4]> {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, bounds)| (name.clone(), rect_of(bounds)))
        .collect()
}

fn areas(view: &Value) -> BTreeMap<String, [f64; 4]> {
    rects(&view["table"]["main_view"]["areas"])
}

fn spaces(view: &Value) -> BTreeMap<String, [f64; 4]> {
    rects(&view["table"]["main_view"]["spaces"])
}

fn disjoint(a: [f64; 4], b: [f64; 4]) -> bool {
    a[2] <= b[0] || a[0] >= b[2] || a[3] <= b[1] || a[1] >= b[3]
}

fn holds(rect: [f64; 4], point: [f64; 2], inset: f64) -> bool {
    point[0] >= rect[0] + inset
        && point[0] <= rect[2] - inset
        && point[1] >= rect[1] + inset
        && point[1] <= rect[3] - inset
}

/// Where the seat presses for each space: its label's point on the map.
fn space_points(view: &Value) -> BTreeMap<String, [f64; 2]> {
    view["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|control| {
            let space = control["name"].as_str()?.strip_prefix("space:")?;
            let at = control["position"].as_array()?;
            Some((space.to_owned(), [at[0].as_f64()?, at[1].as_f64()?]))
        })
        .collect()
}

/// A space whose point lies under an overlaid area, with that area.
fn space_under_an_area(view: &Value) -> Option<(String, String)> {
    let areas = areas(view);
    space_points(view).into_iter().find_map(|(space, at)| {
        areas
            .iter()
            .find(|(_, area)| holds(**area, at, 12.))
            .map(|(area, _)| (space, area.clone()))
    })
}

/// The seat does not rest on `space`, because something of the interface covers it
/// there: the reply names what, and it is not the space. No tag opens.
fn assert_covered(app: &mut Native, space: &str, revision: u64) {
    let missed = app.request(
        json!({"op":"point","revision":revision,"component":format!("space:{space}")}),
        false,
    );
    assert_eq!(missed["status"], "missed", "{missed}");
    let hits = missed["hit"].as_array().unwrap();
    assert!(
        hits.iter().any(|hit| hit["control"]
            .as_str()
            .is_some_and(|name| !name.starts_with("space:"))),
        "nothing of the interface covers {space}: {missed}"
    );
    assert!(
        hits.iter()
            .all(|hit| hit["control"] != format!("space:{space}") && hit["space"] != space),
        "{space} is picked through the interface: {missed}"
    );
    let settled = app.request(json!({"op":"settle"}), true);
    assert!(
        settled["table"]["status"].is_null(),
        "{space}'s tag opened through the interface: {}",
        settled["table"]["status"]
    );
}

/// The space nearest the middle of the clear part of the window: one whose tag has
/// room to open at this camera position.
fn space_in_the_clear(view: &Value) -> String {
    let [l, t, r, b] = rect_of(&view["table"]["main_view"]["clear"]);
    let middle = [(l + r) / 2., (t + b) / 2.];
    space_points(view)
        .into_iter()
        .min_by(|a, b| {
            let far = |at: &[f64; 2]| (at[0] - middle[0]).hypot(at[1] - middle[1]);
            far(&a.1).total_cmp(&far(&b.1))
        })
        .unwrap()
        .0
}

/// Every area is inside the window, where the layout's anchors put it; all thirteen
/// spaces are inside the layout's island box and clear of every area.
fn assert_areas_and_island(view: &Value, layout: &Value, width: f64, height: f64) {
    let scale = (width / 1920.).min(height / 1280.);
    // One frame unit in this window: the frame is as tall as the 1280 reference.
    let unit = scale * 1280. / layout["frame"][1].as_f64().unwrap();
    let frame = [
        layout["frame"][0].as_f64().unwrap(),
        layout["frame"][1].as_f64().unwrap(),
    ];
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let drawn = areas(view);
    let recorded = layout["areas"].as_object().unwrap();
    let mut names: Vec<&String> = recorded.keys().collect();
    names.sort_unstable();
    assert_eq!(drawn.keys().collect::<Vec<_>>(), names);
    // A frame edge measured from the window's start, centre or end.
    let start = |v: f64| v * unit;
    let end = |v: f64, of: f64, window: f64| window - (of - v) * unit;
    let centre = |v: f64| width / 2. + (v - frame[0] / 2.) * unit;
    // An area's left and right edges in this window, from the edges its anchor names.
    let across = |placed: &Value| -> [f64; 2] {
        let [l, _, r, _] = edges(&placed["bounds"]);
        match placed["anchor"].as_str().unwrap() {
            "top-left" | "bottom-left" => [start(l), start(r)],
            "top" => [centre(l), centre(r)],
            "top-right" | "bottom-right" => [end(l, frame[0], width), end(r, frame[0], width)],
            other => panic!("anchor {other}"),
        }
    };
    for (name, placed) in recorded {
        let [l, t, r, b] = edges(&placed["bounds"]);
        let anchor = placed["anchor"].as_str().unwrap();
        let (left, right) = if anchor == "top-between" {
            // #172: centred between the right edge of one area and the left edge of
            // another, at the size the window's scale gives it.
            let [one, other] = [0, 1].map(|i| &recorded[placed["between"][i].as_str().unwrap()]);
            let middle = (across(one)[1] + across(other)[0]) / 2.;
            let half = (r - l) * unit / 2.;
            (middle - half, middle + half)
        } else {
            let [left, right] = across(placed);
            (left, right)
        };
        let (top, bottom) = if anchor.starts_with("top") {
            (start(t), start(b))
        } else {
            (end(t, frame[1], height), end(b, frame[1], height))
        };
        let native = drawn[name];
        for (n, e) in native.iter().zip([left, top, right, bottom]) {
            assert!(
                (n - e).abs() <= tolerance,
                "area {name} at {width}x{height}: native {native:?}, layout {:?}",
                [left, top, right, bottom]
            );
        }
        assert!(
            native[0] >= 0. && native[1] >= 0. && native[2] <= width && native[3] <= height,
            "area {name} leaves the {width}x{height} window: {native:?}"
        );
    }
    // #159: no area overlaps another in any window, and the Available Forces ledger's
    // plate fills its own area, so it keeps its size and its distance from the
    // top-right corner as the area does.
    for (name, rect) in &drawn {
        for (other, against) in &drawn {
            assert!(
                name == other || disjoint(*rect, *against),
                "area {name} {rect:?} overlaps {other} {against:?} at {width}x{height}"
            );
        }
    }
    let plate = view["table"]["available_forces"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["part"] == "plate")
        .map(|part| rect_of(&part["bounds"]))
        .expect("the Available Forces ledger draws its plate");
    for (n, e) in plate.iter().zip(drawn["available-forces"]) {
        assert!(
            (n - e).abs() <= tolerance,
            "the ledger's plate {plate:?} leaves its area at {width}x{height}"
        );
    }
    let [l, t, r, b] = edges(&layout["island"]["bounds"]);
    let island = [
        start(l),
        start(t),
        end(r, frame[0], width),
        end(b, frame[1], height),
    ];
    let reported = rect_of(&view["table"]["main_view"]["island"]);
    for (n, e) in reported.iter().zip(island) {
        assert!((n - e).abs() <= tolerance, "island box {reported:?}");
    }
    let spaces = spaces(view);
    assert_eq!(spaces.len(), 13, "{spaces:?}");
    let mut all = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for (space, bounds) in &spaces {
        assert!(
            bounds[0] >= island[0] - tolerance
                && bounds[1] >= island[1] - tolerance
                && bounds[2] <= island[2] + tolerance
                && bounds[3] <= island[3] + tolerance,
            "{space} {bounds:?} leaves the island box {island:?} at {width}x{height}"
        );
        for (area, rect) in &drawn {
            assert!(
                disjoint(*bounds, *rect),
                "{space} {bounds:?} lies under {area} {rect:?} at {width}x{height}"
            );
        }
        all = [
            all[0].min(bounds[0]),
            all[1].min(bounds[1]),
            all[2].max(bounds[2]),
            all[3].max(bounds[3]),
        ];
    }
    // Home shows the island as large as its box allows, centred in it.
    let (wide, tall) = (all[2] - all[0], all[3] - all[1]);
    assert!(
        (wide - (island[2] - island[0])).abs() <= tolerance
            || (tall - (island[3] - island[1])).abs() <= tolerance,
        "the island {all:?} could be larger in its box {island:?}"
    );
    assert!(
        ((all[0] + all[2]) - (island[0] + island[2])).abs() <= 2. * tolerance
            && ((all[1] + all[3]) - (island[1] + island[3])).abs() <= 2. * tolerance,
        "the island {all:?} is off the centre of its box {island:?}"
    );
}

/// Resting on each of the thirteen spaces opens that space's tag, clear of every area
/// and inside the window; a press on it resolves and changes nothing in the match.
fn assert_every_space_picks(app: &mut Native, before: &Value) {
    let revision = before["observation"]["revision"].as_u64().unwrap();
    let areas = areas(before);
    for space in before["observation"]["spaces"].as_array().unwrap() {
        let id = space["id"].as_str().unwrap();
        app.activate(&format!("space:{id}"), revision);
        let shown = app.point(&format!("space:{id}"));
        let status = &shown["table"]["status"];
        assert_eq!(status["space"], id, "{}", shown["table"]["main_view"]);
        assert_status_geometry(status);
        let tag = rect_of(&status["bounds"]);
        for (area, rect) in &areas {
            assert!(disjoint(tag, *rect), "{id}'s tag {tag:?} covers {area}");
        }
        assert_eq!(shown["observation"], before["observation"]);
    }
}

fn camera(view: &Value) -> Value {
    view["table"]["camera"].clone()
}

/// The median colour of the 9×9 pixels around a point of a capture: the sea's own
/// colour even where one of its pale strokes crosses.
fn median(image: &image::RgbImage, [x, y]: [u32; 2]) -> [u8; 3] {
    std::array::from_fn(|channel| {
        let mut values: Vec<u8> = (0..9)
            .flat_map(|dx| (0..9).map(move |dy| (dx, dy)))
            .map(|(dx, dy)| image.get_pixel(x + dx - 4, y + dy - 4).0[channel])
            .collect();
        values.sort_unstable();
        values[40]
    })
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_main_view_fills_the_window_and_matches_its_layout() {
    let mut app = launch("main-view-", "3840x2160");
    let layout = record();
    // The frame is the owner's display: one frame unit is one pixel of this window.
    assert_eq!(layout["frame"], json!([3840, 2160]));
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let rest = app.observe();
    assert_eq!(rest["table"]["viewport"], json!([3840., 2160.]));
    assert_eq!(
        rect_of(&rest["table"]["map_viewport"]),
        [0., 0., 3840., 2160.],
        "the map camera draws the whole window"
    );
    assert_areas_and_island(&rest, &layout, 3840., 2160.);

    // #174: the session panel is gone. Its status line, the saved revision and the
    // marker supply's text with it, is drawn nowhere: the marker supply's graphic is
    // the one place the count is shown. At rest no notice slip is drawn.
    let parts = rest["table"]["main_view"]["parts"].as_array().unwrap();
    assert!(
        parts
            .iter()
            .all(|part| !part["part"].as_str().unwrap().starts_with("session")),
        "{parts:?}"
    );
    let texts: Vec<&str> = parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect();
    for retired in ["Saved revision", "Terror/Sabotage"] {
        assert!(
            texts.iter().all(|text| !text.contains(retired)),
            "{retired} is still drawn: {texts:?}"
        );
    }
    assert!(
        rest["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .all(|control| control["name"] != "area:session")
    );
    assert_eq!(rest["table"]["slip"]["shown"], false);

    // The accepted areas against the layout record: images on all four edges, text
    // on left, top and right, since its line height is the renderer's.
    let drawn: BTreeMap<String, [f64; 4]> = ["main_view", "eligibility"]
        .iter()
        .flat_map(|field| rest["table"][field]["parts"].as_array().unwrap())
        .chain(rest["table"]["top_parts"].as_array().unwrap())
        .map(|part| {
            (
                part["part"].as_str().unwrap().to_owned(),
                rect_of(&part["bounds"]),
            )
        })
        .collect();
    for (group, compared) in [("parts", 4), ("text", 3)] {
        for (name, rect) in layout[group].as_object().unwrap() {
            let expected = edges(rect);
            let native = drawn
                .get(name)
                .unwrap_or_else(|| panic!("the main view draws no {name}"));
            for (n, e) in native.iter().zip(expected).take(compared) {
                assert!(
                    (n - e).abs() <= tolerance,
                    "{name}: native {native:?} differs from layout {expected:?} by more than {tolerance} px"
                );
            }
        }
    }
    // The card panel is gone (#143): the piles lie bare on the sea.
    assert!(
        !drawn.contains_key("card-frame"),
        "the card panel's frame is drawn"
    );

    // The capture: sea where the desk, the "CUBA" title, the terrain key and the
    // adjacency line were. The sea is the layout's `map` colour with pale strokes.
    app.capture("home");
    let home = image::open(app.root.join("home.png")).unwrap().to_rgb8();
    assert_eq!(home.dimensions(), (3840, 2160));
    let sea = [216u8, 231, 227];
    for at in [
        // The old desk: the corners, the strips above, beside and below the map.
        [20, 20],
        [3820, 20],
        [20, 2140],
        [3820, 2140],
        [1000, 240],
        [30, 1200],
        [2895, 1000],
        [1500, 1700],
        [2000, 2100],
        [2895, 1900],
        // #174: the session panel's place, now sea, with the slip hidden.
        [3300, 1750],
        [3100, 2050],
        // The title, the key's five entries and the adjacency line.
        [145, 275],
        [150, 1483],
        [435, 1483],
        [745, 1483],
        [1085, 1483],
        [1550, 1483],
        [455, 1536],
    ] {
        let colour = median(&home, at);
        assert!(
            colour.iter().zip(sea).all(|(c, s)| c.abs_diff(s) <= 2),
            "{at:?} is {colour:?}, not the sea {sea:?}"
        );
    }
    // No lettering where the title, the key and the line were: nothing as dark as ink.
    for [left, top, right, bottom] in [[100, 255, 200, 295], [100, 1465, 1730, 1560]] {
        for x in left..right {
            for y in top..bottom {
                let pixel = home.get_pixel(x, y).0;
                assert!(
                    pixel.iter().any(|c| *c >= 150),
                    "dark pixel {pixel:?} at {x},{y}"
                );
            }
        }
    }

    // At Home every space picks, and its tag keeps off every area.
    assert_every_space_picks(&mut app, &rest);
    app.capture("home-tag");

    // The camera works where the desk was. Pan north until a space lies below the
    // old opening, on open sea; the wheel and a right-drag there move the camera.
    let home_camera = camera(&app.point("space:las-villas"));
    let mut moved = app.observe();
    let mut below = None;
    for _ in 0..40 {
        let areas = areas(&moved);
        below = space_points(&moved).into_iter().find(|(_, at)| {
            at[1] > OLD_OPENING[3] + 30.
                && holds([0., 0., 3840., 2160.], *at, 40.)
                && areas.values().all(|area| !holds(*area, *at, -40.))
        });
        if below.is_some() {
            break;
        }
        app.request(json!({"op":"key","key":"KeyW","revision":0}), true);
        moved = app.request(json!({"op":"settle"}), true);
    }
    let (below, at) = below.expect("panning brings a space below the old opening");
    assert_ne!(camera(&moved)["target"], home_camera["target"], "KeyW pans");
    assert_eq!(camera(&moved)["distance"], home_camera["distance"]);
    eprintln!("{below} at {at:?} lies below the old opening");
    let rested = app.point(&format!("space:{below}"));
    assert_eq!(rested["table"]["status"]["space"], below.as_str());
    assert_status_geometry(&rested["table"]["status"]);
    app.capture("panned-tag");
    let scrolled = app.request(
        json!({"op":"scroll","revision":0,"component":format!("space:{below}"),"y":2}),
        true,
    );
    assert_eq!(scrolled["status"], "scrolled", "{scrolled}");
    let zoomed = app.request(json!({"op":"settle"}), true);
    let distance = |view: &Value| view["table"]["camera"]["distance"].as_f64().unwrap();
    assert!(
        (distance(&zoomed) - distance(&moved) * 0.9025).abs() < 0.001,
        "the wheel zooms below the old opening: {}",
        zoomed["table"]["camera"]
    );
    app.point(&format!("space:{below}"));
    let dragged = app.request(
        json!({"op":"drag","revision":0,"component":format!("space:{below}"),"button":"right","x":120,"y":30}),
        true,
    );
    assert_eq!(dragged["status"], "dragged", "{dragged}");
    let orbited = app.request(json!({"op":"settle"}), true);
    let yaw = |view: &Value| view["table"]["camera"]["yaw"].as_f64().unwrap();
    assert!(
        (yaw(&orbited) - (yaw(&zoomed) - 0.72)).abs() < 0.0001,
        "a right-drag orbits below the old opening: {}",
        orbited["table"]["camera"]
    );
    // Tags still open at this camera position, clear of every area.
    let clear = space_in_the_clear(&orbited);
    let tagged = app.point(&format!("space:{clear}"));
    assert_eq!(tagged["table"]["status"]["space"], clear.as_str());
    assert_status_geometry(&tagged["table"]["status"]);
    let tag = rect_of(&tagged["table"]["status"]["bounds"]);
    for (area, rect) in areas(&tagged) {
        assert!(disjoint(tag, rect), "{clear}'s tag {tag:?} covers {area}");
    }
    app.capture("orbited-tag");
    app.request(json!({"op":"key","key":"Home","revision":0}), true);
    assert_eq!(camera(&app.observe()), home_camera, "Home returns");

    // An overlaid area takes the pointer over its own rectangle: on each, the wheel,
    // a right-drag and a press leave the camera and the match as they were. Save ·
    // quit's plate covers its whole area (#174), so the wheel and the drag go to the
    // plate, and it is not pressed: that would close the game.
    for area in areas(&rest).keys() {
        let plate = area == "save-quit";
        let target = if plate {
            "quit".to_owned()
        } else {
            format!("area:{area}")
        };
        let over = app.point(&target);
        assert!(over["table"]["status"].is_null(), "{area}");
        let scrolled = app.request(
            json!({"op":"scroll","revision":0,"component":target,"y":2}),
            true,
        );
        assert_eq!(scrolled["status"], "scrolled", "{scrolled}");
        let after = app.request(json!({"op":"settle"}), true);
        assert_eq!(camera(&after), home_camera, "a scroll on {area} zoomed");
        let dragged = app.request(
            json!({"op":"drag","revision":0,"component":target,"button":"right","x":120,"y":30}),
            true,
        );
        assert_eq!(dragged["status"], "dragged", "{dragged}");
        let after = app.request(json!({"op":"settle"}), true);
        assert_eq!(camera(&after), home_camera, "a drag on {area} orbited");
        if plate {
            continue;
        }
        app.activate(&target, 0);
        let after = app.observe();
        assert_eq!(after["observation"], rest["observation"], "{area}");
        assert_eq!(camera(&after), home_camera, "{area}");
        assert!(after["table"]["popover"].is_null(), "{area}");
        assert!(after["table"]["status"].is_null(), "{area}");
    }
    // Zoom in until a space lies under an area: the pointer there is on the area,
    // not on the space, and the space's tag does not open.
    let mut near = app.point("space:camaguey-city");
    let mut under = None;
    for _ in 0..12 {
        under = space_under_an_area(&near);
        if under.is_some() {
            break;
        }
        app.request(
            json!({"op":"scroll","revision":0,"component":"space:camaguey-city","y":2}),
            true,
        );
        near = app.request(json!({"op":"settle"}), true);
    }
    let (covered, area) = under.expect("zooming in brings a space under an area");
    eprintln!("{covered} lies under {area}");
    assert_ne!(camera(&near), home_camera);
    assert_covered(&mut app, &covered, 0);
    app.capture("under-an-area");
    app.request(json!({"op":"key","key":"Home","revision":0}), true);

    // Terror's targets: highlights and popovers at Home and at a moved camera, and
    // no popover through an area.
    app.choose("terror");
    let targeting = app.observe();
    let revision = targeting["observation"]["revision"].as_u64().unwrap();
    assert_map_agrees(&targeting);
    assert_eq!(target_state(&targeting, "santiago"), "selectable");
    app.activate("space:santiago", revision);
    let opened = app.observe();
    assert_eq!(opened["table"]["popover"]["pinned"], true, "{opened}");
    let card = rect_of(&opened["table"]["popover"]["bounds"]);
    for (area, rect) in areas(&opened) {
        assert!(
            disjoint(card, rect),
            "Santiago's card {card:?} covers {area}"
        );
    }
    app.capture("targeting-home-popover");
    app.request(json!({"op":"key","key":"Escape","revision":revision}), true);
    assert!(app.observe()["table"]["popover"].is_null());
    // Orbit and zoom, on a space that stays in view.
    app.point("space:camaguey-city");
    app.request(
        json!({"op":"drag","revision":revision,"component":"space:camaguey-city","button":"right","x":-60,"y":-20}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    app.request(
        json!({"op":"scroll","revision":revision,"component":"space:camaguey-city","y":2}),
        true,
    );
    let turned = app.request(json!({"op":"settle"}), true);
    assert_ne!(camera(&turned), home_camera);
    assert_map_agrees(&turned);
    let santiago = space_points(&turned)["santiago"];
    assert!(
        areas(&turned)
            .values()
            .all(|area| !holds(*area, santiago, -20.)),
        "Santiago stays clear at this camera position"
    );
    app.activate("space:santiago", revision);
    let opened = app.observe();
    assert_eq!(opened["table"]["popover"]["pinned"], true, "{opened}");
    let card = rect_of(&opened["table"]["popover"]["bounds"]);
    assert!(
        card[0] >= 0. && card[1] >= 0. && card[2] <= 3840. && card[3] <= 2160.,
        "{card:?}"
    );
    app.capture("targeting-moved-popover");
    app.request(json!({"op":"key","key":"Escape","revision":revision}), true);
    // Zoom in until a target lies under an area: the seat cannot press it there,
    // and no card opens.
    let mut near = app.observe();
    let mut hidden = None;
    for _ in 0..12 {
        let areas = areas(&near);
        hidden = ["santiago", "oriente"].into_iter().find(|space| {
            let at = space_points(&near)[*space];
            areas.values().any(|area| holds(*area, at, 12.))
        });
        if hidden.is_some() {
            break;
        }
        app.request(
            json!({"op":"scroll","revision":revision,"component":"space:camaguey-city","y":2}),
            true,
        );
        near = app.request(json!({"op":"settle"}), true);
    }
    let hidden = hidden.expect("zooming in brings a Terror target under an area");
    eprintln!("{hidden} is a target under an area");
    assert_covered(&mut app, hidden, revision);
    let pressed = app.request(
        json!({"op":"activate","revision":revision,"component":format!("space:{hidden}")}),
        false,
    );
    assert_eq!(pressed["status"], "missed", "{pressed}");
    let after = app.observe();
    assert!(
        after["table"]["popover"].is_null(),
        "{}",
        after["table"]["popover"]
    );
    assert_eq!(after["observation"], targeting["observation"]);
    app.capture("targeting-under-an-area");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_main_view_keeps_areas_and_picks_in_other_windows() {
    let layout = record();
    // 3:2 and 21:9: the layout is authored at 16:9 only (foundation 4).
    for (width, height) in [(1920, 1280), (2520, 1080)] {
        let mut app = launch(
            &format!("main-view-{width}x{height}-"),
            &format!("{width}x{height}"),
        );
        let rest = app.observe();
        assert_eq!(
            rect_of(&rest["table"]["map_viewport"]),
            [0., 0., f64::from(width), f64::from(height)]
        );
        assert_areas_and_island(&rest, &layout, f64::from(width), f64::from(height));
        app.capture("home");
        assert_every_space_picks(&mut app, &rest);
        app.capture("home-tag");
        // Every other target the window offers is inside it.
        for control in rest["table"]["controls"].as_array().unwrap() {
            let at = [
                control["position"][0].as_f64().unwrap(),
                control["position"][1].as_f64().unwrap(),
            ];
            assert!(
                holds([0., 0., f64::from(width), f64::from(height)], at, 0.),
                "{control} is outside the {width}x{height} window"
            );
        }
        app.quit();
    }
}
