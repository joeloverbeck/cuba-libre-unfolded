//! 26July's action choice in the selected AC-B design, "Briefing tag" (#68, built in
//! #69): a fixed list of every option, a briefing tag beside the panel that opens on
//! hover or keyboard focus and stays pinned when clicked, a commit at the panel's foot
//! for the pinned option, and during Terror a BRIEFING ribbon that reopens the Terror
//! and Kidnap briefings.
//!
//! Layout authority (foundation 4): `art/atlas/atlas-action-choice.figcraft` owns the
//! structure, placement and sizes. The constants in [`fig`] are that file's 16:9 frame
//! coordinates (6144×3456, the 1920×1280 reference ×2.7), copied from its nodes, and
//! `atlas-action-choice.layout.json` records the same nodes for the native comparison.
//! Every drawn plate is an admitted FigCraft or VectorCraft export (`choice.sha256`);
//! the runtime only places them and the text.
//!
//! The game owns every fact. Pinning, focus and the ribbon are presentation state: they
//! change nothing in the match and are not saved. Only the commit submits a decision,
//! through the same checked seam as every other control.
use super::briefing::{self, Briefing};
use super::emblems::{self, Naming};
use super::top::{INK, QUIET};
use super::*;
use crate::{ActionChoice, Observation};
use bevy::text::{Justify, LetterSpacing, LineHeight};

/// Node coordinates in the FigCraft 16:9 frame, `art/atlas/atlas-action-choice.figcraft`.
mod fig {
    /// Frame units per reference pixel.
    pub const SCALE: f32 = 2.7;
    /// The operation panel's top-left; the reference panel sits at 1380, 280.
    pub const PANEL: [f32; 2] = [4686., 756.];
    pub const EMBLEM: [f32; 4] = [4750.8, 821.2, 59.4, 59.4];
    pub const HEADING: [f32; 3] = [4831.8, 815.4, 1139.4];
    pub const HEADING_SIZE: f32 = 59.4;
    pub const SUBTITLE: [f32; 3] = [4750.8, 897.2, 1220.4];
    pub const SUBTITLE_SIZE: f32 = 35.1;
    pub const SUBTITLE_LINE: f32 = 42.;
    /// Section labels: `section:event` and `section:operations`.
    pub const SECTIONS: [(&str, f32, &str); 2] = [
        ("section:event", 1110.8, "EVENT · CARD 17, ALBERTO BAYO"),
        (
            "section:operations",
            1350.,
            "OPERATIONS · SPECIAL ACTIVITIES BENEATH",
        ),
    ];
    pub const CAPS_SIZE: f32 = 28.35;
    pub const CAPS_SPACING: f32 = 2.16;
    pub const ROW_X: f32 = 4750.8;
    pub const ROW_W: f32 = 1220.4;
    pub const ROW_H: f32 = 86.4;
    /// Each option row's top, in the game's listing order.
    pub const ROWS: [f32; 11] = [
        1013.6, 1155.6, 1252.8, 1394.8, 1492., 1589.2, 1686.4, 1783.6, 1880.8, 1978., 2075.2,
    ];
    /// A playable option's bold label inside its row, and its size.
    pub const LABEL: [f32; 2] = [40.5, 17.2];
    pub const LABEL_SIZE: f32 = 43.2;
    /// An unavailable option's label; a Special Activity's starts after its connector.
    pub const QUIET_LABEL: [f32; 2] = [37.8, 18.7];
    pub const QUIET_SIZE: f32 = 40.5;
    pub const SA_LABEL_X: f32 = 91.8;
    pub const CONNECTOR: [f32; 4] = [43.2, 43.2, 27., 2.7];
    /// A playable option's value, right-aligned to this edge.
    pub const VALUE_RIGHT: f32 = 1190.7;
    pub const VALUE_TOP: f32 = 21.2;
    pub const VALUE_SIZE: f32 = 36.45;
    pub const PILL_RIGHT: f32 = 1193.4;
    pub const PILL_TOP: f32 = 16.2;
    pub const PILL_H: f32 = 54.;
    pub const PILL_W: f32 = 287.2;
    pub const PILL_WIDE: f32 = 528.2;
    pub const PILL_TEXT: [f32; 2] = [21.6, 11.5];
    pub const PILL_SIZE: f32 = 25.65;
    pub const PILL_SPACING: f32 = 1.62;
    pub const COMMIT: [f32; 4] = [4750.8, 2354.4, 1220.4, 140.4];
    pub const COMMIT_TOP: f32 = 41.2;
    pub const COMMIT_SIZE: f32 = 48.6;
    /// The tag opens 8 reference px left of the panel, top-aligned with it.
    pub const TAG_GAP: f32 = 21.6;
    pub const TAG_W: f32 = 1263.6;
    /// Padding: top, right, bottom, left.
    pub const PAD: [f32; 4] = [43.2, 59.4, 48.6, 70.2];
    pub const GAP: f32 = 27.;
    pub const TITLE_SIZE: f32 = 59.4;
    pub const TITLE_LINE: f32 = 71.;
    pub const RULE_REF_SIZE: f32 = 37.8;
    pub const RULE_REF_TOP: f32 = 13.;
    pub const SEPARATOR: f32 = 2.7;
    pub const ENTRY_VALUE_TOP: f32 = 39.4;
    pub const VALUE_TEXT_SIZE: f32 = 40.5;
    pub const VALUE_LINE: f32 = 54.;
    pub const RULES_SIZE: f32 = 33.75;
    pub const RULES_LINE: f32 = 41.;
    /// The pin through the tag's top-right corner, relative to the tag.
    pub const PIN: [f32; 4] = [1155.6, -16.2, 81., 81.];
    pub const TAB_H: f32 = 70.2;
    pub const TAB_GAP: f32 = 16.2;
    pub const TAB_TEXT: [f32; 2] = [29.7, 14.1];
    pub const TAB_SIZE: f32 = 35.1;
    /// The tabs, left to right: option, label, width.
    pub const TABS: [(&str, &str, f32); 2] = [
        ("terror:kidnap", "Kidnap", 170.4),
        ("terror", "Terror", 154.4),
    ];
    /// The BRIEFING ribbon, relative to the panel.
    pub const RIBBON: [f32; 4] = [1188., 0., 97.2, 302.4];
}

