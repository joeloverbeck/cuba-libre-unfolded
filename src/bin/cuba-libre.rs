use cuba_libre_unfolded::tabletop::{self, Options};
use std::path::PathBuf;
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut options = Options {
        art: PathBuf::from("art/atlas"),
        cards: PathBuf::new(),
        save: PathBuf::new(),
        recover: None,
        socket: None,
        agent: false,
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
            "--help" => {
                println!(
                    "cuba-libre --cards DIRECTORY --save FILE [--art art/atlas] [--recover-from ORIGINAL] [--agent-seat [--socket SOCKET]]\nPrepared Terror task candidate. Recovery reads ORIGINAL.previous into an unused FILE. Development sessions idle out after 30 minutes."
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
    tabletop::run(options)
}
fn main() {
    if let Err(error) = run() {
        eprintln!("cuba-libre: {error}");
        std::process::exit(1);
    }
}
