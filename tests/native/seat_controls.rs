//! #206: the seat controls in the window, through the development seat: the cards, the
//! line sheet and the strip against their layout record, each state a seat shows, and
//! handing over and taking back by pointer and keyboard at any time. The sessions are the
//! real client's and run no model (`faction_seats`).
use super::{
    available_forces::start,
    faction_seats::{
        DIRECTORIO, GOVERNMENT, M26, SYNDICATE, assert_by_hand, decisions, holding, seat,
    },
    *,
};
use std::time::{Duration, Instant};

const FACTIONS: [(&str, &str); 4] = [
    ("government", "Government"),
    ("26july", "M26"),
    ("directorio", "Directorio"),
    ("syndicate", "Syndicate"),
];

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-seats.layout.json")).unwrap()
}

/// The drawn seat parts by name: each one's words and its rectangle.
fn parts(view: &Value) -> std::collections::BTreeMap<String, (Value, [f64; 4])> {
    view["table"]["seat_cards"]["parts"]
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

/// What `name` says, when it is drawn.
fn says(view: &Value, name: &str) -> Option<String> {
    parts(view)
        .get(name)
        .map(|(text, _)| text.as_str().unwrap().to_owned())
}

/// What a faction's card says of its seat.
fn state_words(view: &Value, slug: &str) -> String {
    says(view, &format!("seat:{slug}-state")).unwrap()
}

/// The shown controls whose names start with `prefix`.
fn controls(view: &Value, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = view["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|control| control["name"].as_str().unwrap().to_owned())
        .filter(|name| name.starts_with(prefix))
        .collect();
    names.sort();
    names
}

impl Native {
    /// Presses a seat control with the pointer, at whatever the table is doing.
    fn press(&mut self, control: &str) -> Value {
        let revision = self.revision();
        self.request(
            json!({"op":"activate","revision":revision,"component":control}),
            true,
        );
        self.settle()
    }
    /// Tabs to `control` and presses Enter on it.
    pub(super) fn press_by_keyboard(&mut self, control: &str) -> Value {
        let revision = self.revision();
        for _ in 0..60 {
            if self.observe()["table"]["choice"]["focus"] == control {
                self.request(
                    json!({"op":"key","revision":revision,"key":"Enter","focused":control}),
                    true,
                );
                return self.settle();
            }
            self.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
            self.settle();
        }
        panic!("Tab does not reach {control}");
    }
    /// Joins as `faction`, which is handed over: the session's identity.
    fn join(&mut self, faction: &str) -> String {
        let (code, joined) = self.ask(&["join", faction]);
        assert_eq!((code, &joined["status"]), (0, &json!("joined")), "{joined}");
        joined["player"].as_str().unwrap().to_owned()
    }
}

fn disjoint(a: [f64; 4], b: [f64; 4]) -> bool {
    a[2] <= b[0] || a[0] >= b[2] || a[3] <= b[1] || a[1] >= b[3]
}

/// `open`, where the slip or Save · quit's tag is drawn, covers no drawn part of a seat
/// card, no other area, the decision panel's and Save · quit's included, and no control
/// the pointer could press: the owner's ruling of 2026-11-25.
fn clear_of_the_cards(view: &Value, what: &str, open: [f64; 4]) {
    let cards = parts(view);
    for (slug, _) in FACTIONS {
        assert!(cards.contains_key(&format!("seat:{slug}")), "{slug}");
    }
    for (name, (_, at)) in &cards {
        assert!(
            disjoint(open, *at),
            "the {what} {open:?} covers {name} {at:?}"
        );
    }
    let areas = view["table"]["main_view"]["areas"].as_object().unwrap();
    for (name, at) in areas {
        let at = rect_of(at);
        assert!(
            disjoint(open, at),
            "the {what} {open:?} covers {name} {at:?}"
        );
    }
    for control in view["table"]["controls"].as_array().unwrap() {
        let name = control["name"].as_str().unwrap();
        let [x, y]: [f64; 2] = serde_json::from_value(control["position"].clone()).unwrap();
        assert!(
            name == "notice-slip" || disjoint(open, [x, y, x, y]),
            "the {what} {open:?} covers {name} at {x}, {y}"
        );
    }
}

/// The notice slip and Save · quit's tag stand above the seat cards in every window the
/// regressions use, with Syndicate's card, the nearest, forward and not (the owner's
/// ruling of 2026-11-25).
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_the_slip_and_save_quits_tag_clear_the_seat_cards() {
    use cuba_libre_unfolded::save::Session;
    let mut app = super::available_forces::unlaunched("slip-clear-", "issue-206");
    let opening = app.save.clone();
    let forward = app.root.join("syndicate-decides.json");
    drop(Session::create_seeded(&opening, Session::KIDNAP_ROLLS_1).unwrap());
    let mut session = Session::create_seeded(&forward, Session::KIDNAP_ROLLS_1).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
        "end-assault",
    ] {
        let view = session.observe();
        let choice = view.choices.iter().find(|c| c.action() == action);
        let choice = choice.unwrap_or_else(|| panic!("{action} is not offered"));
        session
            .submit(choice.owner, view.revision, &choice.id)
            .unwrap();
    }
    drop(session);
    for size in [
        "3840x2160",
        "4096x2160",
        "3440x1440",
        "2560x1080",
        "2520x1080",
        "2160x1440",
        "1920x1280",
        "1920x1080",
        "1600x1280",
        "1440x960",
    ] {
        for (save, owner) in [(&opening, "M26"), (&forward, "Syndicate")] {
            if owner == "M26" && size != "3840x2160" {
                continue;
            }
            app.save = save.clone();
            app.launch_as_is(size);
            let view = app.settle();
            assert_eq!(view["observation"]["owner"], owner);
            assert_eq!(
                parts(&view).contains_key("seat:syndicate-edge"),
                owner == "Syndicate",
                "{size}: which card stands forward"
            );
            // A socket where another table answers refuses the hand-over, as an error.
            let other = std::os::unix::net::UnixListener::bind(app.player_socket()).unwrap();
            let view = app.press("seat:government:hand-over");
            assert_eq!(view["table"]["slip"]["kind"], "error", "{size}");
            let slip = rect_of(&view["table"]["slip"]["bounds"]);
            clear_of_the_cards(&view, "slip", slip);
            app.capture(&format!("slip-{owner}-{size}"));
            drop(other);
            std::fs::remove_file(app.player_socket()).unwrap();
            app.activate(
                "notice-slip",
                view["observation"]["revision"].as_u64().unwrap(),
            );

            // Directorio's and Syndicate's line sheets reach past the slip's left edge, and
            // the slip stands in front of them, so its words are read.
            for seat in ["directorio", "syndicate"] {
                if owner == "Syndicate" {
                    break;
                }
                app.press(&format!("seat:{seat}:hand-over"));
                app.press(&format!("seat:{seat}:copy"));
                app.activate("option:rally", 0);
                app.activate("choice-commit", 0);
                let view = app.settle();
                let [left, _, _, foot] = rect_of(&view["table"]["slip"]["bounds"]);
                let sheet = parts(&view)["seat-line"].1;
                assert!(sheet[0] < left && left < sheet[2] && foot < sheet[3]);
                let name = format!("slip-before-the-sheet-of-{seat}");
                app.capture(&name);
                let capture = image::open(app.root.join(format!("{name}.png"))).unwrap();
                let capture = capture.to_rgb8();
                let edge = (0..3).any(|x| {
                    let pixel = capture.get_pixel(left as u32 + x, foot as u32 - 60);
                    pixel.0.iter().map(|&c| u32::from(c)).sum::<u32>() < 560
                });
                assert!(edge, "{seat}'s sheet covers the slip's left edge");
                app.activate("notice-slip", 0);
                app.press(&format!("seat:{seat}:take-back"));
            }

            let view = app.point("quit");
            assert_eq!(view["table"]["top"]["item"], "save-quit", "{size}");
            let tag = rect_of(&view["table"]["top"]["bounds"]);
            clear_of_the_cards(&view, "tag", tag);
            app.capture(&format!("tag-{owner}-{size}"));
            // Both keep Save · quit's right edge, above it.
            let plate = rect_of(&view["table"]["main_view"]["areas"]["save-quit"]);
            for (what, open) in [("slip", slip), ("tag", tag)] {
                assert!((open[2] - plate[2]).abs() <= 1., "{size}: {what} {open:?}");
                assert!(open[3] < plate[1], "{size}: {what} {open:?}");
            }
            app.quit();
        }
    }
}

/// The selected layout's own position: Directorio's agent thinks, Government waits for
/// one with its line open, Syndicate's is at the table and the human plays 26July.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_the_seat_controls_match_the_selected_layout() {
    let mut app = start("seat-layout-", "issue-206", "3840x2160");
    // Every faction is the human's: 26July decides, its card forward.
    let view = app.settle();
    for (slug, name) in FACTIONS {
        assert_eq!(seat(&view, name)["state"], "human");
        assert_eq!(state_words(&view, slug), "You play it");
        assert_eq!(
            controls(&view, &format!("seat:{slug}:")),
            [format!("seat:{slug}:hand-over")]
        );
        assert_eq!(
            parts(&view).contains_key(&format!("seat:{slug}-edge")),
            slug == "26july",
            "the deciding faction's card stands forward"
        );
    }
    app.capture("every-faction-the-humans");
    // A refusal's slip opens above the cards.
    app.activate("option:rally", 0);
    app.activate("choice-commit", 0);
    let view = app.settle();
    clear_of_the_cards(&view, "slip", rect_of(&view["table"]["slip"]["bounds"]));
    app.capture("the-slip-over-the-seats");
    app.activate("notice-slip", 0);

    for action in ["terror", "kidnap"] {
        app.choose(action);
    }
    app.roll();
    for action in ["terror:santiago", "end-terror"] {
        app.choose(action);
    }
    app.press("seat:directorio:hand-over");
    app.press("seat:syndicate:hand-over");
    app.join("syndicate");
    let directorio = app.join("directorio");
    let (_, told) = app.ask(&["wait", &directorio]);
    assert_eq!(told["status"], "decision");
    app.press("seat:government:hand-over");
    let view = app.press("seat:government:copy");
    app.point("area:standings");
    let view = {
        assert_eq!(view["table"]["seat_cards"]["line"]["faction"], "Government");
        app.settle()
    };
    app.capture("selected-layout");

    // Every part of the frame is drawn where the frame draws it, and no other.
    let layout = record();
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let drawn = parts(&view);
    let rects = |group: &str| layout[group].as_object().unwrap().clone();
    for (name, rect) in rects("parts") {
        let [x, y, w, h] = rect_of(&rect);
        let (_, native) = &drawn[&name];
        for (n, e) in native.iter().zip([x, y, x + w, y + h]) {
            assert!(
                (n - e).abs() <= tolerance,
                "{name}: {native:?}, layout {rect}"
            );
        }
    }
    for (name, rect) in rects("text") {
        let [x, y, w, _] = rect_of(&rect);
        let (text, native) = &drawn[&name];
        assert!(
            (native[0] - x).abs() <= tolerance && (native[1] - y).abs() <= tolerance,
            "{name}: {native:?}, layout {rect}"
        );
        // Centred text spans its box; a left-aligned box is its node's, not its words'.
        if name.ends_with("-label") || name.ends_with("-hint") {
            assert!((native[2] - x - w).abs() <= tolerance, "{name}: {native:?}");
        }
        assert!(text.is_string(), "{name}");
    }
    let named = |group: &str| rects(group).into_iter().map(|(name, _)| name);
    let recorded: std::collections::BTreeSet<String> =
        named("parts").chain(named("text")).collect();
    assert_eq!(
        drawn
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        recorded
    );
    // The words are the frame's, with this table's own line.
    for (name, words) in [
        ("seat:government-state", "Awaiting an agent"),
        ("seat:26july-state", "You play it"),
        ("seat:directorio-state", "Thinking"),
        ("seat:syndicate-state", "Agent at the table"),
        ("seat-strip-title", "Directorio: Thinking"),
        ("seat-strip-note", "Its agent holds the choice."),
        (
            "seat-strip-hint",
            "Hover an option for its briefing. Only Directorio's seat can choose.",
        ),
        (
            "seat-line-title",
            "Paste this in a terminal, in a Claude Code or Codex session",
        ),
        ("seat-line-note", "Copied. One session plays one seat."),
    ] {
        assert_eq!(says(&view, name).unwrap(), words);
    }
    let line = says(&view, "seat-line-text").unwrap();
    assert!(line.contains("Government"), "{line}");
    assert!(
        line.contains(app.player_socket().to_str().unwrap()),
        "{line}"
    );
    assert_eq!(view["table"]["faction_seats"]["copied"], line.as_str());
    app.offers_nothing();

    // The panel keeps its accepted places, its rows in the regular type of the record.
    let kept: std::collections::BTreeMap<String, [f64; 4]> = view["table"]["choice_parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| {
            (
                part["part"].as_str().unwrap().to_owned(),
                rect_of(&part["bounds"]),
            )
        })
        .collect();
    for (name, rect) in rects("panel") {
        let [x, y, w, h] = rect_of(&rect);
        let native = kept[&name];
        let edges = [x, y, x + w, y + h];
        let image = name.ends_with("-emblem") || name.starts_with("pill:");
        for edge in 0..if image { 4 } else { 2 } {
            assert!(
                (native[edge] - edges[edge]).abs() <= tolerance,
                "{name}: {native:?}, layout {rect}"
            );
        }
    }
    // The rows are read with their briefings, and
    // the space tags, the cards and the standings as at any time.
    let view = app.point("option:event:unshaded");
    assert_eq!(view["table"]["briefing"]["option"], "event:unshaded");
    app.capture("an-option-read");
    let view = app.point("option:directorio:limited-operation");
    assert_eq!(
        view["table"]["briefing"]["option"],
        "directorio:limited-operation"
    );
    let view = app.point("space:havana");
    assert_eq!(view["table"]["status"]["space"], "havana");
    let view = app.point("card:current");
    assert!(
        view["table"]["card"]["shown"].is_object(),
        "{}",
        view["table"]["card"]
    );
    let view = app.point("standing:0");
    assert!(view["table"]["victory"].is_object());

    // Escape puts the sheet away; so does a click on the map.
    let view = app.key("Escape");
    assert_eq!(view["table"]["seat_cards"]["line"], Value::Null);
    app.press("seat:government:copy");
    let view = app.press("space:las-villas");
    assert_eq!(view["table"]["seat_cards"]["line"], Value::Null);
    // Each faction's line is its own, over its own card and inside the row; the sheet
    // goes when the seat is joined.
    app.press("seat:syndicate:take-back");
    app.press("seat:syndicate:hand-over");
    let view = app.press("seat:syndicate:copy");
    let copied = view["table"]["faction_seats"]["copied"].as_str().unwrap();
    assert!(copied.contains("Syndicate") && !copied.contains("Government"));
    let drawn = parts(&view);
    let (sheet, card) = (drawn["seat-line"].1, drawn["seat:syndicate"].1);
    assert!(
        (sheet[2] - card[2]).abs() <= tolerance,
        "{sheet:?} {card:?}"
    );
    app.capture("syndicates-line");
    app.press("seat-line:copy");
    app.join("syndicate");
    let view = app.settle();
    assert_eq!(view["table"]["seat_cards"]["line"], Value::Null);
    app.quit();
}

