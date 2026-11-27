//! The shared seat advances each individual input in its own frame. Queue the
//! window's ordinary input messages together here to cover the frame boundary.
use super::*;
use bevy::{
    camera::RenderTarget,
    input::{ButtonState, keyboard::KeyboardInput},
    picking::pointer::{Location, PointerAction, PointerButton, PointerInput},
};
use std::{
    io::{BufRead, BufReader, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    time::Instant,
};

fn pending_giver(save: &std::path::Path) -> crate::save::Session {
    let mut session = crate::save::Session::create_seeded(save, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "ask:3:2",
    ] {
        let view = session.observe();
        session
            .submit(
                view.owner,
                view.revision,
                &format!("{}:{action}", view.revision),
            )
            .unwrap();
    }
    session
}

fn key(app: &mut App, key_code: KeyCode, logical_key: Key, text: Option<&str>) {
    let world = app.world_mut();
    let window = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world)
        .unwrap();
    for state in [ButtonState::Pressed, ButtonState::Released] {
        world.write_message(KeyboardInput {
            key_code,
            logical_key: logical_key.clone(),
            state,
            text: text.filter(|_| state.is_pressed()).map(Into::into),
            repeat: false,
            window,
        });
    }
}

fn pointer(app: &mut App, position: Vec2, action: PointerAction) {
    let world = app.world_mut();
    let (window, mut cursor) = world
        .query_filtered::<(Entity, &mut Window), With<PrimaryWindow>>()
        .single_mut(world)
        .unwrap();
    cursor.set_cursor_position(Some(position));
    let target = world
        .query_filtered::<&RenderTarget, With<IsDefaultUiCamera>>()
        .single(world)
        .unwrap()
        .normalize(Some(window))
        .unwrap();
    world.write_message(PointerInput::new(
        PointerId::Mouse,
        Location { target, position },
        action,
    ));
}

fn press_button(
    app: &mut App,
    seat: &mut shared::Seat<Table>,
    revision: u64,
    component: &str,
) -> Vec2 {
    let pointed = ask(
        seat,
        app,
        json!({"op":"point","revision":revision,"component":component}),
    );
    assert_eq!(pointed["status"], "pointed", "{pointed}");
    let coordinate = |axis| pointed["position"][axis].as_f64().unwrap() as f32;
    let position = Vec2::new(coordinate(0usize), coordinate(1usize));
    pointer(app, position, PointerAction::Press(PointerButton::Primary));
    ask(seat, app, json!({"op":"advance","steps":1}));
    position
}

fn click_with_edit(
    app: &mut App,
    seat: &mut shared::Seat<Table>,
    revision: u64,
    component: &str,
    code: KeyCode,
    text: &str,
) -> Value {
    let position = press_button(app, seat, revision, component);
    assert!(
        app.world()
            .resource::<bevy::input_focus::InputFocus>()
            .get()
            .is_some_and(|focus| app
                .world()
                .get::<proclamation_composer::Editor>(focus)
                .is_some()),
        "the writing field receives text through the release frame",
    );
    key(app, code, Key::Character(text.into()), Some(text));
    pointer(
        app,
        position,
        PointerAction::Release(PointerButton::Primary),
    );
    ask(seat, app, json!({"op":"advance","steps":1}));
    ask(seat, app, json!({"op":"settle"}))
}

fn queue_faction_request(
    app: &mut App,
    seat: &mut shared::Seat<Table>,
    socket: &std::path::Path,
    request: Value,
) -> UnixStream {
    let mut connection = UnixStream::connect(socket).unwrap();
    connection
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    writeln!(connection, "{request}").unwrap();
    connection.shutdown(Shutdown::Write).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while ask(seat, app, json!({"op":"observe"}))["pending"] != json!(["faction seats"]) {
        assert!(Instant::now() < deadline, "request did not reach the table");
        std::thread::sleep(Duration::from_millis(10));
    }
    connection
}

