//! The card tag that reads the current or the preview card (#139). The cards lie on
//! top of the two piles (`piles`, #143). Hovering or focusing a card opens its tag with
//! the shared dwell; a click pins it and Escape closes it. The tag's geometry is the FigCraft frame
//! `art/atlas/atlas-card-tag.figcraft`, authored from the owner's selected sheet T1 at
//! the owner's display (`atlas-card-tag.layout.json`). Every fact it shows comes from
//! the observation's `cards`; the tag only places them. Reading a card changes
//! nothing in the match.
use super::palette::{INK, PAPER, QUIET};
use super::*;
use crate::{CardFacts, Observation};
use bevy::text::{Justify, LetterSpacing, LineHeight};

/// One of the two shown cards, named for the development seat as `card:current` and
/// `card:preview`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Slot {
    Current,
    Preview,
}
impl Slot {
    pub(super) const ALL: [Self; 2] = [Self::Current, Self::Preview];
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Current => "card:current",
            Self::Preview => "card:preview",
        }
    }
    pub(super) fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|slot| slot.name() == name)
    }
    fn facts(self, view: &Observation) -> &CardFacts {
        match self {
            Self::Current => &view.cards.current,
            Self::Preview => &view.cards.preview,
        }
    }
    /// The mark beside the title, and its plate.
    fn mark(self) -> (&'static str, &'static str) {
        match self {
            Self::Current => ("CURRENT CARD", "atlas-card-tag-current.png"),
            Self::Preview => ("PREVIEW", "atlas-card-tag-preview.png"),
        }
    }
}
impl serde::Serialize for Slot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// The card tag's presentation state, kept in `Interface`. None of it is saved.
#[derive(Default)]
pub(super) struct State {
    /// The card whose tag a click pinned.
    pub pinned: Option<Slot>,
}

/// Node coordinates in the FigCraft frames, relative to the tag's plate.
mod fig {
    /// Frame units per reference pixel: the frames are drawn at the owner's display,
    /// 3840×2160, as sheet T1 is.
    pub const SCALE: f32 = 1.6875;
    pub const PLATE: [f32; 2] = [1250., 1090.];
    pub const TITLE: [f32; 3] = [60., 34., 860.];
    pub const TITLE_SIZE: f32 = 54.;
    /// The mark's plate ends at this x; its top and height. Widths by slot below.
    pub const MARK_RIGHT: f32 = 1210.;
    pub const MARK_TOP: f32 = 46.;
    pub const MARK_H: f32 = 46.;
    pub const MARK_W_CURRENT: f32 = 270.;
    pub const MARK_W_PREVIEW: f32 = 190.;
    pub const MARK_TEXT_TOP: f32 = 55.;
    pub const CAPS_SIZE: f32 = 22.;
    pub const CAPS_LINE: f32 = 26.;
    pub const CAPS_SPACING: f32 = 2.;
    pub const RULE: [f32; 4] = [60., 120., 1150., 2.];
    pub const FACE: [f32; 4] = [60., 150., 620., 889.];
    /// The face's corner radius, 5.6% of its width. Every admitted face has black
    /// printed outside its rounded corners; card 18's reach a 30.6 px radius on its
    /// 573 px face (5.3%), beyond the shared card shape's 4.5% (#139).
    pub const FACE_RADIUS: f32 = 35.;
    /// The ledger column: left, top, width.
    pub const COLUMN: [f32; 3] = [720., 150., 490.];
    pub const EMBLEM: f32 = 52.;
    pub const EMBLEM_STEP: f32 = 64.;
    /// From a caption's bottom to the emblems' top, and from them to the next caption.
    pub const EMBLEMS_TOP: f32 = 12.;
    pub const EMBLEMS_BOTTOM: f32 = 40.;
    /// From a caption's bottom to its value, and from a value to the next caption.
    pub const VALUE_TOP: f32 = 8.;
    pub const SECTION_GAP: f32 = 50.;
    pub const VALUE_SIZE: f32 = 32.;
    pub const VALUE_LINE: f32 = 38.;
    pub const HINT: [f32; 3] = [720., 1000., 490.];
    pub const HINT_SIZE: f32 = 26.;
}

