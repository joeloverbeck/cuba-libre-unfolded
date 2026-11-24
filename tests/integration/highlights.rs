//! #72: the shipped map highlights are the selected HF-A exports, unchanged, each with
//! its place on the map in a manifest and admitted by hash at launch.
//! #73 adds the Santiago and Oriente veils, the removal-mode piece markers and the
//! popover's four row marks, admitted the same way. #167 gives every space its own
//! selectable rule, blocked rule and veil, replacing the three grouped veils.
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
fn every_highlight_is_an_hf_a_export_admitted_by_hash() {
    let manifest = manifest();
    let graphics = manifest["graphics"].as_array().unwrap();
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
    // #167: every space has its own selectable rule, blocked rule and veil.
    assert_eq!(graphics.len(), 39);
    let mut proposal_exports = vec![];
    for graphic in graphics {
        let file = graphic["file"].as_str().unwrap();
        let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
        assert_eq!(
            admitted(&format!("art/atlas/{file}")),
            assets::digest(&bytes)
        );
        let artboard = graphic["artboard"].as_str().unwrap();
        assert!(artboard.starts_with("A-"), "{artboard} is not HF-A");
        // The graphics the proposal and #73 exported stay its exports, byte for byte.
        if let Ok(proposal) = std::fs::read(format!(
            "art/proposals/issue-66/graphics/map-{artboard}.png"
        )) {
            assert_eq!(bytes, proposal, "{file}");
            proposal_exports.push(file);
        }
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
    proposal_exports.sort_unstable();
    assert_eq!(
        proposal_exports,
        [
            "atlas-highlight-blocked-oriente.png",
            "atlas-highlight-blocked-santiago.png",
            "atlas-highlight-selectable-oriente.png",
            "atlas-highlight-selectable-santiago.png",
            "atlas-highlight-veil-oriente.png",
            "atlas-highlight-veil-santiago.png",
            "atlas-highlight-veil-sierra-maestra.png",
        ]
    );
}

/// The seven single-space graphics that predate #167 keep the bytes #72 and #73
/// admitted, so Santiago, Oriente and Sierra Maestra look as they did.
#[test]
fn the_seven_single_space_graphics_are_unchanged() {
    for (file, hash) in [
        (
            "atlas-highlight-blocked-oriente.png",
            "1f96b46ef6407affb16f5e8bb55315cf0e35ec341eef3c47890a2ee5736ab3d7",
        ),
        (
            "atlas-highlight-blocked-santiago.png",
            "9caf50133ef8b6f883a518925c83bf062c2d058503e5f70f730a0deca786a8dd",
        ),
        (
            "atlas-highlight-selectable-oriente.png",
            "f44e94b2200e9b652482c8fc8dbd713efee7bdc98ee1cd9a18f3c74529fe29e7",
        ),
        (
            "atlas-highlight-selectable-santiago.png",
            "20dc4199d4db8c4425fbcd5faab11e0d17bf9ccbe70aacc1b045cc828378607e",
        ),
        (
            "atlas-highlight-veil-oriente.png",
            "184c52146d3c5c36ec8b1a01d33689526987aba052437379a4c9166bd9555db2",
        ),
        (
            "atlas-highlight-veil-santiago.png",
            "f13060868a9c4ce0f6e8c4fe4d43ae8546b5e931b6d7924ea58815d4b2a28a88",
        ),
        (
            "atlas-highlight-veil-sierra-maestra.png",
            "42a7e63fe05f47267901d7a5a4ad50951af5e3aac8d5dd7f4cc2f7b23282af1a",
        ),
    ] {
        let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
        assert_eq!(assets::digest(&bytes), hash, "{file}");
        assert_eq!(admitted(&format!("art/atlas/{file}")), hash, "{file}");
    }
    // The three grouped veils are gone from the art and from both manifests.
    for group in ["provinces", "cities", "ecs"] {
        let file = format!("atlas-highlight-veil-{group}.png");
        assert!(!std::path::Path::new(&format!("art/atlas/{file}")).exists());
        assert!(!assets::HIGHLIGHT_MANIFEST.contains(&file));
        assert!(!manifest().to_string().contains(&file));
    }
}

/// #167: each of the 13 spaces has its own selectable rule, blocked rule and veil,
/// each covering that space alone, so the map can put any space in any state. Each is
/// placed at the height its space's veil had before #167: 0.242 on a City, 0.282 on an
/// economic center and 0.142 on a Province, the heights of #72's grouped veils.
#[test]
fn every_space_has_its_own_rule_dashed_rule_and_veil() {
    use cuba_libre_unfolded::Terrain;
    let manifest = manifest();
    let graphics = manifest["graphics"].as_array().unwrap();
    let game = crate::terror_chosen(Game::opening()).observe();
    assert_eq!(game.spaces.len(), 13);
    for space in &game.spaces {
        let here: Vec<&Value> = graphics
            .iter()
            .filter(|g| g["space"] == space.id.as_str())
            .collect();
        let mut states: Vec<&str> = here.iter().map(|g| g["state"].as_str().unwrap()).collect();
        states.sort_unstable();
        assert_eq!(
            states,
            ["blocked", "not-involved", "selectable"],
            "{}",
            space.id
        );
        let height = match space.terrain {
            Terrain::City => 0.242,
            Terrain::EconomicCenter => 0.282,
            _ => 0.142,
        };
        for graphic in &here {
            assert_eq!(graphic["height"].as_f64().unwrap(), height, "{graphic}");
            // The rules and the veil share the space's rectangle.
            assert_eq!(graphic["rect"], here[0]["rect"], "{graphic}");
            let state = match graphic["state"].as_str().unwrap() {
                "selectable" => "selectable",
                "blocked" => "blocked",
                _ => "veil",
            };
            assert_eq!(
                graphic["file"],
                format!("atlas-highlight-{state}-{}.png", space.id),
                "{graphic}"
            );
        }
    }
    assert_eq!(graphics.len(), 3 * game.spaces.len());
}