/// A frame x or y in reference pixels, on the panel's right-anchored reference.
fn rx(x: f32) -> f32 {
    1380. + (x - fig::PANEL[0]) / fig::SCALE
}
fn ry(y: f32) -> f32 {
    280. + (y - fig::PANEL[1]) / fig::SCALE
}
fn rs(v: f32) -> f32 {
    v / fig::SCALE
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    [rx(x), ry(y), rs(w), rs(h)]
}

/// The action choice's presentation state, kept in `Interface`.
#[derive(Default)]
pub(super) struct State {
    /// The pinned option, such as "pass": its briefing stays open and the commit reads it.
    pub pinned: Option<String>,
    /// The control that has keyboard focus, by its seat name.
    pub focus: Option<String>,
    /// The briefing tag is open from the BRIEFING ribbon during Terror.
    pub ribbon: bool,
    /// The ribbon's open tab: "terror" (the default) or "terror:kidnap".
    pub tab: Option<String>,
}
impl State {
    /// A decision committed: the action choice's pin and focus no longer apply.
    pub fn decided(&mut self) {
        self.pinned = None;
        self.focus = None;
    }
    fn tab(&self) -> &str {
        self.tab.as_deref().unwrap_or("terror")
    }
    /// Closes the ribbon's tag. A space popover opened on the map wins over the tag
    /// (owner's clash rule, #58); the tag reopens from the ribbon.
    pub fn close(&mut self) {
        self.ribbon = false;
    }
}

/// Names a drawn part by its node name in `atlas-action-choice.figcraft`, so the seat
/// can compare native placement with the layout record.
#[derive(Component)]
pub(super) struct Part(pub String);

/// The briefing tag's and the active row's entities, redrawn by [`draw`].
#[derive(Component)]
pub(super) struct Tag;

/// What the briefing tag shows, as the development seat reads it.
#[derive(Component, Clone, Debug, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    #[serde(flatten)]
    pub briefing: Briefing,
    pub pinned: bool,
    /// The tabs shown during Terror, empty at the action choice.
    pub tabs: Vec<String>,
}

/// How the panel lists one option: its label, and its value or status pill.
fn listing(action: &str) -> (&'static str, Option<&'static str>) {
    match action {
        "pass" => ("Pass", None),
        "event:unshaded" => ("Event · unshaded", Some("NOT IN THIS TASK")),
        "event:shaded" => ("Event · shaded", Some("NOT IN THIS TASK")),
        "rally" => ("Rally", Some("NOT IN THIS TASK")),
        "rally:infiltrate" => ("Infiltrate · with Rally", Some("NOT IN THIS TASK")),
        "march" => ("March", Some("NOT IN THIS TASK")),
        "march:infiltrate" => ("Infiltrate · with March", Some("NOT IN THIS TASK")),
        "attack" => ("Attack", Some("NOT IN THIS TASK")),
        "attack:ambush" => ("Ambush · with Attack", Some("NOT IN THIS TASK")),
        "terror" => ("Terror · Operation", None),
        "terror:kidnap" => (
            "Kidnap · with Terror",
            Some("OFFERED ONCE TERROR IS CHOSEN"),
        ),
        _ => ("", None),
    }
}

