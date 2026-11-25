//! #208: the newspaper in the window, through the development seat: the page against its
//! layout record and against the report it prints, when an edition opens and for whom,
//! what it holds, every way of putting it away, reopening, and the longest one. The
//! sessions are the real client's and run no model (`faction_seats`).
use super::{
    available_forces::unlaunched,
    faction_seats::{
        DIRECTORIO, GOVERNMENT, M26, SYNDICATE, assert_by_hand_of, decisions, holding, seat,
    },
    *,
};
use crate::{SEED, longest::longest};
use cuba_libre_unfolded::{Game, save::Session};
use std::collections::BTreeMap;

/// A seed whose Kidnap die is a 2, the layout's sample.
const ROLLS_2: u64 = 13;

fn record() -> Value {
    serde_json::from_str(include_str!("../../art/atlas/atlas-newspaper.layout.json")).unwrap()
}

/// The sessions that play handed-over factions, by the observation's faction name.
#[derive(Default)]
struct Sessions(BTreeMap<String, String>);

const SEATS: [(&str, &str, &str); 4] = [
    ("Government", "government", GOVERNMENT),
    ("M26", "26july", M26),
    ("Directorio", "directorio", DIRECTORIO),
    ("Syndicate", "syndicate", SYNDICATE),
];

/// A run whose new match rolls by `seed`, in which `agents` are handed over and joined.
fn start(prefix: &str, size: &str, seed: u64, agents: &[&str]) -> (NativeRun, Sessions) {
    let mut app = unlaunched(prefix, "issue-208");
    drop(Session::create_seeded(&app.save, seed).unwrap());
    app.launch_as_is(size);
    let mut sessions = Sessions::default();
    for agent in agents {
        sessions.join(&mut app, agent);
    }
    (app, sessions)
}

impl Sessions {
    fn join(&mut self, app: &mut Native, faction: &str) {
        let (_, name, key) = SEATS.iter().find(|seat| seat.0 == faction).unwrap();
        let player = app.seat_agent(key, name);
        self.0.insert(faction.into(), player);
    }
    /// Each decision by whoever it is: an agent's session's, or the human's at the table.
    fn play(&self, app: &mut Native, actions: &[&str]) {
        for action in actions {
            let view = app.observe();
            let choice = offered(&view, action);
            let at = view["observation"]["revision"].as_u64().unwrap();
            match self.0.get(choice["owner"].as_str().unwrap()) {
                Some(player) if *action != "advance-card" => {
                    // One after the Kidnap roll is held while the die shows its result.
                    let id = format!("{at}:{action}");
                    let (_, answer) = app.ask(&["choose", player, &at.to_string(), &id]);
                    assert_ne!(answer["status"], "refused", "{action}: {answer}");
                }
                _ if *action == "kidnap:roll" => app.roll(),
                _ => {
                    let row = format!("option:{action}");
                    let controls = view["table"]["controls"].as_array().unwrap();
                    if controls.iter().any(|control| control["name"] == row) {
                        app.activate(&row, at);
                    }
                    app.choose(action);
                }
            }
            assert_eq!(app.settle()["observation"]["revision"], at + 1, "{action}");
        }
    }
}

fn offered<'a>(view: &'a Value, action: &str) -> &'a Value {
    let id = format!("{}:{action}", view["observation"]["revision"]);
    view["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|choice| choice["id"] == id)
        .unwrap_or_else(|| panic!("{action} is offered: {view}"))
}

fn paper(view: &Value) -> &Value {
    &view["table"]["newspaper"]
}

/// The open edition's report, with its number.
fn report(view: &Value) -> Value {
    assert!(
        paper(view)["open"].is_object(),
        "an edition is open: {view}"
    );
    paper(view)["open"]["report"].clone()
}

fn closed(view: &Value) {
    assert_eq!(paper(view)["open"], Value::Null, "no edition is open");
    assert_eq!(paper(view)["due"], false);
}

/// The drawn parts by name.
fn parts(view: &Value) -> BTreeMap<String, Value> {
    paper(view)["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|part| (part["part"].as_str().unwrap().to_owned(), part.clone()))
        .collect()
}

fn words(value: &Value) -> String {
    value.as_str().unwrap().to_owned()
}

/// What the open page says by its part's name, the pile's parts left out.
fn printed(view: &Value) -> BTreeMap<String, String> {
    parts(view)
        .into_iter()
        .filter(|(name, part)| !name.starts_with("editions") && part["text"].is_string())
        .map(|(name, part)| (name, words(&part["text"])))
        .collect()
}

fn faction_name(faction: &Value) -> &'static str {
    match faction.as_str().unwrap() {
        "Government" => "Government",
        "M26" => "26July",
        "Directorio" => "Directorio",
        "Syndicate" => "Syndicate",
        other => panic!("{other}"),
    }
}

