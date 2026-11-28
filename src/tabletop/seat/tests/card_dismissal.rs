use super::*;
use bevy::picking::pointer::PointerButton;

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer"]
fn native_escape_closes_a_pinned_card_while_hover_dwell_is_pending() {
    let cards: std::path::PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let directory = tempfile::tempdir().unwrap();
    let save = directory.path().join("table.json");
    let (mut app, mut seat) = table(&cards, &save, UVec2::new(3840, 2160));
    let opening = ask(&mut seat, &mut app, json!({"op":"observe"}));
    let center = |name| {
        let cards = opening["table"]["card"]["cards"].as_array().unwrap();
        let card = cards.iter().find(|card| card["card"] == name).unwrap();
        let bounds = |edge| card["bounds"][edge].as_f64().unwrap() as f32;
        Vec2::new((bounds(0) + bounds(2)) / 2., (bounds(1) + bounds(3)) / 2.)
    };
    let current = center("card:current");
    let preview = center("card:preview");
    let click_current = |app: &mut App| {
        for action in [
            PointerAction::Move { delta: Vec2::ZERO },
            PointerAction::Press(PointerButton::Primary),
            PointerAction::Release(PointerButton::Primary),
        ] {
            pointer(app, current, action);
            app.update();
        }
        app.update();
    };

    // The app's clock stays still during input. Pinning is immediate, while hover
    // has not had time to open. Ordinary keys can reach the app during this dwell.
    click_current(&mut app);
    let pinned = ask(&mut seat, &mut app, json!({"op":"observe"}));
    assert_eq!(pinned["table"]["card"]["shown"]["card"], "card:current");
    assert_eq!(pinned["table"]["card"]["shown"]["pinned"], true);
    assert!(pinned["table"]["hover"].is_null());
    key(&mut app, KeyCode::Escape, Key::Escape, None);
    app.update();
    ask(&mut seat, &mut app, json!({"op":"advance","steps":60}));
    let dismissed = ask(&mut seat, &mut app, json!({"op":"observe"}));
    assert!(dismissed["table"]["card"]["shown"].is_null());

    click_current(&mut app);
    pointer(&mut app, preview, PointerAction::Move { delta: Vec2::ZERO });
    app.update();
    let pinned = ask(&mut seat, &mut app, json!({"op":"observe"}));
    assert_eq!(pinned["table"]["card"]["shown"]["card"], "card:current");
    assert_eq!(pinned["table"]["card"]["shown"]["pinned"], true);
    assert!(pinned["table"]["hover"].is_null());

    key(&mut app, KeyCode::Escape, Key::Escape, None);
    app.update();
    ask(&mut seat, &mut app, json!({"op":"advance","steps":60}));
    let opened = ask(&mut seat, &mut app, json!({"op":"observe"}));
    assert_eq!(opened["table"]["card"]["shown"]["card"], "card:preview");
    assert_eq!(opened["table"]["card"]["shown"]["pinned"], false);
    assert_eq!(opened["observation"], opening["observation"]);
}