/// A choice id's action, without its revision.
fn action(id: &str) -> &str {
    id.split_once(':').map_or(id, |(_, a)| a)
}

fn resources(n: u8) -> String {
    format!("{n} Resource{}", if n == 1 { "" } else { "s" })
}

/// Restyles a drawn text: Lato Bold, letter spacing, line height or alignment.
fn style(
    commands: &mut Commands,
    entity: Entity,
    art: &Art,
    bold: bool,
    size: f32,
    spacing: f32,
    line: Option<f32>,
) {
    let mut e = commands.entity(entity);
    if bold {
        e.insert(TextFont {
            font: art.bold.clone().into(),
            font_size: size.into(),
            ..default()
        });
    }
    if spacing > 0. {
        e.insert(LetterSpacing::Px(spacing));
    }
    if let Some(line) = line {
        e.insert(LineHeight::Px(line));
    }
}

/// The commit at the panel's foot: what it reads and does for the pinned option.
struct Commit {
    label: String,
    action: Action,
    name: String,
    enabled: bool,
}
fn commit(view: &Observation, pinned: Option<&str>) -> Commit {
    let choice = pinned.and_then(|p| view.choices.iter().find(|c| action(&c.id) == p));
    match choice {
        Some(c) if c.supported => Commit {
            label: match action(&c.id) {
                "pass" => "Pass · +1 Resource".into(),
                "terror" => "Choose Terror".into(),
                other => listing(other).0.into(),
            },
            action: Action::Commit(c.owner, view.revision),
            name: c.id.clone(),
            enabled: true,
        },
        Some(c) => {
            let (name, _) = listing(action(&c.id));
            let label = if action(&c.id) == "terror:kidnap" {
                "Kidnap is offered once Terror is chosen".into()
            } else {
                format!("{} is not in this task", name.split(" · ").next().unwrap_or(name))
            };
            Commit {
                label,
                action: Action::Unsupported(c.explanation.clone()),
                name: "choice-commit".into(),
                enabled: false,
            }
        }
        None => Commit {
            label: "Click an option to choose it".into(),
            action: Action::Unsupported(
                "Pin an option first: click it, or focus it with Tab and press Enter. Its briefing stays open and this button commits it."
                    .into(),
            ),
            name: "choice-commit".into(),
            enabled: false,
        },
    }
}

