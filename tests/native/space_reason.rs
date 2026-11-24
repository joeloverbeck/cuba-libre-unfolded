//! #170: a space's hover tag says why the space is in its state for the open
//! decision, in one line of the tag's text style, in the rules authority's words. One
//! space of each state is hovered during Terror and during Assault, at the owner's
//! display, and each tag is captured.
//!
//! Expected lines are worked from the rules on the accepted tactical handoff's position
//! (`docs/research/tactical-slice-handoff.md`), not read from the implementation:
//! - **Terror** (R 3.3.4: 1 Resource and an Underground Guerrilla a space; R 4.3.3:
//!   Kidnap in a space selected for Terror). 26July starts with 0, so Oriente waits
//!   for a transfer; Santiago may open Kidnap (2 Guerrillas outnumber 1 Police) and is
//!   selectable. Only Santiago and Oriente hold Underground 26July Guerrillas. Seed 2's
//!   Kidnap roll is 5, which owes Santiago's Terror; it then pays 1.
//! - **Assault** (R 3.2.4, 6.3.1: 2 Resources a space at Firm). Government has 1
//!   Resource after the roll: Havana (6 Troops, 4 Police; Syndicate's open Casino) is
//!   blocked until a transfer. Matanzas holds no piece. Sierra Maestra, Mountain, holds
//!   no Troops. After a 3-Resource transfer Government has 4 and Havana is selectable;
//!   Santiago (2 Troops, 1 Police; 2 Active 26July Guerrillas) is then paid for.
use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;

fn submit_all(path: &Path, actions: &[&str]) {
    let mut session = Session::create_seeded(path, 2).unwrap();
    for action in actions {
        let view = session.observe();
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        session.submit(owner, view.revision, &id).unwrap();
    }
}

