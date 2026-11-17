//! #72: the shipped map highlights are the selected HF-A exports, unchanged, each with
//! its place on the map in a presentation manifest and admitted by hash at launch.
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
    assert_eq!(graphics.len(), 8);
    // The manifest itself is admitted, so a changed placement refuses at launch too.
    let json = std::fs::read("art/atlas/highlights.json").unwrap();
    assert_eq!(admitted("art/atlas/highlights.json"), assets::digest(&json));
    assert_eq!(
        assets::HIGHLIGHT_MANIFEST.lines().count(),
        graphics.len() + 1
    );
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

#[test]
fn the_veils_cover_every_space_that_is_never_a_target_and_only_targets_get_rules() {
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
    let mut never_targets: Vec<&str> = game
        .spaces
        .iter()
        .map(|s| s.id.as_str())
        .filter(|id| !matches!(*id, "santiago" | "oriente"))
        .collect();
    never_targets.sort_unstable();
    assert_eq!(veiled, never_targets);
}
