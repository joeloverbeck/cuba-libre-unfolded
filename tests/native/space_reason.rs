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
        Some("Kidnap can start here"),
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
        Some("Terror costs 1 Resource"),
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
        Some("Can close 1 Casino"),
        "assault-havana-selectable",
    );
    hover(
        &mut app,
        "santiago",
        "selectable",
        Some("Can remove 2 Guerrillas"),
        "assault-santiago-selectable",
    );
    hover(
        &mut app,
        "la-habana",
        "no-effect",
        Some("No Troops (R 3.2.4)"),
        "assault-la-habana-no-effect",
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

    // At 3:2 and 21:9 each tag still stands off every area and beside its space.
    for size in ["2160x1440", "3440x1440"] {
        app.launch_size(size);
        for space in ["havana", "la-habana", "santiago", "ec-oriente-sierra"] {
            app.point("area:propaganda");
            let view = app.point(&format!("space:{space}"));
            let status = view["table"]["status"].clone();
            assert_eq!(status["space"], space, "{size}");
            assert!(!status["reason"].is_null(), "{size} {space}");
            assert_status_geometry(&status);
            app.capture(&format!("assault-{space}-{size}"));
        }
        app.quit();
    }

    // #180: twelve spaces paid with every faction's Resources, and Havana left with 1
    // Resource to pay 2. Its Casino would close, so it opens to say why it cannot.
    app.save = app.root.join("unpayable.json");
    let mut actions: Vec<String> = [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
    ]
    .map(str::to_owned)
    .to_vec();
    for (giver, amount) in [(1, 4), (2, 5), (3, 15)] {
        actions.push("transfer".into());
        actions.push(format!("ask:{giver}:{amount}"));
        actions.push(format!("authorize:{amount}"));
    }
    for space in [
        "camaguey-city",
        "santiago",
        "pinar-del-rio",
        "la-habana",
        "matanzas",
        "las-villas",
        "camaguey-province",
        "oriente",
        "sierra-maestra",
        "ec-pinar-la-habana",
        "ec-las-villas-camaguey",
        "ec-oriente-sierra",
    ] {
        actions.push(format!("assault:{space}:none"));
    }
    submit_all(
        &app.save,
        &actions.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    app.launch_size("3840x2160");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([1, 0, 0, 0])
    );
    hover(
        &mut app,
        "havana",
        "unpayable",
        Some("Government cannot pay 2 Resources"),
        "assault-havana-unpayable",
    );
    let saved = fs::read(&app.save).unwrap();
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("space:havana", revision);
    let opened = app.observe();
    assert_eq!(opened["table"]["popover"]["space"], "havana");
    let rows = opened["table"]["popover"]["card"]["rows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 2);
    for row in rows {
        assert_eq!(row["kind"], "unavailable", "{row}");
        assert!(row["button"].is_null(), "{row}");
    }
    app.capture("assault-havana-unpayable-popover");
    // The only space Tab reaches, and Enter closes and opens its popover as a click does.
    let key = |app: &mut Native, key: &str| {
        app.request(json!({"op":"key","revision":revision,"key":key}), true);
        app.request(json!({"op":"settle"}), true);
    };
    key(&mut app, "Escape");
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    for _ in 0..12 {
        if app.observe()["table"]["choice"]["focus"] == "space:havana" {
            break;
        }
        key(&mut app, "Tab");
    }
    app.request(
        json!({"op":"key","revision":revision,"key":"Enter","focused":"space:havana"}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    assert_eq!(app.observe()["table"]["popover"]["space"], "havana");
    assert_eq!(fs::read(&app.save).unwrap(), saved);
    app.quit();
}
