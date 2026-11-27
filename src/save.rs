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
//! A save holds the seed the match opened with, every decision in order, the
//! position they produce, public proclamations and the human's read state. Admission requires exact
//! decision replay ([ADR 0003](../docs/adr/0003-a-save-opens-by-exact-replay.md))
//! and a valid publication record.
//! The shared record runs that check on load, on each candidate before it is
//! written and on the kept `.previous` before it is rotated.
use crate::{
    Choice, Faction, Game, Observation, assets,
    faction_seats::Stands,
    reports::{Report, Reports},
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
use unfolded_record_replacement::save::{self, Baseline, Format, Identity, Write};
/// About 4 MiB, far above a full game; a save at the limit is refused plainly.
const LIMIT: u64 = 4 * 1024 * 1024;
/// The save's shape. It changes whenever what a save holds changes shape.
const FORMAT: &str = "11";
/// Marks a refusal from exact-replay admission.
const REPLAY: &str = "cuba-replay:";

/// Why a decision or publication did not commit. Its words are the refusal's or
/// the save error's own, unchanged.
#[derive(Debug)]
pub enum Submit {
    /// The checked submission was refused.
    Refused(String),
    /// The submission was allowed, and the save could not be written.
    Save(anyhow::Error),
}

impl std::fmt::Display for Submit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(reason) => f.write_str(reason),
            Self::Save(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Submit {}

/// One public proclamation. A published document spends its faction's allowance
/// for this card, whichever controller holds the faction now.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proclamation {
    pub card: u8,
    pub author: Faction,
    pub text: String,
}

impl Proclamation {
    pub const LIMIT: usize = 400;

    pub(crate) fn spent(published: &[Self], card: u8, author: Faction) -> bool {
        published
            .iter()
            .any(|document| document.card == card && document.author == author)
    }

    fn check_length(text: &str) -> Result<(), String> {
        if text.chars().count() <= Self::LIMIT {
            Ok(())
        } else {
            Err(format!(
                "A proclamation may contain at most {} characters.",
                Self::LIMIT
            ))
        }
    }

    fn already_published(card: u8, author: Faction) -> String {
        format!(
            "{} already published a proclamation on card {card}.",
            author.name()
        )
    }
}

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
    /// Public words in publication order, separate from the replayed decisions.
    proclamations: Vec<Proclamation>,
    /// Publication indices the human has acknowledged or authored.
    read_proclamations: BTreeSet<usize>,
}
impl Payload {
    fn opening(seed: u64) -> Self {
        Self {
            seed,
            decisions: vec![],
            position: Game::opening_with_seed(seed),
            proclamations: vec![],
            read_proclamations: BTreeSet::new(),
        }
    }
    /// Exact replay: each decision, submitted at the then-current match revision
    /// as the owner of the offered choice, must be accepted, and the result must
    /// equal the saved position.
    fn replay(&self) -> Result<(), String> {
        let mut game = Game::opening_with_seed(self.seed);
        let opening_card = game.current_card();
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
        self.validate_proclamations(opening_card..=game.current_card())
    }

