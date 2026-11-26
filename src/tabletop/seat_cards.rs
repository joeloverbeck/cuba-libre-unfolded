//! The seat controls in the window (#206): selected A1 · Tarjetas, a place card for each
//! faction on the sea along the bottom, with the sheet that shows a seat's line to copy,
//! and the strip that stands where the decision panel's commit was while an agent's
//! faction decides (A2's panel, "same panel, inert").
//!
//! Layout authority (foundation 4): `art/atlas/atlas-seats.figcraft` owns the structure,
//! places and sizes; [`fig`] copies its frame's numbers, which `atlas-seats.layout.json`
//! records for the native comparison (SEATS.md). The cards and the marks are VectorCraft
//! exports and the plates FigCraft's (`seats.sha256`).
//!
//! What a seat's state is, and who may be handed over, is `faction_seats`'s; this module
//! draws it, and what a control was pressed waits in `Interface::presses` until that
//! frame's `faction_seats::serve` reads it. The controls work at any time, over the
//! transfer modal's scrim and while a card or a die is in the air.
use super::palette::{INK, PAPER, PRIMARY, QUIET};
use super::*;
use crate::faction_seats::SeatState;
use bevy::text::{Justify, LineBreak, LineHeight};
use bevy::ui::widget::NodeImageMode;
use serde_json::json;

/// Node coordinates of `art/atlas/atlas-seats.figcraft`'s 16:9 frame, in its units: the
/// owner's display. A card's parts are measured from the card's own corner, the sheet's
/// from the sheet's.
mod fig {
    /// The main view's area, from whose corner the row is measured, and its width.
    pub const AREA: &str = "seats";
    pub const ROW: f32 = 1940.;
    pub const CARD: [f32; 2] = [470., 190.];
    /// From one card's left to the next one's.
    pub const PITCH: f32 = 490.;
    /// How far the deciding faction's card stands forward.
    pub const FORWARD: f32 = 36.;
    /// The ink edge of the card that stands forward, round its face.
    pub const EDGE: [f32; 4] = [0., 20., 470., 170.];
    pub const EMBLEM: [f32; 4] = [22., 42., 60., 60.];
    pub const NAME: [f32; 3] = [98., 38., 34.];
    pub const STATE: [f32; 3] = [98., 82., 25.];
    pub const CHAIR: [f32; 4] = [376., 36., 76., 76.];
    /// The buttons: the first one's left, then the top and the height of both.
    pub const BUTTONS: [f32; 3] = [22., 130., 48.];
    pub const WIDE: f32 = 190.;
    pub const TAKE_BACK: f32 = 170.;
    pub const BUTTON_GAP: f32 = 14.;
    pub const BUTTON_SIZE: f32 = 24.;
    pub const BUTTON_LINE: f32 = 30.;
    /// The line sheet: its width, and how far its foot is from the area's top.
    pub const SHEET: f32 = 900.;
    pub const SHEET_FOOT: f32 = -6.;
    pub const SHEET_RULE: f32 = 8.;
    /// How far the sheet's shadow reaches past it: left, top, right, bottom.
    pub const SHEET_SHADOW: [f32; 4] = [42., 32., 42., 52.];
    pub const SHEET_PAD: f32 = 28.;
    /// Above the title, above the line's box, and above the button.
    pub const SHEET_GAPS: [f32; 3] = [18., 14., 12.];
    pub const TITLE: [f32; 2] = [28., 34.];
    /// The line's box: its least height, which holds three lines, and its padding to the
    /// sides, above and below. Then the line's type and line height.
    pub const BOX: [f32; 4] = [106., 16., 12., 10.];
    pub const LINE: [f32; 2] = [23., 28.];
    pub const NOTE: [f32; 2] = [6., 24.];
    pub const COPY_AGAIN: [f32; 2] = [200., 46.];
    /// The strip, in the frame: the plate, its mark, title, note and button.
    pub const STRIP: [f32; 4] = [2969., 1424., 764., 118.];
    pub const STRIP_MARK: [f32; 4] = [2993., 1447., 72., 72.];
    pub const STRIP_TITLE: [f32; 3] = [3085., 1438., 30.];
    pub const STRIP_NOTE: [f32; 3] = [3085., 1480., 24.];
    pub const STRIP_BUTTON: [f32; 4] = [3545., 1453., 164., 60.];
    /// The line above the strip.
    pub const HINT: [f32; 4] = [2969., 1380., 764., 24.];
    pub const FRAME: [f32; 2] = [3840., 2160.];
}

