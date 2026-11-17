//! The space popover of the selected HF-A Survey ink design (#68), during Terror
//! execution (#72). Hovering a selectable or blocked space shows it; clicking the
//! space keeps it open. It lists every action the rules authority offers there, each
//! with its state, and the chosen row's preview above the only button that acts, so a
//! step's consequences are visible before the committing click. Blocked rows state
//! what is missing and the in-task remedies the authority names.
//!
//! In removal mode (#73) the same card is the piece popover: hovering a 26July piece
//! shows it, clicking a removable one keeps it open. It names the piece, its state and
//! space, offers a quantity for a stack, previews the removal the authority computes,
//! and acts only through Remove; Cancel closes it. The protected Guerrilla shows its
//! reason on hover, and a click on it opens nothing.
//!
//! Layout: `art/proposals/issue-66/highlight-popover-layouts.figcraft`, page "Popover
//! masters (1920x1280 reference units)", frames A-P1, A-P3 to A-P7. Its frames' sizes,
//! padding, gaps, fills, strokes, corner radii, `Atlas` colour variables and `Popover/*`
//! text styles are implemented here in reference pixels (foundation 4, "Layout
//! authority": the first consumer implements against the layout). The anchor dot and
//! leader are the accepted Atlas status components. Wording is presentation; every
//! fact and number is the authority's own preview, reason or remedy.
use super::*;
use crate::{
    Choice, KIDNAP_SPACE, Observation,
    targets::{PieceState, PieceTarget, SpaceState},
};
use bevy::ui::{BoxShadow, FlexWrap};
use emblems::Naming;
use highlights::Targeting;

/// The FigCraft `Atlas` variables the popover uses.
const INK: Color = Color::srgb_u8(32, 50, 56);
const INK_SECONDARY: Color = Color::srgb_u8(83, 106, 109);
const PAPER: Color = Color::srgb_u8(248, 244, 232);
const DESK: Color = Color::srgb_u8(238, 233, 220);
const RULE: Color = Color::srgb_u8(211, 219, 215);
/// `action/primary`.
const PRIMARY: Color = Color::srgb_u8(160, 53, 49);

/// Layout values in reference pixels, from the A-P1/A-P3/A-P4 masters.
mod frame {
    pub const WIDTH: f32 = 340.;
    /// The protected hover tag, A-P6.
    pub const TAG_WIDTH: f32 = 300.;
    /// The leader from the anchor dot to the card, and the dot's diameter, from the
    /// 16:9 frame's "leader (anchored to Santiago)" and "anchor dot" at its 3.2 scale.
    pub const LEADER: f32 = 39.;
    pub const DOT: f32 = 8.;
    /// Keeps the card this far inside the map and clear of the Operation panel.
    pub const MARGIN: f32 = 8.;
}

/// The Bold Lato face the `Popover/Bold *` styles use; admitted with the regular face.
#[derive(Resource)]
pub(super) struct Bold(Handle<Font>);

pub(super) fn setup(mut commands: Commands, art: Res<Art>, mut fonts: ResMut<Assets<Font>>) {
    commands.insert_resource(Bold(fonts.add(Font::from_bytes(
        art.verified.files["Lato-Bold.ttf"].clone(),
    ))));
}

/// Which space's popover is kept open, and which of its rows is chosen.
#[derive(Resource, Default)]
pub(super) struct Popover {
    /// The space and the match revision it was opened at. A decision closes it.
    pinned: Option<(String, u64)>,
    /// The chosen row's action, without its revision.
    selected: Option<String>,
    /// Removal mode: the 26July group (`space:group`) whose popover is kept open, and
    /// the match revision it was opened at. A decision closes it.
    piece: Option<(String, u64)>,
    /// The quantity its stepper shows, from 1 to what may be removed now.
    quantity: u8,
    /// The 26July group under the pointer, if any.
    pub hovered: Option<String>,
}

