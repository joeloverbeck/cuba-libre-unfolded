//! #142: the main view's layout record and its admission, without a GPU.
use cuba_libre_unfolded::assets;
use serde_json::Value;

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
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

/// Hash admission (`main-view.sha256`): a missing or changed layout record or area
/// frame refuses launch before any save, like the other presentation manifests.
#[test]
fn launch_refuses_a_missing_or_changed_layout_record_or_frame() {
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
    let intact = error();
    assert!(intact.contains("Required private card"), "{intact}");
    let admitted: Vec<&str> = assets::MAIN_VIEW_MANIFEST
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
        [
            "atlas-main-view.layout.json",
            "atlas-layout-operation.png",
            "atlas-layout-session.png",
            "atlas-layout-card.png",
        ]
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

/// The desk, the map's header and footer, and with them the "CUBA" title, the
/// terrain key and the adjacency line, are no longer admitted or shipped: they were
/// lettering inside those exports, so nothing can draw them.
#[test]
fn the_framed_map_s_chrome_is_retired() {
    for name in [
        "atlas-layout-background.png",
        "atlas-layout-map-fill.png",
        "atlas-layout-map-header.png",
        "atlas-layout-map-footer.png",
    ] {
        assert!(
            assets::MANIFESTS.iter().all(|m| !m.contains(name)),
            "{name} is still admitted"
        );
        assert!(
            !std::path::Path::new("art/atlas").join(name).exists(),
            "{name} is still shipped"
        );
    }
}

/// The record places the seven areas apart from each other and from the island box,
/// all inside the 16:9 frame, and says where #143's piles, labels, count discs,
/// captions and Propaganda pips go and where the card tag opens.
#[test]
fn the_record_places_the_areas_the_island_and_the_planned_piles() {
    let layout = record();
    assert_eq!(layout["frame"], serde_json::json!([3840, 2160]));
    let frame = [0., 0., 3840., 2160.];
    let inside = |r: [f64; 4]| r[0] >= 0. && r[1] >= 0. && r[2] <= frame[2] && r[3] <= frame[3];
    let areas = layout["areas"].as_object().unwrap();
    let mut anchors: Vec<(&str, &str)> = areas
        .iter()
        .map(|(name, placed)| (name.as_str(), placed["anchor"].as_str().unwrap()))
        .collect();
    anchors.sort_unstable();
    assert_eq!(
        anchors,
        [
            ("cards", "bottom-left"),
            ("decision", "top-right"),
            ("outside-influence", "top-left"),
            ("resources", "top-right"),
            ("sequence-of-play", "top-left"),
            ("session", "bottom-right"),
            ("standings", "top"),
        ]
    );
    let island = edges(&layout["island"]["bounds"]);
    assert_eq!(layout["island"]["anchor"], "edges");
    assert!(inside(island));
    for (name, placed) in areas {
        let area = edges(&placed["bounds"]);
        assert!(inside(area), "{name}");
        assert!(disjoint(area, island), "{name} covers the island box");
        for (other, placed) in areas {
            assert!(
                name == other || disjoint(area, edges(&placed["bounds"])),
                "{name} overlaps {other}"
            );
        }
    }
    // Every accepted part lies inside the frame.
    for group in ["parts", "text", "interim"] {
        for (name, rect) in layout[group].as_object().unwrap() {
            assert!(inside(edges(rect)), "{name}");
        }
    }
    let planned = layout["planned"].as_object().unwrap();
    for name in [
        "played-pile",
        "draw-deck",
        "played-label",
        "deck-label",
        "played-count-disc",
        "played-count",
        "deck-count-disc",
        "deck-count",
        "played-caption",
        "deck-caption",
        "propaganda-label",
        "propaganda-pip-0",
        "propaganda-pip-1",
        "propaganda-pip-2",
        "propaganda-pip-3",
        "propaganda-caption",
        "card-tag",
    ] {
        let rect = edges(
            planned
                .get(name)
                .unwrap_or_else(|| panic!("the layout plans no {name}")),
        );
        assert!(inside(rect), "{name}");
        if name != "card-tag" {
            // The piles lie on the open sea at the bottom left, clear of the island
            // and of every area but the card area they replace.
            assert!(disjoint(rect, island), "{name} covers the island box");
            for (area, placed) in areas {
                assert!(
                    area == "cards" || disjoint(rect, edges(&placed["bounds"])),
                    "{name} overlaps {area}"
                );
            }
        }
    }
    // The two piles are square to each other and apart (#137, user story 14).
    let (played, deck) = (edges(&planned["played-pile"]), edges(&planned["draw-deck"]));
    assert_eq!((played[1], played[3]), (deck[1], deck[3]));
    assert!(played[2] < deck[0]);
    // The card tag leaves both piles visible.
    let tag = edges(&planned["card-tag"]);
    assert!(disjoint(tag, played) && disjoint(tag, deck));
}
