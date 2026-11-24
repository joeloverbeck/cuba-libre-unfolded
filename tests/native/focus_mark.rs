//! #179: the focus mark is drawn on the control Enter would press, and on no other.
//! The seat's report of the drawn marks is compared with where the keyboard is, and the
//! mark's line is found in the captured pixels.
use super::*;

/// The marks' report: each one's control and form.
fn marks(view: &Value) -> Vec<(String, String)> {
    view["table"]["focus_mark"]
        .as_array()
        .unwrap_or_else(|| panic!("no focus_mark in the table"))
        .iter()
        .map(|m| {
            let of = |key: &str| m[key].as_str().unwrap_or_default().to_owned();
            (of("control"), of("form"))
        })
        .collect()
}

const INK: [u8; 3] = [0x20, 0x32, 0x38];
const INK_SECONDARY: [u8; 3] = [0x53, 0x6a, 0x6d];

/// Whether a pixel at `at` or next to it is `colour`, as drawn.
fn finds(image: &image::RgbImage, at: [f64; 2], colour: [u8; 3]) -> bool {
    let [x, y] = at.map(|v| v.floor() as i64);
    (-1..=1).any(|dx| {
        (-1..=1).any(|dy| {
            let (Ok(x), Ok(y)) = (u32::try_from(x + dx), u32::try_from(y + dy)) else {
                return false;
            };
            image
                .get_pixel_checked(x, y)
                .is_some_and(|p| p.0.iter().zip(colour).all(|(a, b)| a.abs_diff(b) <= 24))
        })
    })
}

/// The mark's line is in the capture on all four sides of each drawn mark, 1.75
/// reference pixels inside its rectangle (`atlas-focus-mark.vectorcraft`).
fn assert_line(app: &Native, view: &Value, capture: &str) {
    let image = image::open(app.root.join(format!("{capture}.png")))
        .unwrap()
        .to_rgb8();
    let s = (f64::from(image.width()) / 1920.).min(f64::from(image.height()) / 1280.);
    for mark in view["table"]["focus_mark"].as_array().unwrap() {
        let [x0, y0, x1, y1] = rect_of(&mark["bounds"]);
        let ink = if mark["form"] == "full" {
            INK
        } else {
            INK_SECONDARY
        };
        let (line, x, y) = (1.75 * s, (x0 + x1) / 2., (y0 + y1) / 2.);
        for at in [
            [x, y0 + line],
            [x, y1 - line],
            [x0 + line, y],
            [x1 - line, y],
        ] {
            assert!(
                finds(&image, at, ink),
                "{capture}: no line at {at:?} of {mark}"
            );
        }
    }
}

/// Only `control` carries the mark, and the capture shows it there.
fn assert_marked(app: &mut Native, control: &str, capture: &str) -> Value {
    app.request(json!({"op":"settle"}), true);
    let view = app.observe();
    assert_eq!(
        marks(&view),
        [(control.to_owned(), "full".to_owned())],
        "{capture}"
    );
    app.capture(capture);
    assert_line(app, &view, capture);
    view
}

/// Nothing carries the mark.
fn assert_unmarked(app: &mut Native, when: &str) {
    assert_eq!(marks(&app.observe()), [], "{when}");
}

/// Whether the control of this name carries the mark while it is a stop: not a space, a
/// card, a pile, the marker supply or a ledger row.
fn carries(stop: &str) -> bool {
    !["space:", "card:", "pile:", "available:", "marker-supply"]
        .iter()
        .any(|prefix| stop.starts_with(prefix))
}

/// The control with keyboard focus, in the modal while it is shown.
fn focus_of(view: &Value) -> String {
    let transfer = &view["table"]["transfer"]["state"];
    let focus = if transfer["shown"] == true {
        &transfer["focus"]
    } else {
        &view["table"]["choice"]["focus"]
    };
    focus.as_str().unwrap_or_default().to_owned()
}

/// Presses Tab and returns the table after it.
fn tab(app: &mut Native) -> Value {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
    app.request(json!({"op":"settle"}), true);
    app.observe()
}