impl Popover {
    /// A region click while targeting. Outside targeting states it does nothing.
    /// Returns whether the click opened (kept open) a space's popover.
    pub fn click_space(&mut self, space: &str, targeting: &Targeting) -> bool {
        let Some(revision) = targeting.view.as_ref().map(|v| v.revision) else {
            return false;
        };
        // In removal mode a region click only closes a piece popover.
        if targeting.removal.is_some() {
            self.piece = None;
            return false;
        }
        match targeting.state(space) {
            None => {}
            Some(SpaceState::Selectable | SpaceState::Blocked) => {
                if self.pinned.as_ref().is_some_and(|(s, _)| s == space) {
                    self.pinned = None;
                } else {
                    self.pinned = Some((space.into(), revision));
                }
                self.selected = None;
            }
            // Closing the popover is the only effect a not-involved space has.
            Some(SpaceState::NotInvolved) => {
                self.pinned = None;
                self.selected = None;
            }
        }
        self.pinned.as_ref().is_some_and(|(s, _)| s == space)
    }
    pub fn choose_row(&mut self, action: &str) {
        self.selected = Some(action.into());
    }
    /// A click on a 26July piece in removal mode: a removable group's popover opens,
    /// or closes when it is the open one; a protected piece opens nothing.
    pub fn click_piece(&mut self, key: &str, targeting: &Targeting) {
        let Some(revision) = targeting.view.as_ref().map(|v| v.revision) else {
            return;
        };
        if targeting
            .piece(key)
            .is_none_or(|t| t.mark != PieceState::Removable)
        {
            return;
        }
        if self.piece.as_ref().is_some_and(|(k, _)| k == key) {
            self.piece = None;
        } else {
            self.piece = Some((key.into(), revision));
            self.quantity = 1;
        }
    }
    /// The stepper: one more or one fewer, within 1 and what may be removed now.
    pub fn step(&mut self, delta: i8, targeting: &Targeting) {
        if let Some(target) = self.piece.as_ref().and_then(|(k, _)| targeting.piece(k)) {
            self.quantity = (i16::from(self.quantity) + i16::from(delta))
                .clamp(1, i16::from(target.removable.max(1))) as u8;
        }
    }
    /// Cancel: closes the piece popover; nothing is decided.
    pub fn close(&mut self) {
        self.piece = None;
    }
    /// Closes any kept-open popover: the AC-B briefing tag opened after it wins
    /// (orchestrator implementation judgement, #58 carry-forward).
    pub fn close_all(&mut self) {
        self.pinned = None;
        self.selected = None;
        self.piece = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Mark {
    /// Can be done now (a filled `action/primary` dot).
    Selectable,
    /// Blocked, with in-task remedies (an `action/primary` ring).
    NotYet,
    /// Blocked with no remedy in this task (an `ink-secondary` ring).
    Unavailable,
    /// Already done here: kept as history (a filled `ink-secondary` dot).
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Tone {
    Plain,
    Strong,
    Quiet,
    /// A line at preview size in `ink-secondary`, such as a limit's reason.
    Secondary,
}

/// Which popover a card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum CardKind {
    /// A space's actions during Terror (A-P1, A-P3, A-P4).
    Space,
    /// A removable 26July group in removal mode (A-P5, A-P7).
    Piece,
    /// The protected Guerrilla's hover tag (A-P6).
    Protected,
}

/// A stack's quantity stepper: the value shown, the most that may be removed now and
/// the stack's size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Stepper {
    pub value: u8,
    pub max: u8,
    pub count: u8,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Row {
    /// The full choice id, or none for history.
    pub choice: Option<String>,
    pub owner: Faction,
    pub mark: Mark,
    pub chosen: bool,
    pub head: String,
    pub kind: String,
    /// The chosen row's preview, or a blocked row's reason.
    pub lines: Vec<(Tone, String)>,
    pub remedies: Vec<String>,
    /// Said instead of remedies when nothing in this task can help.
    pub no_remedy: Option<String>,
    /// The committing button, on the chosen row only.
    pub button: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Card {
    pub kind: CardKind,
    pub space: String,
    /// The 26July group (`space:group`) of a piece popover or tag.
    pub piece: Option<String>,
    pub revision: u64,
    pub title: String,
    pub subtitle: String,
    pub rows: Vec<Row>,
    /// A piece popover's stepper, for a stack of more than one.
    pub stepper: Option<Stepper>,
    /// A piece popover's Cancel button.
    pub cancel: bool,
    pub footer: String,
}

const FOOTER: &str = "Choose a row to see what it does; only the button acts.";
const NO_REMEDY: &str = "Nothing in this task can change this, so no remedy is offered.";

fn action(choice: &Choice) -> &str {
    choice.id.split_once(':').map_or(&choice.id, |(_, a)| a)
}

/// "R 3.3.4. Cost: …" reads as "Cost: … R 3.3.4", as the layout's reason line does.
fn reason(explanation: &str) -> String {
    match explanation.split_once(". ") {
        Some((rule, rest))
            if rule.strip_prefix("R ").is_some_and(|n| {
                !n.is_empty() && n.chars().all(|c| c.is_ascii_digit() || c == '.')
            }) =>
        {
            format!("{rest} {rule}")
        }
        _ => explanation.into(),
    }
}

fn plural(n: u8, one: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {one}s")
    }
}

/// The chosen row's preview: the authority's computed consequences as sentences.
fn preview_lines(view: &Observation, choice: &Choice) -> Vec<(Tone, String)> {
    let mut lines = vec![];
    let Some(preview) = &choice.preview else {
        lines.push((Tone::Plain, choice.explanation.clone()));
        return lines;
    };
    let name = |id: &str| {
        view.spaces
            .iter()
            .find(|s| s.id == id)
            .map_or(id.to_owned(), |s| s.name.clone())
    };
    if let Some(stakes) = &preview.kidnap {
        // The requirement, as the authority states it, then the stakes.
        if let Some((requirement, _)) = choice.explanation.split_once(". ") {
            lines.push((Tone::Plain, format!("{requirement}.")));
        }
        lines.push((
            Tone::Strong,
            format!(
                "The roll takes {} from {} (has {}); at least {}. R 4.3.3",
                preview::proceeds(stakes),
                stakes.target.name(),
                stakes.government,
                stakes.minimum
            ),
        ));
    }
    for r in &preview.resources {
        if r.faction == Faction::M26 && r.to < r.from {
            lines.push((
                Tone::Plain,
                format!(
                    "Pay {}: 26July {} → {}.",
                    plural(r.from - r.to, "Resource"),
                    r.from,
                    r.to
                ),
            ));
        } else {
            lines.push((
                Tone::Plain,
                format!("{} Resources {} → {}.", r.faction.name(), r.from, r.to),
            ));
        }
    }
    for f in &preview.flips {
        lines.push((
            Tone::Plain,
            format!(
                "Flip {} {:?} {:?} there to {:?}.",
                f.count, f.from, f.kind, f.to
            ),
        ));
    }
    for r in &preview.removed {
        lines.push((
            Tone::Plain,
            format!(
                "{}: {} {} {:?} returns to Available.",
                name(&r.space),
                r.count,
                r.faction.name(),
                r.kind
            ),
        ));
    }
    let mut outcome = vec![];
    if !preview.terror.is_empty() {
        outcome.push("Place a Terror marker".to_owned());
    }
    outcome.extend(preview.support.iter().map(|s| s.label.clone()));
    outcome.extend(preview.control.iter().map(|c| {
        format!(
            "Control of {}: {} → {}",
            name(&c.space),
            c.from.map_or("none", Faction::name),
            c.to.map_or("none", Faction::name)
        )
    }));
    let standings = preview
        .standings
        .iter()
        .map(|s| {
            format!(
                "{} {} → {} (margin {} → {})",
                s.faction.name(),
                s.score_from,
                s.score_to,
                s.margin_from,
                s.margin_to
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    if !outcome.is_empty() || !standings.is_empty() {
        let mut sentence = outcome.join("; ");
        if !standings.is_empty() {
            if !sentence.is_empty() {
                sentence.push_str(": ");
            }
            sentence.push_str(&standings);
        }
        sentence.push('.');
        lines.push((Tone::Strong, sentence));
    }
    for u in &preview.unavailable {
        lines.push((Tone::Plain, format!("Then unavailable: {}.", u.label)));
    }
    let terror = action(choice).starts_with("terror:");
    for note in &preview.notes {
        // The procedure and Kidnap's rule are already stated above.
        if (terror && note.starts_with("R 3.3.4:")) || note.starts_with("R 4.3.3:") {
            continue;
        }
        let tone = if note.contains("card 17's Event") {
            Tone::Quiet
        } else {
            Tone::Plain
        };
        lines.push((tone, note.clone()));
    }
    if terror {
        lines.push((Tone::Quiet, "R 3.3.4".into()));
    }
    lines
}

/// What a space's popover shows, or `None` when no action is offered there.
pub(super) fn content(view: &Observation, space: &str, selected: Option<&str>) -> Option<Card> {
    let here: Vec<&Choice> = view
        .choices
        .iter()
        .filter(|c| c.space() == Some(space))
        .collect();
    if here.is_empty() {
        return None;
    }
    let place = view.spaces.iter().find(|s| s.id == space)?;
    let chosen = selected
        .and_then(|a| here.iter().find(|c| c.supported && action(c) == a))
        .or_else(|| here.iter().find(|c| c.supported))
        .map(|c| c.id.clone());
    let head = |choice: &Choice| -> (String, String) {
        if action(choice) == "kidnap" {
            ("Kidnap · from Government".into(), "Special Activity".into())
        } else {
            ("Terror".into(), "Operation".into())
        }
    };
    let mut rows = vec![];
    for choice in here.iter().filter(|c| c.supported) {
        let (head, kind) = head(choice);
        let is_chosen = chosen.as_ref() == Some(&choice.id);
        rows.push(Row {
            choice: Some(choice.id.clone()),
            owner: choice.owner,
            mark: Mark::Selectable,
            chosen: is_chosen,
            head,
            kind,
            lines: if is_chosen {
                preview_lines(view, choice)
            } else {
                vec![]
            },
            remedies: vec![],
            no_remedy: None,
            button: is_chosen.then(|| {
                if action(choice) == "kidnap" {
                    "Open Kidnap · nothing rolled yet".into()
                } else {
                    format!("Terror in {} · pay 1 Resource", place.name)
                }
            }),
        });
    }
    let blocked = |choice: &&&Choice| !choice.supported;
    for choice in here
        .iter()
        .filter(blocked)
        .filter(|c| !c.remedies.is_empty())
    {
        let (head, _) = head(choice);
        rows.push(Row {
            choice: Some(choice.id.clone()),
            owner: choice.owner,
            mark: Mark::NotYet,
            chosen: false,
            head: head.replace(" · from Government", ""),
            kind: "not yet".into(),
            lines: vec![(Tone::Plain, reason(&choice.explanation))],
            remedies: choice.remedies.clone(),
            no_remedy: None,
            button: None,
        });
    }
    for choice in here
        .iter()
        .filter(blocked)
        .filter(|c| c.remedies.is_empty())
    {
        let (head, _) = head(choice);
        // A Terror already resolved here is history, not a missing requirement.
        let done = action(choice).starts_with("terror:") && place.terror > 0;
        rows.push(Row {
            choice: Some(choice.id.clone()),
            owner: choice.owner,
            mark: if done { Mark::Done } else { Mark::Unavailable },
            chosen: false,
            head: head.replace(" · from Government", ""),
            kind: if done { "done" } else { "unavailable" }.into(),
            lines: vec![(Tone::Plain, reason(&choice.explanation))],
            remedies: vec![],
            no_remedy: (!done).then(|| NO_REMEDY.into()),
            button: None,
        });
    }
    if space == KIDNAP_SPACE
        && let Some(kidnap) = &view.kidnap
    {
        rows.push(Row {
            choice: None,
            owner: Faction::M26,
            mark: Mark::Done,
            chosen: false,
            head: format!("Kidnap · rolled {}", kidnap.die),
            kind: "done".into(),
            lines: vec![(
                Tone::Plain,
                format!(
                    "Government {} → {} · 26July {} → {}. The result stays.",
                    kidnap.government_before,
                    kidnap.government_before - kidnap.proceeds,
                    kidnap.m26_before,
                    kidnap.m26_before + kidnap.proceeds
                ),
            )],
            remedies: vec![],
            no_remedy: None,
            button: None,
        });
    }
    let terrain = match place.terrain {
        Terrain::City => "City",
        Terrain::Forest => "Province · Forest",
        Terrain::Grassland => "Province · Grassland",
        Terrain::Mountain => "Province · Mountain",
        Terrain::EconomicCenter => "Economic center",
    };
    let m26 = view.resources[Faction::M26.index()];
    let mut subtitle = format!(
        "{terrain} · 26July executing Terror · 26July has {}",
        plural(m26, "Resource")
    );
    if view.owed_terror.as_deref() == Some(space) {
        subtitle.push_str(&format!(" · {} Terror is owed", place.name));
    }
    Some(Card {
        kind: CardKind::Space,
        space: space.into(),
        piece: None,
        revision: view.revision,
        title: place.name.clone(),
        subtitle,
        rows,
        stepper: None,
        cancel: false,
        footer: FOOTER.into(),
    })
}

const PIECE_FOOTER: &str = "Removal stays open for more pieces until Done.";
const PROTECTED_FOOTER: &str = "Shown on hover; a click opens nothing.";

/// A removal's consequences as the layout words them: what returns, Control and what
/// the Operation still offers, the standings, then the rule's notes.
fn removal_lines(view: &Observation, choice: &Choice) -> Vec<(Tone, String)> {
    let mut lines = vec![];
    let Some(preview) = &choice.preview else {
        return vec![(Tone::Plain, choice.explanation.clone())];
    };
    let name = |id: &str| {
        view.spaces
            .iter()
            .find(|s| s.id == id)
            .map_or(id.to_owned(), |s| s.name.clone())
    };
    let mut base = false;
    for r in &preview.removed {
        base |= r.kind == crate::Kind::Base;
        let noun = if r.kind == crate::Kind::Base {
            "Base"
        } else {
            "Guerrilla"
        };
        let what = match (r.kind, r.count) {
            (crate::Kind::Base, 1) => "The Base".to_owned(),
            (crate::Kind::Base, n) => format!("{n} Bases"),
            (_, n) => format!("{n} {:?} {noun}{}", r.state, if n == 1 { "" } else { "s" }),
        };
        lines.push((
            Tone::Plain,
            format!(
                "{what} {} to Available.",
                if r.count == 1 { "returns" } else { "return" }
            ),
        ));
    }
    let space = choice.removal().map(|(s, ..)| s).unwrap_or_default();
    let control = match preview.control.first() {
        Some(c) => format!(
            "Control of {}: {} → {}.",
            name(&c.space),
            c.from.map_or("none", Faction::name),
            c.to.map_or("none", Faction::name)
        ),
        None => format!(
            "Control: unchanged ({}).",
            view.spaces
                .iter()
                .find(|s| s.id == space)
                .and_then(crate::Space::control)
                .map_or("none", Faction::name)
        ),
    };
    let offered = if preview.unavailable.is_empty() {
        "Every Terror and Kidnap now offered stays offered.".to_owned()
    } else {
        format!(
            "Then unavailable: {}.",
            preview
                .unavailable
                .iter()
                .map(|u| u.label.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    lines.push((Tone::Strong, format!("{control} {offered}")));
    if preview.standings.is_empty() {
        lines.push((Tone::Plain, "Standings: unchanged.".into()));
    }
    for s in &preview.standings {
        lines.push((
            Tone::Strong,
            format!(
                "{} {} → {} (margin {} → {}){}",
                s.faction.name(),
                s.score_from,
                s.score_to,
                s.margin_from,
                s.margin_to,
                if base && s.faction == Faction::M26 {
                    ": its Bases count toward its score."
                } else {
                    "."
                }
            ),
        ));
    }
    for note in &preview.notes {
        let tone = if note.starts_with("R 1.4.1") {
            Tone::Quiet
        } else {
            Tone::Plain
        };
        lines.push((tone, note.clone()));
    }
    lines
}

/// The piece popover for a 26July group in removal mode, or its protected tag. Every
/// fact is the authority's: the removal choice for the stepper's quantity, its preview,
/// and the reason a quantity or the piece itself is refused.
pub(super) fn piece_content(
    view: &Observation,
    target: &PieceTarget,
    quantity: u8,
) -> Option<Card> {
    let place = view.spaces.iter().find(|s| s.id == target.space)?;
    let choice = |n: u8| {
        target
            .choices
            .get(usize::from(n).checked_sub(1)?)
            .and_then(|id| view.choices.iter().find(|c| &c.id == id))
    };
    let base = target.kind == crate::Kind::Base;
    let (one, many) = if base {
        ("Base", "Bases")
    } else {
        ("Guerrilla", "Guerrillas")
    };
    let key = format!("{}:{}", target.space, target.group);
    if target.mark == PieceState::Protected {
        let refused = choice(1)?;
        let (reason, then) = refused
            .explanation
            .rsplit_once(". ")
            .map_or((refused.explanation.clone(), None), |(r, t)| {
                (format!("{r}."), Some(t.to_owned()))
            });
        let mut lines = vec![(Tone::Plain, reason)];
        lines.extend(then.map(|t| (Tone::Secondary, t)));
        return Some(Card {
            kind: CardKind::Protected,
            space: target.space.clone(),
            piece: Some(key),
            revision: view.revision,
            title: format!("Protected · 26July {one}"),
            subtitle: format!(
                "{} · {:?} · {}",
                place.name,
                target.state,
                if target.count == 1 {
                    "the last one"
                } else {
                    "all protected"
                }
            ),
            rows: vec![Row {
                choice: Some(refused.id.clone()),
                owner: refused.owner,
                mark: Mark::Unavailable,
                chosen: false,
                head: String::new(),
                kind: String::new(),
                lines,
                remedies: vec![],
                no_remedy: None,
                button: None,
            }],
            stepper: None,
            cancel: false,
            footer: PROTECTED_FOOTER.into(),
        });
    }
    let n = quantity.clamp(1, target.removable.max(1));
    let removal = choice(n).filter(|c| c.supported)?;
    let mut lines = vec![];
    if target.removable < target.count
        && let Some(limit) = choice(target.removable + 1)
    {
        lines.push((Tone::Secondary, limit.explanation.clone()));
    }
    lines.extend(removal_lines(view, removal));
    let title = if base {
        "26July Base".to_owned()
    } else {
        format!("26July Guerrilla · {:?}", target.state)
    };
    let size = if target.count > 1 {
        format!("stack of {}", target.count)
    } else {
        format!("1 {one}")
    };
    Some(Card {
        kind: CardKind::Piece,
        space: target.space.clone(),
        piece: Some(key),
        revision: view.revision,
        title,
        subtitle: format!("{} · {size} · voluntary removal (R 1.4.1)", place.name),
        rows: vec![Row {
            choice: Some(removal.id.clone()),
            owner: removal.owner,
            mark: Mark::Selectable,
            chosen: true,
            head: String::new(),
            kind: String::new(),
            lines,
            remedies: vec![],
            no_remedy: None,
            button: Some(if base && target.count == 1 {
                "Remove the Base".into()
            } else {
                format!("Remove {n} {}", if n == 1 { one } else { many })
            }),
        }],
        stepper: (target.count > 1).then_some(Stepper {
            value: n,
            max: target.removable,
            count: target.count,
        }),
        cancel: true,
        footer: PIECE_FOOTER.into(),
    })
}

/// The popover's root, kept at its anchor on the space by the shared `LabelAnchor`.
#[derive(Component)]
pub(super) struct Root {
    /// The region's leftmost and rightmost top-surface points, as seen when it opened.
    ends: [Vec3; 2],
    card: Entity,
    leader: Entity,
    pub placed: bool,
}

/// The card, for the development seat.
#[derive(Component, Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Shown {
    pub card: Card,
    pub pinned: bool,
}

#[derive(Component)]
pub(super) struct CardNode;

/// The region's top-surface ends on screen: leftmost and rightmost, in world space.
fn region_ends(
    space: &str,
    camera: (&Camera, &GlobalTransform),
    meshes: &Query<(Entity, &Mesh3d, &GlobalTransform)>,
    mesh_assets: &Assets<Mesh>,
    controls: &Query<&Control>,
    parents: &Query<&ChildOf>,
) -> Option<[Vec3; 2]> {
    let mut points = vec![];
    for (entity, mesh, at) in meshes {
        let mut current = Some(entity);
        let mut belongs = false;
        while let Some(e) = current {
            if let Ok(Control {
                action: Action::Space(found),
                ..
            }) = controls.get(e)
            {
                belongs = found == space;
                break;
            }
            current = parents.get(e).ok().map(ChildOf::parent);
        }
        if !belongs {
            continue;
        }
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(vertices)) = mesh_assets
            .get(&mesh.0)
            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
        {
            points.extend(
                vertices
                    .iter()
                    .map(|v| at.transform_point(Vec3::from_array(*v))),
            );
        }
    }
    let top = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    let screen = |p: &Vec3| {
        camera
            .0
            .world_to_viewport(camera.1, *p)
            .map_or(f32::NAN, |s| s.x)
    };
    let surface: Vec<Vec3> = points
        .into_iter()
        .filter(|p| (p.y - top).abs() < 0.001)
        .collect();
    let left = surface
        .iter()
        .min_by(|a, b| screen(a).total_cmp(&screen(b)))?;
    let right = surface
        .iter()
        .max_by(|a, b| screen(a).total_cmp(&screen(b)))?;
    Some([*left, *right])
}

/// The drawing context every popover text shares: the admitted art for emblems.
struct Ink<'a> {
    art: &'a Art,
}

/// Spawns a line of popover text in the card's flow. Faction names become #60's inline
/// emblems by `naming` (story 60), except 26July on a red button; the words keep every
/// name for the seat (`emblems::Plain`).
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    ink: &Ink,
    parent: Entity,
    text: &str,
    naming: Naming,
    red: bool,
    font: &Handle<Font>,
    size: f32,
    color: Color,
    grow: bool,
) -> Entity {
    let font = TextFont {
        font: font.clone().into(),
        font_size: size.into(),
        ..default()
    };
    let segments = emblems::segments(text, naming, red);
    let (first, rest) = match segments.split_first() {
        Some((emblems::Segment::Words(words), rest)) => (words.clone(), rest),
        _ => (String::new(), &segments[..]),
    };
    let root = commands
        .spawn((
            Text::new(first),
            emblems::Plain(text.into()),
            font.clone(),
            TextColor(color),
            Node {
                flex_grow: if grow { 1. } else { 0. },
                flex_shrink: if grow { 1. } else { 0. },
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(parent),
        ))
        .id();
    emblems::spawn_rest(
        commands,
        ink.art,
        root,
        rest,
        &font,
        TextColor(color),
        size * emblems::HEIGHT,
    );
    root
}

fn rule(commands: &mut Commands, parent: Entity, height: f32, color: Color) {
    commands.spawn((
        Node {
            height: px(height),
            flex_shrink: 0.,
            ..default()
        },
        BackgroundColor(color),
        Pickable::IGNORE,
        ChildOf(parent),
    ));
}

/// One of the four row state marks: the layout's own 10 px marks, exported from
/// FigCraft and admitted with the highlights (`highlights.json` "marks"), so the "not
/// yet" ring is dashed as designed.
fn mark(commands: &mut Commands, art: &Art, parent: Entity, mark: Mark, s: f32) {
    let file = match mark {
        Mark::Selectable => "atlas-popover-mark-selectable.png",
        Mark::NotYet => "atlas-popover-mark-not-yet.png",
        Mark::Unavailable => "atlas-popover-mark-unavailable.png",
        Mark::Done => "atlas-popover-mark-done.png",
    };
    commands.spawn((
        ImageNode::new(art.images[file].clone()),
        Node {
            width: px(10. * s),
            height: px(10. * s),
            flex_shrink: 0.,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(parent),
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn(
    commands: &mut Commands,
    art: &Art,
    bold: &Handle<Font>,
    card: &Card,
    pinned: bool,
    ends: [Vec3; 2],
    s: f32,
) {
    let regular = art.font.clone();
    let ink = Ink { art };
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            LabelAnchor::point(ends[1]).left_edge().within_viewport(),
            GlobalZIndex(8),
            Pickable::IGNORE,
            Shown {
                card: card.clone(),
                pinned,
            },
        ))
        .id();
    let image = |key: &str| ImageNode::new(art.images[key].clone());
    commands.spawn((
        image("atlas-status-anchor.png"),
        Node {
            position_type: PositionType::Absolute,
            left: px(-frame::DOT / 2. * s),
            top: px(-frame::DOT / 2. * s),
            width: px(frame::DOT * s),
            height: px(frame::DOT * s),
            ..default()
        },
        ZIndex(1),
        Pickable::IGNORE,
        ChildOf(root),
    ));
    let leader = commands
        .spawn((
            image("atlas-status-leader.png"),
            Node {
                position_type: PositionType::Absolute,
                left: px(0.),
                top: px(-s),
                width: px(frame::LEADER * s),
                height: px(2. * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();
    let width = if card.kind == CardKind::Protected {
        frame::TAG_WIDTH
    } else {
        frame::WIDTH
    };
    let body = commands
        .spawn((
            CardNode,
            Node {
                position_type: PositionType::Absolute,
                left: px(frame::LEADER * s),
                top: px(0.),
                width: px(width * s),
                flex_direction: FlexDirection::Column,
                padding: UiRect::bottom(px(10. * s)),
                border: UiRect::all(px(s)),
                border_radius: BorderRadius::all(px(2. * s)),
                ..default()
            },
            BackgroundColor(PAPER),
            BorderColor::all(INK),
            BoxShadow::new(INK.with_alpha(0.2), px(0.), px(3. * s), px(0.), px(10. * s)),
            // Opaque to the pointer, so a click on the card never reaches the map.
            ChildOf(root),
        ))
        .id();
    // A-P6's tag carries an ink rule: it informs, and nothing on it acts.
    rule(
        commands,
        body,
        3. * s,
        if card.kind == CardKind::Protected {
            INK
        } else {
            PRIMARY
        },
    );
    let header = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(2. * s),
                padding: UiRect::new(px(14. * s), px(14. * s), px(9. * s), px(8. * s)),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(body),
        ))
        .id();
    // A title introduces its subject, so it keeps faction names (#60).
    words(
        commands,
        &ink,
        header,
        &card.title,
        Naming::Names,
        false,
        bold,
        15. * s,
        INK,
        false,
    );
    words(
        commands,
        &ink,
        header,
        &card.subtitle,
        Naming::Emblems,
        false,
        &regular,
        11. * s,
        INK_SECONDARY,
        false,
    );
    match card.kind {
        CardKind::Space => space_rows(commands, &ink, body, card, bold, s),
        CardKind::Piece | CardKind::Protected => piece_body(commands, &ink, body, card, bold, s),
    }
    let footer = commands
        .spawn((
            Node {
                padding: UiRect::new(px(14. * s), px(14. * s), px(6. * s), px(0.)),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(body),
        ))
        .id();
    words(
        commands,
        &ink,
        footer,
        &card.footer,
        Naming::Emblems,
        false,
        &regular,
        11. * s,
        INK_SECONDARY,
        false,
    );
    commands.entity(root).insert(Root {
        ends,
        card: body,
        leader,
        placed: false,
    });
}

/// The preview or reason lines of a row, in the layout's tones.
fn lines(
    commands: &mut Commands,
    ink: &Ink,
    parent: Entity,
    lines: &[(Tone, String)],
    done: bool,
    bold: &Handle<Font>,
    s: f32,
) {
    let regular = ink.art.font.clone();
    for (tone, line) in lines {
        let (font, size, color) = match tone {
            Tone::Plain => (&regular, 12.5, if done { INK_SECONDARY } else { INK }),
            Tone::Strong => (bold, 12.5, INK),
            Tone::Quiet => (&regular, 11., INK_SECONDARY),
            Tone::Secondary => (&regular, 12.5, INK_SECONDARY),
        };
        words(
            commands,
            ink,
            parent,
            line,
            Naming::Emblems,
            false,
            font,
            size * s,
            color,
            false,
        );
    }
}

/// The committing button: `action/primary`, Bold 14 in `paper`. It submits its choice
/// as the panel's button does. On red, 26July keeps its name (#60).
#[allow(clippy::too_many_arguments)]
fn commit(
    commands: &mut Commands,
    ink: &Ink,
    parent: Entity,
    label: &str,
    owner: Faction,
    revision: u64,
    choice: &str,
    grow: bool,
    bold: &Handle<Font>,
    s: f32,
) {
    let button = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                flex_grow: if grow { 1. } else { 0. },
                padding: UiRect::new(px(12. * s), px(12. * s), px(7. * s), px(7. * s)),
                border_radius: BorderRadius::all(px(2. * s)),
                ..default()
            },
            BackgroundColor(PRIMARY),
            Control {
                name: "popover:commit".into(),
                choice: false,
                action: Action::PopoverCommit(owner, revision, choice.into()),
            },
            ChildOf(parent),
        ))
        .observe(click)
        .id();
    words(
        commands,
        ink,
        button,
        label,
        Naming::Emblems,
        true,
        bold,
        14. * s,
        PAPER,
        false,
    );
}

/// A-P1/A-P3/A-P4: one row per action in the space.
fn space_rows(
    commands: &mut Commands,
    ink: &Ink,
    body: Entity,
    card: &Card,
    bold: &Handle<Font>,
    s: f32,
) {
    let regular = ink.art.font.clone();
    for row in &card.rows {
        rule(commands, body, s, RULE);
        let node = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(if row.chosen { 6. } else { 5. } * s),
                    padding: UiRect::new(
                        px(14. * s),
                        px(14. * s),
                        px(8. * s),
                        px(if row.chosen { 10. } else { 8. } * s),
                    ),
                    ..default()
                },
                BackgroundColor(if row.chosen { DESK } else { Color::NONE }),
                ChildOf(body),
            ))
            .id();
        let action = row
            .choice
            .as_ref()
            .and_then(|c| c.split_once(':'))
            .map(|(_, a)| a.to_owned());
        match &action {
            // Choosing a selectable row only shows its preview.
            Some(action) if row.mark == Mark::Selectable && !row.chosen => {
                commands
                    .entity(node)
                    .insert(Control {
                        name: format!("popover:row:{action}"),
                        choice: false,
                        action: Action::Row(action.clone()),
                    })
                    .observe(click);
            }
            _ => {
                commands.entity(node).insert(Pickable::IGNORE);
            }
        }
        let head = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8. * s),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(node),
            ))
            .id();
        mark(commands, ink.art, head, row.mark, s);
        let quiet_head = matches!(row.mark, Mark::Unavailable | Mark::Done);
        words(
            commands,
            ink,
            head,
            &row.head,
            Naming::Emblems,
            false,
            bold,
            13. * s,
            if quiet_head { INK_SECONDARY } else { INK },
            true,
        );
        words(
            commands,
            ink,
            head,
            &row.kind,
            Naming::Emblems,
            false,
            &regular,
            11. * s,
            if row.mark == Mark::NotYet {
                PRIMARY
            } else {
                INK_SECONDARY
            },
            false,
        );
        let parent = if row.chosen {
            commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4. * s),
                        padding: UiRect::left(px(18. * s)),
                        ..default()
                    },
                    Pickable::IGNORE,
                    ChildOf(node),
                ))
                .id()
        } else {
            node
        };
        lines(
            commands,
            ink,
            parent,
            &row.lines,
            row.mark == Mark::Done,
            bold,
            s,
        );
        if !row.remedies.is_empty() || row.no_remedy.is_some() {
            let remedies = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(6. * s),
                        row_gap: px(5. * s),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                    ChildOf(node),
                ))
                .id();
            let lead = row.no_remedy.as_deref().unwrap_or("In this task:");
            words(
                commands,
                ink,
                remedies,
                lead,
                Naming::Emblems,
                false,
                &regular,
                11. * s,
                INK_SECONDARY,
                false,
            );
            for remedy in &row.remedies {
                let chip = commands
                    .spawn((
                        Node {
                            padding: UiRect::new(px(7. * s), px(7. * s), px(3. * s), px(3. * s)),
                            border: UiRect::all(px(s)),
                            border_radius: BorderRadius::all(px(2. * s)),
                            ..default()
                        },
                        BorderColor::all(PRIMARY),
                        Pickable::IGNORE,
                        ChildOf(remedies),
                    ))
                    .id();
                words(
                    commands,
                    ink,
                    chip,
                    remedy,
                    Naming::Emblems,
                    false,
                    &regular,
                    12. * s,
                    PRIMARY,
                    false,
                );
            }
        }
        if let (Some(label), Some(choice)) = (&row.button, &row.choice) {
            commit(
                commands,
                ink,
                node,
                label,
                row.owner,
                card.revision,
                choice,
                false,
                bold,
                s,
            );
        }
    }
}

