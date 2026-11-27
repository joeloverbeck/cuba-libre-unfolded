//! Counts follow their pieces and scale to the projected diameter of the authored plate.
use super::*;
use bevy::{camera::CameraUpdateSystems, ui::UiSystems};

const ROUNDEL: &str = "inventory-roundel.png";
const DIAMETER: f32 = 0.198;
const GAP: f32 = 0.029;
const FONT_SIZE: f32 = 64.;

pub(super) fn spawn(
    commands: &mut Commands,
    art: &Art,
    space: &str,
    text: String,
    token: Entity,
    mut offset: Vec3,
) {
    offset.x += GAP + DIAMETER / 2.;
    commands
        .spawn((
            Count,
            Roundel { token, offset },
            InScene,
            status::SpaceTarget(space.to_owned()),
            LabelAnchor::entity(token, offset).within_viewport(),
            ImageNode::new(art.images[ROUNDEL].clone()),
            Node {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTransform::from_xy(px(0.), percent(-50.)),
            GlobalZIndex(-1),
        ))
        .with_child((
            Count,
            Text::new(text),
            TextLayout::no_wrap(),
            TextFont {
                font: art.bold.clone().into(),
                font_size: FontSize::Px(FONT_SIZE),
                ..default()
            },
            TextColor(Color::srgb_u8(248, 244, 232)), // own colour: from the Inventory roundel rim
            Node {
                flex_shrink: 0.,
                ..default()
            },
            Pickable::IGNORE,
        ));
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