/// The action choice inside the operation panel: heading, every listed option and
/// the commit. `offset` is the right-anchored letterbox offset.
pub(super) fn panel(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    s: f32,
    o: Vec2,
) {
    let part = |commands: &mut Commands, entity: Entity, name: &str| {
        commands.entity(entity).insert(Part(name.into()));
    };
    let [x, y, w, h] = fig::EMBLEM;
    let emblem = picture(
        commands,
        art,
        &emblems::image(Faction::M26),
        rect(x, y, w, h),
        s,
        o,
        3,
    );
    part(commands, emblem, "choice-heading-emblem");
    let [x, y, w] = fig::HEADING;
    let heading = text(
        commands,
        art,
        Prose::named("Choose 26July's action"),
        rx(x),
        ry(y),
        rs(w),
        rs(fig::HEADING_SIZE),
        s,
        o,
        5,
    );
    commands.entity(heading).insert(TextColor(INK));
    part(commands, heading, "choice-heading");
    let [x, y, w] = fig::SUBTITLE;
    let subtitle = text(
        commands,
        art,
        Prose::named(format!(
            "1st Eligible on card 17 · 26July has {}\nHover or focus an option for its briefing · click to pin it",
            resources(view.resources[Faction::M26.index()])
        )),
        rx(x),
        ry(y),
        rs(w),
        rs(fig::SUBTITLE_SIZE),
        s,
        o,
        5,
    );
    commands.entity(subtitle).insert(TextColor(QUIET));
    style(
        commands,
        subtitle,
        art,
        false,
        0.,
        0.,
        Some(rs(fig::SUBTITLE_LINE) * s),
    );
    part(commands, subtitle, "choice-subtitle");
    for (name, top, words) in fig::SECTIONS {
        let label = text(
            commands,
            art,
            Prose::named(words),
            rx(fig::ROW_X),
            ry(top),
            rs(fig::ROW_W),
            rs(fig::CAPS_SIZE),
            s,
            o,
            5,
        );
        commands.entity(label).insert(TextColor(QUIET));
        style(
            commands,
            label,
            art,
            true,
            rs(fig::CAPS_SIZE) * s,
            rs(fig::CAPS_SPACING) * s,
            None,
        );
        part(commands, label, name);
    }
    for (choice, top) in view.choices.iter().zip(fig::ROWS) {
        let act = action(&choice.id);
        let name = format!("option:{act}");
        let (label, pill) = listing(act);
        let row = rect(fig::ROW_X, top, fig::ROW_W, fig::ROW_H);
        // A playable option has the paper plate; the others only a pickable area.
        let node = if choice.supported {
            let plate = picture(commands, art, "atlas-choice-row.png", row, s, o, 3);
            commands.entity(plate).remove::<Pickable>();
            plate
        } else {
            commands
                .spawn((
                    UiRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(o.x + row[0] * s),
                        top: px(o.y + row[1] * s),
                        width: px(row[2] * s),
                        height: px(row[3] * s),
                        ..default()
                    },
                    GlobalZIndex(3),
                ))
                .id()
        };
        commands
            .entity(node)
            .insert((
                Control {
                    name: name.clone(),
                    choice: false,
                    action: Action::Pin(act.into()),
                },
                Part(name.clone()),
            ))
            .observe(super::click);
        let accompanies = choice.accompanies.is_some();
        let (lx, ly, size) = if choice.supported {
            (fig::LABEL[0], fig::LABEL[1], fig::LABEL_SIZE)
        } else if accompanies {
            (fig::SA_LABEL_X, fig::QUIET_LABEL[1], fig::QUIET_SIZE)
        } else {
            (fig::QUIET_LABEL[0], fig::QUIET_LABEL[1], fig::QUIET_SIZE)
        };
        if accompanies {
            let [cx, cy, cw, ch] = fig::CONNECTOR;
            picture(
                commands,
                art,
                "atlas-briefing-rule.png",
                rect(fig::ROW_X + cx, top + cy, cw, ch),
                s,
                o,
                5,
            );
        }
        let words = text(
            commands,
            art,
            Prose::named(label),
            rx(fig::ROW_X + lx),
            ry(top + ly),
            rs(fig::ROW_W - lx),
            rs(size),
            s,
            o,
            5,
        );
        commands
            .entity(words)
            .insert(TextColor(if choice.supported { INK } else { QUIET }));
        style(
            commands,
            words,
            art,
            choice.supported,
            rs(size) * s,
            0.,
            None,
        );
        part(commands, words, &format!("{name}-label"));
        if let Some(pill) = pill {
            let width = if act == "terror:kidnap" {
                fig::PILL_WIDE
            } else {
                fig::PILL_W
            };
            let left = fig::ROW_X + fig::PILL_RIGHT - width;
            let plate = picture(
                commands,
                art,
                if act == "terror:kidnap" {
                    "atlas-choice-pill-wide.png"
                } else {
                    "atlas-choice-pill.png"
                },
                rect(left, top + fig::PILL_TOP, width, fig::PILL_H),
                s,
                o,
                5,
            );
            part(commands, plate, &format!("pill:{act}"));
            let words = text(
                commands,
                art,
                Prose::named(pill),
                rx(left + fig::PILL_TEXT[0]),
                ry(top + fig::PILL_TOP + fig::PILL_TEXT[1]),
                rs(width - fig::PILL_TEXT[0]),
                rs(fig::PILL_SIZE),
                s,
                o,
                6,
            );
            commands.entity(words).insert(TextColor(QUIET));
            style(
                commands,
                words,
                art,
                true,
                rs(fig::PILL_SIZE) * s,
                rs(fig::PILL_SPACING) * s,
                None,
            );
            part(commands, words, &format!("pill:{act}-label"));
        } else {
            // Pass shows its gain from the game's own label; Terror its unit cost.
            let value = if act == "pass" {
                choice
                    .label
                    .split_once(" · ")
                    .map_or(String::new(), |(_, v)| v.to_owned())
            } else {
                "1 Resource per space".into()
            };
            let words = text(
                commands,
                art,
                Prose::named(value),
                rx(fig::ROW_X),
                ry(top + fig::VALUE_TOP),
                rs(fig::VALUE_RIGHT),
                rs(fig::VALUE_SIZE),
                s,
                o,
                5,
            );
            commands
                .entity(words)
                .insert((TextColor(QUIET), TextLayout::justify(Justify::Right)));
            part(commands, words, &format!("{name}-value"));
        }
    }
    let commit = commit(view, ui.choice.pinned.as_deref());
    let [x, y, w, h] = fig::COMMIT;
    let plate = picture(
        commands,
        art,
        if commit.enabled {
            "atlas-choice-commit.png"
        } else {
            "atlas-choice-commit-disabled.png"
        },
        rect(x, y, w, h),
        s,
        o,
        3,
    );
    commands
        .entity(plate)
        .remove::<Pickable>()
        .insert((
            Control {
                name: commit.name,
                choice: commit.enabled,
                action: commit.action,
            },
            Part("choice-commit".into()),
        ))
        .observe(super::click);
    let words = text(
        commands,
        art,
        Prose::named(commit.label),
        rx(x),
        ry(y + fig::COMMIT_TOP),
        rs(w),
        rs(fig::COMMIT_SIZE),
        s,
        o,
        5,
    );
    commands.entity(words).insert((
        TextColor(if commit.enabled {
            Color::srgb_u8(248, 244, 232)
        } else {
            QUIET
        }),
        TextLayout::justify(Justify::Center),
    ));
    style(
        commands,
        words,
        art,
        true,
        rs(fig::COMMIT_SIZE) * s,
        0.,
        None,
    );
    part(commands, words, "choice-commit-label");
}

