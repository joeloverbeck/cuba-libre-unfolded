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
            let end = Instant::now() + Duration::from_secs(10);
            while self.human(|host, _| host.waits(Faction::Directorio)) != waits {
                assert!(Instant::now() < end, "its seat's wait is not {waits}");
            }
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
}
