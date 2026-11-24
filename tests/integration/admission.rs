//! Hash admission: a missing or changed file of any manifest in `MANIFESTS` refuses
//! launch before any save. New art is covered once its manifest is listed there.
use crate::explore::parallel;
use cuba_libre_unfolded::assets;
use std::path::Path;

/// The files a manifest admits, by their paths under `art/atlas/`.
pub fn admitted(manifest: &str) -> Vec<&str> {
    manifest
        .lines()
        .map(|line| line.split_once("  art/atlas/").unwrap().1)
        .collect()
}

fn all() -> Vec<&'static str> {
    let mut names: Vec<_> = assets::MANIFESTS.iter().flat_map(|m| admitted(m)).collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// A copy of the admitted art.
fn copy(to: &Path) {
    for name in all() {
        let path = to.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(Path::new("art/atlas").join(name), path).unwrap();
    }
}

/// No private cards here: intact art passes and reaches the card check.
fn refusal(art: &Path, cards: &Path) -> String {
    format!("{:#}", assets::inspect(art, cards).err().unwrap())
}

/// Every manifest beside the art is in `MANIFESTS`, so the test below reaches its files.
/// `palette-sources.sha256` is a test record, not an admission manifest.
#[test]
fn every_manifest_is_admitted() {
    let mut written: Vec<String> = std::fs::read_dir("art/atlas")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "sha256"))
        .filter(|path| !path.ends_with("palette-sources.sha256"))
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect();
    let mut listed = assets::MANIFESTS.map(str::to_owned).to_vec();
    written.sort_unstable();
    listed.sort_unstable();
    assert_eq!(listed, written);
}

#[test]
fn launch_refuses_any_missing_or_changed_admitted_file() {
    let names = all();
    let cards = tempfile::tempdir().unwrap();
    // Each share breaks files in its own copy of the art.
    let shares: Vec<_> = names.chunks(16).collect();
    parallel(&shares, |_, share| {
        let art = tempfile::tempdir().unwrap();
        copy(art.path());
        let refused = || refusal(art.path(), cards.path());
        assert!(refused().contains("Required private card"), "{}", refused());
        for name in share.iter() {
            let path = art.path().join(name);
            let original = std::fs::read(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            let missing = refused();
            assert!(
                missing.contains("Required art missing") && missing.contains(name),
                "{missing}"
            );
            let mut changed = original.clone();
            changed[original.len() / 2] ^= 1;
            std::fs::write(&path, changed).unwrap();
            let changed = refused();
            assert!(
                changed.contains("Required art changed") && changed.contains(name),
                "{changed}"
            );
            std::fs::write(&path, original).unwrap();
        }
        assert!(refused().contains("Required private card"));
    });
}
