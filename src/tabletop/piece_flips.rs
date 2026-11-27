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
        .filter(|flip| flip.kind == Kind::Guerrilla && flip.to == State::Active)
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
    mut pieces: Query<(&Motion, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
) {
    let elapsed = (time.elapsed_secs_f64() - flights.started) as f32;
    for (motion, mut at, material) in &mut pieces {
        *at = motion.pose(elapsed);
        if motion.fades_in
            && let Some(mut material) = materials.get_mut(&material.0)
        {
            let alpha = (elapsed / RISE).clamp(0., 1.);
            material.base_color = motion.color.with_alpha(alpha);
            material.alpha_mode = if alpha == 1. {
                AlphaMode::Opaque
            } else {
                AlphaMode::Blend
            };
        }
    }
}
