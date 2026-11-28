//! Exercise the desktop input path while the presentation clock is moving.
use super::*;
use bevy::picking::pointer::PointerButton;
use std::path::Path;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan to render a prepared Directorio recovery flip"]
fn native_prepared_paid_rally_recovery_turns_an_active_guerrilla() {
    use crate::{Group, State, game::CAMAGUEY_RALLY_ACTIONS};
    use std::fs;

    const SPACE: &str = "camaguey-province";
    const FLIP: &str = "paid-rally:camaguey-province:flip:1";
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-283".into());
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("prepared-recovery-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let save = root.join("task.json");
    let mut played = Session::create_seeded(&save, Session::KIDNAP_ROLLS_1).unwrap();
    for action in CAMAGUEY_RALLY_ACTIONS {
        let seen = played.observe();
        played
            .submit(
                seen.owner,
                seen.revision,
                &format!("{}:{action}", seen.revision),
            )
            .unwrap();
    }

    // Prepare a face-up Guerrilla in a copy of the played position. No supported
    // earlier card in this build exposes Directorio's Guerrillas.
    let mut prepared = played.game().clone();
    let space = prepared.spaces.iter_mut().find(|s| s.id == SPACE).unwrap();
    let underground = space
        .pieces
        .iter_mut()
        .find(|p| {
            p.faction == Faction::Directorio
                && p.kind == Kind::Guerrilla
                && p.state == State::Underground
        })
        .unwrap();
    assert_eq!(underground.count, 2);
    underground.count -= 1;
    let active = Group {
        count: 1,
        state: State::Active,
        ..*underground
    };
    space.pieces.push(active);
    let before = prepared.observe();
    let preview = popover::content(&before, SPACE, Some(FLIP)).unwrap();
    let preview = serde_json::to_value(preview).unwrap();
    assert_eq!(
        preview["rally"]["preview"]["recovery_faces"],
        json!({
            "active": [1, 0], "underground": [1, 2]
        })
    );
    assert_eq!(preview["rally"]["preview"]["resources"], json!([5, 4]));
    let choice = format!("{}:{FLIP}", before.revision);
    prepared
        .submit(Faction::Directorio, before.revision, &choice)
        .unwrap();
    let after = prepared.observe();
    let changes = piece_flips::committed(&before, &after, &choice);
    assert_eq!(changes.len(), 1);

    // The ordinary save reaches the same board with the legal zero-flip recovery.
    // Show the prepared transition on that settled board through the real scene
    // systems. Never put a prepared position in a save or bypass its replay.
    played
        .submit(
            Faction::Directorio,
            before.revision,
            &format!("{}:paid-rally:{SPACE}:flip:0", before.revision),
        )
        .unwrap();
    let settled = played.observe();
    assert_eq!(settled.spaces, after.spaces);
    assert_eq!(settled.available, after.available);
    assert_eq!(settled.resources, after.resources);
    drop(played);
    let saved = fs::read(&save).unwrap();

    let cards = PathBuf::from(std::env::var_os("CUBA_TEST_CARDS").expect("Set CUBA_TEST_CARDS"));
    let (mut app, mut seat) = table(&cards, &save, UVec2::new(3840, 2160));
    app.world_mut().resource_mut::<Interface>().piece_flips = changes;
    let start = ask(&mut seat, &mut app, json!({"op":"advance","steps":2}));
    let moving = &start["table"]["piece_flips"][0];
    assert_eq!(moving["flip"]["from"], "Active");
    assert_eq!(moving["flip"]["to"], "Underground");
    assert_eq!(moving["face_mark"], "Active");
    let turning = ask(&mut seat, &mut app, json!({"op":"advance","steps":40}));
    let turning = &turning["table"]["piece_flips"][0];
    assert_ne!(moving["center"], turning["center"]);
    assert_ne!(moving["up"], turning["up"]);
    ask(
        &mut seat,
        &mut app,
        json!({"op":"capture","path":root.join("active-recovery-turn.png")}),
    );
    let landed = ask(&mut seat, &mut app, json!({"op":"advance","steps":40}));
    assert_eq!(landed["table"]["piece_flips"], json!([]));
    assert!(
        landed["table"]["counts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| {
                row["name"] == "count:camaguey-province Directorio Guerrilla Underground"
                    && row["text"] == "2"
            })
    );
    assert_eq!(fs::read(&save).unwrap(), saved);
}

