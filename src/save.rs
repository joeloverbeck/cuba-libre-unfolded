//! Cuba Libre's adapter onto the shared save record.
//!
//! `unfolded_record_replacement::save` owns the envelope, its checksum, the write
//! baseline, the unchanged shortcut, validation of the previous record, the
//! regular-file check and recovery into an unused path. This file keeps what a
//! save holds, what identifies it (format and accepted source), its admission by
//! exact replay, the size limit and the wording of Cuba Libre's refusals, a save
//! over its size limit included. Busy and file-system failures keep the shared
//! module's words.
//!
//! A save holds the seed the match opened with, every decision in order and the
//! position they produce. It is admitted only when replaying the decisions from
//! the opening position through the checked submission play uses reproduces the
//! position exactly ([ADR 0003](../docs/adr/0003-a-save-opens-by-exact-replay.md)).
//! The shared record runs that check on load, on each candidate before it is
//! written and on the kept `.previous` before it is rotated.
use crate::{Choice, Faction, Game, Observation, assets};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use unfolded_record_replacement::save::{self, Baseline, Format, Identity, Write};
/// About 4 MiB, far above a full game; a save at the limit is refused plainly.
const LIMIT: u64 = 4 * 1024 * 1024;
/// The save's shape. It changes whenever what a save holds changes shape.
const FORMAT: &str = "2";
/// Marks a refusal from exact-replay admission.
const REPLAY: &str = "cuba-replay:";

/// What a save holds inside the shared record's envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    /// The random state the match opened with.
    seed: u64,
    /// Every accepted decision in order, each as its choice's action without the
    /// match revision: `terror:santiago` for the choice `3:terror:santiago`.
    decisions: Vec<String>,
    /// The position the decisions produce.
    position: Game,
}
impl Payload {
    fn opening(seed: u64) -> Self {
        Self {
            seed,
            decisions: vec![],
            position: Game::opening_with_seed(seed),
        }
    }
    /// Exact replay: each decision, submitted at the then-current match revision
    /// as the owner of the offered choice, must be accepted, and the result must
    /// equal the saved position.
    fn replay(&self) -> Result<(), String> {
        let mut game = Game::opening_with_seed(self.seed);
        for (index, decision) in self.decisions.iter().enumerate() {
            game.apply(decision).map_err(|refusal| {
                format!(
                    "its decision {} of {}, {decision}, is refused: {refusal}",
                    index + 1,
                    self.decisions.len()
                )
            })?;
        }
        if game != self.position {
            return Err(match self.decisions.len() {
                1 => "its 1 decision does not reproduce its position".to_owned(),
                count => format!("its {count} decisions do not reproduce its position"),
            });
        }
        Ok(())
    }
}

struct Cuba;
impl Cuba {
    /// The one identity this build writes and accepts. Art is admitted at launch,
    /// before any save opens, and is not part of it.
    fn accepted() -> Identity {
        [("format", FORMAT), ("source", assets::SOURCE)]
            .into_iter()
            .collect()
    }
}
impl Format for Cuba {
    type Payload = Payload;
    const LIMIT: u64 = LIMIT;
    fn identity(&self, _: &Payload) -> Identity {
        Self::accepted()
    }
    fn accepts(&self, identity: &Identity) -> bool {
        *identity == Self::accepted()
    }
    // Marked, so a refusal can tell exact-replay admission from a payload that
    // does not decode; `words` removes the mark.
    fn validate(&self, payload: &Payload) -> Result<(), String> {
        payload
            .replay()
            .map_err(|detail| format!("{REPLAY}{detail}"))
    }
}

/// What the player was doing when a save was refused, for its wording.
#[derive(Clone, Copy, PartialEq)]
enum Action {
    Recover,
    Continue,
}

/// A save this build cannot continue: another build wrote it.
fn another_build(detail: &str) -> anyhow::Error {
    anyhow!(
        "Another build wrote this save ({detail}), and development builds promise no save compatibility. Every file was left unchanged. Start a match under an unused save name."
    )
}

