//! #172: the marker supply's size, its admission at launch, and the supply in every
//! position this build reaches.
use cuba_libre_unfolded::{Game, MARKER_SUPPLY, assets};

/// R 3.3.4: Terror and Sabotage markers share one supply of 20.
#[test]
fn the_supply_s_size_is_declared_once() {
    assert_eq!(MARKER_SUPPLY, 20);
}

/// Hash admission (`marker-supply.sha256`): a missing or changed marker export
/// refuses launch before any save, like every other manifest.
#[test]
fn launch_refuses_a_missing_or_changed_marker_export() {
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
    let admitted: Vec<&str> = assets::MARKER_SUPPLY_MANIFEST
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
        ["atlas-marker-supply.png", "atlas-marker-supply-dimmed.png"]
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

/// In every position the explorer reaches, the supply is the 20 less the markers on
/// the map: the Terror markers are those the spaces hold, and no Sabotage marker is
/// placed in this build. No reached position empties the supply, so its zero state is
/// established only by the display model's unit test from a constructed observation.
#[test]
fn every_reached_position_accounts_for_all_20_markers() {
    let starts = crate::explore::starts().into_iter().map(|(_, start)| start);
    let mut positions = 0;
    let mut fewest = MARKER_SUPPLY;
    for start in starts.chain(crate::explore::event_starts()) {
        for node in crate::explore::explore(start) {
            let view = node.game.observe();
            let supply = view.marker_supply;
            let on_spaces: u8 = view.spaces.iter().map(|s| s.terror).sum();
            assert_eq!(supply.terror_on_map, on_spaces, "position {positions}");
            assert_eq!(supply.sabotage_on_map, 0, "position {positions}");
            assert_eq!(
                supply.available + supply.terror_on_map + supply.sabotage_on_map,
                MARKER_SUPPLY,
                "position {positions}"
            );
            fewest = fewest.min(supply.available);
            positions += 1;
        }
    }
    assert!(positions > 100, "{positions}");
    // Two Terror targets, Santiago and Oriente, each resolved once at most.
    assert_eq!(fewest, 18);
    assert_eq!(Game::opening().observe().marker_supply.available, 20);
}
