//! Card 17's eligibility indicator in the selected EL-A design, "Sequence track" (#68,
//! built in #70): a board-like strip in the map's western margin. It lists the card's
//! four factions in order, each marked Eligible or Ineligible, with the 1st and 2nd
//! Eligible ribbons; below are the boxes the 1st Eligible faction's action is recorded
//! in (Pass, Operation Only, Operation + Special Activity) and the Ineligible box.
//! Hovering an action box opens a tag explaining what that box allows the next faction.
//!
//! Layout authority (foundation 4): `art/atlas/atlas-eligibility.figcraft` owns the
//! structure, placement and sizes. The constants in [`fig`] are that file's 16:9 frame
//! units (6144×3456, the 1920×1280 reference ×2.7), copied from its nodes, and
//! `atlas-eligibility.layout.json` records the same nodes for the native comparison.
//! Every drawn plate and mark is an admitted FigCraft or VectorCraft export
//! (`eligibility.sha256`); the runtime only places them and the text.
//!
//! The observation's Eligibility record is the only source of state (#58): the
//! presentation derives nothing. Hovering changes nothing in the match.
use super::emblems::{self, Naming};
use super::palette::{INK, QUIET};
use super::*;
use crate::{ActionChoice, Observation, SequenceBox, resources};
use bevy::text::{Justify, LetterSpacing, LineHeight};

/// Node coordinates in the FigCraft 16:9 frame, `art/atlas/atlas-eligibility.figcraft`.
/// Everything but [`PANEL`](fig::PANEL) is relative to the panel's top-left corner, in
/// the layout after 26July's action is recorded; before it, everything below 26July's
/// row moves up by [`OPEN_SHIFT`](fig::OPEN_SHIFT), because the row has no second line.
mod fig {
    /// Frame units per reference pixel.
    pub const SCALE: f32 = 2.7;
    /// The panel's top-left in the frame: 20 px right and 41.8 px below the map's
    /// top-left corner at the reference, under the map's "CUBA" heading.
    pub const PANEL: [f32; 2] = [162., 480.];
    pub const WIDTH: f32 = 626.4;
    pub const HEIGHT: f32 = 1036.7;
    pub const OPEN_SHIFT: f32 = -10.6;
    /// The ruled header graphic and its centred title.
    pub const HEADER: [f32; 4] = [29.7, 24.3, 572.4, 81.];
    pub const TITLE: [f32; 3] = [29.7, 49.3, 567.];
    pub const TITLE_SIZE: f32 = 25.65;
    pub const TITLE_SPACING: f32 = 2.7;
    /// Caps section labels (FigCraft `Indicator/caps`).
    pub const CAPS_SIZE: f32 = 22.95;
    pub const CAPS_SPACING: f32 = 2.16;
    pub const ELIGIBLE_LABEL: [f32; 2] = [48.6, 137.7];
    pub const RECORDED_LABEL: [f32; 2] = [29.7, 535.4];
    /// The faction rows: left edge, each row's top, and heights.
    pub const ROW_X: f32 = 48.6;
    pub const ROW_TOPS: [f32; 4] = [171.1, 262.7, 343.7, 424.7];
    pub const ROW_H: f32 = 75.6;
    /// 26July's row once it has acted: a second line says where its emblem went.
    pub const ACTED_ROW_H: f32 = 86.2;
    /// Inside a row, relative to its left edge and vertical centre.
    pub const NUMBER: [f32; 2] = [0., -16.];
    pub const NUMBER_SIZE: f32 = 27.;
    pub const EMBLEM: [f32; 3] = [37.8, -27., 54.];
    pub const NAME: [f32; 3] = [108., -19.5, 367.2];
    pub const NAME_SIZE: f32 = 32.4;
    /// An acted row's name and its note beneath.
    pub const ACTED_NAME_TOP: f32 = -35.;
    pub const ACTED_NOTE_TOP: f32 = 4.;
    pub const ACTED_NOTE_SIZE: f32 = 25.65;
    pub const RIBBON: [f32; 4] = [378., -20.25, 97.2, 40.5];
    pub const MARK: [f32; 3] = [491.4, -18.9, 37.8];
    /// The action boxes: left edge, each box's top, width and height.
    pub const BOX_X: f32 = 29.7;
    pub const BOX_TOPS: [f32; 3] = [579.6, 676.8, 774.];
    pub const BOX_W: f32 = 567.;
    pub const BOX_H: f32 = 81.;
    /// A box's caps label, empty and occupied, and the emblem in an occupied box.
    pub const BOX_LABEL_X: f32 = 24.3;
    pub const OCCUPIED_LABEL_X: f32 = 95.9;
    pub const BOX_EMBLEM: [f32; 3] = [25.6, 13.5, 54.];
    pub const BOX_LABEL_SPACING: f32 = 1.62;
    /// The Ineligible box, its label and its note.
    pub const INELIGIBLE: [f32; 2] = [871.2, 135.8];
    pub const INELIGIBLE_LABEL: [f32; 2] = [18.9, 16.2];
    pub const INELIGIBLE_NOTE: [f32; 3] = [18.9, 46.9, 529.2];
    pub const NOTE_SIZE: f32 = 27.;
    pub const NOTE_LINE: f32 = 35.1;
    /// The hover tag: its left edge, width, and its top above its box's top.
    pub const TAG_X: f32 = 648.;
    pub const TAG_W: f32 = 810.;
    pub const TAG_ABOVE_BOX: f32 = 66.6;
    /// Padding: top, right, bottom, left; then the gap between its parts.
    pub const TAG_PAD: [f32; 4] = [32.4, 37.8, 32.4, 48.6];
    pub const TAG_GAP: f32 = 16.2;
    pub const TAG_TITLE_SIZE: f32 = 43.2;
    pub const TAG_TITLE_LINE: f32 = 52.;
    pub const TAG_RULE_REF_SIZE: f32 = 31.05;
    pub const TAG_RULE_REF_TOP: f32 = 7.5;
    pub const TAG_SEPARATOR: f32 = 2.7;
    pub const TAG_BODY_SIZE: f32 = 33.75;
    pub const TAG_BODY_LINE: f32 = 45.9;
    pub const TAG_NOTE_SIZE: f32 = 29.7;
    pub const TAG_NOTE_LINE: f32 = 40.5;
    /// The leader from the panel's right edge to the tag, below its box's top.
    pub const LEADER: [f32; 4] = [626.4, 33.3, 21.6, 5.4];
}

