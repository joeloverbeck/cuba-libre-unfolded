//! #203: agent sessions play handed-over factions at the windowless table, over its socket
//! and with the real client, while the human's side is driven through the development
//! seat. A session runs no model here. Kidnap's die is a 1 (`Session::KIDNAP_ROLLS_1`).
use super::{available_forces::start, *};
use std::{
    process::{Child, Stdio},
    time::{Duration, Instant},
};

/// Handing Government, 26July, Directorio and Syndicate over, and taking them back.
const GOVERNMENT: &str = "F1";
const M26: &str = "F2";
const DIRECTORIO: &str = "F3";
const SYNDICATE: &str = "F4";

/// Keys that could be a generator's state or the face-down order.
fn assert_hidden(answer: &Value) {
    let text = answer.to_string();
    for hidden in ["random", "draw_deck", "seed", "played_cards"] {
        assert!(!text.contains(&format!("\"{hidden}")), "{hidden} is told");
    }
}

/// A session's command in progress.
struct Asked(Child);
impl Drop for Asked {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Asked {
    /// Its exit status and answer, when it has ended.
    fn ended(&mut self) -> Option<(i32, Value)> {
        let status = self.0.try_wait().unwrap()?;
        let mut out = String::new();
        std::io::Read::read_to_string(self.0.stdout.as_mut().unwrap(), &mut out).unwrap();
        let answer = serde_json::from_str(&out).unwrap_or(Value::Null);
        assert_hidden(&answer);
        Some((status.code().unwrap_or(-1), answer))
    }
}

impl Native {
    fn player_socket(&self) -> PathBuf {
        self.socket.with_file_name("players.sock")
    }
    /// Starts a session's command. `session` stands for its tool's own identifier.
    fn player(&self, session: &str, args: &[&str]) -> Asked {
        Asked(
            Command::new(env!("CARGO_BIN_EXE_cuba-player"))
                .arg("--socket")
                .arg(self.player_socket())
                .args(args)
                .env_remove("CLAUDE_CODE_SESSION_ID")
                .env("CODEX_THREAD_ID", session)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        )
    }
    fn settle(&mut self) -> Value {
        self.request(json!({"op":"settle"}), true)
    }
    /// The answer to a command, with the table's frames run meanwhile.
    fn answered(&mut self, asked: &mut Asked) -> (i32, Value) {
        let end = Instant::now() + Duration::from_secs(60);
        loop {
            self.settle();
            if let Some(ended) = asked.ended() {
                return ended;
            }
            assert!(Instant::now() < end, "a command was not answered");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    /// The answer to a command given while `pending` keeps the table unsettled, with
    /// `steps` of table time run at once.
    fn answered_while(&mut self, asked: &mut Asked, steps: u32, pending: &str) -> (i32, Value) {
        loop {
            let view = self.request(json!({"op":"advance","steps":steps}), true);
            assert!(view["pending"].to_string().contains(pending), "{view}");
            if let Some(ended) = asked.ended() {
                return ended;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    fn ask(&mut self, args: &[&str]) -> (i32, Value) {
        let mut asked = self.player(args.get(1).copied().unwrap_or("none"), args);
        self.answered(&mut asked)
    }
    /// Runs frames until the faction's seat waits, as its command asked.
    fn until_waiting(&mut self, faction: &str) {
        let end = Instant::now() + Duration::from_secs(30);
        loop {
            let view = self.settle();
            if seat(&view, faction)["waits"] == true {
                return;
            }
            assert!(Instant::now() < end, "{faction}'s seat does not wait");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    fn key(&mut self, key: &str) -> Value {
        let revision = self.revision();
        self.request(json!({"op":"key","revision":revision,"key":key}), true);
        self.settle()
    }
    fn revision(&mut self) -> u64 {
        self.observe()["observation"]["revision"].as_u64().unwrap()
    }
    /// Hand a faction over and join as it: the session's identity.
    fn seat_agent(&mut self, key: &str, faction: &str) -> String {
        self.key(key);
        let (code, joined) = self.ask(&["join", faction]);
        assert_eq!((code, &joined["status"]), (0, &json!("joined")), "{joined}");
        joined["player"].as_str().unwrap().to_owned()
    }
    /// A session's choice at the present revision.
    fn decide(&mut self, player: &str, action: &str, status: &str) -> Value {
        let revision = self.revision().to_string();
        let id = format!("{revision}:{action}");
        let (code, answer) = self.ask(&["choose", player, &revision, &id]);
        assert_eq!(answer["status"], status, "{action}: {answer}");
        assert_eq!(code, i32::from(status == "refused"));
        answer
    }
    /// A human's input that must submit nothing for an agent's faction.
    fn human_refused(&mut self, request: Value) {
        let before = (self.observe()["observation"].clone(), self.save_bytes());
        self.request(request.clone(), true);
        let view = self.settle();
        let notice = view["table"]["notice"].as_str().unwrap();
        assert!(
            notice.contains("is an agent's to play"),
            "{request}: {notice}"
        );
        assert_eq!((view["observation"].clone(), self.save_bytes()), before);
    }
    fn save_bytes(&self) -> Vec<u8> {
        std::fs::read(&self.save).unwrap()
    }
}

fn seat<'a>(view: &'a Value, faction: &str) -> &'a Value {
    let seats = view["table"]["faction_seats"]["factions"]
        .as_array()
        .unwrap();
    seats.iter().find(|s| s["faction"] == faction).unwrap()
}
fn holding(view: &Value) -> Vec<String> {
    serde_json::from_value(view["table"]["faction_seats"]["holding"].clone()).unwrap()
}
/// The decisions in a save.
fn decisions(save: &Path) -> Vec<String> {
    let record: Value = serde_json::from_slice(&std::fs::read(save).unwrap()).unwrap();
    serde_json::from_value(record["payload"]["decisions"].clone()).unwrap()
}

/// The save is, byte for byte, what the same decisions by hand write.
fn assert_by_hand(save: &Path) {
    use cuba_libre_unfolded::save::Session;
    let dir = tempfile::tempdir().unwrap();
    let by_hand = dir.path().join("by-hand.json");
    let mut session = Session::create_seeded(&by_hand, Session::KIDNAP_ROLLS_1).unwrap();
    for action in decisions(save) {
        let seen = session.observe();
        let id = format!("{}:{action}", seen.revision);
        let owner = seen.choices.iter().find(|c| c.id == id).unwrap().owner;
        session.submit(owner, seen.revision, &id).unwrap();
    }
    assert_eq!(
        std::fs::read(save).unwrap(),
        std::fs::read(&by_hand).unwrap()
    );
    drop(Session::open(save, None).unwrap());
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_an_agent_session_plays_a_handed_over_faction() {
    let mut app = start("faction-seats-play-", "issue-203", "1920x1280");
    // Every faction is the human's, and nothing listens.
    let view = app.observe();
    for faction in ["Government", "M26", "Directorio", "Syndicate"] {
        assert_eq!(seat(&view, faction)["state"], "human");
    }
    assert_eq!(view["table"]["faction_seats"]["socket"], Value::Null);
    let mut unheard = app.player("a", &["join", "directorio"]);
    assert_eq!(unheard.0.wait().unwrap().code(), Some(4));

    // Two sessions claim one seat; another faction is the human's.
    let view = app.key(DIRECTORIO);
    assert_eq!(seat(&view, "Directorio")["state"], "awaiting");
    let (code, answer) = app.ask(&["join", "government"]);
    assert_eq!((code, &answer["status"]), (1, &json!("refused")));
    let mut claims = [
        app.player("a", &["join", "directorio"]),
        app.player("b", &["join", "directorio"]),
    ];
    let ends: Vec<(i32, Value)> = claims.iter_mut().map(|c| app.answered(c)).collect();
    let mut codes: Vec<i32> = ends.iter().map(|(code, _)| *code).collect();
    codes.sort_unstable();
    assert_eq!(codes, [0, 1], "{ends:?}");
    let won = ends.iter().position(|(code, _)| *code == 0).unwrap();
    let player = ends[won].1["player"].as_str().unwrap().to_owned();
    // One session, one seat, by its tool's identifier.
    app.key(GOVERNMENT);
    let mut again = app.player(["a", "b"][won], &["join", "government"]);
    let (code, answer) = app.answered(&mut again);
    assert_eq!(code, 1);
    assert!(
        answer["error"]
            .as_str()
            .unwrap()
            .contains("already plays Directorio")
    );
    app.key(GOVERNMENT);
    let view = app.settle();
    assert_eq!(seat(&view, "Directorio")["state"], "at-table");
    assert_eq!(seat(&view, "Government")["state"], "human");

    // It waits while the human plays 26July, and is told its decision once.
    let mut waiting = app.player("w", &["wait", &player]);
    app.until_waiting("Directorio");
    for action in ["terror", "kidnap"] {
        app.choose(action);
    }
    app.roll();
    for action in ["terror:santiago", "end-terror"] {
        assert!(waiting.ended().is_none(), "{action} is the human's");
        app.choose(action);
    }
    let (code, told) = app.answered(&mut waiting);
    assert_eq!((code, &told["status"]), (0, &json!("decision")));
    let view = app.settle();
    assert_eq!(seat(&view, "Directorio")["state"], "thinking");
    assert_eq!(view["table"]["faction_seats"]["awaits"], "Directorio");
    // After a rolled Kidnap the observation holds the generator's state; no view does.
    assert!(view["observation"]["kidnap"]["random_before"].is_u64());
    assert_eq!(told["view"]["results"]["kidnap"]["die"], 1);
    assert!(
        told["view"]["results"]["kidnap"]
            .get("random_before")
            .is_none()
    );

    // The human may read the options, and submits none by a click or a key.
    let revision = app.revision();
    app.activate("option:event:shaded", revision);
    let commit = format!("{revision}:event:shaded");
    app.human_refused(json!({"op":"activate","revision":revision,"choice":commit}));
    app.capture("an-agent-decides");
    let mut view = app.observe();
    while view["table"]["choice"]["focus"] != commit.as_str() {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        view = app.settle();
    }
    app.human_refused(json!({"op":"key","revision":revision,"key":"Enter","focused":commit}));

    // The pinned briefing holds the session's choice. Putting it away lets it be applied,
    // and the human's focus stays.
    let sent = format!("{revision}:event:unshaded");
    let (code, answer) = app.ask(&["choose", &player, &revision.to_string(), &sent]);
    assert_eq!((code, &answer["status"]), (0, &json!("held")), "{answer}");
    let view = app.settle();
    assert_eq!(seat(&view, "Directorio")["holds_choice"], true);
    assert_eq!(holding(&view), ["a pinned or opened briefing"]);
    assert_eq!(view["observation"]["revision"], revision);
    let view = app.key("Escape");
    assert_eq!(view["observation"]["revision"], revision + 1, "{view}");
    assert_eq!(view["table"]["choice"]["focus"], commit.as_str());
    assert_eq!(decisions(&app.save).last().unwrap(), "event:unshaded");

    // A lost answer costs nothing: the identity gives the whole view again.
    let (_, seen) = app.ask(&["observe", &player]);
    assert_eq!(seen["view"]["revision"], revision + 1);
    assert_eq!(seen["view"]["decision"]["yours"], true);
    // In the second form, a refusal and the next decision are answered at once.
    let at = (revision + 1).to_string();
    let (code, answer) = app.ask(&["choose", &player, &at, &format!("{at}:pass"), "--wait"]);
    assert_eq!((code, &answer["status"]), (1, &json!("refused")));
    let own = format!("{at}:rally:beneficiary:directorio");
    let mut waiting = app.player("w", &["choose", &player, &at, &own, "--wait"]);
    app.until_waiting("Directorio");
    let view = app.settle();
    // Directorio has no Base: its Rally completes the Event, and the human turns the card.
    assert_eq!(view["observation"]["choices"][0]["id"], "7:advance-card");
    assert_eq!(view["table"]["faction_seats"]["awaits"], Value::Null);
    assert_eq!(seat(&view, "Directorio")["state"], "at-table");
    let (code, answer) = app.ask(&["choose", &player, "7", "7:advance-card"]);
    assert_eq!((code, &answer["status"]), (1, &json!("refused")));
    // The card is the human's, though the authority names Directorio. A session's choice
    // waits for the card in the air.
    let government = app.seat_agent(GOVERNMENT, "government");
    app.request(
        json!({"op":"activate","revision":7,"choice":"7:advance-card"}),
        true,
    );
    let view = app.observe();
    assert!(
        holding(&view).contains(&"card advance".to_owned()),
        "{view}"
    );
    assert!(view["pending"].to_string().contains("card advance"));
    let mut sent = app.player("g", &["choose", &government, "8", "8:government:assault"]);
    let (_, answer) = app.answered_while(&mut sent, 1, "card advance");
    assert_eq!(answer["status"], "held", "{answer}");
    let view = app.settle();
    assert_eq!(view["observation"]["executing"], "Government");
    let _ = waiting.ended();

    // Save · quit ends every session, and after a relaunch the human plays every faction.
    let mut waits = app.player(
        "g2",
        &["choose", &government, "9", "9:end-assault", "--wait"],
    );
    app.until_waiting("Government");
    let revision = app.revision();
    assert_eq!(revision, 10);
    let quit = app.request(
        json!({"op":"activate","revision":revision,"component":"quit"}),
        true,
    );
    assert_eq!(quit["status"], "exited", "{quit}");
    for asked in [&mut waiting, &mut waits] {
        let code = asked.0.wait().unwrap().code();
        let (_, answer) = asked.ended().unwrap();
        assert_eq!(
            (code, &answer["status"]),
            (Some(0), &json!("closed")),
            "{answer}"
        );
    }
    assert!(
        !app.player_socket().exists(),
        "the socket goes with the table"
    );
    assert_by_hand(&app.save);
    let saved = app.save_bytes();
    app.launch_as_is("1920x1280");
    let view = app.observe();
    assert_eq!(view["observation"]["revision"], revision);
    assert_eq!(seat(&view, "Government")["state"], "human");
    // Syndicate's options are listed and none is played: the build ends here.
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let (_, seen) = app.ask(&["observe", &syndicate]);
    assert_eq!(seen["view"]["decision"]["stands"], "build-ends");
    assert_eq!(seat(&app.settle(), "Syndicate")["state"], "build-ends");
    let (code, _) = app.ask(&["observe", &government]);
    assert_eq!(code, 1, "an identity ends with its table");
    assert_eq!(app.save_bytes(), saved);
}

/// 26July's choices wait for what moves and for what the human opened, and for nothing
/// its own steps drew.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_an_agents_choice_waits_for_the_table_and_the_human() {
    let mut app = start("faction-seats-held-", "issue-203", "1920x1280");
    let m26 = app.seat_agent(M26, "26july");
    app.decide(&m26, "terror", "applied");
    let view = app.settle();
    assert_eq!(view["table"]["choice"]["pinned"], Value::Null);

    // A space popover, whose button submits nothing for 26July either.
    app.activate("space:santiago", 1);
    app.activate("popover:row:kidnap", 1);
    app.human_refused(json!({"op":"activate","revision":1,"component":"popover:commit"}));
    app.decide(&m26, "transfer", "held");
    assert_eq!(holding(&app.settle()), ["an open popover"]);
    // Looking around holds nothing more and lets nothing through.
    app.request(json!({"op":"key","revision":1,"key":"ArrowLeft"}), true);
    let view = app.settle();
    assert_eq!(view["observation"]["revision"], 1);
    app.activate("space:santiago", 1);
    // Its own transfer modal is drawn, and is not the human's to close.
    let view = app.settle();
    assert_eq!(view["observation"]["revision"], 2, "{view}");
    assert_eq!(view["table"]["transfer"]["state"]["shown"], true);
    app.human_refused(json!({"op":"key","revision":2,"key":"Escape"}));

    // It asks the human's Government, whose modal holds its withdrawal; the human's Wait
    // moves the match on, so that choice is released and not applied.
    app.decide(&m26, "ask:0:3", "applied");
    app.decide(&m26, "end-terror", "held");
    let view = app.settle();
    assert_eq!(holding(&view), ["the transfer modal"]);
    assert_eq!(seat(&view, "M26")["state"], "at-table");
    let view = app.key("Escape");
    assert_eq!(view["observation"]["revision"], 4);
    let (_, seen) = app.ask(&["observe", &m26]);
    assert_eq!(seen["released"][0]["id"], "3:end-terror", "{seen}");
    assert_eq!(seen["view"]["decision"]["faction"], "Government");
    // The human opens it again and gives 2 of Government's 6.
    app.choose("reopen");
    app.choose("authorize:2");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([4, 2, 5, 15])
    );

    // A pinned card tag, and the ribbon's tag.
    let revision = app.revision();
    app.activate("card:preview", revision);
    app.decide(&m26, "kidnap", "held");
    assert_eq!(holding(&app.settle()), ["a pinned card tag"]);
    let view = app.key("Escape");
    assert_eq!(view["observation"]["kidnap_open"], true, "{view}");
    // Its own Kidnap step pins nothing of the human's.
    assert_eq!(view["table"]["choice"]["pinned"], Value::Null);
    app.decide(&m26, "kidnap:cancel", "applied");
    let revision = app.revision();
    app.activate("briefing-ribbon", revision);
    app.decide(&m26, "kidnap", "held");
    assert_eq!(holding(&app.settle()), ["a pinned or opened briefing"]);
    app.activate("briefing-ribbon", revision);
    assert_eq!(app.settle()["observation"]["kidnap_open"], true);

    // A hover that dwells holds nothing, over a map the human just zoomed.
    let revision = app.revision();
    app.point("space:las-villas");
    app.request(
        json!({"op":"scroll","revision":revision,"component":"space:las-villas","y":6}),
        true,
    );
    app.point("area:standings");
    let pointed = json!({"op":"point","revision":revision,"component":"space:las-villas"});
    app.request(pointed, true);
    let cancel = format!("{revision}:kidnap:cancel");
    let mut sent = app.player("c", &["choose", &m26, &revision.to_string(), &cancel]);
    let (_, answer) = app.answered_while(&mut sent, 1, "hover dwell");
    assert_eq!(answer["status"], "applied", "{answer}");
    app.settle();
    app.decide(&m26, "kidnap", "applied");

    let revision = app.revision();
    let roll = format!("{revision}:kidnap:roll");
    let mut rolled = app.player("r", &["choose", &m26, &revision.to_string(), &roll]);
    // One frame at a time, so the die is seen in the air.
    let view = loop {
        let view = app.request(json!({"op":"advance","steps":1}), true);
        if view["observation"]["revision"] == revision + 1 {
            break view;
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        view["table"]["die"]["landed"], false,
        "{}",
        view["table"]["die"]
    );
    while rolled.ended().is_none() {
        std::thread::sleep(Duration::from_millis(20));
    }
    // The next choice waits for the die to land and to show its result.
    let at = revision + 1;
    let terror = format!("{at}:terror:santiago");
    let mut next = app.player("n", &["choose", &m26, &at.to_string(), &terror]);
    let mut seen = vec![];
    let view = loop {
        let view = app.request(json!({"op":"advance","steps":6}), true);
        if view["observation"]["revision"] == at + 1 {
            break view;
        }
        seen.push((
            view["table"]["die"]["landed"].clone(),
            holding(&view),
            view["table"]["die"]["seek"].clone(),
        ));
        std::thread::sleep(Duration::from_millis(5));
    };
    let (_, answer) = app.answered(&mut next);
    assert_eq!(answer["status"], "held", "{answer}");
    let moving = seen
        .iter()
        .filter(|s| s.1.contains(&"Kidnap die landing".into()));
    let shown: Vec<_> = seen.iter().filter(|s| s.1 == ["Kidnap result"]).collect();
    assert!(moving.count() > 2, "{seen:?}");
    // Two seconds, in steps of a tenth.
    assert!((19..=21).contains(&shown.len()), "{} {seen:?}", shown.len());
    assert!(shown.iter().all(|s| s.0 == true), "{shown:?}");
    assert!(view["table"]["die"].is_null(), "{}", view["table"]["die"]);
    // Continue closes the result at once.
    assert_by_hand(&app.save);
}

/// The seat goes back to the human before a choice, with the session waiting, with one
/// held, and with a card or a die in the air.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_the_human_takes_a_seat_back_at_any_time() {
    let mut app = start("faction-seats-back-", "issue-203", "1920x1280");
    // Waiting, before a choice.
    let m26 = app.seat_agent(M26, "26july");
    let (_, told) = app.ask(&["wait", &m26]);
    assert_eq!(told["status"], "decision");
    let (_, again) = app.ask(&["observe", &m26]);
    assert_eq!(again["status"], "observed");
    let (code, answer) = app.ask(&["wait", &m26]);
    assert_eq!(
        (code, &answer["status"]),
        (1, &json!("refused")),
        "told once"
    );
    let syndicate = app.seat_agent(SYNDICATE, "syndicate");
    let mut waiting = app.player("s", &["wait", &syndicate]);
    app.until_waiting("Syndicate");
    let view = app.key(SYNDICATE);
    assert_eq!(seat(&view, "Syndicate")["state"], "human");
    let (_, told) = app.answered(&mut waiting);
    assert_eq!(told["status"], "closed", "{told}");

    // A held choice is not applied.
    app.decide(&m26, "terror", "applied");
    app.activate("space:santiago", 1);
    app.decide(&m26, "kidnap", "held");
    let view = app.key(M26);
    assert_eq!(seat(&view, "M26")["state"], "human");
    app.activate("space:santiago", 1);
    assert_eq!(app.settle()["observation"]["revision"], 1);
    let (_, answer) = app.ask(&["observe", &m26]);
    assert_eq!(answer["status"], "closed");
    assert!(
        answer["reason"]
            .as_str()
            .unwrap()
            .contains("was not applied")
    );
    // The human plays it again.
    app.choose("kidnap");
    assert_eq!(app.observe()["observation"]["kidnap_open"], true);

    // A seat nobody holds waits, and the human's input does nothing for it. With the die
    // in the air it is still the human's to take back.
    app.key(M26);
    let revision = app.revision();
    let cancel = format!("{revision}:kidnap:cancel");
    app.human_refused(json!({"op":"activate","revision":revision,"choice":cancel}));
    assert_eq!(seat(&app.settle(), "M26")["state"], "awaiting");
    app.key(M26);
    let roll = format!("{revision}:kidnap:roll");
    app.request(
        json!({"op":"activate","revision":revision,"choice":roll}),
        true,
    );
    app.request(json!({"op":"key","revision":revision + 1,"key":M26}), true);
    let view = app.observe();
    assert_eq!(seat(&view, "M26")["state"], "awaiting");
    assert_eq!(view["table"]["die"]["landed"], false);
    app.request(json!({"op":"key","revision":revision + 1,"key":M26}), true);
    assert_eq!(seat(&app.observe(), "M26")["state"], "human");
    app.settle();

    // A session whose command died keeps its seat, and its next wait is told.
    app.activate("kidnap-die:continue", revision + 1);
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    let mut dead = app.player("d", &["wait", &directorio]);
    app.until_waiting("Directorio");
    dead.0.kill().unwrap();
    dead.0.wait().unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    while seat(&app.settle(), "Directorio")["waits"] == true {
        assert!(Instant::now() < end, "a dead client's wait is held");
    }
    for action in ["terror:santiago", "end-terror"] {
        app.choose(action);
    }
    assert_eq!(seat(&app.observe(), "Directorio")["state"], "thinking");
    let (_, told) = app.ask(&["wait", &directorio]);
    assert_eq!(told["status"], "decision");
    assert_by_hand(&app.save);

    // The development seat keeps the table running for it with the shared loop.
    let mut kept = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .args(["seat-wait", app.socket.to_str().unwrap(), "--timeout", "60"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    for choice in ["event:shaded", "resolve-shaded"] {
        let at = decisions(&app.save).len().to_string();
        let mut asked = app.player(
            "k",
            &["choose", &directorio, &at, &format!("{at}:{choice}")],
        );
        let end = Instant::now() + Duration::from_secs(30);
        let (_, answer) = loop {
            if let Some(ended) = asked.ended() {
                break ended;
            }
            assert!(Instant::now() < end, "the table does not run");
            std::thread::sleep(Duration::from_millis(50));
        };
        assert!(["applied", "held"].contains(&answer["status"].as_str().unwrap()));
        while !decisions(&app.save).contains(&choice.to_owned()) {
            assert!(Instant::now() < end, "{choice} is not applied");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    // Only the card is left, which is the human's: the loop returns.
    assert!(kept.wait().unwrap().success());
    let mut printed = String::new();
    std::io::Read::read_to_string(kept.stdout.as_mut().unwrap(), &mut printed).unwrap();
    let last: Value = serde_json::from_str(printed.lines().last().unwrap()).unwrap();
    assert_eq!(last["awaits"], Value::Null, "{printed}");
    assert!(printed.contains("\"awaits\":\"Directorio\""), "{printed}");
}
