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

#[test]
fn launch_refuses_any_missing_or_changed_admitted_file() {
    let names = all();
    assert!(names.len() > 200, "{} admitted files", names.len());
    let cards = tempfile::tempdir().unwrap();
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    let shares: Vec<_> = names.chunks(names.len().div_ceil(workers)).collect();
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
