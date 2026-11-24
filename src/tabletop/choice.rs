//! Action choices in the selected AC-B design, "Briefing tag" (#68, built in
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
use super::highlights::Targeting;
use super::palette::{INK, PAPER, QUIET};
use super::popover::Popover;
use super::*;
use crate::{ActionChoice, Choice, EventProgress, Observation, Transfer, resources};
use bevy::text::{Justify, LetterSpacing, LineHeight};

/// Node coordinates in the FigCraft 16:9 frame, `art/atlas/atlas-action-choice.figcraft`.
/// EX-A's execution panel (`execution.rs`) shares the panel place, rows and tag.
pub(super) mod fig {
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
pub(super) fn rx(x: f32) -> f32 {
    1380. + (x - fig::PANEL[0]) / fig::SCALE
}
pub(super) fn ry(y: f32) -> f32 {
    280. + (y - fig::PANEL[1]) / fig::SCALE
}
pub(super) fn rs(v: f32) -> f32 {
    v / fig::SCALE
}
pub(super) fn rect(x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    [rx(x), ry(y), rs(w), rs(h)]
}

/// The action choice's presentation state, kept in `Interface`.
#[derive(Default)]
pub(super) struct State {
    /// The pinned option, such as "pass", or during Terror the pinned step, such as
    /// "terror:santiago" (#84): its tag stays open and the commit reads it.
    pub pinned: Option<String>,
    /// The control that has keyboard focus, by its seat name.
    pub focus: Option<String>,
    /// The briefing tag is open from the BRIEFING ribbon during Terror.
    pub ribbon: bool,
    /// The ribbon's open tab: "terror" (the default) or "terror:kidnap".
    pub tab: Option<String>,
}
impl State {
    /// A decision committed: the pin and focus no longer apply. A step that the
    /// decision opened (Kidnap or the transfer) holds the panel, and its tag opens
    /// pinned (EX-A). Removal mode's tag stays closed until its row is hovered or
    /// pinned: the pieces it removes are on the map's east, which the tag covers.
    pub fn decided(&mut self, view: &Observation) {
        self.pinned = super::execution::held(view)
            .filter(|step| *step != "remove")
            .map(str::to_owned);
        self.focus = None;
    }
    fn tab(&self) -> &str {
        self.tab.as_deref().unwrap_or("terror")
    }
    /// Closes the ribbon's tag and, during Terror, the pinned step's tag. A space
    /// popover opened on the map wins over either tag (owner's clash rule, #58,
    /// extended to step tags in #83's selection); the tag reopens from the ribbon or
    /// from its step's row. Only called during Terror.
    pub fn close(&mut self) {
        self.ribbon = false;
        self.pinned = None;
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

/// Restyles a drawn text: Lato Bold, letter spacing, line height or alignment.
pub(super) fn style(
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
    let choice = pinned.and_then(|p| view.choices.iter().find(|c| c.action() == p));
    match choice {
        Some(c) if c.supported => Commit {
            label: match c.action() {
                // The gain the authority previews, possibly zero at the cap.
                "pass" | "government:pass" => c
                    .preview
                    .iter()
                    .flat_map(|p| &p.resources)
                    .map(|r| format!("Pass · +{}", resources(r.to.saturating_sub(r.from))))
                    .next()
                    .unwrap_or_else(|| {
                        if c.preview.is_some() {
                            format!("Pass · +{}", resources(0))
                        } else {
                            "Pass".into()
                        }
                    }),
                "terror" => "Choose Terror".into(),
                action if crate::Game::assault_payment_space(action).is_some() => {
                    format!("Pay {} · Assault", resources(view.alliance.coin_cost()))
                }
                "event:shaded" => "Choose shaded Event".into(),
                "event:unshaded" => "Choose unshaded Event".into(),
                _ => c.label.clone(),
            },
            action: Action::Commit(c.owner, view.revision),
            name: c.id.clone(),
            enabled: true,
        },
        Some(c) => {
            let label = if crate::Game::assault_payment_space(c.action()).is_some() {
                format!("Cannot pay {}", resources(view.alliance.coin_cost()))
            } else if view.current_card == 18 {
                "Inspection only".into()
            } else if view.eligibility.directorio_handoff() {
                format!("{} is outside this task", c.label)
            } else if c.action() == "terror:kidnap" {
                "Kidnap is offered once Terror is chosen".into()
            } else {
                let (name, _) = listing(c.action());
                format!("{} is not in this task", name.split(" · ").next().unwrap_or(name))
            };
            Commit {
                label,
                action: Action::Unsupported(c.explanation.clone()),
                name: "choice-commit".into(),
                enabled: false,
            }
        }
        None if view.transfer.is_some() => Commit {
            label: "Resource transfer open".into(),
            action: Action::Unsupported("Use the Resource transfer controls first.".into()),
            name: "choice-commit".into(),
            enabled: false,
        },
        None => Commit {
            label: if view.current_card == 18 || view.eligibility.directorio_handoff() {
                "Click an option to inspect it"
            } else {
                "Click an option to choose it"
            }
            .into(),
            action: Action::Unsupported(
                if view.current_card == 18 || view.eligibility.directorio_handoff() {
                    "Hover or focus an option for its briefing. Click or press Enter to pin it."
                } else {
                    "Pin an option first: click it, or focus it with Tab and press Enter. Its briefing stays open and this button commits it."
                }
                .into(),
            ),
            name: "choice-commit".into(),
            enabled: false,
        },
    }
}

fn panel_options(view: &Observation) -> impl Iterator<Item = &Choice> {
    view.choices.iter().filter(|choice| {
        view.transfer.is_none() && (view.removal.is_none() || choice.action() == "done-removal")
    })
}

/// Event execution and the card 18 decisions use rows without section headings.
fn panel_rows(view: &Observation) -> impl Iterator<Item = (&Choice, f32)> {
    let tops = if view.current_card == 18 || view.event.as_ref().is_some_and(EventProgress::pending)
    {
        let within_section_pitch = fig::ROWS[2] - fig::ROWS[1];
        std::array::from_fn(|i| fig::ROWS[0] + i as f32 * within_section_pitch)
    } else {
        fig::ROWS
    };
    panel_options(view).zip(tops)
}

fn panel_subtitle(view: &Observation, executor: Faction, directorio: bool) -> String {
    if view.current_card == 18 {
        let balance = resources(view.resources[view.owner.index()]);
        if let Some(next) = &view.next {
            let result =
                if view.eligibility.government_box == Some(crate::SequenceBox::OperationOnly) {
                    "\nGovernment recorded Operation Only (R 2.3.4)".into()
                } else {
                    view.recorded_resources
                        .iter()
                        .find(|change| change.faction == Faction::Government)
                        .map_or_else(String::new, |change| {
                            format!(
                                "\nGovernment Passed · +{} ({} → {})",
                                resources(change.to - change.from),
                                change.from,
                                change.to
                            )
                        })
                };
            return format!("{} on card 18 · {balance}{result}", next.role);
        }
        if let Some(Transfer::Authorizing { giver, .. }) = view.transfer {
            return format!(
                "{} decides the Resource transfer\nGovernment has {}",
                giver.name(),
                resources(view.resources[Faction::Government.index()])
            );
        }
        if view.removal.is_some() {
            return "Select a marked piece on the map.\nChoose Done to return to Assault.".into();
        }
        if executor == Faction::Government && view.executing.is_some() {
            if let Some(space) = view.choices.iter().find_map(|choice| {
                let id = crate::Game::assault_payment_space(choice.action())?;
                view.spaces.iter().find(|space| space.id == id)
            }) {
                return format!(
                    "{} · Select targets\n{balance} · {} per space",
                    space.name,
                    resources(view.alliance.coin_cost())
                );
            }
            return view.assault_result.as_ref().map_or_else(
                || {
                    format!(
                        "{balance} · {} US Alliance\n{} per space",
                        view.alliance.name(),
                        resources(view.alliance.coin_cost()),
                    )
                },
                |result| {
                    format!(
                        "{} · paid {} · {balance} left\n{}",
                        result.space_name(),
                        resources(result.paid),
                        result.removal_summary(),
                    )
                },
            );
        }
        let eligibility = if view.eligibility.event_restriction.is_some() {
            "Shaded Event keeps 26July Ineligible through card 18"
        } else {
            "26July and Directorio acted on card 17 (R 2.3.7)"
        };
        return format!(
            "1st Eligible on card 18 · {balance} · {} US Alliance\n{eligibility}",
            view.alliance.name()
        );
    }
    if let Some(preview) = view
        .choices
        .iter()
        .find(|choice| choice.action() == "resolve-shaded")
        .and_then(|choice| choice.preview.as_ref())
    {
        return format!(
            "Pending · {}",
            briefing::activation_lines(view, &preview.flips)
        );
    }
    let position = match &view.event {
        Some(EventProgress::ExecuteShaded) => "Alberto Bayo · Directorio executes (R 5.2)".into(),
        Some(EventProgress::ChooseRallyBeneficiary) => {
            "Alberto Bayo · Directorio chooses the beneficiary".into()
        }
        Some(EventProgress::Rally(rally)) => {
            let places = rally
                .remaining_spaces(&view.spaces)
                .map(|space| space.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if places.is_empty() {
                "Directorio's Event · No qualifying Base spaces remain".into()
            } else {
                format!("Directorio's Event · {places} · 0 Resources (R 5.5)")
            }
        }
        _ if directorio => "2nd Eligible on card 17 · Kidnap opened the Event (R 2.3.4)".into(),
        _ => format!(
            "1st Eligible on card 17 · 26July has {}",
            resources(view.resources[Faction::M26.index()])
        ),
    };
    let instruction = match &view.transfer {
        Some(Transfer::Authorizing { giver, .. }) => {
            format!("{} decides the Resource transfer", giver.name())
        }
        Some(Transfer::Selecting) => format!("{} composes the Resource transfer", executor.name()),
        None if view.removal.is_some() => "Click a marked piece, or select Done".into(),
        None => "Hover or focus an option for its briefing · click to pin it".into(),
    };
    format!("{position}\n{instruction}")
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
    let directorio = view.eligibility.directorio_handoff();
    let event_pending = view.event.as_ref().is_some_and(EventProgress::pending);
    let card_18 = view.current_card == 18;
    let executor = view.executing.unwrap_or(view.owner);
    let part = |commands: &mut Commands, entity: Entity, name: &str| {
        commands.entity(entity).insert(Part(name.into()));
    };
    let [x, y, w, h] = fig::EMBLEM;
    let emblem = picture(
        commands,
        art,
        &emblems::image(executor),
        rect(x, y, w, h),
        s,
        o,
        3,
    );
    part(commands, emblem, "choice-heading-emblem");
    let [x, y, w] = fig::HEADING;
    let title = match &view.event {
        _ if view.removal.is_some() => format!("{} · Voluntary removal", executor.name()),
        _ if card_18 && view.executing == Some(Faction::Government) => {
            "Government · Assault".into()
        }
        _ if card_18 => format!("{}'s decision", view.owner.name()),
        Some(EventProgress::ExecuteShaded) => "Directorio · Shaded Event".into(),
        Some(EventProgress::ChooseRallyBeneficiary) => "Choose who Rallies".into(),
        Some(EventProgress::Rally(rally)) => format!("{} · Free Rally", rally.beneficiary.name()),
        _ if directorio => "Directorio's options".into(),
        _ => "Choose 26July's action".into(),
    };
    let heading = text(
        commands,
        art,
        Prose::named(title),
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
        Prose::named(panel_subtitle(view, executor, directorio)),
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
    for (name, top, words) in fig::SECTIONS
        .into_iter()
        .filter(|_| !event_pending && !card_18)
    {
        let words = if directorio && name == "section:operations" {
            "OPERATIONS"
        } else {
            words
        };
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
    for (choice, top) in panel_rows(view) {
        let act = choice.action();
        let name = format!("option:{act}");
        let (label, pill) = if act == "government:pass" {
            ("Pass", None)
        } else if card_18 {
            (
                choice.label.as_str(),
                (!choice.supported).then_some("INSPECT"),
            )
        } else if directorio || event_pending {
            let status = (!choice.supported).then_some("NOT IN THIS TASK");
            (choice.label.as_str(), status)
        } else {
            listing(act)
        };
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
        } else if matches!(act, "pass" | "government:pass" | "terror") {
            // Pass shows its gain from the game's own label; Terror its unit cost.
            let value = if matches!(act, "pass" | "government:pass") {
                choice
                    .label
                    .split_once(" · ")
                    .map_or(String::new(), |(_, v)| v.to_owned())
            } else {
                format!("{} per space", resources(crate::TERROR_COST))
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
    primary(
        commands,
        art,
        Control {
            name: commit.name,
            choice: commit.enabled,
            action: commit.action,
        },
        &commit.label,
        commit.enabled,
        "choice-commit",
        s,
        o,
    );
}

/// The full-width primary action at the panel's foot (FigCraft `choice-commit`): the
/// action choice's commit, and Continue Terror on the presented Kidnap die (#74, the
/// die-presented frame). Its parts are `part_name` and `part_name-label`.
#[allow(clippy::too_many_arguments)]
pub(super) fn primary(
    commands: &mut Commands,
    art: &Art,
    control: Control,
    label: &str,
    enabled: bool,
    part_name: &str,
    s: f32,
    o: Vec2,
) {
    primary_with(
        commands,
        art,
        control,
        Prose::named(label),
        enabled,
        if enabled {
            "atlas-choice-commit.png"
        } else {
            "atlas-choice-commit-disabled.png"
        },
        part_name,
        s,
        o,
    );
}

/// The primary action with its plate and words given: EX-A's commit uses AC-B's
/// plates, or TM-B's giver-coloured primary for a pending request (#84).
#[allow(clippy::too_many_arguments)]
pub(super) fn primary_with(
    commands: &mut Commands,
    art: &Art,
    control: Control,
    label: Prose,
    enabled: bool,
    plate: &str,
    part_name: &str,
    s: f32,
    o: Vec2,
) {
    let [x, y, w, h] = fig::COMMIT;
    let node = picture(commands, art, plate, rect(x, y, w, h), s, o, 3);
    commands
        .entity(node)
        .remove::<Pickable>()
        .insert((control, Part(part_name.into())))
        .observe(super::click);
    let words = text(
        commands,
        art,
        label,
        rx(x),
        ry(y + fig::COMMIT_TOP),
        rs(w),
        rs(fig::COMMIT_SIZE),
        s,
        o,
        5,
    );
    commands.entity(words).insert((
        TextColor(if enabled { PAPER } else { QUIET }),
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
    commands
        .entity(words)
        .insert(Part(format!("{part_name}-label")));
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
            // A step pinned during Terror takes the tag's place from the ribbon's tag.
            ui.choice.ribbon = false;
        }
        Action::Ribbon => ui.choice.ribbon = !ui.choice.ribbon,
        Action::Tab(option) => ui.choice.tab = Some(option.clone()),
        _ => {}
    }
    ui.dirty = true;
}

/// The controls Tab moves through, in order, by their seat names, each with what
/// Enter does to it: the panel's; the spaces the map offers (#168); the Available
/// Forces ledger's four rows above the panel, whose tags focus opens as hover does
/// (#160); then the current and the preview card (#139) and the two piles (#144),
/// whose tags it opens too; and last Save · quit, whose tag focus opens too and which
/// Enter activates (#174).
fn focusable(
    view: &Observation,
    ui: &Interface,
    targeting: &Targeting,
    popover: &Popover,
) -> Vec<(String, Action)> {
    let mut controls = panel_focusable(view, ui);
    controls.extend(map_focusable(view, targeting, popover));
    controls.extend(super::available::focusable());
    controls
        .extend(super::cards::Slot::ALL.map(|slot| (slot.name().to_owned(), Action::Card(slot))));
    controls
        .extend(super::piles::Pile::ALL.map(|pile| (pile.name().to_owned(), Action::Pile(pile))));
    controls.push((super::save_quit::NAME.to_owned(), Action::Quit));
    controls
}

/// While map targeting is active, the spaces it offers in `targets::focus_order`
/// (#168), each Enter-activated as a click on it; an open space popover's rows and
/// button follow its space. None while targeting is suspended.
fn map_focusable(
    view: &Observation,
    targeting: &Targeting,
    popover: &Popover,
) -> Vec<(String, Action)> {
    let Some(targets) = targeting.targets.as_deref() else {
        return vec![];
    };
    let open = popover.space(view.revision);
    let mut stops = vec![];
    for space in crate::targets::focus_order(targets) {
        stops.push((format!("space:{space}"), Action::Space(space.into())));
        if let Some((open, selected)) = open
            && open == space
        {
            stops.extend(super::popover::focusable(view, space, selected));
        }
    }
    stops
}

/// The panel's controls Tab moves through.
fn panel_focusable(view: &Observation, ui: &Interface) -> Vec<(String, Action)> {
    if let Some(advance) = view.choices.iter().find(|c| c.action() == "advance-card") {
        return vec![(
            advance.id.clone(),
            Action::Decision(advance.owner, view.revision),
        )];
    }
    match view.action {
        _ if view.current_card == 18
            || view.action == ActionChoice::Open
            || view.eligibility.directorio_handoff()
            || view.event.as_ref().is_some_and(EventProgress::pending) =>
        {
            if matches!(
                view.transfer,
                Some(Transfer::Authorizing { waited: true, .. })
            ) && let Some(choice) = view
                .choices
                .iter()
                .find(|choice| choice.action() == "reopen")
            {
                return vec![(
                    choice.id.clone(),
                    Action::Decision(choice.owner, view.revision),
                )];
            }
            let commit = commit(view, ui.choice.pinned.as_deref());
            panel_options(view)
                .map(|c| {
                    (
                        format!("option:{}", c.action()),
                        Action::Pin(c.action().into()),
                    )
                })
                .chain([(commit.name, commit.action)])
                .collect()
        }
        ActionChoice::Terror if view.executing.is_some() => {
            let mut controls = super::execution::focusable(view, ui.choice.pinned.as_deref());
            controls.push(("briefing-ribbon".to_owned(), Action::Ribbon));
            if ui.choice.ribbon {
                controls.extend(
                    fig::TABS.map(|(option, ..)| (tab_name(option), Action::Tab(option.into()))),
                );
            }
            controls
        }
        _ => vec![],
    }
}

fn tab_name(option: &str) -> String {
    format!("briefing-tab:{}", option.trim_start_matches("terror:"))
}

/// The stop Tab (or, `back`, Shift+Tab) moves to among `n`, from the focused one
/// `at`, wrapping at either end.
fn step(n: usize, at: Option<usize>, back: bool) -> usize {
    match (at, back) {
        (None, false) => 0,
        (None, true) => n - 1,
        (Some(i), false) => (i + 1) % n,
        (Some(i), true) => (i + n - 1) % n,
    }
}

/// Keyboard reach: Tab and Shift+Tab move focus through the options, the commit, the
/// ribbon and its tabs, the spaces the map offers and an open space popover's rows and
/// button; Enter or Space activates the focused one as a click would; Escape unpins
/// the briefing or closes the ribbon's tag, and closes an open space or piece popover,
/// leaving focus on a closed space popover's space (#168).
pub(super) fn keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
    mut popover: ResMut<Popover>,
    targeting: Res<Targeting>,
    roll: Res<super::die::Roll>,
    mut exit: MessageWriter<AppExit>,
) {
    let activation_keys = [KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space];
    let tab = keys.just_pressed(KeyCode::Tab);
    let enter = keys.any_just_pressed(activation_keys);
    let escape = keys.just_pressed(KeyCode::Escape);
    if !(tab || enter || escape) || play.session.is_none() {
        return;
    }
    // The transfer modal takes the keyboard while it is shown (`transfer::keys`), and so
    // does a presented Kidnap die: Space or Enter skips it, then continues (#74).
    let game = play.game();
    if super::transfer::shown(game.transfer.as_ref())
        || super::die::presented(game.revision, game.kidnap.as_ref(), &roll).is_some()
    {
        return;
    }
    let view = play.game().observe();
    if escape {
        // An open card tag closes first, and alone (#139).
        if super::cards::escape(&mut ui) {
            ui.dirty = true;
            return;
        }
        // An open space or piece popover closes as the tag does (#75); nothing is
        // decided. Keyboard focus on the space or in its popover returns to the space
        // (#168); a popover the pointer opened leaves focus where it was.
        if let Some((space, _)) = popover.space(view.revision) {
            let stop = format!("space:{space}");
            if ui
                .choice
                .focus
                .as_deref()
                .is_some_and(|f| f == stop || f.starts_with("popover:"))
            {
                ui.choice.focus = Some(stop);
            }
        }
        popover.close_all();
        if ui.choice.pinned.is_some() {
            ui.choice.pinned = None;
        } else {
            ui.choice.ribbon = false;
        }
        ui.dirty = true;
        return;
    }
    let order = focusable(&view, &ui, &targeting, &popover);
    if order.is_empty() {
        return;
    }
    let at = ui
        .choice
        .focus
        .as_ref()
        .and_then(|f| order.iter().position(|(n, _)| n == f));
    if tab {
        let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        ui.choice.focus = Some(order[step(order.len(), at, back)].0.clone());
        ui.dirty = true;
        return;
    }
    let Some((focused, action)) = at.map(|i| order[i].clone()) else {
        return;
    };
    // An activated control can open a modal in this update. Its next decision
    // needs a fresh key press, even though the modal's input system runs below.
    for key in activation_keys {
        keys.clear_just_pressed(key);
    }
    match action {
        Action::Pin(_) | Action::Ribbon | Action::Tab(_) => {
            activate(&mut ui, &action);
            // Whichever opened last wins (orchestrator implementation judgement, #58):
            // the ribbon's tag, or a step's tag pinned from its row (#84).
            let step = matches!(action, Action::Pin(_)) && view.action == ActionChoice::Terror;
            if (ui.choice.ribbon && matches!(action, Action::Ribbon))
                || (step && ui.choice.pinned.is_some())
            {
                popover.close_all();
            }
        }
        Action::Commit(owner, revision) | Action::Decision(owner, revision) => {
            decide(&mut play, &mut ui, owner, revision, &focused);
            ui.dirty = true;
        }
        Action::Submit(owner, revision, choice) => {
            decide(&mut play, &mut ui, owner, revision, &choice);
            ui.dirty = true;
        }
        Action::Unsupported(reason) => super::notice::refuse(&mut ui, reason),
        // Save · quit closes the game: every decision is already saved (#174).
        Action::Quit => {
            exit.write(AppExit::Success);
        }
        // Enter or Space on a focused card pins its tag, as a click does.
        Action::Card(slot) => {
            super::cards::activate(&mut ui, slot);
            ui.dirty = true;
        }
        // On a focused space they open or close its popover, and on a popover row they
        // choose it, as a click does (#168); neither decides anything.
        Action::Space(space) => {
            if popover.click_space(&space, &targeting) {
                ui.choice.close();
            }
            ui.dirty = true;
        }
        Action::Row(action) => popover.choose_row(&action),
        _ => {}
    }
}

/// What the tag shows now: at the action choice, the hovered option, else the
/// focused one, else the pinned one; during Terror, the ribbon's open tab, else the
/// hovered, focused or pinned step (#84). A space or piece popover wins over a step's
/// tag (the owner's clash rule, extended to step tags), and the transfer modal hides
/// it. Returns the option or step, whether it is pinned, and whether it is the
/// ribbon's tag.
fn wanted(game: &Game, ui: &Interface, popover: bool) -> Option<(String, bool, bool)> {
    let (prefix, ribbon) = match game.action {
        _ if game.card_18 => ("option:", false),
        ActionChoice::Open => ("option:", false),
        ActionChoice::Terror if game.eligibility().directorio_handoff() => ("option:", false),
        _ if game.event.as_ref().is_some_and(EventProgress::pending) => ("option:", false),
        _ if game.m26_action_ended => return None,
        ActionChoice::Terror if ui.choice.ribbon => {
            return Some((ui.choice.tab().to_owned(), true, true));
        }
        ActionChoice::Terror => ("step:", true),
        ActionChoice::Pass => return None,
    };
    if (ribbon || game.card_18 || game.event.as_ref().is_some_and(EventProgress::pending))
        && (popover || super::transfer::shown(game.transfer.as_ref()))
    {
        return None;
    }
    let hovered = match ui.hover.visible() {
        Some(Target::Choice(name)) => name.strip_prefix(prefix).map(str::to_owned),
        _ => None,
    };
    let focused = ui
        .choice
        .focus
        .as_deref()
        .and_then(|f| f.strip_prefix(prefix))
        .map(str::to_owned);
    let option = hovered.or(focused).or_else(|| ui.choice.pinned.clone())?;
    let pinned = ui.choice.pinned.as_ref() == Some(&option);
    Some((option, pinned, false))
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

/// A row of remedy chips in the tag, HF-A's chip plate hugging each remedy.
fn chips(commands: &mut Commands, art: &Art, chips: &[String], at: Vec2, s: f32) -> Entity {
    use super::execution::fig as ex;
    let row = commands
        .spawn((
            Tag,
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x),
                top: px(at.y),
                flex_direction: FlexDirection::Row,
                column_gap: px(rs(ex::CHIP_GAP) * s),
                ..default()
            },
            GlobalZIndex(6),
            Pickable::IGNORE,
        ))
        .id();
    for chip in chips {
        let plate = commands
            .spawn((
                Node {
                    height: px(rs(ex::CHIP_H) * s),
                    padding: UiRect::horizontal(px(rs(ex::CHIP_PAD) * s)),
                    align_items: AlignItems::Center,
                    flex_shrink: 0.,
                    ..default()
                },
                Part(format!("briefing-chip:{chip}")),
                Pickable::IGNORE,
            ))
            .id();
        let backdrop = commands
            .spawn(super::execution::backdrop(
                art,
                "atlas-popover-chip.png",
                6.,
                s,
            ))
            .id();
        commands.entity(plate).add_child(backdrop);
        let words = commands
            .spawn((
                Tag,
                Text::new(chip.clone()),
                emblems::Plain(chip.clone()),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: (rs(ex::CHIP_SIZE) * s).into(),
                    ..default()
                },
                TextColor(INK),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(plate).add_child(words);
        commands.entity(row).add_child(plate);
    }
    row
}

/// Draws the briefing tag and the active row's outline when what they show changes.
#[allow(clippy::too_many_arguments)] // The shared hover, layout and drawn entities.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, (With<Tag>, Without<ChildOf>)>,
    popover: Res<super::popover::Popover>,
    mut previous: Local<Option<Drawn>>,
) {
    let Some(game) = play.session.as_ref().map(Session::game) else {
        return;
    };
    let wanted = wanted(game, &ui, popover.open());
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
    let directorio = view.eligibility.directorio_handoff();
    let choosing = view.current_card == 18
        || view.action == ActionChoice::Open
        || directorio
        || view.event.as_ref().is_some_and(EventProgress::pending);
    let content = if tabs || choosing {
        briefing::content(&view, &option)
    } else {
        super::execution::tag(&view, &option)
    };
    let Some(briefing) = content else {
        return;
    };
    let s = reference_pixel(&window);
    let o = Vec2::new(window.width() - WIDTH * s, 0.);
    let at = |x: f32, y: f32| o + Vec2::new(rx(x), ry(y)) * s;
    // The active row's outline: an option at the action choice, or a step during
    // Terror (the step a mode holds open is outlined by the panel itself).
    let row = if choosing {
        panel_rows(&view)
            .find(|(c, _)| c.action() == option)
            .map(|(_, top)| [fig::ROW_X, top, fig::ROW_W, fig::ROW_H])
    } else if super::execution::held(&view) == Some(option.as_str()) {
        None
    } else {
        super::execution::row(&option)
    };
    if !tabs && let Some([x, y, w, h]) = row {
        let plate = picture(
            &mut commands,
            &art,
            "atlas-choice-row-active.png",
            rect(x, y, w, h),
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
                if active { PAPER } else { INK },
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
    if !briefing.rule.is_empty() {
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
    }
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
            &entry.label,
            left_x,
            y,
            width,
            fig::CAPS_SIZE,
            QUIET,
            true,
            fig::CAPS_SPACING,
        );
        spawned.push((label, format!("briefing-label:{}", entry.label)));
        y += fig::ENTRY_VALUE_TOP;
        if !entry.text.is_empty() {
            let lines = briefing::lines(&entry.text, briefing::CHARS);
            let p = at(left_x, y);
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
            y += fig::VALUE_LINE * lines.len() as f32;
        }
        if !entry.chips.is_empty() {
            let row = chips(&mut commands, &art, &entry.chips, at(left_x, y), s);
            spawned.push((row, format!("briefing-chips:{}", entry.label)));
            y += super::execution::fig::CHIP_H;
        }
        y += fig::GAP;
    }
    if briefing.rules.is_empty() {
        y += fig::PAD[2] - fig::GAP;
    } else {
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
    }
    let height = y - fig::PANEL[1];
    let faction = if choosing { view.owner } else { Faction::M26 };
    let plate = picture(
        &mut commands,
        &art,
        &format!("atlas-victory-tag-{}.png", slug(faction)),
        rect(tag_x, fig::PANEL[1], fig::TAG_W, height),
        s,
        o,
        5,
    );
    commands.entity(plate).remove::<UiRoot>().insert((
        Tag,
        // Opaque to the pointer: a click on the tag never reaches the map beneath it
        // (it covers Santiago and Oriente; the owner accepted the coverage).
        Pickable::default(),
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
        "focused_space": super::status::focused(ui),
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
            let (label, pill) = listing(choice.action());
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
        // The gain is the authority's preview, not a fixed "+1": at the 49 cap it is
        // nothing (R 1.8).
        let mut capped = view.clone();
        let preview = capped
            .choices
            .iter_mut()
            .find(|c| c.action() == "pass")
            .and_then(|c| c.preview.as_mut())
            .unwrap();
        preview.resources[0].from = 49;
        preview.resources[0].to = 49;
        assert_eq!(commit(&capped, Some("pass")).label, "Pass · +0 Resources");
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

    /// The map's targeting states as `highlights::refresh` computes them for `view`.
    fn aim(view: &Observation) -> Targeting {
        let mut targeting = Targeting::default();
        targeting.targets = crate::targets::space_targets(view);
        targeting.view = Some(view.clone());
        targeting
    }

    fn stops(
        view: &Observation,
        ui: &Interface,
        targeting: &Targeting,
        popover: &Popover,
    ) -> Vec<String> {
        focusable(view, ui, targeting, popover)
            .into_iter()
            .map(|(n, _)| n)
            .collect()
    }

    /// While map targeting is active, Tab goes from the panel's controls to the
    /// selectable spaces, then the blocked ones, each group in the observation's space
    /// order, then on to the stops that follow the panel (#168). An open space popover
    /// puts its rows and its button right after its space. Where targeting is
    /// suspended no space is a stop.
    #[test]
    fn tab_reaches_the_offered_spaces_by_map_state_after_the_panel() {
        let mut game = Game::opening();
        game.submit(Faction::M26, 0, "0:terror").unwrap();
        let view = game.observe();
        let ui = Interface::default();
        let targeting = aim(&view);
        let closed = Popover::default();
        // At 0 Resources neither Terror can be paid (1 Resource a space, R 3.3.4).
        // Kidnap can act in Santiago, where 26July's Guerrillas outnumber the Police
        // (R 4.3.3): Santiago is selectable. Oriente's Terror waits for a Resource
        // transfer: it is blocked.
        let order = stops(&view, &ui, &targeting, &closed);
        let ribbon = order.iter().position(|n| n == "briefing-ribbon").unwrap();
        assert_eq!(
            order[ribbon..ribbon + 4],
            [
                "briefing-ribbon",
                "space:santiago",
                "space:oriente",
                "available:government"
            ],
            "{order:?}"
        );
        // The group decides before the observation's space order: with Oriente
        // selectable and Santiago blocked, Oriente comes first.
        let mut swapped = view.clone();
        for choice in &mut swapped.choices {
            match choice.space() {
                Some("oriente") => choice.supported = true,
                Some("santiago") => {
                    choice.supported = false;
                    choice.remedies = vec!["Request a Resource transfer".into()];
                }
                _ => {}
            }
        }
        let order = stops(&swapped, &ui, &aim(&swapped), &closed);
        let spaces: Vec<_> = order.iter().filter(|n| n.starts_with("space:")).collect();
        assert_eq!(spaces, ["space:oriente", "space:santiago"]);

        // Santiago's popover open: its choosable row (Kidnap; Terror is not yet) and
        // its button follow Santiago, before Oriente.
        let mut open = Popover::default();
        assert!(open.click_space("santiago", &targeting));
        let order = stops(&view, &ui, &targeting, &open);
        let at = order.iter().position(|n| n == "space:santiago").unwrap();
        assert_eq!(
            order[at..at + 4],
            [
                "space:santiago",
                "popover:row:kidnap",
                "popover:commit",
                "space:oriente"
            ],
            "{order:?}"
        );
        let Some((_, Action::Submit(owner, revision, choice))) =
            focusable(&view, &ui, &targeting, &open)
                .into_iter()
                .find(|(n, _)| n == "popover:commit")
        else {
            panic!("the popover's button submits its chosen row's choice")
        };
        assert_eq!(
            (owner, revision, choice.as_str()),
            (Faction::M26, 1, "1:kidnap")
        );

        // Targeting is suspended at the action choice, while Kidnap is open, during a
        // Resource transfer, in removal mode and once Terror has ended: no space is a
        // stop. (A presented die holds the keyboard before any of these: `keys`.)
        let opening = Game::opening().observe();
        let mut suspended = vec![opening];
        for step in ["1:kidnap", "1:transfer", "1:remove", "1:end-terror"] {
            let mut game = game.clone();
            game.submit(Faction::M26, 1, step).unwrap();
            suspended.push(game.observe());
        }
        for view in &suspended {
            let order = stops(view, &ui, &aim(view), &open);
            assert!(
                !order
                    .iter()
                    .any(|n| n.starts_with("space:") || n.starts_with("popover:")),
                "revision {}: {order:?}",
                view.revision
            );
        }
    }

    /// Shift+Tab walks the same stops backwards, and both directions wrap.
    #[test]
    fn shift_tab_reverses_the_tab_order() {
        let forward: Vec<usize> = std::iter::successors(Some(step(5, None, false)), |&i| {
            Some(step(5, Some(i), false))
        })
        .take(6)
        .collect();
        assert_eq!(forward, [0, 1, 2, 3, 4, 0]);
        let back: Vec<usize> =
            std::iter::successors(Some(step(5, None, true)), |&i| Some(step(5, Some(i), true)))
                .take(6)
                .collect();
        assert_eq!(back, [4, 3, 2, 1, 0, 4]);
    }

    #[test]
    fn tab_order_covers_every_option_then_the_commit() {
        let names = |view: &Observation, ui: &Interface| -> Vec<String> {
            stops(view, ui, &aim(view), &Popover::default())
        };
        let view = Game::opening().observe();
        let ui = Interface::default();
        let order = names(&view, &ui);
        assert_eq!(order.len(), 21);
        assert_eq!(order[0], "option:pass");
        assert_eq!(order[11], "choice-commit");
        // Then the Available Forces rows above the panel (#160), both cards and both
        // piles, whose tags focus opens (#139, #144), and last Save · quit (#174).
        assert_eq!(
            order[12..],
            [
                "available:government",
                "available:26july",
                "available:directorio",
                "available:syndicate",
                "card:current",
                "card:preview",
                "pile:played",
                "pile:deck",
                "quit"
            ]
        );
        // During Terror (#84): every listed step, the commit, then the ribbon, then
        // the spaces the map offers (#168).
        let mut game = Game::opening();
        game.submit(Faction::M26, 0, "0:terror").unwrap();
        let mut ui = Interface::default();
        assert_eq!(
            names(&game.observe(), &ui),
            [
                "step:terror:santiago",
                "step:terror:oriente",
                "step:kidnap",
                "step:transfer",
                "step:remove",
                "step:end-terror",
                "step:back-to-choice",
                "execution-commit",
                "briefing-ribbon",
                "space:santiago",
                "space:oriente",
                "available:government",
                "available:26july",
                "available:directorio",
                "available:syndicate",
                "card:current",
                "card:preview",
                "pile:played",
                "pile:deck",
                "quit"
            ]
        );
        ui.choice.ribbon = true;
        let order = names(&game.observe(), &ui);
        assert_eq!(
            order[order.len() - 14..],
            [
                "briefing-ribbon",
                "briefing-tab:kidnap",
                "briefing-tab:terror",
                "space:santiago",
                "space:oriente",
                "available:government",
                "available:26july",
                "available:directorio",
                "available:syndicate",
                "card:current",
                "card:preview",
                "pile:played",
                "pile:deck",
                "quit"
            ]
        );
        ui.choice.pinned = Some("end-terror".into());
        assert!(names(&game.observe(), &ui).contains(&"1:end-terror".to_owned()));
    }
}
