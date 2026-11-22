//! The operation panel in the selected designs: AC-B at the action choice
//! (`choice.rs`), and during Terror the selected EX-A design, "Pinned step tag" (#83's
//! round, recorded on #58; built in #84). EX-A carries AC-B's grammar into execution:
//! a fixed list of steps in AC-B's option rows, each with its state as a pill or a
//! note; hovering or focusing a step opens its preview on the AC-B tag, a click pins
//! it, and the full-width action at the panel's foot commits only the pinned step.
//! After Pass or Operation Only, the panel states what was recorded and what
//! Directorio may do next. Kidnap hands the panel back to the action choices.
//!
//! Layout authority (foundation 4): `art/atlas/atlas-execution.figcraft` owns the
//! structure, placement and sizes. The constants in [`fig`] are that file's 16:9
//! frame coordinates (the ended panels are translated to the 16:9 panel), and
//! `atlas-execution.layout.json` records the same nodes for the native comparison.
//! Every drawn plate is an admitted export: the pills in `execution.sha256`; AC-B's
//! rows, pill, commit and tag plate in `choice.sha256` and `victory.sha256`; HF-A's
//! Cancel and remedy chip in `popover.sha256`; TM-B's giver-coloured primary in
//! `transfer.sha256`. The runtime places them and the text, and dims steps that wait.
//!
//! The game owns every fact. A step's preview is the rules authority's own preview
//! of that choice, or its explanation and in-task remedies; this module only words
//! and places them. Pinning is presentation state. Only the commit, and Kidnap's
//! Cancel, submit a decision, through the same checked seam as every other control.
use super::briefing::{self, Briefing, Entry, entry};
use super::choice::{self, Part, fig as acb, rect, rs, rx, ry, style};
use super::emblems::{self, Prose};
use super::palette::{INK, PAPER, PRIMARY, QUIET};
use super::*;
use crate::{
    ActionChoice, Choice, Observation, Preview, SequenceBox, TERROR_COST, Transfer, resources,
};
use bevy::text::{Justify, LetterSpacing, LineHeight};
use bevy::ui::widget::NodeImageMode;

/// Node coordinates in the FigCraft 16:9 frame, `art/atlas/atlas-execution.figcraft`
/// (the same frame units and panel place as AC-B's, [`acb`]).
pub(super) mod fig {
    /// The section labels: name, words and top.
    pub const SECTIONS: [(&str, &str, f32); 3] = [
        (
            "section:targets",
            "TERROR · 1 RESOURCE PER SPACE · R 3.3.4",
            1013.6,
        ),
        (
            "section:during",
            "WHILE 26JULY EXECUTES · R 1.5.1, 1.4.1",
            1350.,
        ),
        ("section:end", "END THE OPERATION · R 2.3.4", 1589.2),
    ];
    /// Each step's row top and section, in the panel's fixed order
    /// ([`StepId::ALL`](super::StepId::ALL)).
    pub const ROWS: [(f32, usize); 7] = [
        (1058.4, 0),
        (1155.6, 0),
        (1252.8, 0),
        (1394.8, 1),
        (1492., 1),
        (1634., 2),
        (1731.2, 2),
    ];
    /// A step offered now: bold label, left and top inside its row.
    pub const LABEL: [f32; 2] = [40.5, 17.2];
    /// A step not offered now: quiet label.
    pub const QUIET_LABEL: [f32; 2] = [37.8, 18.7];
    /// Kidnap's label starts after its connector: quiet, then bold.
    pub const SA_LABEL_X: [f32; 2] = [86.4, 90.45];
    pub const CONNECTOR: [f32; 4] = [37.8, 41.85, 27., 2.7];
    /// A pill's right edge inside its row; an active row's stroke moves it in.
    pub const PILL_RIGHT: f32 = 1193.4;
    pub const PILL_RIGHT_ACTIVE: f32 = 1189.35;
    /// The pill's text starts this far inside its plate (padding and stroke).
    pub const PILL_PAD: f32 = 21.6;
    /// A note's right edge and top inside its row.
    pub const NOTE: [f32; 2] = [1190.7, 21.2];
    pub const NOTE_SIZE: f32 = 36.45;
    /// Kidnap's Cancel above the commit (HF-A's outlined secondary).
    pub const CANCEL: [f32; 4] = [4750.8, 2235.6, 1220.4, 108.];
    pub const CANCEL_TOP: f32 = 28.;
    pub const CANCEL_SIZE: f32 = 43.2;
    /// The ended panel's first entry, where the step list starts.
    pub const ENDED_TOP: f32 = 1013.6;
    /// A remedy chip in the tag: its height, gap, padding and type.
    pub const CHIP_H: f32 = 65.6;
    pub const CHIP_GAP: f32 = 16.2;
    pub const CHIP_PAD: f32 = 21.6;
    pub const CHIP_SIZE: f32 = 36.45;
    /// The opacity of a step that waits on an open step.
    pub const DIM: f32 = 0.4;
    /// Panel value lines are broken by hand at this many characters: the 1220 frame
    /// unit column holds about 64 at 40.5 (the tag's 1134 hold 58).
    pub const CHARS: usize = 62;
}

/// The panel's steps during Terror, in its fixed order: one table for each step's
/// action, row, name and rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StepId {
    TerrorSantiago,
    TerrorOriente,
    Kidnap,
    Transfer,
    Remove,
    EndTerror,
    BackToChoice,
}
impl StepId {
    pub const ALL: [Self; 7] = [
        Self::TerrorSantiago,
        Self::TerrorOriente,
        Self::Kidnap,
        Self::Transfer,
        Self::Remove,
        Self::EndTerror,
        Self::BackToChoice,
    ];
    /// The step for the authority's action, such as `terror:santiago`.
    pub fn of(action: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.action() == action)
    }
    /// The authority's action the step lists.
    pub fn action(self) -> &'static str {
        match self {
            Self::TerrorSantiago => "terror:santiago",
            Self::TerrorOriente => "terror:oriente",
            Self::Kidnap => "kidnap",
            Self::Transfer => "transfer",
            Self::Remove => "remove",
            Self::EndTerror => "end-terror",
            Self::BackToChoice => "back-to-choice",
        }
    }
    /// A Terror target's space id.
    pub fn space(self) -> Option<&'static str> {
        match self {
            Self::TerrorSantiago => Some("santiago"),
            Self::TerrorOriente => Some("oriente"),
            _ => None,
        }
    }
    /// The row's top and section in the frame.
    pub fn row(self) -> (f32, usize) {
        fig::ROWS[self as usize]
    }
    /// The rule the step's tag cites.
    fn citation(self) -> &'static str {
        match self {
            Self::TerrorSantiago | Self::TerrorOriente => "R 3.3.4",
            Self::Kidnap => "R 4.3.3",
            Self::Transfer => "R 1.5.1",
            Self::Remove => "R 1.4.1",
            Self::EndTerror => "R 2.3.4",
            Self::BackToChoice => "local policy",
        }
    }
    /// The step's name as its row lists it.
    fn label(self, view: &Observation) -> String {
        match self {
            Self::Kidnap => "Kidnap · Santiago · Special Activity".into(),
            Self::Transfer => "Resource transfer".into(),
            Self::Remove => "Remove own pieces".into(),
            Self::EndTerror => "End Terror".into(),
            Self::BackToChoice => "Back to the action choice".into(),
            Self::TerrorSantiago | Self::TerrorOriente => {
                format!(
                    "Terror · {}",
                    space_name(view, self.space().unwrap_or_default())
                )
            }
        }
    }
}

/// What the panel is doing during Terror: listing the steps, or holding one open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Steps,
    /// Kidnap is opened: Roll or Cancel at the foot; the other steps wait.
    Kidnap,
    /// Removal mode: the map's pieces decide; Done at the foot.
    Removal,
    /// The transfer modal, or a Transfer request pending after Wait.
    Transfer,
}
pub(super) fn mode(view: &Observation) -> Mode {
    if view.kidnap_open {
        Mode::Kidnap
    } else if view.removal.is_some() {
        Mode::Removal
    } else if view.transfer.is_some() {
        Mode::Transfer
    } else {
        Mode::Steps
    }
}

/// The step a mode holds open. Its row is active, and its tag opens pinned when the
/// mode starts.
pub(super) fn held(view: &Observation) -> Option<&'static str> {
    if view.finished || view.executing.is_none() {
        return None;
    }
    match mode(view) {
        Mode::Steps => None,
        Mode::Kidnap => Some(StepId::Kidnap.action()),
        Mode::Removal => Some(StepId::Remove.action()),
        Mode::Transfer => Some(StepId::Transfer.action()),
    }
}

/// A step's state pill, in the four EX-A styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Pill {
    /// Selectable and owed, or the open step (red outline).
    Owed,
    /// Blocked, with an in-task remedy (dashed red outline).
    NotYet,
    /// Done: kept as history (ink fill).
    Done,
    /// Not offered: no longer, or until the open step ends (rule outline).
    Unavailable,
}
impl Pill {
    fn plate(self) -> &'static str {
        match self {
            Pill::Owed => "atlas-execution-pill-owed.png",
            Pill::NotYet => "atlas-execution-pill-not-yet.png",
            Pill::Done => "atlas-execution-pill-done.png",
            Pill::Unavailable => "atlas-choice-pill.png",
        }
    }
    fn ink(self) -> Color {
        match self {
            Pill::Owed | Pill::NotYet => PRIMARY,
            Pill::Done => PAPER,
            Pill::Unavailable => QUIET,
        }
    }
}

