//! #144: the two pile tags at the owner's display, against their FigCraft frames in
//! `atlas-pile-tags.layout.json` and the rules model, through the development seat's
//! own pointer and keys.
//!
//! Expectations worked by hand before the model (`tests/integration/pile_tags.rs`
//! states the working): on card 17 the played pile lists 18 cards, 17 down to 13, 49,
//! then 12 down to 1, with 17 current and 49 a Propaganda card; the deck holds 34, 33
//! of them face down, 3 of 4 Propaganda cards, and the next within the next 8 cards.
//! After the advance the pile lists 19 with 18 current, the deck holds 33, 32 face
//! down, and the next is within the next 7.
use super::*;
use std::collections::BTreeMap;

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-pile-tags.layout.json")).unwrap()
}

/// The open tag's drawn parts by name: their words, if any, and their bounds.
fn parts(view: &Value) -> BTreeMap<String, (Value, [f64; 4])> {
    view["table"]["pile_tag"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| {
            (
                part["part"].as_str().unwrap().to_owned(),
                (part["text"].clone(), rect_of(&part["bounds"])),
            )
        })
        .collect()
}

fn near(native: [f64; 4], expected: [f64; 4], edges: &[usize], what: &str) {
    for i in edges {
        assert!(
            (native[*i] - expected[*i]).abs() <= 2.,
            "{what}: native {native:?} differs from layout {expected:?} by more than 2 px"
        );
    }
}

/// The open tag against its frame: the plate where the main view opens it, images on
/// all four edges, text on left, top and right, within the record's 2 px. The seat
/// runs at 3840x2160, where one frame unit is one logical pixel. A played card's row
/// lies in its column's box, one row every 47.
fn assert_matches_layout(view: &Value, frame: &str, open: Option<&str>) {
    let record = record();
    assert_eq!(record["tolerance"], 2);
    let drawn = parts(view);
    let plate = drawn["plate"].1;
    let size = &record["frames"][frame]["size"];
    let expected_plate = match open {
        Some(open) => {
            let [x, y, w, h] = rect_of(&record["open"][open]);
            [x, y, x + w, y + h]
        }
        // The foot and left edge stay; the plate is the frame's size.
        None => [
            plate[0],
            plate[3] - size[1].as_f64().unwrap(),
            plate[0] + size[0].as_f64().unwrap(),
            plate[3],
        ],
    };
    near(
        plate,
        expected_plate,
        &[0, 1, 2, 3],
        &format!("{frame} plate"),
    );
    let at = |r: [f64; 4]| {
        [
            plate[0] + r[0],
            plate[1] + r[1],
            plate[0] + r[0] + r[2],
            plate[1] + r[1] + r[3],
        ]
    };
    let row = record["rows"]["line"].as_f64().unwrap();
    for (name, rect) in record["frames"][frame]["parts"].as_object().unwrap() {
        let expected = at(rect_of(rect));
        if let Some(column) = name.strip_prefix("column-") {
            let rows = drawn
                .iter()
                .filter(|(part, _)| part.starts_with(&format!("column-{column}-row-")))
                .count();
            assert!(rows > 0, "{frame}: no rows in {name}");
            for r in 0..rows {
                let (_, native) = drawn[&format!("column-{column}-row-{r}")];
                let top = expected[1] + row * r as f64;
                near(
                    native,
                    [expected[0], top, expected[2], top + row],
                    &[0, 1, 2],
                    &format!("{frame} {name} row {r}"),
                );
            }
            continue;
        }
        let (_, native) = drawn
            .get(name)
            .unwrap_or_else(|| panic!("the {frame} tag has no {name}"));
        let edges: &[usize] = if ["plate", "rule"].contains(&name.as_str()) {
            &[0, 1, 2, 3]
        } else {
            &[0, 1, 2]
        };
        near(*native, expected, edges, &format!("{frame} {name}"));
    }
}