/// What the human pressed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    HandOver(Faction),
    TakeBack(Faction),
    /// Copy the line that seats an agent as the faction.
    Copy(Faction),
}

/// The open line sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Line {
    pub faction: Faction,
    pub text: String,
    /// Why it is not on the clipboard, when it is not.
    pub failed: Option<String>,
}

impl Line {
    fn note(&self) -> String {
        match &self.failed {
            None => "Copied. One session plays one seat.".into(),
            Some(why) => format!("Not copied: {why}"),
        }
    }
}

/// Whether the development seat's component `name` is a control of this module, which
/// works at any time.
pub(super) fn is_control(name: &str) -> bool {
    name.starts_with("seat:") || name == COPY_AGAIN || name == STRIP_TAKE_BACK
}

const COPY_AGAIN: &str = "seat-line:copy";
const STRIP_TAKE_BACK: &str = "seat-strip:take-back";

/// The agent's faction that decides now, with its seat's state. `None` while the human's
/// faction decides, and while only the card is to be turned, which is the human's.
pub(super) fn deciding(view: &Observation, ui: &Interface) -> Option<(Faction, SeatState)> {
    let state = ui.seats[view.owner.index()];
    (state != SeatState::Human && !view.choices.iter().any(Choice::turns_the_card))
        .then_some((view.owner, state))
}

/// The same, while the strip is shown for it: a choice of the human's own that they
/// pinned, 26July's withdrawal while another giver answers, has its commit there instead.
pub(super) fn strip(view: &Observation, ui: &Interface) -> Option<(Faction, SeatState)> {
    let own = ui.choice.pinned.as_deref().is_some_and(|pinned| {
        view.choices
            .iter()
            .any(|c| c.action() == pinned && c.supported && ui.plays(c.owner))
    });
    deciding(view, ui).filter(|_| !own)
}

/// What a card says of its seat, and in what colour.
fn state_words(state: SeatState) -> (String, Color) {
    match state {
        SeatState::Human => ("You play it".into(), INK),
        SeatState::Awaiting => ("Awaiting an agent".into(), QUIET),
        SeatState::AtTable => ("Agent at the table".into(), QUIET),
        SeatState::Thinking => ("Thinking".into(), INK),
        SeatState::Silent { minutes } => (format!("Silent · {minutes} min"), PRIMARY),
        SeatState::BuildEnds => ("Build ends here".into(), QUIET),
    }
}

fn state_slug(state: SeatState) -> &'static str {
    match state {
        SeatState::Human => "human",
        SeatState::Awaiting => "awaiting",
        SeatState::AtTable => "at-table",
        SeatState::Thinking => "thinking",
        SeatState::Silent { .. } => "silent",
        SeatState::BuildEnds => "build-ends",
    }
}

/// A card's buttons: each one's seat name, words, press and width, and whether it is the
/// filled one.
fn buttons(faction: Faction, state: SeatState) -> Vec<(String, &'static str, Press, f32, bool)> {
    let name = |what: &str| format!("seat:{}:{what}", slug(faction));
    let take_back = (
        name("take-back"),
        "Take back",
        Press::TakeBack(faction),
        fig::TAKE_BACK,
        false,
    );
    match state {
        SeatState::Human => vec![(
            name("hand-over"),
            "Hand over",
            Press::HandOver(faction),
            fig::WIDE,
            false,
        )],
        SeatState::Awaiting => vec![
            (
                name("copy"),
                "Copy its line",
                Press::Copy(faction),
                fig::WIDE,
                true,
            ),
            take_back,
        ],
        _ => vec![take_back],
    }
}

/// The strip's plate, mark and title colour, and its note.
fn strip_look(state: SeatState) -> (&'static str, &'static str, Color, &'static str) {
    match state {
        SeatState::Awaiting => (
            "atlas-seat-strip-quiet.png",
            "atlas-seat-round-awaiting.png",
            QUIET,
            "No agent has joined yet.",
        ),
        SeatState::BuildEnds => (
            "atlas-seat-strip-quiet.png",
            "atlas-seat-round-build-ends.png",
            QUIET,
            "No option here is in this build.",
        ),
        SeatState::Silent { .. } => (
            "atlas-seat-strip-silent.png",
            "atlas-seat-round-silent.png",
            PRIMARY,
            "Its choice is still open to it.",
        ),
        // A deciding seat is in one of these four states; a session that holds it and is
        // at the table has the decision.
        SeatState::Thinking | SeatState::AtTable | SeatState::Human => (
            "atlas-seat-strip-thinking.png",
            "atlas-seat-round-thinking.png",
            INK,
            "Its agent holds the choice.",
        ),
    }
}

