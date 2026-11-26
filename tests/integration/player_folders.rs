//! `cuba-tools player-folders` (#205): each tool's folder for a faction's session, written
//! through the shared module, with the installed client. What the tools then do with them
//! is measured in real sessions and not here.
use cuba_libre_unfolded::{
    Faction, faction_seats,
    player_folders::{Places, checkout},
};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use unfolded_agent_players::folder::CODEX_WAIT;

fn written(data: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .arg("player-folders")
        .env("HOME", data.join("home"))
        .env("XDG_DATA_HOME", data)
        .output()
        .unwrap()
}

/// Every file under `directory`, as a path within it.
fn files(directory: &Path) -> Vec<String> {
    let mut found = vec![];
    let mut left = vec![directory.to_owned()];
    while let Some(next) = left.pop() {
        for entry in fs::read_dir(next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                left.push(path);
            } else {
                let within = path.strip_prefix(directory).unwrap();
                found.push(within.display().to_string());
            }
        }
    }
    found.sort();
    found
}

fn settings(root: &Path) -> Value {
    let settings = root.join("player/claude-code/.claude/settings.json");
    serde_json::from_slice(&fs::read(settings).unwrap()).unwrap()
}

#[test]
fn each_tool_has_a_folder_of_the_instructions_and_its_permission() {
    let data = tempfile::tempdir().unwrap();
    let output = written(data.path());
    assert!(output.status.success(), "{output:?}");
    let root = data.path().join("cuba-libre-unfolded");
    let launch = String::from_utf8(output.stdout).unwrap();
    for (folder, tool) in [("claude-code", "claude"), ("codex", "codex")] {
        let directory = root.join("player").join(folder);
        let command = format!("cd -- '{}' && {tool}", directory.display());
        assert!(
            launch.lines().any(|line| line.trim() == command),
            "{launch}"
        );
    }
    let client = root.join("client/cuba-player");
    let path = client.to_str().unwrap();
    assert_eq!(
        files(&root),
        [
            "client/cuba-player",
            "player/claude-code/.claude/settings.json",
            "player/claude-code/CLAUDE.md",
            "player/codex/.codex/config.toml",
            "player/codex/.codex/rules/player.rules",
            "player/codex/AGENTS.md",
        ]
    );

    // One document for both tools and every seat, the client named by its path alone.
    let read = |name: &str| fs::read_to_string(root.join("player").join(name)).unwrap();
    let instructions = read("claude-code/CLAUDE.md");
    assert_eq!(instructions, read("codex/AGENTS.md"));
    assert_eq!(
        instructions
            .replace(path, "cuba-player")
            .replace(CODEX_WAIT, "{CODEX_WAIT}"),
        include_str!("../../docs/player-instructions.md")
    );
    assert!(!instructions.replace(path, "").contains("cuba-player"));
    assert!(instructions.contains(CODEX_WAIT));

    // The client runs and is not writable; the permission names it and nothing else.
    let mode = fs::metadata(&client).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o555);
    assert!(
        Command::new(&client)
            .arg("--help")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        settings(&root)["permissions"]["allow"],
        json!([format!("Bash({path}:*)")])
    );
    assert!(read("codex/.codex/rules/player.rules").contains(&format!("[\"{path}\"]")));

    // What the human needs to open a session is printed.
    let said = String::from_utf8(output.stdout).unwrap();
    for place in ["player/claude-code", "player/codex", "client/cuba-player"] {
        assert!(said.contains(root.join(place).to_str().unwrap()), "{said}");
    }
}