/// A-P5/A-P7: the stack's quantity, the removal's consequences, Remove and Cancel;
/// A-P6: the protected Guerrilla's reason. One divider above the body.
fn piece_body(
    commands: &mut Commands,
    ink: &Ink,
    body: Entity,
    card: &Card,
    bold: &Handle<Font>,
    s: f32,
) {
    let regular = ink.art.font.clone();
    rule(commands, body, s, RULE);
    let node = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6. * s),
                padding: UiRect::new(px(14. * s), px(14. * s), px(8. * s), px(10. * s)),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(body),
        ))
        .id();
    if let Some(stepper) = card.stepper {
        let head = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(8. * s),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(node),
            ))
            .id();
        words(
            commands,
            ink,
            head,
            "Remove",
            Naming::Emblems,
            false,
            bold,
            13. * s,
            INK,
            false,
        );
        // 80 × 26, radius 2, `paper` with a 1 px `ink` stroke: −, the value in a
        // `rule`-boxed cell, +. A step past the bounds is drawn in `rule` and inert.
        let frame = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(s)),
                    border_radius: BorderRadius::all(px(2. * s)),
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderColor::all(INK),
                Pickable::IGNORE,
                ChildOf(head),
            ))
            .id();
        for (glyph, width, step) in [("−", 24., Some(-1)), ("", 30., None), ("+", 24., Some(1))] {
            let enabled = match step {
                Some(-1) => stepper.value > 1,
                Some(_) => stepper.value < stepper.max,
                None => true,
            };
            let cell = commands
                .spawn((
                    Node {
                        width: px(width * s),
                        height: px(24. * s),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: if step.is_none() {
                            UiRect::all(px(s))
                        } else {
                            UiRect::ZERO
                        },
                        ..default()
                    },
                    BorderColor::all(RULE),
                    ChildOf(frame),
                ))
                .id();
            match step {
                Some(step) if enabled => {
                    commands
                        .entity(cell)
                        .insert(Control {
                            name: format!(
                                "popover:quantity:{}",
                                if step < 0 { "minus" } else { "plus" }
                            ),
                            choice: false,
                            action: Action::Quantity(step),
                        })
                        .observe(click);
                }
                _ => {
                    commands.entity(cell).insert(Pickable::IGNORE);
                }
            }
            let text = if step.is_none() {
                stepper.value.to_string()
            } else {
                glyph.to_owned()
            };
            words(
                commands,
                ink,
                cell,
                &text,
                Naming::Emblems,
                false,
                bold,
                14. * s,
                if enabled { INK } else { RULE },
                false,
            );
        }
        words(
            commands,
            ink,
            head,
            &format!("of {}", stepper.count),
            Naming::Emblems,
            false,
            &regular,
            13. * s,
            INK,
            false,
        );
    }
    let Some(row) = card.rows.first() else {
        return;
    };
    let preview = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4. * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(node),
        ))
        .id();
    lines(commands, ink, preview, &row.lines, false, bold, s);
    let (Some(label), Some(choice)) = (&row.button, &row.choice) else {
        return;
    };
    let actions = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(8. * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(node),
        ))
        .id();
    commit(
        commands,
        ink,
        actions,
        label,
        row.owner,
        card.revision,
        choice,
        true,
        bold,
        s,
    );
    if card.cancel {
        let cancel = commands
            .spawn((
                Node {
                    padding: UiRect::new(px(12. * s), px(12. * s), px(7. * s), px(7. * s)),
                    border: UiRect::all(px(s)),
                    border_radius: BorderRadius::all(px(2. * s)),
                    ..default()
                },
                BorderColor::all(INK_SECONDARY),
                Control {
                    name: "popover:cancel".into(),
                    choice: false,
                    action: Action::Dismiss,
                },
                ChildOf(actions),
            ))
            .observe(click)
            .id();
        words(
            commands,
            ink,
            cancel,
            "Cancel",
            Naming::Emblems,
            false,
            &regular,
            14. * s,
            INK,
            false,
        );
    }
}

