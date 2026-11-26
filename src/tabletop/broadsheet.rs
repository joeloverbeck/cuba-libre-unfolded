//! The selected 3840×2160 Broadsheet, shared by writing and reading. Coordinates
//! are relative to the plate in the four frames of the #228 FigCraft source.
use super::*;
use bevy::text::{Justify, LetterSpacing, LineHeight};
use bevy::ui::widget::NodeImageMode;

pub(super) const PLATE: &str = "proclamation-broadsheet-plate.png";
pub(super) const PRIMARY: &str = "proclamation-primary.png";
pub(super) const SECONDARY: &str = "proclamation-secondary.png";
pub(super) const SCRIM: &str = "atlas-newspaper-scrim.png";
pub(super) const PLATE_BOX: [f32; 4] = [0., 0., 2320., 1400.];
pub(super) const INK: Color = Color::srgb(23. / 255., 63. / 255., 50. / 255.); // own colour: Proclamations variables in havana-proclamations.figcraft.
pub(super) const GOLD: Color = Color::srgb(244. / 255., 215. / 255., 139. / 255.); // own colour: Proclamations variables in havana-proclamations.figcraft.
pub(super) const LIGHT: Color = Color::srgb(246. / 255., 240. / 255., 221. / 255.); // own colour: Proclamations variables in havana-proclamations.figcraft.

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Layout {
    pub scale: f32,
    origin: Vec2,
}

impl Layout {
    pub(super) fn new(window: &Window) -> Self {
        let scale = (window.width() / 3840.).min(window.height() / 2160.);
        Self {
            scale,
            origin: Vec2::new(window.width() / 2. - 1160. * scale, 378. * scale),
        }
    }

    pub(super) fn at(self, [x, y, w, h]: [f32; 4]) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.origin.x + x * self.scale),
            top: px(self.origin.y + y * self.scale),
            width: px(w * self.scale),
            height: px(h * self.scale),
            ..default()
        }
    }
}

pub(super) fn image(
    commands: &mut Commands,
    art: &Art,
    file: &str,
    layout: Layout,
    rect: [f32; 4],
    z: i32,
) -> Entity {
    commands
        .spawn((
            layout.at(rect),
            ImageNode::new(art.images[file].clone()).with_mode(NodeImageMode::Stretch),
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id()
}

#[allow(clippy::too_many_arguments)] // The chosen face, ink, box and leading are independent.
pub(super) fn text(
    commands: &mut Commands,
    layout: Layout,
    face: &Handle<Font>,
    words: impl Into<String>,
    ink: Color,
    [x, y, w, size]: [f32; 4],
    line_height: f32,
    z: i32,
) -> Entity {
    let mut node = layout.at([x, y, w, line_height]);
    node.height = Val::Auto;
    commands
        .spawn((
            node,
            Text::new(words),
            TextFont {
                font: face.clone().into(),
                font_size: (size * layout.scale).into(),
                ..default()
            },
            LineHeight::Px(line_height * layout.scale),
            TextColor(ink),
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id()
}

pub(super) fn header(
    commands: &mut Commands,
    art: &Art,
    layout: Layout,
    faction: Faction,
    card: u8,
    prefix: &str,
    z: i32,
) -> [Entity; 5] {
    let rows = [
        (
            "PUBLIC BROADSHEET".into(),
            &art.bold,
            GOLD,
            [120., 64., 1280., 45.],
            55.,
            7.,
        ),
        (
            prefix.into(),
            &art.bold,
            GOLD,
            [124., 164., 1000., 34.],
            41.,
            4.,
        ),
        (
            faction.name().to_uppercase(),
            &art.black,
            GOLD,
            [108., 200., 1470., 192.],
            201.,
            -4.,
        ),
        (
            format!("Card {card}  /  {}", crate::title(card)),
            &art.bold,
            INK,
            [370., 496., 850., 33.],
            40.,
            0.,
        ),
        (
            "TO ALL FOUR FACTIONS".into(),
            &art.bold,
            INK,
            [1300., 496., 839., 31.],
            38.,
            3.,
        ),
    ];
    rows.map(|(words, face, ink, rect, line_height, spacing)| {
        let entity = text(commands, layout, face, words, ink, rect, line_height, z);
        commands
            .entity(entity)
            .insert(LetterSpacing::Px(spacing * layout.scale));
        if rect[0] == 1300. {
            commands
                .entity(entity)
                .insert(TextLayout::justify(Justify::Right));
        }
        entity
    })
}
