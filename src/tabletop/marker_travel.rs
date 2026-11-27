//! The presentation of changes to Terror markers after a successful decision.
use super::*;
use bevy::{light::NotShadowCaster, world_serialization::WorldInstanceReady};
use serde_json::{Value, json};
use std::collections::VecDeque;

const BADGE_OFFSET: Vec3 = Vec3::new(0.15, 0.025, 0.);

#[derive(Clone, Copy)]
struct Change {
    before: u8,
    after: u8,
}

impl Change {
    fn entering(self) -> bool {
        self.before < self.after
    }

    fn resident(self) -> u8 {
        self.before.min(self.after)
    }

    fn quantity(self) -> u8 {
        self.before.abs_diff(self.after)
    }
}

struct Trip {
    change: Change,
    running: Option<(Entity, f64)>,
}

#[derive(Default)]
pub(super) struct Travel(BTreeMap<String, VecDeque<Trip>>);

impl Travel {
    /// Record differences in saved facts, independently of the kind of decision.
    pub(super) fn committed(&mut self, before: &Observation, after: &Observation) {
        for space in &after.spaces {
            if let Some(was) = before.spaces.iter().find(|was| was.id == space.id)
                && was.terror != space.terror
            {
                self.0.entry(space.id.clone()).or_default().push_back(Trip {
                    change: Change {
                        before: was.terror,
                        after: space.terror,
                    },
                    running: None,
                });
            }
        }
    }

    pub(super) fn moving(&self) -> bool {
        !self.0.is_empty()
    }

    fn resident(&self, space: &crate::Space) -> u8 {
        self.0
            .get(&space.id)
            .and_then(VecDeque::front)
            .map_or(space.terror, |trip| trip.change.resident())
    }
}

/// The in-flight copy owns each fading material. The original glTF material is
/// still used by every resident bell.
#[derive(Component)]
pub(super) struct InFlight {
    space: String,
    change: Change,
    alpha: f32,
    fades: Vec<(Handle<StandardMaterial>, f32)>,
    badge: Option<Entity>,
}

fn bind_fades(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    parents: Query<&Children>,
    meshes: Query<&MeshMaterial3d<StandardMaterial>>,
    mut flying: Query<&mut InFlight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(mut visual) = flying.get_mut(ready.entity) else {
        return;
    };
    for child in parents.iter_descendants(ready.entity) {
        let Ok(mesh) = meshes.get(child) else {
            continue;
        };
        let Some(mut material) = materials.get(&mesh.0).cloned() else {
            continue;
        };
        let original = material.base_color.alpha();
        material.base_color = material.base_color.with_alpha(original * visual.alpha);
        material.alpha_mode = AlphaMode::Blend;
        let fade = materials.add(material);
        commands.entity(child).insert((
            MeshMaterial3d(fade.clone()),
            Pickable::IGNORE,
            NotShadowCaster,
        ));
        visual.fades.push((fade, original));
    }
}

fn rest(space: &crate::Space) -> Transform {
    let [x, z] = map::marker(&space.id);
    Transform::from_translation(map::at(x, z, map::top(space.terrain)))
        .with_scale(Vec3::splat(map::PIECE_SCALE))
}

pub(super) struct Resident {
    quantity: u8,
    bell: Entity,
    badge: Entity,
}

fn resident_bell(
    commands: &mut Commands,
    art: &Art,
    index: usize,
    space: &crate::Space,
    n: u8,
) -> Resident {
    // One authored bell per space. Its glTF meshes stay pickable and the
    // roundel carries the quantity. Place each bell at its own phase in the loop.
    let bell = commands
        .spawn((
            Name::new(format!("{} Terror", space.id)),
            WorldAssetRoot(art.terror.clone()),
            rest(space),
            TerrorToken {
                phase: (index as f64 * 0.618_033_988_75).fract(),
            },
            PieceVisual,
            status::SpaceTarget(space.id.clone()),
        ))
        .id();
    let badge = roundel::spawn(commands, art, &space.id, n.to_string(), bell, BADGE_OFFSET);
    Resident {
        quantity: n,
        bell,
        badge,
    }
}