/// A step's state at its row's right: a pill, or a note (red while it is open).
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Status {
    Pill(Pill, String),
    Note(String, bool),
}

/// One row of the panel.
#[derive(Clone, Debug)]
pub(super) struct Step {
    pub id: StepId,
    pub action: &'static str,
    pub label: String,
    pub status: Status,
    /// Offered and possible now, or the open step: a bold label (on a plate unless
    /// the row is active).
    pub bold: bool,
    /// Pinnable: offered now, or the open step.
    pub interactive: bool,
    /// Waits on the open step.
    pub dim: bool,
    /// The open step.
    pub held: bool,
}

fn listed<'a>(view: &'a Observation, action: &str) -> Option<&'a Choice> {
    view.choices.iter().find(|c| c.action() == action)
}

fn space_name<'a>(view: &'a Observation, id: &'a str) -> &'a str {
    view.spaces
        .iter()
        .find(|s| s.id == id)
        .map_or(id, |s| s.name.as_str())
}

/// What a step that is not offered now waits for, by mode.
fn waits(mode: Mode) -> &'static str {
    match mode {
        Mode::Steps => "UNAVAILABLE",
        Mode::Kidnap => "AFTER ROLL OR CANCEL",
        Mode::Removal => "AFTER DONE",
        Mode::Transfer => "AFTER THE TRANSFER",
    }
}

/// A Terror target that cannot be paid now: the pill names the observed fact that
/// blocks it (R 3.1, 3.3.4). Its tag gives the rules authority's own reason.
fn blocked_target(view: &Observation, id: &str, remedies: bool, mode: Mode) -> Status {
    let m26 = view.resources[Faction::M26.index()];
    let underground = view.spaces.iter().find(|s| s.id == id).is_some_and(|s| {
        s.pieces.iter().any(|p| {
            p.faction == Faction::M26
                && p.kind == Kind::Guerrilla
                && p.state == State::Underground
                && p.count > 0
        })
    });
    let reserved = view.owed_terror.as_deref().is_some_and(|owed| owed != id) && m26 <= 1;
    let short = if !underground {
        "NO UNDERGROUND GUERRILLA".to_owned()
    } else if reserved {
        "1 RESOURCE RESERVED".to_owned()
    } else if m26 == 0 {
        "26JULY HAS 0".to_owned()
    } else {
        return Status::Pill(Pill::Unavailable, waits(mode).into());
    };
    if remedies {
        Status::Pill(Pill::NotYet, format!("NOT YET · {short}"))
    } else {
        Status::Pill(Pill::Unavailable, short)
    }
}

/// Every step of the panel during Terror, in its fixed order, with its state now.
pub(super) fn steps(view: &Observation) -> Vec<Step> {
    let mode = mode(view);
    let open = held(view);
    let owed = view.owed_terror.as_deref();
    StepId::ALL
        .into_iter()
        .map(|id| {
            let action = id.action();
            let choice = listed(view, action);
            let is_held = open == Some(action);
            let note = |words: &str| Status::Note(words.into(), false);
            // A listed step the authority refuses: its pill says whether something in
            // this task can make it available.
            let refused_pill = |c: &Choice| {
                if c.remedies.is_empty() {
                    Status::Pill(Pill::Unavailable, "UNAVAILABLE".into())
                } else {
                    Status::Pill(Pill::NotYet, "NOT YET".into())
                }
            };
            let status = match id {
                StepId::Kidnap => match (&view.kidnap, choice) {
                    (Some(roll), _) => Status::Pill(Pill::Done, format!("ROLLED {}", roll.die)),
                    _ if is_held => Status::Pill(Pill::Owed, "OPEN · NOTHING ROLLED".into()),
                    (None, Some(c)) if c.supported => note("opens its stakes"),
                    (None, Some(c)) => refused_pill(c),
                    (None, None) => Status::Pill(Pill::Unavailable, waits(mode).into()),
                },
                StepId::Transfer => match &view.transfer {
                    Some(Transfer::Authorizing { giver, .. }) => {
                        Status::Note(format!("pending · {} decides", giver.name()), true)
                    }
                    Some(Transfer::Selecting) => Status::Note("open".into(), true),
                    None => note("opens the transfer"),
                },
                StepId::Remove if is_held => Status::Note("removal mode · on".into(), true),
                StepId::Remove => note("removal mode on the map"),
                StepId::EndTerror => match choice {
                    Some(c) if c.supported && mode == Mode::Transfer => {
                        note("withdraws the request")
                    }
                    Some(c) if c.supported => match c.preview.as_ref().and_then(|p| p.records) {
                        Some(record) => note(&format!("records {}", record.name())),
                        None => note("ends the Operation"),
                    },
                    _ if owed.is_some() => {
                        Status::Pill(Pill::NotYet, "NOT YET · SANTIAGO OWED".into())
                    }
                    Some(c) => refused_pill(c),
                    None => Status::Pill(Pill::Unavailable, waits(mode).into()),
                },
                StepId::BackToChoice => match choice {
                    Some(c) if c.supported && mode == Mode::Transfer => {
                        note("withdraws the request")
                    }
                    Some(c) if c.supported => note("until the first effect"),
                    Some(c) => refused_pill(c),
                    None if view.effected => {
                        Status::Pill(Pill::Unavailable, "NO LONGER OFFERED".into())
                    }
                    None => Status::Pill(Pill::Unavailable, waits(mode).into()),
                },
                StepId::TerrorSantiago | StepId::TerrorOriente => {
                    let space = id.space().unwrap_or_default();
                    let resolved = view.resolved.iter().find(|r| r.space == space);
                    let owed_pill = || {
                        Status::Pill(
                            Pill::Owed,
                            format!("OWED · {}", resources(TERROR_COST).to_uppercase()),
                        )
                    };
                    match choice {
                        _ if resolved.is_some() => Status::Pill(
                            Pill::Done,
                            format!("DONE · PAID {}", resolved.map_or(0, |r| r.paid)),
                        ),
                        Some(c) if c.supported && owed == Some(space) => owed_pill(),
                        Some(c) if c.supported => note(&format!("pay {}", resources(TERROR_COST))),
                        Some(c) => blocked_target(view, space, !c.remedies.is_empty(), mode),
                        None if owed == Some(space) => owed_pill(),
                        None => blocked_target(view, space, true, mode),
                    }
                }
            };
            // Resource transfer and removal are offered whenever no step is open, so
            // their plates stay while they wait.
            let always = matches!(id, StepId::Transfer | StepId::Remove);
            let bold = choice.is_some_and(|c| c.supported) || is_held || always;
            let interactive = choice.is_some() || is_held;
            let dim = mode != Mode::Steps
                && !interactive
                && !(mode == Mode::Kidnap && (id.space().is_some() || id == StepId::Kidnap));
            Step {
                id,
                action,
                label: id.label(view),
                status,
                bold,
                interactive,
                dim,
                held: is_held,
            }
        })
        .collect()
}

/// A step row's rectangle in frame units, for the tag's active outline.
pub(super) fn row(action: &str) -> Option<[f32; 4]> {
    StepId::of(action).map(|id| [acb::ROW_X, id.row().0, acb::ROW_W, acb::ROW_H])
}

/// The commit at the panel's foot during Terror: what it reads and does.
pub(super) struct Commit {
    pub label: String,
    pub action: Action,
    pub name: String,
    pub enabled: bool,
    pub plate: String,
}

/// What a disabled step's activation reports: its reason and in-task remedies.
fn refusal(choice: &Choice) -> String {
    if choice.remedies.is_empty() {
        choice.explanation.clone()
    } else {
        format!(
            "{} In this task: {}.",
            choice.explanation,
            choice.remedies.join(" · ")
        )
    }
}

fn enabled(choice: &Choice, label: String, view: &Observation) -> Commit {
    Commit {
        label,
        action: Action::Commit(choice.owner, view.revision),
        name: choice.id.clone(),
        enabled: true,
        plate: "atlas-choice-commit.png".into(),
    }
}

fn disabled(label: impl Into<String>, reason: impl Into<String>) -> Commit {
    Commit {
        label: label.into(),
        action: Action::Unsupported(reason.into()),
        name: "execution-commit".into(),
        enabled: false,
        plate: "atlas-choice-commit-disabled.png".into(),
    }
}

/// The disabled commit for a pinned step the authority refuses: its name, and the
/// authority's reason and in-task remedies.
fn refused(view: &Observation, id: StepId, choice: &Choice) -> Commit {
    let what = match id {
        StepId::TerrorSantiago | StepId::TerrorOriente => format!(
            "Terror in {}",
            space_name(view, id.space().unwrap_or_default())
        ),
        StepId::Kidnap => "Kidnap".into(),
        _ => id.label(view),
    };
    disabled(format!("{what} is not available yet"), refusal(choice))
}