/// Which 26July group the pointer is over, for the piece popover's hover (#73). The
/// nearest hit decides, so a piece in front of its region wins.
pub(super) fn hover_piece(
    hits: Res<HoverMap>,
    pieces: Query<&OwnPiece>,
    parents: Query<&ChildOf>,
    counts: Query<&LabelAnchor, With<Count>>,
    mut popover: ResMut<Popover>,
) {
    let hovered = hits.get(&PointerId::Mouse).and_then(|hits| {
        let (entity, _) = hits.iter().min_by(|a, b| a.1.depth.total_cmp(&b.1.depth))?;
        // A piece's count stands for its piece.
        let entity = match counts.get(*entity) {
            Ok(LabelAnchor {
                to: unfolded_tabletop::label_anchor::Anchor::Entity { entity, .. },
                ..
            }) => *entity,
            _ => *entity,
        };
        let mut current = Some(entity);
        while let Some(e) = current {
            if let Ok(piece) = pieces.get(e) {
                return Some(piece.key.clone());
            }
            current = parents.get(e).ok().map(ChildOf::parent);
        }
        None
    });
    if popover.hovered != hovered {
        popover.hovered = hovered;
    }
}

/// What anchors a popover: the regions' meshes and controls for a space, and the
/// 26July tokens for a piece.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Anchors<'w, 's> {
    meshes: Query<'w, 's, (Entity, &'static Mesh3d, &'static GlobalTransform)>,
    mesh_assets: Res<'w, Assets<Mesh>>,
    controls: Query<'w, 's, &'static Control>,
    parents: Query<'w, 's, &'static ChildOf>,
    pieces: Query<'w, 's, (&'static OwnPiece, &'static GlobalTransform)>,
}

