//! `cuba-tools seat --brief`: what a brief reply keeps of a seat reply (#91). The
//! replies below are trimmed from native evidence (`local/verification/fix-86`): real
//! envelopes and choices, with most of the observation and table left out.
use cuba_libre_unfolded::seat_brief::brief;

/// An observing reply after Terror in Santiago: choices, one of them unsupported.
const OBSERVED: &str = r#"{"ok":true,"mode":"seat","status":"observed","revision":4,"settled":true,"pending":[],"observation":{"revision":4,"owner":"M26","choices":[{"accompanies":null,"explanation":"Ends 26July's Terror and records Operation with Special Activity (R 2.3.4).","id":"4:end-terror","label":"End Terror","owner":"M26","preview":null,"remedies":[],"supported":true},{"accompanies":null,"explanation":"R 3.3.4. Terror already resolved here: each space at most once per Operation (R 3.1).","id":"4:terror:santiago","label":"Terror · Santiago de Cuba","owner":"M26","preview":null,"remedies":[],"supported":false}],"resources":[5,10,15,20]},"table":{"controls":[{"name":"option:terror","choice":null,"position":[10.0,20.0]}],"status":"26July"}}"#;

#[test]
fn an_observing_reply_keeps_status_revision_settled_and_each_choice() {
    assert_eq!(
        brief(OBSERVED),
        r#"{"status":"observed","revision":4,"settled":true,"choices":[{"id":"4:end-terror","label":"End Terror","supported":true},{"id":"4:terror:santiago","label":"Terror · Santiago de Cuba","supported":false,"explanation":"R 3.3.4. Terror already resolved here: each space at most once per Operation (R 3.1)."}]}"#
    );
}

#[test]
fn a_refused_reply_keeps_its_error() {
    let refused = r#"{"ok":false,"mode":"seat","status":"refused","revision":3,"settled":true,"pending":[],"error":"Tab is not a seat key here.","observation":{"choices":[]},"table":{"controls":[]}}"#;
    assert_eq!(
        brief(refused),
        r#"{"status":"refused","revision":3,"settled":true,"error":"Tab is not a seat key here.","choices":[]}"#
    );
}

#[test]
fn a_settle_still_in_progress_says_it_is_unsettled() {
    let unsettled = r#"{"ok":false,"mode":"seat","status":"unsettled","revision":3,"settled":false,"pending":["Kidnap die landing"],"error":"Still in progress after 16 steps: Kidnap die landing.","observation":{"choices":[{"id":"3:end-terror","label":"End Terror","supported":true,"explanation":"Ends Terror."}]},"table":{}}"#;
    assert_eq!(
        brief(unsettled),
        r#"{"status":"unsettled","revision":3,"settled":false,"error":"Still in progress after 16 steps: Kidnap die landing.","choices":[{"id":"3:end-terror","label":"End Terror","supported":true}]}"#
    );
}

#[test]
fn a_reply_without_an_observation_keeps_what_its_envelope_has() {
    // The shared protocol's refused start line (`unfolded_seat_protocol::refusal`).
    let refused_start = r#"{"ok":false,"mode":"seat","status":"refused","error":"no cards"}"#;
    assert_eq!(
        brief(refused_start),
        r#"{"status":"refused","error":"no cards"}"#
    );
}

#[test]
fn a_line_that_is_no_reply_prints_as_it_came() {
    for line in ["not json", "[1,2]", r#"{"ok":true}"#] {
        assert_eq!(brief(line), line);
    }
}

/// A seat on a fresh socket that answers each request line with the next of `replies`,
/// then ends the connection. Returns the socket's directory, which owns it.
fn fake_seat(replies: &'static [&'static str]) -> (tempfile::TempDir, std::path::PathBuf) {
    use std::io::{BufRead, BufReader, Write};
    let directory = tempfile::tempdir().unwrap();
    let socket = directory.path().join("seat.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut writer = stream.try_clone().unwrap();
        for (_, reply) in BufReader::new(stream).lines().zip(replies) {
            writer.write_all(format!("{reply}\n").as_bytes()).unwrap();
        }
    });
    (directory, socket)
}

