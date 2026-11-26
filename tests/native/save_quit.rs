//! #174: Save · quit and the notice slip in the bottom-right corner, against the main
//! view's layout record at the owner's display. A refusal shows on the slip; a
//! committed decision draws none while the seat still reads its notice; a click
//! dismisses the slip; the plate shows its hovered state and its tag under the pointer
//! and under keyboard focus; and Save · quit ends the app, by Enter and by a click,
//! after which the same save resumes the match with an open Kidnap, a landed die and a
//! pending transfer request. Each state's corner is kept, cropped at 1:1, for
//! inspection.
use super::*;

/// The corner of a capture: the slip's and the plate's places, and the sea around them.
fn corner(app: &mut Native, name: &str) {
    app.capture(name);
    let capture = image::open(app.root.join(format!("{name}.png"))).unwrap();
    capture
        .crop_imm(2700, 1560, 1140, 600)
        .save(app.root.join(format!("{name}-corner.png")))
        .unwrap();
}

/// `[x, y, width, height]` as `[left, top, right, bottom]`.
fn edges(rect: &Value) -> [f64; 4] {
    let [x, y, w, h] = rect_of(rect);
    [x, y, x + w, y + h]
}

/// Each of `which` edges (0 left, 1 top, 2 right, 3 bottom) of `native` lies within
/// `tolerance` of the record's.
fn holds(name: &str, native: [f64; 4], recorded: [f64; 4], which: &[usize], tolerance: f64) {
    for &edge in which {
        assert!(
            (native[edge] - recorded[edge]).abs() <= tolerance,
            "{name}: native {native:?} differs from the record {recorded:?} at edge {edge}"
        );
    }
}

/// The outer edges of the card's 1 px ink stroke as drawn in `capture`, found by
/// scanning inward from 20 px outside each edge of the slip's reported `bounds` (left,
/// top, right, bottom): across the bottom padding for the sides, and down and up a
/// column inside the left padding for the top and the foot. A pixel is the stroke's
/// when its channels sum below 560: the sea sums about 674, the paper 724 and
/// the shadow at least 600. The library's launch-error regression measures the same
/// way (`tabletop::seat`).
fn drawn_stroke(capture: &Path, bounds: [f64; 4]) -> [f64; 4] {
    let image = image::open(capture).unwrap().to_rgb8();
    let dark = |x: f64, y: f64| {
        let p = image.get_pixel(x as u32, y as u32);
        p.0.iter().map(|&c| u32::from(c)).sum::<u32>() < 560
    };
    let [left, top, right, foot] = bounds.map(f64::round);
    let row = foot - 4.;
    let column = left + 8.;
    let first = |mut at: f64, step: f64, end: f64, hit: &dyn Fn(f64) -> bool| loop {
        if hit(at) {
            return at;
        }
        assert!((at - end).abs() > 0.5, "no stroke between {at} and {end}");
        at += step;
    };
    [
        first(left - 20., 1., left + 8., &|x| dark(x, row)),
        first(top - 20., 1., top + 8., &|y| dark(column, y)),
        // The last dark pixel's far side is the edge.
        first(right + 20., -1., right - 8., &|x| dark(x, row)) + 1.,
        first(foot + 20., -1., foot - 8., &|y| dark(column, y)) + 1.,
    ]
}

/// The plate's drawn rectangle.
fn plate(view: &Value) -> [f64; 4] {
    rect_of(
        &view["table"]["main_view"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|part| part["part"] == "save-quit")
            .expect("the main view draws Save · quit")["bounds"],
    )
}

/// Tab until Save · quit has focus, then waits out the shared dwell.
fn focus_save_quit(app: &mut Native) -> Value {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    for _ in 0..40 {
        app.request(json!({"op":"key","revision":revision,"key":"Tab"}), true);
        // A stop whose tag focus opens holds the table until its dwell is out.
        let settled = app.request(json!({"op":"settle"}), true);
        if settled["table"]["choice"]["focus"] == "quit" {
            return app.observe();
        }
    }
    panic!("Tab never reached Save · quit");
}

