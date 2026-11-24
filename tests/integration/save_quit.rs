//! #174: Save · quit's plate is admitted at launch, at rest and hovered, and the
//! layout record places it, the notice slip and its tag in the bottom-right corner.
use cuba_libre_unfolded::assets;
use serde_json::Value;

/// Hash admission (`save-quit.sha256`): a missing or changed export of the plate
/// refuses launch before any save, like every other manifest.
#[test]
fn launch_refuses_a_missing_or_changed_save_quit_export() {
    let art = tempfile::tempdir().unwrap();
    for line in assets::MANIFESTS.iter().flat_map(|m| m.lines()) {
        let (_, path) = line.split_once("  ").unwrap();
        let to = art.path().join(path.strip_prefix("art/atlas/").unwrap());
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(path, to).unwrap();
    }
    // No private cards here: intact art passes and reaches the card check.
    let cards = tempfile::tempdir().unwrap();
    let error = || {
        format!(
            "{:#}",
            assets::inspect(art.path(), cards.path()).err().unwrap()
        )
    };
    assert!(error().contains("Required private card"));
    let admitted: Vec<&str> = assets::SAVE_QUIT_MANIFEST
        .lines()
        .map(|line| {
            line.split_once("  ")
                .unwrap()
                .1
                .strip_prefix("art/atlas/")
                .unwrap()
        })
        .collect();
    assert_eq!(
        admitted,
        ["atlas-save-quit.png", "atlas-save-quit-hover.png"]
    );
    for name in admitted {
        let path = art.path().join(name);
        let original = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let missing = error();
        assert!(
            missing.contains("Required art missing") && missing.contains(name),
            "{missing}"
        );
        let mut changed = original.clone();
        changed[original.len() / 2] ^= 1;
        std::fs::write(&path, changed).unwrap();
        let changed = error();
        assert!(
            changed.contains("Required art changed") && changed.contains(name),
            "{changed}"
        );
        std::fs::write(&path, original).unwrap();
    }
    assert!(error().contains("Required private card"));
}

/// [x, y, width, height] as [left, top, right, bottom].
fn edges(rect: &Value) -> [f64; 4] {
    let v: Vec<f64> = rect
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_f64().unwrap())
        .collect();
    [v[0], v[1], v[0] + v[2], v[1] + v[3]]
}

fn disjoint(a: [f64; 4], b: [f64; 4]) -> bool {
    a[2] <= b[0] || a[0] >= b[2] || a[3] <= b[1] || a[1] >= b[3]
}

/// Layout A · Franja on the owner's display: the plate 67.5 (40 reference pixels)
/// from the right and bottom edges; the slip above it, right edges aligned, at the
/// popover's width (340 reference pixels); the tag above the plate at the top display
/// tag's width (468). Neither covers the plate, the decision panel or the island box,
/// and nothing of the session panel is left in the record.
#[test]
fn the_record_places_the_plate_the_slip_and_the_tag_in_the_corner() {
    let layout: Value =
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap();
    let plate = edges(&layout["areas"]["save-quit"]["bounds"]);
    let slip = edges(&layout["notice-slip"]["bounds"]);
    let tag = edges(&layout["save-quit-tag"]["bounds"]);
    let close = |a: f64, b: f64| (a - b).abs() < 0.02;
    assert!(close(3840. - plate[2], 67.5) && close(2160. - plate[3], 67.5));
    assert!(close(slip[2], plate[2]) && close(slip[2] - slip[0], 340. * 1.6875));
    assert!(close(tag[2], plate[2]) && close(tag[2] - tag[0], 468. * 1.6875));
    assert!(slip[3] < plate[1] && tag[3] < plate[1]);
    for anchored in ["notice-slip", "save-quit-tag"] {
        assert_eq!(layout[anchored]["anchor"], "bottom-right");
    }
    let island = edges(&layout["island"]["bounds"]);
    for (name, placed) in layout["areas"].as_object().unwrap() {
        if name == "save-quit" {
            continue;
        }
        let area = edges(&placed["bounds"]);
        assert!(disjoint(slip, area), "the slip covers {name}");
        assert!(disjoint(tag, area), "the tag covers {name}");
    }
    assert!(disjoint(slip, island) && disjoint(tag, island));
    // The slip's words lie inside it.
    let words = edges(&layout["notice-slip"]["text"]);
    assert!(words[0] > slip[0] && words[1] > slip[1] && words[2] < slip[2] && words[3] < slip[3]);
    let all = layout.to_string();
    assert!(
        !all.contains("\"session"),
        "the session panel is still recorded"
    );
}
