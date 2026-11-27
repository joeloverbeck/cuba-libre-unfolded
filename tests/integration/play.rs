use cuba_libre_unfolded::{assets, save::Session};
use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
use tempfile::tempdir;
use unfolded_record_replacement::{previous_path, save};

fn documented_launch() -> &'static str {
    let readme = include_str!("../../README.md");
    let (_, launch) = readme.split_once("## Build and launch\n").unwrap();
    let section = launch.split("\n## ").next().unwrap();
    section
        .split("```sh\n")
        .skip(1)
        .filter_map(|example| example.split_once("\n```"))
        .map(|(command, _)| command)
        .find(|command| *command == "scripts/play")
        .expect("README's ordinary desktop command")
}

fn launcher(home: &Path) -> Command {
    let shim = home.join("cargo");
    fs::write(
        &shim,
        r#"#!/usr/bin/env bash
set -eu
printf '<%s>\n' "$@" >> "$HOME/cargo.log"
if [[ $5 == cuba-tools && $7 == prepare-play ]]; then
  if [[ ${PLAY_TEST_CARDS_PREPARED:-} == 1 ]]; then exit 0; fi
  shift 6
  exec "$CUBA_TOOLS" "$@"
fi
if [[ $5 == cuba-tools && $7 == play-save ]]; then
  shift 6
  exec "$CUBA_TOOLS" "$@"
fi
if [[ $6 == cuba-libre && ${PLAY_TEST_CARDS_PREPARED:-} == 1 && ${10} == --save ]]; then
  printf '%s\n' "${11}"
  exit 0
fi
exit 99
"#,
    )
    .unwrap();
    fs::set_permissions(shim, fs::Permissions::from_mode(0o700)).unwrap();
    let path = env::join_paths(
        [home.to_path_buf()]
            .into_iter()
            .chain(env::split_paths(&env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut command = Command::new(root.join(documented_launch()));
    command
        .current_dir(root)
        .env("PATH", path)
        .env("HOME", home)
        .env_remove("XDG_DATA_HOME")
        .env_remove("PLAY_TEST_CARDS_PREPARED")
        .env("CUBA_TOOLS", env!("CARGO_BIN_EXE_cuba-tools"));
    command
}

#[test]
fn readme_launch_gets_past_an_old_default_and_resumes_the_separate_save() {
    let home = tempdir().unwrap();
    let saves = home.path().join(".local/share/cuba-libre-unfolded/saves");
    fs::create_dir_all(&saves).unwrap();
    let disposable = saves.join("task.json");
    let old = format_two_record();
    let previous = previous_path(&disposable);
    fs::write(&disposable, &old).unwrap();
    fs::write(&previous, &old).unwrap();
    let separate = saves.join("task-2.json");
    let launch = || {
        let mut command = launcher(home.path());
        command.env("PLAY_TEST_CARDS_PREPARED", "1");
        command
    };

    let first = launch().output().unwrap();
    assert!(first.status.success(), "{first:?}");
    assert_eq!(
        String::from_utf8(first.stdout).unwrap().trim_end(),
        separate.to_str().unwrap()
    );
    assert!(
        !separate.exists(),
        "the launcher must leave creation to the desktop app"
    );
    Session::create_seeded(&separate, 7).unwrap();
    let resumed = launch().output().unwrap();
    assert!(resumed.status.success(), "{resumed:?}");
    assert_eq!(
        String::from_utf8(resumed.stdout).unwrap().trim_end(),
        separate.to_str().unwrap()
    );
    assert_eq!(
        Session::open(&separate, None).unwrap().observe().revision,
        0
    );

    let explicit = launch().arg("--save").arg(&disposable).output().unwrap();
    assert_eq!(explicit.status.code(), Some(1));
    let stderr = String::from_utf8(explicit.stderr).unwrap();
    assert!(stderr.contains("in format 2"), "{stderr}");
    assert!(stderr.contains("scripts/play --save"), "{stderr}");
    assert!(explicit.stdout.is_empty());
    assert_eq!(fs::read(disposable).unwrap(), old);
    assert_eq!(fs::read(previous).unwrap(), old);
}

fn format_two_record() -> Vec<u8> {
    let identity = [("format", "2"), ("source", assets::SOURCE)]
        .into_iter()
        .collect();
    save::seal(&identity, &serde_json::json!({}))
}

#[test]
fn readme_launch_preserves_occupied_fallback_names_and_tries_the_next() {
    let home = tempdir().unwrap();
    let saves = home.path().join(".local/share/cuba-libre-unfolded/saves");
    fs::create_dir_all(&saves).unwrap();
    fs::write(saves.join("task.json"), format_two_record()).unwrap();
    let orphaned = saves.join("task-2.json.previous");
    let damaged = saves.join("task-3.json");
    fs::write(&orphaned, b"recovery history").unwrap();
    fs::write(&damaged, b"malformed record").unwrap();
    let result = launcher(home.path())
        .env("PLAY_TEST_CARDS_PREPARED", "1")
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        String::from_utf8(result.stdout).unwrap().trim_end(),
        saves.join("task-4.json").to_str().unwrap()
    );
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(
        stderr.contains("task-2.json") && stderr.contains("recovery history exists"),
        "{stderr}"
    );
    assert!(
        stderr.contains("task-3.json") && stderr.contains("Malformed save"),
        "{stderr}"
    );
    assert_eq!(fs::read(orphaned).unwrap(), b"recovery history");
    assert_eq!(fs::read(damaged).unwrap(), b"malformed record");
}

#[test]
fn readme_launch_reports_damage_or_orphaned_history_before_opening_a_window() {
    let home = tempdir().unwrap();
    let saves = home.path().join(".local/share/cuba-libre-unfolded/saves");
    fs::create_dir_all(&saves).unwrap();
    let default = saves.join("task.json");
    let damaged = b"unreadable save";
    fs::write(&default, damaged).unwrap();
    let launch = || {
        let mut command = launcher(home.path());
        command.env("PLAY_TEST_CARDS_PREPARED", "1");
        command
    };

    let refused = launch().output().unwrap();
    assert_eq!(refused.status.code(), Some(1));
    assert!(
        String::from_utf8(refused.stderr)
            .unwrap()
            .contains("Malformed save")
    );
    assert!(refused.stdout.is_empty());
    assert_eq!(fs::read(&default).unwrap(), damaged);

    fs::remove_file(&default).unwrap();
    let previous = previous_path(&default);
    fs::write(&previous, damaged).unwrap();
    let orphaned = launch().output().unwrap();
    assert_eq!(orphaned.status.code(), Some(1));
    assert!(
        String::from_utf8(orphaned.stderr)
            .unwrap()
            .contains("recovery history exists")
    );
    assert!(orphaned.stdout.is_empty());
    assert_eq!(fs::read(previous).unwrap(), damaged);
    assert!(!saves.join("task-2.json").exists());
}

#[test]
fn readme_launch_refuses_clean_home_before_requesting_the_desktop_build() {
    let home = tempdir().unwrap();
    let output = launcher(home.path()).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Missing private cards"), "{stderr}");
    assert!(
        stderr.contains("scripts/play --card-source DIRECTORY"),
        "{stderr}"
    );
    let calls = fs::read_to_string(home.path().join("cargo.log")).unwrap();
    assert!(calls.contains("<prepare-play>"));
    assert!(!calls.contains("<tabletop>"));
    assert!(
        !home
            .path()
            .join(".local/share/cuba-libre-unfolded")
            .exists()
    );
}
