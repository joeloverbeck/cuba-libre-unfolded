//! #159: the Available Forces ledger's art and its admission, without a GPU.
use cuba_libre_unfolded::assets;

/// Hash admission (`available-forces.sha256`): a missing or changed plate or piece
/// refuses launch before any save, like every other manifest.
#[test]
fn launch_refuses_a_missing_or_changed_ledger_export() {
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
    let admitted: Vec<&str> = assets::AVAILABLE_FORCES_MANIFEST
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
            "atlas-available-forces-plate.png",
            "atlas-available-troops.png",
            "atlas-available-police.png",
            "atlas-available-government-base.png",
            "atlas-available-26july-guerrilla.png",
            "atlas-available-26july-base.png",
            "atlas-available-directorio-guerrilla.png",
            "atlas-available-directorio-base.png",
            "atlas-available-syndicate-guerrilla.png",
            "atlas-available-syndicate-casino.png",
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