/// The admitted exports this module draws, besides the faction emblems and the
/// accepted tag plate, leader and rule.
mod key {
    pub const PANEL: &str = "atlas-eligibility-panel.png";
    pub const PANEL_OPEN: &str = "atlas-eligibility-panel-open.png";
    pub const OCCUPIED: &str = "atlas-eligibility-box-occupied.png";
    pub const HEADER: &str = "atlas-eligibility-header.png";
    pub const FIRST: &str = "atlas-eligibility-tag-1st.png";
    pub const SECOND: &str = "atlas-eligibility-tag-2nd.png";
    pub const ELIGIBLE: &str = "atlas-eligibility-eligible.png";
    pub const INELIGIBLE: &str = "atlas-eligibility-ineligible.png";
    pub const VACANT: &str = "atlas-eligibility-vacant.png";
}

/// One of the boxes the 1st Eligible faction's action is recorded in (R 2.3, record
/// steps). Each is a hover target, named for the development seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) enum Slot {
    #[serde(rename = "eligibility:pass")]
    Pass,
    #[serde(rename = "eligibility:operation-only")]
    OperationOnly,
    #[serde(rename = "eligibility:operation-special-activity")]
    OperationWithSpecialActivity,
}
impl Slot {
    const ALL: [Self; 3] = [
        Self::Pass,
        Self::OperationOnly,
        Self::OperationWithSpecialActivity,
    ];
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Pass => "eligibility:pass",
            Self::OperationOnly => "eligibility:operation-only",
            Self::OperationWithSpecialActivity => "eligibility:operation-special-activity",
        }
    }
    fn of(record: SequenceBox) -> Self {
        match record {
            SequenceBox::Pass => Self::Pass,
            SequenceBox::OperationOnly => Self::OperationOnly,
            SequenceBox::OperationWithSpecialActivity => Self::OperationWithSpecialActivity,
        }
    }
    fn index(self) -> usize {
        self as usize
    }
    /// The box's caps label, its type size and its top inside the box.
    fn label(self) -> (&'static str, f32, f32) {
        match self {
            Self::Pass => ("PASS", 27., 24.5),
            Self::OperationOnly => ("OPERATION ONLY", 27., 24.5),
            Self::OperationWithSpecialActivity => ("OPERATION + SPECIAL ACTIVITY", 25.65, 25.),
        }
    }
}

/// One faction's row, as drawn.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Row {
    pub faction: Faction,
    pub eligible: bool,
    /// "1st" or "2nd" Eligible, if this faction holds that role now.
    pub role: Option<&'static str>,
    /// Its action is recorded: its emblem sits in its box, and the row shows a vacant slot.
    pub acted: bool,
}

/// What the indicator shows, read from the observation's Eligibility record alone.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Track {
    /// The factions in printed card order.
    pub rows: Vec<Row>,
    /// The cylinders now govern card 18 under R 2.3.7.
    pub adjusted: bool,
    /// The box the first action was recorded in, if any.
    pub occupied: Option<Slot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupant: Option<Faction>,
    /// The Ineligible box's contents.
    pub ineligible: Vec<Faction>,
}

fn recorded(view: &Observation) -> Option<(Faction, Slot)> {
    let eligibility = &view.eligibility;
    eligibility
        .government_box
        .map(|record| (Faction::Government, Slot::of(record)))
        .or_else(|| {
            eligibility
                .m26_box
                .map(|record| (Faction::M26, Slot::of(record)))
        })
}

pub(super) fn track(view: &Observation) -> Track {
    let e = &view.eligibility;
    let adjusted = e.next_card.is_some();
    let (occupant, occupied) =
        recorded(view).map_or((None, None), |(faction, slot)| (Some(faction), Some(slot)));
    let rows = e
        .order
        .iter()
        .map(|&faction| Row {
            faction,
            eligible: e.eligible[faction.index()],
            role: if adjusted || (Some(faction) == occupant && e.government_box.is_some()) {
                None
            } else if faction == e.first {
                Some("1st")
            } else if Some(faction) == e.second {
                Some("2nd")
            } else {
                None
            },
            acted: !adjusted && Some(faction) == occupant,
        })
        .collect();
    Track {
        rows,
        adjusted,
        occupied,
        occupant,
        ineligible: e
            .order
            .iter()
            .copied()
            .filter(|f| !e.eligible[f.index()])
            .collect(),
    }
}

