//! A human writes on the selected Broadsheet over the current decision.
use super::broadsheet::{self as paper, GOLD, INK, LIGHT, Layout};
use super::*;
use crate::faction_seats::Stands;
use crate::save::Proclamation;
use bevy::{
    ecs::message::MessageMutator,
    input::keyboard::{Key, KeyboardInput},
    input_focus::{FocusCause, InputFocus},
    text::{EditableText, Justify, LineHeight, TextCursorStyle, TextLayoutInfo},
    ui::widget::NodeImageMode,
    ui_widgets::TextInput,
};
use serde_json::{Value, json};
use unfolded_tabletop::seat::Focus;

pub(super) const WRITE: &str = "proclamation:write";
const FIELD: &str = "proclamation:field";
const FIELD_PLATE: &str = "proclamation-broadsheet-field.png";
const RETURN: &str = "proclamation:return";
const PUBLISH: &str = "proclamation:publish";
const LAYER: i32 = 49;
const INK_RED: Color = Color::srgb(0.45, 0.16, 0.07); // own colour: dark red for refusals on Broadsheet paper
const FIELD_BOX: [f32; 4] = [332., 548., 1851., 538.];
const WORDS_BOX: [f32; 4] = [370., 568., 1775., 505.];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Press {
    Open,
    Author(Faction),
    Field,
    Return,
    Publish,
}