/// The controls Tab moves through: each card's buttons, then the open sheet's.
pub(super) fn focusable(ui: &Interface) -> Vec<(String, Action)> {
    let mut stops: Vec<(String, Action)> = Faction::ALL
        .into_iter()
        .flat_map(|faction| buttons(faction, ui.seats[faction.index()]))
        .map(|(name, _, press, ..)| (name, Action::Seat(press)))
        .collect();
    if let Some(line) = &ui.line {
        stops.push((COPY_AGAIN.into(), Action::Seat(Press::Copy(line.faction))));
    }
    stops
}

/// The strip's stop at the panel's foot, while it has one.
pub(super) fn strip_focusable(view: &Observation, ui: &Interface) -> Option<(String, Action)> {
    let (faction, state) = strip(view, ui)?;
    (state != SeatState::BuildEnds).then(|| {
        (
            STRIP_TAKE_BACK.into(),
            Action::Seat(Press::TakeBack(faction)),
        )
    })
}

/// Where keyboard focus goes after `press`: the card's button that replaces the pressed
/// one, or the same one after copying.
pub(super) fn focus_after(press: Press) -> Option<String> {
    let name = |faction: Faction, what: &str| format!("seat:{}:{what}", slug(faction));
    match press {
        Press::HandOver(faction) => Some(name(faction, "copy")),
        Press::TakeBack(faction) => Some(name(faction, "hand-over")),
        Press::Copy(_) => None,
    }
}

/// Names a drawn part by its node name in `atlas-seats.figcraft`, with its words.
#[derive(Component)]
pub(super) struct Part(String, Option<String>);

/// Where the parts are drawn: the frame unit in window pixels, the point they are
/// measured from, and their layer.
#[derive(Clone, Copy)]
struct Place {
    unit: f32,
    origin: Vec2,
    z: i32,
}

impl Place {
    fn at(self, by: Vec2) -> Self {
        Self {
            origin: self.origin + by * self.unit,
            ..self
        }
    }
    fn picture(
        self,
        commands: &mut Commands,
        art: &Art,
        key: &str,
        rect: [f32; 4],
        z: i32,
    ) -> Entity {
        picture(commands, art, key, rect, self.unit, self.origin, self.z + z)
    }
    /// Text at `[left, top, width]`, in a size.
    #[allow(clippy::too_many_arguments)] // Content, face, colour, place and type.
    fn words(
        self,
        commands: &mut Commands,
        art: &Art,
        content: &str,
        bold: bool,
        color: Color,
        [x, y, w]: [f32; 3],
        size: f32,
        z: i32,
    ) -> Entity {
        let face = if bold { &art.bold } else { &art.font };
        styled(
            commands,
            art,
            Prose::named(content),
            face,
            color,
            [x, y, w, size],
            self.unit,
            self.origin,
            self.z + z,
        )
    }
    /// A button's plate, with its control and its words centred in it.
    fn button(
        self,
        commands: &mut Commands,
        art: &Art,
        name: &str,
        (words, press): (&str, Press),
        rect: [f32; 4],
        filled: bool,
    ) {
        let [x, y, w, h] = rect;
        let plate = self.picture(commands, art, BUTTON_PLATES[usize::from(filled)], rect, 3);
        commands
            .entity(plate)
            .remove::<Pickable>()
            .insert((
                Control {
                    name: name.into(),
                    choice: false,
                    action: Action::Seat(press),
                },
                Part(name.into(), None),
            ))
            .entry::<ImageNode>()
            .and_modify({
                let mode = sliced(BUTTON_SLICE, self.unit);
                move |mut image| image.image_mode = mode
            });
        commands.entity(plate).observe(click);
        let label = self.words(
            commands,
            art,
            words,
            true,
            if filled { PAPER } else { INK },
            [x, y + (h - fig::BUTTON_LINE) / 2., w],
            fig::BUTTON_SIZE,
            4,
        );
        commands.entity(label).insert((
            TextLayout::justify(Justify::Center),
            LineHeight::Px(fig::BUTTON_LINE * self.unit),
            Part(format!("{name}-label"), Some(words.into())),
        ));
    }
}

