//! An Available Forces trip is an effect of a successfully saved decision. During
//! travel only the arriving quantity waits off its map group.
use super::{
    piece_visuals::{add_group_face, group_half_height, spawn_group_body},
    *,
};
use crate::{Group, INVENTORY, Terrain, game::total};

pub(super) struct Trip {
    space: String,
    terrain: Terrain,
    group: Group,
    entering: bool,
    position: [f32; 2],
}

fn same_force(a: &Group, b: &Group) -> bool {
    a.faction == b.faction && a.kind == b.kind && a.state == b.state
}

/// Commit supplies both saved observations. Reconcile each faction and kind
/// against its own ledger change. Face changes within a space conserve its
/// ledger, and an ambiguous set of deltas cannot name a trip.
pub(super) fn committed(before: &Observation, after: &Observation) -> Option<Vec<Trip>> {
    let spaces = before.spaces.iter().zip(&after.spaces);
    if before.available == after.available
        && spaces.clone().all(|(old, new)| old.pieces == new.pieces)
    {
        return None;
    }
    let mut trips = Vec::new();
    for (faction, kind, _) in INVENTORY {
        let was = total(&before.available, faction, kind);
        let now = total(&after.available, faction, kind);
        if was == now {
            continue;
        }
        let entering = was > now;
        let matched: Vec<_> = spaces
            .clone()
            .flat_map(|(old, new)| {
                let (changed, other) = if entering { (new, old) } else { (old, new) };
                changed.pieces.iter().filter_map(move |group| {
                    if group.faction != faction || group.kind != kind {
                        return None;
                    }
                    let other_count = other
                        .pieces
                        .iter()
                        .find(|p| same_force(group, p))
                        .map_or(0, |p| p.count);
                    let count = group.count.saturating_sub(other_count);
                    (count > 0).then(|| Trip {
                        space: changed.id.clone(),
                        terrain: changed.terrain,
                        group: Group { count, ..*group },
                        entering,
                        position: map::slot_for(changed, group),
                    })
                })
            })
            .collect();
        if matched.iter().map(|trip| trip.group.count).sum::<u8>() == was.abs_diff(now) {
            trips.extend(matched);
        }
    }
    Some(trips)
}

#[derive(Resource, Default)]
pub(super) struct Flights {
    started: f64,
    generation: u64,
    active: Vec<Flight>,
}

struct Flight {
    trip: Trip,
    body: Entity,
    roundel: Option<Entity>,
    rest: Transform,
    counter: Option<Vec2>,
    ends: Option<(Vec3, Vec3)>,
}