/// Opens, replaces or closes the popover from the targeting state, the pinned space
/// and the region the pointer rests on.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    art: Res<Art>,
    ui: Res<Interface>,
    targeting: Res<Targeting>,
    mut popover: ResMut<Popover>,
    bold: Res<Bold>,
    fonts: Res<Assets<Font>>,
    mut context: ResMut<bevy::text::FontCx>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    anchors: Anchors,
    roots: Query<Entity, With<Root>>,
    mut previous: Local<Option<(Card, bool, f32)>>,
) {
    let Some(view) = &targeting.view else {
        return;
    };
    // Removal mode (#73): the piece popover replaces the space popover.
    let removable = |key: &str| {
        targeting
            .piece(key)
            .is_some_and(|t| t.mark == PieceState::Removable)
    };
    if popover
        .piece
        .as_ref()
        .is_some_and(|(key, revision)| *revision != view.revision || !removable(key))
    {
        popover.piece = None;
    }
    let offered = |space: &str| {
        matches!(
            targeting.state(space),
            Some(SpaceState::Selectable | SpaceState::Blocked)
        )
    };
    if popover
        .pinned
        .as_ref()
        .is_some_and(|(space, revision)| *revision != view.revision || !offered(space))
    {
        popover.pinned = None;
        popover.selected = None;
    }
    let (s, _) = layout(&window);
    let pinned = popover.pinned.as_ref().map(|(space, _)| space.clone());
    let space = if ui.reader {
        None
    } else {
        pinned
            .clone()
            // A click that closed it keeps it closed until the pointer moves.
            .or_else(|| {
                ui.status
                    .visible()
                    .filter(|space| ui.rested.is_none() && offered(space))
                    .cloned()
            })
    };
    let shown = if targeting.removal.is_some() {
        let kept = popover.piece.as_ref().map(|(key, _)| key.clone());
        let key = if ui.reader {
            None
        } else {
            kept.clone().or_else(|| {
                popover
                    .hovered
                    .clone()
                    .filter(|_| ui.status.visible().is_some() && ui.rested.is_none())
            })
        };
        key.and_then(|key| {
            let quantity = if kept.as_ref() == Some(&key) {
                popover.quantity
            } else {
                1
            };
            piece_content(view, targeting.piece(&key)?, quantity)
                .map(|card| (card, kept.is_some(), s))
        })
    } else {
        space.and_then(|space| {
            content(view, &space, popover.selected.as_deref())
                .map(|card| (card, pinned.is_some(), s))
        })
    };
    let bold_ready = fonts
        .get(&bold.0)
        .is_some_and(|font| unfolded_tabletop::loading::font_registered(font, &mut context));
    if *previous == shown || !bold_ready {
        return;
    }
    let ends = match &shown {
        // A piece popover is anchored beside its piece.
        Some((
            Card {
                piece: Some(key), ..
            },
            ..,
        )) => {
            let Some(at) = anchors
                .pieces
                .iter()
                .find(|(p, _)| &p.key == key)
                .map(|(_, at)| at.translation())
            else {
                return;
            };
            Some([at - Vec3::X * 0.16, at + Vec3::X * 0.16])
        }
        Some((card, ..)) => {
            // Retried next frame until the region's mesh is loaded.
            let Some(ends) = region_ends(
                &card.space,
                *camera,
                &anchors.meshes,
                &anchors.mesh_assets,
                &anchors.controls,
                &anchors.parents,
            ) else {
                return;
            };
            Some(ends)
        }
        None => None,
    };
    for root in &roots {
        commands.entity(root).despawn();
    }
    previous.clone_from(&shown);
    if let (Some((card, pinned, s)), Some(ends)) = (shown, ends) {
        spawn(&mut commands, &art, &bold.0, &card, pinned, ends, s);
    }
}

