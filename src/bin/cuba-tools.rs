use anyhow::{Result, bail};
use cuba_libre_unfolded::seat_brief::BriefWriter;
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
use unfolded_seat_client::{self as seat_client, Connection, Outcome};
fn run() -> Result<bool> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let brief = args.get(1).is_some_and(|flag| flag == "--brief");
    if brief {
        args.remove(1);
    }
    if args.first().map(String::as_str) != Some("seat") || args.len() < 2 {
        bail!(
            "Usage: cuba-tools seat [--brief] SOCKET [JSON_REQUEST ...]\nWithout requests, reads JSON lines from stdin. Exits nonzero for refused/missed/skipped/malformed or missing replies.\n--brief prints each reply as one short JSON line: its status, revision, settled and error, and each rules choice's id, label and supported, with the explanation of an unsupported one."
        );
    }
    let requests = seat_client::requests(&args[2..], &mut std::io::stdin().lock())?;
    let connection = Connection::open(Path::new(&args[1]))?;
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
    Ok(outcome == Outcome::Answered)
}
fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("cuba-tools: {e:#}");
            std::process::exit(1);
        }
    }
}
