//! Required selected art and the privately supplied card faces and card back,
//! admitted at launch before any save opens. No save's identity names any of it.
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use unfolded_assets::{read_bounded, read_verified, regular_file};
pub const MANIFEST: &str = include_str!("../art/atlas/runtime.sha256");
pub const STATUS_MANIFEST: &str = include_str!("../art/atlas/status.sha256");
pub const VICTORY_MANIFEST: &str = include_str!("../art/atlas/victory.sha256");
/// The main view's layout record and the area frames it places (#142): stateless
/// presentation. The record says where each area sits over the full-window map.
pub const MAIN_VIEW_MANIFEST: &str = include_str!("../art/atlas/main-view.sha256");
pub const TERRAIN_MANIFEST: &str = include_str!("../art/atlas/terrain.sha256");
pub const TERROR_MANIFEST: &str = include_str!("../art/atlas/terror.sha256");
pub const TOP_MANIFEST: &str = include_str!("../art/atlas/top.sha256");
/// The selected map highlight family (HF-A, #72): stateless presentation.
pub const HIGHLIGHT_MANIFEST: &str = include_str!("../art/atlas/highlights.sha256");
pub const CHOICE_MANIFEST: &str = include_str!("../art/atlas/choice.sha256");
/// The selected EL-A eligibility indicator (#70): stateless presentation.
pub const ELIGIBILITY_MANIFEST: &str = include_str!("../art/atlas/eligibility.sha256");
/// The selected transfer modal (TM-B, #71): stateless presentation.
pub const TRANSFER_MANIFEST: &str = include_str!("../art/atlas/transfer.sha256");
/// The Kidnap die, its tray and its record (#74): presentation, like the Terror bell.
pub const KIDNAP_MANIFEST: &str = include_str!("../art/atlas/kidnap.sha256");
/// The selected HF-A popover's production plates (#58 review): stateless presentation.
pub const POPOVER_MANIFEST: &str = include_str!("../art/atlas/popover.sha256");
/// The selected EX-A execution panel's pills (#84): stateless presentation.
pub const EXECUTION_MANIFEST: &str = include_str!("../art/atlas/execution.sha256");
/// The card tag's mark plates (#139): stateless presentation.
pub const CARD_TAG_MANIFEST: &str = include_str!("../art/atlas/card-tag.sha256");
/// The Available Forces ledger's plate and its nine pieces (#159): stateless presentation.
pub const AVAILABLE_FORCES_MANIFEST: &str = include_str!("../art/atlas/available-forces.sha256");
/// The marker supply's marker, at rest and dimmed (#172): stateless presentation.
pub const MARKER_SUPPLY_MANIFEST: &str = include_str!("../art/atlas/marker-supply.sha256");
/// Save · quit's plate, at rest and hovered (#174): stateless presentation.
pub const SAVE_QUIT_MANIFEST: &str = include_str!("../art/atlas/save-quit.sha256");
/// The focus mark and its light form (#179): stateless presentation.
pub const FOCUS_MARK_MANIFEST: &str = include_str!("../art/atlas/focus-mark.sha256");
/// The seat controls (#206): the selected A1 · Tarjetas cards and A2's strip.
pub const SEATS_MANIFEST: &str = include_str!("../art/atlas/seats.sha256");
/// Every manifest admitted at launch, in the order `inspect` reads them.
pub const MANIFESTS: [&str; 20] = [
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
    AVAILABLE_FORCES_MANIFEST,
    MARKER_SUPPLY_MANIFEST,
    SAVE_QUIT_MANIFEST,
    FOCUS_MARK_MANIFEST,
    SEATS_MANIFEST,
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

/// The original shared card back (foundation 4's exception, #136): a private input
/// beside the faces, shown while a card lies or turns face down (#145).
pub(crate) struct CardBack {
    pub(crate) name: &'static str,
    size: (u32, u32),
    length: u64,
    hash: &'static str,
}

pub(crate) const CARD_BACK: CardBack = CardBack {
    name: "cuba-libre-card-back.png",
    size: (1475, 2110),
    length: 8214679,
    hash: "c905d7a7313480b254e49dc70c56c41fe77e3039b037b238485e148cc82dbaa9",
};

#[cfg(feature = "tabletop")]
pub(crate) fn card_face(number: u8) -> &'static CardFace {
    &CARD_FACES[usize::from(number - 17)]
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
                cards.join(face.name).display(), face.name, crate::title(face.number), face.hash,
            )
        })?;
        decode(&bytes, face.name, Some(face.size))?;
        files.insert(face.name.into(), bytes);
    }
    let back = &CARD_BACK;
    let bytes = read_verified(cards, back.name, back.length, back.hash).with_context(|| {
        format!(
            "Required private card back {} missing or incompatible. Restore {} (the original shared card back, SHA-256 {}) or set --cards to a directory with it and the selected faces.",
            cards.join(back.name).display(),
            back.name,
            back.hash,
        )
    })?;
    decode(&bytes, back.name, Some(back.size))?;
    files.insert(back.name.into(), bytes);
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