impl Press {
    fn name(self) -> String {
        match self {
            Self::Open => WRITE.into(),
            Self::Author(faction) => format!("proclamation:author:{}", slug(faction)),
            Self::Field => FIELD.into(),
            Self::Return => RETURN.into(),
            Self::Publish => PUBLISH.into(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Page {
    author: Faction,
    revision: u64,
    card: u8,
    focus: Press,
    refusal: Option<String>,
}

#[derive(Default)]
pub(super) struct State {
    page: Option<Page>,
    card: Option<u8>,
    drafts: [String; 4],
    publishing: bool,
}

impl State {
    pub(super) fn is_open(&self) -> bool {
        self.page.is_some()
    }

    fn draft(&self, author: Faction) -> &str {
        &self.drafts[author.index()]
    }
}

#[derive(Component)]
pub(super) struct Editor(Faction);

#[derive(Component)]
pub(super) struct Printed;

#[derive(Component)]
struct Part(&'static str);

#[derive(PartialEq)]
pub(super) struct Sheet {
    page: Page,
    layout: Layout,
    length: usize,
    human: [bool; 4],
    spent: bool,
}

fn available(play: &Play, ui: &Interface) -> bool {
    play.session.as_ref().is_some_and(|session| {
        Stands::of(session.game()) == Stands::Open
            && ui.played().into_iter().any(|played| played)
            && !ui.editions.holds()
    })
}

pub(super) fn press(play: &Play, ui: &mut Interface, press: Press) {
    if matches!(press, Press::Open | Press::Return) {
        ui.dirty = true;
    }
    let human = ui.played();
    let can_open = press == Press::Open && available(play, ui);
    let state = &mut ui.composer;
    if can_open && state.page.is_none() {
        let game = play.session.as_ref().expect("An admitted match").game();
        let card = game.current_card();
        if state.card != Some(card) {
            state.drafts = default();
            state.card = Some(card);
        }
        let author = Some(game.owner())
            .filter(|owner| human[owner.index()])
            .or_else(|| {
                Faction::ALL
                    .into_iter()
                    .find(|faction| human[faction.index()])
            });
        if let Some(author) = author {
            state.page = Some(Page {
                author,
                revision: game.revision,
                card,
                focus: Press::Field,
                refusal: None,
            });
        }
    } else if let Some(page) = &mut state.page {
        match press {
            Press::Open => {}
            Press::Author(author) => {
                page.author = author;
                page.refusal = None;
                page.focus = Press::Field;
            }
            Press::Field => page.focus = Press::Field,
            Press::Return => state.page = None,
            Press::Publish => state.publishing = true,
        }
    }
}

fn tabs() -> [Press; 7] {
    let [government, m26, directorio, syndicate] = Faction::ALL.map(Press::Author);
    [
        government,
        m26,
        directorio,
        syndicate,
        Press::Field,
        Press::Return,
        Press::Publish,
    ]
}

/// The shared seat puts a named printable key in the logical key, with no winit
/// text payload. Complete that event before Bevy's focused editor dispatch.
pub(super) fn seat_text(mut events: MessageMutator<KeyboardInput>, ui: Res<Interface>) {
    if !ui.composer.is_open() {
        events.clear();
        return;
    }
    for event in events.read() {
        if event.text.is_none() {
            event.text = match &event.logical_key {
                Key::Character(text) => Some(text.clone()),
                Key::Space => Some(" ".into()),
                _ => None,
            };
        }
    }
}

pub(super) fn keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    play: Res<Play>,
    mut ui: ResMut<Interface>,
    roll: Res<die::Roll>,
    advance: Res<piles::Advance>,
    time: Res<Time>,
) {
    if !ui.composer.is_open() {
        if keys.just_pressed(KeyCode::KeyP) && available(&play, &ui) {
            let game = play.game();
            if !advance.in_the_air(&game.observe(), time.elapsed_secs_f64())
                && die::presented(game.revision, game.kidnap.as_ref(), &roll).is_none()
            {
                press(&play, &mut ui, Press::Open);
                keys.clear_just_pressed(KeyCode::KeyP);
            }
        }
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        press(&play, &mut ui, Press::Return);
    } else if let Some(page) = &mut ui.composer.page {
        if keys.just_pressed(KeyCode::Tab) {
            let tabs = tabs();
            let index = tabs
                .iter()
                .position(|stop| *stop == page.focus)
                .unwrap_or(0);
            let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            page.focus = tabs[(index + if back { tabs.len() - 1 } else { 1 }) % tabs.len()];
        } else if page.focus != Press::Field
            && keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space])
        {
            let focused = page.focus;
            press(&play, &mut ui, focused);
        }
    }
    let consumed: Vec<_> = keys.get_just_pressed().copied().collect();
    for key in consumed {
        if !faction_seats::KEYS.contains(&key) {
            keys.clear_just_pressed(key);
        }
    }
}

pub(super) fn focus(ui: &Interface) -> Option<Focus> {
    ui.composer.page.as_ref().map(|page| Focus::Control {
        choice: None,
        label: page.focus.name(),
    })
}

/// Read the editor before a switch, close or publication, even if it no longer
/// represents the newly selected author. It stays alive across count redraws.
fn keep_draft(state: &mut State, field: &Query<(Entity, &Editor, &EditableText)>) {
    if let Ok((_, editor, text)) = field.single() {
        let old = &mut state.drafts[editor.0.index()];
        let edited = text.value().to_string();
        if *old != edited {
            *old = edited;
            if let Some(page) = &mut state.page
                && page.author == editor.0
            {
                page.refusal = None;
            }
        }
    }
}

/// Seat handovers are processed first, so the same frame's custody change
/// also governs a queued Publish press. Both controllers use Session::publish.
pub(super) fn publish(
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
    field: Query<(Entity, &Editor, &EditableText)>,
) {
    keep_draft(&mut ui.composer, &field);
    if !std::mem::take(&mut ui.composer.publishing) {
        return;
    }
    let Some(page) = ui.composer.page.as_ref() else {
        return;
    };
    let (author, revision, card) = (page.author, page.revision, page.card);
    let result = if ui.plays(author) {
        play.session
            .as_mut()
            .expect("An admitted match")
            .publish(author, revision, card, ui.composer.draft(author))
            .map_err(|failure| failure.to_string())
    } else {
        Err(format!("{} is controlled by another seat.", author.name()))
    };
    match result {
        Ok(()) => {
            ui.composer.drafts[author.index()].clear();
            ui.composer.page = None;
            ui.notice =
                notice::Notice::success(format!("{} published a proclamation.", author.name()));
            ui.dirty = true;
        }
        Err(refused) => {
            ui.composer
                .page
                .as_mut()
                .expect("The open composer")
                .refusal = Some(refused);
        }
    }
}

pub(super) fn draw(
    mut commands: Commands,
    art: Res<Art>,
    play: Res<Play>,
    ui: Res<Interface>,
    window: Single<&Window, With<PrimaryWindow>>,
    field: Query<(Entity, &Editor, &EditableText)>,
    printed: Query<Entity, With<Printed>>,
    mut focus: ResMut<InputFocus>,
    mut last: Local<Option<Sheet>>,
) {
    let layout = Layout::new(&window);
    let next = ui.composer.page.as_ref().and_then(|page| {
        let session = play.session.as_ref()?;
        Some(Sheet {
            page: page.clone(),
            layout,
            length: ui.composer.draft(page.author).chars().count(),
            human: ui.played(),
            spent: Proclamation::spent(session.proclamations(), page.card, page.author),
        })
    });
    if next == *last {
        return;
    }
    for entity in &printed {
        commands.entity(entity).despawn();
    }
    let rebuild_field = match (&next, &*last) {
        (Some(next), Some(last)) => {
            next.page.author != last.page.author || next.layout != last.layout
        }
        _ => true,
    };
    if rebuild_field && let Ok((entity, _, _)) = field.single() {
        if focus.get() == Some(entity) {
            focus.clear();
        }
        commands.entity(entity).despawn();
    }
    if let Some(sheet) = &next {
        let entity = if rebuild_field {
            editor(
                &mut commands,
                &art,
                sheet,
                ui.composer.draft(sheet.page.author),
            )
        } else {
            field.single().expect("The current writing field").0
        };
        match (
            sheet.page.focus == Press::Field,
            focus.get() == Some(entity),
        ) {
            (true, false) => focus.set(entity, FocusCause::Navigated),
            (false, true) => focus.clear(),
            _ => {}
        }
        print(&mut commands, &art, sheet);
    }
    *last = next;
}

fn editor(commands: &mut Commands, art: &Art, sheet: &Sheet, draft: &str) -> Entity {
    let layout = sheet.layout;
    let mut node = layout.at(WORDS_BOX);
    node.overflow = Overflow::clip();
    commands
        .spawn((
            Editor(sheet.page.author),
            Part("draft"),
            node,
            control(Press::Field),
            TextInput,
            EditableText {
                allow_newlines: true,
                visible_lines: None,
                cursor_width: 0.05,
                ..EditableText::new(draft)
            },
            TextFont {
                font: art.font.clone().into(),
                font_size: (58. * layout.scale).into(),
                ..default()
            },
            TextCursorStyle {
                color: INK,
                selection_color: GOLD,
                ..default()
            },
            TextLayout::default().with_linebreak(bevy::text::LineBreak::WordOrCharacter),
            LineHeight::Px(73. * layout.scale),
            TextColor(INK),
            GlobalZIndex(LAYER + 3),
        ))
        .observe(click)
        .id()
}

fn control(press: Press) -> Control {
    Control {
        name: press.name(),
        choice: false,
        action: Action::ComposeProclamation(press),
    }
}

fn print(commands: &mut Commands, art: &Art, sheet: &Sheet) {
    let layout = sheet.layout;
    let page = &sheet.page;
    commands.spawn((
        Printed,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ImageNode::new(art.images[paper::SCRIM].clone()).with_mode(NodeImageMode::Stretch),
        GlobalZIndex(LAYER),
    ));
    let plate = paper::image(
        commands,
        art,
        paper::PLATE,
        layout,
        paper::PLATE_BOX,
        LAYER + 1,
    );
    commands.entity(plate).insert((Printed, Part("plate")));
    let field = paper::image(commands, art, FIELD_PLATE, layout, FIELD_BOX, LAYER + 2);
    commands.entity(field).insert((Printed, Part("field")));
    for row in paper::header(
        commands,
        art,
        layout,
        page.author,
        page.card,
        "WRITING AS",
        LAYER + 2,
    ) {
        commands.entity(row).insert(Printed);
    }

    let mut sheet_print = SheetPrint {
        commands,
        art,
        layout,
    };
    for (faction, human) in Faction::ALL.into_iter().zip(sheet.human) {
        sheet_print.author(faction, human, page);
    }
    let allowance = if !sheet.human[page.author.index()] {
        "This faction is controlled by another seat."
    } else if sheet.spent {
        "This faction has already published on this card."
    } else {
        "1 proclamation available this card"
    };
    sheet_print.label(
        "allowance",
        allowance,
        [370., 1175., 1290., 32.],
        INK,
        Justify::Left,
    );
    let limit = Proclamation::LIMIT;
    let counter = format!("{} / {limit} characters", sheet.length);
    let ink = if sheet.length > limit { INK_RED } else { INK };
    sheet_print.label(
        "count",
        counter,
        [1662., 1175., 476., 32.],
        ink,
        Justify::Right,
    );
    if let Some(refusal) = &page.refusal {
        sheet_print.label(
            "refusal",
            refusal.as_str(),
            [370., 1091., 1768., 32.],
            INK_RED,
            Justify::Left,
        );
    }
    sheet_print.button(
        Press::Return,
        "Return to choice",
        [370., 1258., 480., 94.],
        false,
        page.focus == Press::Return,
    );
    sheet_print.button(
        Press::Publish,
        "Publish proclamation",
        [1658., 1258., 480., 94.],
        true,
        page.focus == Press::Publish,
    );
}

struct SheetPrint<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    art: &'a Art,
    layout: Layout,
}

impl SheetPrint<'_, '_, '_> {
    fn label(
        &mut self,
        part: &'static str,
        words: impl Into<String>,
        rect: [f32; 4],
        ink: Color,
        justify: Justify,
    ) {
        let label = paper::text(
            self.commands,
            self.layout,
            &self.art.bold,
            words,
            ink,
            rect,
            rect[3] * 1.25,
            LAYER + 3,
        );
        self.commands
            .entity(label)
            .insert((Printed, Part(part), TextLayout::justify(justify)));
    }

