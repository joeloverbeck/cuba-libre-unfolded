//! Report each ceiling against the limit in force for that session (#197 retrospective).
use serde_json::{Value, json};
use std::{fs, process::Command};

fn report(close_limit: u32, idle: Value) -> (i32, String) {
    let evidence = json!({
        "idle": {"observed": idle, "limit": 0},
        "wait_per_event": {"observed": 1, "limit": 1},
        "approval": {"observed": 0, "limit": 0},
        "close": {"observed": 2, "limit": close_limit}
    });
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), evidence.to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .arg("session-costs")
        .arg(file.path())
        .output()
        .unwrap();
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn the_two_close_calls_in_205_fail_the_original_ceiling() {
    let (code, rows) = report(1, json!(0));
    assert_eq!(code, 1);
    assert!(rows.contains("| Idle | 0 | 0 | passed |"), "{rows}");
    assert!(
        rows.contains("| Waiting per event | 1 | 1 | passed |"),
        "{rows}"
    );
    assert!(rows.contains("| Approval | 0 | 0 | passed |"), "{rows}");
    assert!(rows.contains("| Table close | 2 | 1 | failed |"), "{rows}");
    assert_eq!(report(2, json!(0)).0, 0);
}

#[test]
fn an_unmeasured_event_cannot_pass_or_hide_a_known_failure() {
    let (code, rows) = report(2, Value::Null);
    assert_eq!(code, 3);
    assert!(rows.contains("| Idle | — | 0 | unmeasured |"), "{rows}");
    assert_eq!(report(1, Value::Null).0, 1);
}
