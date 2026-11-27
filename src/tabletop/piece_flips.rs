//! Render the state changes of a saved decision while the session already holds the
//! resulting position. The stationary groups represent what stayed in each slot.
use super::*;
use crate::{Flip, Group};
use unfolded_tabletop::card_motion::{CardMove, Lift, Turn};

const RISE: f32 = 0.18;
const TRAVEL: f32 = 0.35;
const TURN: f32 = 0.52;

pub(super) struct Change {
    pub(super) flip: Flip,
    source_remaining: u8,
    destination_before: u8,
}

impl Change {
    fn between(flip: &Flip, previous: &crate::Space, after: &crate::Space) -> Option<Self> {
        let count = |space: &crate::Space, state| {
            space
                .pieces
                .iter()
                .filter(|p| p.faction == flip.faction && p.kind == flip.kind && p.state == state)
                .map(|p| p.count)
                .sum::<u8>()
        };
        let from_before = count(previous, flip.from);
        let source_remaining = count(after, flip.from);
        let destination_before = count(previous, flip.to);
        let to_after = count(after, flip.to);
        // Some choice previews describe a later resolution. Both changes must
        // actually have happened in the successful submission.
        if flip.count == 0
            || from_before.checked_sub(source_remaining) != Some(flip.count)
            || to_after.checked_sub(destination_before) != Some(flip.count)
        {
            return None;
        }
        Some(Self {
            flip: flip.clone(),
            source_remaining,
            destination_before,
        })
    }
}

pub(super) fn committed(before: &Observation, after: &Observation, choice: &str) -> Vec<Change> {
    let preview = before
        .choices
        .iter()
        .find(|c| c.id == choice)
        .and_then(|choice| choice.preview.as_ref());
    let Some(preview) = preview else {
        return Vec::new();
    };
    preview
        .flips
        .iter()
        .filter(|flip| {
            matches!(
                (flip.kind, flip.from, flip.to),
                (Kind::Guerrilla, State::Underground, State::Active)
                    | (Kind::Guerrilla, State::Active, State::Underground)
                    | (Kind::Casino, State::Closed, State::Open)
                    | (Kind::Casino, State::Open, State::Closed)
            )
        })
        .filter_map(|flip| {
            let previous = before.spaces.iter().find(|s| s.id == flip.space)?;
            let after = after.spaces.iter().find(|s| s.id == flip.space)?;
            Change::between(flip, previous, after)
        })
        .collect()
}

#[derive(Resource, Default)]
pub(super) struct Flights {
    started: f64,
    changes: Vec<Change>,
}

impl Flights {
    pub(super) fn refresh(&mut self, pending: &mut Vec<Change>, now: f64) -> bool {
        if !pending.is_empty() {
            self.changes = std::mem::take(pending);
            self.started = now;
            return true;
        }
        if self.moving() && now - self.started >= f64::from(RISE + TRAVEL + TURN) {
            self.changes.clear();
            return true;
        }
        false
    }

    pub(super) fn moving(&self) -> bool {
        !self.changes.is_empty()
    }

    pub(super) fn in_space(&self, space: &str) -> impl Iterator<Item = (Group, &Change)> {
        self.changes
            .iter()
            .filter(move |change| change.flip.space == space)
            .map(|change| {
                let flip = &change.flip;
                (
                    Group {
                        faction: flip.faction,
                        kind: flip.kind,
                        state: flip.to,
                        count: flip.count,
                    },
                    change,
                )
            })
    }

    pub(super) fn count(&self, space: &str, group: &Group) -> u8 {
        self.changes
            .iter()
            .find(|change| {
                let flip = &change.flip;
                flip.space == space
                    && flip.faction == group.faction
                    && flip.kind == group.kind
                    && flip.to == group.state
            })
            .map_or(group.count, |change| {
                group.count.saturating_sub(change.flip.count)
            })
    }
}

#[derive(Component)]
pub(super) struct Motion {
    rise: CardMove,
    travel: CardMove,
    turn: CardMove,
    color: Color,
    fades_in: bool,
    flip: Flip,
}