fn prepare(save: &Path, actions: &[&str]) -> App {
    let mut session = Session::create_seeded(save, 2).unwrap();
    for action in actions {
        let view = session.observe();
        session
            .submit(
                view.owner,
                view.revision,
                &format!("{}:{action}", view.revision),
            )
            .unwrap_or_else(|error| panic!("prepare {action}: {error:?}"));
    }
    drop(session);
    let cards = PathBuf::from(std::env::var_os("CUBA_TEST_CARDS").expect("Set CUBA_TEST_CARDS"));
    let (mut app, _) = table(&cards, save, UVec2::new(3840, 2160));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    for _ in 0..4 {
        app.update();
    }
    app
}

fn frame(app: &mut App, center: Vec3, distance: f32) {
    app.world_mut()
        .resource_mut::<Orbit>()
        .focus(Framing::new(center, distance));
    for _ in 0..4 {
        app.update();
    }
}

fn slot(app: &App, space: &str, group: crate::Group) -> Vec3 {
    let game = app.world().resource::<Play>().game();
    let space = game.spaces.iter().find(|row| row.id == space).unwrap();
    let [x, y] = map::slot_for(space, &group);
    map::at(
        x,
        y,
        map::top(space.terrain) + group_half_height(group.kind) * map::PIECE_SCALE,
    )
}

fn project(app: &mut App, at: Vec3) -> Vec2 {
    let world = app.world_mut();
    let (camera, camera_at) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .single(world)
        .unwrap();
    camera.world_to_viewport(camera_at, at).unwrap()
}

fn mouse(app: &mut App, position: Vec2, action: PointerAction) {
    pointer(app, position, action);
    app.update();
}

fn press(app: &mut App, position: Vec2) {
    mouse(app, position, PointerAction::Move { delta: Vec2::ZERO });
    mouse(app, position, PointerAction::Press(PointerButton::Primary));
}

fn release(app: &mut App, position: Vec2) {
    mouse(
        app,
        position,
        PointerAction::Release(PointerButton::Primary),
    );
    app.update();
}

fn click_at(app: &mut App, position: Vec2) {
    press(app, position);
    release(app, position);
}

fn click_control(app: &mut App, name: &str) {
    let (_, _, at) = controls(app.world_mut())
        .into_iter()
        .find(|(_, control, _)| control.name == name)
        .unwrap_or_else(|| panic!("no control {name}"));
    click_at(app, at);
}

fn revision(app: &App) -> u64 {
    app.world().resource::<Play>().game().revision
}

fn popover(app: &mut App) -> Value {
    Table.observe(app.world_mut()).table["popover"].clone()
}

fn still_flying(app: &App) -> bool {
    app.world().resource::<piece_flips::Flights>().moving()
}