/// Each state on the card and the strip, the silent one from a session whose wait died;
/// a seat nobody holds never acts; taking back with a choice held; the build's end.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_each_seat_state_is_shown_and_a_held_choice_is_taken_back() {
    let mut app = start("seat-states-", "issue-206", "3840x2160");
    // All four are handed over by the pointer, and the human still works the table.
    for (slug, name) in FACTIONS {
        let view = app.press(&format!("seat:{slug}:hand-over"));
        assert_eq!(seat(&view, name)["state"], "awaiting");
        assert_eq!(state_words(&view, slug), "Awaiting an agent");
    }
    let view = app.settle();
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "26July: Awaiting an agent"
    );
    assert_eq!(
        says(&view, "seat-strip-note").unwrap(),
        "No agent has joined yet."
    );
    app.offers_nothing();
    let saved = app.save_bytes();
    let before = view["observation"].clone();
    // Nobody holds them: they wait and nothing acts, however long.
    for _ in 0..20 {
        app.request(json!({"op":"advance","steps":60}), true);
    }
    let view = app.settle();
    assert_eq!(
        (view["observation"].clone(), app.save_bytes()),
        (before, saved)
    );
    let camera = view["table"]["camera"].clone();
    app.point("space:las-villas");
    let zoomed = app.request(
        json!({"op":"scroll","revision":0,"component":"space:las-villas","y":4}),
        true,
    );
    assert_ne!(zoomed["table"]["camera"], camera);
    app.key("Home");
    let view = app.point("option:terror");
    assert_eq!(view["table"]["briefing"]["option"], "terror");
    app.capture("all-four-handed-over");

    // 26July's agent joins and thinks; its wait is told, and it plays to the handoff.
    let m26 = app.join("26july");
    let view = app.settle();
    assert_eq!(state_words(&view, "26july"), "Thinking");
    assert_eq!(says(&view, "seat-strip-title").unwrap(), "26July: Thinking");
    assert_eq!(state_words(&view, "government"), "Awaiting an agent");
    let directorio = app.join("directorio");
    let view = app.settle();
    assert_eq!(state_words(&view, "directorio"), "Agent at the table");
    for action in ["terror", "kidnap", "kidnap:roll"] {
        app.decide(&m26, action, "applied");
        app.settle();
    }
    app.capture("an-agents-terror");
    // Directorio's session waits, and its command dies before the decision comes: its seat
    // is not left thinking, and its silence counts from the death.
    let mut dead = app.player("d", &["wait", &directorio]);
    app.until_waiting("Directorio");
    dead.0.kill().unwrap();
    dead.0.wait().unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    while seat(&app.settle(), "Directorio")["waits"] == true {
        assert!(Instant::now() < end, "a dead client's wait is held");
    }
    let since = Instant::now();
    for action in ["terror:santiago", "end-terror"] {
        // The landed die's result holds the next choice for a while.
        let revision = app.revision().to_string();
        let id = format!("{revision}:{action}");
        app.ask(&["choose", &m26, &revision, &id]);
        app.settle();
        assert_eq!(decisions(&app.save).last().unwrap(), action);
    }
    // 26July's edition is put away.
    app.key("Escape");

    let view = app.settle();
    assert_eq!(state_words(&view, "directorio"), "Thinking");
    std::thread::sleep(Duration::from_secs(61).saturating_sub(since.elapsed()));
    let view = app.settle();
    assert_eq!(seat(&view, "Directorio")["state"], "silent");
    assert_eq!(state_words(&view, "directorio"), "Silent · 1 min");
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "Directorio: Silent · 1 min"
    );
    assert_eq!(
        says(&view, "seat-strip-note").unwrap(),
        "Its choice is still open to it."
    );
    // 26July's session is as silent by now, with nothing asked of it.
    assert_eq!(state_words(&view, "26july"), "Agent at the table");
    app.capture("silent");
    // Nothing replaces it: the decision is still Directorio's, and its next word ends
    // the silence.
    assert_eq!(decisions(&app.save).last().unwrap(), "end-terror");
    app.ask(&["observe", &directorio]);
    assert_eq!(state_words(&app.settle(), "directorio"), "Thinking");

    // The human reads a pinned briefing, which holds its choice, and takes it back on the
    // strip: nothing was applied, and the panel offers the same options to the human.
    let revision = app.revision();
    app.activate("option:event:shaded", revision);
    app.decide(&directorio, "event:unshaded", "held");
    let view = app.settle();
    assert_eq!(holding(&view), ["a pinned or opened briefing"]);
    let view = app.press("seat-strip:take-back");
    assert_eq!(seat(&view, "Directorio")["state"], "human");
    assert_eq!(view["observation"]["revision"], revision);
    assert_eq!(says(&view, "seat-strip-title"), None);
    let (_, told) = app.ask(&["observe", &directorio]);
    assert_eq!(told["status"], "closed");
    assert!(told["reason"].as_str().unwrap().contains("was not applied"));
    app.key("Escape");
    assert_eq!(app.settle()["observation"]["revision"], revision);
    for choice in ["event:shaded", "resolve-shaded"] {
        let revision = app.revision();
        app.activate(&format!("option:{choice}"), revision);
        app.choose(choice);
    }

    // Turning the card is the human's, whoever plays its factions. With the card in the
    // air Government is taken back at once, by a click that also lands it as any does.
    let revision = app.revision();
    let turn = format!("{revision}:advance-card");
    app.press("seat:directorio:hand-over");
    app.request(
        json!({"op":"activate","revision":revision,"choice":turn}),
        true,
    );
    let view = app.observe();
    assert!(view["pending"].to_string().contains("card advance"));
    app.request(
        json!({"op":"activate","revision":revision + 1,"component":"seat:government:take-back"}),
        true,
    );
    assert_eq!(seat(&app.observe(), "Government")["state"], "human");
    app.press("seat:government:hand-over");

    // Government's agent opens Assault and removal mode, whose pieces and spaces are read
    // on the map without a button. It ends the Operation and hands off to Syndicate.
    let government = app.join("government");
    app.decide(&government, "government:assault", "applied");
    let view = app.settle();
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "Government: Thinking"
    );
    app.offers_nothing();
    app.capture("an-agents-assault");
    app.decide(&government, "remove", "applied");
    let revision = app.revision();
    app.activate("piece:santiago Government Troops Ordinary", revision);
    let view = app.observe();
    let piece = &view["table"]["popover"];
    assert_eq!(piece["piece"], "santiago:troops", "{piece}");
    assert!(piece["card"]["rows"][0]["button"].is_null(), "{piece}");
    app.offers_nothing();
    app.capture("an-agents-removal");
    // Closing the popover lets the choice it holds be applied.
    app.decide(&government, "done-removal", "held");
    app.activate("piece:santiago Government Troops Ordinary", revision);
    app.settle();
    assert_eq!(decisions(&app.save).last().unwrap(), "done-removal");
    app.decide(&government, "end-assault", "applied");
    app.settle();
    app.key("Escape");
    let syndicate = app.join("syndicate");
    app.decide(&syndicate, "syndicate:pass", "applied");
    let view = app.key("Escape");
    assert_eq!(view["observation"]["current_card"], 18);
    assert_eq!(seat(&view, "Syndicate")["state"], "at-table");
    assert_eq!(state_words(&view, "syndicate"), "Agent at the table");
    assert_eq!(says(&view, "seat-strip-title"), None);
    assert_eq!(
        choice_part(&view, "advance-card-label")["text"],
        "Advance to Sierra Maestra Manifesto"
    );
    let saved = app.save_bytes();
    let refused = app.decide(&syndicate, "advance-card", "refused");
    assert_eq!(
        refused["error"],
        "The human turns the card; it is no agent's choice."
    );
    assert_eq!(refused["view"]["decision"]["stands"], "card");
    assert_eq!(refused["view"]["decision"]["yours"], false);
    assert_eq!(refused["view"]["choices"], json!([]));
    assert_eq!(app.save_bytes(), saved);
    let mut syndicate_wait = app.player("s", &["wait", &syndicate]);
    app.until_waiting("Syndicate");
    app.capture("human-advance-while-syndicate-waits");
    app.choose("advance-card");
    assert_eq!(app.observe()["observation"]["current_card"], 19);
    assert!(syndicate_wait.ended().is_none());

    let directorio = app.join("directorio");
    app.press("seat:26july:take-back");
    for choice in ["m26:event", "manifesto:decline"] {
        let revision = app.revision();
        app.activate(&format!("option:{choice}"), revision);
        app.choose(choice);
    }
    app.decide(&directorio, "manifesto:decline", "applied");
    let (code, woke) = app.answered(&mut syndicate_wait);
    assert_eq!((code, &woke["status"]), (0, &json!("decision")), "{woke}");
    assert_eq!(woke["view"]["decision"]["faction"], "Syndicate");
    assert_eq!(woke["view"]["decision"]["yours"], true);
    app.decide(&syndicate, "manifesto:decline", "applied");
    app.decide(&government, "manifesto:decline", "applied");

    let view = app.settle();
    assert_eq!(view["observation"]["owner"], "Directorio");
    assert_eq!(seat(&view, "Directorio")["state"], "build-ends");
    assert_eq!(state_words(&view, "directorio"), "Build ends here");
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "Directorio: Build ends here"
    );
    assert_eq!(
        says(&view, "seat-strip-note").unwrap(),
        "No option here is in this build."
    );
    assert_eq!(controls(&view, "seat-strip:"), [""; 0]);
    app.offers_nothing();
    app.capture("build-ends-here");

    // Save · quit ends the session that waits, and after a relaunch every faction is the
    // human's again.
    let mut waiting = app.player("g", &["wait", &government]);
    app.until_waiting("Government");
    let revision = app.revision();
    let quit = app.request(
        json!({"op":"activate","revision":revision,"component":"quit"}),
        true,
    );
    assert_eq!(quit["status"], "exited", "{quit}");
    let code = waiting.0.wait().unwrap().code();
    let (_, answer) = waiting.ended().unwrap();
    assert_eq!(
        (code, &answer["status"]),
        (Some(0), &json!("closed")),
        "{answer}"
    );
    assert_by_hand(&app.save);
    app.launch_as_is("3840x2160");
    let view = app.settle();
    assert_eq!(view["observation"]["revision"], revision);
    for (slug, name) in FACTIONS {
        assert_eq!(seat(&view, name)["state"], "human");
        assert_eq!(state_words(&view, slug), "You play it");
    }
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_terror_withdrawal_follows_its_choice_owner() {
    let mut app = start("terror-withdrawal-", "issue-250", "3840x2160");
    for action in ["terror", "transfer", "ask:0:1", "wait"] {
        app.choose(action);
    }
    let revision = app.revision();
    assert_eq!(revision, 4);
    let before = app.save_bytes();

    app.press("seat:26july:hand-over");
    for action in ["back-to-choice", "end-terror"] {
        let pinned = app.press_by_keyboard(&format!("step:{action}"));
        assert_eq!(pinned["observation"]["owner"], "Government");
        assert_eq!(pinned["table"]["choice"]["pinned"], action);
        assert_eq!(
            choice_part(&pinned, &format!("step:{action}-note"))["text"],
            "26July's seat"
        );
        assert_eq!(
            choice_part(&pinned, "execution-commit-label")["text"],
            "26July's seat chooses"
        );
        assert!(controls(&pinned, &format!("{revision}:{action}")).is_empty());
        assert!(
            pinned["table"]["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["name"] == "execution-commit" && c["choice"] == false)
        );
        assert_eq!(app.save_bytes(), before);
        app.capture(&format!("held-{action}"));
        let pointer = app.press("execution-commit");
        assert_eq!(pointer["observation"], pinned["observation"]);
        assert_eq!(app.save_bytes(), before);
        let keyboard = app.press_by_keyboard("execution-commit");
        assert_eq!(
            keyboard["table"]["notice"],
            "Only 26July's seat can choose this option."
        );
        assert_eq!(keyboard["observation"], pinned["observation"]);
        assert_eq!(app.save_bytes(), before);
    }

    app.choose("reopen");
    assert_eq!(app.observe()["table"]["transfer"]["state"]["shown"], true);
    app.choose("wait");
    app.press("step:back-to-choice");
    let restored = app.press("seat:26july:take-back");
    let revision = app.revision();
    assert_eq!(restored["table"]["choice"]["pinned"], "back-to-choice");
    let end = app.press_by_keyboard(&format!("{revision}:back-to-choice"));
    assert_eq!(end["observation"]["revision"], revision + 1);
    assert_eq!(end["observation"]["action"], "Open");
    assert_eq!(end["observation"]["transfer"], Value::Null);

    // Government can still answer with 26July's seat handed over.
    for action in ["terror", "transfer", "ask:0:1", "wait"] {
        app.choose(action);
    }
    app.press("seat:26july:hand-over");
    app.choose("reopen");
    app.choose("decline");
    let declined = app.observe();
    assert_eq!(declined["observation"]["transfer"], Value::Null);
    assert_eq!(seat(&declined, "M26")["state"], "awaiting");

    // In the reverse assignment, 26July can end Terror while Government waits.
    app.press("seat:26july:take-back");
    app.choose("transfer");
    app.press("seat:government:hand-over");
    app.choose("ask:0:1");
    let pinned = app.press_by_keyboard("step:end-terror");
    assert_eq!(pinned["observation"]["owner"], "Government");
    assert_eq!(says(&pinned, "seat-strip-title"), None);
    app.capture("human-end-terror");
    let revision = app.revision();
    let ended = app.press_by_keyboard(&format!("{revision}:end-terror"));
    assert_eq!(ended["observation"]["revision"], revision + 1);
    assert_eq!(ended["observation"]["finished"], true);
    assert_eq!(ended["observation"]["transfer"], Value::Null);
    app.quit();
}

/// Handing over and taking back by the keyboard, with the transfer modal shown, with the
/// die in the air, and in windows of other shapes.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_a_seat_changes_hands_by_keyboard_and_at_any_time() {
    let mut app = start("seat-any-time-", "issue-206", "1920x1280");
    // A socket where another table answers refuses the hand-over, as an error on the slip.
    let other = std::os::unix::net::UnixListener::bind(app.player_socket()).unwrap();
    let view = app.press("seat:government:hand-over");
    assert_eq!(seat(&view, "Government")["state"], "human");
    assert_eq!(view["table"]["slip"]["kind"], "error");
    let refused = view["table"]["slip"]["text"].as_str().unwrap();
    assert!(refused.contains("another game answers"), "{refused}");
    drop(other);
    std::fs::remove_file(app.player_socket()).unwrap();
    app.activate("notice-slip", 0);

    // The keyboard reaches each control; focus goes to the one that takes its place.
    let view = app.press_by_keyboard("seat:government:hand-over");
    assert_eq!(seat(&view, "Government")["state"], "awaiting");
    assert_eq!(view["table"]["choice"]["focus"], "seat:government:copy");
    let marks = view["table"]["focus_mark"].as_array().unwrap().clone();
    assert_eq!(marks[0]["control"], "seat:government:copy");
    let view = app.press_by_keyboard("seat:government:copy");
    assert_eq!(view["table"]["seat_cards"]["line"]["faction"], "Government");
    app.capture("focused-copy");
    let view = app.press_by_keyboard("seat-line:copy");
    assert_eq!(view["table"]["seat_cards"]["line"]["faction"], "Government");
    let view = app.press_by_keyboard("seat:government:take-back");
    assert_eq!(seat(&view, "Government")["state"], "human");
    assert_eq!(view["table"]["seat_cards"]["line"], Value::Null);
    // F1 to F4 do both.
    for (key, (_, name)) in [GOVERNMENT, M26, DIRECTORIO, SYNDICATE]
        .into_iter()
        .zip(FACTIONS)
    {
        let view = app.key(key);
        assert_eq!(seat(&view, name)["state"], "awaiting");
        let view = app.key(key);
        assert_eq!(seat(&view, name)["state"], "human");
    }

    // With the modal: its scrim covers the cards, and they work over it. Handing over the
    // composing faction puts the modal away, and taking it back shows it again.
    for action in ["terror", "transfer"] {
        app.choose(action);
    }
    let view = app.press("seat:directorio:hand-over");
    assert_eq!(view["table"]["transfer"]["state"]["shown"], true);
    assert_eq!(seat(&view, "Directorio")["state"], "awaiting");
    app.capture("over-the-modal");
    let view = app.press("seat:26july:hand-over");
    assert_eq!(view["table"]["transfer"]["state"]["shown"], false);
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "26July: Awaiting an agent"
    );
    app.offers_nothing();
    app.capture("an-agents-transfer");
    let view = app.press("seat-strip:take-back");
    assert_eq!(view["table"]["transfer"]["state"]["shown"], true);

    // The human's 26July asks an agent's Government: the panel says whose answer it
    // waits for, and the human may withdraw.
    app.key(GOVERNMENT);
    app.choose("ask:0:3");
    let view = app.settle();
    assert_eq!(view["table"]["transfer"]["state"]["shown"], false);
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "Government: Awaiting an agent"
    );
    assert_eq!(
        says(&view, "seat-strip-hint").unwrap(),
        "Only Government's seat can answer. Pin End Terror or Back to withdraw."
    );
    app.offers_nothing();
    app.capture("an-agent-answers");
    let revision = app.revision();
    app.activate("step:back-to-choice", revision);
    let view = app.settle();
    assert_eq!(says(&view, "seat-strip-title"), None);
    app.capture("the-humans-withdrawal");
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:back-to-choice")}),
        true,
    );
    let view = app.settle();
    assert_eq!(view["observation"]["action"], "Open");
    assert_eq!(view["observation"]["transfer"], Value::Null);

    // An open Kidnap has neither Roll nor Cancel while an agent's 26July decides.
    for action in ["terror", "kidnap"] {
        app.choose(action);
    }
    app.press("seat:26july:hand-over");
    app.offers_nothing();
    app.capture("an-agents-kidnap");
    app.press("seat:26july:take-back");

    // With the die in the air.
    let revision = app.revision();
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:kidnap:roll")}),
        true,
    );
    for (control, state) in [
        ("seat:26july:hand-over", "awaiting"),
        ("seat:26july:take-back", "human"),
    ] {
        app.request(
            json!({"op":"activate","revision":revision + 1,"component":control}),
            true,
        );
        let view = app.observe();
        assert_eq!(seat(&view, "M26")["state"], state);
        assert_eq!(view["table"]["die"]["landed"], false, "{control}");
    }
    app.settle();
    app.quit();
    assert_by_hand(&app.save);

    // After 26July's Pass nothing of Directorio's is in the build and no option is listed:
    // the strip stands below what was recorded.
    app.save = app.root.join("passed.json");
    app.launch_size("3840x2160");
    app.choose("pass");
    let view = app.press("seat:directorio:hand-over");
    assert_eq!(
        says(&view, "seat-strip-title").unwrap(),
        "Directorio: Build ends here"
    );
    assert_eq!(says(&view, "seat-strip-hint"), None);
    let strip = parts(&view)["seat-strip"].1;
    for part in view["table"]["choice_parts"].as_array().unwrap() {
        let name = part["part"].as_str().unwrap();
        if name.starts_with("ended-") {
            assert!(rect_of(&part["bounds"])[3] < strip[1], "{name}");
        }
    }
    app.capture("build-ends-after-pass");
    app.quit();

    // The row keeps its place in a wider window and shrinks in a narrower one, clear of
    // its neighbours (`native_main_view_keeps_areas_and_picks_in_other_windows`).
    for size in ["1920x1080", "2520x1080"] {
        app.launch_as_is(size);
        let view = app.press("seat:government:hand-over");
        app.press("seat:government:copy");
        app.capture(&format!("seats-{size}"));
        let areas = &view["table"]["main_view"]["areas"];
        let row = rect_of(&areas["seats"]);
        for (slug, _) in FACTIONS {
            let card = parts(&view)[&format!("seat:{slug}")].1;
            assert!(
                card[0] >= row[0] - 1. && card[2] <= row[2] + 1.,
                "{card:?} {row:?}"
            );
            assert!(
                card[1] >= row[1] - 1. && card[3] <= row[3] + 1.,
                "{card:?} {row:?}"
            );
        }
        app.quit();
    }
}