fn words(error: save::Error, action: Action) -> anyhow::Error {
    match error {
        save::Error::Unsupported(Some(recorded)) => another_build(&match recorded.get("format") {
            Some(format) if format != FORMAT => {
                format!("in format {format}; this build reads format {FORMAT}")
            }
            Some(_) if recorded.get("source") != Some(assets::SOURCE) => {
                "for another rules source".to_owned()
            }
            _ => format!("with identity {recorded}"),
        }),
        save::Error::Unsupported(None) => another_build("before the shared save record"),
        save::Error::Malformed(detail) => {
            anyhow::Error::msg(detail).context("Malformed save; original file preserved")
        }
        save::Error::Checksum => anyhow!("Save checksum mismatch; original file preserved"),
        save::Error::TooLarge => anyhow!("Save exceeds its size limit; original file preserved"),
        // A record with the identity accepted and the checksum intact: its recorded
        // decisions do not replay to its position, or its payload does not decode
        // in this build's shape, whether a field of the save or of its position is
        // missing, unknown or of another type; so another build wrote it. On load
        // it is the save; on a write it is the kept `.previous`, or a save another
        // session replaced, and these words do not tell which.
        save::Error::Unreproducible(detail) => match detail.strip_prefix(REPLAY) {
            Some(detail) => another_build(detail),
            None => another_build(&format!("the save is not in this build's shape: {detail}")),
        },
        // Before a write: this build's own candidate failed its check.
        save::Error::Invalid(detail) => match detail.strip_prefix(REPLAY) {
            Some(detail) => anyhow!(detail.to_owned()),
            None => anyhow::Error::msg(detail).context("Malformed save; original file preserved"),
        },
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
        save::Error::Orphaned if action == Action::Recover => {
            anyhow!("Destination changed before creation")
        }
        save::Error::Occupied => anyhow!("Destination changed before creation"),
        save::Error::Orphaned => anyhow!(
            "Main save is missing but recovery history exists; recover explicitly to an unused path"
        ),
        other => anyhow!(other),
    }
}

/// The save session: the match's save on disk and its payload in memory. It keeps
/// the decision list beside the position and adds each accepted decision; the
/// rules state carries no history of its own.
pub struct Session {
    path: PathBuf,
    baseline: Baseline,
    payload: Payload,
}
impl Session {
    /// Opens the save at `path`, recovers `recover_from`'s previous record into it,
    /// or creates a new save seeded once from OS entropy. Native callers invoke this
    /// only after art decoding and render admission.
    pub fn open(path: &Path, recover_from: Option<&Path>) -> Result<Self> {
        if let Some(source) = recover_from {
            let (payload, baseline) =
                save::recover(&Cuba, source, path, |_| Ok::<_, save::Error>(()))
                    .map_err(|error| words(error, Action::Recover))?;
            return Ok(Self::new(path, baseline, payload));
        }
        match save::load(&Cuba, path).map_err(|error| words(error, Action::Continue))? {
            Some((payload, baseline)) => Ok(Self::new(path, baseline, payload)),
            // Draw OS entropy once. Reopening any existing save never reseeds it.
            None => Self::create_seeded(path, unfolded_random::fresh_seed()?),
        }
    }
    /// Creates a new save at `path` whose match opens with `seed`, at 26July's
    /// action choice. A file or recovery history already there is refused. For
    /// tests and verification runs that need a known die roll; the app's launch
    /// never calls it and always draws OS entropy.
    pub fn create_seeded(path: &Path, seed: u64) -> Result<Self> {
        let payload = Payload::opening(seed);
        let mut baseline = Baseline::default();
        save::write(&Cuba, path, &mut baseline, &payload, Write::Create)
            .map_err(|error| words(error, Action::Continue))?;
        Ok(Self::new(path, baseline, payload))
    }
    fn new(path: &Path, baseline: Baseline, payload: Payload) -> Self {
        Self {
            path: path.to_path_buf(),
            baseline,
            payload,
        }
    }
    pub fn game(&self) -> &Game {
        &self.payload.position
    }
    pub fn observe(&self) -> Observation {
        self.payload.position.observe()
    }
    /// Submits a decision through the checked submission and saves it, with its
    /// place in the decision list, before the changed position is published.
    pub fn submit(&mut self, owner: Faction, revision: u64, choice: &str) -> Result<()> {
        let mut candidate = self.payload.clone();
        candidate
            .position
            .submit(owner, revision, choice)
            .map_err(anyhow::Error::msg)?;
        // Accepted, so `choice` is an offered choice's id.
        candidate
            .decisions
            .push(Choice::action_of(choice).to_owned());
        self.persist(&candidate)?;
        self.payload = candidate;
        Ok(())
    }
    pub fn save(&mut self) -> Result<()> {
        self.persist(&self.payload.clone())
    }
    fn persist(&mut self, candidate: &Payload) -> Result<()> {
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
