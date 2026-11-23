//! #143: the two piles of cards at the owner's display, against the main view's
//! FigCraft layout record and the rules model, through the development seat.
//!
//! The numbers were worked by hand from the March 2019 Living Rules and the prepared
//! deck before the model was read (`tests/integration/piles.rs` states the working):
//! on card 17 the played pile holds 18 cards with the current card and the draw deck
//! 34 with the preview, one of four Propaganda cards is played, and playing card 18
//! makes them 19 and 33. A pile shows a seam every six to seven cards (the owner's
//! selection in #140): 3 layers for 18 or 19 cards, 5 for 34 or 33. A card is 0.3 mm
//! of 63 mm, 300 / 210 px thick at the layout's 300 px, and half of a pile's height
//! shows as its near side: 12.86 px for 18 cards, 24.29 px for 34.
use super::*;
use std::collections::BTreeMap;

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
}

/// A record rectangle [x, y, width, height] as [left, top, right, bottom].
fn edges(rect: &Value) -> [f64; 4] {
    let [x, y, w, h] = rect_of(rect);
    [x, y, x + w, y + h]
}

/// The main view's drawn parts by name: their words, if any, and their bounds.
fn parts(view: &Value) -> BTreeMap<String, (Value, [f64; 4])> {
    view["table"]["main_view"]["parts"]
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

/// How many layers a pile of `cards` is drawn in: the count divided by 7, rounded,
/// and at least 2 (the owner's selection in #140).
fn layers(cards: u64) -> u64 {
    ((cards as f64 / 7.).round() as u64).max(2)
}

/// The piles at rest: the model's numbers, on the table and on the discs.
pub(super) fn assert_piles(view: &Value, played: u64, deck: u64, current: u64, preview: u64) {
    assert_eq!(played + deck, 52, "the two piles hold the whole deck");
    let model = &view["observation"]["piles"];
    assert_eq!(model["played"], played, "{model}");
    assert_eq!(model["deck"], deck, "{model}");
    assert_eq!(model["propaganda_played"], 1, "{model}");
    assert_eq!(model["propaganda_cards"], 4, "{model}");
    assert_eq!(view["observation"]["current_card"], current);
    assert_eq!(view["observation"]["preview_card"], preview);
    assert_eq!(
        view["table"]["settled"], true,
        "{}",
        view["table"]["pending"]
    );
    let piles = &view["table"]["piles"];
    let drawn = piles["piles"].as_array().unwrap();
    assert_eq!(drawn.len(), 2, "{piles}");
    for (pile, name, card, cards, top) in [
        (&drawn[0], "pile:played", "card:current", played, current),
        (&drawn[1], "pile:deck", "card:preview", deck, preview),
    ] {
        assert_eq!(pile["pile"], name, "{pile}");
        assert_eq!(pile["card"], card, "{pile}");
        assert_eq!(pile["cards"], cards, "{pile}");
        assert_eq!(pile["top"], top, "{pile}");
        assert_eq!(pile["layers"], layers(cards), "{pile}");
    }
    assert_eq!(piles["faces"], 2, "one face-up card on each pile");
    assert_eq!(piles["pips"], json!([true, false, false, false]));
    let parts = parts(view);
    for (part, words) in [
        ("played-label", "CURRENT CARD".to_owned()),
        ("deck-label", "PREVIEW".to_owned()),
        ("played-count", played.to_string()),
        ("deck-count", deck.to_string()),
        ("played-caption", "played pile".to_owned()),
        ("deck-caption", "draw deck".to_owned()),
        ("propaganda-label", "PROPAGANDA".to_owned()),
        ("propaganda-caption", "1 of 4 played".to_owned()),
    ] {
        let drawn = parts
            .get(part)
            .unwrap_or_else(|| panic!("the card area draws no {part}"));
        assert_eq!(drawn.0, words, "{part}");
    }
    // The left pile is the played pile and the right one the draw deck, apart.
    let (left, right) = (rect_of(&drawn[0]["face"]), rect_of(&drawn[1]["face"]));
    assert!(left[2] < right[0], "{left:?} {right:?}");
    // The order of the face-down cards is nowhere in what the seat reads.
    assert!(view["observation"].get("draw_deck").is_none());
    for pile in drawn {
        let mut fields: Vec<&str> = pile
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            ["card", "cards", "face", "foot", "layers", "pile", "top"],
            "a pile tells its count and its top card only"
        );
    }
}

