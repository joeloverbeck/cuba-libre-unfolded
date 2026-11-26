//! A faction's public words stay on the table while the human reads them.
//! The selected Broadsheet frame supplies the page geometry and typography.
use super::{broadsheet, *};
use bevy::input::{
    keyboard::KeyboardInput,
    mouse::{MouseScrollUnit, MouseWheel},
};
use bevy::text::{LineBreak, TextLayout};
use bevy::ui::widget::NodeImageMode;
use serde_json::json;
use std::collections::{BTreeSet, VecDeque};

const LAYER: i32 = 55;
const LINE: f32 = 73.;
const BODY: [f32; 4] = [370., 568., 1775., 580.];
const RETURN_BOX: [f32; 4] = [1658., 1258., 480., 94.];
pub(super) const INBOX: &str = "proclamations";
pub(super) const RETURN: &str = "proclamation:return";

#[derive(Clone, Copy)]
pub(super) enum Press {
    Open,
    Return,
    Previous,
    Next,
}

#[derive(Clone, Copy)]
struct Open {
    index: usize,
    archive: bool,
}

#[derive(Default)]
pub(super) struct State {
    read: BTreeSet<usize>,
    queued: VecDeque<usize>,
    open: Option<Open>,
    scroll: f32,
}

impl State {
    pub(super) fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// The end of the authoritative record just received a faction publication.
    pub(super) fn receive(&mut self, index: usize) {
        self.queued.push_back(index);
        if self.open.is_none() {
            self.open = self.next_delivery();
        }
    }

    fn next_delivery(&mut self) -> Option<Open> {
        self.queued.pop_front().map(|index| Open {
            index,
            archive: false,
        })
    }

    fn press(&mut self, press: Press, count: usize) -> bool {
        let next = match (press, self.open) {
            (Press::Open, None) if count > 0 => Some(Open {
                index: self.queued.front().copied().unwrap_or(0),
                archive: true,
            }),
            (Press::Return, Some(_)) => None,
            (Press::Previous, Some(open)) if open.archive && open.index > 0 => Some(Open {
                index: open.index - 1,
                ..open
            }),
            (Press::Next, Some(open)) if open.archive && open.index + 1 < count => Some(Open {
                index: open.index + 1,
                ..open
            }),
            (Press::Next, Some(open)) if !open.archive && !self.queued.is_empty() => {
                self.next_delivery()
            }
            _ => return false,
        };
        if let Some(open) = self.open {
            self.read.insert(open.index);
            self.queued.retain(|queued| *queued != open.index);
        }
        self.open = next;
        self.scroll = 0.;
        true
    }

    fn unread(&self, count: usize) -> usize {
        count - self.read.len()
    }
}

pub(super) fn press(ui: &mut Interface, press: Press, count: usize) {
    ui.dirty |= ui.proclamation_reader.press(press, count);
}

#[derive(Component)]
pub(super) struct Body;

pub(super) fn keys(
    play: Res<Play>,
    mut ui: ResMut<Interface>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut keyboard: ResMut<Messages<KeyboardInput>>,
    mut wheel: MessageReader<MouseWheel>,
    body: Query<&ComputedNode, With<Body>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Some(session) = &play.session else {
        return;
    };
    let count = session.proclamations().len();
    if !ui.proclamation_reader.is_open() {
        wheel.clear();
        if keys.just_pressed(KeyCode::KeyR)
            && ui.editions.open().is_none()
            && ui.line.is_none()
            && count > 0
        {
            press(&mut ui, Press::Open, count);
        }
        return;
    }

    const DISMISS: [KeyCode; 4] = [
        KeyCode::Escape,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
    ];
    if keys.any_just_pressed(DISMISS) {
        press(&mut ui, Press::Return, count);
        wheel.clear();
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        press(&mut ui, Press::Previous, count);
        wheel.clear();
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        press(&mut ui, Press::Next, count);
        wheel.clear();
    } else {
        let scale = broadsheet::Layout::new(&window).scale;
        let lines = f32::from(keys.just_pressed(KeyCode::ArrowDown))
            - f32::from(keys.just_pressed(KeyCode::ArrowUp));
        let scroll = lines * LINE * scale
            - wheel
                .read()
                .map(|event| match event.unit {
                    MouseScrollUnit::Line => event.y * LINE * scale,
                    MouseScrollUnit::Pixel => event.y,
                })
                .sum::<f32>();
        if scroll != 0.
            && let Ok(node) = body.single()
        {
            let max = (node.content_size().y - node.size().y).max(0.) * node.inverse_scale_factor()
                / scale;
            ui.proclamation_reader.scroll =
                (ui.proclamation_reader.scroll + scroll / scale).clamp(0., max);
            ui.dirty = true;
        }
    }

    let pressed = keys.get_just_pressed().copied().collect::<Vec<_>>();
    for key in pressed {
        if !faction_seats::KEYS.contains(&key) {
            keys.clear_just_pressed(key);
        }
    }
    keyboard.clear();
}