/// Where the tag opens, as the main view's layout records it (`card-tag` in
/// `atlas-main-view.layout.json`): above the piles, its left edge on the played pile's
/// and its foot 16 px above the piles' labels on the owner's display, so both piles
/// stay visible beneath it. These are reference pixels, measured from the window's
/// left edge and from the top of a reference that lies against the window's bottom.
const TAG_LEFT: f32 = 110. / fig::SCALE;
const TAG_BOTTOM: f32 = HEIGHT - (2160. - 1546.) / fig::SCALE;

fn r(v: f32) -> f32 {
    v / fig::SCALE
}

/// A click on a card pins its tag, or unpins it.
pub(super) fn activate(ui: &mut Interface, slot: Slot) {
    ui.card.pinned = if ui.card.pinned == Some(slot) {
        None
    } else {
        Some(slot)
    };
    ui.hover.rearm();
}

/// Escape closes an open card tag: it unpins it and keeps a hovered or focused card's
/// tag closed until the pointer or focus moves on. Returns whether a tag was open.
pub(super) fn escape(ui: &mut Interface) -> bool {
    let Some(slot) = shown(ui) else {
        return false;
    };
    ui.card.pinned = None;
    if hovered(ui) == Some(slot) {
        ui.hover.dismiss();
    }
    true
}

fn hovered(ui: &Interface) -> Option<Slot> {
    match ui.hover.visible() {
        Some(Target::Card(slot)) => Some(*slot),
        _ => None,
    }
}

/// The card whose tag is open: the hovered or focused one, else the pinned one.
pub(super) fn shown(ui: &Interface) -> Option<Slot> {
    hovered(ui).or(ui.card.pinned)
}

/// The card that has keyboard focus, if one has.
pub(super) fn focused(ui: &Interface) -> Option<Slot> {
    ui.choice.focus.as_deref().and_then(Slot::named)
}

#[derive(Component)]
pub(super) struct Tag;

/// Names a drawn part of the tag by its node name in `atlas-card-tag.figcraft`.
#[derive(Component)]
pub(super) struct Part(pub &'static str);

/// What the open tag shows, for the development seat.
#[derive(Component, Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Shown {
    pub card: Slot,
    pub pinned: bool,
    pub number: u8,
    pub title: String,
    pub mark: &'static str,
    pub order: Vec<&'static str>,
    pub event: String,
    pub status: String,
    pub lasting: String,
    pub rules: String,
    /// The plate in logical window pixels: left, top, right, bottom.
    pub bounds: [f32; 4],
}

fn content(view: &Observation, slot: Slot, pinned: bool) -> Shown {
    let facts = slot.facts(view);
    Shown {
        card: slot,
        pinned,
        number: facts.number,
        title: format!("{} · {}", facts.number, facts.title),
        mark: slot.mark().0,
        order: facts.order.iter().map(|f| f.name()).collect(),
        event: facts.words.event.into(),
        status: facts.words.status.clone(),
        lasting: facts.words.lasting.clone(),
        rules: facts.words.rules.clone(),
        bounds: [0.; 4],
    }
}

/// Spawns one tag text: its face, colour, size and line height in frame units.
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    font: &Handle<Font>,
    content: &str,
    size: f32,
    line: f32,
    ink: Color,
    s: f32,
    node: Node,
    part: &'static str,
) -> Entity {
    commands
        .spawn((
            Tag,
            Part(part),
            Text::new(content),
            TextFont {
                font: font.clone().into(),
                font_size: (r(size) * s).into(),
                ..default()
            },
            LineHeight::Px(r(line) * s),
            TextColor(ink),
            node,
            Pickable::IGNORE,
        ))
        .id()
}

fn caps(
    commands: &mut Commands,
    art: &Art,
    content: &str,
    s: f32,
    node: Node,
    part: &'static str,
) -> Entity {
    let entity = words(
        commands,
        &art.bold,
        content,
        fig::CAPS_SIZE,
        fig::CAPS_LINE,
        QUIET,
        s,
        node,
        part,
    );
    commands
        .entity(entity)
        .insert(LetterSpacing::Px(r(fig::CAPS_SPACING) * s));
    entity
}

