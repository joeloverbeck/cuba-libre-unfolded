//! Development seat adapter: resolves real controls and never executes rules directly.
use super::*;
use bevy::{input::keyboard::Key, time::TimeUpdateStrategy};
use serde_json::json;
use std::sync::LazyLock;
use unfolded_tabletop::{
    capture,
    label_anchor::Anchor,
    seat::{
        self as shared, Focus, Game as SeatGame, Hit, Logical, Name as TargetName, Observed,
        SeatKey,
        start::{self, Start},
    },
};
/// Cuba Libre's seat start: its marker, the shared limits and its words for an
/// admission that ran out of time. The order of the steps and the refusals are shared.
const START: Start = Start::new("CUBA_LIBRE_AGENT_SEAT_DETACHED", |_, limit| {
    format!(
        "Native admission failed: not finished within {} seconds.",
        limit.as_secs()
    )
});
pub(super) fn run(options: Options) -> Result<(), String> {
    let socket = options.socket.clone();
    START.run(socket.as_deref(), || {
        let size = options.seat_size;
        let mut app = build(options, true)?;
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        // The 3D camera is marked for the surface where the table spawns it.
        capture::install::<Camera2d>(&mut app, size);
        Ok(start::Table::new(app, Table))
    })
}
struct Table;
/// The digits the transfer modal's amount field takes, as the seat names them.
const DIGITS: [(&str, KeyCode, &str); 10] = [
    ("0", KeyCode::Digit0, "0"),
    ("1", KeyCode::Digit1, "1"),
    ("2", KeyCode::Digit2, "2"),
    ("3", KeyCode::Digit3, "3"),
    ("4", KeyCode::Digit4, "4"),
    ("5", KeyCode::Digit5, "5"),
    ("6", KeyCode::Digit6, "6"),
    ("7", KeyCode::Digit7, "7"),
    ("8", KeyCode::Digit8, "8"),
    ("9", KeyCode::Digit9, "9"),
];
/// The keys the seat may press: Tab moves keyboard focus through the action choice,
/// the BRIEFING ribbon and its tabs, or the transfer modal; Enter activates the
/// focused control and can commit, so a request names that control; Escape unpins a
/// briefing, closes the ribbon's tag or cancels the transfer (Wait in a giver's
/// decision); Backspace and the digits edit the transfer amount. Then the shared
/// camera's keys, none of which commits.
static KEYS: LazyLock<Vec<SeatKey>> = LazyLock::new(|| {
    let key = |name, code, logical, commits| SeatKey {
        name,
        code,
        logical: Logical::Named(logical),
        commits,
    };
    [
        key("Tab", KeyCode::Tab, Key::Tab, false),
        key("Enter", KeyCode::Enter, Key::Enter, true),
        key("Escape", KeyCode::Escape, Key::Escape, false),
        key("Backspace", KeyCode::Backspace, Key::Backspace, false),
    ]
    .into_iter()
    // The transfer modal's amount field takes typed digits.
    .chain(DIGITS.iter().map(|&(name, code, typed)| SeatKey {
        name,
        code,
        logical: Logical::Typed(typed),
        commits: false,
    }))
    .chain(unfolded_tabletop::camera::SEAT_KEYS.iter().cloned())
    .collect()
});
/// A UI node's rectangle in logical window pixels: [left, top, right, bottom].
pub(super) fn logical_bounds(node: &ComputedNode, at: &UiGlobalTransform) -> [f32; 4] {
    let center = at.translation * node.inverse_scale_factor;
    let half = node.size() * node.inverse_scale_factor / 2.;
    [
        center.x - half.x,
        center.y - half.y,
        center.x + half.x,
        center.y + half.y,
    ]
}
fn named(control: &Control) -> TargetName {
    if control.choice {
        TargetName::Choice(control.name.clone())
    } else {
        TargetName::Component(control.name.clone())
    }
}
fn piece_name(world: &World, entity: Entity) -> Option<String> {
    if world.get::<Count>(entity).is_some()
        && let Some(LabelAnchor {
            to: Anchor::Entity { entity: token, .. },
            ..
        }) = world.get::<LabelAnchor>(entity)
    {
        Some(format!("count:{}", world.get::<Name>(*token)?.as_str()))
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
        let victory_text = world
            .query_filtered::<(&emblems::Plain, &ComputedNode, &UiGlobalTransform), With<victory::Tag>>()
            .iter(world)
            .map(|(text, node, at)| json!({"text":text.0,"bounds":logical_bounds(node, at)}))
            .collect::<Vec<_>>();
        let top = world.query::<&top::Shown>().iter(world).next().cloned();
        let preview = world.query::<&preview::Shown>().iter(world).next().cloned();
        let map_targets = world.resource::<highlights::Targeting>().targets.clone();
        let mut highlights = world
            .query::<(&highlights::Decal, &Visibility)>()
            .iter(world)
            .filter(|(_, v)| **v != Visibility::Hidden)
            .map(|(d, _)| d.file.clone())
            .collect::<Vec<_>>();
        highlights.sort();
        let popover = popover::observe(world);
        let transfer = transfer::observe(world);
        let briefing = world.query::<&choice::Shown>().iter(world).next().cloned();
        let mut choice_parts = world
            .query::<(&choice::Part, &ComputedNode, &UiGlobalTransform, Option<&emblems::Plain>)>()
            .iter(world)
            .map(|(part, node, at, text)| {
                (part.0.clone(), json!({"part":part.0,"text":text.map(|t| &t.0),"bounds":logical_bounds(node, at)}))
            })
            .collect::<Vec<_>>();
        choice_parts.sort_by(|a, b| a.0.cmp(&b.0));
        let choice_parts = choice_parts
            .into_iter()
            .map(|(_, part)| part)
            .collect::<Vec<_>>();
        let mut top_parts = world
            .query::<(&top::Part, &ComputedNode, &UiGlobalTransform, Option<&Text>)>()
            .iter(world)
            .map(|(part, node, at, text)| {
                (part.0.clone(), json!({"part":part.0,"text":text.map(|t| &t.0),"bounds":logical_bounds(node, at)}))
            })
            .collect::<Vec<_>>();
        top_parts.sort_by(|a, b| a.0.cmp(&b.0));
        let top_parts = top_parts
            .into_iter()
            .map(|(_, part)| part)
            .collect::<Vec<_>>();
        let top_text = world
            .query_filtered::<(&emblems::Plain, &ComputedNode, &UiGlobalTransform), With<top::Tag>>(
            )
            .iter(world)
            .map(|(text, node, at)| json!({"text":text.0,"bounds":logical_bounds(node, at)}))
            .collect::<Vec<_>>();
        let inline_text = emblems::observe(world);
        let node = world.resource::<TerrorLoop>().node;
        let mut terror = world
            .query::<(&TerrorPlayer, &AnimationPlayer)>()
            .iter(world)
            .map(|(token, player)| {
                (
                    token.0.clone(),
                    player.animation(node).map(|active| active.seek_time()),
                )
            })
            .collect::<Vec<_>>();
        terror.sort_by(|a, b| a.0.cmp(&b.0));
        let terror = terror
            .into_iter()
            .map(|(space, seek)| json!({"space":space,"seek":seek}))
            .collect::<Vec<_>>();
        let play = world.resource::<Play>();
        let ui = world.resource::<Interface>();
        let pose = world.resource::<Orbit>().pose();
        let water_frame = world.get_resource::<Water>().map(|water| water.frame);
        Observed {
            observation: serde_json::to_value(play.game().observe()).unwrap(),
            table: json!({"controls":controls,"surfaces":surfaces,"standings":standings,"map_viewport":map_viewport,"viewport":size.map(|s| [s.x,s.y]),"status":status,"victory":victory,"victory_count":victory_count,"victory_text":victory_text,"top":top,"preview":preview,"briefing":briefing,"choice":choice::observe(ui),"choice_parts":choice_parts,"map_targets":map_targets,"highlights":highlights,"popover":popover,"transfer":transfer,"top_text":top_text,"inline_text":inline_text,"top_parts":top_parts,"water_frame":water_frame,"terror":terror,"settled":pending.is_empty(),"pending":pending,"reader":ui.reader,"hover":ui.hover.visible(),"notice":ui.notice,"error":play.error,"camera":{"yaw":pose.yaw,"pitch":pose.pitch,"distance":pose.distance,"target":[pose.target.x,pose.target.y,pose.target.z]},"native_limits":"windowless real Bevy input/render path; no physical mouse comfort or owner acceptance"}),
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
        // A newly placed bell is ready once its glTF scene spawned and plays the
        // loop. The loop itself never holds the table unsettled.
        let tokens = world.query::<&TerrorToken>().iter(world).count();
        let playing = world.query::<&TerrorPlayer>().iter(world).count();
        if playing < tokens {
            pending.push("Terror token spawn".into());
        }
        pending.extend(popover::unsettled(world));
        pending
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
    fn keys(&self) -> &[SeatKey] {
        &KEYS
    }
    fn focus(&self, world: &mut World) -> Focus {
        let ui = world.resource::<Interface>();
        let modal = transfer::shown(world.resource::<Play>().game().transfer.as_ref(), ui);
        let focused = if modal {
            ui.transfer.focus.clone()
        } else {
            ui.choice.focus.clone()
        };
        let Some(name) = focused else {
            return Focus::Nothing;
        };
        match controls(world).into_iter().find(|(_, c, _)| c.name == name) {
            Some((_, control, _)) => Focus::Control {
                choice: control.choice.then(|| control.name.clone()),
                label: control.name,
            },
            None => Focus::Other(json!({"stale focus": name})),
        }
    }
}