/// Every word the edition `number` of `report` has: its own, and the page's fixed ones.
fn expected(report: &Value, number: u64) -> BTreeMap<String, String> {
    let mut page = BTreeMap::new();
    let mut put = |name: String, text: String| assert!(page.insert(name, text).is_none());
    let action = match report["action"].as_str().unwrap() {
        "pass" => "Pass",
        "terror" => "Terror",
        "assault" => "Assault",
        "event-shaded" | "event-unshaded" => "Event",
        other => panic!("{other}"),
    };
    let complete = report["complete"].as_bool().unwrap();
    put(
        "edition-folio".into(),
        format!(
            "No. {number}{} · Card {} · {}",
            if complete { "" } else { " · partway" },
            report["card"],
            words(&report["card_title"])
        ),
    );
    let what = match (complete, report["recorded"].as_str()) {
        (true, Some(recorded)) => recorded.to_owned(),
        (true, None) => action.to_owned(),
        (false, _) => format!("{action} in progress"),
    };
    put(
        "edition-about".into(),
        format!(
            "{} · {} · {what}",
            faction_name(&report["faction"]),
            words(&report["role"])
        ),
    );
    put("edition-headline".into(), words(&report["headline"]));
    if let Some(summary) = report["summary"].as_str() {
        put("edition-summary".into(), summary.into());
    }
    put("edition-standings".into(), "The standings".into());
    put("edition-heading:resources".into(), "RESOURCES".into());
    put("edition-heading:margin".into(), "MARGIN".into());
    for (row, slug) in report["standings"].as_array().unwrap().iter().zip([
        "government",
        "26july",
        "directorio",
        "syndicate",
    ]) {
        put(
            format!("edition-faction:{slug}"),
            faction_name(&row["faction"]).into(),
        );
        for what in ["resources", "margin"] {
            let figure = |end: &str| row[what][end].to_string().replace('-', "−");
            let (from, to) = (figure("from"), figure("to"));
            put(
                format!("edition-{what}:{slug}"),
                if from == to {
                    to
                } else {
                    format!("{from} → {to}")
                },
            );
        }
    }
    let lines =
        |of: &Value| -> Vec<String> { serde_json::from_value(of["lines"].clone()).unwrap() };
    let articles = report["articles"].as_array().unwrap();
    for (i, article) in articles.iter().enumerate() {
        put(format!("edition-block-{i}-title"), words(&article["title"]));
        if !lines(article).is_empty() {
            put(
                format!("edition-block-{i}-text-0"),
                lines(article).join(" "),
            );
        }
    }
    for (i, note) in report["notes"].as_array().unwrap().iter().enumerate() {
        let i = articles.len() + i;
        put(format!("edition-block-{i}-title"), words(&note["title"]));
        for (j, line) in lines(note).into_iter().enumerate() {
            put(format!("edition-block-{i}-text-{j}"), line);
        }
    }
    put(
        "edition-source".into(),
        "Written from the committed result".into(),
    );
    put(
        "edition-away".into(),
        "Click, Escape, Enter or Space puts it away".into(),
    );
    page
}

/// Whether the one page shown says what the edition of `report` says and nothing else.
fn says_only(view: &Value, report: &Value) -> Result<(), String> {
    let number = paper(view)["open"]["number"].as_u64().unwrap();
    let (printed, expected) = (printed(view), expected(report, number));
    for name in printed.keys().chain(expected.keys()) {
        if printed.get(name) != expected.get(name) {
            return Err(format!(
                "{name}: printed {:?}, the report has {:?}",
                printed.get(name),
                expected.get(name)
            ));
        }
    }
    let dice = |of: &Value| of["die"].as_u64();
    let articles = report["articles"].as_array().unwrap();
    for (i, article) in articles.iter().enumerate() {
        let drawn = parts(view).contains_key(&format!("edition-block-{i}-die"));
        if drawn != dice(article).is_some() {
            return Err(format!("article {i}'s die"));
        }
    }
    Ok(())
}