const BUTTON_PLATES: [&str; 2] = ["atlas-seat-button.png", "atlas-seat-button-primary.png"];
/// The plates' corner slices in frame units, clear of their corners' radius and, on the
/// sheet, of its shadow. The exports are 2 px per unit.
const BUTTON_SLICE: f32 = 8.;
const BOX_SLICE: f32 = 8.;
const SHEET_SLICE: f32 = 90.;

/// Draws the four cards and the open sheet in the main view's `seats` area. The transfer
/// modal's scrim covers the area, and the seats stay over it.
pub(super) fn draw(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    layout: &areas::Layout,
    window: &Window,
) {
    let area = layout.area(fig::AREA, window);
    let modal = transfer::shown(view.transfer.as_ref(), view.owner, ui);
    let row = Place {
        unit: area.width() / fig::ROW,
        origin: area.min,
        z: if modal { transfer::LAYER } else { 0 },
    };
    let turning = view.choices.iter().any(Choice::turns_the_card);
    for faction in Faction::ALL {
        let forward = faction == view.owner && !turning;
        let by = Vec2::new(
            faction.index() as f32 * fig::PITCH,
            if forward { 0. } else { fig::FORWARD },
        );
        card(
            commands,
            art,
            faction,
            ui.seats[faction.index()],
            forward,
            row.at(by),
        );
    }
    if let Some(line) = &ui.line {
        sheet(commands, art, line, row, window);
    }
}

fn card(
    commands: &mut Commands,
    art: &Art,
    faction: Faction,
    state: SeatState,
    forward: bool,
    place: Place,
) {
    let seat = format!("seat:{}", slug(faction));
    let part = |commands: &mut Commands, entity: Entity, what: &str, words: Option<&str>| {
        commands
            .entity(entity)
            .insert(Part(format!("{seat}{what}"), words.map(Into::into)));
    };
    let [w, h] = fig::CARD;
    let key = format!("atlas-seat-card-{}.png", slug(faction));
    let plate = place.picture(commands, art, &key, [0., 0., w, h], 2);
    part(commands, plate, "", None);
    if forward {
        let edge = place.picture(commands, art, "atlas-seat-edge.png", fig::EDGE, 3);
        part(commands, edge, "-edge", None);
    }
    let emblem = place.picture(commands, art, &emblems::image(faction), fig::EMBLEM, 3);
    part(commands, emblem, "-emblem", None);
    let chair = format!("atlas-seat-chair-{}.png", state_slug(state));
    let chair = place.picture(commands, art, &chair, fig::CHAIR, 3);
    part(commands, chair, "-chair", None);
    let room = fig::CHAIR[0] - fig::NAME[0];
    let [x, y, size] = fig::NAME;
    let name = place.words(
        commands,
        art,
        faction.name(),
        true,
        INK,
        [x, y, room],
        size,
        4,
    );
    part(commands, name, "-name", Some(faction.name()));
    let (words, color) = state_words(state);
    let [x, y, size] = fig::STATE;
    let says = place.words(commands, art, &words, false, color, [x, y, room], size, 4);
    part(commands, says, "-state", Some(&words));
    let [mut left, top, height] = fig::BUTTONS;
    for (name, words, press, width, filled) in buttons(faction, state) {
        let rect = [left, top, width, height];
        place.button(commands, art, &name, (words, press), rect, filled);
        left += width + fig::BUTTON_GAP;
    }
}

