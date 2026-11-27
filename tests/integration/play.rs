use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
use tempfile::tempdir;

fn documented_launch() -> &'static str {
    let readme = include_str!("../../README.md");
    let (_, launch) = readme.split_once("## Build and launch\n").unwrap();
    let section = launch.split("\n## ").next().unwrap();
    let (_, example) = section.split_once("```sh\n").unwrap();
    let (command, _) = example.split_once("\n```").unwrap();
    assert_eq!(command, "scripts/play", "README's ordinary desktop command");
    command
}

fn launcher(home: &Path) -> Command {
    let shim = home.join("cargo");
    fs::write(
        &shim,
        r#"#!/usr/bin/env bash
set -eu
printf '<%s>\n' "$@" >> "$HOME/cargo.log"
if [[ $5 == cuba-tools && $7 == prepare-play ]]; then
  if [[ ${PLAY_TEST_FIXTURE:-} == ready ]]; then
    exit 0
  fi
  shift 6
  exec "$CUBA_TOOLS" "$@"
fi
[[ $6 == cuba-libre && "$PLAY_TEST_FIXTURE" == ready ]]
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
        .env_remove("PLAY_TEST_FIXTURE")
        .env("CUBA_TOOLS", env!("CARGO_BIN_EXE_cuba-tools"));
    command
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

#[test]
fn readme_launch_hands_the_prepared_paths_to_the_desktop() {
    let home = tempdir().unwrap();
    let output = launcher(home.path())
        .env("PLAY_TEST_FIXTURE", "ready")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let calls = fs::read_to_string(home.path().join("cargo.log")).unwrap();
    let data = home.path().join(".local/share/cuba-libre-unfolded");
    assert_eq!(
        calls,
        format!(
            "<run>\n<--locked>\n<--quiet>\n<--bin>\n<cuba-tools>\n<-->\n<prepare-play>\n<{}>\n\
             <run>\n<--locked>\n<--features>\n<tabletop>\n<--bin>\n<cuba-libre>\n<-->\n<--cards>\n<{}>\n<--save>\n<{}>\n",
            data.display(),
            data.join("cards").display(),
            data.join("saves/task.json").display()
        )
    );
    assert!(data.join("saves").is_dir());
}
