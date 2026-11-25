//! #142 and #143: the main view's layout record and its admission, without a GPU.
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

/// `main-view.sha256` admits the layout record, the area frame and the exports.
#[test]
fn the_layout_record_frame_and_exports_are_admitted() {
    assert_eq!(
        crate::admission::admitted(assets::MAIN_VIEW_MANIFEST),
        [
            "atlas-main-view.layout.json",
            "atlas-layout-operation.png",
            "atlas-main-view-count-disc.png",
            "atlas-main-view-pip-played.png",
            "atlas-main-view-pip.png",
            "atlas-main-view-pile-shadow.png",
        ]
    );
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
        // #143: the card panel's frame went with the panel.
        "atlas-layout-card.png",
        // #174: the session panel's frame went with the panel.
        "atlas-layout-session.png",
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

/// The record places the eleven areas apart from each other and from the island box,
/// all inside the 16:9 frame, and the card tray with its two piles, their labels,
/// count discs, captions and the Propaganda pips, and says where the card tag opens.
#[test]
fn the_record_places_the_areas_the_island_and_the_piles() {
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
            ("available-forces", "top-right"),
            ("cards", "bottom-left"),
            ("decision", "top-right"),
            // #172: centred between outside-influence and the standings.
            ("marker-supply", "top-between"),
            ("outside-influence", "top-left"),
            ("propaganda", "bottom-left"),
            ("resources", "top-right"),
            // #174: Save · quit in place of the session panel.
            ("save-quit", "bottom-right"),
            // #206: the seat cards, between the Propaganda pips and Save · quit.
            ("seats", "bottom-between"),
            ("sequence-of-play", "top-left"),
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
    for group in ["parts", "text"] {
        for (name, rect) in layout[group].as_object().unwrap() {
            assert!(inside(edges(rect)), "{name}");
        }
    }
    // The card panel's frame is no longer placed, and nothing is left as a plan.
    assert!(layout.get("interim").is_none() && layout.get("planned").is_none());

    // The card tray (#143): the part of the window its own camera draws.
    let tray = edges(&layout["tray"]["bounds"]);
    assert_eq!(layout["tray"]["anchor"], "bottom-left");
    assert!(inside(tray));
    let (played, deck) = (
        edges(&layout["tray"]["played-pile"]),
        edges(&layout["tray"]["draw-deck"]),
    );
    // The two piles are square to each other and apart (#137, user story 14): the
    // played pile on the left, the draw deck on the right, equal cards on one line.
    assert_eq!((played[1], played[3]), (deck[1], deck[3]));
    assert_eq!(played[2] - played[0], deck[2] - deck[0]);
    assert!(played[2] < deck[0]);
    let within = |inner: [f64; 4], outer: [f64; 4]| {
        inner[0] >= outer[0] && inner[1] >= outer[1] && inner[2] <= outer[2] && inner[3] <= outer[3]
    };
    let cards = edges(&areas["cards"]["bounds"]);
    let propaganda = edges(&areas["propaganda"]["bounds"]);
    for (pile, face, base, cards_in_it) in [
        ("played", played, "played-pile-base", 18.),
        ("deck", deck, "deck-pile-base", 34.),
    ] {
        // A pile's foot shows half its height below its top card. A card is 0.3 mm
        // of 63 mm: 300 / 210 of a frame unit seen edge-on.
        let foot = edges(&layout["tray"][base]);
        assert_eq!((foot[0], foot[2]), (face[0], face[2]), "{pile}");
        let shown = foot[3] - face[3];
        assert!(
            (shown - cards_in_it * (300. / 210.) / 2.).abs() < 0.01,
            "{pile}'s side shows {shown}"
        );
        // The whole pile lies in the tray and takes the pointer in the card area.
        for rect in [face, foot] {
            assert!(within(rect, tray), "{pile} leaves the tray");
            assert!(within(rect, cards), "{pile} leaves the card area");
            assert!(disjoint(rect, island), "{pile} covers the island box");
        }
    }
    // What lies over the tray: on the open sea at the bottom left, clear of the
    // island and of every area but its own.
    for (group, name, area) in [
        ("text", "played-label", None),
        ("text", "deck-label", None),
        ("parts", "played-count-disc", Some(cards)),
        ("text", "played-count", Some(cards)),
        ("parts", "deck-count-disc", Some(cards)),
        ("text", "deck-count", Some(cards)),
        ("text", "played-caption", Some(cards)),
        ("text", "deck-caption", Some(cards)),
        ("text", "propaganda-label", Some(propaganda)),
        ("parts", "propaganda-pip-0", Some(propaganda)),
        ("parts", "propaganda-pip-1", Some(propaganda)),
        ("parts", "propaganda-pip-2", Some(propaganda)),
        ("parts", "propaganda-pip-3", Some(propaganda)),
        ("text", "propaganda-caption", Some(propaganda)),
    ] {
        let rect = edges(
            layout[group]
                .get(name)
                .unwrap_or_else(|| panic!("the layout places no {name}")),
        );
        assert!(disjoint(rect, island), "{name} covers the island box");
        match area {
            Some(area) => assert!(within(rect, area), "{name} leaves its area"),
            // The labels stand just above the card area, on open sea.
            None => {
                for (area, placed) in areas {
                    assert!(
                        disjoint(rect, edges(&placed["bounds"])),
                        "{name} overlaps {area}"
                    );
                }
            }
        }
    }
    // Each count's number lies on its disc, and each disc on its pile's near corner.
    for (disc, count, face) in [
        ("played-count-disc", "played-count", played),
        ("deck-count-disc", "deck-count", deck),
    ] {
        let disc = edges(&layout["parts"][disc]);
        assert!(within(edges(&layout["text"][count]), disc));
        assert!(
            disc[0] < face[2] && face[2] < disc[2],
            "the disc is off the corner"
        );
    }
    // The card tag leaves both piles and their labels visible.
    let tag = edges(&layout["card-tag"]);
    assert!(inside(tag));
    for rect in [played, deck] {
        assert!(disjoint(tag, rect));
    }
    for name in ["played-label", "deck-label"] {
        assert!(disjoint(tag, edges(&layout["text"][name])), "{name}");
    }
    // Each export names a file the manifest admits, at the size the record states.
    for (node, export) in layout["exports"].as_object().unwrap() {
        let file = export["file"].as_str().unwrap();
        assert!(
            assets::MAIN_VIEW_MANIFEST
                .lines()
                .any(|line| line.ends_with(&format!("art/atlas/{file}"))),
            "{node}: {file} is not admitted"
        );
        let image = image::open(std::path::Path::new("art/atlas").join(file)).unwrap();
        assert_eq!(
            serde_json::json!([image.width(), image.height()]),
            export["size"],
            "{file}"
        );
    }
}