/// The BRIEFING ribbon hanging from the panel's top edge while 26July executes Terror.
pub(super) fn ribbon(commands: &mut Commands, art: &Art, s: f32, o: Vec2) {
    let [x, y, w, h] = fig::RIBBON;
    let node = picture(
        commands,
        art,
        "atlas-briefing-ribbon.png",
        rect(fig::PANEL[0] + x, fig::PANEL[1] + y, w, h),
        s,
        o,
        4,
    );
    commands
        .entity(node)
        .remove::<Pickable>()
        .insert((
            Control {
                name: "briefing-ribbon".into(),
                choice: false,
                action: Action::Ribbon,
            },
            Part("briefing-ribbon".into()),
        ))
        .observe(super::click);
}

/// A click on an option row, the ribbon or a tab. None of them changes the match.
pub(super) fn activate(ui: &mut Interface, action: &Action) {
    match action {
        Action::Pin(option) => {
            ui.choice.pinned = if ui.choice.pinned.as_deref() == Some(option) {
                None
            } else {
                Some(option.clone())
            };
        }
        Action::Ribbon => ui.choice.ribbon = !ui.choice.ribbon,
        Action::Tab(option) => ui.choice.tab = Some(option.clone()),
        _ => {}
    }
    ui.dirty = true;
}

/// The controls Tab moves through, in order, by their seat names.
fn focusable(view: &Observation, ui: &Interface) -> Vec<String> {
    if view.finished {
        return vec![];
    }
    match view.action {
        ActionChoice::Open => view
            .choices
            .iter()
            .map(|c| format!("option:{}", action(&c.id)))
            .chain([commit(view, ui.choice.pinned.as_deref()).name])
            .collect(),
        ActionChoice::Terror if view.executing.is_some() => {
            let mut names = vec!["briefing-ribbon".to_owned()];
            if ui.choice.ribbon {
                names.extend(fig::TABS.map(|(option, ..)| tab_name(option)));
            }
            names
        }
        _ => vec![],
    }
}

fn tab_name(option: &str) -> String {
    format!("briefing-tab:{}", option.trim_start_matches("terror:"))
}

/// Keyboard reach: Tab and Shift+Tab move focus through the options, the commit, the
/// ribbon and its tabs; Enter or Space activates the focused one as a click would;
/// Escape unpins the briefing or closes the ribbon's tag.
pub(super) fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
) {
    let tab = keys.just_pressed(KeyCode::Tab);
    let enter = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]);
    let escape = keys.just_pressed(KeyCode::Escape);
    if !(tab || enter || escape) || ui.reader || play.session.is_none() {
        return;
    }
    // The transfer modal takes the keyboard while it is shown (`transfer::keys`).
    if super::transfer::shown(play.game().transfer.as_ref(), &ui) {
        return;
    }
    let view = play.game().observe();
    if escape {
        if ui.choice.pinned.is_some() {
            ui.choice.pinned = None;
        } else {
            ui.choice.ribbon = false;
        }
        ui.dirty = true;
        return;
    }
    let order = focusable(&view, &ui);
    if order.is_empty() {
        return;
    }
    let at = ui
        .choice
        .focus
        .as_ref()
        .and_then(|f| order.iter().position(|n| n == f));
    if tab {
        let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let n = order.len();
        let next = match (at, back) {
            (None, false) => 0,
            (None, true) => n - 1,
            (Some(i), false) => (i + 1) % n,
            (Some(i), true) => (i + n - 1) % n,
        };
        ui.choice.focus = Some(order[next].clone());
        ui.dirty = true;
        return;
    }
    let Some(focused) = at.map(|i| order[i].clone()) else {
        return;
    };
    if let Some(option) = focused.strip_prefix("option:") {
        activate(&mut ui, &Action::Pin(option.into()));
    } else if focused == "briefing-ribbon" {
        activate(&mut ui, &Action::Ribbon);
    } else if let Some((option, ..)) = fig::TABS
        .iter()
        .find(|(option, ..)| tab_name(option) == focused)
    {
        activate(&mut ui, &Action::Tab((*option).into()));
    } else {
        match commit(&view, ui.choice.pinned.as_deref()).action {
            Action::Commit(owner, revision) => {
                decide(&mut play, &mut ui, owner, revision, &focused);
            }
            Action::Unsupported(reason) => ui.notice = reason,
            _ => {}
        }
        ui.dirty = true;
    }
}