/// The commit for the open step, or for the pinned step when none is open.
pub(super) fn commit(view: &Observation, pinned: Option<&str>) -> Commit {
    let mode = mode(view);
    let fixed = |action: &str| {
        listed(view, action).map(|c| {
            if c.supported {
                enabled(c, c.label.clone(), view)
            } else {
                disabled(format!("{} is not available yet", c.label), refusal(c))
            }
        })
    };
    let none = || {
        disabled(
            "Click a step to choose it",
            "Pin a step first: click it, or focus it with Tab and press Enter. Its preview stays open and this button commits it.",
        )
    };
    match mode {
        Mode::Kidnap => fixed("kidnap:roll").unwrap_or_else(none),
        Mode::Removal => fixed("done-removal").unwrap_or_else(none),
        Mode::Transfer => {
            // Withdrawing the request ends or leaves the Operation: only End Terror
            // and Back are steps while it is pending.
            if let Some((id, c)) = pinned
                .and_then(StepId::of)
                .filter(|id| matches!(id, StepId::EndTerror | StepId::BackToChoice))
                .and_then(|id| Some((id, listed(view, id.action())?)))
            {
                return if c.supported {
                    enabled(c, c.label.clone(), view)
                } else {
                    refused(view, id, c)
                };
            }
            match (listed(view, "reopen"), &view.transfer) {
                // TM-B's pending tab already carries the choice by its id, so this
                // commit is the component `execution-commit` and submits it by id.
                (Some(c), Some(Transfer::Authorizing { giver, .. })) => Commit {
                    label: format!("Open the request · {} decides", giver.name()),
                    action: Action::Submit(c.owner, view.revision, c.id.clone()),
                    name: "execution-commit".into(),
                    enabled: true,
                    plate: format!("atlas-transfer-primary-{}.png", super::slug(*giver)),
                },
                _ => disabled(
                    "The Resource transfer is open",
                    "The transfer decides first: give, ask, authorize, decline, Wait or Cancel.",
                ),
            }
        }
        Mode::Steps => {
            let Some((id, c)) = pinned
                .and_then(StepId::of)
                .and_then(|id| Some((id, listed(view, id.action())?)))
            else {
                return none();
            };
            if !c.supported {
                return refused(view, id, c);
            }
            let words = match id {
                StepId::TerrorSantiago | StepId::TerrorOriente => format!(
                    "Terror in {} · pay {}",
                    space_name(view, id.space().unwrap_or_default()),
                    resources(TERROR_COST)
                ),
                StepId::Kidnap => "Open Kidnap · nothing rolled yet".into(),
                StepId::Transfer => "Open the Resource transfer".into(),
                StepId::Remove => "Remove own pieces · removal mode".into(),
                StepId::EndTerror => match c.preview.as_ref().and_then(|p| p.records) {
                    Some(SequenceBox::OperationWithSpecialActivity) => {
                        "End Terror · Operation with Special Activity".into()
                    }
                    Some(record) => format!("End Terror · records {}", record.name()),
                    None => "End Terror".into(),
                },
                StepId::BackToChoice => id.label(view),
            };
            enabled(c, words, view)
        }
    }
}

/// Kidnap's Cancel above the commit while Kidnap is open.
fn cancel(view: &Observation) -> Option<&Choice> {
    (mode(view) == Mode::Kidnap)
        .then(|| listed(view, "kidnap:cancel"))
        .flatten()
}

/// The controls Tab moves through during Terror, in order: the steps that can be
/// pinned, Kidnap's Cancel, then the commit. Each with what Enter does.
pub(super) fn focusable(view: &Observation, pinned: Option<&str>) -> Vec<(String, Action)> {
    let mut controls: Vec<(String, Action)> = steps(view)
        .into_iter()
        .filter(|s| s.interactive)
        .map(|s| (format!("step:{}", s.action), Action::Pin(s.action.into())))
        .collect();
    if let Some(c) = cancel(view) {
        controls.push((c.id.clone(), Action::Commit(c.owner, view.revision)));
    }
    let commit = commit(view, pinned);
    controls.push((commit.name, commit.action));
    controls
}

/// Applies a dimmed step's opacity to a drawn plate or text.
fn fade(commands: &mut Commands, entity: Entity, color: Color, dim: bool, text: bool) {
    let color = if dim {
        color.with_alpha(fig::DIM)
    } else {
        color
    };
    if text {
        commands.entity(entity).insert(TextColor(color));
    } else {
        commands
            .entity(entity)
            .entry::<ImageNode>()
            .and_modify(move |mut image| image.color = color);
    }
}

/// A text in frame units: content, left, top, width and size.
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    art: &Art,
    content: impl Into<Prose>,
    at: [f32; 4],
    ink: Color,
    bold: bool,
    spacing: f32,
    s: f32,
    o: Vec2,
    z: i32,
) -> Entity {
    let [x, y, w, size] = at;
    let entity = text(
        commands,
        art,
        content,
        rx(x),
        ry(y),
        rs(w),
        rs(size),
        s,
        o,
        z,
    );
    commands.entity(entity).insert(TextColor(ink));
    style(
        commands,
        entity,
        art,
        bold,
        rs(size) * s,
        rs(spacing) * s,
        None,
    );
    entity
}

/// A plate drawn behind a node's content, over the node's whole box: the node hugs
/// its words, and the plate is nine-sliced so its corners keep their reference size
/// (`corner` reference pixels; the exports are 3x) and its dashes their length. A
/// pill's corner stops just short of half its height (9 of 20 reference pixels), so
/// its straight middle rows exist in the source and its ends stay whole.
pub(super) fn backdrop(art: &Art, key: &str, corner: f32, s: f32) -> impl Bundle {
    (
        ImageNode::new(art.images[key].clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(corner * 3.),
            max_corner_scale: s / 3.,
            sides_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
            center_scale_mode: SliceScaleMode::Tile { stretch_value: 1. },
        })),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.),
            top: px(0.),
            right: px(0.),
            bottom: px(0.),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// A pill hugging its words, its right edge at `right` (frame units): the plate is