/// Puts the card beside its space: to the right of the region if it fits before the
/// Operation panel, else to its left, and vertically inside the map. The shared
/// `LabelAnchor` keeps the root on the region's end as the camera moves.
#[allow(clippy::type_complexity)]
pub(super) fn place(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut roots: Query<(&mut Root, &mut LabelAnchor)>,
    mut nodes: Query<(&mut Node, &ComputedNode)>,
) {
    let (camera, camera_at) = *camera;
    let (s, _) = layout(&window);
    let Some(map) = camera.logical_viewport_rect() else {
        return;
    };
    let panel_left = window.width() - 540. * s;
    let margin = frame::MARGIN * s;
    for (mut root, mut anchor) in &mut roots {
        let Ok((_, card)) = nodes.get(root.card) else {
            continue;
        };
        let width = frame::LEADER * s + card.size().x * card.inverse_scale_factor;
        let height = card.size().y * card.inverse_scale_factor;
        if height <= 0. {
            continue;
        }
        let [left_end, right_end] = root.ends;
        let project = |p: Vec3| camera.world_to_viewport(camera_at, p).ok();
        let (Some(left_at), Some(right_at)) = (project(left_end), project(right_end)) else {
            continue;
        };
        let right = right_at.x + width + margin <= panel_left.min(window.width());
        let fits_left = left_at.x - width - margin >= map.min.x;
        let (to, at, sign) = if right || !fits_left {
            (right_end, right_at, 1.)
        } else {
            (left_end, left_at, -1.)
        };
        // Centered on the anchor where the map allows, else inside its top or bottom.
        let top = (at.y - height / 2.)
            .min(map.max.y - margin - height)
            .max(map.min.y + margin)
            - at.y;
        let card_left = if sign > 0. { frame::LEADER * s } else { -width };
        let leader_left = if sign > 0. { 0. } else { -frame::LEADER * s };
        let wanted = LabelAnchor::point(to).left_edge().within_viewport();
        let mut changed = *anchor != wanted;
        if changed {
            *anchor = wanted;
        }
        // UI layout snaps the card to whole physical pixels, so its measured height
        // can alternate by a pixel as its top moves by half of one (#72's Oriente card
        // at 3840×2160 never settled). Moving it only for a change beyond a pixel lets
        // it rest.
        let off =
            |value: Val, wanted: f32| !matches!(value, Val::Px(at) if (at - wanted).abs() <= 1.);
        for (entity, left, top) in [(root.card, card_left, top), (root.leader, leader_left, -s)] {
            if let Ok((mut node, _)) = nodes.get_mut(entity)
                && (off(node.left, left) || off(node.top, top))
            {
                node.left = px(left);
                node.top = px(top);
                changed = true;
            }
        }
        root.placed = !changed;
    }
}

