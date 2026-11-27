//! An agent session's client: one request to the running match, and its one answer,
//! printed as the table wrote it. The requests and their rules are the host's
//! (`cuba_libre_unfolded::faction_seats`).
use cuba_libre_unfolded::faction_seats::default_socket;
use serde_json::{Value, json};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};
use unfolded_agent_players::client::{self, Unanswered};

const HELP: &str = "\
cuba-player [--socket SOCKET] COMMAND

Play one faction of a running Cuba Libre match. Each command prints one line of JSON.

  join FACTION [--player PLAYER]            government, 26july, directorio or syndicate.
                                            Keep the answer's \"player\": it is your
                                            identity on every later command
  observe PLAYER                            your whole view
  publish PLAYER REVISION CARD TEXT          issue optional public faction speech
  choose PLAYER REVISION ID                 make a choice of your view at its revision
  choose PLAYER REVISION ID --wait [--timeout SECONDS]
                                            make it, and then wait
  wait PLAYER [--timeout SECONDS]           until your faction has a decision
  leave PLAYER                              your seat waits for another session

A join's answer also holds your seat's \"brief\". A wait has no time limit unless you give
one. Join again with --player only to learn whether your seat is still yours, and to read
its brief again: a session that holds a seat is refused another.

Answers (\"status\"): joined, observed, published, left; applied, held (it waits for the table to
settle) or refused for a choice; decision, released (a held choice was not applied, with
the reason), replaced (a newer wait of yours), nothing_yet (your time limit) or closed
(your session ended: stop) for a wait.

Exit status: 0 answered, 1 refused, 3 nothing yet, 4 no match listens there, 2 anything
else. SOCKET is the one the human gave you, else $CUBA_LIBRE_PLAYER_SOCKET, else
cuba-libre-players.sock in $XDG_RUNTIME_DIR.";

/// The request a command line spells, and how long its answer may take at most.
fn request(args: &[String]) -> Result<(Value, Option<Duration>), String> {
    let (command, rest) = args.split_first().ok_or("Name a command; see --help.")?;
    // TEXT is a single argument and may start with a dash or contain an option's spelling.
    if command == "publish" {
        let [player, revision, card, text] = rest else {
            return Err("Use publish PLAYER REVISION CARD TEXT; see --help.".into());
        };
        let revision: u64 = revision
            .parse()
            .map_err(|_| "REVISION is the number in your view.")?;
        let card: u8 = card
            .parse()
            .map_err(|_| "CARD is the current card's number in your view.")?;
        return Ok((
            json!({"op": "publish", "player": player, "revision": revision, "card": card, "text": text}),
            None,
        ));
    }
    let mut given = vec![];
    let (mut wait, mut timeout, mut player) = (false, None, None);
    let mut rest = rest.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--wait" if command == "choose" => wait = true,
            "--timeout" if command == "choose" || command == "wait" => {
                let seconds = rest.next().and_then(|s| s.parse::<u64>().ok());
                timeout = Some(
                    seconds
                        .filter(|s| *s > 0)
                        .ok_or("--timeout needs a number of seconds")?,
                );
            }
            "--player" if command == "join" => {
                player = Some(rest.next().ok_or("--player needs your identity")?);
            }
            option if option.starts_with("--") => {
                return Err(format!("{command} takes no {option}; see --help."));
            }
            _ => given.push(arg.as_str()),
        }
    }
    let mut request = match (command.as_str(), &given[..]) {
        ("join", [faction]) => json!({"op": "join", "faction": faction}),
        ("observe" | "leave" | "wait", [player]) => json!({"op": command, "player": player}),
        ("choose", [player, revision, id]) => {
            let revision: u64 = revision
                .parse()
                .map_err(|_| "REVISION is the number in your view.")?;
            json!({"op": "choose", "player": player, "revision": revision, "id": id})
        }
        _ => return Err("That is no command; see --help.".into()),
    };
    if command == "join" {
        if let Some(player) = player {
            request["player"] = json!(player);
        }
        if let Some(session) = client::session() {
            request["session"] = json!(session);
        }
    }
    if wait {
        request["wait"] = json!(true);
    }
    let waits = wait || command == "wait";
    if timeout.is_some() && !waits {
        return Err("--timeout goes with --wait.".into());
    }
    if let Some(seconds) = timeout {
        request["timeout_seconds"] = json!(seconds);
    }
    // The table answers at the period it was asked for; this bounds a table that stops.
    let within = timeout.map(|seconds| Duration::from_secs(seconds.saturating_add(30)));
    Ok((request, within))
}

/// Why there is no answer, and the status to exit with.
fn unanswered(failed: Unanswered<String>, socket: &Path) -> (String, u8) {
    let words = |what: &str, error: &dyn Display| format!("{what} {}: {error}", socket.display());
    match failed {
        Unanswered::NotListening(error) => (
            words(
                "No match takes agent sessions on this socket. The human hands a faction over in the game first;",
                &error,
            ),
            4,
        ),
        Unanswered::Closed => (
            "The table closed the connection without answering.".into(),
            2,
        ),
        Unanswered::Stopped(reason) => (reason, 2),
        other => (words("Cannot ask the match on", &other), 2),
    }
}

fn run() -> Result<u8, (String, u8)> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut socket = default_socket();
    if args.first().is_some_and(|arg| arg == "--socket") {
        if args.len() < 2 {
            return Err(("--socket needs a path".into(), 2));
        }
        socket = PathBuf::from(args.remove(1));
        args.remove(0);
    }
    if args.is_empty()
        || (args[0] != "publish" && args.iter().any(|arg| arg == "--help"))
        || (args.len() == 2 && args[0] == "publish" && args[1] == "--help")
    {
        println!("{HELP}");
        return Ok(0);
    }
    let (request, within) = request(&args).map_err(|reason| (reason, 2))?;
    let deadline = within.and_then(|within| Instant::now().checked_add(within));
    let answer = client::request(&socket, &request.to_string(), || {
        match deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            true => Err("The table did not answer in time.".to_owned()),
            false => Ok(()),
        }
    })
    .map_err(|failed| unanswered(failed, &socket))?;
    answer
        .write_to(&mut std::io::stdout().lock())
        .map_err(|error| (error.to_string(), 2))?;
    Ok(match answer.status().as_deref() {
        Some("refused") => 1,
        Some("nothing_yet") => 3,
        Some(_) => 0,
        None => 2,
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(status) => ExitCode::from(status),
        Err((reason, status)) => {
            eprintln!("cuba-player: {reason}");
            ExitCode::from(status)
        }
    }
}