/// The tray, both piles and everything that lies over them against the layout record,
/// within its tolerance: the tray, the faces, the pile's foot and the images on all
/// four edges; text on left, top and right.
fn assert_matches_layout(view: &Value) {
    let layout = record();
    assert_eq!(layout["frame"], json!([3840, 2160]));
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let close = |name: &str, native: [f64; 4], expected: [f64; 4], compared: usize| {
        for (n, e) in native.iter().zip(expected).take(compared) {
            assert!(
                (n - e).abs() <= tolerance,
                "{name}: native {native:?} differs from layout {expected:?} by more than {tolerance} px"
            );
        }
    };
    let piles = &view["table"]["piles"];
    close(
        "card-tray",
        rect_of(&piles["tray"]),
        edges(&layout["tray"]["bounds"]),
        4,
    );
    let drawn = piles["piles"].as_array().unwrap();
    for (pile, face, base) in [
        (&drawn[0], "played-pile", "played-pile-base"),
        (&drawn[1], "draw-deck", "deck-pile-base"),
    ] {
        close(
            face,
            rect_of(&pile["face"]),
            edges(&layout["tray"][face]),
            4,
        );
        // The foot is recorded for card 17's 18 and 34 cards; one card more or less
        // moves it by 0.71 px.
        let foot = rect_of(&pile["foot"]);
        let expected = edges(&layout["tray"][base]);
        for edge in [0, 2, 3] {
            assert!(
                (foot[edge] - expected[edge]).abs() <= tolerance,
                "{base}: native {foot:?} differs from layout {expected:?}"
            );
        }
        // The side shows half the pile's height: 300 / 210 px a card, halved.
        let shown = foot[3] - rect_of(&pile["face"])[3];
        let cards = pile["cards"].as_f64().unwrap();
        assert!(
            (shown - cards * (300. / 210.) / 2.).abs() <= 0.5,
            "{base}: {shown} px of side for {cards} cards"
        );
    }
    let parts = parts(view);
    for (group, compared, names) in [
        (
            "parts",
            4,
            &[
                "played-count-disc",
                "deck-count-disc",
                "propaganda-pip-0",
                "propaganda-pip-1",
                "propaganda-pip-2",
                "propaganda-pip-3",
            ][..],
        ),
        (
            "text",
            3,
            &[
                "played-label",
                "deck-label",
                "played-count",
                "deck-count",
                "played-caption",
                "deck-caption",
                "propaganda-label",
                "propaganda-caption",
            ][..],
        ),
    ] {
        for name in names {
            close(name, parts[*name].1, edges(&layout[group][*name]), compared);
        }
    }
    // The seat's targets: each card is its pile's top card, each pile its count disc.
    let controls: BTreeMap<&str, &Value> = view["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|control| Some((control["name"].as_str()?, control)))
        .collect();
    for name in ["card:current", "card:preview", "pile:played", "pile:deck"] {
        assert!(controls.contains_key(name), "the seat has no {name}");
    }
    for gone in ["reader", "close"] {
        assert!(!controls.contains_key(gone), "{gone} remains");
    }
    let cards = view["table"]["card"]["cards"].as_array().unwrap();
    for (card, pile) in cards.iter().zip(drawn) {
        assert_eq!(card["card"], pile["card"]);
        close(
            "card target",
            rect_of(&card["bounds"]),
            rect_of(&pile["face"]),
            4,
        );
    }
    for (disc, part) in piles["discs"]
        .as_array()
        .unwrap()
        .iter()
        .zip(["played-count-disc", "deck-count-disc"])
    {
        close(part, rect_of(&disc["bounds"]), parts[part].1, 4);
    }
    // The card panel and its text line are gone.
    assert!(!parts.contains_key("card-frame"), "the card panel's frame");
    for (name, (words, _)) in &parts {
        assert!(
            !words.as_str().is_some_and(|w| w.contains("in draw deck")),
            "{name} is the card panel's line: {words}"
        );
    }
}

/// The mean colour of a block of an image around a point.
fn block(image: &image::RgbImage, centre: [f64; 2], half: u32) -> [f64; 3] {
    let (cx, cy) = (centre[0] as u32, centre[1] as u32);
    let mut sum = [0f64; 3];
    let mut count = 0.;
    for x in cx.saturating_sub(half)..(cx + half).min(image.width()) {
        for y in cy.saturating_sub(half)..(cy + half).min(image.height()) {
            for (total, value) in sum.iter_mut().zip(image.get_pixel(x, y).0) {
                *total += f64::from(value);
            }
            count += 1.;
        }
    }
    sum.map(|total| total / count)
}

/// How far the picture in `rect` of a capture is from a card's printed face: the mean
/// difference of block colours over a grid, in levels. The grid keeps off the card's
/// edge and off its bottom-right corner, where the count disc lies.
fn face_distance(capture: &image::RgbImage, rect: [f64; 4], face: &image::RgbImage) -> f64 {
    let (width, height) = (rect[2] - rect[0], rect[3] - rect[1]);
    let mut total = 0.;
    let mut cells = 0.;
    for i in 0..6 {
        for j in 0..9 {
            if i >= 4 && j >= 7 {
                continue;
            }
            let (fx, fy) = ((f64::from(i) + 0.75) / 7.5, (f64::from(j) + 0.75) / 10.5);
            let shown = block(capture, [rect[0] + fx * width, rect[1] + fy * height], 7);
            let printed = block(
                face,
                [fx * f64::from(face.width()), fy * f64::from(face.height())],
                (7. * f64::from(face.width()) / width) as u32,
            );
            total += shown
                .iter()
                .zip(printed)
                .map(|(s, p)| (s - p).abs())
                .sum::<f64>()
                / 3.;
            cells += 1.;
        }
    }
    total / cells
}

/// No pixel in a square is as dark as the black printed outside a face's corners.
fn assert_no_black(capture: &image::RgbImage, left: f64, top: f64, side: u32, what: &str) {
    for x in left as u32..left as u32 + side {
        for y in top as u32..top as u32 + side {
            let pixel = capture.get_pixel(x, y).0;
            assert!(
                pixel.iter().any(|c| *c >= 70),
                "{what}: black pixel {pixel:?} at {x},{y}"
            );
        }
    }
}

/// A pile's paper edge is warmer than the card's green, the sea and the shadow.
fn paper(pixel: [u8; 3]) -> bool {
    pixel[0] >= pixel[1].saturating_add(2) && pixel[1] > pixel[2]
}

/// What a capture shows of the piles: each top card is its own printed face and no
/// other, no corner shows black, and each pile's side is paper as deep as its cards,
/// with a seam for each layer but the last.
pub(super) fn assert_piles_drawn(app: &Native, capture: &str, view: &Value) {
    let shot = image::open(app.root.join(format!("{capture}.png")))
        .unwrap()
        .to_rgb8();
    assert_eq!(shot.dimensions(), (3840, 2160));
    let faces: BTreeMap<u64, image::RgbImage> = [17, 18, 19]
        .into_iter()
        .map(|number| {
            (
                number,
                image::open(app.cards.join(format!("card-{number}.png")))
                    .unwrap()
                    .to_rgb8(),
            )
        })
        .collect();
    for pile in view["table"]["piles"]["piles"].as_array().unwrap() {
        let name = pile["pile"].as_str().unwrap();
        let face = rect_of(&pile["face"]);
        let foot = rect_of(&pile["foot"]);
        let top = pile["top"].as_u64().unwrap();
        let own = face_distance(&shot, face, &faces[&top]);
        assert!(
            own < 8.,
            "{name} in {capture}: {own} levels from card {top}"
        );
        for (other, printed) in &faces {
            if *other != top {
                let far = face_distance(&shot, face, printed);
                assert!(
                    far > 2.5 * own && far > 12.,
                    "{name} in {capture}: card {other} is {far} levels away, card {top} {own}"
                );
            }
        }
        // Three corners of the top card; the count disc lies on the fourth.
        for (corner, x, y) in [
            ("top left", face[0] - 4., face[1] - 4.),
            ("top right", face[2] - 28., face[1] - 4.),
            ("bottom left", face[0] - 4., face[3] - 28.),
            ("foot left", foot[0] - 4., foot[3] - 28.),
        ] {
            assert_no_black(&shot, x, y, 32, &format!("{name} {corner} in {capture}"));
        }
        // Down a column through the near side, a third of the way along the card.
        let x = (face[0] + (face[2] - face[0]) / 3.) as u32;
        let side: Vec<[u8; 3]> = (face[3] as u32 - 2..foot[3] as u32 + 4)
            .map(|y| shot.get_pixel(x, y).0)
            .collect();
        let deep = side.iter().filter(|pixel| paper(**pixel)).count() as f64;
        let cards = pile["cards"].as_f64().unwrap();
        let expected = cards * (300. / 210.) / 2.;
        assert!(
            (deep - expected).abs() <= 2.,
            "{name} in {capture}: {deep} px of paper for {cards} cards, expected {expected}: {side:?}"
        );
        // Each seam's groove catches the light: one bright band for each seam.
        let mut seams = 0;
        let mut bright = false;
        for pixel in &side {
            let lit = paper(*pixel) && pixel[0] >= 210;
            if lit && !bright {
                seams += 1;
            }
            bright = lit;
        }
        assert_eq!(
            seams,
            pile["layers"].as_u64().unwrap() - 1,
            "{name} in {capture}: {side:?}"
        );
        // Beside the pile and below its shadow the sea shows: nothing frames it.
        let sea = shot.get_pixel(face[0] as u32 - 45, face[1] as u32 + 40).0;
        assert!(
            sea[1] > sea[0] && sea.iter().all(|c| *c >= 150),
            "{name} in {capture}: {sea:?} beside the pile is not the sea"
        );
    }
    // Each count stands in the middle of its disc: inside the disc's ring only the
    // number is light.
    for disc in view["table"]["piles"]["discs"].as_array().unwrap() {
        let bounds = rect_of(&disc["bounds"]);
        let centre = [(bounds[0] + bounds[2]) / 2., (bounds[1] + bounds[3]) / 2.];
        let (mut sum, mut light) = ([0f64; 2], 0f64);
        for x in bounds[0] as u32..bounds[2] as u32 {
            for y in bounds[1] as u32..bounds[3] as u32 {
                let from = [
                    f64::from(x) + 0.5 - centre[0],
                    f64::from(y) + 0.5 - centre[1],
                ];
                if from[0].hypot(from[1]) < 33. && shot.get_pixel(x, y).0.iter().all(|c| *c >= 180)
                {
                    sum = [sum[0] + from[0], sum[1] + from[1]];
                    light += 1.;
                }
            }
        }
        assert!(light > 150., "{disc} in {capture}: no number on the disc");
        let off = [sum[0] / light, sum[1] / light];
        assert!(
            off[0].abs() <= 3. && off[1].abs() <= 4.,
            "{disc} in {capture}: the number is {off:?} px off the disc's centre"
        );
    }
}

fn launch(prefix: &str) -> Native {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-143".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    // The socket's directory outlives the test, like the evidence beside it.
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
    eprintln!("Card piles evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app
}

fn settled(app: &mut Native) -> Value {
    app.request(json!({"op":"settle"}), true);
    app.observe()
}

/// Hovering each card on its pile opens its tag, which stays inside the window and
/// off both piles; its face has no black corner. Reading changes nothing.
fn assert_tags_open(app: &mut Native, name: &str, before: &Value) {
    let revision = before["observation"]["revision"].as_u64().unwrap();
    for (slot, key) in [("card:current", "current"), ("card:preview", "preview")] {
        app.point("quit");
        app.point(slot);
        let shown = app.observe();
        assert_eq!(shown["table"]["hover"], slot);
        assert_card_tag_placed(&shown, slot);
        let card = &shown["table"]["card"];
        assert_eq!(
            card["shown"]["number"], before["observation"]["cards"][key]["number"],
            "{card}"
        );
        assert_eq!(shown["observation"], before["observation"]);
        assert_eq!(shown["table"]["piles"], before["table"]["piles"]);
        let capture = format!("{name}-tag-{key}");
        app.capture(&capture);
        let shot = image::open(app.root.join(format!("{capture}.png")))
            .unwrap()
            .to_rgb8();
        let face = card["parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|part| part["part"] == "face")
            .map(|part| rect_of(&part["bounds"]))
            .unwrap();
        for (corner, x, y) in [
            ("top left", face[0], face[1]),
            ("top right", face[2] - 40., face[1]),
            ("bottom left", face[0], face[3] - 40.),
            ("bottom right", face[2] - 40., face[3] - 40.),
        ] {
            assert_no_black(&shot, x, y, 40, &format!("{capture} tag face {corner}"));
        }
        // The piles are still drawn under the open tag's window.
        assert_piles_drawn(app, &capture, &shown);
        close_card_tag(app);
    }
    // Keyboard focus opens a card's tag as the pointer does.
    app.point("quit");
    let mut focus = Value::Null;
    for _ in 0..24 {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        // A focused card waits for the shared dwell before its tag opens.
        app.request(json!({"op":"settle"}), true);
        focus = app.observe()["table"]["choice"]["focus"].clone();
        if focus == "card:preview" {
            break;
        }
    }
    assert_eq!(focus, "card:preview");
    let focused = settled(app);
    assert_card_tag_placed(&focused, "card:preview");
    assert_eq!(focused["observation"], before["observation"]);
    close_card_tag(app);
    // Move focus off the card, so its tag can open again later. It moves to the played
    // pile, whose tag waits for the shared dwell (#144).
    app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
    app.request(json!({"op":"settle"}), true);
    app.point("quit");
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_card_piles_match_layout_and_follow_the_match() {
    let mut app = launch("card-piles-");
    let rest = settled(&mut app);
    let revision = rest["observation"]["revision"].as_u64().unwrap();

    // Card 17: 18 played with it, 34 in the deck with the preview, 1 of 4 Propaganda.
    assert_piles(&rest, 18, 34, 17, 18);
    assert_matches_layout(&rest);
    app.capture("card-17");
    assert_piles_drawn(&app, "card-17", &rest);

    // The tray's camera is a second 3D view that is always there. The piece counts
    // still follow the map's camera: each stands over the island, not where an
    // unplaced node lies, at the window's corner.
    let island = rect_of(&rest["table"]["main_view"]["island"]);
    let counts = rest["table"]["counts"].as_array().unwrap();
    assert!(counts.len() >= 10, "{counts:?}");
    for count in counts {
        let bounds = rect_of(&count["bounds"]);
        let centre = [(bounds[0] + bounds[2]) / 2., (bounds[1] + bounds[3]) / 2.];
        assert!(
            centre[0] > island[0] - 60.
                && centre[0] < island[2] + 60.
                && centre[1] > island[1] - 60.
                && centre[1] < island[3] + 60.,
            "{count} is not over the island {island:?}"
        );
    }

    // The tray is fixed to the screen: the map camera moves under it.
    app.request(json!({"op":"key","key":"KeyW","revision":revision}), true);
    app.request(json!({"op":"key","key":"KeyA","revision":revision}), true);
    app.point("space:camaguey-city");
    app.request(
        json!({"op":"scroll","revision":revision,"component":"space:camaguey-city","y":2}),
        true,
    );
    app.request(
        json!({"op":"drag","revision":revision,"component":"space:camaguey-city","button":"right","x":-60,"y":-20}),
        true,
    );
    app.point("quit");
    let moved = settled(&mut app);
    assert_ne!(moved["table"]["camera"], rest["table"]["camera"]);
    assert_eq!(moved["table"]["piles"], rest["table"]["piles"]);
    app.capture("card-17-camera-moved");
    for pile in rest["table"]["piles"]["piles"].as_array().unwrap() {
        let face = rect_of(&pile["face"]);
        assert_eq!(
            changes_within(
                &app.root.join("card-17.png"),
                &app.root.join("card-17-camera-moved.png"),
                face
            ),
            0,
            "{} moved with the map camera",
            pile["pile"]
        );
    }
    assert_piles_drawn(&app, "card-17-camera-moved", &moved);
    app.request(json!({"op":"key","key":"Home","revision":revision}), true);
    let home = settled(&mut app);
    assert_eq!(home["table"]["camera"], rest["table"]["camera"]);

    // Both cards read from their piles, by hover and by focus.
    assert_tags_open(&mut app, "card-17", &rest);

    // Each pile is a target of its own. Resting on it opens its tag, not a card's
    // (#144, `pile_tags`); pressing it changes nothing.
    for pile in ["pile:played", "pile:deck"] {
        app.point(pile);
        let rested = app.observe();
        assert!(rested["table"]["card"]["shown"].is_null(), "{pile}");
        assert_eq!(rested["table"]["hover"], pile);
        assert_eq!(rested["table"]["pile_tag"]["shown"]["pile"], pile);
        app.activate(pile, revision);
        let pressed = app.observe();
        assert_eq!(pressed["observation"], rest["observation"], "{pile}");
        assert!(pressed["table"]["card"]["shown"].is_null(), "{pile}");
        assert_eq!(
            pressed["table"]["camera"], rest["table"]["camera"],
            "{pile}"
        );
    }
    let untouched = std::fs::read(&app.save).unwrap();
    app.quit();

    // A restart before the advance shows the same piles at rest.
    app.launch_size("3840x2160");
    let again = settled(&mut app);
    assert_eq!(again["observation"], rest["observation"]);
    assert_eq!(again["table"]["piles"], rest["table"]["piles"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), untouched);
    app.capture("card-17-restarted");
    assert_piles_drawn(&app, "card-17-restarted", &again);
    app.quit();

    // Card 17 played to its end; the piles have not changed, since no card has.
    card_advance::finish_event(&app.save, "shaded");
    app.launch_size("3840x2160");
    let ended = settled(&mut app);
    assert_piles(&ended, 18, 34, 17, 18);
    assert_eq!(ended["table"]["piles"], rest["table"]["piles"]);

    // The advance (R 2.3.8): card 18 lies on the played pile and card 19 is revealed.
    app.choose("advance-card");
    let advanced = settled(&mut app);
    assert_piles(&advanced, 19, 33, 18, 19);
    assert_matches_layout(&advanced);
    app.capture("card-18");
    assert_piles_drawn(&app, "card-18", &advanced);
    assert_tags_open(&mut app, "card-18", &advanced);
    let saved = std::fs::read(&app.save).unwrap();
    app.quit();

    // A restart after the advance shows the saved cards at rest.
    app.launch_size("3840x2160");
    let resumed = settled(&mut app);
    assert_eq!(resumed["observation"], advanced["observation"]);
    assert_eq!(resumed["table"]["piles"], advanced["table"]["piles"]);
    assert_piles(&resumed, 19, 33, 18, 19);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("card-18-restarted");
    assert_piles_drawn(&app, "card-18-restarted", &resumed);
    for pile in resumed["table"]["piles"]["piles"].as_array().unwrap() {
        assert_eq!(
            changes_within(
                &app.root.join("card-18.png"),
                &app.root.join("card-18-restarted.png"),
                rect_of(&pile["face"])
            ),
            0,
            "{} differs after the restart",
            pile["pile"]
        );
    }
    app.quit();
}
