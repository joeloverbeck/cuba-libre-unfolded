//! #179: the focus mark's exports are admitted at launch.
use cuba_libre_unfolded::assets;

/// `focus-mark.sha256` admits the mark, its light form and its round cut.
#[test]
fn the_mark_s_exports_are_admitted() {
    assert_eq!(
        crate::admission::admitted(assets::FOCUS_MARK_MANIFEST),
        [
            "atlas-focus-mark.png",
            "atlas-focus-mark-field.png",
            "atlas-focus-mark-pill.png"
        ]
    );
}