#[allow(clippy::too_many_arguments)] // Bevy inputs to the presentation system.
pub(super) fn present(
    mut commands: Commands,
    play: Res<Play>,
    mut ui: ResMut<Interface>,
    art: Res<Art>,
    layout: Res<areas::Layout>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<TableCamera>>,
    time: Res<Time>,
    mut flying: Query<(&mut Transform, &mut InFlight)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut appearance: roundel::Appearance,
    children: Query<&Children>,
    mut counts: ResMut<Counts>,
    mut residents: Local<BTreeMap<String, Resident>>,
) {
    let game = play.session.as_ref().map(Session::game);
    #[cfg(feature = "occupancy-review")]
    let game = play
        .options
        .occupancy_review
        .as_ref()
        .map(|review| &review.position)
        .or(game);
    let Some(game) = game else {
        return;
    };
    let travel = &mut ui.marker_travel;
    let now = time.elapsed_secs_f64();
    let (camera, camera_at) = *camera;
    let supply = supply::endpoint(&layout, &window);

    for (id, queue) in &mut travel.0 {
        if let Some((entity, start)) = queue.front().and_then(|trip| trip.running)
            && now - start >= f64::from(flight::DURATION)
        {
            commands.entity(entity).despawn();
            queue.pop_front();
        }
        let Some(trip) = queue.front_mut() else {
            continue;
        };
        let space = game
            .spaces
            .iter()
            .find(|space| &space.id == id)
            .expect("a marker change names a map space");
        let rest = rest(space);
        let Some(supply) = flight::screen_point_at(camera, camera_at, supply, rest.translation)
        else {
            // A minimized window can lose its projection. Release the
            // presentation so the saved decision can settle.
            if let Some((entity, _)) = trip.running {
                commands.entity(entity).despawn();
            }
            queue.clear();
            continue;
        };
        let age = trip.running.map_or(0., |(_, start)| (now - start) as f32);
        let (pose, alpha) = flight::sample(supply, rest, trip.change.entering(), age);
        if let Some((entity, _)) = trip.running {
            let Ok((mut current, mut visual)) = flying.get_mut(entity) else {
                continue;
            };
            *current = pose;
            visual.alpha = alpha;
            for (handle, original) in &visual.fades {
                if let Some(mut material) = materials.get_mut(handle) {
                    material.base_color = material.base_color.with_alpha(original * alpha);
                }
            }
            if let Some(badge) = visual.badge {
                appearance.opacity(badge, alpha);
            }
        } else {
            let entity = commands
                .spawn((
                    Name::new("Terror marker in flight"),
                    WorldAssetRoot(art.terror.clone()),
                    pose,
                    Pickable::IGNORE,
                ))
                .observe(bind_fades)
                .id();
            let quantity = trip.change.quantity();
            let badge = (quantity > 1)
                .then(|| roundel::moving(&mut commands, &art, quantity, entity, BADGE_OFFSET));
            commands.entity(entity).insert(InFlight {
                space: id.clone(),
                change: trip.change,
                alpha,
                fades: Vec::new(),
                badge,
            });
            trip.running = Some((entity, now));
        }
    }
    travel.0.retain(|_, queue| !queue.is_empty());

    for (index, space) in game.spaces.iter().enumerate() {
        let n = travel.resident(space);
        let previous = residents
            .get(&space.id)
            .map_or(0, |resident| resident.quantity);
        if n == previous {
            continue;
        }
        if n == 0 {
            let resident = residents
                .remove(&space.id)
                .expect("a previous marker quantity had a bell");
            commands.entity(resident.bell).despawn();
            commands.entity(resident.badge).despawn();
        } else if let Some(resident) = residents.get_mut(&space.id) {
            if let Ok(digits) = children.get(resident.badge) {
                for digit in digits.iter() {
                    commands.entity(digit).insert(Text::new(n.to_string()));
                }
            }
            resident.quantity = n;
        } else {
            residents.insert(
                space.id.clone(),
                resident_bell(&mut commands, &art, index, space, n),
            );
        }
        counts.unsettle();
    }
}

/// The projected position and material opacity of each in-flight copy, in the
/// same logical window pixels as the marker supply and roundels.
pub(super) fn observe(world: &mut World) -> Value {
    let camera = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .single(world)
        .ok()
        .map(|(camera, at)| (camera.clone(), *at));
    let mut trips = world.query::<(&InFlight, &GlobalTransform)>().iter(world)
        .map(|(visual, at)| {
            let point = camera.as_ref().and_then(|(camera, camera_at)| {
                camera.world_to_viewport(camera_at, at.translation()).ok()
            });
            let direction = if visual.change.entering() { "placement" } else { "return" };
            json!({"space":visual.space,"direction":direction,"quantity":visual.change.quantity(),"at":point.map(|p| [p.x,p.y]),"alpha":visual.alpha})
        }).collect::<Vec<_>>();
    trips.sort_by_key(|trip| trip["space"].to_string());
    json!(trips)
}
