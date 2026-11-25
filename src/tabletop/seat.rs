//! Development seat adapter: resolves real controls and never executes rules directly.
use super::*;
use bevy::{camera::primitives::Aabb, input::keyboard::Key, time::TimeUpdateStrategy};
use serde_json::json;
use std::sync::LazyLock;
use unfolded_tabletop::{
    capture,
    label_anchor::Anchor,
    seat::{
        self as shared, Focus, Game as SeatGame, Hit, Interrupt, Logical, Name as TargetName,
        Observed, SeatKey,
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
/// the BRIEFING ribbon and its tabs, or the transfer modal; Enter and Space activate
/// the focused control and can commit, so a request names that control (they skip
/// the Kidnap die in the air as `kidnap-die`, and close the landed die with Continue,
/// #74, #87); Escape unpins a
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
        key("Space", KeyCode::Space, Key::Space, true),
        key("Escape", KeyCode::Escape, Key::Escape, false),
        key("Backspace", KeyCode::Backspace, Key::Backspace, false),
        // Each hands a faction over to an agent, or takes it back (`faction_seats`).
        key("F1", KeyCode::F1, Key::F1, false),
        key("F2", KeyCode::F2, Key::F2, false),
        key("F3", KeyCode::F3, Key::F3, false),
        key("F4", KeyCode::F4, Key::F4, false),
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
/// A text's type size in logical window pixels, when it is set in pixels.
pub(super) fn type_size(font: &TextFont) -> Option<f32> {
    match font.font_size {
        FontSize::Px(px) => Some(px),
        _ => None,
    }
}
fn named(control: &Control) -> TargetName {
    if control.choice {
        TargetName::Choice(control.name.clone())
    } else {
        TargetName::Component(control.name.clone())
    }
}
/// A grid of points across a piece's body as the map camera shows it: the bounds of
/// its meshes on screen.
fn body_points(world: &mut World, piece: Entity) -> Vec<Vec2> {
    let Ok((camera, camera_at)) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .single(world)
        .map(|(camera, at)| (camera.clone(), *at))
    else {
        return vec![];
    };
    let meshes: Vec<(Entity, Aabb, GlobalTransform)> = world
        .query_filtered::<(Entity, &Aabb, &GlobalTransform), With<Mesh3d>>()
        .iter(world)
        .map(|(entity, bounds, at)| (entity, *bounds, *at))
        .collect();
    let mut seen: Option<Rect> = None;
    for (entity, bounds, at) in meshes {
        let mut current = Some(entity);
        let mut belongs = false;
        while let Some(e) = current {
            if e == piece {
                belongs = true;
                break;
            }
            current = world.get::<ChildOf>(e).map(ChildOf::parent);
        }
        if !belongs {
            continue;
        }
        for x in [-1., 1.] {
            for y in [-1., 1.] {
                for z in [-1., 1.] {
                    let corner = Vec3::from(bounds.center)
                        + Vec3::from(bounds.half_extents) * Vec3::new(x, y, z);
                    if let Ok(p) = camera.world_to_viewport(&camera_at, at.transform_point(corner))
                    {
                        seen = super::grow(seen, p);
                    }
                }
            }
        }
    }
    let Some(seen) = seen else {
        return vec![];
    };
    (1..8)
        .flat_map(|x| (1..8).map(move |y| Vec2::new(x as f32, y as f32) / 8.))
        .map(|at| seen.min + seen.size() * at)
        .collect()
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
            .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
            .single(world)
            && let Ok(point) = camera.world_to_viewport(at, position)
        {
            result.push((entity, c, point));
        }
    }
    {
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
                    .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
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
            .query_filtered::<&Camera, With<TableCamera>>()
            .single(world)
            .ok()
            .and_then(Camera::logical_viewport_rect)
            .map(|r| [r.min.x, r.min.y, r.max.x, r.max.y]);
        let status = world.query::<&status::Shown>().iter(world).next().cloned();
        // The space tag's drawn texts, the contribution and the open decision's line
        // (#170), with their laid-out bounds: a line that wrapped is twice as tall.
        let status_text = world
            .query_filtered::<(&emblems::Plain, &ComputedNode, &UiGlobalTransform, Has<status::Reason>), With<status::Tag>>()
            .iter(world)
            .map(|(text, node, at, reason)| json!({"text":text.0,"reason":reason,"bounds":logical_bounds(node, at)}))
            .collect::<Vec<_>>();
        let victory = world.query::<&victory::Shown>().iter(world).next().cloned();
        let victory_count = world.query::<&victory::Shown>().iter(world).count();
        let victory_text = world
            .query_filtered::<(&emblems::Plain, &ComputedNode, &UiGlobalTransform), With<victory::Tag>>()
            .iter(world)
            .map(|(text, node, at)| json!({"text":text.0,"bounds":logical_bounds(node, at)}))
            .collect::<Vec<_>>();
        let top = world.query::<&top::Shown>().iter(world).next().cloned();
        let map_targets = world.resource::<highlights::Targeting>().targets.clone();
        let mut highlights = world
            .query::<(&highlights::Decal, &Visibility)>()
            .iter(world)
            .filter(|(_, v)| **v != Visibility::Hidden)
            .map(|(d, _)| d.file.clone())
            .collect::<Vec<_>>();
        highlights.sort();
        let removal = world.resource::<highlights::Targeting>().removal.clone();
        // Each removal-mode marker under its piece: its group, graphic and the screen
        // point of its centre, in logical window pixels.
        let markers = {
            let camera = world
                .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
                .single(world)
                .ok()
                .map(|(c, at)| (c.clone(), *at));
            let mut markers = world
                .query::<(&highlights::Marker, &GlobalTransform)>()
                .iter(world)
                .map(|(m, at)| {
                    let point = camera
                        .as_ref()
                        .and_then(|(c, cat)| c.world_to_viewport(cat, at.translation()).ok())
                        .map(|p| [p.x, p.y]);
                    json!({"piece":m.piece,"file":m.file,"at":point})
                })
                .collect::<Vec<_>>();
            markers.sort_by_key(|m| m["piece"].to_string());
            markers
        };
        let popover = popover::observe(world);
        let eligibility = eligibility::observe(world);
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
        // Each count's type size and drawn bounds, so captures can be judged for
        // legibility at the owner's display (#50).
        let count_rows: Vec<_> = world
            .query_filtered::<(Entity, &Text, &TextFont, &ComputedNode, &UiGlobalTransform), With<Count>>()
            .iter(world)
            .map(|(entity, text, font, node, at)| {
                (
                    entity,
                    text.0.clone(),
                    type_size(font),
                    logical_bounds(node, at),
                )
            })
            .collect();
        let mut counts: Vec<_> = count_rows
            .into_iter()
            .map(|(entity, text, size, bounds)| {
                json!({"name":piece_name(world, entity),"text":text,"font_size":size,"bounds":bounds})
            })
            .collect();
        counts.sort_by_key(|count| count["name"].to_string());
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
        let die = die::observe(world);
        let card = cards::observe(world);
        let piles = piles::observe(world);
        let pile_tag = pile_tags::observe(world);
        let available_forces = available::observe(world);
        let marker_supply = supply::observe(world);
        let main_view = areas::observe(world);
        let slip = notice::observe(world);
        let save_quit = save_quit::observe(world);
        let focus_mark = focus_mark::observe(world);
        let seats = faction_seats::observe(world);
        let play = world.resource::<Play>();
        let ui = world.resource::<Interface>();
        let pose = world.resource::<Orbit>().pose();
        let water_frame = world.get_resource::<Water>().map(|water| water.frame);
        // One more key would pass the macro's recursion limit, so the piles join after.
        let mut table = json!({"controls":controls,"surfaces":surfaces,"standings":standings,"map_viewport":map_viewport,"viewport":size.map(|s| [s.x,s.y]),"status":status,"victory":victory,"victory_count":victory_count,"victory_text":victory_text,"top":top,"briefing":briefing,"choice":choice::observe(ui),"choice_parts":choice_parts,"eligibility":eligibility,"map_targets":map_targets,"highlights":highlights,"removal":removal,"markers":markers,"popover":popover,"transfer":transfer,"top_text":top_text,"inline_text":inline_text,"counts":counts,"top_parts":top_parts,"water_frame":water_frame,"terror":terror,"die":die,"settled":pending.is_empty(),"pending":pending,"card":card,"main_view":main_view,"hover":ui.hover.visible(),"notice":ui.notice.text,"error":play.error,"camera":{"yaw":pose.yaw,"pitch":pose.pitch,"distance":pose.distance,"target":[pose.target.x,pose.target.y,pose.target.z]},"native_limits":"windowless real Bevy input/render path; no physical mouse comfort or owner acceptance"});
        table["status_text"] = json!(status_text);
        table["piles"] = piles;
        table["pile_tag"] = pile_tag;
        table["available_forces"] = available_forces;
        table["marker_supply"] = marker_supply;
        // #174: the notice slip as drawn, and Save · quit's plate.
        table["slip"] = slip;
        table["save_quit"] = save_quit;
        table["focus_mark"] = focus_mark;
        table["faction_seats"] = seats;
        Observed {
            observation: serde_json::to_value(play.game().observe()).unwrap(),
            table,
        }
    }
    fn revision(&self, world: &mut World) -> u64 {
        world.resource::<Play>().game().observe().revision
    }
    fn resolve(&self, world: &mut World, name: &TargetName) -> Result<Vec<Vec2>, String> {
        let mut found: Vec<_> = controls(world)
            .into_iter()
            .filter(|(_, c, _)| &named(c) == name)
            .collect();
        // A pile is its count disc, its side and its caption: the seat rests on the
        // disc first and on the others' middles after it.
        if found.len() > 1
            && found
                .iter()
                .all(|(_, c, _)| matches!(c.action, Action::Pile(_)))
        {
            found.sort_by_key(|(entity, _, _)| world.get::<piles::Disc>(*entity).is_none());
            let mut points: Vec<Vec2> = found.iter().map(|(_, _, at)| *at).collect();
            points.dedup();
            return Ok(points);
        }
        if found.len() != 1 {
            return Err(format!(
                "Expected exactly one shown target {name:?}, found {}",
                found.len()
            ));
        }
        let mut points = vec![found[0].2];
        // An area's own controls can cover its middle: its edges and corners are
        // bare. The shared seat still verifies the actual picked name.
        if found[0].1.name.starts_with("area:")
            && let Some(node) = world.get::<ComputedNode>(found[0].0)
        {
            points = areas::bare_points(found[0].2, node.size() * node.inverse_scale_factor);
        }
        // A count can cover the center of its solid piece. Try exposed body
        // points too; the shared seat still verifies the actual picked name.
        if found[0].1.name.starts_with("piece:") {
            for x in [-6., -3., 0., 3., 6.] {
                for y in [-6., -3., 0., 3., 6.] {
                    points.push(found[0].2 + Vec2::new(x, y));
                }
            }
            // A piece is as large on screen as the camera makes it, and its count may
            // cover all of those: try points across its body too (#142).
            points.extend(body_points(world, found[0].0));
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
        let mut pending: Vec<String> = settled::unsettled(world)
            .into_iter()
            .map(settled::Unsettled::words)
            .collect();
        if faction_seats::due(world) {
            pending.push("faction seats".into());
        }
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
    /// The Kidnap die's skip, a click on its tray, Enter or Space, reaches the table
    /// while the die is in the air and holds it unsettled. It only ends the tumble: the roll was
    /// saved before the die moved (#74, #87). A card advance is landed the same way
    /// while a card is in the air: by Enter or Space, or by an activation of any shown
    /// component, the owner's click, which then does nothing else (#145). An activation
    /// naming a choice is refused as busy.
    fn interrupts(&self, world: &mut World, input: Interrupt<'_>) -> bool {
        let (skips, lands) = match input {
            // The owner's click lands a card in the air wherever it falls, and does
            // nothing else there (#145, #137's review); a choice named by its id asks
            // for a decision instead, and waits.
            Interrupt::Activate(TargetName::Component(name)) => (name == die::TRAY, true),
            Interrupt::Activate(TargetName::Choice(_)) => (false, false),
            // A faction is handed over and taken back at any time (#203).
            Interrupt::Key(key) if faction_seats::KEY_NAMES.contains(&key) => return true,
            Interrupt::Key(key) => {
                let key = matches!(key, "Enter" | "Space");
                (key, key)
            }
        };
        (skips && die::focus(world) == Some(die::TRAY)) || (lands && piles::playing(world))
    }
    fn focus(&self, world: &mut World) -> Focus {
        // A card in the air holds the keyboard: Enter or Space lands it (#145).
        if piles::playing(world) {
            return Focus::Control {
                choice: None,
                label: piles::ADVANCE.into(),
            };
        }
        // A presented Kidnap die holds the keyboard: Enter reaches the tray's skip while
        // the die is in the air, and Continue once it has landed (#74, #87).
        if die::shown(world) {
            return die::focus(world).map_or(Focus::Nothing, |label| Focus::Control {
                choice: None,
                label: label.into(),
            });
        }
        let ui = world.resource::<Interface>();
        let modal = transfer::shown(world.resource::<Play>().game().transfer.as_ref());
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// One seat request. A table that has exited fails the test here, with its reason,
    /// rather than answering every later request as if it were still being played.
    fn ask(seat: &mut shared::Seat<Table>, app: &mut App, request: Value) -> Value {
        seat.respond(app, &request.to_string())
            .unwrap_or_else(|exited| panic!("{request}: {exited}"))
    }

    /// An admitted table of `size` in process, on a save of a known die when there is
    /// none yet, as `tests/native.rs` launches.
    fn table(
        cards: &std::path::Path,
        save: &std::path::Path,
        size: UVec2,
    ) -> (App, shared::Seat<Table>) {
        if !save.exists() {
            use crate::save::Session;
            drop(Session::create_seeded(save, Session::KIDNAP_ROLLS_1).unwrap());
        }
        let mut app = build(
            Options {
                art: "art/atlas".into(),
                cards: cards.into(),
                save: save.into(),
                recover: None,
                socket: None,
                players: None,
                agent: true,
                seat_size: size,
            },
            true,
        )
        .unwrap();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        capture::install::<Camera2d>(&mut app, size);
        let mut seat = shared::Seat::open(Table, &mut app).unwrap();
        seat.admit(&mut app, Duration::from_secs(180)).unwrap();
        (app, seat)
    }

    /// Such a table played to a rolled Kidnap die that is still in the air, at revision 3.
    fn rolled(
        cards: &std::path::Path,
        save: &std::path::Path,
        size: UVec2,
    ) -> (App, shared::Seat<Table>) {
        let (mut app, mut seat) = table(cards, save, size);
        // AC-B (#69): Terror is pinned before its commit submits it.
        let pinned = ask(
            &mut seat,
            &mut app,
            json!({"op":"activate","revision":0,"component":"option:terror"}),
        );
        assert_eq!(pinned["status"], "activated", "{pinned}");
        ask(&mut seat, &mut app, json!({"op":"settle"}));
        for (revision, choice) in [(0, "0:terror"), (1, "1:kidnap"), (2, "2:kidnap:roll")] {
            // EX-A (#84): a Terror step is pinned by its row before its commit.
            if choice == "1:kidnap" {
                let step = ask(
                    &mut seat,
                    &mut app,
                    json!({"op":"activate","revision":revision,"component":"step:kidnap"}),
                );
                assert_eq!(step["status"], "activated", "{step}");
                ask(&mut seat, &mut app, json!({"op":"settle"}));
            }
            let done = ask(
                &mut seat,
                &mut app,
                json!({"op":"activate","revision":revision,"choice":choice}),
            );
            assert_eq!(done["status"], "activated", "{done}");
            if revision < 2 {
                ask(&mut seat, &mut app, json!({"op":"settle"}));
            }
        }
        (app, seat)
    }

    /// Every count's name and bounds (left, top, right, bottom), as the seat reports them.
    fn count_bounds(view: &Value) -> Vec<(String, [f64; 4])> {
        let counts = view["table"]["counts"].as_array().unwrap();
        counts
            .iter()
            .map(|c| {
                let bounds = serde_json::from_value(c["bounds"].clone()).unwrap();
                (c["name"].as_str().unwrap().to_owned(), bounds)
            })
            .collect()
    }

    /// Every count in `is` stands within 2 px of where it stands in `at`.
    fn assert_counts_at(is: &[(String, [f64; 4])], at: &[(String, [f64; 4])], when: &str) {
        assert!(!is.is_empty(), "no counts {when}");
        assert_eq!(is.len(), at.len(), "{when}");
        for ((name, is), (expected_name, at)) in is.iter().zip(at) {
            assert_eq!(name, expected_name, "{when}");
            assert!(
                is.iter().zip(at).all(|(a, b)| (a - b).abs() <= 2.),
                "{name} {when}: {is:?} against {at:?}"
            );
        }
    }

    /// #177: the piece counts follow a window resize while the Kidnap die is shown, in the
    /// air and landed. The seat has no resize request, so this resizes the seat's surface
    /// in process. The map is framed anew for the size, so each size's counts must match
    /// those a table opened at that size shows before the die, and those after Continue.
    #[test]
    #[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
    fn native_counts_follow_a_resize_while_the_die_is_shown() {
        let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into();
        let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| "local/verification/issue-177".into());
        std::fs::create_dir_all(&evidence).unwrap();
        let root = tempfile::Builder::new()
            .prefix("resize-")
            .tempdir_in(evidence)
            .unwrap()
            .keep()
            .canonicalize()
            .unwrap();
        eprintln!("Resize while the die is shown evidence: {}", root.display());
        let sizes = [
            UVec2::new(1920, 1080),
            UVec2::new(1440, 960),
            UVec2::new(2560, 1080),
        ];
        // Where a table opened at each size shows its counts, with no die.
        let opened: Vec<_> = sizes
            .iter()
            .map(|size| {
                let save = root.join(format!("{}x{}.json", size.x, size.y));
                let (mut app, mut seat) = table(&cards, &save, *size);
                ask(&mut seat, &mut app, json!({"op":"settle"}));
                count_bounds(&ask(&mut seat, &mut app, json!({"op":"observe"})))
            })
            .collect();
        // Every two sizes put every count more than 20 px apart, so a count left where
        // another size had it cannot pass.
        for (i, one) in opened.iter().enumerate() {
            for other in &opened[i + 1..] {
                for ((name, a), (_, b)) in one.iter().zip(other) {
                    let apart = (a[0] - b[0]).hypot(a[1] - b[1]);
                    assert!(apart > 20., "{name}: {a:?} {b:?}");
                }
            }
        }
        let save = root.join("resized.json");
        let (mut app, mut seat) = rolled(&cards, &save, sizes[0]);
        let air = ask(&mut seat, &mut app, json!({"op":"settle","steps":3}));
        assert_eq!(air["status"], "unsettled", "{air}");

        // In the air: to 1440x960. The steps are too few for the die to land.
        resize(&mut app, sizes[1]);
        let air = ask(&mut seat, &mut app, json!({"op":"settle","steps":8}));
        assert_eq!(air["status"], "unsettled", "{air}");
        let air = ask(&mut seat, &mut app, json!({"op":"observe"}));
        assert_eq!(air["table"]["die"]["landed"], false, "{air}");
        assert_counts_at(&count_bounds(&air), &opened[1], "resized in the air");

        // Landed: to 2560x1080.
        ask(&mut seat, &mut app, json!({"op":"settle"}));
        resize(&mut app, sizes[2]);
        // The table was settled, so it is stepped to see the new size before it settles.
        let stepped = ask(&mut seat, &mut app, json!({"op":"advance","steps":8}));
        assert_eq!(stepped["status"], "advanced", "{stepped}");
        ask(&mut seat, &mut app, json!({"op":"settle"}));
        let landed = ask(&mut seat, &mut app, json!({"op":"observe"}));
        assert_eq!(landed["table"]["die"]["landed"], true, "{landed}");
        let at_landing = count_bounds(&landed);
        assert_counts_at(&at_landing, &opened[2], "resized after landing");
        ask(
            &mut seat,
            &mut app,
            json!({"op":"capture","path":root.join("resized-landed.png")}),
        );

        // Continue moves no count.
        let done = ask(
            &mut seat,
            &mut app,
            json!({"op":"activate","revision":3,"component":"kidnap-die:continue"}),
        );
        assert_eq!(done["status"], "activated", "{done}");
        ask(&mut seat, &mut app, json!({"op":"settle"}));
        let continued = ask(&mut seat, &mut app, json!({"op":"observe"}));
        assert!(continued["table"]["die"].is_null(), "{continued}");
        assert_counts_at(&count_bounds(&continued), &at_landing, "after Continue");
    }

    /// #190: once the table and its die are drawn, the app holds no pixels of the images
    /// it only draws: every file the table loads, and every texture a drawn material
    /// uses, the models' included. Their sizes stay, for the interface's layout.
    #[test]
    #[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
    fn native_the_table_keeps_no_copy_of_an_image_it_only_draws() {
        let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into();
        let saves = tempfile::tempdir().unwrap();
        let save = saves.path().join("task.json");
        let (mut app, mut seat) = rolled(&cards, &save, UVec2::new(1920, 1280));
        ask(&mut seat, &mut app, json!({"op":"settle"}));
        let files: Vec<_> = app
            .world()
            .resource::<Art>()
            .images
            .values()
            .cloned()
            .collect();
        let materials: Vec<_> = app
            .world_mut()
            .query::<&MeshMaterial3d<StandardMaterial>>()
            .iter(app.world())
            .map(|m| m.id())
            .collect();
        let assets = app.world().resource::<Assets<StandardMaterial>>();
        let textures: Vec<_> = materials
            .into_iter()
            .flat_map(|id| {
                let m = assets.get(id).unwrap();
                [
                    &m.base_color_texture,
                    &m.emissive_texture,
                    &m.metallic_roughness_texture,
                    &m.normal_map_texture,
                    &m.occlusion_texture,
                ]
            })
            .flatten()
            .cloned()
            .collect();
        // The models' own textures, so a search that missed them fails.
        let models: std::collections::BTreeSet<_> = textures
            .iter()
            .filter_map(Handle::path)
            .filter(|path| path.path().extension().is_some_and(|e| e == "glb"))
            .map(ToString::to_string)
            .collect();
        let expected = [
            "atlas-kidnap-die.glb#Texture0",
            "atlas-terrain-map.glb#Texture0",
            "atlas-terrain-map.glb#Texture1",
            "atlas-terrain-map.glb#Texture2",
        ];
        assert!(models.iter().eq(expected), "{models:?}");
        let art = app.world().resource::<Art>();
        for key in [assets::CARD_BACK.name, "water/water-c-00023.png"] {
            assert!(art.images.contains_key(key), "{key}");
        }
        let images = app.world().resource::<Assets<Image>>();
        for handle in files.iter().chain(&textures) {
            let image = images.get(handle).unwrap();
            assert!(image.size().min_element() > 0, "{handle:?}");
            assert!(image.data.is_none(), "{handle:?} keeps its pixels");
        }
    }

    /// Resizes the in-process surface as a window resize does: the primary window and the
    /// image the cameras render.
    fn resize(app: &mut App, size: UVec2) {
        let mut windows = app.world_mut().query::<&mut Window>();
        let mut window = windows.single_mut(app.world_mut()).unwrap();
        window.resolution.set(size.x as f32, size.y as f32);
        let image = app.world().resource::<capture::Surface>().image();
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .get_mut(&image)
            .unwrap()
            .resize(bevy::render::render_resource::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            });
    }

    /// The Kidnap die's skip through the seat's own requests (#87). While the die is in
    /// the air, Enter or Space naming the tray's skip, or an activation of the tray,
    /// lands it at once on the committed face, and the save is unchanged. Tab is still
    /// refused as busy, and Enter naming Continue is refused, since Enter reaches the
    /// skip then.
    #[test]
    #[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
    fn native_seat_skips_the_die_by_enter_space_or_a_click_on_the_tray() {
        let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into();
        let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| "local/verification/issue-87".into());
        std::fs::create_dir_all(&evidence).unwrap();
        let root = tempfile::Builder::new()
            .prefix("skip-")
            .tempdir_in(evidence)
            .unwrap()
            .keep()
            .canonicalize()
            .unwrap();
        eprintln!("Kidnap die skip evidence: {}", root.display());
        for by in ["enter", "space", "click"] {
            let size = UVec2::new(1920, 1080);
            let save = root.join(format!("{by}.json"));
            let (mut app, mut seat) = rolled(&cards, &save, size);
            let mut ask = |app: &mut App, request: Value| ask(&mut seat, app, request);
            let bytes = std::fs::read(&save).unwrap();
            let air = ask(&mut app, json!({"op":"settle","steps":3}));
            assert_eq!(air["status"], "unsettled", "{air}");
            let air = ask(&mut app, json!({"op":"observe"}));
            let die = &air["table"]["die"];
            assert_eq!(die["landed"], false, "{die}");
            assert!(die["seek"].as_f64().unwrap() < die["landing"].as_f64().unwrap());
            // Anything but the skip is still refused while the die is in the air,
            // and Enter reaches the skip then, not Continue.
            let tab = ask(&mut app, json!({"op":"key","revision":3,"key":"Tab"}));
            assert_eq!(tab["status"], "refused", "{tab}");
            assert!(
                tab["error"]
                    .as_str()
                    .unwrap()
                    .starts_with("The table is busy ("),
                "{tab}"
            );
            let continued = ask(
                &mut app,
                json!({"op":"key","revision":3,"key":"Enter","focused":"kidnap-die:continue"}),
            );
            assert_eq!(continued["status"], "refused", "{continued}");
            assert_eq!(
                continued["error"],
                "Enter names \"kidnap-die:continue\", but focus is on \"kidnap-die\", which has no choice identity. Nothing was pressed."
            );
            ask(
                &mut app,
                json!({"op":"capture","path":root.join(format!("{by}-air.png"))}),
            );
            let (skip, status) = if by == "enter" || by == "space" {
                let key = if by == "enter" { "Enter" } else { "Space" };
                (
                    json!({"op":"key","revision":3,"key":key,"focused":"kidnap-die"}),
                    "pressed",
                )
            } else {
                (
                    json!({"op":"activate","revision":3,"component":"kidnap-die"}),
                    "activated",
                )
            };
            let skipped = ask(&mut app, skip);
            assert_eq!(skipped["status"], status, "{by}: {skipped}");
            assert_eq!(skipped["revision"], 3, "{by}: a skip acts on no decision");
            ask(&mut app, json!({"op":"settle"}));
            let landed = ask(&mut app, json!({"op":"observe"}));
            let die = &landed["table"]["die"];
            assert_eq!(die["landed"], true, "{by}: {die}");
            assert_eq!(die["seek"], die["duration"], "{by}: skipped to the end");
            assert_eq!(die["face"], landed["observation"]["kidnap"]["die"]);
            // #67's legibility floor at this very size (`die::FACE_FLOOR`, the study's
            // "about 70 px" made explicit as 69.5 px).
            let face_px = die["top_face"].as_f64().unwrap();
            eprintln!("{by}: landed top face {face_px:.1} px at 1920x1080");
            assert!(
                face_px >= f64::from(die::FACE_FLOOR),
                "{by}: landed face {face_px} px at 1920x1080"
            );
            assert_eq!(landed["table"]["settled"], true, "{landed}");
            assert_eq!(std::fs::read(&save).unwrap(), bytes, "skip saves nothing");
            ask(
                &mut app,
                json!({"op":"capture","path":root.join(format!("{by}-skipped.png"))}),
            );
        }
    }

    /// The outer edges of the card's 1 px ink stroke as drawn in `capture`, found by
    /// scanning inward from 20 px outside each edge of the slip's reported `bounds`
    /// (left, top, right, bottom): across the bottom padding for the sides, and down
    /// and up a column inside the left padding for the top and the foot. A pixel is
    /// the stroke's when its channels sum below 560: the sea sums about 674, the paper 724
    /// and the shadow at least 600.
    pub(crate) fn drawn_stroke(capture: &std::path::Path, bounds: [f64; 4]) -> [f64; 4] {
        let image = image::open(capture).unwrap().to_rgb8();
        let dark = |x: f64, y: f64| {
            let p = image.get_pixel(x as u32, y as u32);
            p.0.iter().map(|&c| u32::from(c)).sum::<u32>() < 560
        };
        let [left, top, right, foot] = bounds.map(f64::round);
        let row = foot - 4.;
        let column = left + 8.;
        let first = |mut at: f64, step: f64, end: f64, hit: &dyn Fn(f64) -> bool| loop {
            if hit(at) {
                return at;
            }
            assert!((at - end).abs() > 0.5, "no stroke between {} and {end}", at);
            at += step;
        };
        [
            first(left - 20., 1., left + 8., &|x| dark(x, row)),
            first(top - 20., 1., top + 8., &|y| dark(column, y)),
            // The last dark pixel's far side is the edge.
            first(right + 20., -1., right - 8., &|x| dark(x, row)) + 1.,
            first(foot + 20., -1., foot - 8., &|y| dark(column, y)) + 1.,
        ]
    }

    /// Every file in `dir`, by name, with its bytes.
    fn files(dir: &std::path::Path) -> Vec<(std::ffi::OsString, Vec<u8>)> {
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), std::fs::read(entry.path()).unwrap())
            })
            .collect();
        files.sort();
        files
    }

    /// #174 (spec #166's review, P2): before the match opens, the slip shows the loading
    /// state, and a save an earlier build wrote is refused on the slip as a launch error,
    /// in its words, with every file left as it was. The seat admits no refused table, so
    /// this sits at the table in process, as the die's regression does, and reads the
    /// slip before admission and after the refusal. At the owner's display, 3840x2160.
    #[test]
    #[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
    fn native_the_slip_shows_loading_then_an_earlier_build_s_save_refused() {
        use unfolded_record_replacement::save as record;
        let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into();
        let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| "local/verification/issue-174".into());
        std::fs::create_dir_all(&evidence).unwrap();
        let root = tempfile::Builder::new()
            .prefix("launch-error-")
            .tempdir_in(evidence)
            .unwrap()
            .keep()
            .canonicalize()
            .unwrap();
        eprintln!("Launch error evidence: {}", root.display());
        // A save of the earlier format "2": this build's opening save, resealed with
        // that format, so only the format tells it from one this build opens.
        let saves = root.join("saves");
        std::fs::create_dir_all(&saves).unwrap();
        let save = saves.join("earlier.json");
        drop(crate::save::Session::open(&save, None).unwrap());
        let mut written: Value = serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        written["identity"]["format"] = "2".into();
        let identity: record::Identity =
            serde_json::from_value(written["identity"].clone()).unwrap();
        std::fs::write(&save, record::seal(&identity, &written["payload"])).unwrap();
        let before = files(&saves);

        let size = UVec2::new(3840, 2160);
        let mut app = build(
            Options {
                art: "art/atlas".into(),
                cards,
                save: save.clone(),
                recover: None,
                socket: None,
                players: None,
                agent: true,
                seat_size: size,
            },
            true,
        )
        .unwrap();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        capture::install::<Camera2d>(&mut app, size);
        let mut seat = shared::Seat::open(Table, &mut app).unwrap();
        let ask = |seat: &mut shared::Seat<Table>, app: &mut App, request: Value| -> Value {
            seat.respond(app, &request.to_string())
                .unwrap_or_else(|exited| panic!("{request}: {exited}"))
        };

        // The slip stands where the main view's record puts it: its width, right edge
        // and foot; it grows upward with its lines.
        let layout: Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let tolerance = layout["tolerance"].as_f64().unwrap();
        let [rx, ry, rw, rh]: [f64; 4] =
            serde_json::from_value(layout["notice-slip"]["bounds"].clone()).unwrap();
        let placed = |slip: &Value| {
            // The seat reports the slip's edges: left, top, right, bottom.
            let [left, _, right, foot]: [f64; 4] =
                serde_json::from_value(slip["bounds"].clone()).unwrap();
            assert!((right - (rx + rw)).abs() <= tolerance, "{slip}");
            assert!((foot - (ry + rh)).abs() <= tolerance, "{slip}");
            assert!((right - left - rw).abs() <= tolerance, "{slip}");
        };
        // The card is drawn on that place: its stroke's outer edges lie on the reported
        // bounds, and its right edge on Save · quit's.
        let drawn_on_its_place = |capture: &std::path::Path, view: &Value| {
            let slip = &view["table"]["slip"];
            let bounds: [f64; 4] = serde_json::from_value(slip["bounds"].clone()).unwrap();
            let stroke = drawn_stroke(capture, bounds);
            eprintln!(
                "{}: stroke {stroke:?} on bounds {bounds:?}",
                capture.display()
            );
            for edge in 0..4 {
                assert!(
                    (stroke[edge] - bounds[edge]).abs() <= tolerance,
                    "{}: the drawn stroke {stroke:?} leaves the slip's bounds {bounds:?}",
                    capture.display()
                );
            }
            let plate: [f64; 4] = serde_json::from_value(
                view["table"]["main_view"]["parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|part| part["part"] == "save-quit")
                    .expect("the main view draws Save · quit")["bounds"]
                    .clone(),
            )
            .unwrap();
            assert!(
                (stroke[2] - plate[2]).abs() <= tolerance,
                "{}: the slip's right edge {} is not Save · quit's {}",
                capture.display(),
                stroke[2],
                plate[2]
            );
        };

        // Before admission: no match is open, and the slip shows the loading state.
        let loading = ask(&mut seat, &mut app, json!({"op":"observe"}));
        std::fs::write(root.join("loading.json"), loading.to_string()).unwrap();
        let slip = &loading["table"]["slip"];
        assert_eq!(slip["shown"], true, "{slip}");
        assert_eq!(slip["kind"], "loading", "{slip}");
        assert_eq!(slip["text"], notice::LOADING, "{slip}");
        assert!(loading["table"]["error"].is_null());
        placed(slip);
        ask(
            &mut seat,
            &mut app,
            json!({"op":"capture","path":root.join("loading.png")}),
        );
        drawn_on_its_place(&root.join("loading.png"), &loading);

        // Admission refuses the save, in the words the table then shows.
        let refusal = match seat.admit(&mut app, Duration::from_secs(180)) {
            Err(shared::Unadmitted::Refused(reason)) => reason,
            Err(shared::Unadmitted::Unfinished) => panic!("admission ran out of time"),
            Ok(()) => panic!("a save of format 2 was admitted"),
        };
        for words in [
            "Another build wrote this save",
            "Every file was left unchanged",
            "Start a match under an unused save name",
        ] {
            assert!(refusal.contains(words), "{refusal}");
        }
        ask(&mut seat, &mut app, json!({"op":"advance","steps":5}));
        let refused = ask(&mut seat, &mut app, json!({"op":"observe"}));
        std::fs::write(root.join("refused.json"), refused.to_string()).unwrap();
        let slip = &refused["table"]["slip"];
        assert_eq!(slip["shown"], true, "{slip}");
        assert_eq!(slip["kind"], "error", "{slip}");
        assert_eq!(slip["text"], refusal.as_str(), "{slip}");
        assert_eq!(refused["table"]["error"], refusal.as_str());
        placed(slip);
        ask(
            &mut seat,
            &mut app,
            json!({"op":"capture","path":root.join("refused.png")}),
        );
        drawn_on_its_place(&root.join("refused.png"), &refused);
        assert_eq!(files(&saves), before, "the refused launch changed a file");
    }
}