impl Motion {
    pub(super) fn new(
        from: Vec3,
        mut destination: Transform,
        half_height: f32,
        color: Color,
        change: &Change,
    ) -> Self {
        if change.destination_before > 0 {
            // The flying solid can rest on the resident's top before the two are
            // represented again by one body and the new quantity.
            destination.translation.y += 2. * half_height;
        }
        let turn = CardMove::new(destination.translation, destination, TURN)
            .lifted(Lift::arc(0.22))
            .turned(Turn::over(Vec3::Z));
        let mut above_source = turn.pose(0.);
        above_source.translation = from + Vec3::Y * 0.24;
        let rise = CardMove::new(from, above_source, RISE);
        let travel =
            CardMove::new(above_source.translation, turn.pose(0.), TRAVEL).lifted(Lift::arc(0.07));
        Self {
            rise,
            travel,
            turn,
            color,
            fades_in: change.source_remaining > 0,
            flip: change.flip.clone(),
        }
    }

    fn opacity(&self, elapsed: f32) -> f32 {
        if self.fades_in {
            (elapsed / RISE).clamp(0., 1.)
        } else {
            1.
        }
    }

    fn pose(&self, elapsed: f32) -> Transform {
        if !self.rise.finished(elapsed) {
            self.rise.pose(elapsed)
        } else if !self.travel.finished(elapsed - RISE) {
            self.travel.pose(elapsed - RISE)
        } else {
            self.turn.pose(elapsed - RISE - TRAVEL)
        }
    }
}

pub(super) fn animate(
    flights: Res<Flights>,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pieces: Query<(
        &Motion,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Option<&Children>,
    )>,
    faces: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let elapsed = (time.elapsed_secs_f64() - flights.started) as f32;
    for (motion, mut at, body, children) in &mut pieces {
        *at = motion.pose(elapsed);
        if !motion.fades_in {
            continue;
        }
        let alpha = motion.opacity(elapsed);
        if let Some(mut material) = materials.get_mut(&body.0)
            && material.base_color.alpha() != alpha
        {
            material.base_color = motion.color.with_alpha(alpha);
            material.alpha_mode = if alpha == 1. {
                AlphaMode::Opaque
            } else {
                AlphaMode::Blend
            };
        }
        for face in children.into_iter().flatten() {
            if let Ok(handle) = faces.get(*face)
                && let Some(mut material) = materials.get_mut(&handle.0)
                && material.base_color.alpha() != alpha
            {
                material.base_color.set_alpha(alpha);
            }
        }
    }
}

/// The moving mesh's actual pose, opacity and authored mark facing up, for the
/// native development seat. The rules observation still describes the saved result.
pub(super) fn observe(world: &mut World) -> Vec<serde_json::Value> {
    let mut motions = world.query::<(
        &Motion,
        &GlobalTransform,
        &MeshMaterial3d<StandardMaterial>,
        Option<&Children>,
    )>();
    let materials = world.resource::<Assets<StandardMaterial>>();
    let art = world.resource::<Art>();
    let marks = [
        (State::Active, art.images["atlas-active-star.png"].id()),
        (State::Open, art.images["atlas-casino-gold.png"].id()),
    ];
    let mut shown: Vec<_> = motions
        .iter(world)
        .map(|(motion, at, body, faces)| {
            let face_mark = faces.into_iter().flatten().find_map(|face| {
                let transform = world.get::<GlobalTransform>(*face)?;
                if transform.up().y <= 0.01 {
                    return None;
                }
                let handle = world.get::<MeshMaterial3d<StandardMaterial>>(*face)?;
                let texture = materials.get(handle)?.base_color_texture.as_ref()?;
                marks
                    .iter()
                    .find_map(|(state, id)| (*id == texture.id()).then_some(*state))
            });
            serde_json::json!({
                "flip": motion.flip,
                "center": at.translation().to_array(),
                "up": at.up().to_array(),
                "opacity": materials.get(body).map(|material| material.base_color.alpha()),
                "face_mark": face_mark,
            })
        })
        .collect();
    shown.sort_by_key(|row| row["flip"]["space"].to_string());
    shown
}