    fn author(&mut self, faction: Faction, human: bool, page: &Page) {
        let press = Press::Author(faction);
        let selected = page.author == faction;
        let rect = [124. + 320. * faction.index() as f32, 390., 300., 60.];
        self.control_plate(press, rect, !selected, page.focus == press);
        let [x, y, width, _] = rect;
        let words = if human {
            faction.name().into()
        } else {
            format!("{} · held", faction.name())
        };
        let ink = match (selected, human) {
            (true, _) => INK,
            (false, true) => LIGHT,
            (false, false) => GOLD,
        };
        self.label(
            "author",
            words,
            [x, y + 11., width, 28.],
            ink,
            Justify::Center,
        );
    }

    fn control_plate(&mut self, press: Press, rect: [f32; 4], primary: bool, focused: bool) {
        let file = if primary {
            paper::PRIMARY
        } else {
            paper::SECONDARY
        };
        let fill = paper::image(self.commands, self.art, file, self.layout, rect, LAYER + 2);
        self.commands
            .entity(fill)
            .remove::<Pickable>()
            .insert((Printed, control(press)))
            .observe(click);
        if focused {
            let mark = if primary {
                "atlas-focus-mark-field.png"
            } else {
                "atlas-focus-mark.png"
            };
            popover::plate(
                self.commands,
                self.art,
                fill,
                (mark, Some(9.), [4.; 4]),
                self.layout.scale,
            );
        }
    }