/// The Ineligible box's note.
fn ineligible_note(track: &Track) -> String {
    if track.ineligible.is_empty() {
        "None on card 17: Eligibility changes after the card (R 2.3.7).".into()
    } else {
        track
            .ineligible
            .iter()
            .map(|f| f.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// What a box's hover tag says.
#[derive(Component, Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Shown {
    pub slot: Slot,
    pub title: &'static str,
    pub rule: &'static str,
    pub body: String,
    pub note: &'static str,
    /// An action is recorded in this box.
    pub occupied: bool,
}

/// R 2.3.7 adjusts Eligibility only after the card, so nobody becomes Ineligible here.
const AFTER_THE_CARD: &str =
    "Nobody becomes Ineligible during this card: Eligibility is adjusted after it (R 2.3.7).";

/// A box's tag. An occupied box says what 26July did there and what it allows the
/// next faction (R 2.3.3, 2.3.4); an empty one states the rule for that box.
pub(super) fn content(view: &Observation, slot: Slot) -> Shown {
    let e = &view.eligibility;
    let occupant = recorded(view)
        .and_then(|(faction, recorded_slot)| (recorded_slot == slot).then_some(faction));
    let occupied = occupant.is_some();
    let (title, rule) = match slot {
        Slot::Pass => ("Pass", "R 2.3.3"),
        Slot::OperationOnly => ("Operation Only", "R 2.3.4"),
        Slot::OperationWithSpecialActivity => ("Operation + Special Activity", "R 2.3.4"),
    };
    let operation = match view.action {
        ActionChoice::Terror => "Terror",
        _ => "an Operation",
    };
    // An occupied box words the authority's record: the Resources the recording step
    // moved, 26July's Eligibility for the next card, and the next faction's role and
    // options (R 2.3.3, 2.3.4, 2.3.7).
    let next = view.next.as_ref().map_or_else(String::new, |n| {
        format!(
            " {}, {}, may: {}.",
            n.faction.name(),
            n.role,
            n.options.join(" · ")
        )
    });
    let body = match (slot, occupied) {
        (Slot::Pass, true) if occupant == Some(Faction::Government) => {
            let income = view
                .recorded_resources
                .iter()
                .find(|r| r.faction == Faction::Government)
                .map_or_else(String::new, |r| {
                    format!(" +{} ({} → {}),", resources(r.to - r.from), r.from, r.to)
                });
            format!("Government Passed:{income} and stays Eligible for the next card.{next}")
        }
        (Slot::Pass, true) => {
            let gained: Vec<String> = view
                .recorded_resources
                .iter()
                .map(|r| format!("+{}", resources(r.to.saturating_sub(r.from))))
                .collect();
            let eligibility = match e.m26_next_card {
                Some(true) => ", and it stays Eligible",
                Some(false) => ", and it becomes Ineligible after this card",
                None => "",
            };
            format!(
                "26July Passed: {}{eligibility}.{next} {} is now 2nd Eligible.",
                gained.join(", "),
                e.second.expect("card 17 has a 2nd Eligible faction").name()
            )
        }
        (Slot::OperationOnly, true) if occupant == Some(Faction::Government) => format!(
            "Government finished Assault and recorded Operation Only.{next}"
        ),
        (Slot::OperationOnly, true) => format!(
            "26July executed an Operation without a Special Activity ({operation}). An End Terror with no space resolved also records this box (accepted zero-space reading, local policy).{next}"
        ),
        (Slot::OperationWithSpecialActivity, true) => format!(
            "26July executed an Operation with a Special Activity ({operation} with {}).{next}",
            view.special_activity.as_deref().unwrap_or("a Special Activity"),
        ),
        (Slot::Pass, false) => "A 1st or 2nd Eligible faction that Passes gets +1 Resource (Government +3) and stays Eligible. The next Eligible faction in card order takes its place, with the same options.".into(),
        (Slot::OperationOnly, false) => "Records a 1st Eligible faction's Operation without a Special Activity. The 2nd Eligible faction may then Pass or take a Limited Operation: 1 space, no Special Activity.".into(),
        (Slot::OperationWithSpecialActivity, false) => "Records a 1st Eligible faction's Operation with a Special Activity. The 2nd Eligible faction may then Pass, take a Limited Operation, or take the card's Event instead.".into(),
    };
    Shown {
        slot,
        title,
        rule,
        body,
        note: if e.next_card.is_some() || view.current_card == 18 {
            "Eligibility for card 18 was adjusted after card 17 (R 2.3.7)."
        } else {
            AFTER_THE_CARD
        },
        occupied,
    }
}

/// Names a drawn part by its node name in `atlas-eligibility.figcraft`, so the seat
/// can compare native placement with the layout record.
#[derive(Component)]
pub(super) struct Part(pub String);

/// The hover tag's entities, redrawn by [`draw`].
#[derive(Component)]
pub(super) struct Tag;

/// The indicator as drawn, for the seat.
#[derive(Component, Clone)]
pub(super) struct Drawn(pub Track);

/// A frame position in reference pixels: the layout is left-anchored, like the map.
fn r(v: f32) -> f32 {
    v / fig::SCALE
}
/// A rectangle relative to the panel, in reference pixels.
fn rect(x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    [r(fig::PANEL[0] + x), r(fig::PANEL[1] + y), r(w), r(h)]
}

/// The vertical layout of one state: each row's top and height, and the shift of
/// everything below the first row.
struct Rows {
    tops: [f32; 4],
    heights: [f32; 4],
    shift: f32,
}
fn rows(track: &Track) -> Rows {
    let acted = track.rows.iter().position(|row| row.acted);
    let shift = if acted.is_some() { 0. } else { fig::OPEN_SHIFT };
    let mut tops = fig::ROW_TOPS;
    for top in tops.iter_mut().skip(1) {
        *top += fig::OPEN_SHIFT;
    }
    let mut heights = [fig::ROW_H; 4];
    if let Some(index) = acted {
        heights[index] = fig::ACTED_ROW_H;
        for top in tops.iter_mut().skip(index + 1) {
            *top -= fig::OPEN_SHIFT;
        }
    }
    Rows {
        tops,
        heights,
        shift,
    }
}

/// A caps or bold label: Lato Bold with letter spacing.
#[allow(clippy::too_many_arguments)] // Content, layout rectangle, type and letterboxing.
fn label(
    commands: &mut Commands,
    art: &Art,
    words: &str,
    at: [f32; 3],
    size: f32,
    spacing: f32,
    ink: Color,
    bold: bool,
    s: f32,
    o: Vec2,
) -> Entity {
    let entity = text(
        commands,
        art,
        Prose::named(words),
        r(fig::PANEL[0] + at[0]),
        r(fig::PANEL[1] + at[1]),
        r(at[2]),
        r(size),
        s,
        o,
        4,
    );
    let mut e = commands.entity(entity);
    e.insert(TextColor(ink));
    if bold {
        e.insert(TextFont {
            font: art.bold.clone().into(),
            font_size: (r(size) * s).into(),
            ..default()
        });
    }
    if spacing > 0. {
        e.insert(LetterSpacing::Px(r(spacing) * s));
    }
    entity
}

/// Draws the indicator in the map's western margin, from the observation. `o` is the
/// left-anchored letterbox offset.
pub(super) fn indicator(commands: &mut Commands, art: &Art, view: &Observation, s: f32, o: Vec2) {
    let track = track(view);
    let layout = rows(&track);
    let shift = layout.shift;
    let part = |commands: &mut Commands, entity: Entity, name: String| {
        commands.entity(entity).insert(Part(name));
    };
    let plate = picture(
        commands,
        art,
        if shift == 0. {
            key::PANEL
        } else {
            key::PANEL_OPEN
        },
        rect(0., 0., fig::WIDTH, fig::HEIGHT + shift),
        s,
        o,
        2,
    );
    commands.entity(plate).insert(Drawn(track.clone()));
    part(commands, plate, "eligibility-panel".into());
    let [x, y, w, h] = fig::HEADER;
    let header = picture(commands, art, key::HEADER, rect(x, y, w, h), s, o, 3);
    part(commands, header, "eligibility-header".into());
    let [x, y, w] = fig::TITLE;
    let title = label(
        commands,
        art,
        &if track.adjusted {
            "CARD 18 · ELIGIBILITY".to_owned()
        } else {
            format!("CARD {} · SEQUENCE OF PLAY", view.current_card)
        },
        [x, y, w],
        fig::TITLE_SIZE,
        fig::TITLE_SPACING,
        INK,
        true,
        s,
        o,
    );
    commands
        .entity(title)
        .insert(TextLayout::justify(Justify::Center));
    part(commands, title, "eligibility-title".into());
    let [x, y] = fig::ELIGIBLE_LABEL;
    let eligible = label(
        commands,
        art,
        if track.adjusted {
            "AFTER CARD 17"
        } else {
            "ELIGIBLE FACTIONS · CARD ORDER"
        },
        [x, y, fig::BOX_W],
        fig::CAPS_SIZE,
        fig::CAPS_SPACING,
        QUIET,
        true,
        s,
        o,
    );
    part(commands, eligible, "eligibility-label:eligible".into());
    for (index, row) in track.rows.iter().enumerate() {
        let key_name = slug(row.faction);
        let center = layout.tops[index] + layout.heights[index] / 2.;
        let number = label(
            commands,
            art,
            &(index + 1).to_string(),
            [fig::ROW_X + fig::NUMBER[0], center + fig::NUMBER[1], 30.],
            fig::NUMBER_SIZE,
            0.,
            QUIET,
            true,
            s,
            o,
        );
        part(
            commands,
            number,
            format!("eligibility-row:{key_name}:number"),
        );
        let [ex, ey, size] = fig::EMBLEM;
        let emblem = picture(
            commands,
            art,
            &if row.acted {
                key::VACANT.to_owned()
            } else {
                emblems::image(row.faction)
            },
            rect(fig::ROW_X + ex, center + ey, size, size),
            s,
            o,
            3,
        );
        part(
            commands,
            emblem,
            format!("eligibility-row:{key_name}:emblem"),
        );
        let [nx, ny, nw] = fig::NAME;
        let name = label(
            commands,
            art,
            row.faction.name(),
            [
                fig::ROW_X + nx,
                center + if row.acted { fig::ACTED_NAME_TOP } else { ny },
                nw,
            ],
            fig::NAME_SIZE,
            0.,
            INK,
            row.role.is_some(),
            s,
            o,
        );
        part(commands, name, format!("eligibility-row:{key_name}:name"));
        if row.acted {
            let note = label(
                commands,
                art,
                "acted: in its box below",
                [fig::ROW_X + nx, center + fig::ACTED_NOTE_TOP, nw],
                fig::ACTED_NOTE_SIZE,
                0.,
                QUIET,
                false,
                s,
                o,
            );
            part(commands, note, format!("eligibility-row:{key_name}:acted"));
        }
        if let Some(role) = row.role {
            let [rx_, ry_, rw, rh] = fig::RIBBON;
            let ribbon = picture(
                commands,
                art,
                if role == "1st" {
                    key::FIRST
                } else {
                    key::SECOND
                },
                rect(fig::ROW_X + rx_, center + ry_, rw, rh),
                s,
                o,
                3,
            );
            part(
                commands,
                ribbon,
                format!("eligibility-row:{key_name}:{role}"),
            );
        }
        let [mx, my, size] = fig::MARK;
        let mark = picture(
            commands,
            art,
            if row.eligible {
                key::ELIGIBLE
            } else {
                key::INELIGIBLE
            },
            rect(fig::ROW_X + mx, center + my, size, size),
            s,
            o,
            3,
        );
        part(
            commands,
            mark,
            format!(
                "eligibility-row:{key_name}:{}",
                if row.eligible {
                    "eligible"
                } else {
                    "ineligible"
                }
            ),
        );
    }
    let [x, y] = fig::RECORDED_LABEL;
    let recorded = label(
        commands,
        art,
        if track.adjusted {
            "CARD 17 · RECORDED ACTION"
        } else {
            "1ST ELIGIBLE · ACTION RECORDED"
        },
        [x, y + shift, fig::BOX_W],
        fig::CAPS_SIZE,
        fig::CAPS_SPACING,
        QUIET,
        true,
        s,
        o,
    );
    part(commands, recorded, "eligibility-label:recorded".into());
    for slot in Slot::ALL {
        let top = fig::BOX_TOPS[slot.index()] + shift;
        let area = rect(fig::BOX_X, top, fig::BOX_W, fig::BOX_H);
        let occupied = track.occupied == Some(slot);
        if let Some(faction) = track.occupant.filter(|_| occupied) {
            let plate = picture(commands, art, key::OCCUPIED, area, s, o, 3);
            part(commands, plate, "eligibility-box-occupied".into());
            let [ex, ey, size] = fig::BOX_EMBLEM;
            let emblem = picture(
                commands,
                art,
                &emblems::image(faction),
                rect(fig::BOX_X + ex, top + ey, size, size),
                s,
                o,
                4,
            );
            part(commands, emblem, "eligibility-box-emblem".into());
        }
        let (words, size, label_top) = slot.label();
        let x = if occupied {
            fig::OCCUPIED_LABEL_X
        } else {
            fig::BOX_LABEL_X
        };
        let caps = label(
            commands,
            art,
            words,
            [fig::BOX_X + x, top + label_top, fig::BOX_W - x],
            size,
            fig::BOX_LABEL_SPACING,
            if occupied { INK } else { QUIET },
            true,
            s,
            o,
        );
        part(commands, caps, format!("{}:label", slot.name()));
        // The box itself is the hover target; it decides nothing.
        let [x, y, w, h] = area;
        let target = commands
            .spawn((
                UiRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(o.x + x * s),
                    top: px(o.y + y * s),
                    width: px(w * s),
                    height: px(h * s),
                    ..default()
                },
                GlobalZIndex(5),
                Control {
                    name: slot.name().into(),
                    choice: false,
                    action: Action::Sequence(slot),
                },
                Part(slot.name().into()),
            ))
            .id();
        commands.entity(target).observe(super::click);
    }
    let [top, height] = fig::INELIGIBLE;
    let area = rect(fig::BOX_X, top + shift, fig::BOX_W, height);
    let ineligible_box = commands
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
            Pickable::IGNORE,
        ))
        .id();
    part(
        commands,
        ineligible_box,
        "eligibility-box:ineligible".into(),
    );
    let [lx, ly] = fig::INELIGIBLE_LABEL;
    let caps = label(
        commands,
        art,
        "INELIGIBLE",
        [fig::BOX_X + lx, top + shift + ly, fig::BOX_W - lx],
        fig::CAPS_SIZE,
        fig::CAPS_SPACING,
        QUIET,
        true,
        s,
        o,
    );
    part(commands, caps, "eligibility-box:ineligible:label".into());
    let [nx, ny, nw] = fig::INELIGIBLE_NOTE;
    let note = label(
        commands,
        art,
        &ineligible_note(&track),
        [fig::BOX_X + nx, top + shift + ny, nw],
        fig::NOTE_SIZE,
        0.,
        QUIET,
        false,
        s,
        o,
    );
    commands
        .entity(note)
        .insert(LineHeight::Px(r(fig::NOTE_LINE) * s));
    part(commands, note, "eligibility-box:ineligible:note".into());
}