fn bounds(node: &ComputedNode, at: &UiGlobalTransform) -> [f32; 4] {
    let center = at.translation * node.inverse_scale_factor;
    let half = node.size() * node.inverse_scale_factor / 2.;
    [
        center.x - half.x,
        center.y - half.y,
        center.x + half.x,
        center.y + half.y,
    ]
}

/// The shown popover for the development seat: its card, whether it is kept open,
/// the card's bounds and the anchor dot's centre, in logical window pixels.
pub(super) fn observe(world: &mut World) -> Option<serde_json::Value> {
    let (shown, root, anchor) = world
        .query::<(&Shown, &Root, &Node)>()
        .iter(world)
        .next()
        .map(|(shown, root, node)| (shown.clone(), root.card, (node.left, node.top)))?;
    let card = world
        .query::<(&ComputedNode, &UiGlobalTransform)>()
        .get(world, root)
        .ok()
        .map(|(node, at)| bounds(node, at));
    let anchor = match anchor {
        (Val::Px(x), Val::Px(y)) => Some([x, y]),
        _ => None,
    };
    Some(serde_json::json!({
        "space": shown.card.space,
        "piece": shown.card.piece,
        "kind": shown.card.kind,
        "pinned": shown.pinned,
        "card": shown.card,
        "bounds": card,
        "anchor": anchor,
    }))
}