    fn button(&mut self, press: Press, words: &str, rect: [f32; 4], primary: bool, focused: bool) {
        self.control_plate(press, rect, primary, focused);
        let [x, y, width, _] = rect;
        self.label(
            match press {
                Press::Return => "return",
                Press::Publish => "publish",
                _ => "write",
            },
            words,
            [x + 4., y + 24., width - 8., 35.],
            if primary { LIGHT } else { INK },
            Justify::Center,
        );
    }
}

pub(super) fn entry(
    commands: &mut Commands,
    art: &Art,
    play: &Play,
    ui: &Interface,
    window: &Window,
) {
    if !available(play, ui) || ui.composer.is_open() {
        return;
    }
    let scale = (window.width() / 3840.).min(window.height() / 2160.);
    let rect = [100., 208., 410., 69.];
    let z = transfer::LAYER + 2;
    let button = picture(commands, art, paper::SECONDARY, rect, scale, Vec2::ZERO, z);
    commands
        .entity(button)
        .remove::<Pickable>()
        .insert(control(Press::Open))
        .observe(click);
    let label = styled(
        commands,
        art,
        "Write a proclamation · P",
        &art.bold,
        INK,
        [100., 224., 410., 29.],
        scale,
        Vec2::ZERO,
        z + 1,
    );
    commands
        .entity(label)
        .insert(TextLayout::justify(Justify::Center));
}

pub(super) fn observe(world: &mut World) -> Value {
    let Some((page, draft, spent)) = ({
        let ui = world.resource::<Interface>();
        let session = world.resource::<Play>().session.as_ref();
        ui.composer
            .page
            .as_ref()
            .zip(session)
            .map(|(page, session)| {
                (
                    page.clone(),
                    ui.composer.draft(page.author).to_owned(),
                    Proclamation::spent(session.proclamations(), page.card, page.author),
                )
            })
    }) else {
        return Value::Null;
    };
    let mut parts = world
        .query::<(
            &Part,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&TextLayoutInfo>,
        )>()
        .iter(world)
        .map(|(part, node, at, text)| {
            json!({
                "part": part.0,
                "bounds": seat::logical_bounds(node, at),
                "text_size": text.map(|info| (info.size * node.inverse_scale_factor).to_array()),
            })
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a["part"].as_str().cmp(&b["part"].as_str()));
    let editor = world
        .query::<&EditableText>()
        .iter(world)
        .next()
        .map_or(draft, |field| field.value().to_string());
    json!({
        "author": page.author, "revision": page.revision, "card": page.card,
        "limit": Proclamation::LIMIT, "spent": spent,
        "count": editor.chars().count(), "draft": editor,
        "focus": page.focus.name(), "refusal": page.refusal, "parts": parts,
    })
}
