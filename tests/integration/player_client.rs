//! The player's client, `cuba-player`, over the shared socket against a faction seat host
//! whose frames a thread runs: what it sends, prints and exits with, and that a client
//! that died ends its wait. No window and no model (#203).
use cuba_libre_unfolded::seat_wait::Seen;
use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, Output, Stdio},
};

fn client(socket: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cuba-player"));
    command
        .arg("--socket")
        .arg(socket)
        .args(args)
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("CODEX_THREAD_ID")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn ended(output: &Output) -> (i32, Value) {
    let answer = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.code().unwrap(), answer)
}

fn ask(socket: &Path, args: &[&str]) -> (i32, Value) {
    ended(&client(socket, args).output().unwrap())
}

#[test]
fn a_command_the_client_cannot_send_says_why() {
    let nowhere = Path::new("/nonexistent/cuba.sock");
    let (code, _) = ask(nowhere, &["observe", "p"]);
    assert_eq!(code, 4);
    for wrong in [
        &["join"][..],
        &["join", "directorio", "government"],
        &["observe"],
        &["wait", "p", "--wait"],
        &["wait", "p", "--timeout", "soon"],
        &["wait", "p", "--timeout", "0"],
        &["choose", "p", "five", "5:pass"],
        &["choose", "p", "5", "5:pass", "--timeout", "9"],
        &["say", "p", "hello"],
    ] {
        let output = client(nowhere, wrong).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{wrong:?}");
        assert!(!output.stderr.is_empty() && output.stdout.is_empty());
    }
    let help = client(nowhere, &["--help"]).output().unwrap();
    assert!(help.status.success());
}

/// When the development seat's loop gives its driver the table back.
#[test]
fn seat_wait_returns_for_the_human_and_for_what_holds_a_choice() {
    let needs = |awaits: Value, holding: Value| {
        Seen::of(&json!({
            "observation": {"revision": 5},
            "table": {"faction_seats": {"awaits": awaits, "holding": holding}},
        }))
        .unwrap()
        .needs_the_seat()
    };
    assert!(!needs(json!("Directorio"), json!([])));
    assert!(needs(json!("Directorio"), json!(["an open popover"])));
    // The human decides, or only the card is left, or the build ends.
    assert!(needs(Value::Null, Value::Null));
    let other = json!({"observation": {"revision": 5}, "table": {}});
    assert_eq!(Seen::of(&other), None);
}

