use std::{env, fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
use tempfile::tempdir;

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
  shift 6
  exec "$CUBA_TOOLS" "$@"
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