/// The save's own report that starts after `from` and is still or last open at `to`,
/// derived again from its decisions, apart from the app.
fn derived(save: &Path, report: &Value) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let copy = dir.path().join("match.json");
    std::fs::copy(save, &copy).unwrap();
    let session = Session::open(&copy, None).unwrap();
    let from = report["from_revision"].as_u64().unwrap();
    let to = report["to_revision"].as_u64().unwrap();
    let found = session
        .reports_since(from)
        .into_iter()
        .map(|found| serde_json::to_value(found).unwrap())
        .find(|found| found["to_revision"] == to && found["from_revision"] == from);
    found.unwrap_or_else(|| panic!("no report of {from} to {to}"))
}

/// The open edition prints the report the save's decisions give, and nothing else; and the
/// check finds each figure planted wrong in it.
fn check(app: &mut Native) -> Value {
    let view = app.settle();
    let report = report(&view);
    assert_eq!(derived(&app.save, &report), report);
    says_only(&view, &report).unwrap();
    report
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_the_newspaper_matches_the_selected_layout() {
    let layout = record();
    let (mut app, mut sessions) = start(
        "newspaper-layout-",
        "3840x2160",
        ROLLS_2,
        &["M26", "Syndicate"],
    );
    let decisions: Vec<String> =
        serde_json::from_value(layout["terror"]["decisions"].clone()).unwrap();
    for action in decisions {
        let view = app.observe();
        closed(&view);
        sessions.play(&mut app, &[&action]);
    }
    // The pieces and the die are settled when it opens, and it is the one edition there is.
    let report = check(&mut app);
    let view = app.settle();
    assert_eq!(view["table"]["settled"], true);
    assert_eq!(paper(&view)["open"]["number"], 1);
    assert_eq!(paper(&view)["pile"], 0);
    assert_eq!(report["faction"], "M26");
    app.capture("selected-layout");
    compare(&view, "terror");
    plant(&view, &report);
    let view = app.key("Escape");
    closed(&view);
    assert_eq!(paper(&view)["pile"], 1);
    app.capture("put-away");

    // The human's Directorio has no edition, and the card's own leave with the card.
    sessions.play(&mut app, &["event:shaded", "resolve-shaded"]);
    let view = app.settle();
    closed(&view);
    assert_eq!(paper(&view)["pile"], 1);
    sessions.play(&mut app, &["advance-card"]);
    let view = app.settle();
    closed(&view);
    assert_eq!(paper(&view)["pile"], 0, "the card's editions left with it");
    assert!(parts(&view).is_empty(), "{:?}", parts(&view));
    sessions.join(&mut app, "Government");
    sessions.play(&mut app, &["government:pass"]);
    let report = check(&mut app);
    app.capture("pass");
    plant(&app.settle(), &report);
    let revision = app.revision();
    app.activate("edition", revision);
    closed(&app.settle());
    app.activate("editions", revision);
    let view = app.settle();
    assert_eq!(paper(&view)["open"]["reopened"], true);
    app.capture("pass-reopened");
    compare(&view, "pass");
    app.quit();
}

/// Puts the open edition away with `key`: the table then.
fn away(app: &mut Native, key: &str) -> Value {
    let revision = app.revision();
    let mut request = json!({"op":"key","revision":revision,"key":key});
    if key != "Escape" {
        request["focused"] = json!("edition");
    }
    app.request(request, true);
    app.settle()
}

/// Every part the layout's `frame` records is drawn where it draws it and says what it
/// says, and no other part is drawn.
fn compare(view: &Value, frame: &str) {
    let layout = &record()[frame];
    let tolerance = record()["tolerance"].as_f64().unwrap();
    let drawn = parts(view);
    let group = |name: &str| layout[name].as_object().unwrap().clone();
    let near = |name: &str, n: f64, e: f64, what: &str| {
        assert!(
            (n - e).abs() <= tolerance,
            "{name}'s {what}: native {n}, layout {e}"
        );
    };
    for (name, rect) in group("parts") {
        let [x, y, w, h] = rect_of(&rect);
        let native = rect_of(&drawn[&name]["bounds"]);
        for (n, e) in native.iter().zip([x, y, x + w, y + h]) {
            near(&name, *n, e, "edge");
        }
    }
    for (name, rect) in group("text") {
        let [x, y, w, h] = rect_of(&rect);
        let native = rect_of(&drawn[&name]["bounds"]);
        near(&name, native[0], x, "left");
        near(&name, native[1], y, "top");
        // Its lines, at their height.
        near(&name, native[3], y + h, "foot");
        // Two texts' boxes are their words' own width there.
        if !name.starts_with("edition-faction") && name != "editions-caption" {
            near(&name, native[2], x + w, "right");
        }
        // The words stand at their side of the box, and in it.
        if let Some(ink) = drawn[&name]["ink"].as_array() {
            let ink = rect_of(&json!(ink));
            assert!(
                ink[0] >= native[0] - 1. && ink[2] <= native[2] + 1.,
                "{name}"
            );
            let right = ["edition-about", "edition-away", "edition-heading"];
            let right = right.iter().any(|of| name.starts_with(of))
                || name.starts_with("edition-resources")
                || name.starts_with("edition-margin");
            if right {
                near(&name, ink[2], native[2], "words");
            } else if ["edition-headline", "edition-summary"].contains(&name.as_str()) {
                near(&name, ink[0] + ink[2], native[0] + native[2], "words");
            } else {
                near(&name, ink[0], native[0], "words");
            }
        }
    }
    for (name, said) in group("words") {
        assert_eq!(drawn[&name]["text"], said, "{name}");
    }
    let recorded: std::collections::BTreeSet<String> = ["parts", "text"]
        .into_iter()
        .flat_map(|name| group(name).into_iter().map(|(name, _)| name))
        .collect();
    assert_eq!(
        drawn
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        recorded
    );
}

/// The check finds each figure of the page when it is another, and each word left out.
fn plant(view: &Value, report: &Value) {
    let mut planted = 0;
    let drawn = paper(view)["parts"].as_array().unwrap();
    for (i, part) in drawn.iter().enumerate() {
        let Some(text) = part["text"].as_str() else {
            continue;
        };
        if part["part"].as_str().unwrap().starts_with("editions") {
            continue;
        }
        let mut wrong = vec![];
        for (at, digit) in text.char_indices().filter(|(_, c)| c.is_ascii_digit()) {
            let other = if digit == '9' { '8' } else { '9' };
            wrong.push(format!("{}{other}{}", &text[..at], &text[at + 1..]));
        }
        wrong.push(text.split_once(' ').map_or("", |(_, rest)| rest).to_owned());
        for text in wrong {
            let mut view = view.clone();
            view["table"]["newspaper"]["parts"][i]["text"] = json!(text);
            assert!(says_only(&view, report).is_err(), "{} {text}", part["part"]);
            planted += 1;
        }
    }
    assert!(planted > 20, "{planted}");
}

/// An edition for each kind of action, one partway when the human must decide, and none
/// for the human's own, whichever faction that is.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_an_edition_tells_each_action_of_a_faction_the_human_does_not_play() {
    // A Pass of 26July, which ends the build.
    let (mut app, sessions) = start("newspaper-kinds-", "1920x1080", ROLLS_2, &["M26"]);
    sessions.play(&mut app, &["pass"]);
    assert_eq!(check(&mut app)["headline"], "26July Passes");
    app.capture("26july-passes");
    app.quit();

    // Terror without Kidnap, partway when the human's Government must answer its request,
    // and once for the rest of it. A request set aside and taken up again is no new one.
    let (mut app, mut sessions) = start(
        "newspaper-terror-",
        "1920x1080",
        ROLLS_2,
        &["M26", "Syndicate"],
    );
    sessions.play(
        &mut app,
        &[
            "terror",
            "transfer",
            "ask:3:2",
            "authorize:2",
            "terror:oriente",
            "transfer",
            "ask:0:2",
        ],
    );
    let report = check(&mut app);
    assert_eq!(report["complete"], false);
    assert_eq!(
        (&report["headline"], &report["summary"]),
        (
            &json!("26July executes Terror: Oriente so far"),
            &json!("26July asks Government for 2 Resources. Government decides.")
        )
    );
    let view = app.settle();
    assert_eq!(paper(&view)["open"]["number"], 1);
    // It is open over the modal that asks for the decision, which waits beneath it.
    assert_eq!(view["table"]["transfer"]["state"]["shown"], true);
    app.capture("partway-over-the-modal");
    let revision = app.revision();
    closed(&away(&mut app, "Enter"));
    assert_eq!(
        app.revision(),
        revision,
        "the modal's button was not pressed"
    );
    let view = app.key("Escape");
    assert_eq!(
        view["observation"]["transfer"]["Authorizing"]["waited"], true,
        "{view}"
    );
    closed(&view);
    sessions.play(&mut app, &["reopen", "authorize:1"]);
    closed(&app.settle());
    // It asks again, which is another decision of the human's.
    sessions.play(&mut app, &["transfer", "ask:0:1"]);
    let report = check(&mut app);
    assert_eq!(
        (&report["headline"], &report["continues"]),
        (&json!("26July continues Terror"), &json!(true))
    );
    assert_eq!(report["articles"].as_array().unwrap().len(), 1, "told once");
    closed(&away(&mut app, "Space"));
    sessions.play(&mut app, &["decline", "end-terror"]);
    let report = check(&mut app);
    assert_eq!(report["headline"], "26July ends Terror");
    assert_eq!(paper(&app.settle())["open"]["number"], 3);
    app.key("Escape");
    // Directorio has nothing to choose after Terror without Kidnap: the build ends.
    sessions.join(&mut app, "Directorio");
    closed(&app.settle());
    app.quit();

    // The human's own Terror with Kidnap has none, with an agent's Syndicate its giver.
    // Directorio's unshaded Event has one when 26July is the human's to Rally, and one for
    // the rest. Government's Pass ends the match.
    let (mut app, mut sessions) = start(
        "newspaper-rally-",
        "1920x1080",
        ROLLS_2,
        &["Directorio", "Syndicate"],
    );
    sessions.play(
        &mut app,
        &[
            "terror",
            "transfer",
            "ask:3:2",
            "authorize:2",
            "terror:santiago",
            "kidnap",
            "kidnap:roll",
            "end-terror",
        ],
    );
    closed(&app.settle());
    assert_eq!(paper(&app.settle())["pile"], 0);
    sessions.play(&mut app, &["event:unshaded"]);
    closed(&app.settle());
    sessions.play(&mut app, &["rally:beneficiary:26july"]);
    let report = check(&mut app);
    assert_eq!(
        (&report["headline"], &report["summary"], &report["complete"]),
        (
            &json!("Directorio executes the Event, unshaded text"),
            &json!("Directorio grants the free Rally to 26July."),
            &json!(false)
        )
    );
    app.capture("partway-rally");
    app.key("Escape");
    sessions.play(&mut app, &["rally:sierra-maestra:place:2"]);
    let report = check(&mut app);
    assert_eq!(
        report["headline"],
        "Directorio ends the Event, unshaded text"
    );
    assert_eq!(
        report["articles"][0]["title"],
        "26July Rallies in Sierra Maestra"
    );
    app.capture("rally-ends");
    app.key("Escape");
    sessions.play(&mut app, &["advance-card"]);
    sessions.join(&mut app, "Government");
    sessions.play(&mut app, &["government:pass"]);
    assert_eq!(check(&mut app)["headline"], "Government Passes");
    app.quit();

    // Terror with Kidnap whole, the shaded Event with a removal, and an Assault in several
    // spaces that closes a Casino and removes Guerrillas, partway when it asks the human's
    // 26July. Every edition of one action follows the one before it.
    let (mut app, mut sessions) = start(
        "newspaper-assault-",
        "1920x1080",
        ROLLS_2,
        &["M26", "Directorio", "Syndicate"],
    );
    sessions.play(
        &mut app,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ],
    );
    let report = check(&mut app);
    assert_eq!(report["recorded"], "Operation with Special Activity");
    assert_eq!(report["articles"][0]["die"], 2);
    app.key("Escape");
    sessions.play(
        &mut app,
        &[
            "event:shaded",
            "remove",
            "remove:havana:underground:1",
            "done-removal",
            "resolve-shaded",
        ],
    );
    let report = check(&mut app);
    assert_eq!(
        report["headline"],
        "Directorio executes the Event, shaded text"
    );
    assert_eq!(paper(&app.settle())["open"]["number"], 2);
    app.capture("shaded-event");
    app.key("Escape");
    sessions.play(&mut app, &["advance-card"]);
    // The human takes 26July back. Its own later acts are its giver's only.
    app.key(M26);
    sessions.0.remove("M26");
    sessions.join(&mut app, "Government");
    sessions.play(
        &mut app,
        &[
            "government:assault",
            "assault:havana:syndicate",
            "transfer",
            "ask:1:1",
        ],
    );
    let report = check(&mut app);
    assert_eq!(
        report["headline"],
        "Government executes Assault: Havana so far"
    );
    assert_eq!(
        paper(&app.settle())["open"]["number"],
        1,
        "a card's own count"
    );
    app.key("Escape");
    sessions.play(
        &mut app,
        &["authorize:1", "assault:santiago:26july", "end-assault"],
    );
    let report = check(&mut app);
    assert_eq!(
        report["headline"],
        "Government ends Assault: Santiago de Cuba"
    );
    assert_eq!(paper(&app.settle())["open"]["number"], 2);
    app.capture("assault-ends");
    app.key("Escape");
    assert_by_hand_of(&app.save, ROLLS_2);
    app.quit();

    // Directorio's own Rally, and an Assault in no space.
    let (mut app, sessions) = start(
        "newspaper-none-",
        "1920x1080",
        ROLLS_2,
        &["M26", "Directorio", "Government"],
    );
    sessions.play(
        &mut app,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ],
    );
    app.key("Escape");
    sessions.play(
        &mut app,
        &["event:unshaded", "rally:beneficiary:directorio"],
    );
    let report = check(&mut app);
    assert_eq!(
        report["headline"],
        "Directorio executes the Event, unshaded text"
    );
    app.key("Escape");
    sessions.play(
        &mut app,
        &["advance-card", "government:assault", "end-assault"],
    );
    assert_eq!(
        check(&mut app)["headline"],
        "Government executes Assault in no space"
    );
    app.quit();
}