/// A kept-open popover not yet drawn, or a drawn one not yet beside its space.
pub(super) fn unsettled(world: &mut World) -> Vec<String> {
    let popover = world.resource::<Popover>();
    let pinned = (popover.pinned.is_some() || popover.piece.is_some())
        && !world.resource::<Interface>().reader;
    let roots: Vec<bool> = world
        .query::<&Root>()
        .iter(world)
        .map(|r| r.placed)
        .collect();
    if (pinned && roots.is_empty()) || roots.iter().any(|placed| !placed) {
        vec!["space popover placement".into()]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choose(game: &mut Game, action: &str) {
        let view = game.observe();
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        game.submit(owner, view.revision, &id).unwrap();
    }

    fn card(game: &Game, space: &str, selected: Option<&str>) -> Card {
        content(&game.observe(), space, selected).unwrap()
    }

    fn text(row: &Row) -> String {
        row.lines
            .iter()
            .map(|(_, l)| l.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn santiago_at_0_resources_previews_opening_kidnap_and_states_terror_s_remedies() {
        let card = card(&Game::prepared(), "santiago", None);
        assert_eq!(card.title, "Santiago de Cuba");
        assert_eq!(card.rows.len(), 2);
        let kidnap = &card.rows[0];
        assert_eq!(kidnap.choice.as_deref(), Some("0:kidnap"));
        assert!(kidnap.chosen);
        assert_eq!(
            kidnap.button.as_deref(),
            Some("Open Kidnap · nothing rolled yet")
        );
        let lines = text(kidnap);
        for fact in [
            "2 Guerrillas (Active + Underground) vs 1 Police; must outnumber.",
            "The roll takes 1–6 Resources from Government (has 6); at least 1. R 4.3.3",
            "Opening rolls nothing",
            "accepted commitment policy",
        ] {
            assert!(lines.contains(fact), "{fact}: {lines}");
        }
        let terror = &card.rows[1];
        assert_eq!(terror.mark, Mark::NotYet);
        assert_eq!(
            text(terror),
            "Cost: 1 Resource per space (26July has 0). R 3.3.4"
        );
        assert_eq!(
            terror.remedies,
            ["Kidnap here first", "Request a Resource transfer"]
        );
        assert!(terror.button.is_none());
    }

    #[test]
    fn a_funded_santiago_terror_previews_its_cost_flip_shift_and_standing() {
        let mut game = Game::prepared();
        for step in ["transfer", "transfer:3:1", "amount:1", "authorize"] {
            choose(&mut game, step);
        }
        // Both are selectable: Terror is listed first in the authority's order.
        let first = card(&game, "santiago", None);
        assert_eq!(first.rows[0].head, "Terror");
        let terror = card(&game, "santiago", Some("terror:santiago"));
        let row = terror.rows.iter().find(|r| r.chosen).unwrap();
        let lines = text(row);
        // Handoff arithmetic: 26July 1 → 0; Santiago Neutral → Passive Opposition,
        // Pop 1, so 26July 7 → 8 and margin −8 → −7.
        for fact in [
            "Pay 1 Resource: 26July 1 → 0.",
            "Flip 1 Underground Guerrilla there to Active.",
            "Place a Terror marker; Neutral → Passive Opposition: 26July 7 → 8 (margin -8 → -7).",
            "R 3.3.4",
        ] {
            assert!(lines.contains(fact), "{fact}: {lines}");
        }
        assert_eq!(
            row.button.as_deref(),
            Some("Terror in Santiago de Cuba · pay 1 Resource")
        );
        // Choosing Kidnap instead shows its stakes and keeps Terror one click away.
        let kidnap = card(&game, "santiago", Some("kidnap"));
        assert!(
            kidnap
                .rows
                .iter()
                .any(|r| r.chosen && r.head.starts_with("Kidnap"))
        );
        assert!(kidnap.rows.iter().any(|r| !r.chosen && r.head == "Terror"));
    }

    #[test]
    fn after_a_removal_santiago_s_kidnap_has_no_remedy() {
        let mut game = Game::prepared();
        for step in ["remove", "remove:santiago:underground:1", "done-removal"] {
            choose(&mut game, step);
        }
        let card = card(&game, "santiago", None);
        let kidnap = card.rows.iter().find(|r| r.head == "Kidnap").unwrap();
        assert_eq!(kidnap.mark, Mark::Unavailable);
        assert_eq!(kidnap.no_remedy.as_deref(), Some(NO_REMEDY));
        let terror = card.rows.iter().find(|r| r.head == "Terror").unwrap();
        assert_eq!(terror.remedies, ["Request a Resource transfer"]);
    }

    #[test]
    fn a_rolled_kidnap_stays_as_history_beside_the_owed_terror() {
        let mut game = Game::prepared();
        choose(&mut game, "kidnap");
        choose(&mut game, "kidnap:roll");
        let view = game.observe();
        let kidnap = view.kidnap.clone().unwrap();
        let card = content(&view, "santiago", None).unwrap();
        assert!(
            card.subtitle.ends_with("Santiago de Cuba Terror is owed"),
            "{}",
            card.subtitle
        );
        assert_eq!(card.rows[0].head, "Terror");
        let history = card.rows.last().unwrap();
        assert_eq!(history.mark, Mark::Done);
        assert_eq!(history.head, format!("Kidnap · rolled {}", kidnap.die));
        assert_eq!(
            text(history),
            format!(
                "Government 6 → {} · 26July 0 → {}. The result stays.",
                6 - kidnap.proceeds,
                kidnap.proceeds
            )
        );
    }

    fn piece(game: &Game, key: &str, quantity: u8) -> Card {
        let view = game.observe();
        let removal = crate::targets::removal_targets(&view).unwrap();
        let target = removal
            .pieces
            .iter()
            .find(|p| format!("{}:{}", p.space, p.group) == key)
            .unwrap();
        piece_content(&view, target, quantity).unwrap()
    }

    /// A-P5 before Kidnap: a stack of 2 with nothing owed offers 1 or 2; each quantity
    /// previews its own removal and acts through the authority's choice for it.
    #[test]
    fn a_santiago_stack_offers_each_quantity_with_its_own_preview() {
        let mut game = Game::prepared();
        choose(&mut game, "remove");
        let one = piece(&game, "santiago:underground", 1);
        assert_eq!(one.kind, CardKind::Piece);
        assert_eq!(one.title, "26July Guerrilla · Underground");
        assert_eq!(
            one.subtitle,
            "Santiago de Cuba · stack of 2 · voluntary removal (R 1.4.1)"
        );
        assert_eq!(
            one.stepper,
            Some(Stepper {
                value: 1,
                max: 2,
                count: 2
            })
        );
        let row = &one.rows[0];
        assert_eq!(
            row.choice.as_deref(),
            Some("1:remove:santiago:underground:1")
        );
        assert_eq!(row.button.as_deref(), Some("Remove 1 Guerrilla"));
        let lines = text(row);
        for fact in [
            "1 Underground Guerrilla returns to Available.",
            "Control: unchanged (Government).",
            "Then unavailable: Kidnap · Santiago · Government, Back to the action choice.",
            "Standings: unchanged.",
            "R 1.4.1",
        ] {
            assert!(lines.contains(fact), "{fact}: {lines}");
        }
        assert!(one.cancel);
        assert_eq!(one.footer, PIECE_FOOTER);
        let two = piece(&game, "santiago:underground", 2);
        assert_eq!(
            two.rows[0].choice.as_deref(),
            Some("1:remove:santiago:underground:2")
        );
        assert_eq!(two.rows[0].button.as_deref(), Some("Remove 2 Guerrillas"));
        assert!(text(&two.rows[0]).contains("2 Underground Guerrillas return to Available."));
        // A quantity past the bound shows the most allowed.
        assert_eq!(piece(&game, "santiago:underground", 9), two);
    }

    /// A-P5 after the roll: at most 1, with the owed Terror's reason; then A-P6.
    #[test]
    fn the_owed_terror_limits_the_stack_then_protects_the_last_guerrilla() {
        let mut game = Game::prepared();
        choose(&mut game, "kidnap");
        choose(&mut game, "kidnap:roll");
        choose(&mut game, "remove");
        let card = piece(&game, "santiago:underground", 2);
        assert_eq!(card.stepper.map(|s| (s.value, s.max)), Some((1, 1)));
        assert_eq!(
            card.rows[0].lines[0],
            (
                Tone::Secondary,
                "At most 1: the other is protected while Santiago Terror is owed (accepted commitment policy, local).".into()
            )
        );
        choose(&mut game, "remove:santiago:underground:1");
        let tag = piece(&game, "santiago:underground", 1);
        assert_eq!(tag.kind, CardKind::Protected);
        assert_eq!(tag.title, "Protected · 26July Guerrilla");
        assert_eq!(
            tag.subtitle,
            "Santiago de Cuba · Underground · the last one"
        );
        assert_eq!(tag.rows[0].button, None);
        assert_eq!(
            text(&tag.rows[0]),
            "Santiago Terror is owed. Under the accepted commitment policy (local, not a publisher ruling), Santiago de Cuba's last Underground Guerrilla stays until that Terror resolves. It becomes removable once Terror in Santiago de Cuba has resolved."
        );
        assert_eq!(tag.footer, PROTECTED_FOOTER);
        assert!(!tag.cancel);
    }

    /// A-P7: the Base, with Control and its standing (7 → 6, margin −8 → −9).
    #[test]
    fn the_base_previews_control_and_its_standing() {
        let mut game = Game::prepared();
        choose(&mut game, "remove");
        let card = piece(&game, "sierra-maestra:base", 1);
        assert_eq!(card.title, "26July Base");
        assert_eq!(
            card.subtitle,
            "Sierra Maestra · 1 Base · voluntary removal (R 1.4.1)"
        );
        assert_eq!(card.stepper, None);
        assert_eq!(card.rows[0].button.as_deref(), Some("Remove the Base"));
        let lines = text(&card.rows[0]);
        for fact in [
            "The Base returns to Available.",
            "Control of Sierra Maestra: 26July → none.",
            "26July 7 → 6 (margin -8 → -9): its Bases count toward its score.",
        ] {
            assert!(lines.contains(fact), "{fact}: {lines}");
        }
    }

    #[test]
    fn spaces_without_an_action_have_no_popover() {
        let view = Game::prepared().observe();
        assert!(content(&view, "sierra-maestra", None).is_none());
        assert!(content(&Game::opening().observe(), "santiago", None).is_none());
    }
}