fn faction_reply(connection: UnixStream, status: &str) -> Value {
    let mut line = String::new();
    BufReader::new(connection).read_line(&mut line).unwrap();
    let reply: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(reply["status"], status, "{reply}");
    reply
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_proclamation_publish_raced_by_delivery_requires_explicit_retry() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let directory = tempfile::tempdir().unwrap();
    let save = directory.path().join("table.json");
    let socket = save.with_extension("sock");
    let revision = pending_giver(&save).observe().revision;
    let (mut app, mut seat) = table(&cards, &save, UVec2::new(1920, 1080));
    ask(
        &mut seat,
        &mut app,
        json!({"op":"key","revision":revision,"key":"F3"}),
    );
    let join = queue_faction_request(
        &mut app,
        &mut seat,
        &socket,
        json!({
            "op":"join","faction":"directorio",
        }),
    );
    ask(&mut seat, &mut app, json!({"op":"advance","steps":1}));
    let joined = faction_reply(join, "joined");
    let before = open_composer(&mut seat, &mut app, revision);
    let position = press_button(&mut app, &mut seat, revision, "proclamation:publish");
    let incoming = queue_faction_request(
        &mut app,
        &mut seat,
        &socket,
        json!({
            "op":"publish","player":joined["player"],"revision":revision,
            "card":17,"text":"We heard your request.",
        }),
    );
    key(
        &mut app,
        KeyCode::KeyA,
        Key::Character("á".into()),
        Some("á"),
    );
    pointer(
        &mut app,
        position,
        PointerAction::Release(PointerButton::Primary),
    );
    let held = ask(&mut seat, &mut app, json!({"op":"advance","steps":1}));
    faction_reply(incoming, "published");
    assert_eq!(held["table"]["proclamations"]["open"], 1);
    assert_eq!(held["observation"], before["observation"]);
    ask(
        &mut seat,
        &mut app,
        json!({"op":"key","revision":revision,"key":"Escape"}),
    );
    let restored = ask(&mut seat, &mut app, json!({"op":"settle"}));
    let composer = &restored["table"]["proclamation_composer"];
    assert_eq!(composer["draft"], "á");
    assert_eq!(composer["focus"], "proclamation:field");
    assert_eq!(
        composer["refusal"],
        "Publish interrupted. Your draft is kept. Press Publish again.",
    );
    assert_eq!(
        restored["table"]["proclamations"]["documents"]
            .as_array()
            .unwrap()
            .len(),
        1,
    );
    let retried = ask(
        &mut seat,
        &mut app,
        json!({"op":"activate","revision":revision,
        "component":"proclamation:publish"}),
    );
    assert_eq!(retried["status"], "activated", "{retried}");
    let published = ask(&mut seat, &mut app, json!({"op":"settle"}));
    assert_eq!(
        published["table"]["proclamations"]["documents"][1]["text"],
        "á"
    );
    assert!(published["table"]["proclamation_composer"].is_null());
    assert_eq!(published["observation"], before["observation"]);
}

fn open_composer(seat: &mut shared::Seat<Table>, app: &mut App, revision: u64) -> Value {
    let opened = ask(
        seat,
        app,
        json!({"op":"key","revision":revision,"key":"KeyP"}),
    );
    assert_eq!(opened["status"], "pressed", "{opened}");
    ask(seat, app, json!({"op":"settle"}))
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_proclamation_editor_keeps_same_frame_edits() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let directory = tempfile::tempdir().unwrap();
    let save = directory.path().join("table.json");
    let revision = pending_giver(&save).observe().revision;
    let (mut app, mut seat) = table(&cards, &save, UVec2::new(1920, 1080));

    let opened = open_composer(&mut seat, &mut app, revision);
    assert_eq!(
        opened["table"]["proclamation_composer"]["author"],
        "Syndicate"
    );
    click_with_edit(
        &mut app,
        &mut seat,
        revision,
        "proclamation:compose:return",
        KeyCode::KeyA,
        "á",
    );
    let reopened = open_composer(&mut seat, &mut app, revision);
    let after_return = &reopened["table"]["proclamation_composer"]["draft"];

    click_with_edit(
        &mut app,
        &mut seat,
        revision,
        "proclamation:author:26july",
        KeyCode::KeyB,
        "b",
    );
    let switched = ask(
        &mut seat,
        &mut app,
        json!({"op":"activate","revision":revision,
        "component":"proclamation:author:syndicate"}),
    );
    assert_eq!(switched["status"], "activated", "{switched}");
    let restored = ask(&mut seat, &mut app, json!({"op":"settle"}));
    let after_switch = &restored["table"]["proclamation_composer"]["draft"];

    let published = click_with_edit(
        &mut app,
        &mut seat,
        revision,
        "proclamation:publish",
        KeyCode::KeyC,
        "c",
    );
    assert_eq!(
        json!([
            after_return,
            after_switch,
            published["table"]["proclamations"]["documents"][0]["text"]
        ]),
        json!(["á", "áb", "ábc"]),
    );
    assert!(published["table"]["proclamation_composer"].is_null());
    assert_eq!(published["observation"], opened["observation"]);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_proclamation_opening_consumes_the_giver_confirmation() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let directory = tempfile::tempdir().unwrap();
    let save = directory.path().join("table.json");
    let mut session = pending_giver(&save);
    let revision = session.observe().revision;
    session
        .publish_as_human(Faction::Government, revision, 17, "Read our offer.")
        .unwrap();
    drop(session);
    let (mut app, mut seat) = table(&cards, &save, UVec2::new(1920, 1080));
    let tab = ask(
        &mut seat,
        &mut app,
        json!({"op":"key","revision":revision,"key":"Tab"}),
    );
    assert_eq!(tab["table"]["transfer"]["state"]["focus"], "transfer:field");
    let before = ask(&mut seat, &mut app, json!({"op":"settle"}));
    for (code, text) in [(KeyCode::KeyR, "r"), (KeyCode::KeyP, "p")] {
        key(&mut app, code, Key::Character(text.into()), Some(text));
        key(&mut app, KeyCode::Enter, Key::Enter, None);
        let opened = ask(&mut seat, &mut app, json!({"op":"advance","steps":1}));
        assert_eq!(opened["observation"], before["observation"], "{text}+Enter");
        if code == KeyCode::KeyR {
            assert_eq!(opened["table"]["proclamations"]["open"], 1);
        } else {
            assert_eq!(opened["table"]["proclamation_composer"]["draft"], "");
        }
        ask(
            &mut seat,
            &mut app,
            json!({"op":"key","revision":revision,"key":"Escape"}),
        );
        let restored = ask(&mut seat, &mut app, json!({"op":"settle"}));
        assert_eq!(restored["observation"], before["observation"]);
        assert_eq!(restored["table"]["transfer"], before["table"]["transfer"]);
    }
}
