//! Counts follow their pieces and scale to the projected diameter of the authored plate.
use super::*;
use bevy::{camera::CameraUpdateSystems, ecs::system::SystemParam, ui::UiSystems};

const ROUNDEL: &str = "inventory-roundel.png";
const DIAMETER: f32 = 0.198;
const GAP: f32 = 0.016;
const FONT_SIZE: f32 = 64.;

/// Give the badge room beside the body or its wider Cash carrier.
pub(super) fn side_reach(kind: Kind, carries_cash: bool) -> f32 {
    match kind {
        Kind::Troops | Kind::Police if !carries_cash => 0.09,
        Kind::Guerrilla if !carries_cash => 0.10,
        Kind::Troops | Kind::Police | Kind::Guerrilla => 0.13,
        Kind::Base | Kind::Casino => 0.15,
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    art: &Art,
    space: &str,
    text: String,
    token: Entity,
    offset: Vec3,
) -> Entity {
    spawn_plate(commands, art, Some(space), text, token, offset)
}

pub(super) fn moving(
    commands: &mut Commands,
    art: &Art,
    count: u8,
    token: Entity,
    offset: Vec3,
) -> Entity {
    spawn_plate(commands, art, None, count.to_string(), token, offset)
}

fn spawn_plate(
    commands: &mut Commands,
    art: &Art,
    space: Option<&str>,
    text: String,
    token: Entity,
    offset: Vec3,
) -> Entity {
    let offset = offset + Vec3::X * (GAP + DIAMETER / 2.);
    let mut image = ImageNode::new(art.images[ROUNDEL].clone());
    let opacity = if space.is_some() { 1. } else { 0. };
    image.color = image.color.with_alpha(opacity);
    let mut roundel = commands.spawn((
        Roundel { token, offset },
        InScene,
        LabelAnchor::entity(token, offset).within_viewport(),
        image,
        Node {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        UiTransform::from_xy(px(0.), percent(-50.)),
        GlobalZIndex(-1),
    ));
    if let Some(space) = space {
        roundel.insert((Count, status::SpaceTarget(space.to_owned())));
    } else {
        roundel.insert(Pickable::IGNORE);
    }
    roundel
        .with_children(|children| {
            let mut digit = children.spawn((
                Text::new(text),
                TextLayout::no_wrap(),
                TextFont {
                    font: art.bold.clone().into(),
                    font_size: FontSize::Px(FONT_SIZE),
                    ..default()
                },
                TextColor(Color::srgb_u8(248, 244, 232).with_alpha(opacity)), // own colour: Inventory roundel rim
                Node {
                    flex_shrink: 0.,
                    ..default()
                },
                Pickable::IGNORE,
            ));
            if space.is_some() {
                digit.insert(Count);
            }
        })
        .id()
}

/// The plate and its digit fade together during a trip.
#[derive(SystemParam)]
pub(super) struct Appearance<'w, 's> {
    plates: Query<'w, 's, (&'static mut ImageNode, &'static Children)>,
    digits: Query<'w, 's, &'static mut TextColor>,
}

impl Appearance<'_, '_> {
    pub(super) fn opacity(&mut self, roundel: Entity, opacity: f32) {
        let Ok((mut plate, children)) = self.plates.get_mut(roundel) else {
            return;
        };
        plate.color = plate.color.with_alpha(opacity);
        for child in children.iter() {
            if let Ok(mut digit) = self.digits.get_mut(child) {
                digit.0 = digit.0.with_alpha(opacity);
            }
        }
    }
}

#[derive(Component)]
struct Roundel {
    token: Entity,
    offset: Vec3,
}

pub(super) fn install(app: &mut App) {
    app.add_systems(
        PostUpdate,
        size_roundels
            .after(CameraUpdateSystems)
            .after(TransformSystems::Propagate)
            .before(UiSystems::Layout),
    );
}

fn size_roundels(
    camera: Single<(&Camera, &GlobalTransform), With<TableCamera>>,
    tokens: Query<&Transform>,
    mut roundels: Query<(&Roundel, &Children, &mut Node)>,
    mut digits: Query<&mut UiTransform, With<Text>>,
) {
    let (camera, camera_at) = *camera;
    for (badge, children, mut node) in &mut roundels {
        let Ok(token) = tokens.get(badge.token) else {
            continue;
        };
        let point = token.translation + token.scale * badge.offset;
        let delta = |axis: Vec3| -> Option<Vec2> {
            let radius = axis * (DIAMETER * map::PIECE_SCALE) / 2.;
            Some(
                camera.world_to_viewport(camera_at, point + radius).ok()?
                    - camera.world_to_viewport(camera_at, point - radius).ok()?,
            )
        };
        let (Some(x), Some(y)) = (delta(Vec3::X), delta(Vec3::Z)) else {
            continue;
        };
        // Estimate the minor projected diameter from the two screen-space chords.
        let x2 = x.length_squared();
        let y2 = y.length_squared();
        let difference = (x2 - y2).hypot(2. * x.dot(y));
        let side = ((x2 + y2 - difference) / 2.).max(0.).sqrt();
        let size = px(side);
        if node.width != size {
            node.width = size;
            node.height = size;
        }
        // Bevy retains a glyph atlas for each raster size. Zoom scales the same glyphs.
        let scale = Vec2::splat(side * 0.56 / FONT_SIZE);
        for child in children.iter() {
            if let Ok(mut digit) = digits.get_mut(child)
                && digit.scale != scale
            {
                digit.scale = scale;
            }
        }
    }
}