/// Claude Code's file tools may read neither the saves nor the checkout, whose ignored
/// `local/` holds saves and captures too (correction 28). The library's denials stay.
#[test]
fn claude_code_is_denied_reading_the_saves_and_the_checkout() {
    let data = tempfile::tempdir().unwrap();
    assert!(written(data.path()).status.success());
    let root = data.path().join("cuba-libre-unfolded");
    let settings = settings(&root);
    let denied: Vec<&str> = settings["permissions"]["deny"]
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule.as_str().unwrap())
        .collect();
    let (reads, writes): (Vec<&str>, _) = denied.iter().partition(|r| r.starts_with("Read("));
    let saves = root.join("saves");
    assert_eq!(
        reads,
        [
            format!("Read(/{}/**)", saves.display()),
            format!("Read(/{}/**)", checkout().display()),
            format!(
                "Read(/{}/home/.local/share/cuba-libre-unfolded/saves/**)",
                data.path().display()
            ),
        ]
    );
    assert!(writes.contains(&"Edit(/.claude/**)"), "{writes:?}");

    // Setup can be retried if it stopped while replacing the generated settings.
    let path = root.join("player/claude-code/.claude/settings.json");
    fs::remove_file(&path).unwrap();
    assert!(written(data.path()).status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(path).unwrap()).unwrap(),
        settings
    );
}

/// A worktree's checkout is the one that holds it, so a session started by a build of a
/// worktree is denied the main checkout's saves as well.
#[test]
fn the_checkout_is_the_one_that_holds_the_repository() {
    let built = Path::new(env!("CARGO_MANIFEST_DIR"));
    let checkout = checkout();
    assert!(built.starts_with(&checkout));
    let repository = built.ancestors().any(|at| at.join(".git").is_dir());
    assert!(checkout.join(".git").is_dir() || (!repository && checkout == built));
}

#[test]
fn a_file_a_session_left_is_refused_and_named_and_notes_are_kept() {
    let data = tempfile::tempdir().unwrap();
    assert!(written(data.path()).status.success());
    let player = data.path().join("cuba-libre-unfolded/player");
    let note = player.join("codex/notes/directorio.md");
    fs::create_dir_all(note.parent().unwrap()).unwrap();
    fs::write(&note, "mine").unwrap();
    let planted = player.join("codex/AGENTS.override.md");
    fs::write(&planted, "other").unwrap();
    let instructions = player.join("claude-code/CLAUDE.md");
    fs::write(&instructions, "changed").unwrap();

    let refused = written(data.path());
    assert_eq!(refused.status.code(), Some(1));
    let error = String::from_utf8(refused.stderr).unwrap();
    assert!(error.contains(planted.to_str().unwrap()), "{error}");
    assert_eq!(fs::read(&instructions).unwrap(), b"changed");

    fs::remove_file(planted).unwrap();
    assert!(written(data.path()).status.success());
    assert_ne!(fs::read(&instructions).unwrap(), b"changed");
    assert_eq!(fs::read(note).unwrap(), b"mine");
}

#[test]
fn a_place_a_permission_cannot_spell_is_refused_before_anything_is_written() {
    let data = tempfile::tempdir().unwrap();
    let refused = written(&data.path().join("my data"));
    assert_eq!(refused.status.code(), Some(1));
    assert!(files(data.path()).is_empty());
    let wrong = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .args(["player-folders", "elsewhere"])
        .env("XDG_DATA_HOME", data.path())
        .output()
        .unwrap();
    assert!(!wrong.status.success());
    assert!(files(data.path()).is_empty());
}

/// The line names the faction, the socket and the instructions, for each tool.
#[test]
fn a_line_names_the_faction_the_socket_and_the_instructions() {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap());
    let within = home.join(".local/share/cuba-libre-unfolded");
    let socket = Path::new("/run/user/1000/cuba-libre-players.sock");
    for (data, folders) in [
        (&*within, "~/.local/share/cuba-libre-unfolded/player"),
        (Path::new("/srv/cuba"), "/srv/cuba/player"),
    ] {
        for faction in Faction::ALL {
            assert_eq!(
                faction_seats::line(faction, socket, Some(&Places::of(data))),
                format!(
                    "Play {} in Cuba Libre at the socket {}, by the instructions in {folders}: claude-code/CLAUDE.md or codex/AGENTS.md.",
                    faction.name(),
                    socket.display()
                )
            );
        }
    }
}
