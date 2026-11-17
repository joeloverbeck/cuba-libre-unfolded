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
pub const LAYOUT_MANIFEST: &str = include_str!("../art/atlas/layout.sha256");
pub const TERRAIN_MANIFEST: &str = include_str!("../art/atlas/terrain.sha256");
pub const TERROR_MANIFEST: &str = include_str!("../art/atlas/terror.sha256");
pub const TOP_MANIFEST: &str = include_str!("../art/atlas/top.sha256");
/// The selected map highlight family (HF-A, #72): stateless presentation, like the
/// status tags, so it does not change the save-bound identity.
pub const HIGHLIGHT_MANIFEST: &str = include_str!("../art/atlas/highlights.sha256");
pub const CHOICE_MANIFEST: &str = include_str!("../art/atlas/choice.sha256");
pub const SOURCE: &str = "deacd635db5f71f4d3f092bd9cc99a9d96bb7aab;R:8152cb0056e3690fdc01c7b4ff893369ad7eff288941abc540ceee2c9d606792;P:dcaa5afd6f01c0c9ed4746c18199c1b09058823d67b627384fd3184ed4015fe8";
const CARD_17: &str = "141f592b5dee5a8c7a6d26651410b0ee1171a7dcf2c2b5331a63b48407cec9ab";
const CARD_18: &str = "50df7058e660bb4dc620096823945eeb095fe4123e0aead68fec6244ab097c4e";
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn identity() -> String {
    digest(format!("{MANIFEST}\ncard-17.png:{CARD_17}\ncard-18.png:{CARD_18}").as_bytes())
}
pub struct Verified {
    pub root: PathBuf,
    pub files: BTreeMap<String, Vec<u8>>,
}
pub fn inspect(root: &Path, cards: &Path) -> Result<Verified> {
    let mut files = BTreeMap::new();
    for line in MANIFEST
        .lines()
        .chain(STATUS_MANIFEST.lines())
        .chain(VICTORY_MANIFEST.lines())
        .chain(LAYOUT_MANIFEST.lines())
        .chain(TERRAIN_MANIFEST.lines())
        .chain(TERROR_MANIFEST.lines())
        .chain(TOP_MANIFEST.lines())
        .chain(HIGHLIGHT_MANIFEST.lines())
        .chain(CHOICE_MANIFEST.lines())
    {
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
    for (name, length, hash, size) in [
        ("card-17.png", 1138467, CARD_17, (572, 820)),
        ("card-18.png", 1159661, CARD_18, (573, 820)),
    ] {
        let bytes=read_verified(cards,name,length,hash).with_context(||format!("Required private card {} missing or incompatible. Set --cards to the directory containing the selected card faces.",cards.join(name).display()))?;
        decode(&bytes, name, Some(size))?;
        files.insert(name.into(), bytes);
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