/// The plate is drawn to its foot: paper just above its bottom edge and its right
/// edge, where an image drawn short of its node would leave the sea.
fn assert_plate_drawn(app: &Native, capture: &str, view: &Value) {
    let shot = image::open(app.root.join(format!("{capture}.png")))
        .unwrap()
        .to_rgb8();
    let plate = rect_of(&view["table"]["pile_tag"]["shown"]["bounds"]);
    let middle = ((plate[0] + plate[2]) / 2.) as u32;
    for (x, y) in [
        (middle, plate[3] as u32 - 12),
        (plate[2] as u32 - 12, ((plate[1] + plate[3]) / 2.) as u32),
        (middle, plate[1] as u32 + 12),
    ] {
        let p = shot.get_pixel(x, y).0;
        let paper = [248u8, 244, 232];
        assert!(
            p.iter().zip(paper).all(|(a, b)| a.abs_diff(b) <= 6),
            "{capture}: {p:?} at {x},{y} is not the plate's paper"
        );
    }
}

/// The tag lies inside the window, off both piles, their labels and discs.
fn assert_placed(view: &Value) {
    let plate = rect_of(&view["table"]["pile_tag"]["shown"]["bounds"]);
    let viewport = view["table"]["viewport"].as_array().unwrap();
    let (width, height) = (viewport[0].as_f64().unwrap(), viewport[1].as_f64().unwrap());
    assert!(
        plate[0] >= 0. && plate[1] >= 0. && plate[2] <= width && plate[3] <= height,
        "the tag {plate:?} leaves the {width}x{height} window"
    );
    let piles = &view["table"]["piles"];
    let covered = piles["piles"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|pile| [&pile["face"], &pile["foot"]])
        .chain(
            piles["discs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| &d["bounds"]),
        );
    for bounds in covered {
        let bounds = rect_of(bounds);
        assert!(
            plate[2] <= bounds[0]
                || bounds[2] <= plate[0]
                || plate[3] <= bounds[1]
                || bounds[3] <= plate[1],
            "the tag {plate:?} covers {bounds:?} of the piles"
        );
    }
    let labels = view["table"]["main_view"]["parts"].as_array().unwrap();
    for label in labels
        .iter()
        .filter(|p| p["part"] == "played-label" || p["part"] == "deck-label")
    {
        assert!(plate[3] <= rect_of(&label["bounds"])[1] + 0.5, "{label}");
    }
}

/// The played pile's tag states the model's list: every card, most recent first, in
/// balanced columns of at most 18, Propaganda cards in bold.
fn assert_played_states_the_model(view: &Value, cards: usize, columns: usize) {
    let model = &view["observation"]["played_pile"];
    let shown = &view["table"]["pile_tag"]["shown"];
    assert_eq!(shown["pile"], "pile:played");
    assert_eq!(shown["ledger"][0][1], model["words"]["cards"]);
    assert_eq!(shown["ledger"][1][1], model["words"]["propaganda"]);
    let listed: Vec<Value> = shown["columns"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c.as_array().unwrap().clone())
        .collect();
    assert_eq!(listed.len(), cards);
    assert_eq!(&Value::Array(listed.clone()), &model["words"]["rows"]);
    assert_eq!(shown["columns"].as_array().unwrap().len(), columns);
    let drawn = parts(view);
    let rows = shown["columns"][0].as_array().unwrap().len();
    for (i, line) in listed.iter().enumerate() {
        let part = format!("column-{}-row-{}", i / rows, i % rows);
        assert_eq!(&drawn[&part].0, line, "{part}");
    }
    let propaganda: Vec<Value> = model["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["propaganda"] == true)
        .map(|c| c["number"].clone())
        .collect();
    assert_eq!(shown["propaganda"], Value::Array(propaganda));
    assert_eq!(drawn["note"].0, model["words"]["note"]);
}

fn assert_deck_states_the_model(view: &Value) {
    let words = &view["observation"]["deck"]["words"];
    let shown = &view["table"]["pile_tag"]["shown"];
    assert_eq!(shown["pile"], "pile:deck");
    let drawn = parts(view);
    for (i, field, part) in [
        (0, "cards", "cards"),
        (1, "face_down", "face-down"),
        (2, "propaganda_left", "propaganda-left"),
        (3, "next_propaganda", "next-propaganda"),
    ] {
        assert_eq!(shown["ledger"][i][1], words[field], "{field}");
        assert_eq!(drawn[part].0, words[field], "{part}");
    }
    assert_eq!(drawn["note"].0, words["note"]);
    let note = words["note"].as_str().unwrap();
    assert!(
        note.contains("assumes the deck was prepared with exactly equal piles of 12"),
        "{note}"
    );
    // Nothing on the tag lists the face-down cards: no card number beside a title.
    for (part, (text, _)) in &drawn {
        if let Some(text) = text.as_str() {
            assert!(!text.contains(" · ") || part == "cards", "{part}: {text}");
        }
    }
}

fn launch(prefix: &str) -> Native {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-144".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket = tempfile::tempdir().unwrap().keep().join("seat.sock");
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into(),
        socket,
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Pile tags evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app
}

/// Hovers a pile and returns the settled view with its tag open, checking that
/// reading it changed nothing in the match.
fn hover(app: &mut Native, pile: &str, before: &Value) -> Value {
    app.point("area:propaganda");
    app.point(pile);
    let shown = app.observe();
    assert_eq!(shown["table"]["hover"], pile);
    assert_eq!(
        shown["table"]["pile_tag"]["shown"]["pile"], pile,
        "{}",
        shown["table"]["pile_tag"]
    );
    assert!(shown["table"]["card"]["shown"].is_null());
    assert_eq!(shown["observation"], before["observation"]);
    assert_placed(&shown);
    shown
}

/// Tab moves focus to `pile`, whose tag opens after the shared dwell.
fn focus(app: &mut Native, pile: &str, before: &Value) -> Value {
    let revision = before["observation"]["revision"].as_u64().unwrap();
    app.point("area:propaganda");
    app.request(json!({"op":"settle"}), true);
    let mut focus = Value::Null;
    for _ in 0..30 {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        // A focused card or pile waits for the shared dwell before its tag opens.
        app.request(json!({"op":"settle"}), true);
        focus = app.observe()["table"]["choice"]["focus"].clone();
        if focus == pile {
            break;
        }
    }
    assert_eq!(focus, pile);
    app.request(json!({"op":"settle"}), true);
    let shown = app.observe();
    assert_eq!(shown["table"]["pile_tag"]["shown"]["pile"], pile);
    assert_eq!(shown["observation"], before["observation"]);
    assert_placed(&shown);
    shown
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_pile_tags_match_layout_and_read_the_model() {
    let mut app = launch("pile-tags-");
    app.request(json!({"op":"settle"}), true);
    let rest = app.observe();
    assert!(rest["table"]["pile_tag"]["shown"].is_null());
    assert!(rest["observation"].get("draw_deck").is_none());

    // Card 17, by hover: 18 cards in one column, and the deck's facts.
    let played = hover(&mut app, "pile:played", &rest);
    assert_played_states_the_model(&played, 18, 1);
    let rows = &played["observation"]["played_pile"]["words"]["rows"];
    assert_eq!(rows[0], "17 · Alberto Bayo · current");
    assert_eq!(rows[5], "49 · Propaganda!");
    assert_matches_layout(&played, "played-card-17", Some("played-pile-tag"));
    app.capture("card-17-played");
    assert_plate_drawn(&app, "card-17-played", &played);
    let deck = hover(&mut app, "pile:deck", &rest);
    assert_deck_states_the_model(&deck);
    assert_eq!(
        deck["observation"]["deck"]["next_propaganda"],
        json!({"earliest":2,"latest":8})
    );
    assert_eq!(
        deck["observation"]["deck"]["words"]["next_propaganda"],
        "within the next 8 cards"
    );
    assert_eq!(deck["observation"]["deck"]["face_down"], 33);
    assert_matches_layout(&deck, "deck-card-17", Some("deck-pile-tag"));
    app.capture("card-17-deck");
    assert_plate_drawn(&app, "card-17-deck", &deck);

    // By keyboard focus, with the same dwell.
    let focused = focus(&mut app, "pile:played", &rest);
    assert_played_states_the_model(&focused, 18, 1);
    let focused = focus(&mut app, "pile:deck", &rest);
    assert_deck_states_the_model(&focused);
    // Focus moves on, past the seat cards to Save · quit; the pointer leaves: no tag.
    let revision = rest["observation"]["revision"].as_u64().unwrap();
    for _ in 0..5 {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        app.request(json!({"op":"settle"}), true);
        assert!(app.observe()["table"]["pile_tag"]["shown"].is_null());
    }
    assert_eq!(app.observe()["table"]["choice"]["focus"], "quit");
    // Story 35, "hover the played pile": a pile is what the owner sees of it, its
    // visible side and its caption as well as its disc. The pointer comes to rest on
    // each from Save · quit, carried there by a middle-button drag,
    // which clicks nothing (the seat presses only where no hover is waiting).
    let main_parts = card_piles::parts(&rest);
    let quit = rest["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|control| control["name"] == "quit")
        .unwrap()["position"]
        .clone();
    let from = [quit[0].as_f64().unwrap(), quit[1].as_f64().unwrap()];
    for (index, (pile, caption)) in [
        ("pile:played", "played-caption"),
        ("pile:deck", "deck-caption"),
    ]
    .into_iter()
    .enumerate()
    {
        let drawn = &rest["table"]["piles"]["piles"][index];
        let face = rect_of(&drawn["face"]);
        let foot = rect_of(&drawn["foot"]);
        let words = main_parts[caption].1;
        // The side's left part (the disc lies on its right), and the caption's middle.
        for (part, to) in [
            ("side", [face[0] + 60., (face[3] + foot[3]) / 2.]),
            (
                "caption",
                [(words[0] + words[2]) / 2., (words[1] + words[3]) / 2.],
            ),
        ] {
            assert!(
                foot[3] - face[3] > 8.,
                "{pile}'s side shows: {face:?} {foot:?}"
            );
            app.point("area:propaganda");
            app.request(json!({"op":"settle"}), true);
            assert!(app.observe()["table"]["pile_tag"]["shown"].is_null());
            app.request(
                json!({"op":"drag","revision":revision,"component":"quit","button":"middle",
                       "x":to[0] - from[0],"y":to[1] - from[1]}),
                true,
            );
            app.request(json!({"op":"settle"}), true);
            let shown = app.observe();
            assert_eq!(shown["table"]["hover"], pile, "{pile} by its {part}");
            assert_eq!(
                shown["table"]["pile_tag"]["shown"]["pile"], pile,
                "{pile} by its {part}"
            );
            assert!(
                shown["table"]["card"]["shown"].is_null(),
                "{pile} by its {part}"
            );
            assert_eq!(shown["observation"], rest["observation"]);
            if part == "side" {
                app.capture(&format!("card-17-{}-from-side", &pile[5..]));
            }
        }
    }
    let untouched = std::fs::read(&app.save).unwrap();
    app.quit();
    assert_eq!(std::fs::read(&app.save).unwrap(), untouched);

    // After the advance: 19 cards in two balanced columns, and one card fewer.
    card_advance::finish_event(&app.save, "shaded");
    app.launch_size("3840x2160");
    app.request(json!({"op":"settle"}), true);
    app.choose("advance-card");
    app.request(json!({"op":"settle"}), true);
    let advanced = app.observe();
    assert_eq!(advanced["observation"]["current_card"], 18);
    let played = hover(&mut app, "pile:played", &advanced);
    assert_played_states_the_model(&played, 19, 2);
    let rows = &played["observation"]["played_pile"]["words"]["rows"];
    assert_eq!(rows[0], "18 · Pact of Caracas · current");
    assert_eq!(rows[1], "17 · Alberto Bayo");
    assert_matches_layout(&played, "played-card-18", None);
    app.capture("card-18-played");
    assert_plate_drawn(&app, "card-18-played", &played);
    let deck = hover(&mut app, "pile:deck", &advanced);
    assert_deck_states_the_model(&deck);
    assert_eq!(
        deck["observation"]["deck"]["words"]["cards"],
        "33 · counts the preview"
    );
    assert_eq!(
        deck["observation"]["deck"]["next_propaganda"],
        json!({"earliest":2,"latest":7})
    );
    assert_eq!(
        deck["observation"]["deck"]["words"]["next_propaganda"],
        "within the next 7 cards"
    );
    // The deck's frame is card 17's; on card 18 only its words differ.
    assert_matches_layout(&deck, "deck-card-17", Some("deck-pile-tag"));
    app.capture("card-18-deck");
    assert_plate_drawn(&app, "card-18-deck", &deck);
    app.quit();
}
