//! #70: the eligibility indicator ships the selected EL-A graphics unchanged and its
//! layout plates, every file admitted by hash at launch, at the sizes its layout
//! record places them.
use cuba_libre_unfolded::assets;
use serde_json::Value;

fn admitted(path: &str) -> String {
    assets::ELIGIBILITY_MANIFEST
        .lines()
        .find_map(|line| {
            let (hash, file) = line.split_once("  ").unwrap();
            (file == path).then(|| hash.to_owned())
        })
        .unwrap_or_else(|| panic!("{path} is not admitted"))
}

/// A PNG's width and height from its IHDR chunk.
fn size(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&bytes[12..16], b"IHDR");
    let read = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    (read(16), read(20))
}

fn read(file: &str) -> Vec<u8> {
    let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
    assert_eq!(
        admitted(&format!("art/atlas/{file}")),
        assets::digest(&bytes),
        "{file}"
    );
    bytes
}

/// The VectorCraft graphics are the #64 proposal's exports of the selected EL-A
/// artboards, byte for byte (`briefing-eligibility-graphics.vectorcraft`, 3×).
#[test]
fn the_graphics_are_the_selected_el_a_exports_unchanged() {
    for (file, artboard) in [
        ("atlas-eligibility-header.png", "track-header"),
        ("atlas-eligibility-tag-1st.png", "tag-1st"),
        ("atlas-eligibility-tag-2nd.png", "tag-2nd"),
        ("atlas-eligibility-eligible.png", "mark-eligible"),
        ("atlas-eligibility-ineligible.png", "mark-ineligible"),
        ("atlas-eligibility-vacant.png", "slot-vacant"),
    ] {
        let proposal =
            std::fs::read(format!("art/proposals/issue-64/graphics/{artboard}.png")).unwrap();
        assert_eq!(read(file), proposal, "{file}");
    }
}

/// The plates are FigCraft exports of the layout's production frame at 3 px per
/// reference pixel (10/9 of a frame unit), the size the layout record places them.
#[test]
fn the_plates_are_admitted_at_three_times_their_reference_size() {
    let layout: Value =
        serde_json::from_slice(&std::fs::read("art/atlas/atlas-eligibility.layout.json").unwrap())
            .unwrap();
    let px = |frame_units: f64| (frame_units / 0.9).round() as u32;
    let bounds = |state: &str, part: &str| -> (u32, u32) {
        let r = &layout["states"][state]["images"][part];
        (px(r[2].as_f64().unwrap()), px(r[3].as_f64().unwrap()))
    };
    for (file, expected) in [
        (
            "atlas-eligibility-panel.png",
            bounds("operation-special-activity", "eligibility-panel"),
        ),
        (
            "atlas-eligibility-panel-open.png",
            bounds("open", "eligibility-panel"),
        ),
        (
            "atlas-eligibility-box-occupied.png",
            bounds("pass", "eligibility-box-occupied"),
        ),
    ] {
        let (w, h) = size(&read(file));
        assert!(
            w.abs_diff(expected.0) <= 1 && h.abs_diff(expected.1) <= 1,
            "{file}: {w}×{h}, layout {expected:?}"
        );
    }
    assert_eq!(assets::ELIGIBILITY_MANIFEST.lines().count(), 9);
}