/// What [`draw`] last drew: the revision, the hovered box and the window size.
type Drawing = (u64, Option<Slot>, [u32; 2]);

/// Draws the hovered box's tag beside the panel, with its leader, when the shared
/// hover dwell shows a box. The plate is the accepted top-display tag, stretched to
/// its content as the other hover tags are.
#[allow(clippy::too_many_arguments)] // The shared hover, layout and drawn entities.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    // The tag's lines carry `Tag` too; despawning the roots takes them with it (#86).
    roots: Query<Entity, (With<Tag>, Without<ChildOf>)>,
    mut previous: Local<Option<Drawing>>,
) {
    let Some(game) = play.session.as_ref().map(Session::game) else {
        return;
    };
    let slot = ui.hover.visible().and_then(|t| match t {
        Target::Sequence(slot) => Some(*slot),
        _ => None,
    });
    let key = (
        game.revision,
        slot,
        [window.width().to_bits(), window.height().to_bits()],
    );
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some(slot) = slot else {
        return;
    };
    let view = game.observe();
    let shown = content(&view, slot);
    let shift = rows(&track(&view)).shift;
    let s = reference_pixel(&window);
    let box_top = fig::BOX_TOPS[slot.index()] + shift;
    // The track is anchored to the window's top left, where the reference's corner is.
    let at = |x: f32, y: f32| Vec2::new(r(fig::PANEL[0] + x), r(fig::PANEL[1] + y)) * s;
    let k = |v: f32| r(v) * s;
    let [lx, ly, lw, lh] = fig::LEADER;
    let leader_at = at(lx, box_top + ly);
    commands.spawn((
        Tag,
        Part("eligibility-leader".into()),
        ImageNode::new(art.images["atlas-status-leader.png"].clone())
            .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
        Node {
            position_type: PositionType::Absolute,
            left: px(leader_at.x),
            top: px(leader_at.y),
            width: px(k(lw)),
            height: px(k(lh)),
            ..default()
        },
        GlobalZIndex(7),
        Pickable::IGNORE,
    ));
    let tag_at = at(fig::TAG_X, box_top - fig::TAG_ABOVE_BOX);
    let [pt, pr, pb, pl] = fig::TAG_PAD;
    let tag = commands
        .spawn((
            Tag,
            Part("eligibility-tag".into()),
            Node {
                position_type: PositionType::Absolute,
                left: px(tag_at.x),
                top: px(tag_at.y),
                width: px(k(fig::TAG_W)),
                flex_direction: FlexDirection::Column,
                row_gap: px(k(fig::TAG_GAP)),
                padding: UiRect {
                    left: px(k(pl)),
                    right: px(k(pr)),
                    top: px(k(pt)),
                    bottom: px(k(pb)),
                },
                ..default()
            },
            GlobalZIndex(7),
            Pickable::IGNORE,
            shown.clone(),
        ))
        .id();
    // The plate fills the whole tag behind its padded content: an image on the padded
    // node itself would be drawn inside the padding.
    commands.spawn((
        Tag,
        ImageNode::new(art.images["atlas-top-tag.png"].clone())
            .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
        Node {
            position_type: PositionType::Absolute,
            left: px(0.),
            top: px(0.),
            width: percent(100.),
            height: percent(100.),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(tag),
    ));
    let words = |commands: &mut Commands,
                 parent: Entity,
                 prose: Prose,
                 size: f32,
                 line: Option<f32>,
                 ink: Color,
                 name: &str| {
        let font = TextFont {
            font: art.font.clone().into(),
            font_size: k(size).into(),
            ..default()
        };
        let segments = emblems::segments(&prose.text, prose.naming, false);
        let (first, rest) = match segments.split_first() {
            Some((emblems::Segment::Words(w), rest)) => (w.clone(), rest),
            _ => (String::new(), &segments[..]),
        };
        let entity = commands
            .spawn((
                Tag,
                Part(name.into()),
                Text::new(first),
                emblems::Plain(prose.text.clone()),
                font.clone(),
                TextColor(ink),
                Node::default(),
                Pickable::IGNORE,
                ChildOf(parent),
            ))
            .id();
        if let Some(line) = line {
            commands.entity(entity).insert(LineHeight::Px(k(line)));
        }
        emblems::spawn_rest(
            commands,
            &art,
            entity,
            rest,
            &font,
            TextColor(ink),
            k(size) * emblems::HEIGHT,
        );
        entity
    };
    let title_line = commands
        .spawn((
            Tag,
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                height: px(k(fig::TAG_TITLE_LINE)),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(tag),
        ))
        .id();
    words(
        &mut commands,
        title_line,
        Prose::named(shown.title),
        fig::TAG_TITLE_SIZE,
        None,
        INK,
        "eligibility-tag-title",
    );
    let rule = words(
        &mut commands,
        title_line,
        Prose::named(shown.rule),
        fig::TAG_RULE_REF_SIZE,
        None,
        QUIET,
        "eligibility-tag-rule-ref",
    );
    commands.entity(rule).insert(Node {
        margin: UiRect::top(px(k(fig::TAG_RULE_REF_TOP))),
        ..default()
    });
    commands.spawn((
        Tag,
        Part("eligibility-tag-separator".into()),
        ImageNode::new(art.images["atlas-briefing-rule.png"].clone())
            .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
        Node {
            height: px(k(fig::TAG_SEPARATOR)),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(tag),
    ));
    words(
        &mut commands,
        tag,
        Prose {
            text: shown.body.clone(),
            naming: Naming::FirstMention,
            red: false,
        },
        fig::TAG_BODY_SIZE,
        Some(fig::TAG_BODY_LINE),
        INK,
        "eligibility-tag-body",
    );
    words(
        &mut commands,
        tag,
        Prose::named(shown.note),
        fig::TAG_NOTE_SIZE,
        Some(fig::TAG_NOTE_LINE),
        QUIET,
        "eligibility-tag-note",
    );
}

/// The indicator and its tag as the seat reads them: the drawn track, the shown tag
/// and every named part's bounds in logical pixels.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let track = world
        .query::<&Drawn>()
        .iter(world)
        .next()
        .map(|d| d.0.clone());
    let tag = world.query::<&Shown>().iter(world).next().cloned();
    let mut parts = world
        .query::<(&Part, &ComputedNode, &UiGlobalTransform, Option<&emblems::Plain>)>()
        .iter(world)
        .map(|(part, node, at, text)| {
            let center = at.translation * node.inverse_scale_factor;
            let half = node.size() * node.inverse_scale_factor / 2.;
            (
                part.0.clone(),
                serde_json::json!({
                    "part": part.0,
                    "text": text.map(|t| &t.0),
                    "bounds": [center.x - half.x, center.y - half.y, center.x + half.x, center.y + half.y],
                }),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    serde_json::json!({
        "track": track,
        "tag": tag,
        "parts": parts.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    fn submit(game: &mut Game, action: &str) {
        let view = game.observe();
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        game.submit(owner, view.revision, &id)
            .unwrap_or_else(|e| panic!("{id}: {e}"));
    }

    /// The opening and every Terror step before End Terror: card 17's order, all four
    /// Eligible (R 2.3.1: nobody acted on a previous card in this position), 26July
    /// 1st and Directorio 2nd (R 2.3.2), and no box occupied yet.
    #[test]
    fn the_opening_shows_card_17_s_order_with_26july_first_and_no_box() {
        let opening = track(&Game::opening().observe());
        let mut terror = Game::opening();
        submit(&mut terror, "terror");
        for track in [opening, track(&terror.observe())] {
            let names: Vec<_> = track.rows.iter().map(|r| r.faction.name()).collect();
            assert_eq!(names, ["26July", "Directorio", "Government", "Syndicate"]);
            assert!(track.rows.iter().all(|r| r.eligible && !r.acted));
            let roles: Vec<_> = track.rows.iter().map(|r| r.role).collect();
            assert_eq!(roles, [Some("1st"), Some("2nd"), None, None]);
            assert_eq!(track.occupied, None);
            assert!(track.ineligible.is_empty());
            assert_eq!(
                ineligible_note(&track),
                "None on card 17: Eligibility changes after the card (R 2.3.7)."
            );
        }
    }

    /// R 2.3.3: a Passing 1st Eligible faction stays Eligible and the next leftmost
    /// Eligible faction replaces it: Directorio 1st, Government 2nd.
    #[test]
    fn after_pass_26july_sits_in_the_pass_box_and_directorio_is_first() {
        let mut game = Game::opening();
        submit(&mut game, "pass");
        let track = track(&game.observe());
        assert_eq!(track.occupied, Some(Slot::Pass));
        let rows: Vec<_> = track
            .rows
            .iter()
            .map(|r| (r.faction.name(), r.eligible, r.role, r.acted))
            .collect();
        assert_eq!(
            rows,
            [
                ("26July", true, None, true),
                ("Directorio", true, Some("1st"), false),
                ("Government", true, Some("2nd"), false),
                ("Syndicate", true, None, false),
            ]
        );
        assert!(track.ineligible.is_empty());
    }

    /// R 2.3.4 records the 1st Eligible faction's Operation in Operation Only, or in
    /// Operation + Special Activity after Kidnap. R 2.3.7: nobody turns Ineligible
    /// inside the card, so 26July keeps its check and its 1st Eligible ribbon.
    #[test]
    fn end_terror_records_operation_only_or_with_kidnap_and_nobody_turns_ineligible() {
        let mut only = Game::opening();
        submit(&mut only, "terror");
        submit(&mut only, "end-terror");
        let mut kidnap = Game::opening();
        submit(&mut kidnap, "terror");
        submit(&mut kidnap, "kidnap");
        submit(&mut kidnap, "kidnap:roll");
        submit(&mut kidnap, "terror:santiago");
        submit(&mut kidnap, "end-terror");
        for (game, slot) in [
            (only, Slot::OperationOnly),
            (kidnap, Slot::OperationWithSpecialActivity),
        ] {
            let track = track(&game.observe());
            assert_eq!(track.occupied, Some(slot));
            let rows: Vec<_> = track
                .rows
                .iter()
                .map(|r| (r.faction.name(), r.eligible, r.role, r.acted))
                .collect();
            assert_eq!(
                rows,
                [
                    ("26July", true, Some("1st"), true),
                    ("Directorio", true, Some("2nd"), false),
                    ("Government", true, None, false),
                    ("Syndicate", true, None, false),
                ]
            );
            assert!(track.ineligible.is_empty());
        }
    }

    /// The occupied box's tag states what it allows the next faction, agreeing with
    /// the rules authority's own list of Directorio's options; empty boxes state their
    /// rule (R 2.3.3, 2.3.4). Every tag says Eligibility changes only after the card.
    #[test]
    fn each_box_explains_what_it_allows_the_next_faction() {
        let mut pass = Game::opening();
        submit(&mut pass, "pass");
        let view = pass.observe();
        let shown = content(&view, Slot::Pass);
        assert!(shown.occupied);
        assert_eq!((shown.title, shown.rule), ("Pass", "R 2.3.3"));
        assert!(
            shown
                .body
                .starts_with("26July Passed: +1 Resource, and it stays Eligible.")
        );
        assert!(shown.body.contains("Directorio, 1st Eligible, may: Pass"));
        assert!(shown.body.contains("Government is now 2nd Eligible"));
        assert_eq!(view.next.as_ref().unwrap().options.len(), 3);
        assert!(
            view.next
                .as_ref()
                .unwrap()
                .options
                .iter()
                .any(|o| o == "Event")
        );
        assert!(shown.body.contains(" · Event."));
        for slot in [Slot::OperationOnly, Slot::OperationWithSpecialActivity] {
            assert!(!content(&view, slot).occupied);
        }

        let mut only = Game::opening();
        submit(&mut only, "terror");
        submit(&mut only, "end-terror");
        let view = only.observe();
        let shown = content(&view, Slot::OperationOnly);
        assert!(shown.occupied);
        assert_eq!((shown.title, shown.rule), ("Operation Only", "R 2.3.4"));
        assert!(shown.body.contains(
            "Directorio, 2nd Eligible, may: Pass (+1 if executed later) · Limited Operation."
        ));
        assert!(shown.body.contains("local policy"));
        assert!(
            !view
                .next
                .as_ref()
                .unwrap()
                .options
                .iter()
                .any(|o| o == "Event")
        );
        assert!(!shown.body.contains("Event"));

        let mut kidnap = Game::opening();
        submit(&mut kidnap, "terror");
        submit(&mut kidnap, "kidnap");
        submit(&mut kidnap, "kidnap:roll");
        submit(&mut kidnap, "terror:santiago");
        submit(&mut kidnap, "end-terror");
        let view = kidnap.observe();
        let shown = content(&view, Slot::OperationWithSpecialActivity);
        assert!(shown.occupied);
        assert_eq!(shown.title, "Operation + Special Activity");
        assert!(shown.body.contains("(Terror with Kidnap)"));
        assert!(
            view.next
                .as_ref()
                .unwrap()
                .options
                .iter()
                .any(|o| o == "Event")
        );
        assert!(shown.body.ends_with(
            "Directorio, 2nd Eligible, may: Pass (+1 if executed later) · Limited Operation · Event."
        ));

        // An occupied box words the authority's record: changed there to facts the
        // old fixed sentences ("+1 Resource", "stays Eligible", a fixed option list)
        // would misstate, the tag follows.
        let mut changed = pass.observe();
        changed.recorded_resources[0].from = 49;
        changed.recorded_resources[0].to = 49;
        changed.eligibility.m26_next_card = Some(false);
        changed.next.as_mut().unwrap().options = vec!["Event".into()];
        let shown = content(&changed, Slot::Pass);
        assert!(
            shown.body.starts_with(
                "26July Passed: +0 Resources, and it becomes Ineligible after this card. Directorio, 1st Eligible, may: Event."
            ),
            "{}",
            shown.body
        );

        let opening = Game::opening().observe();
        for slot in Slot::ALL {
            let shown = content(&opening, slot);
            assert!(!shown.occupied);
            assert!(!shown.body.contains("26July"), "{}", shown.body);
            assert_eq!(shown.note, AFTER_THE_CARD);
        }
        assert!(
            content(&opening, Slot::Pass)
                .body
                .contains("+1 Resource (Government +3)")
        );
        assert!(
            content(&opening, Slot::OperationOnly)
                .body
                .contains("Limited Operation")
        );
        assert!(
            content(&opening, Slot::OperationWithSpecialActivity)
                .body
                .contains("Event")
        );
    }

    /// Tags state facts and never recommend a move.
    #[test]
    fn no_tag_recommends_a_move() {
        let mut games = vec![Game::opening()];
        let mut pass = Game::opening();
        submit(&mut pass, "pass");
        games.push(pass);
        let mut only = Game::opening();
        submit(&mut only, "terror");
        submit(&mut only, "end-terror");
        games.push(only);
        for game in games {
            for slot in Slot::ALL {
                let shown = content(&game.observe(), slot);
                let lower = format!("{} {}", shown.body, shown.note).to_lowercase();
                for advice in [
                    "should",
                    "recommend",
                    "best",
                    "better",
                    "consider",
                    "suggest",
                    "you ",
                ] {
                    assert!(!lower.contains(advice), "{advice}: {lower}");
                }
            }
        }
    }

    /// The layout constants convert to the reference: the panel sits 20 px right of
    /// the map's left edge (40) and under its header (136 + 40), 232 px wide; the
    /// boxes step by 36 px; the tag opens 8 px right of the panel.
    #[test]
    fn layout_constants_convert_to_the_reference() {
        let [x, y, w, h] = rect(0., 0., fig::WIDTH, fig::HEIGHT);
        assert!((x - 60.).abs() < 1e-3);
        assert!((y - 177.78).abs() < 0.01);
        assert!((w - 232.).abs() < 1e-3);
        assert!((h - 383.96).abs() < 0.01);
        for pair in fig::BOX_TOPS.windows(2) {
            assert!((r(pair[1] - pair[0]) - 36.).abs() < 1e-3);
        }
        assert!((r(fig::TAG_X - fig::WIDTH) - 8.).abs() < 1e-3);
        assert!((r(fig::LEADER[2]) - 8.).abs() < 1e-3);
        // The opening layout ends where the shorter first row ends.
        assert!((fig::ACTED_ROW_H - fig::ROW_H + fig::OPEN_SHIFT).abs() < 1e-3);
    }

    #[test]
    fn box_names_serialize_as_the_seat_reads_them() {
        for slot in Slot::ALL {
            assert_eq!(serde_json::to_value(slot).unwrap(), slot.name());
        }
    }
}
