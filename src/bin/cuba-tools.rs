use anyhow::{Context, Result, anyhow, bail};
use cuba_libre_unfolded::{
    player_folders::{CLIENT, Places},
    seat_brief::BriefWriter,
    seat_wait,
};
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
use unfolded_seat_client::{
    self as seat_client, Connection, Outcome,
    hosting::{self, Kept, Next},
};

const USAGE: &str = "Usage: cuba-tools seat [--brief] SOCKET [JSON_REQUEST ...]
       cuba-tools seat-wait SOCKET [--timeout SECONDS]
       cuba-tools player-folders

seat: Without requests, reads JSON lines from stdin. Exits nonzero for refused/missed/skipped/malformed or missing replies.
--brief prints each reply as one short JSON line: its status, revision, settled and error, and each rules choice's id, label and supported, with the explanation of an unsupported one.

seat-wait: keeps a held seat's windowless table running, so that agent sessions can play its handed-over factions, and answers nothing itself. Prints a line whenever the revision, the faction the table waits for or what holds an agent's choice changes, and returns when the table needs this seat: no agent's faction is awaited, or something opened here holds an agent's choice. Exits 0 then, 3 when the time (240 s unless given) is up, 4 when no seat listens there, 1 otherwise.

player-folders: writes the folders an agent player's sessions run in, one for Claude Code and one for Codex, and installs their client, the cuba-player beside this program. Refuses, and names, any file it did not write.";

/// How long past its time a seat that stopped answering is waited for.
const GRACE: Duration = Duration::from_secs(10);

fn seat(mut args: Vec<String>) -> Result<u8> {
    let brief = args.first().is_some_and(|flag| flag == "--brief");
    if brief {
        args.remove(0);
    }
    let [socket, given @ ..] = &args[..] else {
        bail!("{USAGE}");
    };
    let requests = seat_client::requests(given, &mut std::io::stdin().lock())?;
    let connection = Connection::open(Path::new(socket))?;
    let deadline = Instant::now() + Duration::from_secs(180);
    let stdout = std::io::stdout().lock();
    let mut output: Box<dyn Write> = if brief {
        Box::new(BriefWriter::new(stdout))
    } else {
        Box::new(stdout)
    };
    let outcome = connection.send(&requests, &mut output, || {
        if Instant::now() >= deadline {
            bail!("Seat response deadline exceeded");
        }
        Ok(())
    })?;
    output.flush()?;
    if let Outcome::Short { answered } = outcome {
        eprintln!("Seat answered {answered}/{} requests", requests.len());
    }
    Ok(u8::from(outcome != Outcome::Answered))
}

fn wait(args: &[String]) -> Result<u8> {
    let (socket, seconds) = match args {
        [socket] => (socket, 240),
        [socket, flag, seconds] if flag == "--timeout" => {
            (socket, seconds.parse().context("--timeout needs seconds")?)
        }
        _ => bail!("{USAGE}"),
    };
    let until = Instant::now()
        .checked_add(Duration::from_secs(seconds))
        .context("--timeout is too long")?;
    let mut shown = None;
    let kept = hosting::keep_running(
        Path::new(socket),
        until,
        || match Instant::now() >= until + GRACE {
            true => bail!("The seat did not answer in time."),
            false => Ok(()),
        },
        |pause| {
            std::thread::sleep(pause);
            Ok(())
        },
        |answer| {
            let seen = seat_wait::Seen::of(answer)
                .ok_or_else(|| anyhow!("The seat's answer describes no faction seats: {answer}"))?;
            if answer["table"]["settled"] != true {
                return Ok(Next::Go);
            }
            if shown.as_ref() != Some(&seen) {
                println!("{}", serde_json::to_string(&seen)?);
                std::io::stdout().flush()?;
            }
            let needed = seen.needs_the_seat();
            shown = Some(seen);
            Ok(if needed { Next::Stop(()) } else { Next::Rest })
        },
    )
    .map_err(|failed| anyhow!("{failed}"))?;
    Ok(match kept {
        Kept::Stopped(()) => 0,
        Kept::TimedOut => 3,
        Kept::Gone(gone) => {
            eprintln!("cuba-tools: {gone}");
            4
        }
    })
}

fn player_folders() -> Result<u8> {
    let places = Places::here().context("Cannot find the data directory: set HOME.")?;
    let built = std::env::current_exe()?.with_file_name(CLIENT);
    if !built.is_file() {
        bail!(
            "{} is missing. Build it: cargo build --locked --bin {CLIENT}",
            built.display()
        );
    }
    places.write(&built)?;
    println!(
        "Claude Code's folder: {}\nCodex's folder: {}\nClient: {}",
        places.claude_code().display(),
        places.codex().display(),
        places.client().display()
    );
    Ok(0)
}

fn run() -> Result<u8> {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        bail!("{USAGE}");
    };
    let args: Vec<_> = args.collect();
    match command.as_str() {
        "seat" => seat(args),
        "seat-wait" => wait(&args),
        "player-folders" if args.is_empty() => player_folders(),
        _ => bail!("{USAGE}"),
    }
}
fn main() {
    match run() {
        Ok(status) => std::process::exit(status.into()),
        Err(e) => {
            eprintln!("cuba-tools: {e:#}");
            std::process::exit(1);
        }
    }
}
