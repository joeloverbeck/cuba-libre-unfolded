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
    confirmed_flips(&preview.flips, &before.spaces, &after.spaces)
}

fn confirmed_flips(flips: &[Flip], before: &[crate::Space], after: &[crate::Space]) -> Vec<Change> {
    flips
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
            let previous = before.iter().find(|s| s.id == flip.space)?;
            let after = after.iter().find(|s| s.id == flip.space)?;
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

    pub(super) fn source_pose(&self) -> Transform {
        self.rise.pose(0.)
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

#[cfg(test)]
mod tests {
    use super::*;

    // Renderer inputs only. The current opening cannot put three 26July
    // Guerrillas in a Terror target, and its Base spaces cannot start a reverse
    // Rally with an Active group. These positions never enter Game or Session.
    fn space(groups: &[(State, u8)]) -> crate::Space {
        crate::Space {
            id: "santiago".into(),
            name: "Santiago de Cuba".into(),
            terrain: Terrain::City,
            population: 1,
            economy: 0,
            support: 0,
            pieces: groups
                .iter()
                .map(|&(state, count)| Group {
                    faction: Faction::M26,
                    kind: Kind::Guerrilla,
                    state,
                    count,
                })
                .collect(),
            terror: 0,
        }
    }

    fn flip(from: State, to: State) -> Flip {
        Flip {
            space: "santiago".into(),
            faction: Faction::M26,
            kind: Kind::Guerrilla,
            from,
            to,
            count: 1,
        }
    }

    #[test]
    fn one_of_three_leaves_two_and_reaches_the_other_slot_before_turning() {
        let before = space(&[(State::Underground, 3)]);
        let committed = space(&[(State::Underground, 2), (State::Active, 1)]);
        let flip = flip(State::Underground, State::Active);
        assert!(
            confirmed_flips(
                std::slice::from_ref(&flip),
                std::slice::from_ref(&before),
                std::slice::from_ref(&before),
            )
            .is_empty()
        );
        let mut pending = confirmed_flips(&[flip], &[before], std::slice::from_ref(&committed));
        let mut flights = Flights::default();
        assert!(flights.refresh(&mut pending, 0.));
        assert_eq!(flights.count(&committed.id, &committed.pieces[0]), 2);
        assert_eq!(flights.count(&committed.id, &committed.pieces[1]), 0);

        let (body, effect) = flights.in_space(&committed.id).next().unwrap();
        assert_eq!(body.count, 1);
        assert_eq!(body.state, State::Active);
        let source = Vec3::new(0., 0.34, 0.);
        let destination = Transform::from_xyz(0., 0.34, 0.5);
        let motion = Motion::new(source, destination, 0.1, Color::WHITE, effect);
        assert_eq!(motion.pose(0.).translation, source);
        assert_eq!(motion.opacity(0.), 0.);
        assert!(motion.opacity(0.08) > 0. && motion.opacity(0.08) < 1.);
        let crossing = motion.pose(0.35);
        assert!(crossing.translation.z > source.z);
        assert!(crossing.translation.z < destination.translation.z);
        assert!(crossing.translation.y > source.y);
        assert!(crossing.up().y < -0.98);
        assert_eq!(motion.opacity(0.35), 1.);
        let turning = motion.pose(0.79);
        assert!((turning.translation.z - destination.translation.z).abs() < 0.001);
        assert!(turning.translation.y > destination.translation.y + 0.17);
        assert!(turning.up().y.abs() < 0.04);
        assert_eq!(motion.pose(2.), destination);

        assert!(flights.refresh(&mut pending, 2.));
        assert!(!flights.refresh(&mut pending, 3.));
        assert_eq!(flights.count(&committed.id, &committed.pieces[0]), 2);
        assert_eq!(flights.count(&committed.id, &committed.pieces[1]), 1);
        assert_eq!(flights.in_space(&committed.id).count(), 0);
    }

    #[test]
    fn reverse_flip_preserves_the_resident_count_until_the_singleton_lands() {
        let before = space(&[(State::Active, 1), (State::Underground, 2)]);
        let committed = space(&[(State::Underground, 3)]);
        let flip = flip(State::Active, State::Underground);
        let mut pending = confirmed_flips(&[flip], &[before], std::slice::from_ref(&committed));
        let mut flights = Flights::default();
        assert!(flights.refresh(&mut pending, 0.));
        assert_eq!(flights.count(&committed.id, &committed.pieces[0]), 2);
        let (body, effect) = flights.in_space(&committed.id).next().unwrap();
        assert_eq!(body.state, State::Underground);
        assert_eq!(effect.flip.from, State::Active);

        let source = Vec3::new(0., 0.34, 0.5);
        let destination = Transform::from_xyz(0., 0.34, 0.);
        let half_height = 0.1;
        let motion = Motion::new(source, destination, half_height, Color::WHITE, effect);
        assert_eq!(motion.opacity(0.), 1.);
        // Local -Y carries the retreating Active symbol. The same old-side
        // mount is exercised by an open Casino's native closure.
        assert!((motion.pose(0.).rotation * Vec3::NEG_Y).y > 0.98);
        assert!((motion.pose(0.9).rotation * Vec3::NEG_Y).y < -0.25);
        let arriving = motion.pose(2.);
        assert!(
            (arriving.translation.y - destination.translation.y - 2. * half_height).abs() < 0.001
        );
        assert_eq!(arriving.translation.xz(), destination.translation.xz());

        assert!(flights.refresh(&mut pending, 2.));
        assert_eq!(flights.count(&committed.id, &committed.pieces[0]), 3);
        assert_eq!(flights.in_space(&committed.id).count(), 0);
    }
}