/// What the tag shows now: at the action choice, the hovered option, else the
/// focused one, else the pinned one; during Terror, the ribbon's open tab.
fn wanted(game: &Game, ui: &Interface) -> Option<(String, bool, bool)> {
    if ui.reader || game.finished {
        return None;
    }
    match game.action {
        ActionChoice::Open => {
            let hovered = match ui.hover.visible() {
                Some(Target::Choice(name)) => name.strip_prefix("option:").map(str::to_owned),
                _ => None,
            };
            let focused = ui
                .choice
                .focus
                .as_deref()
                .and_then(|f| f.strip_prefix("option:"))
                .map(str::to_owned);
            let option = hovered.or(focused).or_else(|| ui.choice.pinned.clone())?;
            let pinned = ui.choice.pinned.as_ref() == Some(&option);
            Some((option, pinned, false))
        }
        ActionChoice::Terror if ui.choice.ribbon => Some((ui.choice.tab().to_owned(), true, true)),
        _ => None,
    }
}

/// A text run of one briefing: faction names become emblems after the briefing's
/// first mention of each (`seen`), as #60's rule asks.
#[allow(clippy::too_many_arguments)] // Content, layout rectangle and type.
fn run(
    commands: &mut Commands,
    art: &Art,
    words: &str,
    seen: &mut [bool; 4],
    at: [f32; 3],
    size: f32,
    line: f32,
    z: i32,
) -> Entity {
    let font = TextFont {
        font: art.font.clone().into(),
        font_size: size.into(),
        ..default()
    };
    let segments = emblems::segments_after(words, Naming::FirstMention, false, seen);
    let (first, rest) = match segments.split_first() {
        Some((emblems::Segment::Words(w), rest)) => (w.clone(), rest),
        _ => (String::new(), &segments[..]),
    };
    let root = commands
        .spawn((
            Tag,
            Text::new(first),
            emblems::Plain(words.into()),
            font.clone(),
            TextColor(INK),
            LineHeight::Px(line),
            Node {
                position_type: PositionType::Absolute,
                left: px(at[0]),
                top: px(at[1]),
                width: px(at[2]),
                ..default()
            },
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id();
    emblems::spawn_rest(
        commands,
        art,
        root,
        rest,
        &font,
        TextColor(INK),
        size * emblems::HEIGHT,
    );
    root
}

/// What [`draw`] last drew: the revision, the shown option (pinned, with tabs), the
/// open tab and the window size.
type Drawn = (u64, Option<(String, bool, bool)>, String, [u32; 2]);

/// Draws the briefing tag and the active row's outline when what they show changes.
#[allow(clippy::too_many_arguments)] // The shared hover, layout and drawn entities.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Drawn>>,
) {
    let Some(game) = play.session.as_ref().map(Session::game) else {
        return;
    };
    let wanted = wanted(game, &ui);
    let key = (
        game.revision,
        wanted.clone(),
        if wanted.as_ref().is_some_and(|w| w.2) {
            ui.choice.tab().to_owned()
        } else {
            String::new()
        },
        [window.width().to_bits(), window.height().to_bits()],
    );
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some((option, pinned, tabs)) = wanted else {
        return;
    };
    let view = game.observe();
    let Some(briefing) = briefing::content(&view, &option) else {
        return;
    };
    let (s, left) = layout(&window);
    let o = left + Vec2::new(window.width() - WIDTH * s, 0.);
    let at = |x: f32, y: f32| o + Vec2::new(rx(x), ry(y)) * s;
    // The active row's outline at the action choice.
    if !tabs && let Some(i) = view.choices.iter().position(|c| action(&c.id) == option) {
        let plate = picture(
            &mut commands,
            &art,
            "atlas-choice-row-active.png",
            rect(fig::ROW_X, fig::ROWS[i], fig::ROW_W, fig::ROW_H),
            s,
            o,
            4,
        );
        commands
            .entity(plate)
            .remove::<UiRoot>()
            .insert((Tag, Part("option-active".into())));
    }
    let tag_x = fig::PANEL[0] - fig::TAG_GAP - fig::TAG_W;
    let left_x = tag_x + fig::PAD[3];
    let width = fig::TAG_W - fig::PAD[1] - fig::PAD[3];
    let mut y = fig::PANEL[1] + fig::PAD[0];
    let mut spawned: Vec<(Entity, String)> = vec![];
    let words = |commands: &mut Commands,
                 content: &str,
                 x: f32,
                 y: f32,
                 w: f32,
                 size: f32,
                 ink: Color,
                 bold: bool,
                 spacing: f32| {
        let p = at(x, y);
        let entity = text(
            commands,
            &art,
            Prose::named(content),
            p.x,
            p.y,
            rs(w) * s,
            rs(size) * s,
            1.,
            Vec2::ZERO,
            6,
        );
        commands
            .entity(entity)
            .remove::<UiRoot>()
            .insert((Tag, TextColor(ink)));
        style(
            commands,
            entity,
            &art,
            bold,
            rs(size) * s,
            rs(spacing) * s,
            None,
        );
        entity
    };
    if tabs {
        let mut x = left_x;
        for (tab, label, w) in fig::TABS {
            let active = tab == option;
            let key = format!(
                "atlas-briefing-tab-{}{}.png",
                label.to_lowercase(),
                if active { "-active" } else { "" }
            );
            let plate = picture(
                &mut commands,
                &art,
                &key,
                rect(x, y, w, fig::TAB_H),
                s,
                o,
                6,
            );
            let name = tab_name(tab);
            commands
                .entity(plate)
                .remove::<(UiRoot, Pickable)>()
                .insert((
                    Tag,
                    Control {
                        name: name.clone(),
                        choice: false,
                        action: Action::Tab(tab.into()),
                    },
                    Part(name),
                ))
                .observe(super::click);
            let label = words(
                &mut commands,
                label,
                x + fig::TAB_TEXT[0],
                y + fig::TAB_TEXT[1],
                w - fig::TAB_TEXT[0],
                fig::TAB_SIZE,
                if active {
                    Color::srgb_u8(248, 244, 232)
                } else {
                    INK
                },
                true,
                0.,
            );
            commands.entity(label).insert(GlobalZIndex(7));
            x += w + fig::TAB_GAP;
        }
        y += fig::TAB_H + fig::GAP;
    }
    let title = words(
        &mut commands,
        &briefing.title,
        left_x,
        y,
        width,
        fig::TITLE_SIZE,
        INK,
        false,
        0.,
    );
    spawned.push((title, "briefing-title".into()));
    let rule = words(
        &mut commands,
        briefing.rule,
        left_x,
        y + fig::RULE_REF_TOP,
        width,
        fig::RULE_REF_SIZE,
        QUIET,
        false,
        0.,
    );
    commands
        .entity(rule)
        .insert(TextLayout::justify(Justify::Right));
    spawned.push((rule, "briefing-rule-ref".into()));
    y += fig::TITLE_LINE + fig::GAP;
    let separator = picture(
        &mut commands,
        &art,
        "atlas-briefing-rule.png",
        rect(left_x, y, width, fig::SEPARATOR),
        s,
        o,
        6,
    );
    spawned.push((separator, "briefing-separator".into()));
    y += fig::SEPARATOR + fig::GAP;
    let mut seen = [false; 4];
    for entry in &briefing.entries {
        let label = words(
            &mut commands,
            entry.label,
            left_x,
            y,
            width,
            fig::CAPS_SIZE,
            QUIET,
            true,
            fig::CAPS_SPACING,
        );
        spawned.push((label, format!("briefing-label:{}", entry.label)));
        let lines = briefing::lines(&entry.text, briefing::CHARS);
        let p = at(left_x, y + fig::ENTRY_VALUE_TOP);
        let value = run(
            &mut commands,
            &art,
            &lines.join("\n"),
            &mut seen,
            [p.x, p.y, rs(width) * s],
            rs(fig::VALUE_TEXT_SIZE) * s,
            rs(fig::VALUE_LINE) * s,
            6,
        );
        spawned.push((value, format!("briefing-value:{}", entry.label)));
        y += fig::ENTRY_VALUE_TOP + fig::VALUE_LINE * lines.len() as f32 + fig::GAP;
    }
    let lines = briefing::lines(&briefing.rules, briefing::RULES_CHARS);
    let rules = words(
        &mut commands,
        &lines.join("\n"),
        left_x,
        y,
        width,
        fig::RULES_SIZE,
        QUIET,
        false,
        0.,
    );
    commands
        .entity(rules)
        .insert(LineHeight::Px(rs(fig::RULES_LINE) * s));
    spawned.push((rules, "briefing-rules".into()));
    y += fig::RULES_LINE * lines.len() as f32 + fig::PAD[2];
    let height = y - fig::PANEL[1];
    let plate = picture(
        &mut commands,
        &art,
        "atlas-victory-tag-26july.png",
        rect(tag_x, fig::PANEL[1], fig::TAG_W, height),
        s,
        o,
        5,
    );
    commands.entity(plate).remove::<UiRoot>().insert((
        Tag,
        Part("briefing".into()),
        Shown {
            briefing,
            pinned,
            tabs: if tabs {
                fig::TABS.iter().map(|(o, ..)| (*o).to_owned()).collect()
            } else {
                vec![]
            },
        },
    ));
    if pinned {
        let [px_, py, pw, ph] = fig::PIN;
        let pin = picture(
            &mut commands,
            &art,
            "atlas-briefing-pin.png",
            rect(tag_x + px_, fig::PANEL[1] + py, pw, ph),
            s,
            o,
            7,
        );
        spawned.push((pin, "briefing-pin".into()));
    }
    for (entity, name) in spawned {
        commands
            .entity(entity)
            .remove::<UiRoot>()
            .insert((Tag, Part(name)));
    }
}

/// The action choice's presentation state as the seat reads it.
pub(super) fn observe(ui: &Interface) -> serde_json::Value {
    serde_json::json!({
        "pinned": ui.choice.pinned,
        "focus": ui.choice.focus,
        "ribbon": ui.choice.ribbon,
        "tab": ui.choice.tab(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout's row tops step by the authored pitch (97.2 frame units, 36 px at
    /// the reference) apart from the two section labels, and the commit sits at the
    /// accepted panel foot (592 reference px from the panel top, 52 tall).
    #[test]
    fn layout_constants_convert_to_the_reference_panel() {
        assert!((rx(fig::PANEL[0]) - 1380.).abs() < 1e-3);
        assert!((ry(fig::PANEL[1]) - 280.).abs() < 1e-3);
        let [x, y, w, h] = rect(
            fig::COMMIT[0],
            fig::COMMIT[1],
            fig::COMMIT[2],
            fig::COMMIT[3],
        );
        assert!(
            (x - 1404.).abs() < 0.01 && (y - 872.).abs() < 0.01,
            "{x} {y}"
        );
        assert!((w - 452.).abs() < 0.01 && (h - 52.).abs() < 0.01, "{w} {h}");
        let tag = rx(fig::PANEL[0] - fig::TAG_GAP - fig::TAG_W);
        assert!((tag - 904.).abs() < 0.01, "{tag}");
        assert!((rs(fig::TAG_W) - 468.).abs() < 0.01);
    }

    #[test]
    fn every_listed_option_has_a_row_label() {
        let view = Game::opening().observe();
        assert_eq!(view.choices.len(), fig::ROWS.len());
        for choice in &view.choices {
            let (label, pill) = listing(action(&choice.id));
            assert!(!label.is_empty(), "{}", choice.id);
            assert_eq!(pill.is_none(), choice.supported, "{}", choice.id);
        }
    }

    /// The commit follows the pinned option: Pass and Terror commit their choice;
    /// an option not in this task is refused with the game's own explanation.
    #[test]
    fn the_commit_reads_the_pinned_option() {
        let view = Game::opening().observe();
        let none = commit(&view, None);
        assert!(!none.enabled);
        assert_eq!(none.name, "choice-commit");
        let pass = commit(&view, Some("pass"));
        assert!(pass.enabled);
        assert_eq!(pass.name, "0:pass");
        assert_eq!(pass.label, "Pass · +1 Resource");
        let terror = commit(&view, Some("terror"));
        assert_eq!(
            (terror.name.as_str(), terror.label.as_str()),
            ("0:terror", "Choose Terror")
        );
        let rally = commit(&view, Some("rally"));
        assert!(!rally.enabled);
        assert_eq!(rally.label, "Rally is not in this task");
        let Action::Unsupported(reason) = rally.action else {
            panic!("Rally commits nothing")
        };
        assert!(reason.contains("not in this task"), "{reason}");
        assert_eq!(
            commit(&view, Some("terror:kidnap")).label,
            "Kidnap is offered once Terror is chosen"
        );
    }

    #[test]
    fn tab_order_covers_every_option_then_the_commit() {
        let view = Game::opening().observe();
        let ui = Interface::default();
        let order = focusable(&view, &ui);
        assert_eq!(order.len(), 12);
        assert_eq!(order[0], "option:pass");
        assert_eq!(order[11], "choice-commit");
        let mut game = Game::opening();
        game.submit(Faction::M26, 0, "0:terror").unwrap();
        let mut ui = Interface::default();
        assert_eq!(focusable(&game.observe(), &ui), ["briefing-ribbon"]);
        ui.choice.ribbon = true;
        assert_eq!(
            focusable(&game.observe(), &ui),
            [
                "briefing-ribbon",
                "briefing-tab:kidnap",
                "briefing-tab:terror"
            ]
        );
    }
}