fn landing(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    for _ in 0..12 {
        app.update();
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    assert!(!still_flying(app), "the flip has landed");
}

fn piece_entity(app: &mut App, key: &str) -> Option<Entity> {
    let world = app.world_mut();
    world
        .query::<(Entity, &OwnPiece)>()
        .iter(world)
        .find_map(|(entity, piece)| (piece.key == key).then_some(entity))
}

fn roundel_entity(app: &mut App, token: Entity) -> Option<Entity> {
    let world = app.world_mut();
    world
        .query_filtered::<(Entity, &LabelAnchor), With<Count>>()
        .iter(world)
        .find_map(|(entity, anchor)| {
            matches!(anchor.to, Anchor::Entity { entity, .. } if entity == token).then_some(entity)
        })
}

fn exposed_point(app: &mut App, entity: Entity, center: Vec2) -> Vec2 {
    let candidates = std::iter::once(center).chain(body_points(app.world_mut(), entity));
    for at in candidates {
        mouse(app, at, PointerAction::Move { delta: Vec2::ZERO });
        if app.world().resource::<HoverMap>()[&PointerId::Mouse].contains_key(&entity) {
            return at;
        }
    }
    let world = app.world();
    let hits = world.resource::<HoverMap>()[&PointerId::Mouse]
        .keys()
        .map(|hit| {
            (
                *hit,
                world.get::<Name>(*hit),
                world.get::<Control>(*hit).map(|c| &c.name),
            )
        })
        .collect::<Vec<_>>();
    panic!("piece {entity} at {center} is covered; last ray hit {hits:?}");
}

fn removal_during_terror(save: &Path) -> App {
    let mut app = prepare(save, &["terror", "transfer", "ask:2:2", "authorize:2"]);
    let label = app
        .world()
        .resource::<Art>()
        .labels
        .iter()
        .find(|label| label.space == "santiago")
        .unwrap();
    let [x, y, w, h] = map::label_bounds(label);
    frame(&mut app, map::at(x + w / 2., y + h / 2., 0.), 5.);
    click_control(&mut app, "space:santiago");
    click_control(&mut app, "popover:commit");
    assert_eq!(revision(&app), 5);
    assert!(still_flying(&app));
    click_control(&mut app, "step:remove");
    click_control(&mut app, "5:remove");
    assert_eq!(revision(&app), 6);
    assert!(still_flying(&app));
    app
}

fn guerrilla(state: State) -> crate::Group {
    crate::Group {
        faction: Faction::M26,
        kind: Kind::Guerrilla,
        state,
        count: 1,
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_motion_leaves_the_new_active_target_pickable_during_terror() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = removal_during_terror(&temp.path().join("task.json"));
    let center = slot(&app, "santiago", guerrilla(State::Active));
    frame(&mut app, center, 5.);
    let token = piece_entity(&mut app, "santiago:active").unwrap();
    let projected = project(&mut app, center);
    let at = exposed_point(&mut app, token, projected);
    click_at(&mut app, at);
    let read = popover(&mut app);
    assert_eq!(
        read["card"]["rows"][0]["choice"], "6:remove:santiago:active:1",
        "click at {at}: {read}"
    );
    let world = app.world_mut();
    assert!(
        world
            .get::<MeshMaterial3d<StandardMaterial>>(token)
            .is_none()
    );
    assert!(
        world
            .query::<&highlights::Marker>()
            .iter(world)
            .any(|marker| marker.piece == "santiago:active")
    );
    assert!(roundel_entity(&mut app, token).is_none());
    assert!(still_flying(&app));
    assert_eq!(revision(&app), 6);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_motion_landing_preserves_an_unrelated_piece_press() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = removal_during_terror(&temp.path().join("task.json"));
    let key = "oriente:underground";
    let before = piece_entity(&mut app, key).unwrap();
    let badge = roundel_entity(&mut app, before).unwrap();
    let center = slot(&app, "oriente", guerrilla(State::Underground));
    frame(&mut app, center, 5.);
    let at = project(&mut app, center);
    let at = exposed_point(&mut app, before, at);
    press(&mut app, at);
    assert!(
        app.world().resource::<HoverMap>()[&PointerId::Mouse].contains_key(&before),
        "press at {at} hits the unrelated group"
    );
    landing(&mut app);
    release(&mut app, at);
    let read = popover(&mut app);
    assert_eq!(
        (
            read["card"]["rows"][0]["choice"].as_str(),
            piece_entity(&mut app, key)
        ),
        (Some("6:remove:oriente:underground:1"), Some(before)),
        "press and release on the same unrelated group: {read}"
    );
    assert_eq!(roundel_entity(&mut app, before), Some(badge));
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_motion_landing_preserves_the_arriving_piece_press() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = removal_during_terror(&temp.path().join("task.json"));
    let key = "santiago:active";
    let center = slot(&app, "santiago", guerrilla(State::Active));
    frame(&mut app, center, 5.);
    let point = project(&mut app, center);
    let before = piece_entity(&mut app, key).unwrap();
    let at = exposed_point(&mut app, before, point);
    press(&mut app, at);
    assert!(still_flying(&app));
    landing(&mut app);
    let after = piece_entity(&mut app, key).unwrap();
    assert!(
        app.world().resource::<HoverMap>()[&PointerId::Mouse].contains_key(&after),
        "the same pointer position {at} still hits the arriving group"
    );
    release(&mut app, at);
    let read = popover(&mut app);
    assert_eq!(
        (read["card"]["rows"][0]["choice"].as_str(), after),
        (Some("6:remove:santiago:active:1"), before),
        "the landing still completes the press on the arriving piece: {read}"
    );
    let badge = roundel_entity(&mut app, after).unwrap();
    let world = app.world();
    assert!(
        world
            .get::<MeshMaterial3d<StandardMaterial>>(after)
            .is_some()
    );
    let digit = world.get::<Children>(badge).unwrap().iter().next().unwrap();
    assert_eq!(world.get::<Text>(digit).unwrap().0, "1");
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_motion_leaves_the_closed_casino_return_available_during_its_turn() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = prepare(
        &temp.path().join("task.json"),
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "transfer",
            "give:0:4",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
            "assault:havana:syndicate",
            "end-assault",
            "syndicate:limited-operation",
            "construct:havana:open",
            "remove",
        ],
    );
    let casino = |state| crate::Group {
        faction: Faction::Syndicate,
        kind: Kind::Casino,
        state,
        count: 1,
    };
    let center = slot(&app, "pinar-del-rio", casino(State::Open));
    frame(&mut app, center, 5.);
    let open = project(&mut app, center);
    let entity = piece_entity(&mut app, "pinar-del-rio:open-casino").unwrap();
    let open = exposed_point(&mut app, entity, open);
    click_at(&mut app, open);
    assert_eq!(
        popover(&mut app)["kind"],
        "open-casino",
        "click open Casino at {open}"
    );
    click_control(&mut app, "popover:casino-close");
    assert_eq!(revision(&app), 17);
    assert!(still_flying(&app));

    let closed_name = Some("own-piece:pinar-del-rio:closed-casino");
    for _ in 0..24 {
        if app.world().resource::<Interface>().choice.focus.as_deref() == closed_name {
            break;
        }
        key(&mut app, KeyCode::Tab, Key::Tab, None);
        app.update();
    }
    assert_eq!(
        app.world().resource::<Interface>().choice.focus.as_deref(),
        closed_name
    );
    key(&mut app, KeyCode::Enter, Key::Enter, None);
    app.update();
    app.update();
    let keyboard = popover(&mut app);
    assert_eq!(
        keyboard["card"]["rows"][1]["button"], "Return 1 Casino to Available",
        "keyboard during turn: {keyboard}"
    );
    key(&mut app, KeyCode::Escape, Key::Escape, None);
    app.update();

    let center = slot(&app, "pinar-del-rio", casino(State::Closed));
    let closed = project(&mut app, center);
    click_at(&mut app, closed);
    let pointer = popover(&mut app);
    assert_eq!(
        pointer["card"]["rows"][1]["button"], "Return 1 Casino to Available",
        "pointer during turn at {closed}: {pointer}"
    );
    assert!(still_flying(&app));
    click_control(&mut app, "popover:casino-return");
    assert_eq!(revision(&app), 18);
    assert!(
        !still_flying(&app),
        "there is no longer a Casino to finish turning"
    );
    let trips = force_travel::observe(app.world_mut());
    assert_eq!(trips[0]["from"], "pinar-del-rio");
    assert_eq!(trips[0]["group"]["state"], "Closed");
}
