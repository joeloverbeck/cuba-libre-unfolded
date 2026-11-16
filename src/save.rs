//! Cuba Libre's adapter onto the shared save record.
//!
//! `unfolded_record_replacement::save` owns the envelope, its checksum, the write
//! baseline, the unchanged shortcut, validation of the previous record, the
//! regular-file check and recovery into an unused path. This file keeps what
//! identifies a compatible save (format, accepted source, selected art), the
//! game's validity check, the size limit and the wording of every refusal.
use crate::{Faction, Game, Observation, assets};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use unfolded_record_replacement::save::{self, Baseline, Format, Identity, Write};
const LIMIT: u64 = 256 * 1024;

struct Cuba;
impl Cuba {
    fn current() -> Identity {
        [
            ("format", "1"),
            ("source", assets::SOURCE),
            ("art", &assets::identity()),
        ]
        .into_iter()
        .collect()
    }
}
impl Format for Cuba {
    type Payload = Game;
    const LIMIT: u64 = LIMIT;
    fn identity(&self, _: &Game) -> Identity {
        Self::current()
    }
    fn accepts(&self, identity: &Identity) -> bool {
        *identity == Self::current()
    }
    fn validate(&self, game: &Game) -> Result<(), String> {
        game.validate()
    }
}

/// What the player was doing when a save was refused, for its wording.
#[derive(Clone, Copy, PartialEq)]
enum Action {
    Open,
    Recover,
    Continue,
}

fn words(error: save::Error, action: Action) -> anyhow::Error {
    match error {
        save::Error::Unsupported(_) => anyhow!(
            "Incompatible save format, accepted source or selected art; original file preserved"
        ),
        save::Error::Malformed(detail) => {
            anyhow::Error::msg(detail).context("Malformed save; original file preserved")
        }
        save::Error::Checksum => anyhow!("Save checksum mismatch; original file preserved"),
        save::Error::Unreproducible(detail) | save::Error::Invalid(detail) => anyhow!(detail),
        save::Error::NotRegularFile => {
            anyhow!("Save must be a regular file, not a symlink or directory")
        }
        save::Error::Stale => anyhow!("Save changed in another session; reload before continuing"),
        save::Error::Missing => {
            anyhow!("Save disappeared; refusing to overwrite an unobserved baseline")
        }
        save::Error::Occupied | save::Error::SameRecord if action == Action::Recover => {
            anyhow!("Recovery requires an unused destination; neither record was changed")
        }
        save::Error::Occupied => anyhow!("Destination changed before creation"),
        save::Error::Orphaned => anyhow!(
            "Main save is missing but recovery history exists; recover explicitly to an unused path"
        ),
        other => anyhow!(other),
    }
}

pub struct Session {
    path: PathBuf,
    baseline: Baseline,
    game: Game,
}
impl Session {
    /// Native callers invoke this only after art decoding and render admission.
    pub fn open(path: &Path, recover_from: Option<&Path>) -> Result<Self> {
        let (game, baseline) = if let Some(source) = recover_from {
            save::recover(&Cuba, source, path, |_| Ok::<_, save::Error>(()))
                .map_err(|error| words(error, Action::Recover))?
        } else if let Some(saved) =
            save::load(&Cuba, path).map_err(|error| words(error, Action::Open))?
        {
            saved
        } else {
            // Draw OS entropy once, then persist the exact shared RNG state.
            // Reopening any existing record never reseeds it.
            let game = Game::prepared_with_seed(unfolded_random::fresh_seed()?);
            let mut baseline = Baseline::default();
            save::write(&Cuba, path, &mut baseline, &game, Write::Create)
                .map_err(|error| words(error, Action::Open))?;
            (game, baseline)
        };
        Ok(Self {
            path: path.to_path_buf(),
            baseline,
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
        save::write(
            &Cuba,
            &self.path,
            &mut self.baseline,
            candidate,
            Write::Replace,
        )
        .map_err(|error| words(error, Action::Continue))?;
        Ok(())
    }
}