/// nine-sliced so its rounded ends keep their reference size.
#[allow(clippy::too_many_arguments)]
fn pill(
    commands: &mut Commands,
    art: &Art,
    kind: Pill,
    content: &str,
    [right, top]: [f32; 2],
    name: &str,
    dim: bool,
    s: f32,
    o: Vec2,
) {
    let alpha = if dim { fig::DIM } else { 1. };
    // A zero-width anchor at the right edge: the pill overflows to its left.
    let anchor = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(o.x + rx(right) * s),
                top: px(o.y + ry(top) * s),
                width: px(0.),
                height: px(rs(acb::PILL_H) * s),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            GlobalZIndex(5),
            Pickable::IGNORE,
        ))
        .id();
    let plate = commands
        .spawn((
            Node {
                height: px(rs(acb::PILL_H) * s),
                padding: UiRect::horizontal(px(rs(fig::PILL_PAD) * s)),
                align_items: AlignItems::Center,
                flex_shrink: 0.,
                ..default()
            },
            Part(name.into()),
            Pickable::IGNORE,
        ))
        .id();
    let backdrop = commands.spawn(backdrop(art, kind.plate(), 9., s)).id();
    fade(commands, backdrop, Color::WHITE, dim, false);
    commands.entity(plate).add_child(backdrop);
    let label = commands
        .spawn((
            Text::new(content),
            emblems::Plain(content.into()),
            TextFont {
                font: art.bold.clone().into(),
                font_size: (rs(acb::PILL_SIZE) * s).into(),
                ..default()
            },
            TextColor(kind.ink().with_alpha(alpha)),
            LetterSpacing::Px(rs(acb::PILL_SPACING) * s),
            Part(format!("{name}-label")),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(plate).add_child(label);
    commands.entity(anchor).add_child(plate);
}

/// The panel's header: the 26July emblem, the heading and two subtitle lines.
fn header(commands: &mut Commands, art: &Art, heading: &str, subtitle: &str, s: f32, o: Vec2) {
    let [x, y, w, h] = acb::EMBLEM;
    let emblem = picture(
        commands,
        art,
        &emblems::image(Faction::M26),
        rect(x, y, w, h),
        s,
        o,
        3,
    );
    commands
        .entity(emblem)
        .insert(Part("execution-heading-emblem".into()));
    let [x, y, w] = acb::HEADING;
    let title = words(
        commands,
        art,
        Prose::named(heading),
        [x, y, w, acb::HEADING_SIZE],
        INK,
        false,
        0.,
        s,
        o,
        5,
    );
    commands
        .entity(title)
        .insert(Part("execution-heading".into()));
    let [x, y, w] = acb::SUBTITLE;
    let sub = words(
        commands,
        art,
        Prose::named(subtitle),
        [x, y, w, acb::SUBTITLE_SIZE],
        QUIET,
        false,
        0.,
        s,
        o,
        5,
    );
    style(
        commands,
        sub,
        art,
        false,
        0.,
        0.,
        Some(rs(acb::SUBTITLE_LINE) * s),
    );
    commands
        .entity(sub)
        .insert(Part("execution-subtitle".into()));
}

/// The operation panel: AC-B at the action choice, EX-A during Terror, and the ended
/// panel once the Operation is recorded. `offset` is the right-anchored letterbox
/// offset, so wide windows keep the panel at the right edge.
pub(super) fn panel(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    s: f32,
    o: Vec2,
) {
    picture(commands, art, "atlas-panel.png", PANEL, s, o, 2);
    if view.eligibility.directorio_handoff() {
        return choice::panel(commands, art, view, ui, s, o);
    }
    if view.finished {
        return ended(commands, art, view, s, o);
    }
    match view.action {
        ActionChoice::Open => choice::panel(commands, art, view, ui, s, o),
        ActionChoice::Terror => terror(commands, art, view, ui, s, o),
        ActionChoice::Pass => ended(commands, art, view, s, o),
    }
}

/// The EX-A panel while 26July executes Terror.
fn terror(commands: &mut Commands, art: &Art, view: &Observation, ui: &Interface, s: f32, o: Vec2) {
    choice::ribbon(commands, art, s, o);
    let m26 = view.resources[Faction::M26.index()];
    let mode = mode(view);
    let (heading, subtitle) = match mode {
        Mode::Kidnap => (
            "26July · Kidnap opened".to_owned(),
            "Kidnap (Special Activity, R 4.1, 4.3.3). Nothing is rolled yet.\nRoll or Cancel first; the other steps wait".to_owned(),
        ),
        Mode::Removal => (
            "26July · Voluntary removal".to_owned(),
            format!(
                "Resource transfers open: 26July is executing Terror (R 1.5.1)\n26July has {} · removal mode stays on until Done",
                resources(m26)
            ),
        ),
        Mode::Steps | Mode::Transfer => (
            "26July · executing Terror".to_owned(),
            format!(
                "Resource transfers open: 26July is executing Terror (R 1.5.1)\n26July has {} · hover a step for its preview, click to pin it",
                resources(m26)
            ),
        ),
    };
    header(commands, art, &heading, &subtitle, s, o);
    let steps = steps(view);
    let pinned = ui.choice.pinned.as_deref();
    for (i, (name, caps, top)) in fig::SECTIONS.iter().enumerate() {
        let dim = steps
            .iter()
            .filter(|step| step.id.row().1 == i)
            .all(|step| step.dim);
        let label = words(
            commands,
            art,
            Prose::named(*caps),
            [acb::ROW_X, *top, acb::ROW_W, acb::CAPS_SIZE],
            QUIET,
            true,
            acb::CAPS_SPACING,
            s,
            o,
            5,
        );
        fade(commands, label, QUIET, dim, true);
        commands.entity(label).insert(Part((*name).into()));
    }
    for step in &steps {
        let top = step.id.row().0;
        let name = format!("step:{}", step.action);
        let area = rect(acb::ROW_X, top, acb::ROW_W, acb::ROW_H);
        // The open step is outlined; an offered step has the paper plate; the others
        // only their words.
        let node = if step.held {
            picture(commands, art, "atlas-choice-row-active.png", area, s, o, 4)
        } else if step.bold {
            let plate = picture(commands, art, "atlas-choice-row.png", area, s, o, 3);
            fade(commands, plate, Color::WHITE, step.dim, false);
            plate
        } else {
            commands
                .spawn((
                    UiRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(o.x + area[0] * s),
                        top: px(o.y + area[1] * s),
                        width: px(area[2] * s),
                        height: px(area[3] * s),
                        ..default()
                    },
                    GlobalZIndex(3),
                    Pickable::IGNORE,
                ))
                .id()
        };
        commands.entity(node).insert(Part(name.clone()));
        if step.interactive {
            commands
                .entity(node)
                .remove::<Pickable>()
                .insert(Control {
                    name: name.clone(),
                    choice: false,
                    action: Action::Pin(step.action.into()),
                })
                .observe(super::click);
        }
        let kidnap = step.id == StepId::Kidnap;
        let (lx, ly, size) = match (kidnap, step.bold) {
            (true, b) => (
                fig::SA_LABEL_X[usize::from(b)],
                if b {
                    fig::LABEL[1]
                } else {
                    fig::QUIET_LABEL[1]
                },
                if b { acb::LABEL_SIZE } else { acb::QUIET_SIZE },
            ),
            (false, true) => (fig::LABEL[0], fig::LABEL[1], acb::LABEL_SIZE),
            (false, false) => (fig::QUIET_LABEL[0], fig::QUIET_LABEL[1], acb::QUIET_SIZE),
        };
        if kidnap {
            let [cx, cy, cw, ch] = fig::CONNECTOR;
            let line = picture(
                commands,
                art,
                "atlas-briefing-rule.png",
                rect(acb::ROW_X + cx, top + cy, cw, ch),
                s,
                o,
                5,
            );
            fade(commands, line, Color::WHITE, step.dim, false);
            commands
                .entity(line)
                .insert(Part(format!("{name}-connector")));
        }
        let ink = if step.bold { INK } else { QUIET };
        let label = words(
            commands,
            art,
            Prose::named(&step.label),
            [acb::ROW_X + lx, top + ly, acb::ROW_W - lx, size],
            ink,
            step.bold,
            0.,
            s,
            o,
            5,
        );
        fade(commands, label, ink, step.dim, true);
        commands.entity(label).insert(Part(format!("{name}-label")));
        match &step.status {
            Status::Pill(kind, content) => {
                let active = step.held || pinned == Some(step.action);
                let right = acb::ROW_X
                    + if active {
                        fig::PILL_RIGHT_ACTIVE
                    } else {
                        fig::PILL_RIGHT
                    };
                pill(
                    commands,
                    art,
                    *kind,
                    content,
                    [right, top + acb::PILL_TOP],
                    &format!("{name}-pill"),
                    step.dim,
                    s,
                    o,
                );
            }
            Status::Note(content, red) => {
                let ink = if *red { PRIMARY } else { QUIET };
                let note = text(
                    commands,
                    art,
                    content.clone(),
                    rx(acb::ROW_X),
                    ry(top + fig::NOTE[1]),
                    rs(fig::NOTE[0]),
                    rs(fig::NOTE_SIZE),
                    s,
                    o,
                    5,
                );
                commands
                    .entity(note)
                    .insert(TextLayout::justify(Justify::Right));
                fade(commands, note, ink, step.dim, true);
                commands.entity(note).insert(Part(format!("{name}-note")));
            }
        }
    }
    if let Some(c) = cancel(view) {
        let [x, y, w, h] = fig::CANCEL;
        let plate = commands
            .spawn((
                UiRoot,
                ImageNode::new(art.images["atlas-popover-cancel.png"].clone()).with_mode(
                    NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect::all(18.),
                        max_corner_scale: s / 3.,
                        ..default()
                    }),
                ),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(o.x + rx(x) * s),
                    top: px(o.y + ry(y) * s),
                    width: px(rs(w) * s),
                    height: px(rs(h) * s),
                    ..default()
                },
                GlobalZIndex(3),
                Control {
                    name: c.id.clone(),
                    choice: true,
                    action: Action::Commit(c.owner, view.revision),
                },
                Part("execution-cancel".into()),
            ))
            .observe(super::click)
            .id();
        let _ = plate;
        let label = words(
            commands,
            art,
            c.label.clone(),
            [x, y + fig::CANCEL_TOP, w, fig::CANCEL_SIZE],
            INK,
            false,
            0.,
            s,
            o,
            5,
        );
        commands.entity(label).insert((
            TextLayout::justify(Justify::Center),
            Part("execution-cancel-label".into()),
        ));
    }
    let commit = commit(view, pinned);
    // 26July keeps its name on the red commit (#60's rule, accepted in #68); a giver's
    // colour keeps its name too, so its emblem does not vanish on its own colour.
    let content = if commit.plate.starts_with("atlas-transfer-primary-") {
        Prose::named(commit.label.clone())
    } else {
        Prose::from(commit.label.clone()).on_red()
    };
    choice::primary_with(
        commands,
        art,
        Control {
            name: commit.name,
            // A commit named apart from its choice (`Submit`) is a component.
            choice: commit.enabled && matches!(commit.action, Action::Commit(..)),
            action: commit.action,
        },
        content,
        commit.enabled,
        &commit.plate,
        "execution-commit",
        s,
        o,
    );
}

fn event_heading(faction: Faction) -> String {
    format!("EVENT FOR {}", faction.name().to_uppercase())
}

/// The ended panel's entries: what was recorded, Eligibility, and what the next
/// faction may do (shown, not executed). Every fact is the authority's: the box
/// 26July's cylinder occupies, its Eligibility for the next card, the next faction
/// with its role and options, the Resources the recording step moved, the Terror
/// resolved and what it paid. `None` until the action is recorded.
pub(super) fn ending(view: &Observation) -> Option<(String, String, Vec<Entry>)> {
    let e = &view.eligibility;
    let record = e.m26_box?;
    let shown = "Shown, not executed: the card and Eligibility do not advance";
    let following = view.next.as_ref();
    let next = entry(
        "NEXT · SHOWN, NOT EXECUTED",
        following.map_or_else(
            || "The next faction is not recorded yet.".to_owned(),
            |n| {
                format!(
                    "{}, {}, may: {}",
                    n.faction.name(),
                    n.role,
                    n.options.join(" · ")
                )
            },
        ),
    );
    let subtitle = format!("{} recorded ({})\n{shown}", record.name(), record.rule());
    let eligibility = match e.m26_next_card {
        Some(true) => format!("26July stays Eligible ({}).", record.rule()),
        Some(false) => "26July becomes Ineligible after this card (R 2.3.7).".to_owned(),
        None => "26July's Eligibility for the next card is not recorded yet.".to_owned(),
    };
    let moved = view
        .recorded_resources
        .iter()
        .map(|r| format!("{} Resources: {} → {}", r.faction.name(), r.from, r.to));
    if record == SequenceBox::Pass {
        let result: Vec<String> = moved
            .chain(["No Operation: no piece, Terror or support changed.".to_owned()])
            .collect();
        return Some((
            "26July · passed".into(),
            subtitle,
            vec![
                entry("RESULT", result.join("\n")),
                entry("ELIGIBILITY", eligibility),
                next,
            ],
        ));
    }
    let mut result = vec![if view.resolved.is_empty() {
        "No Terror resolved: no effect, no Pass income.".to_owned()
    } else {
        let places: Vec<&str> = view
            .resolved
            .iter()
            .map(|r| space_name(view, &r.space))
            .collect();
        format!(
            "Terror resolved: {} · paid {}",
            places.join(", "),
            resources(view.resolved.iter().map(|r| r.paid).sum())
        )
    }];
    if let Some(roll) = &view.kidnap {
        result.push(format!(
            "Kidnap rolled {} · Government paid {} to 26July",
            roll.die,
            resources(roll.proceeds)
        ));
    }
    result.extend(moved);
    // Card 17's Event is open to the next faction exactly when the authority lists it
    // among that faction's options.
    let event = match following {
        Some(n) if n.options.iter().any(|o| o == "Event") => entry(
            event_heading(n.faction),
            format!(
                "After a Special Activity, card 17's Event is open to {}, either text (R 2.3.4, 5.2).",
                n.faction.name()
            ),
        ),
        _ => entry(
            "NO EVENT",
            format!(
                "No Event for {}: it follows only an Operation with Special Activity (R 2.3.4).",
                following.map_or("the next faction", |n| n.faction.name())
            ),
        ),
    };
    Some((
        "26July · Terror ended".into(),
        subtitle,
        vec![
            entry("RESULT", result.join("\n")),
            entry("ELIGIBILITY", eligibility),
            next,
            event,
        ],
    ))
}

