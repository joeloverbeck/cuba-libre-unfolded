//! #139: the card tag, at the owner's display, against its FigCraft layout record and
//! the rules model, through the development seat's own pointer and keys.
use super::*;
use std::collections::BTreeMap;

/// The tag's drawn parts by name, in logical window pixels.
fn parts(view: &Value) -> BTreeMap<String, [f64; 4]> {
    view["table"]["card"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| {
            (
                part["part"].as_str().unwrap().to_owned(),
                rect_of(&part["bounds"]),
            )
        })
        .collect()
}

/// The open tag against its frame in `atlas-card-tag.layout.json`: images on all four
/// edges, text on left, top and right, within 2 px, relative to the plate.
fn assert_matches_layout(view: &Value, frame: &str) {
    let layout: Value =
        serde_json::from_str(include_str!("../../art/atlas/atlas-card-tag.layout.json")).unwrap();
    // The frames are drawn at the owner's display, the 1920x1280 reference at x1.6875,
    // and the seat runs at 3840x2160, where the window draws the reference at the same
    // scale: one frame unit is one logical pixel.
    assert_eq!(layout["scale"], 1.6875);
    let ratio = 1.;
    let drawn = parts(view);
    let plate = drawn["plate"];
    for (group, edges) in [("parts", 4), ("text", 3)] {
        for (name, rect) in layout["frames"][frame][group].as_object().unwrap() {
            let r: Vec<f64> = rect
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() * ratio)
                .collect();
            let expected = [
                plate[0] + r[0],
                plate[1] + r[1],
                plate[0] + r[0] + r[2],
                plate[1] + r[1] + r[3],
            ];
            let native = drawn
                .get(name)
                .unwrap_or_else(|| panic!("the {frame} tag has no {name}"));
            for (n, e) in native.iter().zip(expected).take(edges) {
                assert!(
                    (n - e).abs() <= 2.,
                    "{frame} {name}: native {native:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    }
}

/// The open tag states what the rules model says about the card in `slot`.
fn assert_states_the_model(view: &Value, slot: &str) {
    let key = slot.trim_start_matches("card:");
    let facts = &view["observation"]["cards"][key];
    let shown = &view["table"]["card"]["shown"];
    assert_eq!(shown["card"], slot);
    assert_eq!(shown["number"], facts["number"]);
    assert_eq!(
        shown["title"],
        format!("{} · {}", facts["number"], facts["title"].as_str().unwrap())
    );
    assert_eq!(
        shown["mark"],
        if key == "current" {
            "CURRENT CARD"
        } else {
            "PREVIEW"
        }
    );
    let order: Vec<&str> = facts["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| match f.as_str().unwrap() {
            "M26" => "26July",
            other => other,
        })
        .collect();
    assert_eq!(shown["order"], json!(order));
    for field in ["event", "status", "lasting", "rules"] {
        assert_eq!(shown[field], facts["words"][field], "{field}");
    }
    // Each fact is drawn as the model words it.
    let drawn = |part: &str| -> Value {
        view["table"]["card"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["part"] == part)
            .unwrap_or_else(|| panic!("no {part}"))["text"]
            .clone()
    };
    for field in ["event", "status", "lasting", "rules", "title"] {
        assert_eq!(drawn(field), shown[field], "{field}");
    }
    assert_eq!(drawn("mark-label"), shown["mark"]);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_card_tag_matches_layout_and_reads_the_model() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-139".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("card-tag-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Card tag native evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    let opening = app.observe();
    assert_eq!(opening["table"]["card"]["shown"], Value::Null);
    let controls = opening["table"]["controls"].as_array().unwrap();
    for gone in ["reader", "close"] {
        assert!(
            !controls.iter().any(|c| c["name"] == gone),
            "the reader's {gone} target remains"
        );
    }

    // Hover waits for the shared dwell, then opens the tag; it changes nothing.
    for slot in ["card:current", "card:preview"] {
        // An open tag switches at once; the dwell applies from nothing open.
        app.point("quit");
        app.request(json!({"op":"point","revision":0,"component":slot}), true);
        let waiting = app.observe();
        assert!(
            waiting["table"]["card"]["shown"].is_null(),
            "opened before the dwell"
        );
        assert!(
            waiting["table"]["pending"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p == "standing hover dwell"),
            "{}",
            waiting["table"]["pending"]
        );
        let shown = app.request(json!({"op":"settle"}), true);
        let shown = if shown["table"].is_null() {
            app.observe()
        } else {
            shown
        };
        assert_eq!(shown["table"]["hover"], slot);
        assert_eq!(shown["table"]["card"]["shown"]["pinned"], false);
        assert_states_the_model(&shown, slot);
        assert_card_tag_placed(&shown, slot);
        // The face reads at about 620 px on the owner's display, with rounded corners.
        let face = parts(&shown)["face"];
        assert!((face[2] - face[0] - 620.).abs() <= 1., "face {face:?}");
        assert_eq!(shown["observation"], opening["observation"]);
        app.capture(&format!(
            "opening-hover-{}",
            slot.trim_start_matches("card:")
        ));
    }

    // Keyboard focus opens each tag with the same dwell; Enter pins; Escape closes.
    app.point("quit");
    let mut focus = Value::Null;
    for _ in 0..20 {
        app.request(json!({"op":"key","revision":0,"key":"Tab"}), true);
        focus = app.observe()["table"]["choice"]["focus"].clone();
        if focus == "card:current" {
            break;
        }
    }
    assert_eq!(focus, "card:current");
    let focused = app.request(json!({"op":"settle"}), true);
    let focused = if focused["table"].is_null() {
        app.observe()
    } else {
        focused
    };
    assert_states_the_model(&focused, "card:current");
    app.request(json!({"op":"key","revision":0,"key":"Tab"}), true);
    app.request(json!({"op":"settle"}), true);
    let preview = app.observe();
    assert_eq!(preview["table"]["choice"]["focus"], "card:preview");
    assert_states_the_model(&preview, "card:preview");
    app.request(
        json!({"op":"key","revision":0,"key":"Enter","focused":"card:preview"}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    let pinned = app.observe();
    assert_eq!(pinned["table"]["card"]["shown"]["pinned"], true);
    app.capture("opening-focus-preview-pinned");
    close_card_tag(&mut app);
    // Escape keeps the focused card's tag closed until focus moves.
    app.request(json!({"op":"advance","steps":60}), true);
    assert!(app.observe()["table"]["card"]["shown"].is_null());
    // Tab wraps from the last card to the first option.
    app.request(json!({"op":"key","revision":0,"key":"Tab"}), true);
    assert_eq!(app.observe()["table"]["choice"]["focus"], "option:pass");

    // A click pins the tag: it stays open after the pointer leaves, until Escape.
    app.activate("card:current", 0);
    app.point("quit");
    let kept = app.observe();
    assert_eq!(kept["table"]["card"]["shown"]["card"], "card:current");
    assert_eq!(kept["table"]["card"]["shown"]["pinned"], true);
    assert_eq!(kept["observation"], opening["observation"]);
    close_card_tag(&mut app);
    // A second click unpins.
    app.activate("card:preview", 0);
    app.activate("card:preview", 0);
    app.point("quit");
    assert!(app.observe()["table"]["card"]["shown"].is_null());
    let untouched = std::fs::read(&app.save).unwrap();

    // Directorio's handoff after Terror with Kidnap: the layout record's state.
    app.choose("terror");
    app.choose("transfer");
    app.choose("ask:2:2");
    app.choose("authorize:2");
    app.choose("terror:santiago");
    app.choose("terror:oriente");
    app.choose("kidnap");
    app.roll();
    app.choose("end-terror");
    let handoff = app.observe();
    assert_eq!(
        handoff["observation"]["cards"]["current"]["words"]["status"],
        "Event open to Directorio as 2nd Eligible"
    );
    let revision = handoff["observation"]["revision"].as_u64().unwrap();
    let saved = std::fs::read(&app.save).unwrap();
    assert_ne!(saved, untouched);
    for (slot, frame) in [("card:current", "current"), ("card:preview", "preview")] {
        app.activate(slot, revision);
        let view = app.observe();
        assert_states_the_model(&view, slot);
        assert_card_tag_placed(&view, slot);
        assert_matches_layout(&view, frame);
        assert_eq!(view["observation"], handoff["observation"]);
        app.capture(&format!("handoff-{frame}"));
        close_card_tag(&mut app);
    }

    // Each later stage of card 17's Event, then card 18 after the advance.
    for (action, current, preview) in [
        (
            "event:shaded",
            "Event in progress: Directorio is executing the shaded text",
            "Event not yet open: this card is played after card 17",
        ),
        (
            "resolve-shaded",
            "Event resolved: Directorio executed the shaded text",
            "Event not yet open: this card is played after card 17",
        ),
        (
            "advance-card",
            "Event open to Government as 1st Eligible",
            "Event not yet open: this card is played after card 18",
        ),
    ] {
        app.choose(action);
        let view = app.observe();
        let revision = view["observation"]["revision"].as_u64().unwrap();
        for (slot, status) in [("card:current", current), ("card:preview", preview)] {
            app.activate(slot, revision);
            let shown = app.observe();
            assert_states_the_model(&shown, slot);
            assert_eq!(shown["table"]["card"]["shown"]["status"], status);
            assert_card_tag_placed(&shown, slot);
            assert_eq!(shown["observation"], view["observation"]);
            app.capture(&format!("{action}-{}", slot.trim_start_matches("card:")));
            close_card_tag(&mut app);
        }
    }
    let card_18 = app.observe();
    assert_eq!(
        card_18["observation"]["cards"]["current"]["title"],
        "Pact of Caracas"
    );
    assert_eq!(
        card_18["observation"]["cards"]["preview"]["title"],
        "Sierra Maestra Manifesto"
    );
    app.quit();
}