/// Presses Tab round every stop, and compares the mark with each.
fn assert_round(app: &mut Native, when: &str) -> Vec<String> {
    let mut passed: Vec<String> = vec![];
    for _ in 0..60 {
        let view = tab(app);
        let focus = focus_of(&view);
        if passed.first() == Some(&focus) {
            return passed;
        }
        // The field's own two marks are compared apart.
        if focus != "transfer:field" {
            let expected: Vec<_> = carries(&focus)
                .then(|| (focus.clone(), "full".to_owned()))
                .into_iter()
                .collect();
            assert_eq!(marks(&view), expected, "{when}: {focus} has focus");
        }
        passed.push(focus);
    }
    panic!("{when}: Tab never came round: {passed:?}");
}

/// Moves the modal's focus to `stop` with Tab.
fn modal_to(app: &mut Native, stop: &str) {
    for _ in 0..20 {
        if focus_of(&tab(app)) == stop {
            return;
        }
    }
    panic!("Tab never reached {stop} in the modal");
}

fn escape(app: &mut Native) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.request(json!({"op":"key","revision":revision,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
}

/// Every stop of the action choice, Terror, its popover, Kidnap, the die and the modal.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_focus_mark_is_on_what_enter_presses() {
    let mut run = rally::event_app("focus-mark-", "issue-179");
    let app = &mut *run;
    app.launch_size("3840x2160");
    app.point("area:propaganda");
    assert_unmarked(app, "nothing has focus");

    // The action choice: a playable and an unsupported option, a disabled and an enabled
    // commit, and Save · quit.
    let order = assert_round(app, "action choice");
    assert!(order.iter().any(|n| n == "option:pass") && order.iter().any(|n| n == "quit"));
    for (stop, capture) in [
        ("option:terror", "option"),
        ("option:rally", "option-unsupported"),
        ("choice-commit", "commit-disabled"),
        ("quit", "save-quit"),
    ] {
        map_keys::tab_to(app, stop);
        assert_marked(app, stop, capture);
    }
    // The pointer on one option leaves the mark on the focused other, and the hovered
    // option's outline where it was.
    map_keys::tab_to(app, "option:terror");
    map_keys::press(app, "Enter", "option:terror");
    app.point("option:pass");
    let hovering = assert_marked(app, "option:terror", "option-focused-other-hovered");
    let outline = choice_part(&hovering, "option-active")["bounds"].clone();
    assert_eq!(outline, choice_part(&hovering, "option:pass")["bounds"]);
    assert_eq!(hovering["table"]["choice"]["pinned"], "terror");
    app.point("area:propaganda");
    map_keys::tab_to(app, "0:terror");
    assert_marked(app, "0:terror", "commit");
    map_keys::press(app, "Enter", "0:terror");
    assert_unmarked(app, "a decision clears focus");

    // Terror: a step, the disabled commit, the ribbon and its tabs.
    assert_round(app, "Terror");
    for (stop, capture) in [
        ("step:transfer", "step"),
        ("execution-commit", "execution-commit-disabled"),
        ("briefing-ribbon", "ribbon"),
    ] {
        map_keys::tab_to(app, stop);
        assert_marked(app, stop, capture);
    }
    map_keys::press(app, "Enter", "briefing-ribbon");
    map_keys::tab_to(app, "briefing-tab:kidnap");
    assert_marked(app, "briefing-tab:kidnap", "briefing-tab");
    escape(app);
    assert_unmarked(app, "the closed ribbon's tab is no stop");

    // The popover: no mark on its space; one on its row, and one on its button, whose
    // popover is otherwise drawn the same.
    map_keys::tab_to(app, "space:santiago");
    assert_unmarked(app, "a focused space shows its tag");
    map_keys::press(app, "Enter", "space:santiago");
    assert_unmarked(app, "a focused space with its popover open");
    map_keys::tab_to(app, "popover:row:kidnap");
    let row = assert_marked(app, "popover:row:kidnap", "popover-row");
    map_keys::tab_to(app, "popover:commit");
    let button = assert_marked(app, "popover:commit", "popover-button");
    assert_eq!(row["table"]["popover"], button["table"]["popover"]);
    let changed = changes_within(
        &app.root.join("popover-row.png"),
        &app.root.join("popover-button.png"),
        rect_of(&button["table"]["focus_mark"][0]["bounds"]),
    );
    assert!(changed > 1_000, "{changed} pixels differ round the button");
    // The pointer closes the popover, opens another, and opens the first again: the
    // mark is on the button while a popover shows one.
    app.activate("space:santiago", 1);
    assert_unmarked(app, "the popover is closed");
    app.activate("space:oriente", 1);
    assert_unmarked(app, "Oriente's popover has no button");
    app.activate("space:santiago", 1);
    assert_marked(app, "popover:commit", "popover-swapped");
    escape(app);
    assert_unmarked(app, "Escape leaves focus on the space");

    // Kidnap: the held step, Cancel and Roll; then Continue, which Enter presses
    // without Tab once the die has landed.
    map_keys::press(app, "Enter", "space:santiago");
    map_keys::tab_to(app, "popover:commit");
    map_keys::press(app, "Enter", "popover:commit");
    assert_round(app, "Kidnap open");
    for (stop, capture) in [
        ("2:kidnap:cancel", "kidnap-cancel"),
        ("2:kidnap:roll", "kidnap-roll"),
    ] {
        map_keys::tab_to(app, stop);
        assert_marked(app, stop, capture);
    }
    map_keys::press(app, "Enter", "2:kidnap:roll");
    assert_marked(app, "kidnap-die:continue", "die-continue");
    map_keys::press(app, "Enter", "kidnap-die:continue");
    assert_unmarked(app, "Continue closed the die");
    // Both Terrors can be paid now: the pointer swaps one popover's button for
    // another's, which Enter would press.
    map_keys::tab_to(app, "space:santiago");
    map_keys::press(app, "Enter", "space:santiago");
    map_keys::tab_to(app, "popover:commit");
    let santiago = assert_marked(app, "popover:commit", "santiago-button");
    let at = santiago["observation"]["revision"].as_u64().unwrap();
    app.activate("space:oriente", at);
    let oriente = assert_marked(app, "popover:commit", "oriente-button");
    assert_eq!(oriente["table"]["popover"]["space"], "oriente");
    assert_ne!(
        santiago["table"]["focus_mark"][0]["bounds"],
        oriente["table"]["focus_mark"][0]["bounds"]
    );
    escape(app);
    // Removal mode's panel.
    app.choose("remove");
    assert_round(app, "removal mode");
    app.choose("done-removal");

    // The modal: the primary at rest, while blocked and while it can commit.
    app.choose("transfer");
    let at = app.observe()["observation"]["revision"].as_u64().unwrap();
    assert_marked(app, "transfer:primary", "modal-opened");
    let order = assert_round(app, "modal");
    assert_eq!(order.len(), 11, "{order:?}");
    app.activate("transfer:tab:syndicate", at);
    app.activate("transfer:direction:ask", at);
    app.activate("transfer:quick:2", at);
    let primary = format!("{at}:ask:3:2");
    modal_to(app, &primary);
    assert_marked(app, &primary, "modal-primary");
    for (stop, capture) in [
        ("transfer:tab:government".to_owned(), "modal-tab"),
        ("transfer:direction:give".to_owned(), "modal-direction"),
        ("transfer:quick:1".to_owned(), "modal-quick"),
        (format!("{at}:back"), "modal-cancel"),
    ] {
        modal_to(app, &stop);
        assert_marked(app, &stop, capture);
    }
    // The amount field: the light form on it, and the mark on the primary.
    modal_to(app, "transfer:field");
    let field = app.observe();
    let mut both = marks(&field);
    both.sort();
    assert_eq!(
        both,
        [
            (primary, "full".to_owned()),
            ("transfer:field".to_owned(), "light".to_owned())
        ]
    );
    app.capture("modal-field");
    assert_line(app, &field, "modal-field");

    // The giver's decision, in the giver's colour; then the pending request.
    app.choose("ask:3:2");
    let at = at + 1;
    assert_marked(app, &format!("{at}:authorize:2"), "giver");
    modal_to(app, &format!("{at}:decline"));
    assert_marked(app, &format!("{at}:decline"), "giver-decline");
    app.choose("wait");
    assert_unmarked(app, "a waiting request");
    map_keys::tab_to(app, "execution-commit");
    assert_marked(app, "execution-commit", "execution-commit-opens-request");
    app.quit();
}

/// A pending request's Open and the card advance, in Directorio's Event.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_focus_mark_is_on_a_pending_request_and_the_card_advance() {
    let mut run = rally::event_app("focus-mark-event-", "issue-179");
    let app = &mut *run;
    rally::open_event_handoff(app);
    assert_round(app, "Directorio handoff");
    rally::choose_rally_option(app, "event:shaded");
    assert_round(app, "Directorio's Event");
    rally::choose_rally_option(app, "transfer");
    app.choose("ask:3:2");
    app.choose("wait");
    let at = app.observe()["observation"]["revision"].as_u64().unwrap();
    let open = format!("{at}:reopen");
    map_keys::tab_to(app, &open);
    assert_marked(app, &open, "pending-open");
    map_keys::press(app, "Enter", &open);
    app.choose("authorize:2");
    rally::choose_rally_option(app, "resolve-shaded");
    let at = app.observe()["observation"]["revision"].as_u64().unwrap();
    let advance = format!("{at}:advance-card");
    map_keys::tab_to(app, &advance);
    assert_marked(app, &advance, "advance-card");
    let flying = app.request(
        json!({"op":"key","revision":at,"key":"Enter","focused":advance}),
        true,
    );
    assert_eq!(marks(&flying), [], "a card is in the air");
    app.request(json!({"op":"settle"}), true);
    assert_unmarked(app, "the cards have landed");
    // Card 18: Government's options, then Assault's panel, spaces and popover.
    assert_round(app, "card 18");
    rally::choose_rally_option(app, "government:assault");
    let order = assert_round(app, "Assault");
    assert!(order.iter().any(|n| n == "space:havana"), "{order:?}");
    app.quit();
}