pub(super) fn draw(
    commands: &mut Commands,
    art: &Art,
    session: Option<&Session>,
    ui: &Interface,
    window: &Window,
) {
    let Some(session) = session else {
        return;
    };
    let documents = session.proclamations();
    if documents.is_empty() {
        return;
    }
    let layout = broadsheet::Layout::new(window);
    let state = &ui.proclamation_reader;
    let Some(open) = state.open else {
        // The archive is above the seat cards, clear of the edition pile and the transfer.
        let unread = state.unread(documents.len());
        let label = if unread == 0 {
            "Proclamations · R".to_owned()
        } else {
            format!("Proclamations · {unread} unread · R")
        };
        button(
            commands,
            art,
            layout,
            [820., 1420., 680., 82.],
            (INBOX, Press::Open),
            &label,
            false,
            transfer::LAYER + 2,
        );
        return;
    };
    let document = &documents[open.index];
    commands.spawn((
        UiRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(LAYER),
        // The full-window hit surface keeps the table below the page from taking input.
        Pickable::default(),
    ));
    commands.spawn((
        UiRoot,
        ImageNode::new(art.images[broadsheet::SCRIM].clone()).with_mode(NodeImageMode::Stretch),
        Node {
            position_type: PositionType::Absolute,
            top: px(206. * layout.scale),
            width: percent(100),
            bottom: px(0.),
            ..default()
        },
        GlobalZIndex(LAYER),
        Pickable::IGNORE,
    ));
    let plate = broadsheet::image(
        commands,
        art,
        broadsheet::PLATE,
        layout,
        broadsheet::PLATE_BOX,
        LAYER + 1,
    );
    commands.entity(plate).insert(UiRoot);
    for part in broadsheet::header(
        commands,
        art,
        layout,
        document.author,
        document.card,
        "ISSUED BY",
        LAYER + 2,
    ) {
        commands.entity(part).insert(UiRoot);
    }

    let body = commands
        .spawn((
            UiRoot,
            Body,
            Node {
                overflow: Overflow::scroll_y(),
                ..layout.at(BODY)
            },
            ScrollPosition(Vec2::new(0., state.scroll * layout.scale)),
            GlobalZIndex(LAYER + 2),
            Pickable::IGNORE,
        ))
        .id();
    commands.spawn((
        Text::new(document.text.clone()),
        TextFont {
            font: art.font.clone().into(),
            font_size: (58. * layout.scale).into(),
            ..default()
        },
        TextLayout::linebreak(LineBreak::WordOrCharacter),
        bevy::text::LineHeight::Px(LINE * layout.scale),
        TextColor(broadsheet::INK),
        Node {
            width: percent(100),
            flex_shrink: 0.,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(body),
    ));

    let card = if document.card == session.game().current_card() {
        "this card".to_owned()
    } else {
        format!("on card {}", document.card)
    };
    let allowance = format!(
        "{} has published its proclamation {card}",
        document.author.name()
    );
    let caption = broadsheet::text(
        commands,
        layout,
        &art.bold,
        allowance,
        broadsheet::INK,
        [370., 1175., 1768., 32.],
        39.,
        LAYER + 2,
    );
    commands.entity(caption).insert(UiRoot);
    if let Some(Transfer::Authorizing { giver, .. }) = &session.game().transfer {
        let words = format!("{} is deciding the Resource request", giver.name());
        let pending = broadsheet::text(
            commands,
            layout,
            &art.font,
            words,
            broadsheet::INK,
            [370., 1282., 1260., 31.],
            38.,
            LAYER + 2,
        );
        commands.entity(pending).insert(UiRoot);
    }

    if open.archive && open.index > 0 {
        button(
            commands,
            art,
            layout,
            [1268., 1157., 420., 79.],
            ("proclamation:previous", Press::Previous),
            "Previous proclamation",
            false,
            LAYER + 2,
        );
    }
    if (open.archive && open.index + 1 < documents.len())
        || (!open.archive && !state.queued.is_empty())
    {
        button(
            commands,
            art,
            layout,
            [1718., 1157., 420., 79.],
            ("proclamation:next", Press::Next),
            "Next proclamation",
            false,
            LAYER + 2,
        );
    }
    button(
        commands,
        art,
        layout,
        RETURN_BOX,
        (RETURN, Press::Return),
        "Return to choice",
        true,
        LAYER + 2,
    );
}

#[allow(clippy::too_many_arguments)] // Authored button rectangle and rendering layer.
fn button(
    commands: &mut Commands,
    art: &Art,
    layout: broadsheet::Layout,
    area: [f32; 4],
    (name, press): (&str, Press),
    label: &str,
    primary: bool,
    z: i32,
) {
    let image = if primary {
        broadsheet::PRIMARY
    } else {
        broadsheet::SECONDARY
    };
    let ink = if primary {
        broadsheet::LIGHT
    } else {
        broadsheet::INK
    };
    let background = broadsheet::image(commands, art, image, layout, area, z);
    commands
        .entity(background)
        .insert((
            UiRoot,
            Pickable::default(),
            Control {
                name: name.into(),
                choice: false,
                action: Action::ReadProclamation(press),
            },
        ))
        .observe(click);
    let [x, y, w, h] = area;
    let words = broadsheet::text(
        commands,
        layout,
        &art.bold,
        label,
        ink,
        [x, y + (h - 42.) / 2., w, 33.],
        42.,
        z + 1,
    );
    commands
        .entity(words)
        .insert((UiRoot, TextLayout::justify(bevy::text::Justify::Center)));
}

pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let Some(session) = &world.resource::<Play>().session else {
        return serde_json::Value::Null;
    };
    let state = &world.resource::<Interface>().proclamation_reader;
    let documents = session.proclamations();
    let listed = documents
        .iter()
        .enumerate()
        .map(|(index, document)| {
            json!({
                "number": index + 1, "card": document.card, "author": document.author,
                "text": document.text, "read": state.read.contains(&index),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "documents": listed, "unread": state.unread(documents.len()),
        "open": state.open.map(|open| open.index + 1),
        "scroll": state.scroll,
    })
}