    fn validate_proclamations(&self, played: std::ops::RangeInclusive<u8>) -> Result<(), String> {
        if self
            .read_proclamations
            .last()
            .is_some_and(|index| *index >= self.proclamations.len())
        {
            return Err("The human's read record names an unpublished proclamation.".to_owned());
        }
        let mut latest = *played.start();
        for (index, document) in self.proclamations.iter().enumerate() {
            if !played.contains(&document.card) || document.card < latest {
                return Err(format!(
                    "Proclamation {} names a card outside publication order: {}.",
                    index + 1,
                    document.card
                ));
            }
            if Proclamation::spent(&self.proclamations[..index], document.card, document.author) {
                return Err(Proclamation::already_published(
                    document.card,
                    document.author,
                ));
            }
            Proclamation::check_length(&document.text)?;
            latest = document.card;
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
    /// What the decisions did (#207). In no save: an opened one's follow from its own.
    reports: Reports,
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
    /// A seed whose first Kidnap roll is 1, which leaves 26July the fewest Resources: the
    /// native regressions' own (docs/agents/native-tests.md).
    pub const KIDNAP_ROLLS_1: u64 = 19;
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
        let opening = Game::opening_with_seed(payload.seed);
        Self {
            path: path.to_path_buf(),
            baseline,
            reports: Reports::replayed(opening, &payload.decisions),
            payload,
        }
    }
    pub fn game(&self) -> &Game {
        &self.payload.position
    }
    pub fn observe(&self) -> Observation {
        self.payload.position.observe()
    }
    /// Every faction's public proclamation, in publication order.
    pub fn proclamations(&self) -> &[Proclamation] {
        &self.payload.proclamations
    }
    /// Whether the human has already acknowledged or authored this publication.
    pub fn proclamation_is_read(&self, index: usize) -> bool {
        self.payload.read_proclamations.contains(&index)
    }
    /// Acknowledges a publication for the human. Rereading never replaces a save.
    pub fn read_proclamation(&mut self, index: usize) -> Result<()> {
        if index >= self.proclamations().len() {
            return Err(anyhow!("No proclamation at that index."));
        }
        if self.proclamation_is_read(index) {
            return Ok(());
        }
        let mut candidate = self.payload.clone();
        candidate.read_proclamations.insert(index);
        self.persist(&candidate)?;
        self.payload = candidate;
        Ok(())
    }
    /// A report of each action of the match so far, whole, in order (#207).
    pub fn reports(&self) -> Vec<Report> {
        self.reports.all()
    }
    /// The reports of what followed `revision`, the first of which may continue an action
    /// already reported up to there.
    pub fn reports_since(&self, revision: u64) -> Vec<Report> {
        self.reports.since(revision)
    }
    /// Submits a decision through the checked submission and saves it, with its
    /// place in the decision list, before the changed position is published. A
    /// failure says which it was: the rules refused the decision, or the save could
    /// not be written (#174: the table draws each as its own kind of notice).
    pub fn submit(
        &mut self,
        owner: Faction,
        revision: u64,
        choice: &str,
    ) -> std::result::Result<(), Submit> {
        let mut candidate = self.payload.clone();
        candidate
            .position
            .submit(owner, revision, choice)
            .map_err(Submit::Refused)?;
        // Accepted, so `choice` is an offered choice's id.
        candidate
            .decisions
            .push(Choice::action_of(choice).to_owned());
        self.persist(&candidate).map_err(Submit::Save)?;
        self.payload = candidate;
        self.reports.record(&self.observe());
        Ok(())
    }
    /// Publishes on behalf of an author whose controller the table has checked.
    /// The document is visible only after the save succeeds. The game position,
    /// decision revision and reports all continue from the same decision.
    pub fn publish(
        &mut self,
        author: Faction,
        revision: u64,
        card: u8,
        text: &str,
    ) -> std::result::Result<(), Submit> {
        self.publish_with_read_state(author, revision, card, text, false)
    }
    /// Publishes the human controller's own words as already read in the same save.
    pub fn publish_as_human(
        &mut self,
        author: Faction,
        revision: u64,
        card: u8,
        text: &str,
    ) -> std::result::Result<(), Submit> {
        self.publish_with_read_state(author, revision, card, text, true)
    }
    fn publish_with_read_state(
        &mut self,
        author: Faction,
        revision: u64,
        card: u8,
        text: &str,
        read: bool,
    ) -> std::result::Result<(), Submit> {
        let game = self.game();
        if revision != game.revision || card != game.current_card() {
            return Err(Submit::Refused(
                "Stale proclamation view. Use the current revision and card.".into(),
            ));
        }
        if Stands::of(game) != Stands::Open {
            return Err(Submit::Refused(
                "The current card is closed to proclamations.".into(),
            ));
        }
        if Proclamation::spent(self.proclamations(), card, author) {
            return Err(Submit::Refused(Proclamation::already_published(
                card, author,
            )));
        }
        Proclamation::check_length(text).map_err(Submit::Refused)?;
        let mut candidate = self.payload.clone();
        if read {
            candidate
                .read_proclamations
                .insert(candidate.proclamations.len());
        }
        candidate.proclamations.push(Proclamation {
            author,
            card,
            text: text.to_owned(),
        });
        self.persist(&candidate).map_err(Submit::Save)?;
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