/// Ends the app through Save · quit, by Enter on its focus or by a click, and waits
/// for it to close its log; then relaunches it on the same save and returns the
/// resumed view. The match, and the save's bytes, are as they were.
pub(super) fn save_quit_and_resume(app: &mut Native, by_enter: bool) -> Value {
    use rustix::{
        event::{PollFd, PollFlags, Timespec, poll},
        fs::inotify::{self, CreateFlags, WatchFlags},
    };
    let before = app.observe();
    let revision = before["observation"]["revision"].as_u64().unwrap();
    let saved = std::fs::read(&app.save).unwrap();
    let request = if by_enter {
        focus_save_quit(app);
        json!({"op":"key","revision":revision,"key":"Enter","focused":"quit"})
    } else {
        json!({"op":"activate","revision":revision,"component":"quit"})
    };
    let mut runtime = app.socket.as_os_str().to_owned();
    runtime.push(".log");
    let close = inotify::init(CreateFlags::CLOEXEC).unwrap();
    inotify::add_watch(&close, Path::new(&runtime), WatchFlags::CLOSE_WRITE).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
        .args(["seat", app.socket.to_str().unwrap(), &request.to_string()])
        .output()
        .unwrap();
    let reply = String::from_utf8_lossy(&output.stdout).into_owned();
    std::fs::write(
        app.root
            .join(format!("{:03}-save-quit-response.jsonl", app.step)),
        &reply,
    )
    .unwrap();
    app.step += 1;
    // The table exited in the request's own steps: the seat answers `exited`.
    assert!(reply.contains("exited"), "{reply}");
    let mut pending = [PollFd::new(&close, PollFlags::IN)];
    let timeout = Timespec {
        tv_sec: 30,
        tv_nsec: 0,
    };
    assert_eq!(
        poll(&mut pending, Some(&timeout)).unwrap(),
        1,
        "the app did not close its log after Save · quit"
    );
    // The session ends with the table and removes its socket.
    for _ in 0..300 {
        if !app.socket.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(!app.socket.exists(), "the seat's socket outlived the app");
    assert_eq!(
        std::fs::read(&app.save).unwrap(),
        saved,
        "Save · quit wrote"
    );
    app.launch_size("3840x2160");
    let resumed = app.observe();
    assert_eq!(resumed["observation"], before["observation"]);
    resumed
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_save_quit_and_the_notice_slip_match_layout_and_resume_the_save() {
    use super::available_forces::start;
    // 3840x2160 is the owner's display and the record's frame: one unit is one pixel.
    let mut app = start("save-quit-", "issue-174", "3840x2160");
    let layout: Value =
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap();
    assert_eq!(layout["frame"], json!([3840, 2160]));
    let tolerance = layout["tolerance"].as_f64().unwrap();
    let recorded_slip = edges(&layout["notice-slip"]["bounds"]);
    let recorded_tag = edges(&layout["save-quit-tag"]["bounds"]);
    let decision = edges(&layout["areas"]["decision"]["bounds"]);

    // At rest: the plate where the record puts it, at rest, and no slip.
    let rest = app.observe();
    let drawn = plate(&rest);
    holds(
        "save-quit",
        drawn,
        edges(&layout["parts"]["save-quit"]),
        &[0, 1, 2, 3],
        tolerance,
    );
    assert!((3840. - drawn[2] - 67.5).abs() <= tolerance, "{drawn:?}");
    assert!((2160. - drawn[3] - 67.5).abs() <= tolerance, "{drawn:?}");
    assert_eq!(rest["table"]["save_quit"]["hovered"], false);
    assert_eq!(rest["table"]["slip"]["shown"], false);
    corner(&mut app, "rest");

    // A refusal: Rally is not in this task. The slip shows the reported notice word
    // for word, as a refusal, above the plate with their right edges aligned.
    app.activate("option:rally", 0);
    app.activate("choice-commit", 0);
    let refused = app.observe();
    assert_eq!(refused["observation"]["revision"], 0);
    let notice = refused["table"]["notice"].as_str().unwrap().to_owned();
    assert!(notice.contains("not in this task"), "{notice}");
    let slip = &refused["table"]["slip"];
    assert_eq!(slip["shown"], true);
    assert_eq!(slip["kind"], "refusal");
    assert_eq!(slip["text"], notice.as_str());
    let bounds = rect_of(&slip["bounds"]);
    // Its width, its right edge and its foot are the record's; it grows upward.
    holds("notice-slip", bounds, recorded_slip, &[0, 2, 3], tolerance);
    assert!(bounds[3] < drawn[1], "the slip covers the plate");
    assert!(
        bounds[1] > decision[3],
        "the slip covers the decision panel"
    );
    corner(&mut app, "refusal");
    // The card is drawn on that place, three lines tall: its stroke's outer edges lie on
    // the reported bounds, and its right edge on Save · quit's.
    let stroke = drawn_stroke(&app.root.join("refusal.png"), bounds);
    eprintln!("refusal: stroke {stroke:?} on bounds {bounds:?}");
    holds(
        "the drawn refusal slip",
        stroke,
        bounds,
        &[0, 1, 2, 3],
        tolerance,
    );
    assert!(
        (stroke[2] - drawn[2]).abs() <= tolerance,
        "the slip's right edge {} is not Save · quit's {}",
        stroke[2],
        drawn[2]
    );

    // A click on the slip dismisses it; the seat still reads the notice.
    app.activate("notice-slip", 0);
    let dismissed = app.observe();
    assert_eq!(dismissed["table"]["slip"]["shown"], false);
    assert_eq!(dismissed["table"]["notice"], notice.as_str());
    corner(&mut app, "dismissed");

    // The next refusal shows again; then 26July commits Terror, whose notice the seat
    // reads while no slip is drawn: its result is on the table.
    app.activate("choice-commit", 0);
    assert_eq!(app.observe()["table"]["slip"]["shown"], true);
    app.choose("terror");
    let committed = app.observe();
    assert_eq!(committed["observation"]["revision"], 1);
    let success = committed["table"]["notice"].as_str().unwrap();
    assert!(!success.is_empty() && success != notice, "{success}");
    assert_eq!(committed["table"]["slip"]["shown"], false);
    corner(&mut app, "after-terror");

    // Under the pointer: the hovered plate and its tag, the top display's tag with one
    // sentence, where the record opens it.
    let hovered = app.point("quit");
    assert_eq!(hovered["table"]["hover"], "save-quit");
    assert_eq!(hovered["table"]["save_quit"]["hovered"], true);
    let tag = &hovered["table"]["top"];
    assert_eq!(tag["item"], "save-quit");
    assert_eq!(tag["title"], "Save · quit");
    assert_eq!(tag["rows"], json!([]));
    assert_eq!(
        tag["notes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|note| note.as_str().unwrap().replace('\n', " "))
            .collect::<Vec<_>>(),
        ["Every decision is saved as it is made, and this button closes the game."]
    );
    holds(
        "save-quit-tag",
        rect_of(&tag["bounds"]),
        recorded_tag,
        &[0, 1, 2, 3],
        tolerance,
    );
    corner(&mut app, "hover");
    let away = app.point("area:propaganda");
    assert_eq!(away["table"]["save_quit"]["hovered"], false);
    assert!(away["table"]["top"].is_null());

    // Keyboard focus: Tab reaches Save · quit, which shows its hovered state and tag.
    let focused = focus_save_quit(&mut app);
    assert_eq!(focused["table"]["save_quit"]["hovered"], true);
    assert_eq!(focused["table"]["top"]["item"], "save-quit");
    corner(&mut app, "focus");

    // Enter ends the app with Kidnap open; the same save resumes it.
    app.choose("kidnap");
    let open = save_quit_and_resume(&mut app, true);
    assert_eq!(open["observation"]["kidnap_open"], true, "Kidnap is open");
    corner(&mut app, "resumed-kidnap");

    // A click ends it once the die has landed; the roll is resumed, not rolled again.
    app.choose("kidnap:roll");
    app.request(json!({"op":"settle"}), true);
    let landed = save_quit_and_resume(&mut app, false);
    assert!(
        landed["observation"]["kidnap"]["die"].is_u64(),
        "{}",
        landed["observation"]["kidnap"]
    );
    corner(&mut app, "resumed-landed-die");

    // And with a transfer request waiting for Syndicate's answer.
    let revision = landed["observation"]["revision"].as_u64().unwrap();
    if landed["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .any(|control| control["name"] == "kidnap-die:continue")
    {
        app.activate("kidnap-die:continue", revision);
    }
    // Syndicate's decision opens over a scrim, which covers the corner too; Wait sets
    // it aside and the request stays pending (#75).
    app.choose("transfer");
    app.choose("ask:3:1");
    app.choose("wait");
    let asked = app.observe();
    assert_eq!(
        asked["observation"]["transfer"]["Authorizing"]["waited"],
        true
    );
    assert!(
        !asked["observation"]["transfer"].is_null(),
        "{}",
        asked["observation"]["transfer"]
    );
    let pending = save_quit_and_resume(&mut app, false);
    assert_eq!(
        pending["observation"]["transfer"],
        asked["observation"]["transfer"]
    );
    corner(&mut app, "resumed-request");
    app.quit();
}
