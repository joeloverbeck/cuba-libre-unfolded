//! Development seat adapter: resolves real controls and never executes rules directly.
use super::*;
use bevy::{input::keyboard::Key, time::TimeUpdateStrategy};
use serde_json::json;
use unfolded_tabletop::{
    capture,
    seat::{
        self as shared, Focus, Game as SeatGame, Hit, Logical, Name as TargetName, Observed, Seat,
        SeatKey, socket,
    },
};
const DETACHED: &str = "CUBA_LIBRE_AGENT_SEAT_DETACHED";
pub(super) fn run(options: Options) -> Result<(), String> {
    let listener = if let Some(path) = &options.socket {
        if !socket::detached(DETACHED) {
            return socket::launch(path, DETACHED).or_else(shared::refuse);
        }
        Some(socket::detach(path, DETACHED).or_else(shared::refuse)?)
    } else {
        None
    };
    let size = options.seat_size;
    let mut app = build(options, true).or_else(shared::refuse)?;
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    capture::install::<Camera2d>(&mut app, size);
    app.add_systems(
        PostStartup,
        |mut commands: Commands,
         surface: Res<capture::Surface>,
         camera: Single<Entity, With<Camera3d>>| {
            commands
                .entity(*camera)
                .insert(RenderTarget::Image(surface.image().into()));
        },
    );
    let mut seat = Seat::open(Table, &mut app)?;
    seat.admit(&mut app, Duration::from_secs(120))
        .map_err(|e| format!("Native admission failed: {e:?}"))?;
    if let Some(listener) = listener {
        socket::announce(&seat.ready(&mut app))?;
        listener.serve(&mut seat, &mut app, Duration::from_secs(30 * 60))
    } else {
        seat.serve(&mut app, std::io::stdin().lock(), std::io::stdout().lock())
    }
}
struct Table;
fn named(control: &Control) -> TargetName {
    if control.choice {
        TargetName::Choice(control.name.clone())
    } else {
        TargetName::Component(control.name.clone())
    }
}
fn piece_name(world: &World, entity: Entity) -> Option<String> {
    if let Some(label) = world.get::<count_label::Label>(entity) {
        Some(format!(
            "count:{}",
            world.get::<Name>(label.token)?.as_str()
        ))
    } else {
        Some(format!("piece:{}", world.get::<Name>(entity)?.as_str()))
    }
}
fn controls(world: &mut World) -> Vec<(Entity, Control, Vec2)> {
    let size = world
        .query_filtered::<&Camera, With<Camera2d>>()
        .single(world)
        .ok()
        .and_then(Camera::logical_viewport_size);
    let rows: Vec<_> = world
        .query::<(Entity, &Control)>()
        .iter(world)
        .map(|(e, c)| (e, c.clone()))
        .collect();
    let mut result = vec![];
    for (entity, c) in rows {
        if let Some(target) = shared::ui_target(world, entity, size) {
            if target.on_screen {
                result.push((entity, c, target.position));
            }
            continue;
        }
        let Action::Space(ref id) = c.action else {
            continue;
        };
        if world.resource::<Interface>().reader {
            continue;
        }
        let Some(label) = world
            .resource::<Art>()
            .labels
            .iter()
            .find(|l| &l.space == id)
        else {
            continue;
        };
        let [x, y, w, h] = label.bounds;
        let position = map_at(x + w / 2., y + h / 2., 0.30);
        if let Ok((camera, at)) = world
            .query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>()
            .single(world)
            && let Ok(point) = camera.world_to_viewport(at, position)
        {
            result.push((entity, c, point));
        }
    }
    if !world.resource::<Interface>().reader {
        let targets = world
            .query::<(Entity, &status::SpaceTarget)>()
            .iter(world)
            .map(|(e, t)| (e, t.0.clone()))
            .collect::<Vec<_>>();
        for (entity, space) in targets {
            let Some(name) = piece_name(world, entity) else {
                continue;
            };
            let position = if let Some(target) = shared::ui_target(world, entity, size) {
                target.on_screen.then_some(target.position)
            } else if let Some(at) = world.get::<GlobalTransform>(entity).copied() {
                world
                    .query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>()
                    .single(world)
                    .ok()
                    .and_then(|(camera, camera_at)| {
                        camera
                            .world_to_viewport(camera_at, at.translation())
                            .ok()
                            .filter(|p| {
                                camera
                                    .logical_viewport_rect()
                                    .is_some_and(|r| r.contains(*p))
                            })
                    })
            } else {
                None
            };
            if let Some(position) = position {
                result.push((
                    entity,
                    Control {
                        name,
                        choice: false,
                        action: Action::Space(space),
                    },
                    position,
                ));
            }
        }
    }
    result
}
impl SeatGame for Table {
    fn observe(&self, world: &mut World) -> Observed {
        // Inspect actual loaded region surfaces, including their side faces.
        let surfaces = world
            .query::<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>)>()
            .iter(world)
            .filter_map(|(entity, mesh, material)| {
                let mut current = Some(entity);
                while let Some(e) = current {
                    if let Some(Control {
                        action: Action::Space(space),
                        ..
                    }) = world.get::<Control>(e)
                    {
                        let material = world
                            .resource::<Assets<StandardMaterial>>()
                            .get(&material.0)?;
                        let mesh = world.resource::<Assets<Mesh>>().get(&mesh.0)?;
                        return Some(
                            json!({"space":space,"textured":material.base_color_texture.is_some(),
                            "material":world.get::<bevy::gltf::GltfMaterialName>(entity).map(|name| &name.0),
                            "uv":mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some()}),
                        );
                    }
                    current = world.get::<ChildOf>(e).map(ChildOf::parent);
                }
                None
            })
            .collect::<Vec<_>>();
        let controls = controls(world)
            .into_iter()
            .map(|(_, c, p)| json!({"name":c.name,"choice":c.choice,"position":[p.x,p.y]}))
            .collect::<Vec<_>>();
        let pending = self.unsettled(world);
        let size = world
            .query_filtered::<&Camera, With<Camera2d>>()
            .single(world)
            .ok()
            .and_then(Camera::logical_viewport_size);
        let standing_rows: Vec<_> = world
            .query::<(Entity, &StandingText, &Text)>()
            .iter(world)
            .map(|(e, f, t)| (e, f.0, t.0.clone()))
            .collect();
        let standings:Vec<_>=standing_rows.into_iter().map(|(entity,faction,text)|json!({"faction":faction,"text":text,"visible":shared::ui_target(world,entity,size).is_some_and(|t|t.on_screen)})).collect();
        let map_viewport = world
            .query_filtered::<&Camera, With<Camera3d>>()
            .single(world)
            .ok()
            .and_then(Camera::logical_viewport_rect)
            .map(|r| [r.min.x, r.min.y, r.max.x, r.max.y]);
        let status = world.query::<&status::Shown>().iter(world).next().cloned();
        let victory = world.query::<&victory::Shown>().iter(world).next().cloned();
        let victory_count = world.query::<&victory::Shown>().iter(world).count();
        let victory_text = world.query_filtered::<(&Text, &ComputedNode, &UiGlobalTransform), With<victory::Tag>>()
            .iter(world).map(|(text, node, at)| {
                let center = at.translation * node.inverse_scale_factor;
                let half = node.size() * node.inverse_scale_factor / 2.;
                json!({"text":text.0,"bounds":[center.x-half.x,center.y-half.y,center.x+half.x,center.y+half.y]})
            }).collect::<Vec<_>>();
        let play = world.resource::<Play>();
        let ui = world.resource::<Interface>();
        let pose = world.resource::<Orbit>().pose();
        let water_frame = world.get_resource::<Water>().map(|water| water.frame);
        Observed {
            observation: serde_json::to_value(play.game().observe()).unwrap(),
            table: json!({"controls":controls,"surfaces":surfaces,"standings":standings,"map_viewport":map_viewport,"viewport":size.map(|s| [s.x,s.y]),"status":status,"victory":victory,"victory_count":victory_count,"victory_text":victory_text,"water_frame":water_frame,"settled":pending.is_empty(),"pending":pending,"reader":ui.reader,"hover":ui.hover.visible(),"notice":ui.notice,"error":play.error,"camera":{"yaw":pose.yaw,"pitch":pose.pitch,"distance":pose.distance,"target":[pose.target.x,pose.target.y,pose.target.z]},"native_limits":"windowless real Bevy input/render path; no physical mouse comfort or owner acceptance"}),
        }
    }
    fn revision(&self, world: &mut World) -> u64 {
        world.resource::<Play>().game().observe().revision
    }
    fn resolve(&self, world: &mut World, name: &TargetName) -> Result<Vec<Vec2>, String> {
        let found: Vec<_> = controls(world)
            .into_iter()
            .filter(|(_, c, _)| &named(c) == name)
            .collect();
        if found.len() != 1 {
            return Err(format!(
                "Expected exactly one shown target {name:?}, found {}",
                found.len()
            ));
        }
        let mut points = vec![found[0].2];
        // A count can cover the center of its solid piece. Try exposed body
        // points too; the shared seat still verifies the actual picked name.
        if found[0].1.name.starts_with("piece:") {
            for x in [-6., -3., 0., 3., 6.] {
                for y in [-6., -3., 0., 3., 6.] {
                    points.push(found[0].2 + Vec2::new(x, y));
                }
            }
        }
        Ok(points)
    }
    fn identify(&self, world: &mut World, entity: Entity) -> Hit {
        let mut current = Some(entity);
        while let Some(e) = current {
            if let Some(target) = world.get::<status::SpaceTarget>(e) {
                return Hit {
                    name: piece_name(world, e).map(TargetName::Component),
                    description: json!({"space":target.0}),
                };
            }
            if let Some(control) = world.get::<Control>(e) {
                return Hit {
                    name: Some(named(control)),
                    description: json!({"control":control.name}),
                };
            }
            current = world.get::<ChildOf>(e).map(ChildOf::parent);
        }
        Hit {
            name: None,
            description: json!({"surface":"map or authored chrome"}),
        }
    }
    fn focus(&self, _: &mut World) -> Focus {
        Focus::Nothing
    }
    fn unsettled(&self, world: &mut World) -> Vec<String> {
        let play = world.resource::<Play>();
        let ui = world.resource::<Interface>();
        let mut pending = vec![];
        if play.session.is_none() {
            pending.push("asset/save admission".into());
        }
        if ui.hover.waiting() {
            pending.push("standing hover dwell".into());
        }
        if ui.status.waiting() {
            pending.push("region status hover dwell".into());
        }
        if ui.dirty {
            pending.push("control redraw".into());
        }
        if world.resource::<Orbit>().is_moving() {
            pending.push("camera movement".into());
        }
        pending
    }
    fn advance(&self, world: &mut World, by: Duration) {
        *world.resource_mut::<TimeUpdateStrategy>() = TimeUpdateStrategy::ManualDuration(by);
    }
    fn admission(&self, world: &World) -> shared::Admission {
        let play = world.resource::<Play>();
        if let Some(error) = &play.error {
            shared::Admission::Refused(error.clone())
        } else if play.session.is_some() {
            shared::Admission::Admitted
        } else {
            shared::Admission::Checking
        }
    }
    fn pipelines_ready(&self, world: &World) -> bool {
        world
            .resource::<RenderReady>()
            .pipelines
            .load(Ordering::Acquire)
    }
    fn keys(&self) -> &[SeatKey] {
        &KEYS
    }
}
static KEYS: [SeatKey; 11] = [
    SeatKey {
        name: "KeyW",
        code: KeyCode::KeyW,
        logical: Logical::Typed("w"),
        commits: false,
    },
    SeatKey {
        name: "KeyA",
        code: KeyCode::KeyA,
        logical: Logical::Typed("a"),
        commits: false,
    },
    SeatKey {
        name: "KeyS",
        code: KeyCode::KeyS,
        logical: Logical::Typed("s"),
        commits: false,
    },
    SeatKey {
        name: "KeyD",
        code: KeyCode::KeyD,
        logical: Logical::Typed("d"),
        commits: false,
    },
    SeatKey {
        name: "Equal",
        code: KeyCode::Equal,
        logical: Logical::Typed("="),
        commits: false,
    },
    SeatKey {
        name: "Minus",
        code: KeyCode::Minus,
        logical: Logical::Typed("-"),
        commits: false,
    },
    SeatKey {
        name: "ArrowLeft",
        code: KeyCode::ArrowLeft,
        logical: Logical::Named(Key::ArrowLeft),
        commits: false,
    },
    SeatKey {
        name: "ArrowRight",
        code: KeyCode::ArrowRight,
        logical: Logical::Named(Key::ArrowRight),
        commits: false,
    },
    SeatKey {
        name: "ArrowUp",
        code: KeyCode::ArrowUp,
        logical: Logical::Named(Key::ArrowUp),
        commits: false,
    },
    SeatKey {
        name: "ArrowDown",
        code: KeyCode::ArrowDown,
        logical: Logical::Named(Key::ArrowDown),
        commits: false,
    },
    SeatKey {
        name: "Home",
        code: KeyCode::Home,
        logical: Logical::Named(Key::Home),
        commits: false,
    },
];