impl Flights {
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn count(&self, space: &str, group: &Group, shown: u8) -> u8 {
        self.active
            .iter()
            .filter(|moving| {
                let trip = &moving.trip;
                trip.entering && trip.space == space && same_force(&trip.group, group)
            })
            .fold(shown, |count, moving| {
                count.saturating_sub(moving.trip.group.count)
            })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn stage(
    mut commands: Commands,
    mut ui: ResMut<Interface>,
    mut flights: ResMut<Flights>,
    art: Res<Art>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    counters: Query<(&available::Counter, &ComputedNode, &UiGlobalTransform)>,
) {
    let now = time.elapsed_secs_f64();
    let pending = ui.force_travel.take();
    if pending.is_some()
        || (!flights.active.is_empty() && now - flights.started >= f64::from(flight::DURATION))
    {
        let pending = pending.unwrap_or_default();
        if !pending.is_empty() || !flights.active.is_empty() {
            flights.generation += 1;
        }
        for moving in flights.active.drain(..) {
            commands.entity(moving.body).despawn();
        }
        flights.started = now;
        flights.active = pending
            .into_iter()
            .map(|trip| spawn(&mut commands, &art, &mut meshes, &mut materials, trip))
            .collect();
    }
    for moving in &mut flights.active {
        let group = &moving.trip.group;
        if let Some((_, node, at)) = counters.iter().find(|(key, node, _)| {
            **key == available::Counter(group.faction, group.kind) && node.size().x > 0.
        }) {
            moving.counter = Some(at.translation * node.inverse_scale_factor);
        }
    }
}

fn spawn(
    commands: &mut Commands,
    art: &Art,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    trip: Trip,
) -> Flight {
    let group = &trip.group;
    let scale = map::PIECE_SCALE;
    let height = group_half_height(group.kind);
    let [x, z] = trip.position;
    let at = map::at(x, z, map::top(trip.terrain)) + Vec3::Y * height * scale;
    let rest = Transform::from_translation(at).with_scale(Vec3::splat(scale));
    let body = spawn_group_body(
        commands,
        meshes,
        materials,
        group,
        String::from("Travel · force"),
        rest,
    );
    commands
        .entity(body)
        .insert((Visibility::Hidden, bevy::light::NotShadowCaster));
    add_group_face(commands, art, meshes, materials, body, group, false);
    let roundel = (group.count > 1).then(|| {
        let reach = roundel::side_reach(group.kind, false);
        roundel::moving(
            commands,
            art,
            group.count,
            body,
            Vec3::new(reach, -height + 0.025, 0.),
        )
    });
    Flight {
        trip,
        body,
        roundel,
        rest,
        counter: None,
        ends: None,
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn animate(
    mut flights: ResMut<Flights>,
    time: Res<Time>,
    camera: Single<(&Camera, &Transform), With<TableCamera>>,
    mut bodies: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
            Option<&Children>,
        ),
        Without<TableCamera>,
    >,
    faces: Query<&MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut roundels: roundel::Appearance,
) {
    let elapsed = (time.elapsed_secs_f64() - flights.started) as f32;
    let (camera, camera_at) = *camera;
    let camera_at = GlobalTransform::from(*camera_at);
    for moving in &mut flights.active {
        let Some(counter) = moving.counter.and_then(|pos| {
            flight::screen_point_at(camera, &camera_at, pos, moving.rest.translation)
        }) else {
            continue;
        };
        let Ok((mut at, mut visible, material, children)) = bodies.get_mut(moving.body) else {
            continue;
        };
        let (pose, opacity) = flight::sample(counter, moving.rest, moving.trip.entering, elapsed);
        *at = pose;
        *visible = Visibility::Visible;
        let map = moving.rest.translation;
        moving.ends = Some(if moving.trip.entering {
            (counter, map)
        } else {
            (map, counter)
        });
        for handle in std::iter::once(material).chain(
            children
                .into_iter()
                .flatten()
                .filter_map(|face| faces.get(*face).ok()),
        ) {
            if let Some(mut material) = materials.get_mut(&handle.0) {
                material.base_color = material.base_color.with_alpha(opacity);
                material.alpha_mode = AlphaMode::Blend;
            }
        }
        if let Some(roundel) = moving.roundel {
            roundels.opacity(roundel, opacity);
        }
    }
}

pub(super) fn unsettled(world: &mut World) -> Option<String> {
    let pending = world
        .resource::<Interface>()
        .force_travel
        .as_ref()
        .is_some_and(|trips| !trips.is_empty());
    (pending || !world.resource::<Flights>().active.is_empty()).then(|| "forces travelling".into())
}

/// Observe the actual scene body, its material and the projected endpoints.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    use serde_json::json;
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>();
    let Ok((camera, camera_at)) = cameras.single(world) else {
        return json!([]);
    };
    let point = |at| {
        camera
            .world_to_viewport(camera_at, at)
            .ok()
            .map(|p| p.to_array())
    };
    let materials = world.resource::<Assets<StandardMaterial>>();
    let visible = world
        .resource::<Flights>()
        .active
        .iter()
        .filter_map(|moving| {
            let (start, end) = moving.ends?;
            let at = world.get::<GlobalTransform>(moving.body)?;
            let material = world.get::<MeshMaterial3d<StandardMaterial>>(moving.body)?;
            let opacity = materials.get(&material.0)?.base_color.alpha();
            let marked_face = world
                .get::<Children>(moving.body)
                .into_iter()
                .flatten()
                .filter_map(|child| world.get::<MeshMaterial3d<StandardMaterial>>(*child))
                .any(|face| {
                    materials
                        .get(&face.0)
                        .is_some_and(|material| material.base_color_texture.is_some())
                });
            let roundel = moving
                .roundel
                .and_then(|entity| world.get::<Children>(entity))
                .into_iter()
                .flatten()
                .find_map(|child| world.get::<Text>(*child).map(|text| &text.0));
            let trip = &moving.trip;
            let key = available::counter_name(trip.group.faction, trip.group.kind);
            let (from, to) = if trip.entering {
                (key.as_str(), trip.space.as_str())
            } else {
                (trip.space.as_str(), key.as_str())
            };
            Some(json!({
                "group":trip.group,
                "from":from,
                "to":to,
                "start":point(start),
                "position":point(at.translation()),
                "end":point(end),
                "opacity":opacity,
                "marked_face":marked_face,
                "roundel":roundel,
            }))
        })
        .collect::<Vec<_>>();
    json!(visible)
}