/// `cuba-tools seat`, with `flags` before the socket, sending `requests`.
fn seat(flags: &[&str], socket: &std::path::Path, requests: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .arg("seat")
        .args(flags)
        .arg(socket)
        .args(requests)
        .output()
        .unwrap()
}

const REFUSED: &str = r#"{"ok":false,"mode":"seat","status":"refused","revision":4,"settled":true,"pending":[],"error":"Revision 3 is stale; the table is at revision 4.","observation":{"choices":[]},"table":{"controls":[]}}"#;

#[test]
fn the_brief_flag_prints_each_reply_in_brief() {
    let (_directory, socket) = fake_seat(&[OBSERVED, OBSERVED]);
    let output = seat(
        &["--brief"],
        &socket,
        &[r#"{"op":"observe"}"#, r#"{"op":"settle"}"#],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n{}\n", brief(OBSERVED), brief(OBSERVED))
    );
}

#[test]
fn without_the_flag_each_reply_prints_unchanged() {
    let (_directory, socket) = fake_seat(&[OBSERVED]);
    let output = seat(&[], &socket, &[r#"{"op":"observe"}"#]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, format!("{OBSERVED}\n").as_bytes());
}

#[test]
fn the_brief_flag_keeps_a_failure_s_exit_status_and_messages() {
    let requests = [
        r#"{"op":"activate","revision":3,"choice":"3:pass"}"#,
        r#"{"op":"observe"}"#,
    ];
    let (_full_directory, full_socket) = fake_seat(&[REFUSED]);
    let full = seat(&[], &full_socket, &requests);
    let (_brief_directory, brief_socket) = fake_seat(&[REFUSED]);
    let short = seat(&["--brief"], &brief_socket, &requests);
    assert!(!full.status.success(), "{full:?}");
    assert_eq!(short.status.code(), full.status.code());
    assert_eq!(
        String::from_utf8_lossy(&short.stderr),
        String::from_utf8_lossy(&full.stderr)
    );
    assert!(
        String::from_utf8_lossy(&full.stderr).contains("answered 1/2"),
        "{full:?}"
    );
    assert_eq!(
        String::from_utf8(short.stdout).unwrap(),
        concat!(
            r#"{"status":"refused","revision":4,"settled":true,"error":"Revision 3 is stale; the table is at revision 4.","choices":[]}"#,
            "\n"
        )
    );
}

#[test]
fn a_brief_writer_passes_on_a_line_that_is_not_utf8_byte_for_byte() {
    use std::io::Write;
    let mut written = Vec::new();
    let mut writer = cuba_libre_unfolded::seat_brief::BriefWriter::new(&mut written);
    writer.write_all(b"\xff not text\n").unwrap();
    writer
        .write_all(br#"{"status":"bye","revision":2}"#)
        .unwrap();
    writer.flush().unwrap();
    drop(writer);
    assert_eq!(
        written,
        b"\xff not text\n{\"status\":\"bye\",\"revision\":2}\n"
    );
}

#[test]
fn a_choice_keeps_no_null_field() {
    let odd = r#"{"status":"observed","revision":1,"settled":true,"error":null,"observation":{"choices":[{"id":"1:pass","label":null,"supported":false,"explanation":null}]}}"#;
    assert_eq!(
        brief(odd),
        r#"{"status":"observed","revision":1,"settled":true,"choices":[{"id":"1:pass","supported":false}]}"#
    );
}

#[test]
fn the_brief_flag_reads_requests_from_standard_input_too() {
    use std::io::Write;
    let (_directory, socket) = fake_seat(&[OBSERVED, OBSERVED]);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .args(["seat", "--brief"])
        .arg(&socket)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"op\":\"observe\"}\n\n{\"op\":\"settle\"}\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n{}\n", brief(OBSERVED), brief(OBSERVED))
    );
}
