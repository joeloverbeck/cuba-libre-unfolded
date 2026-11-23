//! #159: the Available Forces ledger at rest, against its layout record, and its
//! counts after a voluntary removal. The Rally case is in `rally.rs`, beside the
//! Rally it plays.
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
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-159".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("available-forces-")
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
    // 3840x2160 is the owner's display and the record's frame: one unit is one pixel.
    app.launch_size("3840x2160");
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
