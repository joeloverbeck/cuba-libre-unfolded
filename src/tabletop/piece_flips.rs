//! Presentation of a saved Terror activation. The session's position is already
//! authoritative while the affected representative travels and the count catches up.
use super::*;
use crate::{Flip, Group};
use unfolded_tabletop::card_motion::{CardMove, Lift, Turn};

const TRAVEL: f32 = 0.4;
const TURN: f32 = 0.52;

#[derive(Resource, Default)]
pub(super) struct Flights {
    started: f64,
    flips: Vec<Flip>,
}

impl Flights {
    pub(super) fn refresh(&mut self, pending: &mut Vec<Flip>, now: f64) -> bool {
        if !pending.is_empty() {
            self.flips = std::mem::take(pending);
            self.started = now;
            return true;
        }
        if self.moving() && now - self.started >= f64::from(TRAVEL + TURN) {
            self.flips.clear();
            return true;
        }
        false
    }

    pub(super) fn moving(&self) -> bool {
        !self.flips.is_empty()
    }

    pub(super) fn in_space(&self, space: &str) -> impl Iterator<Item = (Group, &Flip)> {
        self.flips
            .iter()
            .filter(move |flip| flip.space == space)
            .map(|flip| {
                (
                    Group {
                        faction: flip.faction,
                        kind: flip.kind,
                        state: flip.to,
                        count: flip.count,
                    },
                    flip,
                )
            })
    }

    pub(super) fn count(&self, space: &str, group: &Group) -> u8 {
        self.flips
            .iter()
            .find(|f| {
                f.space == space
                    && f.faction == group.faction
                    && f.kind == group.kind
                    && f.to == group.state
            })
            .map_or(group.count, |flip| group.count.saturating_sub(flip.count))
    }
}

#[derive(Component)]
pub(super) struct Motion {
    travel: CardMove,
    turn: CardMove,
    color: Color,
}

impl Motion {
    pub(super) fn new(from: Vec3, rest: Transform, color: Color) -> Self {
        let turn = CardMove::new(rest.translation, rest, TURN)
            .lifted(Lift::arc(0.22))
            .turned(Turn::over(Vec3::Z));
        let travel = CardMove::new(from, turn.pose(0.), TRAVEL).lifted(Lift::arc(0.14));
        Self {
            travel,
            turn,
            color,
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
        *at = if motion.travel.finished(elapsed) {
            motion.turn.pose(elapsed - TRAVEL)
        } else {
            motion.travel.pose(elapsed)
        };
        if let Some(mut material) = materials.get_mut(&material.0) {
            let alpha = (elapsed / 0.14).clamp(0., 1.);
            material.base_color = motion.color.with_alpha(alpha);
            material.alpha_mode = if alpha == 1. {
                AlphaMode::Opaque
            } else {
                AlphaMode::Blend
            };
        }
    }
}
