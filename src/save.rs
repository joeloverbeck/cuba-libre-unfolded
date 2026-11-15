//! Cuba semantic admission around shared locked replacement; no parallel filesystem protocol.
use crate::{Faction, Game, Observation, assets};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use unfolded_tabletop::record_replacement::{self as records, Replacement};
const LIMIT: u64 = 256 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u32,
    source: String,
    art: String,
    game: Game,
    checksum: String,
}
impl Record {
    fn checksum(&self) -> Result<String> {
        Ok(assets::digest(&serde_json::to_vec(&(
            self.format,
            &self.source,
            &self.art,
            &self.game,
        ))?))
    }
}
fn encode(game: &Game) -> Result<Vec<u8>> {
    game.validate().map_err(anyhow::Error::msg)?;
    let mut record = Record {
        format: 1,
        source: assets::SOURCE.into(),
        art: assets::identity(),
        game: game.clone(),
        checksum: String::new(),
    };
    record.checksum = record.checksum()?;
    Ok(serde_json::to_vec_pretty(&record)?)
}
fn decode(bytes: &[u8]) -> Result<Game> {
    let record: Record =
        serde_json::from_slice(bytes).context("Malformed save; original file preserved")?;
    if record.format != 1 || record.source != assets::SOURCE || record.art != assets::identity() {
        bail!("Incompatible save format, accepted source or selected art; original file preserved");
    }
    if record.checksum != record.checksum()? {
        bail!("Save checksum mismatch; original file preserved");
    }
    record.game.validate().map_err(anyhow::Error::msg)?;
    Ok(record.game)
}
fn read(path: &Path) -> Result<Vec<u8>> {
    if !std::fs::symlink_metadata(path)?.is_file() {
        bail!("Save must be a regular file, not a symlink or directory");
    }
    records::read(path, LIMIT, anyhow::Error::msg)
}
pub struct Session {
    path: PathBuf,
    baseline: Vec<u8>,
    game: Game,
}
impl Session {
    /// Native callers invoke this only after art decoding and render admission.
    pub fn open(path: &Path, recover_from: Option<&Path>) -> Result<Self> {
        let (game, baseline) = if let Some(source) = recover_from {
            if path.try_exists()? {
                bail!("Recovery requires an unused destination; neither record was changed");
            }
            let bytes = read(&records::previous_path(source))?;
            let game = decode(&bytes)?;
            (game, None)
        } else if path.try_exists()? {
            let bytes = read(path)?;
            let game = decode(&bytes)?;
            (game, Some(bytes))
        } else {
            if records::previous_path(path).try_exists()? {
                bail!(
                    "Main save is missing but recovery history exists; recover explicitly to an unused path"
                );
            }
            // Ubuntu application: draw OS entropy once, then persist the exact
            // shared RNG state. Reopening any existing record never reseeds it.
            let mut seed = [0; 8];
            std::fs::File::open("/dev/urandom")?.read_exact(&mut seed)?;
            (Game::prepared_with_seed(u64::from_ne_bytes(seed)), None)
        };
        let bytes = baseline.clone().unwrap_or(encode(&game)?);
        if baseline.is_none() {
            records::replace(
                path,
                &bytes,
                LIMIT,
                anyhow::Error::msg,
                |existing| {
                    if existing.is_some() || records::previous_path(path).try_exists()? {
                        bail!("Destination changed before creation");
                    }
                    Ok(Replacement::Create)
                },
                |_| Ok(()),
            )?;
        }
        Ok(Self {
            path: path.to_path_buf(),
            baseline: bytes,
            game,
        })
    }
    pub fn game(&self) -> &Game {
        &self.game
    }
    pub fn observe(&self) -> Observation {
        self.game.observe()
    }
    pub fn submit(&mut self, owner: Faction, revision: u64, choice: &str) -> Result<()> {
        let mut candidate = self.game.clone();
        candidate
            .submit(owner, revision, choice)
            .map_err(anyhow::Error::msg)?;
        self.persist(&candidate)?;
        self.game = candidate;
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        self.persist(&self.game.clone())
    }
    fn persist(&mut self, candidate: &Game) -> Result<()> {
        let bytes = encode(candidate)?;
        records::replace(
            &self.path,
            &bytes,
            LIMIT,
            anyhow::Error::msg,
            |existing| {
                let existing = existing
                    .context("Save disappeared; refusing to overwrite an unobserved baseline")?;
                decode(existing)?;
                if existing != self.baseline {
                    bail!("Save changed in another session; reload before continuing");
                }
                Ok(if existing == bytes {
                    Replacement::Unchanged
                } else {
                    Replacement::Replace
                })
            },
            |previous| {
                decode(&read(previous)?)?;
                Ok(())
            },
        )?;
        self.baseline = bytes;
        Ok(())
    }
}
