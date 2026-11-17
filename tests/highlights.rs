//! #72: the shipped map highlights are the selected HF-A exports, unchanged, each with
//! its place on the map in a presentation manifest and admitted by hash at launch.
//! #73 adds the Santiago and Oriente veils, the removal-mode piece markers and the
//! popover's four row marks, admitted the same way.
use cuba_libre_unfolded::{Game, assets};
use serde_json::Value;

fn manifest() -> Value {
    serde_json::from_slice(&std::fs::read("art/atlas/highlights.json").unwrap()).unwrap()
}

fn admitted(path: &str) -> String {
    assets::HIGHLIGHT_MANIFEST
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

#[test]
fn every_highlight_is_an_unchanged_hf_a_export_admitted_by_hash() {
    let manifest = manifest();
    let graphics = manifest["graphics"].as_array().unwrap();
    assert_eq!(graphics.len(), 10);
    let pieces = manifest["pieces"].as_array().unwrap();
    let marks = manifest["marks"].as_array().unwrap();
    // The manifest itself is admitted, so a changed placement refuses at launch too.
    let json = std::fs::read("art/atlas/highlights.json").unwrap();
    assert_eq!(admitted("art/atlas/highlights.json"), assets::digest(&json));
    assert_eq!(
        assets::HIGHLIGHT_MANIFEST.lines().count(),
        graphics.len() + pieces.len() + marks.len() + 1
    );
    // Removal mode's markers: the proposal's HF-A piece exports, byte for byte, at
    // 8 px per unit over the square they are placed on.
    let mut states = vec![];
    for piece in pieces {
        let file = piece["file"].as_str().unwrap();
        let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
        assert_eq!(
            admitted(&format!("art/atlas/{file}")),
            assets::digest(&bytes)
        );
        let artboard = piece["artboard"].as_str().unwrap();
        assert!(artboard.starts_with("A-piece-"), "{artboard}");
        let proposal = std::fs::read(format!(
            "art/proposals/issue-66/graphics/piece-{artboard}.png"
        ))
        .unwrap();
        assert_eq!(bytes, proposal, "{file}");
        let side = piece["size"].as_f64().unwrap() * 8.;
        assert_eq!(size(&bytes), (side as u32, side as u32), "{file}");
        states.push(piece["mark"].as_str().unwrap().to_owned());
    }
    states.sort();
    assert_eq!(states, ["protected", "removable"]);
    // The four popover row marks, exported from the layout at 8x.
    let mut kinds = vec![];
    for mark in marks {
        let file = mark["file"].as_str().unwrap();
        let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
        assert_eq!(
            admitted(&format!("art/atlas/{file}")),
            assets::digest(&bytes)
        );
        let side = mark["size"].as_f64().unwrap() * 8.;
        assert_eq!(size(&bytes), (side as u32, side as u32), "{file}");
        kinds.push(mark["mark"].as_str().unwrap().to_owned());
    }
    kinds.sort();
    assert_eq!(kinds, ["done", "not-yet", "selectable", "unavailable"]);
    for graphic in graphics {
        let file = graphic["file"].as_str().unwrap();
        let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
        assert_eq!(
            admitted(&format!("art/atlas/{file}")),
            assets::digest(&bytes)
        );
        // The proposal's export of the selected alternative's artboard, byte for byte.
        let artboard = graphic["artboard"].as_str().unwrap();
        assert!(artboard.starts_with("A-"), "{artboard} is not HF-A");
        let proposal = std::fs::read(format!(
            "art/proposals/issue-66/graphics/map-{artboard}.png"
        ))
        .unwrap();
        assert_eq!(bytes, proposal, "{file}");
        // Exported at 4 px per map unit over the rectangle it is placed on.
        let rect: Vec<f64> = graphic["rect"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let (w, h) = size(&bytes);
        assert_eq!(
            (f64::from(w), f64::from(h)),
            (rect[2] * 4., rect[3] * 4.),
            "{file}"
        );
    }
}

/// Every space has a veil, so any space with nothing left in this task, Santiago and
/// Oriente included, dims (#72's known gap, closed by #73); only those two get rules.
#[test]
fn every_space_has_a_veil_and_only_targets_get_rules() {
    let manifest = manifest();
    let game = Game::prepared().observe();
    let mut veiled = vec![];
    for graphic in manifest["graphics"].as_array().unwrap() {
        let spaces: Vec<&str> = graphic["spaces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        for space in &spaces {
            assert!(game.spaces.iter().any(|s| s.id == *space), "{space}");
        }
        match graphic["state"].as_str().unwrap() {
            "not-involved" => veiled.extend(spaces),
            "selectable" | "blocked" => {
                assert!(
                    spaces == ["santiago"] || spaces == ["oriente"],
                    "{spaces:?}"
                );
            }
            other => panic!("unknown state {other}"),
        }
    }
    veiled.sort_unstable();
    let mut every: Vec<&str> = game.spaces.iter().map(|s| s.id.as_str()).collect();
    every.sort_unstable();
    assert_eq!(veiled, every);
}
