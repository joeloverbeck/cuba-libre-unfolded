//! #208: the newspaper in the window, through the development seat: the page against its
//! layout record and against the report it prints, when an edition opens and for whom,
//! what it holds, every way of putting it away, reopening, and the longest one. The
//! sessions are the real client's and run no model (`faction_seats`).
use super::{
    available_forces::unlaunched,
    faction_seats::{DIRECTORIO, GOVERNMENT, M26, SYNDICATE, assert_by_hand, holding, seat},
    *,
};
use cuba_libre_unfolded::{Faction, save::Session};
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
