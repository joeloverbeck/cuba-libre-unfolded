//! #58 review: the HF-A popover's chrome is the production plates of its FigCraft
//! source (`art/atlas/atlas-popover.figcraft`), exported at 3x, recorded in
//! `atlas-popover.layout.json` and admitted by hash at launch.
use cuba_libre_unfolded::assets;
use serde_json::Value;

fn record() -> Value {
    serde_json::from_slice(&std::fs::read("art/atlas/atlas-popover.layout.json").unwrap()).unwrap()
}

/// A PNG's width and height from its IHDR chunk.
fn size(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&bytes[12..16], b"IHDR");
    let read = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (read(16), read(20))
}

#[test]
fn every_popover_plate_is_a_recorded_export_admitted_by_hash() {
    let record = record();
    let plates = record["plates"].as_object().unwrap();
    assert_eq!(plates.len(), 11);
    assert_eq!(assets::POPOVER_MANIFEST.lines().count(), plates.len());
    for (name, plate) in plates {
        let path = format!("art/atlas/{name}.png");
        let bytes = std::fs::read(&path).unwrap();
        let hash = assets::POPOVER_MANIFEST
            .lines()
            .find_map(|line| {
                let (hash, file) = line.split_once("  ").unwrap();
                (file == path).then_some(hash)
            })
            .unwrap_or_else(|| panic!("{path} is not admitted"));
        assert_eq!(hash, assets::digest(&bytes), "{path}");
        // Drawn at reference size and exported at 3x, with the card's shadow around it.
        let at = |key: &str, i: usize| plate[key][i].as_f64().unwrap();
        let (w, h) = size(&bytes);
        assert_eq!(
            (f64::from(w), f64::from(h)),
            (
                3. * (at("size", 0) + at("outset", 0) + at("outset", 2)),
                3. * (at("size", 1) + at("outset", 1) + at("outset", 3))
            ),
            "{name}"
        );
    }
    // Only the cards carry a margin: their drop shadow, `ink` at 20%, (0, 3), blur 10.
    for card in ["atlas-popover-card", "atlas-popover-card-slip"] {
        let outset = &plates[card]["outset"];
        assert_eq!(
            outset,
            &serde_json::json!([15.0, 12.0, 15.0, 18.0]),
            "{card}"
        );
    }
}

#[test]
fn the_record_is_the_a_p1_master_at_reference_size() {
    let record = record();
    assert_eq!(record["size"], serde_json::json!([340.0, 474.0]));
    let parts = record["parts"].as_object().unwrap();
    // The card is 340 wide; its rules and rows sit inside its 1 px stroke.
    for name in [
        "popover-rule",
        "popover-header",
        "popover-row-chosen",
        "popover-row",
    ] {
        let r = &parts[name];
        assert_eq!(
            (r[0].as_f64(), r[2].as_f64()),
            (Some(1.), Some(338.)),
            "{name}"
        );
    }
    // The commit button fills the chosen row inside its 14 px padding.
    assert_eq!(
        parts["popover-commit"],
        serde_json::json!([15.0, 296.0, 310.0, 31.0])
    );
}