#[cfg(feature = "tabletop")]
mod over_the_socket {
    use super::*;
    use cuba_libre_unfolded::{
        Faction, Game,
        faction_seats::{Answer, Host, Player, Table, closed},
        save::Session,
    };
    use std::{
        process::Child,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    use unfolded_tabletop::agent_transport::{Protocol, Reply, Transport};

    struct Words;
    impl Protocol for Words {
        fn players(&self) -> &str {
            "Faction seats"
        }
        fn closed(&self) -> String {
            closed("The table closed.").to_string()
        }
    }

    struct Room(Session);
    impl Table for Room {
        fn game(&self) -> &Game {
            self.0.game()
        }
        fn reports(&self) -> Vec<cuba_libre_unfolded::reports::Report> {
            self.0.reports()
        }
        fn settled(&self) -> bool {
            true
        }
        fn commit(&mut self, faction: Faction, revision: u64, choice: &str) -> Result<(), String> {
            let submitted = self.0.submit(faction, revision, choice);
            submitted.map_err(|refused| refused.to_string())
        }
    }

    /// What the human does between frames.
    type Human = Box<dyn FnOnce(&mut Host<Reply>, &mut Session) + Send>;

    /// A table whose frames a thread runs until it is dropped.
    struct Running {
        human: Option<mpsc::Sender<Human>>,
        frames: Option<thread::JoinHandle<()>>,
    }
    impl Running {
        fn open(socket: &Path, session: Session) -> Self {
            let transport = Transport::open(socket, Words, None).unwrap();
            let (human, asked) = mpsc::channel::<Human>();
            let frames = thread::spawn(move || {
                let mut room = Room(session);
                let mut host = Host::new();
                let send = |answers: Vec<Answer<Reply>>| {
                    for (reply, answer) in answers {
                        reply.send(answer.to_string());
                    }
                };
                loop {
                    match asked.recv_timeout(Duration::from_millis(5)) {
                        Ok(act) => act(&mut host, &mut room.0),
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                    let now = Instant::now();
                    host.end_hung_up(Reply::hung_up, now);
                    for incoming in transport.take() {
                        send(host.receive(&incoming.line, incoming.reply, &mut room, now));
                    }
                    send(host.poll(&mut room, now));
                }
                send(host.close("The table closed."));
            });
            Self {
                human: Some(human),
                frames: Some(frames),
            }
        }
        /// What `act` finds between two frames.
        fn human<T: Send + 'static>(
            &self,
            act: impl FnOnce(&mut Host<Reply>, &mut Session) -> T + Send + 'static,
        ) -> T {
            let (done, found) = mpsc::channel();
            let act: Human = Box::new(move |host, session| done.send(act(host, session)).unwrap());
            self.human.as_ref().unwrap().send(act).unwrap();
            found.recv().unwrap()
        }
        fn until_it_waits(&self, waits: bool) {
            self.until_waits(Faction::Directorio, waits);
        }
        fn until_waits(&self, faction: Faction, waits: bool) {
            let end = Instant::now() + Duration::from_secs(10);
            while self.human(move |host, _| host.waits(faction)) != waits {
                assert!(Instant::now() < end, "{faction:?}'s wait is not {waits}");
            }
        }
        /// The position, the save, the generator and who holds each seat.
        fn standing(&self, save: &Path) -> (Game, Vec<u8>, u64, [Player; 4]) {
            let save = save.to_owned();
            self.human(move |host, session| {
                (
                    session.game().clone(),
                    std::fs::read(&save).unwrap(),
                    session.game().random_state(),
                    host.players(),
                )
            })
        }
        fn play(&self, actions: &'static [&'static str]) {
            self.human(move |_, session| {
                for action in actions {
                    let seen = session.observe();
                    let id = format!("{}:{action}", seen.revision);
                    let owner = seen.choices.iter().find(|c| c.id == id).unwrap().owner;
                    session.submit(owner, seen.revision, &id).unwrap();
                }
            });
        }
    }
    impl Drop for Running {
        fn drop(&mut self) {
            drop(self.human.take());
            let ended = self.frames.take().unwrap().join();
            if !thread::panicking() {
                ended.unwrap();
            }
        }
    }

    fn spawn(socket: &Path, args: &[&str]) -> Child {
        client(socket, args).spawn().unwrap()
    }

    #[test]
    fn a_session_plays_its_faction_with_the_client() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("players.sock");
        let save = dir.path().join("match.json");
        let table = Running::open(&socket, Session::create_seeded(&save, 5).unwrap());
        let socket = socket.as_path();
        assert_eq!(ask(socket, &["join", "Directorio"]).0, 1);
        table.human(|host, _| host.hand_over(Faction::Directorio));
        let (code, joined) = ask(socket, &["join", "Directorio"]);
        assert_eq!((code, &joined["status"]), (0, &json!("joined")));
        assert_eq!(joined["brief"]["faction"], "Directorio");
        let player = joined["player"].as_str().unwrap();
        // It is known by its identity, and told its brief again.
        let (code, again) = ask(socket, &["join", "Directorio", "--player", player]);
        assert_eq!((code, &again["view"]["faction"]), (1, &json!("Directorio")));
        assert_eq!(again["brief"], joined["brief"]);

        // A wait with a period, and one whose client died.
        let (code, answer) = ask(socket, &["wait", player, "--timeout", "1"]);
        assert_eq!((code, &answer["status"]), (3, &json!("nothing_yet")));
        let mut dead = spawn(socket, &["wait", player]);
        table.until_it_waits(true);
        dead.kill().unwrap();
        dead.wait().unwrap();
        table.until_it_waits(false);
        // Its event is told to the next.
        let waiting = spawn(socket, &["wait", player]);
        table.until_it_waits(true);
        table.play(&[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
        ]);
        let (code, told) = ended(&waiting.wait_with_output().unwrap());
        assert_eq!((code, &told["status"]), (0, &json!("decision")));