/// The ended panel: the header, then each entry's caps label and value lines.
fn ended(commands: &mut Commands, art: &Art, view: &Observation, s: f32, o: Vec2) {
    let Some((heading, subtitle, entries)) = ending(view) else {
        return;
    };
    header(commands, art, &heading, &subtitle, s, o);
    let mut y = fig::ENDED_TOP;
    for e in entries {
        let label = words(
            commands,
            art,
            Prose::named(&e.label),
            [acb::ROW_X, y, acb::ROW_W, acb::CAPS_SIZE],
            QUIET,
            true,
            acb::CAPS_SPACING,
            s,
            o,
            5,
        );
        commands
            .entity(label)
            .insert(Part(format!("ended-label:{}", e.label)));
        let lines = briefing::lines(&e.text, fig::CHARS);
        let value = words(
            commands,
            art,
            lines.join("\n"),
            [
                acb::ROW_X,
                y + acb::ENTRY_VALUE_TOP,
                acb::ROW_W,
                acb::VALUE_TEXT_SIZE,
            ],
            INK,
            false,
            0.,
            s,
            o,
            5,
        );
        commands.entity(value).insert((
            LineHeight::Px(rs(acb::VALUE_LINE) * s),
            Part(format!("ended-value:{}", e.label)),
        ));
        y += acb::ENTRY_VALUE_TOP + acb::VALUE_LINE * lines.len() as f32 + acb::GAP;
    }
}

/// An explanation without its leading rule reference, which the tag's title cites.
fn reason(explanation: &str) -> String {
    let words = explanation.trim();
    match words.strip_prefix("R ") {
        Some(rest) => rest
            .split_once(". ")
            .filter(|(num, _)| num.chars().all(|c| c.is_ascii_digit() || c == '.'))
            .map_or_else(|| words.to_owned(), |(_, rest)| rest.to_owned()),
        None => words.to_owned(),
    }
}

