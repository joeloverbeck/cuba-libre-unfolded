//! Motion between a drawn supply counter and a point on the table.
use bevy::prelude::*;
use unfolded_tabletop::card_motion::{CardMove, Lift};

pub(super) const DURATION: f32 = 0.68;

/// Find the world point behind a logical UI position at the same camera
/// depth as the matching map slot.
pub(super) fn screen_point_at(
    camera: &Camera,
    camera_at: &GlobalTransform,
    logical: Vec2,
    map: Vec3,
) -> Option<Vec3> {
    camera
        .viewport_to_world(camera_at, logical)
        .ok()?
        .plane_intersection_point(map, InfinitePlane3d::new(*camera_at.forward()))
}

/// Sample the common travel and fade timing. The caller owns the material
/// and the rules that selected these endpoints.
pub(super) fn sample(
    counter: Vec3,
    rest: Transform,
    entering: bool,
    elapsed: f32,
) -> (Transform, f32) {
    let (from, to) = if entering {
        (counter, rest)
    } else {
        (
            rest.translation,
            Transform {
                translation: counter,
                ..rest
            },
        )
    };
    let motion = CardMove::new(from, to, DURATION).lifted(Lift::arc(0.25));
    let fraction = (elapsed / DURATION).clamp(0., 1.);
    // Let the small solids read while crossing the table. Most of the fade
    // occurs near the counter, where the chrome begins to cover the solid.
    let fade = if entering { 1. - fraction } else { fraction };
    let opacity = 1. - fade * fade;
    (motion.pose(elapsed), opacity)
}
