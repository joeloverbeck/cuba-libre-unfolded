//! Required selected art and privately supplied card faces, admitted before saves.
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use unfolded_assets::{read_bounded, read_verified, regular_file};
pub const MANIFEST: &str = include_str!("../art/atlas/runtime.sha256");
// Stateless hover disclosures are required at launch, but do not redefine the
// map/cards identity to which existing game records are bound.
pub const STATUS_MANIFEST: &str = include_str!("../art/atlas/status.sha256");
pub const VICTORY_MANIFEST: &str = include_str!("../art/atlas/victory.sha256");
/// The main view's layout record and the area frames it places (#142): stateless
/// presentation. The record says where each area sits over the full-window map.
pub const MAIN_VIEW_MANIFEST: &str = include_str!("../art/atlas/main-view.sha256");
pub const TERRAIN_MANIFEST: &str = include_str!("../art/atlas/terrain.sha256");
pub const TERROR_MANIFEST: &str = include_str!("../art/atlas/terror.sha256");
pub const TOP_MANIFEST: &str = include_str!("../art/atlas/top.sha256");
/// The selected map highlight family (HF-A, #72): stateless presentation, like the
/// status tags, so it does not change the save-bound identity.
pub const HIGHLIGHT_MANIFEST: &str = include_str!("../art/atlas/highlights.sha256");
pub const CHOICE_MANIFEST: &str = include_str!("../art/atlas/choice.sha256");
/// The selected EL-A eligibility indicator (#70): stateless presentation.
pub const ELIGIBILITY_MANIFEST: &str = include_str!("../art/atlas/eligibility.sha256");
/// The selected transfer modal (TM-B, #71): stateless presentation.
pub const TRANSFER_MANIFEST: &str = include_str!("../art/atlas/transfer.sha256");
/// The Kidnap die, its tray and its record (#74). Like the Terror bell, presentation
/// art that does not change what a save records.
pub const KIDNAP_MANIFEST: &str = include_str!("../art/atlas/kidnap.sha256");
/// The selected HF-A popover's production plates (#58 review): stateless presentation.
pub const POPOVER_MANIFEST: &str = include_str!("../art/atlas/popover.sha256");
/// The selected EX-A execution panel's pills (#84): stateless presentation.
pub const EXECUTION_MANIFEST: &str = include_str!("../art/atlas/execution.sha256");
/// The card tag's mark plates (#139): stateless presentation.
pub const CARD_TAG_MANIFEST: &str = include_str!("../art/atlas/card-tag.sha256");
/// Every manifest admitted at launch, in the order `inspect` reads them.
pub const MANIFESTS: [&str; 15] = [
    MANIFEST,
    STATUS_MANIFEST,
    VICTORY_MANIFEST,
    MAIN_VIEW_MANIFEST,
    TERRAIN_MANIFEST,
    TERROR_MANIFEST,
    TOP_MANIFEST,
    HIGHLIGHT_MANIFEST,
    CHOICE_MANIFEST,
    ELIGIBILITY_MANIFEST,
    TRANSFER_MANIFEST,
    KIDNAP_MANIFEST,
    POPOVER_MANIFEST,
    EXECUTION_MANIFEST,
    CARD_TAG_MANIFEST,
];
pub const SOURCE: &str = "deacd635db5f71f4d3f092bd9cc99a9d96bb7aab;R:8152cb0056e3690fdc01c7b4ff893369ad7eff288941abc540ceee2c9d606792;P:dcaa5afd6f01c0c9ed4746c18199c1b09058823d67b627384fd3184ed4015fe8";
pub(crate) struct CardFace {
    pub(crate) name: &'static str,
    pub(crate) number: u8,
    pub(crate) size: (u32, u32),
    length: u64,
    hash: &'static str,
}

const CARD_FACES: [CardFace; 3] = [
    CardFace {
        name: "card-17.png",
        number: 17,
        size: (572, 820),
        length: 1138467,
        hash: "141f592b5dee5a8c7a6d26651410b0ee1171a7dcf2c2b5331a63b48407cec9ab",
    },
    CardFace {
        name: "card-18.png",
        number: 18,
        size: (573, 820),
        length: 1159661,
        hash: "50df7058e660bb4dc620096823945eeb095fe4123e0aead68fec6244ab097c4e",
    },
    CardFace {
        name: "card-19.png",
        number: 19,
        size: (573, 819),
        length: 1089575,
        hash: "10bf73402f7fbb79d14aad0b928bc8ea20949ea5bd888799a869a6f4e270bc3b",
    },
];

#[cfg(feature = "tabletop")]
pub(crate) fn card_face(number: u8) -> &'static CardFace {
    &CARD_FACES[usize::from(number - 17)]
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn identity() -> String {
    card_identity(&CARD_FACES[..2])
}

pub(crate) fn continuation_identity() -> String {
    card_identity(&CARD_FACES)
}

fn card_identity(faces: &[CardFace]) -> String {
    use std::fmt::Write;

    let mut source = MANIFEST.to_owned();
    for face in faces {
        write!(source, "\n{}:{}", face.name, face.hash).expect("writing to a String");
    }
    digest(source.as_bytes())
}
pub struct Verified {
    pub root: PathBuf,
    pub files: BTreeMap<String, Vec<u8>>,
}
pub fn inspect(root: &Path, cards: &Path) -> Result<Verified> {
    let mut files = BTreeMap::new();
    for line in MANIFESTS.iter().flat_map(|manifest| manifest.lines()) {
        let (hash, path) = line
            .split_once("  ")
            .context("Invalid compiled art manifest")?;
        let local = path
            .strip_prefix("art/atlas/")
            .context("Invalid art path")?;
        let bytes = read_bounded(
            &regular_file(root, local)
                .with_context(|| format!("Required art missing: {}", root.join(local).display()))?,
            4 * 1024 * 1024,
        )?;
        if digest(&bytes) != hash {
            bail!(
                "Required art changed: {}. Restore the selected Atlas asset.",
                root.join(local).display()
            );
        }
        if local.ends_with(".png") {
            decode(&bytes, local, None)?;
        }
        files.insert(local.to_owned(), bytes);
    }
    for face in &CARD_FACES {
        let bytes = read_verified(cards, face.name, face.length, face.hash).with_context(|| {
            format!(
                "Required private card {} missing or incompatible. Restore {} ({}, SHA-256 {}) or set --cards to a directory with the selected faces.",
                cards.join(face.name).display(), face.name, crate::CardFacts::title_of(face.number), face.hash,
            )
        })?;
        decode(&bytes, face.name, Some(face.size))?;
        files.insert(face.name.into(), bytes);
    }
    Ok(Verified {
        root: root.canonicalize()?,
        files,
    })
}
fn decode(bytes: &[u8], name: &str, size: Option<(u32, u32)>) -> Result<()> {
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .with_context(|| format!("Cannot decode {name}"))?;
    if size.is_some_and(|expected| expected != (image.width(), image.height())) {
        bail!("Wrong dimensions for {name}");
    }
    Ok(())
}
