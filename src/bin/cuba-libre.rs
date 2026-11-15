use cuba_libre_unfolded::tabletop::{self, Options};
use std::path::PathBuf;
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut sized_seat = false;
    let mut options = Options {
        art: PathBuf::from("art/atlas"),
        cards: PathBuf::new(),
        save: PathBuf::new(),
        recover: None,
        socket: None,
        agent: false,
        seat_size: bevy::prelude::UVec2::new(1920, 1280),
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--art" => options.art = args.next().ok_or("--art needs a directory")?.into(),
            "--cards" => options.cards = args.next().ok_or("--cards needs a directory")?.into(),
            "--save" => options.save = args.next().ok_or("--save needs a file")?.into(),
            "--recover-from" => {
                options.recover = Some(
                    args.next()
                        .ok_or("--recover-from needs the original save path")?
                        .into(),
                )
            }
            "--socket" => options.socket = Some(args.next().ok_or("--socket needs a path")?.into()),
            "--agent-seat" => options.agent = true,
            "--seat-size" => {
                sized_seat = true;
                let size = args.next().ok_or("--seat-size needs WIDTHxHEIGHT")?;
                let (width, height) = size
                    .split_once('x')
                    .ok_or("--seat-size needs WIDTHxHEIGHT")?;
                let width: u32 = width.parse().map_err(|_| "Invalid seat width")?;
                let height: u32 = height.parse().map_err(|_| "Invalid seat height")?;
                if !(640..=4096).contains(&width) || !(480..=2160).contains(&height) {
                    return Err("Seat size must be within 640x480 through 4096x2160".into());
                }
                options.seat_size = bevy::prelude::UVec2::new(width, height);
            }
            "--help" => {
                println!(
                    "cuba-libre --cards DIRECTORY --save FILE [--art art/atlas] [--recover-from ORIGINAL] [--agent-seat [--socket SOCKET] [--seat-size WIDTHxHEIGHT]]\nPrepared Terror task candidate. Recovery reads ORIGINAL.previous into an unused FILE. Development sessions idle out after 30 minutes."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }
    if options.cards.as_os_str().is_empty() || options.save.as_os_str().is_empty() {
        return Err("Both --cards DIRECTORY and --save FILE are required; use --help.".into());
    }
    if options.socket.is_some() && !options.agent {
        return Err("--socket requires --agent-seat".into());
    }
    if sized_seat && !options.agent {
        return Err("--seat-size requires --agent-seat".into());
    }
    tabletop::run(options)
}
fn main() {
    if let Err(error) = run() {
        eprintln!("cuba-libre: {error}");
        std::process::exit(1);
    }
}
