//! #159: the Available Forces ledger at rest, against its layout record, and its
//! counts after a voluntary removal. The Rally case is in `rally.rs`, beside the
//! Rally it plays. #160: each row's tag, on hover and on focus, at three windows.
use super::*;

/// The ledger's counts as the seat reports them: (faction, kind, count) for every
/// cell, row by row.
pub(super) fn ledger(view: &Value) -> Vec<(String, String, u64)> {
    view["table"]["available_forces"]["rows"]
        .as_array()
        .expect("the seat reports the ledger's rows")
        .iter()
        .flat_map(|row| {
            row["cells"].as_array().unwrap().iter().map(|cell| {
                (
                    row["faction"].as_str().unwrap().to_owned(),
                    cell["kind"].as_str().unwrap().to_owned(),
                    cell["count"].as_u64().unwrap(),
                )
            })
        })
        .collect()
}

/// The drawn count beside a piece, as the seat reads its text.
pub(super) fn drawn_count(view: &Value, part: &str) -> String {
    part_of(view, &format!("{part}-count"))["text"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn part_of(view: &Value, name: &str) -> Value {
    view["table"]["available_forces"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|part| part["part"] == name)
        .unwrap_or_else(|| panic!("the ledger draws no {name}"))
        .clone()
}

/// The prepared scenario's Available Forces, from the accepted tactical handoff,
/// which derived them from the p. 20 chart.
pub(super) fn prepared() -> Vec<(String, String, u64)> {
    [
        ("Government", "Troops", 3),
        ("Government", "Police", 8),
        ("Government", "Base", 2),
        ("M26", "Guerrilla", 12),
        ("M26", "Base", 3),
        ("Directorio", "Guerrilla", 12),
        ("Directorio", "Base", 4),
        ("Syndicate", "Guerrilla", 6),
        ("Syndicate", "Casino", 7),
    ]
    .into_iter()
    .map(|(f, k, n)| (f.to_owned(), k.to_owned(), n))
    .collect()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_available_forces_ledger_matches_layout_and_follows_a_removal() {
    // 3840x2160 is the owner's display and the record's frame: one unit is one pixel.
    let mut app = start("available-forces-", "issue-159", "3840x2160");
    let rest = app.observe();
    let layout: Value = serde_json::from_str(include_str!(
        "../../art/atlas/atlas-available-forces.layout.json"
    ))
    .unwrap();
    assert_eq!(layout["frame"], json!([3840, 2160]));
    let tolerance = layout["tolerance"].as_f64().unwrap();
    // Images on all four edges; text on left, top and right, since its line height
    // is the renderer's rather than the layout's.
    for (group, edges) in [("parts", 4), ("text", 3)] {
        for (name, rect) in layout[group].as_object().unwrap() {
            let [x, y, w, h] = rect_of(rect);
            let expected = [x, y, x + w, y + h];
            let native = rect_of(&part_of(&rest, name)["bounds"]);
            for (n, e) in native.iter().zip(expected).take(edges) {
                assert!(
                    (n - e).abs() <= tolerance,
                    "{name}: native {native:?} differs from layout {expected:?} by more than {tolerance} px"
                );
            }
        }
    }
    // Directly left of the Resources ledger, at its top and height.
    let plate = rect_of(&part_of(&rest, "plate")["bounds"]);
    let resources = rect_of(
        &rest["table"]["top_parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|part| part["part"] == "ledger")
            .unwrap()["bounds"],
    );
    assert!(plate[2] < resources[0], "{plate:?} {resources:?}");
    assert!((plate[1] - resources[1]).abs() <= tolerance);
    assert!((plate[3] - resources[3]).abs() <= tolerance);
    assert_eq!(part_of(&rest, "caption")["text"], "AVAILABLE FORCES");
    // The prepared counts, drawn as the rows report them, and no piece dimmed.
    let rows = ledger(&rest);
    assert_eq!(rows, prepared());
    for (faction, kind, count) in &rows {
        let slug = match faction.as_str() {
            "Government" => "government",
            "M26" => "26july",
            "Directorio" => "directorio",
            _ => "syndicate",
        };
        let part = format!("{slug}-{}", kind.to_lowercase());
        assert_eq!(drawn_count(&rest, &part), count.to_string(), "{part}");
        assert_eq!(part_of(&rest, &part)["dimmed"], false, "{part}");
    }
    app.capture("ledger-at-rest");

    // The Resources tag opens beside the Resources ledger, over this one (#157's
    // open question for the owner). The capture is for the owner's judgement.
    let tagged = app.point("resources");
    assert_eq!(tagged["table"]["top"]["item"], "resources");
    assert_eq!(ledger(&tagged), prepared());
    app.capture("resources-tag-over-ledger");

    // A voluntary removal returns the Sierra Maestra Base to 26July's Available
    // Forces (R 1.4.1): 3 Bases become 4, and nothing else changes.
    app.choose("terror");
    app.choose("remove");
    app.remove("sierra-maestra:base", 1);
    app.choose("done-removal");
    let removed = app.observe();
    let mut expected = prepared();
    expected[4].2 = 4;
    assert_eq!(ledger(&removed), expected);
    assert_eq!(drawn_count(&removed, "26july-base"), "4");
    app.capture("after-removal");
    app.quit();
}

/// A faction's tag as the seat reports it (`table.top`): its rows as (label, value).
fn tag_rows(view: &Value) -> Rows {
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

fn pairs(rows: &[(&str, &str)]) -> Rows {
    rows.iter()
        .map(|(l, v)| ((*l).to_owned(), (*v).to_owned()))
        .collect()
}

/// A tag's rows as (label, value).
type Rows = Vec<(String, String)>;

/// The prepared scenario's four tags, from the p. 20 chart and the accepted tactical
/// handoff: (component, title, rows).
fn prepared_tags() -> Vec<(&'static str, &'static str, Rows)> {
    vec![
        (
            "available:government",
            "Available Government Forces",
            pairs(&[
                ("Troops", "3 of 15 · 12 on the map"),
                ("Police", "8 of 15 · 7 on the map"),
                ("Bases", "2 of 2 · 0 on the map"),
            ]),
        ),
        (
            "available:26july",
            "Available 26July Forces",
            pairs(&[
                ("Guerrillas", "12 of 15 · 3 on the map"),
                ("Bases", "3 of 4 · 1 on the map"),
            ]),
        ),
        (
            "available:directorio",
            "Available Directorio Forces",
            pairs(&[
                ("Guerrillas", "12 of 15 · 3 on the map"),
                ("Bases", "4 of 4 · 0 on the map"),
            ]),
        ),
        (
            "available:syndicate",
            "Available Syndicate Forces",
            pairs(&[
                ("Guerrillas", "6 of 6 · 0 on the map"),
                ("Casinos", "7 of 10 · 3 on the map"),
            ]),
        ),
    ]
}

const RULE: &str = "Pieces are placed only from Available Forces.\nWith none of a kind left, a placement is skipped\nand a replacement only removes (R 1.4.1).\nWhile executing, a faction may return its own\npieces from the map to Available.";

fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// The open tag stays inside the window and covers no area of the main view: neither
/// ledger, the Cash supply, the standings, the decision panel nor any other. Returns
/// the spaces whose drawn bounds it covers, which are all it covers.
fn assert_tag_covers_only_the_map(view: &Value, at: &str) -> Vec<String> {
    let plate = rect_of(&view["table"]["top"]["bounds"]);
    let [w, h] = [
        view["table"]["viewport"][0].as_f64().unwrap(),
        view["table"]["viewport"][1].as_f64().unwrap(),
    ];
    assert!(
        plate[0] >= 0. && plate[1] >= 0. && plate[2] <= w && plate[3] <= h,
        "{at}: {plate:?} leaves the {w}x{h} window"
    );
    for (name, area) in view["table"]["main_view"]["areas"].as_object().unwrap() {
        assert!(
            !overlaps(plate, rect_of(area)),
            "{at}: the tag {plate:?} covers {name}"
        );
    }
    for part in view["table"]["top_parts"].as_array().unwrap() {
        assert!(
            !overlaps(plate, rect_of(&part["bounds"])),
            "{at}: the tag {plate:?} covers {}",
            part["part"]
        );
    }
    for part in view["table"]["available_forces"]["parts"]
        .as_array()
        .unwrap()
    {
        assert!(
            !overlaps(plate, rect_of(&part["bounds"])),
            "{at}: the tag {plate:?} covers {}",
            part["part"]
        );
    }
    for line in view["table"]["top_text"].as_array().unwrap() {
        let text = rect_of(&line["bounds"]);
        assert!(
            text[0] >= plate[0] && text[2] <= plate[2] && text[3] <= plate[3],
            "{at}: tag line {line} leaves its plate {plate:?}"
        );
    }
    view["table"]["main_view"]["spaces"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, bounds)| overlaps(plate, rect_of(bounds)))
        .map(|(space, _)| space.clone())
        .collect()
}

/// Launches the app at `size`, its evidence in a fresh `prefix` directory under
/// `local/verification/<issue>` unless `CUBA_NATIVE_EVIDENCE` names another root.
fn start(prefix: &str, issue: &str, size: &str) -> NativeRun {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| format!("local/verification/{issue}").into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket = tempfile::tempdir().unwrap();
    let mut app = NativeRun {
        app: Native {
            cards: std::env::var_os("CUBA_TEST_CARDS")
                .expect("Set CUBA_TEST_CARDS")
                .into(),
            socket: socket.path().join("seat.sock"),
            save: root.join("task.json"),
            root,
            step: 0,
        },
        _socket: socket,
    };
    eprintln!("Available Forces native evidence: {}", app.root.display());
    app.launch_size(size);
    app
}

/// Tab until `name` has focus, then waits out the shared dwell.
fn focus(app: &mut Native, name: &str) -> Value {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    let mut focus = Value::Null;
    for _ in 0..30 {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        focus = app.observe()["table"]["choice"]["focus"].clone();
        if focus == name {
            break;
        }
        // During Terror the map's spaces come before the rows (#168); a focused
        // space's tag holds the table for its dwell.
        app.request(json!({"op":"settle"}), true);
    }
    assert_eq!(focus, name);
    app.request(json!({"op":"settle"}), true);
    app.observe()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_available_forces_tags_open_on_hover_and_focus_and_follow_a_removal() {
    let mut app = start("available-tags-", "issue-160", "3840x2160");
    let layout: Value = serde_json::from_str(include_str!(
        "../../art/atlas/atlas-available-forces.layout.json"
    ))
    .unwrap();
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let rest = app.observe();
    assert!(rest["table"]["top"].is_null());
    let mut covered = serde_json::Map::new();
    // Hover: each row opens its faction's tag after the shared dwell.
    for (component, title, rows) in prepared_tags() {
        let shown = app.point(component);
        assert_eq!(shown["table"]["hover"], component);
        assert_eq!(shown["table"]["top"]["item"], component);
        assert_eq!(shown["table"]["top"]["title"], title);
        assert_eq!(tag_rows(&shown), rows, "{component}");
        let notes: Vec<&str> = shown["table"]["top"]["notes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap())
            .collect();
        assert_eq!(notes.join("\n"), RULE);
        // Reading a tag changes nothing.
        assert_eq!(shown["observation"], rest["observation"]);
        assert_eq!(ledger(&shown), prepared());
        let spaces = assert_tag_covers_only_the_map(&shown, component);
        covered.insert(format!("3840x2160 {component}"), json!(spaces));
        // Every tag opens at the record's place; the Government's is the frame.
        let plate = rect_of(&shown["table"]["top"]["bounds"]);
        let [x, y, w, h] = rect_of(&layout["tag"]["open"]);
        let frame = &layout["tag"]["frame"];
        assert!((plate[0] - x).abs() <= tolerance && (plate[1] - y).abs() <= tolerance);
        assert!((plate[2] - (x + w)).abs() <= tolerance, "{plate:?}");
        if component == "available:government" {
            assert!((plate[3] - (y + h)).abs() <= tolerance, "{plate:?}");
            assert!((w - frame[0].as_f64().unwrap()).abs() < 0.01);
            // The text where the frame puts it: left, top and right.
            let lines = shown["table"]["top_text"].as_array().unwrap();
            let line = |words: &str| {
                rect_of(
                    &lines
                        .iter()
                        .find(|l| l["text"] == words)
                        .unwrap_or_else(|| panic!("no tag line {words}"))["bounds"],
                )
            };
            let mut expected = vec![("title", title.to_owned())];
            for (n, (label, value)) in rows.iter().enumerate() {
                expected.push((
                    ["row-0-label", "row-1-label", "row-2-label"][n],
                    label.clone(),
                ));
                expected.push((
                    ["row-0-value", "row-1-value", "row-2-value"][n],
                    value.clone(),
                ));
            }
            for (part, words) in expected {
                let native = line(&words);
                let [px, py, pw, _] = rect_of(&layout["tag"]["text"][part]);
                let record = [x + px, y + py, x + px + pw];
                for (n, e) in native.iter().zip(record) {
                    assert!(
                        (n - e).abs() <= tolerance,
                        "{part}: {native:?} against {record:?}"
                    );
                }
            }
            // The rule is one drawn block, as the frame's one `notes` box sets it.
            assert_eq!(shown["table"]["top"]["notes"], json!([RULE]));
            let notes = line(RULE);
            let [px, py, pw, _] = rect_of(&layout["tag"]["text"]["notes"]);
            for (n, e) in notes.iter().zip([x + px, y + py, x + px + pw]) {
                assert!((n - e).abs() <= tolerance, "notes: {notes:?}");
            }
        }
        app.capture(&format!(
            "tag-{}",
            component.trim_start_matches("available:")
        ));
    }
    // The tag closes when the pointer leaves, as the top display's tags do.
    let left = app.point("quit");
    assert!(left["table"]["top"].is_null(), "{}", left["table"]["top"]);

    // A voluntary removal returns the Sierra Maestra Base to 26July's Available
    // Forces (R 1.4.1): its tag reads 4 of 4 and none on the map, the ledger 4.
    app.choose("terror");
    app.choose("remove");
    app.remove("sierra-maestra:base", 1);
    app.choose("done-removal");
    let removed = app.point("available:26july");
    assert_eq!(
        tag_rows(&removed),
        pairs(&[
            ("Guerrillas", "12 of 15 · 3 on the map"),
            ("Bases", "4 of 4 · 0 on the map"),
        ])
    );
    assert_eq!(drawn_count(&removed, "26july-base"), "4");
    app.capture("tag-26july-after-removal");
    app.point("quit");

    // Focus: Tab reaches the four rows after the panel's controls and the map's spaces
    // (#168), and each opens its
    // tag with the same dwell; the next stop, the current card, closes it.
    let focused = focus(&mut app, "available:government");
    assert_eq!(focused["table"]["top"]["item"], "available:government");
    assert_eq!(tag_rows(&focused), prepared_tags()[0].2);
    app.capture("focus-government");
    let focused = focus(&mut app, "available:26july");
    assert_eq!(focused["table"]["top"]["item"], "available:26july");
    assert_eq!(tag_rows(&focused)[1].1, "4 of 4 · 0 on the map");
    for name in ["available:directorio", "available:syndicate"] {
        assert_eq!(focus(&mut app, name)["table"]["top"]["item"], name);
    }
    let card = focus(&mut app, "card:current");
    assert!(card["table"]["top"].is_null(), "{}", card["table"]["top"]);
    app.quit();

    // At 3:2 and 21:9 the Government's tag still covers only the map, inside the
    // window. The match resumes from its save.
    for size in ["1920x1280", "2520x1080"] {
        app.launch_size(size);
        let shown = app.point("available:government");
        assert_eq!(shown["table"]["top"]["item"], "available:government");
        let spaces = assert_tag_covers_only_the_map(&shown, size);
        covered.insert(format!("{size} available:government"), json!(spaces));
        app.capture(&format!("tag-government-{size}"));
        app.quit();
    }
    std::fs::write(
        app.root.join("covered-spaces.json"),
        serde_json::to_vec_pretty(&covered).unwrap(),
    )
    .unwrap();
}
