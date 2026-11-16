use anyhow::{Result, bail};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use unfolded_seat_client::{self as seat_client, Connection, Outcome};
fn run() -> Result<bool> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("seat") || args.len() < 2 {
        bail!(
            "Usage: cuba-tools seat SOCKET [JSON_REQUEST ...]\nWithout requests, reads JSON lines from stdin. Exits nonzero for refused/missed/skipped/malformed or missing replies."
        );
    }
    let requests = seat_client::requests(&args[2..], &mut std::io::stdin().lock())?;
    let connection = Connection::open(Path::new(&args[1]))?;
    let deadline = Instant::now() + Duration::from_secs(180);
    let outcome = connection.send(&requests, &mut std::io::stdout().lock(), || {
        if Instant::now() >= deadline {
            bail!("Seat response deadline exceeded");
        }
        Ok(())
    })?;
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