/// The sheet of `line`: over its own card, kept inside the row. It grows upward with the
/// line.
fn sheet(commands: &mut Commands, art: &Art, line: &Line, row: Place, window: &Window) {
    let u = row.unit;
    let left = (line.faction.index() as f32 * fig::PITCH).min(fig::ROW - fig::SHEET);
    let foot = row.origin.y + fig::SHEET_FOOT * u;
    let z = row.z + 5;
    let node = |commands: &mut Commands, parent: Entity, node: Node| {
        commands
            .spawn((node, Pickable::IGNORE, ChildOf(parent)))
            .id()
    };
    let words = |commands: &mut Commands,
                 parent: Entity,
                 name: &str,
                 content: &str,
                 bold: bool,
                 color: Color,
                 [size, line]: [f32; 2]| {
        let face = if bold { &art.bold } else { &art.font };
        let words = commands.spawn((
            Text::new(content),
            emblems::Plain(content.into()),
            TextFont {
                font: face.clone().into(),
                font_size: (size * u).into(),
                ..default()
            },
            LineHeight::Px(line * u),
            TextLayout {
                linebreak: LineBreak::WordOrCharacter,
                ..default()
            },
            TextColor(color),
            Part(name.into(), Some(content.into())),
            Pickable::IGNORE,
            ChildOf(parent),
        ));
        words.id()
    };
    let sheet = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(row.origin.x + left * u),
                bottom: px(window.height() - foot),
                width: px(fig::SHEET * u),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(z),
            // The sheet takes the pointer over its own rectangle, as an area does.
            Control {
                name: "seat-line".into(),
                choice: false,
                action: Action::Area,
            },
            Part("seat-line".into(), None),
        ))
        .id();
    let [l, t, r, b] = fig::SHEET_SHADOW.map(|reach| px(-reach * u));
    commands.spawn((
        ImageNode::new(art.images["atlas-seat-line.png"].clone()).with_mode(sliced(SHEET_SLICE, u)),
        Node {
            position_type: PositionType::Absolute,
            left: l,
            top: t,
            right: r,
            bottom: b,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(sheet),
    ));
    commands.spawn((
        ImageNode::new(art.images[popover::plate::RULE_PRIMARY].clone())
            .with_mode(NodeImageMode::Stretch),
        Node {
            height: px(fig::SHEET_RULE * u),
            flex_shrink: 0.,
            ..default()
        },
        Part("seat-line-rule".into(), None),
        Pickable::IGNORE,
        ChildOf(sheet),
    ));
    let [above_title, above_box, above_button] = fig::SHEET_GAPS.map(|gap| px(gap * u));
    let pad = px(fig::SHEET_PAD * u);
    let title = node(
        commands,
        sheet,
        Node {
            margin: UiRect::new(pad, pad, above_title, px(0.)),
            height: px(fig::TITLE[1] * u),
            ..default()
        },
    );
    words(
        commands,
        title,
        "seat-line-title",
        "Paste this in a terminal, in a Claude Code or Codex session",
        true,
        INK,
        fig::TITLE,
    );
    let [least, side, top, foot_pad] = fig::BOX;
    let boxed = node(
        commands,
        sheet,
        Node {
            margin: UiRect::new(pad, pad, above_box, px(0.)),
            padding: UiRect::new(px(side * u), px(side * u), px(top * u), px(foot_pad * u)),
            min_height: px(least * u),
            ..default()
        },
    );
    commands
        .entity(boxed)
        .insert(Part("seat-line-box".into(), None));
    commands.spawn((
        ImageNode::new(art.images["atlas-seat-line-box.png"].clone())
            .with_mode(sliced(BOX_SLICE, u)),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            left: px(0.),
            top: px(0.),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(boxed),
    ));
    words(
        commands,
        boxed,
        "seat-line-text",
        &line.text,
        false,
        INK,
        fig::LINE,
    );
    let foot = node(
        commands,
        sheet,
        Node {
            margin: UiRect::new(pad, pad, above_button, above_button),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
    );
    let [above_note, size] = fig::NOTE;
    let note = node(
        commands,
        foot,
        Node {
            margin: UiRect::top(px(above_note * u)),
            ..default()
        },
    );
    words(
        commands,
        note,
        "seat-line-note",
        &line.note(),
        false,
        QUIET,
        [size, 29.],
    );
    let [w, h] = fig::COPY_AGAIN;
    let again = commands
        .spawn((
            ImageNode::new(art.images[BUTTON_PLATES[0]].clone()).with_mode(sliced(BUTTON_SLICE, u)),
            Node {
                width: px(w * u),
                height: px(h * u),
                flex_shrink: 0.,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Control {
                name: COPY_AGAIN.into(),
                choice: false,
                action: Action::Seat(Press::Copy(line.faction)),
            },
            Part(COPY_AGAIN.into(), None),
            ChildOf(foot),
        ))
        .observe(click)
        .id();
    let label = words(
        commands,
        again,
        "seat-line:copy-label",
        "Copy again",
        true,
        INK,
        [fig::BUTTON_SIZE, fig::BUTTON_LINE],
    );
    commands.entity(label).insert((
        TextLayout::justify(Justify::Center),
        Node {
            width: percent(100),
            ..default()
        },
    ));
}

/// Draws the strip where the panel's commit stands, with the line above it, for the
/// agent's faction that decides. `s` and `o` are the panel's reference pixel and offset.
pub(super) fn panel_strip(
    commands: &mut Commands,
    art: &Art,
    (faction, state): (Faction, SeatState),
    hint: Option<&str>,
    s: f32,
    o: Vec2,
) {
    let unit = s * HEIGHT / fig::FRAME[1];
    let place = Place {
        unit,
        // The panel's place, measured from the window's right edge.
        origin: o + Vec2::new(WIDTH * s - fig::FRAME[0] * unit, 0.),
        z: 0,
    };
    let part = |commands: &mut Commands, entity: Entity, name: &str, words: Option<&str>| {
        commands
            .entity(entity)
            .insert(Part(format!("seat-strip{name}"), words.map(Into::into)));
    };
    if let Some(hint) = hint {
        let [x, y, w, size] = fig::HINT;
        let line = place.words(commands, art, hint, false, QUIET, [x, y, w], size, 5);
        commands
            .entity(line)
            .insert(TextLayout::justify(Justify::Center));
        part(commands, line, "-hint", Some(hint));
    }
    let (plate, mark, color, note) = strip_look(state);
    let plate = place.picture(commands, art, plate, fig::STRIP, 3);
    part(commands, plate, "", None);
    let mark = place.picture(commands, art, mark, fig::STRIP_MARK, 4);
    part(commands, mark, "-mark", None);
    let room = fig::STRIP_BUTTON[0] - fig::STRIP_TITLE[0];
    let title = format!("{}: {}", faction.name(), state_words(state).0);
    let [x, y, size] = fig::STRIP_TITLE;
    let words = place.words(commands, art, &title, true, color, [x, y, room], size, 5);
    part(commands, words, "-title", Some(&title));
    let [x, y, size] = fig::STRIP_NOTE;
    let words = place.words(commands, art, note, false, INK, [x, y, room], size, 5);
    part(commands, words, "-note", Some(note));
    if state != SeatState::BuildEnds {
        place.button(
            commands,
            art,
            STRIP_TAKE_BACK,
            ("Take back", Press::TakeBack(faction)),
            fig::STRIP_BUTTON,
            false,
        );
    }
}

/// The line above the strip of a panel that lists options: how to read them, and who
/// chooses.
pub(super) fn hint(faction: Faction) -> String {
    format!(
        "Hover an option for its briefing. Only {}'s seat can choose.",
        faction.name()
    )
}

/// The drawn seats for the development seat: each part's rectangle in logical window
/// pixels and its words, and the open line sheet.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let mut parts = world
        .query::<(&Part, &ComputedNode, &bevy::ui::UiGlobalTransform)>()
        .iter(world)
        .map(|(part, node, at)| {
            (
                part.0.clone(),
                json!({"part": part.0, "text": part.1, "bounds": seat::logical_bounds(node, at)}),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    let line = world
        .resource::<Interface>()
        .line
        .as_ref()
        .map(|line| json!({"faction": line.faction, "text": line.text, "note": line.note()}));
    json!({
        "parts": parts.into_iter().map(|(_, part)| part).collect::<Vec<_>>(),
        "line": line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn record() -> Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-seats.layout.json")).unwrap()
    }

    fn rect(group: &str, name: &str) -> [f32; 4] {
        serde_json::from_value(record()[group][name].clone())
            .unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    /// The constants are the layout record's: each card's parts from its own corner, the
    /// sheet's from the sheet's, the strip's in the frame.
    #[test]
    fn the_numbers_are_the_layout_record_s() {
        let origin: [f32; 2] = serde_json::from_value(record()["origin"].clone()).unwrap();
        let main: Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let area: [f32; 4] =
            serde_json::from_value(main["areas"][fig::AREA]["bounds"].clone()).unwrap();
        assert_eq!(origin, [area[0], area[1]]);
        assert_eq!(area[2], fig::ROW);
        assert_eq!(area[2], 3. * fig::PITCH + fig::CARD[0]);
        assert_eq!(area[3], fig::CARD[1] + fig::FORWARD);
        // The frame draws Government waiting, 26July human, Directorio deciding and
        // Syndicate at the table.
        let drawn = [
            (Faction::Government, SeatState::Awaiting),
            (Faction::M26, SeatState::Human),
            (Faction::Directorio, SeatState::Thinking),
            (Faction::Syndicate, SeatState::AtTable),
        ];
        for (faction, state) in drawn {
            let seat = format!("seat:{}", slug(faction));
            let forward = faction == Faction::Directorio;
            let x = origin[0] + faction.index() as f32 * fig::PITCH;
            let y = origin[1] + if forward { 0. } else { fig::FORWARD };
            let at = |[dx, dy, w, h]: [f32; 4]| [x + dx, y + dy, w, h];
            let [w, h] = fig::CARD;
            assert_eq!(rect("parts", &seat), at([0., 0., w, h]));
            assert_eq!(rect("parts", &format!("{seat}-emblem")), at(fig::EMBLEM));
            assert_eq!(rect("parts", &format!("{seat}-chair")), at(fig::CHAIR));
            if forward {
                assert_eq!(rect("parts", &format!("{seat}-edge")), at(fig::EDGE));
            }
            for (what, place) in [("-name", fig::NAME), ("-state", fig::STATE)] {
                let [left, top, ..] = rect("text", &format!("{seat}{what}"));
                assert_eq!([left, top], [x + place[0], y + place[1]], "{seat}{what}");
            }
            let [mut left, top, height] = fig::BUTTONS;
            for (name, _, _, width, _) in buttons(faction, state) {
                assert_eq!(rect("parts", &name), at([left, top, width, height]));
                let [lx, ly, lw, lh] = rect("text", &format!("{name}-label"));
                let inset = (height - fig::BUTTON_LINE) / 2.;
                assert_eq!([lx, ly, lw, lh], at([left, top + inset, width, 30.]));
                left += width + fig::BUTTON_GAP;
            }
        }
        // The sheet over Government's card, with its three lines.
        let [x, y, w, h] = rect("parts", "seat-line");
        assert_eq!(
            [x, w, y + h],
            [origin[0], fig::SHEET, origin[1] + fig::SHEET_FOOT]
        );
        assert_eq!(rect("parts", "seat-line-rule"), [x, y, w, fig::SHEET_RULE]);
        let [above_title, above_box, above_button] = fig::SHEET_GAPS;
        let title = y + fig::SHEET_RULE + above_title;
        assert_eq!(rect("text", "seat-line-title")[1], title);
        assert_eq!(rect("text", "seat-line-title")[3], fig::TITLE[1]);
        let top = title + fig::TITLE[1] + above_box;
        let [bx, by, bw, bh] = rect("parts", "seat-line-box");
        assert_eq!(
            [bx, by, bw, bh],
            [
                x + fig::SHEET_PAD,
                top,
                w - 2. * fig::SHEET_PAD,
                fig::BOX[0]
            ]
        );
        let [tx, ty, tw, th] = rect("text", "seat-line-text");
        assert_eq!([tx, ty], [bx + fig::BOX[1], by + fig::BOX[2]]);
        assert_eq!([tw, th], [bw - 2. * fig::BOX[1], 3. * fig::LINE[1]]);
        assert_eq!(bh, fig::BOX[2] + th + fig::BOX[3]);
        let [cx, cy, cw, ch] = rect("parts", COPY_AGAIN);
        assert_eq!([cw, ch], fig::COPY_AGAIN);
        assert_eq!([cx + cw, cy], [bx + bw, by + bh + above_button]);
        assert_eq!(cy + ch + above_button, y + h);
        assert_eq!(rect("text", "seat-line-note")[1], cy + fig::NOTE[0]);
        // The strip, in the decision panel.
        assert_eq!(rect("parts", "seat-strip"), fig::STRIP);
        assert_eq!(rect("parts", "seat-strip-mark"), fig::STRIP_MARK);
        assert_eq!(rect("parts", STRIP_TAKE_BACK), fig::STRIP_BUTTON);
        for (name, place) in [
            ("seat-strip-title", fig::STRIP_TITLE),
            ("seat-strip-note", fig::STRIP_NOTE),
        ] {
            let [left, top, ..] = rect("text", name);
            assert_eq!([left, top], [place[0], place[1]], "{name}");
        }
        assert_eq!(rect("text", "seat-strip-hint")[..3], fig::HINT[..3]);
    }

    /// Every export the cards, the sheet and the strip place is the record's, with its
    /// size, and admitted.
    #[test]
    fn the_exports_are_the_record_s_and_admitted() {
        for (file, size) in record()["exports"].as_object().unwrap() {
            let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
            let image = image::load_from_memory(&bytes).unwrap();
            let size: [u32; 2] = serde_json::from_value(size.clone()).unwrap();
            assert_eq!([image.width(), image.height()], size, "{file}");
            assert!(
                crate::assets::SEATS_MANIFEST.contains(&format!("art/atlas/{file}")),
                "{file} is not admitted"
            );
        }
    }

    const STATES: [SeatState; 6] = [
        SeatState::Human,
        SeatState::Awaiting,
        SeatState::AtTable,
        SeatState::Thinking,
        SeatState::Silent { minutes: 12 },
        SeatState::BuildEnds,
    ];

    /// A card says each state in the layout's words and has its own chair, and a seat can
    /// be taken back in every state but the human's, and handed over in that one.
    #[test]
    fn each_state_has_its_words_its_mark_and_its_buttons() {
        let words = [
            "You play it",
            "Awaiting an agent",
            "Agent at the table",
            "Thinking",
            "Silent · 12 min",
            "Build ends here",
        ];
        for (state, words) in STATES.into_iter().zip(words) {
            assert_eq!(state_words(state).0, words);
            let chair = format!("atlas-seat-chair-{}.png", state_slug(state));
            assert!(crate::assets::SEATS_MANIFEST.contains(&chair), "{chair}");
            let presses: Vec<Press> = buttons(Faction::M26, state)
                .into_iter()
                .map(|(_, _, press, ..)| press)
                .collect();
            let expected = match state {
                SeatState::Human => vec![Press::HandOver(Faction::M26)],
                SeatState::Awaiting => {
                    vec![Press::Copy(Faction::M26), Press::TakeBack(Faction::M26)]
                }
                _ => vec![Press::TakeBack(Faction::M26)],
            };
            assert_eq!(presses, expected, "{state:?}");
            let (plate, mark, ..) = strip_look(state);
            for file in [plate, mark] {
                assert!(crate::assets::SEATS_MANIFEST.contains(file), "{file}");
            }
        }
    }

    /// The strip is an agent's deciding faction's, whoever else the human plays; the card
    /// is the human's to turn, and their own withdrawal has its commit.
    #[test]
    fn the_strip_is_the_deciding_agents() {
        let mut game = crate::terror_chosen(Game::opening());
        let mut ui = Interface::default();
        assert_eq!(strip(&game.observe(), &ui), None);
        ui.seats[Faction::M26.index()] = SeatState::Thinking;
        let m26 = Some((Faction::M26, SeatState::Thinking));
        assert_eq!(strip(&game.observe(), &ui), m26);
        // A step of its own that the human pinned to read stays without a commit.
        ui.choice.pinned = Some("end-terror".into());
        assert_eq!(strip(&game.observe(), &ui), m26);
        assert_eq!(
            strip_focusable(&game.observe(), &ui).unwrap().0,
            STRIP_TAKE_BACK
        );

        // The human's 26July asks an agent's Government: its withdrawal is the human's.
        ui.seats = [SeatState::Human; 4];
        ui.seats[Faction::Government.index()] = SeatState::Awaiting;
        game.apply("transfer").unwrap();
        game.apply("ask:0:3").unwrap();
        ui.choice.pinned = None;
        let view = game.observe();
        assert_eq!(
            strip(&view, &ui),
            Some((Faction::Government, SeatState::Awaiting))
        );
        ui.choice.pinned = Some("end-terror".into());
        assert_eq!(strip(&view, &ui), None);
        assert!(strip_focusable(&view, &ui).is_none());
        ui.choice.pinned = Some("transfer".into());
        assert!(strip(&view, &ui).is_some(), "no choice of the human's");
    }

    /// Turning the card is the human's, whichever faction the authority names for it.
    #[test]
    fn the_card_is_the_humans() {
        let mut game = crate::terror_chosen(Game::opening_with_seed(2));
        for action in ["kidnap", "kidnap:roll", "terror:santiago", "end-terror"] {
            game.apply(action).unwrap();
        }
        let mut ui = Interface {
            seats: [SeatState::Thinking; 4],
            ..default()
        };
        assert_eq!(
            deciding(&game.observe(), &ui).map(|d| d.0),
            Some(Faction::Directorio)
        );
        for action in ["event:shaded", "resolve-shaded"] {
            game.apply(action).unwrap();
        }
        assert_eq!(deciding(&game.observe(), &ui), None);
        ui.seats = [SeatState::Human; 4];
        game.apply("advance-card").unwrap();
        assert_eq!(deciding(&game.observe(), &ui), None);
    }
}
