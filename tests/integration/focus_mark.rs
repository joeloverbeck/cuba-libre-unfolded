//! #179: the focus mark's exports are admitted at launch.
use cuba_libre_unfolded::assets;

/// Hash admission (`focus-mark.sha256`): a missing or changed export of the mark
/// refuses launch before any save, like every other manifest.
#[test]
fn launch_refuses_a_missing_or_changed_focus_mark_export() {
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
    let mut names = vec![];
    for line in assets::FOCUS_MARK_MANIFEST.lines() {
        let name = line.split_once("  art/atlas/").unwrap().1;
        names.push(name);
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
    assert_eq!(
        names,
        [
            "atlas-focus-mark.png",
            "atlas-focus-mark-field.png",
            "atlas-focus-mark-pill.png"
        ]
    );
    assert!(error().contains("Required private card"));
}