/// While an edition is open nothing is decided, by the human or by an agent, and nothing
/// beneath it is reached; each way of putting it away decides nothing; and the card's
/// editions are read again.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_an_open_edition_holds_every_decision_until_it_is_put_away() {
    let (mut app, sessions) = start(
        "newspaper-holds-",
        "1920x1080",
        ROLLS_2,
        &["M26", "Directorio"],
    );
    let directorio = sessions.0["Directorio"].clone();
    sessions.play(
        &mut app,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ],
    );
    // It opened once everything had settled, and it holds the next agent's choice.
    let first = check(&mut app);
    let revision = app.revision();
    let saved = app.save_bytes();
    let camera = |view: &Value| view["table"]["camera"].clone();
    let at_rest = camera(&app.observe());
    app.decide(&directorio, "event:shaded", "held");
    let view = app.settle();
    assert_eq!(holding(&view), ["an edition"]);
    assert_eq!(seat(&view, "Directorio")["holds_choice"], true);
    assert_eq!(view["observation"]["revision"], revision);
    // Nothing beneath it is reached by the pointer: the seat finds the edition there.
    for beneath in [
        "quit",
        "card:current",
        "option:event:shaded",
        "seat:government:hand-over",
        "standing:0",
    ] {
        let refused = app.request(
            json!({"op":"activate","revision":revision,"component":beneath}),
            false,
        );
        let error = refused["error"].as_str().unwrap();
        assert!(error.contains("edition"), "{beneath}: {error}");
        app.request(
            json!({"op":"point","revision":revision,"component":beneath}),
            false,
        );
    }
    // Nor by a key: not the camera, the panel's focus, or the edition's own pages, which
    // are one. The wheel and a drag do nothing.
    for key in [
        "KeyW",
        "ArrowLeft",
        "ArrowRight",
        "Home",
        "Tab",
        "Backspace",
        "5",
    ] {
        let view = app.key(key);
        assert_eq!(camera(&view), at_rest, "{key}");
        assert_eq!(view["table"]["choice"]["focus"], Value::Null, "{key}");
        assert_eq!(report(&view), first, "{key}");
    }
    app.request(
        json!({"op":"scroll","revision":revision,"component":"edition","y":6}),
        true,
    );
    let drag = json!({
        "op":"drag","revision":revision,"component":"edition","x":120,"y":40,"button":"right"
    });
    app.request(drag, true);
    let view = app.settle();
    assert_eq!(camera(&view), at_rest);
    // The seats' own keys work at any time, and the edition stays.
    let view = app.key(SYNDICATE);
    assert_eq!(seat(&view, "Syndicate")["state"], "awaiting");
    assert_eq!(report(&view), first);
    app.key(SYNDICATE);
    assert_eq!(report(&app.settle()), first);
    assert_eq!(app.save_bytes(), saved);
    app.capture("holding-an-agents-choice");

    // Put away by a click, the held choice is applied, and no other.
    app.activate("edition", revision);
    let view = app.settle();
    closed(&view);
    assert_eq!(view["observation"]["revision"], revision + 1, "{view}");
    assert_eq!(decisions(&app.save).last().unwrap(), "event:shaded");
    sessions.play(&mut app, &["resolve-shaded"]);
    let second = check(&mut app);
    assert_eq!(paper(&app.settle())["open"]["number"], 2);
    assert_eq!(paper(&app.settle())["pile"], 1);

    // Every way of putting it away decides nothing and reaches nothing beneath: the card
    // is the human's to turn, and stays. Enter and Space do not press the pile that opened
    // it.
    let revision = app.revision();
    let saved = app.save_bytes();
    let untouched = |app: &mut Native, view: &Value, what: &str| {
        closed(view);
        assert_eq!(view["observation"]["revision"], revision, "{what}");
        assert_eq!(view["table"]["card"]["pinned"], Value::Null, "{what}");
        assert_eq!(camera(view), at_rest, "{what}");
        assert_eq!(app.save_bytes(), saved, "{what}");
    };
    let view = app.key("Escape");
    untouched(&mut app, &view, "Escape");
    assert_eq!(paper(&view)["pile"], 2);
    assert_eq!(
        view["table"]["main_view"]["areas"]["editions"].is_array(),
        true
    );
    for way in ["Escape", "Enter", "Space"] {
        // Reopened by the keyboard, at the latest.
        for _ in 0..60 {
            if app.observe()["table"]["choice"]["focus"] == "editions" {
                break;
            }
            app.key("Tab");
        }
        app.request(
            json!({"op":"key","revision":revision,"key":"Enter","focused":"editions"}),
            true,
        );
        let view = app.settle();
        assert_eq!(report(&view), second, "{way}");
        assert_eq!(paper(&view)["open"]["reopened"], true);
        assert_eq!(
            view["table"]["focus_mark"],
            json!([]),
            "Enter presses no control"
        );
        let view = away(&mut app, way);
        untouched(&mut app, &view, way);
        assert_eq!(view["table"]["choice"]["focus"], "editions");
    }
    // Reopened by the pointer, with a tab for each. A tab turns to its edition, and Tab to
    // the next. A click on the page or on the pile puts it away.
    for away in ["edition", "editions"] {
        app.activate("editions", revision);
        let view = app.settle();
        assert_eq!(report(&view), second);
        let tabs = parts(&view);
        assert_eq!(tabs["edition-tab:2-label"]["text"], "No. 2 · Event ends");
        assert_eq!(tabs["edition-tab:1-label"]["text"], "No. 1 · Terror ends");
        app.activate("edition-tab:1", revision);
        let view = app.settle();
        assert_eq!(report(&view), first);
        app.capture(&format!("reopened-{away}"));
        assert_eq!(report(&app.key("Tab")), second);
        assert_eq!(report(&app.key("Tab")), first);
        app.activate(away, revision);
        let view = app.settle();
        untouched(&mut app, &view, away);
    }
    // It opens over nothing that was held: the card is turned, and its editions are gone.
    sessions.play(&mut app, &["advance-card"]);
    let view = app.settle();
    assert_eq!(paper(&view)["pile"], 0);
    assert_eq!(view["table"]["main_view"]["areas"]["editions"], Value::Null);
    app.quit();

    // Editions are in no save. One left unread as the table closes is not shown again, and
    // neither are those that were read.
    let (mut app, sessions) = start("newspaper-unread-", "1920x1080", ROLLS_2, &["M26"]);
    sessions.play(
        &mut app,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ],
    );
    check(&mut app);
    let saved = app.save_bytes();
    app.quit();
    assert_eq!(app.save_bytes(), saved);
    app.launch_as_is("1920x1080");
    let view = app.settle();
    closed(&view);
    assert_eq!(paper(&view)["pile"], 0);
    assert_eq!(view["observation"]["revision"], 5);
    // What follows is told from there. Save · quit itself leaves it as read.
    let mut sessions = Sessions::default();
    sessions.join(&mut app, "Directorio");
    sessions.play(&mut app, &["event:shaded", "resolve-shaded"]);
    let next = check(&mut app);
    assert_eq!(
        (&next["from_revision"], &next["faction"]),
        (&json!(5), &json!("Directorio"))
    );
    assert_eq!(paper(&app.settle())["open"]["number"], 1);
    away(&mut app, "Space");
    assert_eq!(paper(&app.settle())["pile"], 1);
    let revision = app.revision();
    let quit = app.request(
        json!({"op":"activate","revision":revision,"component":"quit"}),
        true,
    );
    assert_eq!(quit["status"], "exited", "{quit}");
    app.launch_as_is("1920x1080");
    let view = app.settle();
    closed(&view);
    assert_eq!(paper(&view)["pile"], 0);
    assert_by_hand_of(&app.save, ROLLS_2);
    app.quit();
}