        // Both forms of submission.
        let (code, answer) = ask(socket, &["choose", player, "5", "5:directorio:pass"]);
        assert_eq!((code, &answer["status"]), (1, &json!("refused")));
        let (code, answer) = ask(socket, &["choose", player, "5", "5:event:shaded"]);
        assert_eq!((code, &answer["status"]), (0, &json!("applied")));
        let last = ["choose", player, "6", "6:resolve-shaded"];
        let and_wait = [&last[..], &["--wait", "--timeout", "1"]].concat();
        let (code, answer) = ask(socket, &and_wait);
        assert_eq!((code, &answer["choice"]["status"]), (3, &json!("applied")));
        let (code, answer) = ask(socket, &["observe", player]);
        assert_eq!((code, &answer["view"]["revision"]), (0, &json!(7)));

        // Leaving, and the table's end.
        assert_eq!(ask(socket, &["leave", player]).1["status"], "left");
        let holder = table.human(|host, _| host.player(Faction::Directorio));
        assert_eq!(holder, Player::Nobody);
        let (_, joined) = ask(socket, &["join", "Directorio"]);
        let waiting = spawn(socket, &["wait", joined["player"].as_str().unwrap()]);
        table.until_it_waits(true);
        drop(table);
        let (code, told) = ended(&waiting.wait_with_output().unwrap());
        assert_eq!((code, &told["status"]), (0, &json!("closed")));
        assert_eq!(ask(socket, &["observe", "p"]).0, 4);
    }

    /// Over the socket, a choice at a stale revision and one by a session that holds no
    /// seat are refused, and the position, the save and the generator stay as they were.
    /// An agent giver's Wait sets its decision aside: its next wait is held for what comes
    /// next and not told the decision again (#197's review).
    #[test]
    fn refusals_change_nothing_and_a_giver_waits_over_the_socket() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("players.sock");
        let save = dir.path().join("match.json");
        let table = Running::open(&socket, Session::create_seeded(&save, 5).unwrap());
        let socket = socket.as_path();
        table.human(|host, _| host.hand_over(Faction::Syndicate));
        let (_, joined) = ask(socket, &["join", "Syndicate"]);
        let player = joined["player"].as_str().unwrap();
        table.play(&["terror", "transfer", "ask:3:2"]);
        let before = table.standing(&save);
        assert_eq!(before.0.observe().revision, 3);
        for (session, revision, id) in [
            (player, "2", "2:authorize:2"),
            (player, "3", "2:authorize:2"),
            ("never-seated", "3", "3:authorize:2"),
        ] {
            let (code, answer) = ask(socket, &["choose", session, revision, id]);
            assert_eq!((code, &answer["status"]), (1, &json!("refused")), "{id}");
            assert_eq!(table.standing(&save), before, "{session} {id}");
        }

        let (code, answer) = ask(socket, &["choose", player, "3", "3:wait"]);
        assert_eq!(
            (code, &answer["status"]),
            (0, &json!("applied")),
            "{answer}"
        );
        assert_eq!(answer["view"]["decision"]["yours"], true);
        let mut held = spawn(socket, &["wait", player]);
        table.until_waits(Faction::Syndicate, true);
        thread::sleep(Duration::from_secs(1));
        assert!(held.try_wait().unwrap().is_none(), "told again");
        table.human(|host, session| {
            for (reply, answer) in host.take_back(Faction::Syndicate) {
                reply.send(answer.to_string());
            }
            assert_eq!(session.observe().revision, 4);
        });
        let (code, told) = ended(&held.wait_with_output().unwrap());
        assert_eq!((code, &told["status"]), (0, &json!("closed")), "{told}");
    }
}