/// Rests the pointer on `space`, reads its tag and its drawn line, and keeps a 1:1
/// crop of the tag. `reason` is the line expected, or `None` for a tag without one.
fn hover(app: &mut Native, space: &str, state: &str, reason: Option<&str>, name: &str) {
    app.point("area:propaganda");
    let view = app.point(&format!("space:{space}"));
    let status = view["table"]["status"].clone();
    assert_eq!(status["space"], space, "{}", view["table"]);
    let target = view["table"]["map_targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["space"] == space)
        .unwrap()
        .clone();
    assert_eq!(target["state"], state, "{space}");
    assert_eq!(status["reason"].as_str(), reason, "{space}");
    // The authority supplies the line: the seat reads the same words in the
    // observation.
    let supplied = view["observation"]["space_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["space"] == space)
        .map(|r| r["reason"].as_str().unwrap().to_owned());
    assert_eq!(supplied.as_deref(), reason, "{space}");
    assert_status_geometry(&status);
    let drawn: Vec<&Value> = view["table"]["status_text"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["reason"] == true)
        .collect();
    let tag = rect_of(&status["bounds"]);
    if let Some(reason) = reason {
        assert_eq!(drawn.len(), 1, "{}", view["table"]["status_text"]);
        assert_eq!(drawn[0]["text"], reason);
        let row = rect_of(&status["reason_bounds"]);
        let text = rect_of(&drawn[0]["bounds"]);
        // One line of the tag's text style: the laid-out text sits inside its row,
        // which is shorter than two lines of 17-pixel Lato.
        assert!(
            text[1] >= row[1] && text[3] <= row[3] && text[0] >= row[0] && text[2] <= row[2],
            "{space}: text {text:?} outside its row {row:?}"
        );
        // The row is the tag's last, and the tag still keeps off every area.
        assert!((row[3] - tag[3]).abs() <= 1., "{space}: {row:?} {tag:?}");
    } else {
        assert!(drawn.is_empty(), "{}", view["table"]["status_text"]);
        assert!(status["reason_bounds"].is_null());
    }
    app.capture(name);
    let capture = image::open(app.root.join(format!("{name}.png"))).unwrap();
    let x = (tag[0] - 40.).max(0.) as u32;
    let y = (tag[1] - 40.).max(0.) as u32;
    let w = ((tag[2] - tag[0]) + 80.).min(3840. - x as f64) as u32;
    let h = ((tag[3] - tag[1]) + 80.).min(2160. - y as f64) as u32;
    capture
        .crop_imm(x, y, w, h)
        .save(app.root.join(format!("{name}-tag.png")))
        .unwrap();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained space tag captures at 3840x2160"]
fn native_space_tags_say_why_each_space_is_in_its_state() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-170"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("space-reason-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let socket = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        save: root.join("terror.json"),
        socket: socket.path().join("seat.sock"),
        root,
        step: 0,
    };
    eprintln!("Space tag reason evidence: {}", app.root.display());

    // Terror, with 26July at 0.
    submit_all(&app.save, &["terror"]);
    app.launch_size("3840x2160");
    let blocked = "A Resource transfer opens it";
    let no_guerrilla = "No Underground 26July Guerrilla";
    hover(
        &mut app,
        "oriente",
        "blocked",
        Some(blocked),
        "terror-oriente-blocked",
    );
    hover(
        &mut app,
        "santiago",
        "selectable",
        None,
        "terror-santiago-selectable",
    );
    hover(
        &mut app,
        "havana",
        "not-involved",
        Some(no_guerrilla),
        "terror-havana-not-involved",
    );
    hover(
        &mut app,
        "ec-oriente-sierra",
        "not-involved",
        Some(no_guerrilla),
        "terror-ec-not-involved",
    );
    app.choose("kidnap");
    app.roll();
    hover(
        &mut app,
        "santiago",
        "selectable",
        Some("Terror is owed here (R 4.3.3)"),
        "terror-santiago-owed",
    );
    hover(
        &mut app,
        "oriente",
        "selectable",
        None,
        "terror-oriente-selectable",
    );
    app.choose("terror:santiago");
    hover(
        &mut app,
        "santiago",
        "not-involved",
        Some("Terror here paid 1 Resource"),
        "terror-santiago-resolved",
    );
    // A removal leaves Oriente's Terror refused with no remedy (#171).
    app.choose("remove");
    app.remove("oriente:underground", 1);
    app.choose("done-removal");
    hover(
        &mut app,
        "oriente",
        "not-involved",
        Some(no_guerrilla),
        "terror-oriente-refused",
    );
    app.quit();

    // Assault, with Government at 1 Resource.
    app.save = app.root.join("assault.json");
    submit_all(
        &app.save,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
        ],
    );
    app.launch_size("3840x2160");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([1, 4, 5, 15])
    );
    hover(
        &mut app,
        "havana",
        "blocked",
        Some(blocked),
        "assault-havana-blocked",
    );
    hover(
        &mut app,
        "matanzas",
        "no-effect",
        Some("No enemy pieces (R 3.2.4)"),
        "assault-matanzas-no-effect",
    );
    hover(
        &mut app,
        "sierra-maestra",
        "no-effect",
        Some("No Troops (R 3.2.4)"),
        "assault-sierra-maestra-no-effect",
    );
    hover(
        &mut app,
        "ec-pinar-la-habana",
        "no-effect",
        Some("No enemy pieces (R 3.2.4)"),
        "assault-ec-no-effect",
    );
    // The transfer is an option row of the action-choice panel: pinned, then committed.
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("option:transfer", revision);
    app.choose("transfer");
    app.choose("ask:1:3");
    app.choose("authorize:3");
    hover(
        &mut app,
        "havana",
        "selectable",
        None,
        "assault-havana-selectable",
    );
    app.choose("assault:santiago:26july");
    hover(
        &mut app,
        "santiago",
        "not-involved",
        Some("Assaulted · paid 2 Resources"),
        "assault-santiago-paid",
    );
    app.quit();
}