/// The longest editions found, at the owner's display: the unshaded Event with all its
/// side decisions, an Assault in 12 spaces and Terror, each with every transfer and
/// removal it can have. Four sessions play every decision of `longest`.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_the_longest_editions_are_read_at_real_size() {
    for (name, decisions, pages) in longest()
        .into_iter()
        .zip([
            ("terror", 1),
            ("shaded", 1),
            ("unshaded", 2),
            ("assault", 1),
        ])
        .map(|(decisions, (name, pages))| (name, decisions, pages))
        .filter(|(name, ..)| *name != "shaded")
    {
        let mut app = unlaunched(&format!("newspaper-longest-{name}-"), "issue-208");
        drop(Session::create_seeded(&app.save, SEED).unwrap());
        app.launch_as_is("3840x2160");
        let mut sessions = Sessions::default();
        for (faction, ..) in SEATS {
            sessions.join(&mut app, faction);
        }
        let mut game = Game::opening_with_seed(SEED);
        let mut ended = false;
        for action in &decisions {
            // Each action before the last has its edition too.
            if std::mem::take(&mut ended) {
                app.settle();
                closed(&app.key("Escape"));
            }
            let seen = game.observe();
            let choice = seen.choices.iter().find(|c| c.action() == *action).unwrap();
            if action == "advance-card" {
                app.choose(action);
                app.settle();
            } else {
                let owner = serde_json::to_value(choice.owner).unwrap();
                let player = &sessions.0[owner.as_str().unwrap()];
                let at = seen.revision.to_string();
                let (_, answer) = app.ask(&["choose", player, &at, &choice.id]);
                assert_ne!(answer["status"], "refused", "{action}: {answer}");
            }
            ended = ["end-terror", "resolve-shaded"].contains(&action.as_str());
            game.submit(choice.owner, seen.revision, &choice.id)
                .unwrap();
        }
        let view = app.settle();
        let report = report(&view);
        assert_eq!(derived(&app.save, &report), report);
        assert_eq!(paper(&view)["open"]["pages"], pages, "{name}");
        // Page by page it says everything, once, and nothing else, inside the window and
        // with no text over another.
        let mut said = BTreeMap::new();
        for page in 1..=pages {
            let view = app.settle();
            assert_eq!(paper(&view)["open"]["page"], page);
            app.capture(&format!("{name}-page-{page}"));
            measured(&view);
            for (part, text) in printed(&view) {
                let repeated = [
                    "edition-folio",
                    "edition-about",
                    "edition-source",
                    "edition-away",
                ];
                if part == "edition-pages" {
                    assert_eq!(
                        text,
                        format!("Page {page} of {pages} · Left and Right turn it")
                    );
                } else if let Some(before) = said.insert(part.clone(), text.clone()) {
                    assert!(
                        repeated.contains(&part.as_str()) && before == text,
                        "{part}"
                    );
                }
            }
            app.key("ArrowRight");
        }
        assert_eq!(said, expected(&report, 1 + u64::from(name == "unshaded")));
        if pages > 1 {
            assert_eq!(
                paper(&app.settle())["open"]["page"],
                pages,
                "no page follows"
            );
            assert_eq!(paper(&app.key("ArrowLeft"))["open"]["page"], pages - 1);
        }
        closed(&app.key("Escape"));
        app.quit();
    }
}