/// The mark keeps to its control in the other windows.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_focus_mark_follows_the_window() {
    for (size, name) in [
        ("1920x1280", "3-2"),
        ("1920x1080", "16-9"),
        ("2520x1080", "21-9"),
    ] {
        let mut run = rally::event_app(&format!("focus-mark-{name}-"), "issue-179");
        let app = &mut *run;
        app.launch_size(size);
        app.point("area:propaganda");
        for (stop, capture) in [
            ("option:terror", "option"),
            ("choice-commit", "commit"),
            ("quit", "save-quit"),
        ] {
            map_keys::tab_to(app, stop);
            assert_marked(app, stop, &format!("{name}-{capture}"));
        }
        app.choose("terror");
        for (stop, capture) in [("step:transfer", "step"), ("briefing-ribbon", "ribbon")] {
            map_keys::tab_to(app, stop);
            assert_marked(app, stop, &format!("{name}-{capture}"));
        }
        map_keys::press(app, "Enter", "briefing-ribbon");
        map_keys::tab_to(app, "briefing-tab:kidnap");
        assert_marked(app, "briefing-tab:kidnap", &format!("{name}-tab"));
        escape(app);
        app.choose("transfer");
        assert_marked(app, "transfer:primary", &format!("{name}-modal"));
        for (stop, capture) in [
            ("transfer:tab:government", "modal-tab"),
            ("transfer:direction:give", "modal-direction"),
            ("transfer:quick:1", "modal-quick"),
        ] {
            modal_to(app, stop);
            assert_marked(app, stop, &format!("{name}-{capture}"));
        }
        modal_to(app, "transfer:field");
        let field = app.observe();
        assert_eq!(marks(&field).len(), 2);
        app.capture(&format!("{name}-modal-field"));
        assert_line(app, &field, &format!("{name}-modal-field"));
        app.choose("back");
        map_keys::tab_to(app, "space:santiago");
        map_keys::press(app, "Enter", "space:santiago");
        for (stop, capture) in [
            ("popover:row:kidnap", "popover-row"),
            ("popover:commit", "popover"),
        ] {
            map_keys::tab_to(app, stop);
            assert_marked(app, stop, &format!("{name}-{capture}"));
        }
        let kidnap = map_keys::press(app, "Enter", "popover:commit");
        let at = kidnap["observation"]["revision"].as_u64().unwrap();
        let cancel = format!("{at}:kidnap:cancel");
        map_keys::tab_to(app, &cancel);
        assert_marked(app, &cancel, &format!("{name}-kidnap-cancel"));
        app.roll();
        app.quit();
    }
}