fn image(
    commands: &mut Commands,
    art: &Art,
    key: &str,
    node: Node,
    z: i32,
    part: &'static str,
) -> Entity {
    commands
        .spawn((
            Tag,
            Part(part),
            ImageNode::new(art.images[key].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            node,
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id()
}

/// Draws the open card tag, if any, above the piles.
#[allow(clippy::too_many_arguments)] // A Bevy system: its parameters are the resources it reads.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, (With<Tag>, Without<ChildOf>)>,
    mut previous: Local<Option<Shown>>,
    advance: Res<super::piles::Advance>,
    time: Res<Time>,
) {
    let s = reference_pixel(&window);
    // The tag opens above the piles, which lie against the window's bottom.
    let left = bottom(&window);
    let view = play.game().observe();
    // The tag states the model's cards, which are a card ahead of the table's while a
    // played card is in the air: as a pile's tag, it shows again as the cards land, on
    // the card that then lies there, and a pin holds through the motion.
    let open = shown(&ui).filter(|_| !advance.in_the_air(&view, time.elapsed_secs_f64()));
    let shown = play.session.as_ref().and(open).map(|slot| {
        let mut shown = content(&view, slot, ui.card.pinned == Some(slot));
        let [w, h] = fig::PLATE.map(r);
        let x = left.x + TAG_LEFT * s;
        let y = left.y + (TAG_BOTTOM - h) * s;
        shown.bounds = [x, y, x + w * s, y + h * s];
        shown
    });
    if *previous == shown {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    previous.clone_from(&shown);
    let Some(shown) = shown else {
        return;
    };
    let [x, y, ..] = shown.bounds;
    let slot = shown.card;
    let at = |fx: f32, fy: f32| (px(x + r(fx) * s), px(y + r(fy) * s));
    let absolute = |fx: f32, fy: f32, fw: f32, fh: Option<f32>| {
        let (left, top) = at(fx, fy);
        Node {
            position_type: PositionType::Absolute,
            left,
            top,
            width: px(r(fw) * s),
            height: fh.map_or(Val::Auto, |fh| px(r(fh) * s)),
            ..default()
        }
    };
    let plate = image(
        &mut commands,
        &art,
        "atlas-top-tag.png",
        absolute(0., 0., fig::PLATE[0], Some(fig::PLATE[1])),
        20,
        "plate",
    );
    commands.entity(plate).insert(shown.clone());
    let [rx, ry, rw, rh] = fig::RULE;
    image(
        &mut commands,
        &art,
        "atlas-victory-rule.png",
        absolute(rx, ry, rw, Some(rh)),
        21,
        "rule",
    );
    let mark_w = match slot {
        Slot::Current => fig::MARK_W_CURRENT,
        Slot::Preview => fig::MARK_W_PREVIEW,
    };
    let (mark, plate_key) = slot.mark();
    let mark_x = fig::MARK_RIGHT - mark_w;
    image(
        &mut commands,
        &art,
        plate_key,
        absolute(mark_x, fig::MARK_TOP, mark_w, Some(fig::MARK_H)),
        21,
        "mark",
    );
    let label = words(
        &mut commands,
        &art.bold,
        mark,
        fig::CAPS_SIZE,
        fig::CAPS_LINE,
        PAPER,
        s,
        absolute(mark_x, fig::MARK_TEXT_TOP, mark_w, None),
        "mark-label",
    );
    commands.entity(label).insert((
        LetterSpacing::Px(r(fig::CAPS_SPACING) * s),
        TextLayout::justify(Justify::Center),
        GlobalZIndex(22),
    ));
    let [tx, ty, tw] = fig::TITLE;
    let title = words(
        &mut commands,
        &art.font,
        &shown.title,
        fig::TITLE_SIZE,
        fig::TITLE_SIZE * 1.2,
        INK,
        s,
        absolute(tx, ty, tw, None),
        "title",
    );
    commands.entity(title).insert(GlobalZIndex(22));
    let number = if slot == Slot::Current {
        view.current_card
    } else {
        view.preview_card
    };
    let [fx, fy, fw, fh] = fig::FACE;
    let mut face_node = absolute(fx, fy, fw, Some(fh));
    face_node.border_radius = BorderRadius::all(px(r(fig::FACE_RADIUS) * s));
    image(
        &mut commands,
        &art,
        assets::card_face(number).name,
        face_node,
        21,
        "face",
    );
    // The ledger column flows: a value that wraps pushes the captions below it down.
    let [cx, cy, cw] = fig::COLUMN;
    let column = commands
        .spawn((
            Tag,
            Node {
                flex_direction: FlexDirection::Column,
                ..absolute(cx, cy, cw, None)
            },
            GlobalZIndex(22),
            Pickable::IGNORE,
        ))
        .id();
    let margin = |top: f32| Node {
        margin: UiRect::top(px(r(top) * s)),
        ..default()
    };
    let mut children = vec![caps(
        &mut commands,
        &art,
        "FACTION ORDER",
        s,
        margin(0.),
        "label-order",
    )];
    let row = commands
        .spawn((
            Tag,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(r(fig::EMBLEM_STEP - fig::EMBLEM) * s),
                ..margin(fig::EMBLEMS_TOP)
            },
            Pickable::IGNORE,
        ))
        .id();
    const EMBLEMS: [&str; 4] = ["emblem-0", "emblem-1", "emblem-2", "emblem-3"];
    for (part, faction) in EMBLEMS.into_iter().zip(slot.facts(&view).order) {
        let emblem = image(
            &mut commands,
            &art,
            &format!("atlas-victory-icon-{}.png", slug(faction)),
            Node {
                width: px(r(fig::EMBLEM) * s),
                height: px(r(fig::EMBLEM) * s),
                flex_shrink: 0.,
                ..default()
            },
            22,
            part,
        );
        commands.entity(row).add_child(emblem);
    }
    children.push(row);
    let mut gap = fig::EMBLEMS_BOTTOM;
    for (caption, label_part, value, part, ink) in [
        ("EVENT", "label-event", &shown.event, "event", INK),
        ("STATUS", "label-status", &shown.status, "status", INK),
        (
            "LASTING EFFECT",
            "label-lasting",
            &shown.lasting,
            "lasting",
            INK,
        ),
        ("RULES", "label-rules", &shown.rules, "rules", QUIET),
    ] {
        children.push(caps(
            &mut commands,
            &art,
            caption,
            s,
            margin(gap),
            label_part,
        ));
        children.push(words(
            &mut commands,
            &art.font,
            value,
            fig::VALUE_SIZE,
            fig::VALUE_LINE,
            ink,
            s,
            margin(fig::VALUE_TOP),
            part,
        ));
        gap = fig::SECTION_GAP;
    }
    commands.entity(column).add_children(&children);
    let [hx, hy, hw] = fig::HINT;
    let hint = words(
        &mut commands,
        &art.font,
        "Click pins · Esc closes",
        fig::HINT_SIZE,
        fig::HINT_SIZE * 1.2,
        QUIET,
        s,
        absolute(hx, hy, hw, None),
        "hint",
    );
    commands.entity(hint).insert(GlobalZIndex(22));
}

/// The open tag, each of its drawn parts and each card's bounds, in logical window
/// pixels, for the seat.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let shown = world.query::<&Shown>().iter(world).next().cloned();
    let mut parts = world
        .query::<(&Part, &ComputedNode, &UiGlobalTransform, Option<&Text>)>()
        .iter(world)
        .map(|(part, node, at, text)| {
            (
                part.0,
                serde_json::json!({"part":part.0,"text":text.map(|t| &t.0),"bounds":seat::logical_bounds(node, at)}),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by_key(|(name, _)| *name);
    let mut cards = world
        .query::<(&Control, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .filter_map(|(control, node, at)| match control.action {
            Action::Card(slot) => Some((
                slot.name(),
                serde_json::json!({"card":slot,"bounds":seat::logical_bounds(node, at)}),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    cards.sort_by_key(|(name, _)| *name);
    serde_json::json!({
        "shown": shown,
        "parts": parts.into_iter().map(|(_, part)| part).collect::<Vec<_>>(),
        "cards": cards.into_iter().map(|(_, card)| card).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_face_reads_at_about_620_pixels_on_the_owner_s_display() {
        // The owner's 3840×2160 display draws the 1920×1280 reference at ×1.6875.
        let scale = (3840. / WIDTH).min(2160. / HEIGHT);
        assert_eq!(scale, fig::SCALE);
        assert!((r(fig::FACE[2]) * scale - 620.).abs() < 0.01);
        // No smaller than the face's own resolution.
        assert!(r(fig::FACE[2]) * scale >= 573.);
    }

    #[test]
    fn the_tag_opens_where_the_main_view_s_layout_records_it() {
        let layout: serde_json::Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let rect = |value: &serde_json::Value| -> [f32; 4] {
            std::array::from_fn(|i| value[i].as_f64().unwrap() as f32)
        };
        let [x, y, w, h] = rect(&layout["card-tag"]);
        assert_eq!([w, h], fig::PLATE);
        // On the owner's display the reference is drawn at the frame's own scale.
        let on_display = |v: f32| v * fig::SCALE;
        assert!((on_display(TAG_LEFT) - x).abs() < 0.01);
        let low = 2160. - on_display(HEIGHT);
        assert!((on_display(TAG_BOTTOM) + low - (y + h)).abs() < 0.01);
        // Inside the reference window, so inside every window.
        let [w, h] = fig::PLATE.map(r);
        assert!(TAG_BOTTOM - h >= 0. && TAG_LEFT >= 0. && TAG_LEFT + w <= WIDTH);
        // Above both piles and their labels, which stay visible beneath it.
        for name in ["played-pile", "draw-deck"] {
            assert!(y + h <= rect(&layout["tray"][name])[1], "{name}");
        }
        for name in ["played-label", "deck-label"] {
            assert!(y + h <= rect(&layout["text"][name])[1], "{name}");
        }
    }

    #[test]
    fn the_corner_radius_covers_the_printed_corners() {
        // The darkest printed corner of an admitted face, card 18's, needs a radius of
        // 30.6 px on its 573 px face to clear every dark pixel.
        const { assert!(fig::FACE_RADIUS / fig::FACE[2] >= 30.6 / 573.) };
    }

    #[test]
    fn a_click_pins_and_escape_closes() {
        let mut ui = Interface::default();
        activate(&mut ui, Slot::Preview);
        assert_eq!(shown(&ui), Some(Slot::Preview));
        activate(&mut ui, Slot::Current);
        assert_eq!(shown(&ui), Some(Slot::Current));
        assert!(escape(&mut ui));
        assert_eq!(shown(&ui), None);
        assert!(!escape(&mut ui));
        activate(&mut ui, Slot::Current);
        activate(&mut ui, Slot::Current);
        assert_eq!(shown(&ui), None);
    }

    #[test]
    fn the_tag_states_the_model_s_facts() {
        let view = crate::Game::opening().observe();
        let shown = content(&view, Slot::Current, false);
        assert_eq!(shown.title, "17 · Alberto Bayo");
        assert_eq!(shown.mark, "CURRENT CARD");
        assert_eq!(
            shown.order,
            ["26July", "Directorio", "Government", "Syndicate"]
        );
        assert_eq!(shown.event, view.cards.current.words.event);
        assert_eq!(shown.status, view.cards.current.words.status);
        assert_eq!(shown.lasting, "None");
        assert_eq!(shown.rules, "R 2.3.4 · R 5.2");
        let preview = content(&view, Slot::Preview, true);
        assert_eq!(preview.title, "18 · Pact of Caracas");
        assert_eq!(preview.mark, "PREVIEW");
        assert!(preview.pinned);
        assert_eq!(preview.status, view.cards.preview.words.status);
    }

    #[test]
    fn every_mark_plate_is_admitted() {
        for slot in Slot::ALL {
            let key = slot.mark().1;
            assert!(
                crate::assets::CARD_TAG_MANIFEST
                    .lines()
                    .any(|line| line.ends_with(&format!("art/atlas/{key}"))),
                "{key} is not admitted"
            );
        }
    }
}