/// The page is inside the window, its texts inside it and clear of each other, and each is
/// as large as the layout's: on the owner's display a pixel is 0.18 mm.
fn measured(view: &Value) {
    let drawn = parts(view);
    let page = rect_of(&drawn["edition-page"]["bounds"]);
    assert!(page[1] >= 0. && page[3] <= 2160. - 42., "{page:?}");
    let texts: Vec<(&String, [f64; 4], f64)> = drawn
        .iter()
        .filter(|(name, part)| part["ink"].is_array() && !name.starts_with("editions"))
        .map(|(name, part)| {
            (
                name,
                rect_of(&part["ink"]),
                part["font_size"].as_f64().unwrap(),
            )
        })
        .collect();
    assert!(texts.len() >= 8);
    for (i, (name, ink, size)) in texts.iter().enumerate() {
        assert!(
            ink[0] >= page[0] + 39. && ink[2] <= page[2] - 39. && ink[3] <= page[3],
            "{name} {ink:?} in {page:?}"
        );
        let least = match name.as_str() {
            name if name.contains("-text-") => 27.,
            name if name.contains("-title") => 38.,
            "edition-headline" => 84.,
            name if name.starts_with("edition-heading") => 20.,
            "edition-source" | "edition-away" | "edition-pages" => 22.,
            _ => 25.,
        };
        assert!(*size >= least, "{name}: {size}");
        for (other, against, _) in &texts[..i] {
            let apart = ink[0] >= against[2]
                || against[0] >= ink[2]
                || ink[1] >= against[3]
                || against[1] >= ink[3];
            assert!(apart, "{name} {ink:?} over {other} {against:?}");
        }
    }
}