/// A preview's lines under their labels: the authority's computed changes, then its
/// notes. Notes that cite a rule's procedure become the tag's rules line.
fn previewed(view: &Observation, action: &str, preview: &Preview) -> (Vec<Entry>, Vec<String>) {
    let mut entries = vec![];
    let mut rules = vec![];
    let name = |id: &str| space_name(view, id).to_owned();
    if let Some(stakes) = &preview.kidnap {
        entries.push(entry(
            "TARGET",
            format!(
                "{} in {}, which has {}",
                stakes.target.name(),
                name(&stakes.space),
                resources(stakes.government)
            ),
        ));
        entries.push(entry(
            "POSSIBLE PROCEEDS",
            format!(
                "{} · guaranteed minimum {}",
                briefing::proceeds(stakes),
                stakes.minimum
            ),
        ));
    }
    let (own, others): (Vec<_>, Vec<_>) = preview
        .resources
        .iter()
        .partition(|r| r.faction == Faction::M26);
    for r in own {
        entries.push(entry(
            "COST",
            format!("26July Resources: {} → {}", r.from, r.to),
        ));
    }
    if !others.is_empty() {
        entries.push(entry(
            "RESOURCES",
            others
                .iter()
                .map(|r| format!("{} Resources: {} → {}", r.faction.name(), r.from, r.to))
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    let mut map = vec![];
    let piece = |kind: Kind, state: State| match (kind, state) {
        (Kind::Guerrilla, State::Underground) => "Underground Guerrilla".to_owned(),
        (Kind::Guerrilla, State::Active) => "Active Guerrilla".to_owned(),
        (kind, _) => format!("{kind:?}"),
    };
    for f in &preview.flips {
        map.push(format!(
            "{}: flip {} {} to {}",
            name(&f.space),
            f.count,
            piece(f.kind, f.from),
            piece(f.kind, f.to).replace(" Guerrilla", "")
        ));
    }
    for r in &preview.removed {
        map.push(format!(
            "{}: {} {} {} returns to Available",
            name(&r.space),
            r.count,
            r.faction.name(),
            piece(r.kind, r.state)
        ));
    }
    for space in &preview.terror {
        map.push(format!("{}: Terror marker placed", name(space)));
    }
    for shift in &preview.support {
        map.push(format!("{}: {}", name(&shift.space), shift.label));
    }
    let control =
        |f: Option<Faction>| f.map_or("uncontrolled".into(), |f| format!("{} Control", f.name()));
    for c in &preview.control {
        map.push(format!(
            "{}: {} → {}",
            name(&c.space),
            control(c.from),
            control(c.to)
        ));
    }
    if !map.is_empty() {
        entries.push(entry("ON THE MAP", map.join("\n")));
    }
    if !preview.standings.is_empty() {
        entries.push(entry(
            "STANDING",
            preview
                .standings
                .iter()
                .map(|s| {
                    format!(
                        "{} victory standing {} → {} (margin {} → {})",
                        s.faction.name(),
                        s.score_from,
                        s.score_to,
                        s.margin_from,
                        s.margin_to
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }
    if let Some(records) = preview.records {
        entries.push(entry(
            "RECORDS",
            format!("{} ({})", records.name(), records.rule()),
        ));
    }
    if let Some(next) = &preview.next {
        entries.push(entry(
            "NEXT",
            format!(
                "{}, {}, may: {}",
                next.faction.name(),
                next.role,
                next.options.join(" · ")
            ),
        ));
    }
    // Kidnap's notes already say what its roll makes unavailable.
    if preview.kidnap.is_none() && !preview.unavailable.is_empty() {
        entries.push(entry(
            "THEN UNAVAILABLE",
            preview
                .unavailable
                .iter()
                .map(|u| u.label.as_str())
                .collect::<Vec<_>>()
                .join(" · "),
        ));
    }
    if action.starts_with("terror:")
        && let Some(owed) = &view.owed_terror
    {
        entries.push(entry(
            "OWED",
            format!(
                "{} Terror owed (accepted commitment policy): keep 1 Resource and 1 Underground Guerrilla there.",
                name(owed).split(' ').next().unwrap_or_default()
            ),
        ));
    }
    for note in &preview.notes {
        let label = if note.starts_with("R ") {
            rules.push(note.clone());
            continue;
        } else if note.starts_with("No Event") {
            "NO EVENT"
        } else if note.contains("card 17's Event") {
            let faction = preview
                .next
                .as_ref()
                .map_or(view.eligibility.second, |n| n.faction);
            entries.push(entry(event_heading(faction), note.clone()));
            continue;
        } else if note.starts_with("No Terror resolved") {
            "ZERO SPACES"
        } else if note.starts_with("Rolling before") {
            "OWES SANTIAGO'S TERROR"
        } else if note.contains("Ineligible after this card") {
            "26JULY"
        } else if note.contains("withdrawn") {
            "PENDING REQUEST"
        } else {
            rules.push(note.clone());
            continue;
        };
        entries.push(entry(label, note.clone()));
    }
    (entries, rules)
}

/// A step's tag: the rules authority's preview of the step, or its explanation and
/// in-task remedies when it cannot be taken now. An open step's tag says what the
/// open step does and how it ends.
pub(super) fn tag(view: &Observation, action: &str) -> Option<Briefing> {
    if view.finished || view.action != ActionChoice::Terror {
        return None;
    }
    let id = StepId::of(action)?;
    let mode = mode(view);
    let held = held(view) == Some(action);
    let title = id.label(view);
    let briefing = |title: String, entries: Vec<Entry>, rules: String| Briefing {
        option: action.into(),
        title,
        rule: id.citation(),
        entries,
        rules,
    };
    match (mode, held) {
        (Mode::Kidnap, true) => {
            let roll = listed(view, "kidnap:roll")?;
            let Some(preview) = &roll.preview else {
                return Some(briefing(
                    "Kidnap · Santiago de Cuba".into(),
                    vec![entry("NOT YET", reason(&roll.explanation))],
                    String::new(),
                ));
            };
            let (entries, rules) = previewed(view, "kidnap", preview);
            return Some(briefing(
                "Kidnap · Santiago de Cuba".into(),
                entries,
                rules.join(" "),
            ));
        }
        (Mode::Removal, true) => {
            let mut entries = vec![entry(
                "ON THE MAP",
                "Each 26July piece on the map carries a mark: a red ring where it may be removed. Click a ringed piece to see what removing it changes, choose how many from a stack, then Remove or Cancel.",
            )];
            if view.owed_terror.is_some() {
                entries.push(entry(
                    "OWED",
                    "Santiago Terror is owed: its last Underground Guerrilla carries the ink lock and stays (accepted commitment policy, local).",
                ));
            }
            if let Some(done) = listed(view, "done-removal") {
                entries.push(entry("DONE", done.explanation.clone()));
            }
            return Some(briefing(
                "Voluntary removal".into(),
                entries,
                "Removal stays open for more pieces until Done. Earlier Terror markers, support shifts and Kidnap results stay (R 1.4.1).".into(),
            ));
        }
        (Mode::Transfer, true) => {
            return Some(match &view.transfer {
                Some(Transfer::Authorizing {
                    giver,
                    amount,
                    waited,
                    ..
                }) => {
                    let g = giver.name();
                    // Only a step the authority supports withdraws the request.
                    let offered = |id: StepId| {
                        listed(view, id.action())
                            .filter(|c| c.supported)
                            .map(|_| id.label(view))
                    };
                    let end = listed(view, StepId::EndTerror.action());
                    let withdraw = match (offered(StepId::BackToChoice), offered(StepId::EndTerror))
                    {
                        (Some(back), Some(end)) => format!(
                            "26July may withdraw the request: {back} or {end} withdraws it, and nothing has moved."
                        ),
                        (Some(one), None) | (None, Some(one)) => format!(
                            "26July may withdraw the request: {one} withdraws it, and nothing has moved."
                        ),
                        (None, None) => match end.filter(|c| !c.supported) {
                            // The authority's remedy carries its own timing.
                            Some(c) if !c.remedies.is_empty() => format!(
                                "End Terror would withdraw it, but not yet: {} In this task: {}.",
                                reason(&c.explanation),
                                c.remedies.join(" · ")
                            ),
                            Some(c) => format!(
                                "End Terror would withdraw it, but not now: {}",
                                reason(&c.explanation)
                            ),
                            None => "26July cannot withdraw the request now.".into(),
                        },
                    };
                    briefing(
                        "Transfer request pending".into(),
                        vec![
                            entry(
                                "REQUEST",
                                format!(
                                    "26July asked {g} for {amount}. Nothing moves until {g} authorizes; Open in the top band shows its decision."
                                ),
                            ),
                            entry("WITHDRAW", withdraw),
                        ],
                        if *waited {
                            format!(
                                "Waiting for {g}. The request stays pending; Save / quit keeps it."
                            )
                        } else {
                            format!("{g} decides the actual amount, from 0 to its limit (R 1.5.1).")
                        },
                    )
                }
                _ => briefing(
                    "Resource transfer".into(),
                    vec![entry(
                        "OPEN",
                        "Give 26July's Resources to a faction, or ask one for Resources, which it decides. A request is not a Resource transfer.",
                    )],
                    "Cancel closes the transfer without moving anything (R 1.5.1).".into(),
                ),
            });
        }
        _ => {}
    }
    let choice = listed(view, action)?;
    if !choice.supported {
        let resolved = id
            .space()
            .is_some_and(|space| view.resolved.iter().any(|r| r.space == space));
        let mut entries = vec![entry(
            if resolved {
                "DONE"
            } else if choice.remedies.is_empty() {
                "UNAVAILABLE"
            } else {
                "NOT YET"
            },
            reason(&choice.explanation),
        )];
        let rules = if choice.remedies.is_empty() {
            if resolved {
                String::new()
            } else {
                "Nothing in this task can make it available.".into()
            }
        } else {
            entries.push(Entry {
                chips: choice.remedies.clone(),
                ..entry("IN THIS TASK", "")
            });
            "Only remedies possible in this task are offered.".into()
        };
        return Some(briefing(title, entries, rules));
    }
    match &choice.preview {
        Some(preview) => {
            let (entries, rules) = previewed(view, action, preview);
            let rules = if rules.is_empty() {
                choice.explanation.clone()
            } else {
                rules.join(" ")
            };
            Some(briefing(title, entries, rules))
        }
        None => {
            let mut entries = vec![entry("WHAT IT DOES", reason(&choice.explanation))];
            if action == "remove"
                && let Some(owed) = &view.owed_terror
            {
                entries.push(entry(
                    "OWED",
                    format!(
                        "{} Terror is owed: its last Underground Guerrilla stays (accepted commitment policy, local).",
                        name_of(view, owed)
                    ),
                ));
            }
            Some(briefing(title, entries, String::new()))
        }
    }
}

fn name_of(view: &Observation, id: &str) -> String {
    space_name(view, id)
        .split(' ')
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag_of(game: &Game, action: &str) -> Briefing {
        tag(&game.observe(), action).unwrap_or_else(|| panic!("no tag for {action}"))
    }
    fn words(b: &Briefing) -> String {
        let mut all: Vec<String> = b
            .entries
            .iter()
            .map(|e| format!("{}: {} {}", e.label, e.text, e.chips.join(" · ")))
            .collect();
        all.push(b.rules.clone());
        all.join(" | ")
    }
    fn step<'a>(steps: &'a [Step], action: &str) -> &'a Step {
        steps.iter().find(|s| s.action == action).unwrap()
    }

    /// The layout's sections and rows follow the authored pitch: rows 97.2 frame
    /// units apart within a section (36 px at the reference), and every row sits
    /// between the header and Kidnap's Cancel.
    #[test]
    fn rows_follow_the_authored_pitch() {
        for pair in fig::ROWS.windows(2) {
            let ((a, sa), (b, sb)) = (pair[0], pair[1]);
            if sa == sb {
                assert!((b - a - 97.2).abs() < 0.01, "{a} {b}");
            }
        }
        const { assert!(fig::ROWS[0].0 > acb::SUBTITLE[1]) };
        const { assert!(fig::ROWS[6].0 + acb::ROW_H < fig::CANCEL[1]) };
        const { assert!(fig::CANCEL[1] + fig::CANCEL[3] < acb::COMMIT[1]) };
    }

    /// Terror chosen at 0 Resources (EX-A state 1): both targets are blocked with
    /// their in-task remedies, Kidnap and Back are offered, nothing is pinned and the
    /// commit asks for a step.
    #[test]
    fn terror_chosen_at_zero_lists_every_step() {
        let view = Game::prepared().observe();
        let steps = steps(&view);
        assert_eq!(
            steps.iter().map(|s| s.action).collect::<Vec<_>>(),
            StepId::ALL.map(StepId::action)
        );
        for target in ["terror:santiago", "terror:oriente"] {
            assert_eq!(
                step(&steps, target).status,
                Status::Pill(Pill::NotYet, "NOT YET · 26JULY HAS 0".into()),
                "{target}"
            );
            assert!(step(&steps, target).interactive && !step(&steps, target).bold);
        }
        assert_eq!(
            step(&steps, "kidnap").status,
            Status::Note("opens its stakes".into(), false)
        );
        assert_eq!(
            step(&steps, "back-to-choice").status,
            Status::Note("until the first effect".into(), false)
        );
        assert!(steps.iter().all(|s| !s.dim && !s.held));
        let none = commit(&view, None);
        assert!(!none.enabled);
        assert_eq!(none.name, "execution-commit");
        assert_eq!(none.label, "Click a step to choose it");
    }

    /// A blocked target's tag states the authority's reason and its remedies as
    /// chips; its commit is disabled and names it (#62's honest hints).
    #[test]
    fn a_blocked_target_states_its_reason_and_remedies() {
        let game = Game::prepared();
        let b = tag_of(&game, "terror:santiago");
        assert_eq!(b.title, "Terror · Santiago de Cuba");
        assert_eq!(b.entries[0].label, "NOT YET");
        assert!(
            b.entries[0]
                .text
                .contains("Cost: 1 Resource per space (26July has 0)"),
            "{}",
            words(&b)
        );
        assert_eq!(b.entries[1].label, "IN THIS TASK");
        assert_eq!(
            b.entries[1].chips,
            ["Kidnap here first", "Request a Resource transfer"]
        );
        assert!(!words(&b).contains("Nothing in this task"));
        let commit = commit(&game.observe(), Some("terror:santiago"));
        assert!(!commit.enabled);
        assert_eq!(
            commit.label,
            "Terror in Santiago de Cuba is not available yet"
        );
        let Action::Unsupported(reason) = commit.action else {
            panic!("a blocked step commits nothing")
        };
        assert!(
            reason.contains("In this task: Kidnap here first"),
            "{reason}"
        );
    }

    /// #75: while Santiago's Terror is owed, End Terror's tag names resolving it, the
    /// authority's remedy, and never says nothing in this task can help.
    #[test]
    fn the_owed_end_terror_tag_names_resolving_santiago() {
        let mut game = Game::prepared_with_seed(19);
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        game.submit(Faction::M26, 1, "1:kidnap:roll").unwrap();
        let b = tag_of(&game, "end-terror");
        assert!(
            words(&b).contains("Santiago Terror is owed"),
            "{}",
            words(&b)
        );
        assert_eq!(b.entries[1].chips, ["Resolve Santiago's Terror"]);
        assert!(!words(&b).contains("Nothing in this task"));
        let steps = steps(&game.observe());
        assert_eq!(
            step(&steps, "end-terror").status,
            Status::Pill(Pill::NotYet, "NOT YET · SANTIAGO OWED".into())
        );
        assert_eq!(
            step(&steps, "terror:santiago").status,
            Status::Pill(Pill::Owed, "OWED · 1 RESOURCE".into())
        );
        let roll = game.observe().kidnap.unwrap();
        assert_eq!(
            step(&steps, "kidnap").status,
            Status::Pill(Pill::Done, format!("ROLLED {}", roll.die))
        );
        assert_eq!(
            step(&steps, "back-to-choice").status,
            Status::Pill(Pill::Unavailable, "NO LONGER OFFERED".into())
        );
    }

    /// End Terror before anything happened (EX-A state 2): what it records,
    /// Directorio's options, no Event, the zero-space warning, and that Back is still
    /// offered.
    #[test]
    fn the_zero_space_end_terror_tag_warns_and_names_directorio() {
        let b = tag_of(&Game::prepared(), "end-terror");
        let all = words(&b);
        for fact in [
            "RECORDS: Operation Only (R 2.3.4)",
            "NEXT: Directorio, 2nd Eligible, may: Pass (+1 if executed later) · Limited Operation",
            "NO EVENT: No Event for Directorio",
            "ZERO SPACES: No Terror resolved",
            "no Pass income",
            "Back to the action choice is still offered",
        ] {
            assert!(all.contains(fact), "{fact}: {all}");
        }
        let commit = commit(&Game::prepared().observe(), Some("end-terror"));
        assert_eq!(commit.label, "End Terror · records Operation Only");
        assert_eq!(commit.name, "0:end-terror");
    }

    /// Kidnap opened (EX-A state 3): the stakes stay on the held Kidnap step; Roll
    /// commits at the foot and Cancel sits above it; the steps that wait are dimmed.
    #[test]
    fn an_opened_kidnap_holds_its_stakes_and_roll() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        let view = game.observe();
        assert_eq!(held(&view), Some("kidnap"));
        let b = tag_of(&game, "kidnap");
        let all = words(&b);
        for fact in [
            "TARGET: Government in Santiago de Cuba, which has 6 Resources",
            "POSSIBLE PROCEEDS: 1–6 Resources · guaranteed minimum 1",
            "OWES SANTIAGO'S TERROR: Rolling before Santiago's Terror owes it",
            "EVENT FOR DIRECTORIO",
            "R 4.3.3",
        ] {
            assert!(all.contains(fact), "{fact}: {all}");
        }
        assert!(!all.contains("No Resources"), "{all}");
        let commit = commit(&view, None);
        assert_eq!(
            (commit.name.as_str(), commit.label.as_str()),
            ("1:kidnap:roll", "Roll the Kidnap die · commits the result")
        );
        assert_eq!(cancel(&view).unwrap().id, "1:kidnap:cancel");
        let steps = steps(&view);
        assert!(step(&steps, "kidnap").held);
        for waiting in ["transfer", "remove", "end-terror", "back-to-choice"] {
            assert!(step(&steps, waiting).dim, "{waiting}");
            assert!(!step(&steps, waiting).interactive, "{waiting}");
        }
        assert_eq!(
            step(&steps, "back-to-choice").status,
            Status::Pill(Pill::Unavailable, "AFTER ROLL OR CANCEL".into())
        );
        let order: Vec<String> = focusable(&view, None).into_iter().map(|c| c.0).collect();
        assert_eq!(order, ["step:kidnap", "1:kidnap:cancel", "1:kidnap:roll"]);
    }

    /// A selectable Terror's tag is the authority's preview: cost, the map, the
    /// standing, then the procedure as its rules line; the commit names the space.
    #[test]
    fn a_selectable_target_previews_from_the_authority() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:transfer").unwrap();
        game.submit(Faction::M26, 1, "1:ask:2:1").unwrap();
        game.submit(Faction::Directorio, 2, "2:authorize:1")
            .unwrap();
        let b = tag_of(&game, "terror:oriente");
        let all = words(&b);
        // Oriente: Pop 2, Passive → Active Opposition, two levels of opposition
        // population: 26July's standing 7 → 9, margin −8 → −6 (handoff arithmetic).
        for fact in [
            "COST: 26July Resources: 1 → 0",
            "Oriente: flip 1 Underground Guerrilla to Active",
            "Oriente: Terror marker placed",
            "Oriente: Passive Opposition → Active Opposition",
            "26July victory standing 7 → 9 (margin -8 → -6)",
            "R 3.3.4: pay 1 Resource",
        ] {
            assert!(all.contains(fact), "{fact}: {all}");
        }
        let commit = commit(&game.observe(), Some("terror:oriente"));
        assert_eq!(commit.label, "Terror in Oriente · pay 1 Resource");
        assert!(commit.enabled);
    }

    /// GLOSSARY.md: "Voluntary removal" and "Resource transfer", never "donation",
    /// "gift" or "voluntary own removal", in the panel, its tags and the offered
    /// choices.
    #[test]
    fn transfer_and_removal_steps_use_the_glossary_terms() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:remove").unwrap();
        let view = game.observe();
        assert_eq!(held(&view), Some("remove"));
        let mut state = choice::State::default();
        state.decided(&view);
        assert_eq!(state.pinned, None, "removal's tag would cover the pieces");
        let b = tag_of(&game, "remove");
        assert_eq!(b.title, "Voluntary removal");
        assert_eq!(commit(&view, None).label, "Done · leave removal mode");
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:transfer").unwrap();
        let text = words(&tag_of(&game, "transfer")).to_lowercase();
        assert!(
            text.contains("a request is not a resource transfer"),
            "{text}"
        );
        for words in [text].into_iter().chain(
            game.observe()
                .choices
                .iter()
                .map(|c| format!("{} {}", c.label, c.explanation).to_lowercase()),
        ) {
            assert!(
                !words.contains("gift") && !words.contains("donat"),
                "{words}"
            );
        }
    }

    /// A pending request after Wait (EX-A state 5): the transfer step holds it, End
    /// Terror and Back withdraw it, and the commit opens it in the giver's colour.
    #[test]
    fn a_pending_request_holds_the_transfer_step() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:transfer").unwrap();
        game.submit(Faction::M26, 1, "1:ask:3:3").unwrap();
        game.submit(Faction::Syndicate, 2, "2:wait").unwrap();
        let view = game.observe();
        let steps = steps(&view);
        assert!(step(&steps, "transfer").held);
        assert_eq!(
            step(&steps, "transfer").status,
            Status::Note("pending · Syndicate decides".into(), true)
        );
        for withdraw in ["end-terror", "back-to-choice"] {
            assert_eq!(
                step(&steps, withdraw).status,
                Status::Note("withdraws the request".into(), false)
            );
            assert!(step(&steps, withdraw).interactive);
        }
        let open = commit(&view, None);
        assert_eq!(open.label, "Open the request · Syndicate decides");
        assert_eq!(open.plate, "atlas-transfer-primary-syndicate.png");
        assert_eq!(open.name, "execution-commit");
        assert!(matches!(open.action, Action::Submit(_, 3, ref id) if id == "3:reopen"));
        let end = commit(&view, Some("end-terror"));
        assert_eq!(end.name, "3:end-terror");
        let b = tag_of(&game, "transfer");
        assert!(
            words(&b).contains("26July asked Syndicate for 3"),
            "{}",
            words(&b)
        );
        assert!(b.rules.starts_with("Waiting for Syndicate"));
    }

    /// The ended panels state the record, Eligibility and Directorio's options from
    /// the observation (shown, not executed).
    #[test]
    fn the_ended_panel_states_the_record_and_directorio_next() {
        let mut game = Game::opening();
        game.submit(Faction::M26, 0, "0:pass").unwrap();
        let (heading, _, entries) = ending(&game.observe()).unwrap();
        assert_eq!(heading, "26July · passed");
        assert_eq!(
            entries[0].text.lines().next(),
            Some("26July Resources: 0 → 1")
        );
        assert!(
            entries[2]
                .text
                .starts_with("Directorio, 1st Eligible, may: Pass")
        );
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:end-terror").unwrap();
        let (heading, subtitle, entries) = ending(&game.observe()).unwrap();
        assert_eq!(heading, "26July · Terror ended");
        assert!(subtitle.starts_with("Operation Only recorded (R 2.3.4)"));
        assert!(entries[0].text.starts_with("No Terror resolved"));
        assert_eq!(entries[3].label, "NO EVENT");
        for e in entries {
            for line in briefing::lines(&e.text, fig::CHARS) {
                assert!(line.chars().count() <= fig::CHARS, "{line}");
            }
        }
    }

    /// Review of #84: an early Kidnap owes Santiago's Terror, and a request then
    /// waits on its giver. End Terror would withdraw the request, but the authority
    /// refuses it until Santiago is resolved, so the row, its tag and the commit all
    /// say not yet, with the authority's reason and remedy, and the commit is disabled.
    /// The remedy cannot be taken until the giver decides, so every surface states
    /// that timing, from the authority's remedy, for each faction that can be asked.
    #[test]
    fn a_pending_request_does_not_offer_an_end_terror_the_authority_refuses() {
        for giver in [Faction::Government, Faction::Directorio, Faction::Syndicate] {
            let g = giver.name();
            let remedy = format!("Resolve Santiago's Terror once {g} has decided");
            let mut game = Game::prepared_with_seed(19);
            game.submit(Faction::M26, 0, "0:kidnap").unwrap();
            game.submit(Faction::M26, 1, "1:kidnap:roll").unwrap();
            game.submit(Faction::M26, 2, "2:transfer").unwrap();
            game.submit(Faction::M26, 3, &format!("3:ask:{}:1", giver.index()))
                .unwrap();
            for waited in [false, true] {
                if waited {
                    game.submit(giver, 4, "4:wait").unwrap();
                }
                let view = game.observe();
                assert_eq!(mode(&view), Mode::Transfer);
                let end = listed(&view, "end-terror").unwrap();
                assert!(!end.supported, "the authority refuses it");
                assert_eq!(end.remedies, std::slice::from_ref(&remedy));
                let steps = steps(&view);
                let row = step(&steps, "end-terror");
                assert_eq!(
                    row.status,
                    Status::Pill(Pill::NotYet, "NOT YET · SANTIAGO OWED".into())
                );
                assert!(row.interactive && !row.bold);
                let b = tag_of(&game, "end-terror");
                assert_eq!(b.entries[0].label, "NOT YET");
                assert!(
                    words(&b).contains("Santiago Terror is owed"),
                    "{}",
                    words(&b)
                );
                assert_eq!(b.entries[1].chips, std::slice::from_ref(&remedy));
                let commit = commit(&view, Some("end-terror"));
                assert!(!commit.enabled);
                assert_eq!(commit.label, "End Terror is not available yet");
                let Action::Unsupported(reason) = commit.action else {
                    panic!("a refused End Terror commits nothing")
                };
                assert!(reason.contains("Santiago Terror is owed"), "{reason}");
                assert!(
                    reason.contains(&format!("In this task: {remedy}.")),
                    "{reason}"
                );
                // The pending request's tag does not offer End Terror as a withdrawal,
                // and words the same remedy with the same timing.
                let pending = words(&tag_of(&game, "transfer"));
                assert!(!pending.contains("may withdraw"), "{pending}");
                assert!(pending.contains("but not yet"), "{pending}");
                assert!(
                    pending.contains(&format!("In this task: {remedy}.")),
                    "{pending}"
                );
                // Back is no longer offered after the roll, so nothing else withdraws.
                assert!(listed(&view, "back-to-choice").is_none());
            }
        }
    }

    /// The ended panel words the authority's record, not presentation's guesses: the
    /// recorded box, 26July's Eligibility for the next card, the next faction with its
    /// role and options, the Event's availability, the Terror resolved and what it
    /// paid, and the Resources Pass moved. Each fact is changed here to one the old
    /// derivations (from the roll or the box, a fixed sentence, `r - 1 → r`, the first
    /// Eligible faction that is not 26July, and the map's markers) would misstate.
    #[test]
    fn the_ended_panel_reads_the_authority_s_record() {
        let mut game = Game::opening();
        game.submit(Faction::M26, 0, "0:pass").unwrap();
        let mut view = game.observe();
        assert_eq!(
            view.recorded_resources,
            [crate::ResourceChange {
                faction: Faction::M26,
                from: 0,
                to: 1
            }]
        );
        // At the 49 cap, Pass gains nothing (R 1.8): 49 → 49, not 48 → 49.
        view.resources[Faction::M26.index()] = 49;
        view.recorded_resources[0].from = 49;
        view.recorded_resources[0].to = 49;
        let (_, _, entries) = ending(&view).unwrap();
        assert_eq!(
            entries[0].text.lines().next(),
            Some("26July Resources: 49 → 49")
        );
        assert_eq!(entries[1].text, "26July stays Eligible (R 2.3.3).");
        assert!(
            entries[2]
                .text
                .starts_with("Directorio, 1st Eligible, may:")
        );

        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:end-terror").unwrap();
        let mut view = game.observe();
        assert!(view.kidnap.is_none());
        assert_eq!(view.eligibility.m26_next_card, Some(false));
        let next = view.next.clone().unwrap();
        assert_eq!(
            (next.faction, next.role.as_str()),
            (Faction::Directorio, "2nd Eligible")
        );
        // The box says Special Activity, but the authority lists no Event: the old
        // derivation from the box offered it anyway.
        view.eligibility.m26_box = Some(SequenceBox::OperationWithSpecialActivity);
        let (heading, subtitle, entries) = ending(&view).unwrap();
        assert_eq!(heading, "26July · Terror ended");
        assert!(
            subtitle.starts_with("Operation with Special Activity recorded (R 2.3.4)"),
            "{subtitle}"
        );
        assert_eq!(
            entries[1].text,
            "26July becomes Ineligible after this card (R 2.3.7)."
        );
        assert_eq!(entries[3].label, "NO EVENT");
        // The next faction is the authority's, not the first Eligible one that is not
        // 26July: changing the Eligibility record changes nothing here.
        view.eligibility.second = Faction::Government;
        let (_, _, entries) = ending(&view).unwrap();
        assert!(
            entries[2]
                .text
                .starts_with("Directorio, 2nd Eligible, may:"),
            "{}",
            entries[2].text
        );
        view.next = Some(crate::Next {
            faction: Faction::Syndicate,
            role: "1st Eligible".into(),
            options: vec!["Pass (+1 if executed later)".into(), "Event".into()],
        });
        let (_, _, entries) = ending(&view).unwrap();
        assert_eq!(
            entries[2].text,
            "Syndicate, 1st Eligible, may: Pass (+1 if executed later) · Event"
        );
        assert_eq!(entries[3].label, "EVENT FOR SYNDICATE");
        assert!(
            entries[3].text.contains("open to Syndicate"),
            "{}",
            entries[3].text
        );
        // The Terror resolved and what it paid are the authority's record, not the
        // map's markers counted at 1 Resource each.
        assert!(view.spaces.iter().all(|s| s.terror == 0));
        assert!(entries[0].text.starts_with("No Terror resolved"));
        view.resolved = vec![crate::Resolved {
            space: "santiago".into(),
            paid: 2,
        }];
        let (_, _, entries) = ending(&view).unwrap();
        assert_eq!(
            entries[0].text.lines().next(),
            Some("Terror resolved: Santiago de Cuba · paid 2 Resources")
        );
        view.eligibility.m26_next_card = None;
        let (_, _, entries) = ending(&view).unwrap();
        assert!(
            !entries[1].text.contains("Ineligible"),
            "{}",
            entries[1].text
        );
        // Nothing recorded yet: no ended panel.
        assert!(ending(&Game::prepared().observe()).is_none());
    }

    #[test]
    fn event_preview_names_the_next_faction() {
        let mut game = Game::prepared_with_seed(5);
        let mut view = game.observe();
        // Kidnap has yet to end the Operation. Its preview still knows which
        // faction is 2nd Eligible, through the current eligibility record.
        view.eligibility.second = Faction::Syndicate;
        let kidnap = tag(&view, "kidnap").unwrap();
        assert!(
            kidnap
                .entries
                .iter()
                .any(|e| e.label == "EVENT FOR SYNDICATE")
        );

        for (revision, action) in ["kidnap", "kidnap:roll", "terror:santiago"]
            .iter()
            .enumerate()
        {
            game.submit(
                Faction::M26,
                revision as u64,
                &format!("{revision}:{action}"),
            )
            .unwrap();
        }
        let mut view = game.observe();
        let end = view
            .choices
            .iter_mut()
            .find(|c| c.action() == "end-terror")
            .unwrap();
        end.preview.as_mut().unwrap().next.as_mut().unwrap().faction = Faction::Government;
        let end = tag(&view, "end-terror").unwrap();
        assert!(
            end.entries
                .iter()
                .any(|e| e.label == "EVENT FOR GOVERNMENT")
        );
    }

    /// A resolved target's pill states what the authority recorded it paid, and a
    /// Terror marker alone does not make it done; an offered target's note and
    /// commit, and the End Terror row, word the authority's cost and record.
    #[test]
    fn the_steps_word_the_authority_s_record_and_costs() {
        let mut game = Game::prepared_with_seed(5);
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        game.submit(Faction::M26, 1, "1:kidnap:roll").unwrap();
        game.submit(Faction::M26, 2, "2:terror:santiago").unwrap();
        let mut view = game.observe();
        assert_eq!(
            view.resolved,
            [crate::Resolved {
                space: "santiago".into(),
                paid: 1
            }]
        );
        let row = |view: &Observation, action: &str| {
            steps(view)
                .into_iter()
                .find(|s| s.action == action)
                .unwrap()
                .status
        };
        assert_eq!(
            row(&view, "terror:santiago"),
            Status::Pill(Pill::Done, "DONE · PAID 1".into())
        );
        assert_eq!(
            row(&view, "terror:oriente"),
            Status::Note("pay 1 Resource".into(), false)
        );
        assert_eq!(
            row(&view, "end-terror"),
            Status::Note("records Operation with Special Activity".into(), false)
        );
        assert_eq!(
            commit(&view, Some("terror:oriente")).label,
            "Terror in Oriente · pay 1 Resource"
        );
        view.resolved[0].paid = 2;
        assert_eq!(
            row(&view, "terror:santiago"),
            Status::Pill(Pill::Done, "DONE · PAID 2".into())
        );
        let target_status = |view: &Observation| {
            let tag = tag(view, "terror:santiago").unwrap();
            let card = super::super::popover::content(view, "santiago", None).unwrap();
            let terror = card.rows.iter().find(|r| r.head == "Terror").unwrap();
            (tag.entries[0].label.to_string(), terror.mark)
        };
        use super::super::popover::Mark;
        // The matching resolution still counts without a marker.
        let mut no_marker = view.clone();
        no_marker
            .spaces
            .iter_mut()
            .find(|s| s.id == "santiago")
            .unwrap()
            .terror = 0;
        assert_eq!(target_status(&no_marker), ("DONE".into(), Mark::Done));

        view.resolved[0].space = "oriente".into();
        assert_eq!(
            target_status(&view),
            ("UNAVAILABLE".into(), Mark::Unavailable)
        );
        view.resolved.clear();
        assert_eq!(
            target_status(&view),
            ("UNAVAILABLE".into(), Mark::Unavailable)
        );
        assert_ne!(
            row(&view, "terror:santiago"),
            Status::Pill(Pill::Done, "DONE · PAID 1".into())
        );
    }
}
