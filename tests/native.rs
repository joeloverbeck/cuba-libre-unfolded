#![cfg(feature = "tabletop")]
//! Real application/seat/client regression. Requires the accepted local card faces.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
struct Native {
    root: PathBuf,
    cards: PathBuf,
    socket: PathBuf,
    save: PathBuf,
    step: usize,
}
impl Native {
    fn launch(&mut self) {
        self.launch_size("1920x1280");
    }
    fn launch_size(&mut self, size: &str) {
        let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
            .args([
                "--cards",
                self.cards.to_str().unwrap(),
                "--save",
                self.save.to_str().unwrap(),
                "--agent-seat",
                "--seat-size",
                size,
                "--socket",
                self.socket.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        std::fs::write(
            self.root.join(format!("{}-launch.stdout", self.step)),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            self.root.join(format!("{}-launch.stderr", self.step)),
            &output.stderr,
        )
        .unwrap();
        self.step += 1;
        assert!(
            output.status.success(),
            "launch: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn request(&mut self, request: Value, success: bool) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
            .args(["seat", self.socket.to_str().unwrap(), &request.to_string()])
            .output()
            .unwrap();
        std::fs::write(
            self.root.join(format!("{:03}-response.jsonl", self.step)),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            self.root.join(format!("{:03}-request.json", self.step)),
            serde_json::to_vec(&request).unwrap(),
        )
        .unwrap();
        self.step += 1;
        assert_eq!(
            output.status.success(),
            success,
            "request {request}: {} {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn observe(&mut self) -> Value {
        self.request(json!({"op":"observe"}), true)
    }
    fn point(&mut self, component: &str) -> Value {
        let revision = self.observe()["observation"]["revision"].as_u64().unwrap();
        self.request(
            json!({"op":"point","revision":revision,"component":component}),
            true,
        );
        self.request(json!({"op":"settle"}), true)
    }
    fn activate(&mut self, component: &str, revision: u64) {
        self.request(
            json!({"op":"activate","revision":revision,"component":component}),
            true,
        );
        self.request(json!({"op":"settle"}), true);
    }
    /// Submits `action` through its control. At the action choice (#69) an option is
    /// committed by pinning its row and then pressing the commit, as a player does.
    fn choose(&mut self, action: &str) {
        let mut view = self.observe();
        let mut revision = view["observation"]["revision"].as_u64().unwrap();
        if view["observation"]["action"] == "Open" && matches!(action, "pass" | "terror") {
            if view["table"]["choice"]["pinned"] != action {
                self.activate(&format!("option:{action}"), revision);
            }
            view = self.observe();
            assert_eq!(view["table"]["choice"]["pinned"], action, "pinned");
            revision = view["observation"]["revision"].as_u64().unwrap();
        }
        self.request(
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:{action}")}),
            true,
        );
        self.request(json!({"op":"settle"}), true);
    }
    /// Removes `n` of a 26July group through removal mode's real route (#73): a click
    /// on the piece opens its popover, the stepper sets the quantity and Remove
    /// commits the authority's choice for that quantity. `group` is `space:group` as
    /// removal ids name it.
    fn remove(&mut self, group: &str, n: u8) {
        let revision = self.observe()["observation"]["revision"].as_u64().unwrap();
        // As a player would: rest on the piece, then click it.
        self.point(&format!("piece:{}", piece(group)));
        self.activate(&format!("piece:{}", piece(group)), revision);
        for _ in 1..n {
            self.activate("popover:quantity:plus", revision);
        }
        let popover = self.observe()["table"]["popover"].clone();
        assert_eq!(popover["piece"], group, "{popover}");
        assert_eq!(
            popover["card"]["rows"][0]["choice"],
            format!("{revision}:remove:{group}:{n}"),
            "{popover}"
        );
        self.activate("popover:commit", revision);
        // The pointer rests on the map where the card was; let its hover dwell end.
        self.request(json!({"op":"settle"}), true);
        assert_eq!(
            self.observe()["observation"]["revision"].as_u64().unwrap(),
            revision + 1
        );
    }
    fn capture(&mut self, name: &str) {
        self.request(
            json!({"op":"capture","path":self.root.join(format!("{name}.png"))}),
            true,
        );
    }
    fn quit(&mut self) {
        self.request(json!({"op":"quit"}), true);
    }
}
/// The seat's name for a 26July group's piece: `santiago:underground` is the piece
/// `santiago M26 Guerrilla Underground`.
fn piece(group: &str) -> String {
    let (space, group) = group.split_once(':').unwrap();
    let kind = match group {
        "base" => "Base Ordinary",
        "active" => "Guerrilla Active",
        _ => "Guerrilla Underground",
    };
    format!("{space} M26 {kind}")
}
impl Drop for Native {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_cuba-tools"))
            .args(["seat", self.socket.to_str().unwrap(), "{\"op\":\"quit\"}"])
            .output();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained aspect-ratio captures"]
fn native_responsive_layout_uses_width_and_preserves_picks() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-16".into());
    std::fs::create_dir_all(&evidence).unwrap();
    for (width, height) in [(1920, 1280), (1920, 1080), (2520, 1080)] {
        let root = tempfile::Builder::new()
            .prefix(&format!("layout-{width}x{height}-"))
            .tempdir_in(&evidence)
            .unwrap()
            .keep()
            .canonicalize()
            .unwrap();
        let socket_dir = tempfile::tempdir().unwrap();
        let mut app = Native {
            cards: cards.clone(),
            socket: socket_dir.path().join("seat.sock"),
            save: root.join("task.json"),
            root,
            step: 0,
        };
        if let Some(original) = std::env::var_os("CUBA_COMPAT_SAVE") {
            std::fs::copy(original, &app.save).unwrap();
        }
        eprintln!("Responsive native evidence: {}", app.root.display());
        app.launch_size(&format!("{width}x{height}"));
        let before = app.observe();
        let saved = std::fs::read(&app.save).unwrap();
        let revision = before["observation"]["revision"].as_u64().unwrap();
        assert_eq!(
            before["table"]["viewport"],
            json!([width as f64, height as f64])
        );
        let scale = height as f64 / 1280.;
        let viewport = before["table"]["map_viewport"].as_array().unwrap();
        assert!((viewport[0].as_f64().unwrap() - 40. * scale).abs() <= 1.);
        assert!((viewport[2].as_f64().unwrap() - (width as f64 - 580. * scale)).abs() <= 1.);
        let controls = before["table"]["controls"].as_array().unwrap();
        assert!(
            !controls
                .iter()
                .any(|c| matches!(c["name"].as_str(), Some("cancel" | "close")))
        );
        assert!(before["table"].get("inspected").is_none());
        assert!(before["table"].get("details").is_none());
        let quit = controls.iter().find(|c| c["name"] == "quit").unwrap();
        assert!(
            (quit["position"][0].as_f64().unwrap() - (width as f64 - 147. * scale)).abs() <= 1.
        );
        let centers: Vec<_> = controls
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("standing:"))
            .map(|c| c["position"][0].as_f64().unwrap())
            .collect();
        assert_eq!(centers.len(), 4);
        assert!((centers.iter().sum::<f64>() / 4. - width as f64 / 2.).abs() <= 1.);
        app.capture("home");
        let surfaces = before["table"]["surfaces"].as_array().unwrap();
        for space in before["observation"]["spaces"].as_array().unwrap() {
            let rows: Vec<_> = surfaces
                .iter()
                .filter(|s| s["space"] == space["id"])
                .collect();
            assert!(!rows.is_empty(), "loaded GLB must contain {}", space["id"]);
            let province = matches!(
                space["terrain"].as_str(),
                Some("Forest" | "Grassland" | "Mountain")
            );
            assert_eq!(
                rows.iter()
                    .any(|s| s["textured"] == true && s["uv"] == true),
                province,
                "province terrain must use authored texture on the loaded mesh: {}",
                space["id"]
            );
            if province {
                let expected = format!(
                    "Engraved_Atlas_{}",
                    space["terrain"].as_str().unwrap().to_lowercase()
                );
                assert!(
                    rows.iter()
                        .any(|s| s["textured"] == true && s["material"] == expected),
                    "loaded surface must match the space's actual terrain: {}",
                    space["id"]
                );
            }
        }
        for space in before["observation"]["spaces"].as_array().unwrap() {
            let id = space["id"].as_str().unwrap();
            app.activate(&format!("space:{id}"), revision);
            let shown = app.point(&format!("space:{id}"));
            if space["terrain"] != "EconomicCenter" {
                assert_eq!(shown["table"]["status"]["space"], id);
            }
            assert_eq!(shown["observation"], before["observation"]);
        }
        for prefix in ["piece:havana", "count:havana"] {
            let target = controls
                .iter()
                .filter_map(|c| c["name"].as_str())
                .find(|name| name.starts_with(prefix))
                .unwrap();
            assert_eq!(app.point(target)["table"]["status"]["space"], "havana");
        }
        app.capture("region-hover");
        app.point("standing:0");
        app.capture("victory-hover");
        app.activate("reader", revision);
        assert!(app.observe()["table"]["status"].is_null());
        app.capture("reader");
        app.activate("close", revision);
        app.capture("after-reader");
        let before_pixels = image::open(app.root.join("home.png")).unwrap().to_rgb8();
        let after_pixels = image::open(app.root.join("after-reader.png"))
            .unwrap()
            .to_rgb8();
        let panel_corner = ((45. * scale) as u32, (970. * scale) as u32);
        assert_eq!(
            after_pixels.get_pixel(panel_corner.0, panel_corner.1),
            before_pixels.get_pixel(panel_corner.0, panel_corner.1),
            "authored card panel must remain above the desk after UI redraw"
        );
        assert_eq!(app.observe()["observation"], before["observation"]);
        assert_eq!(std::fs::read(&app.save).unwrap(), saved);
        app.quit();
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_victory_emblems_disclose_live_margins_without_inspection() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-16".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("victory-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Victory native evidence: {}", app.root.display());
    app.launch();
    let initial = app.observe();
    let margins = initial["table"]["standings"].as_array().unwrap();
    for (faction, expected) in [
        ("Government", "-2"),
        ("M26", "-8"),
        ("Directorio", "-8"),
        ("Syndicate", "-15"),
    ] {
        assert!(
            margins
                .iter()
                .any(|s| s["faction"] == faction && s["text"] == expected),
            "only signed margins belong under emblems: {margins:?}"
        );
    }
    app.capture("open-emblems");
    app.activate("space:havana", 0);
    let picked = app.observe()["observation"].clone();
    for (index, faction) in ["Government", "M26", "Directorio", "Syndicate"]
        .into_iter()
        .enumerate()
    {
        for target in [format!("standing:{index}"), format!("margin:{index}")] {
            let pointed = app.request(json!({"op":"point","revision":0,"component":target}), true);
            if index > 0 || target.starts_with("margin:") {
                assert_eq!(
                    pointed["table"]["victory"]["faction"], faction,
                    "open tags must switch without another dwell"
                );
            } else {
                assert!(
                    pointed["table"]["victory"].is_null(),
                    "the first disclosure must wait for its dwell"
                );
            }
            let shown = app.request(json!({"op":"settle"}), true);
            assert_eq!(shown["observation"], picked);
            assert_eq!(shown["table"]["victory"]["faction"], faction);
            assert_eq!(shown["table"]["victory_count"], 1);
            assert_eq!(
                shown["table"]["victory"]["title"],
                if faction == "M26" { "26July" } else { faction }
            );
            assert_victory_geometry(&shown);
            assert_eq!(
                shown["table"]["victory"]["formula"],
                [
                    "16 Support − 18 = -2",
                    "6 Opposition + 1 Base − 15 = -8",
                    "1 Controlled pop. + 0 Bases − 9 = -8",
                    "Lower of -4 and -15 = -15"
                ][index]
            );
            assert_eq!(
                shown["table"]["victory"]["rows"].as_array().unwrap().len(),
                [4, 5, 1, 3][index]
            );
            if index == 0 {
                assert_eq!(
                    shown["table"]["victory"]["conditions"],
                    json!([
                        "Support >18 · not met",
                        "All Cities Active Support · 1/3 · not met",
                        "Still needed: Camagüey City",
                        "Still needed: Santiago de Cuba"
                    ])
                );
            }
            assert!(shown["table"]["status"].is_null());
            app.activate(&target, 0);
            assert_eq!(app.observe()["observation"], picked);
        }
        app.capture(&format!("victory-{index}"));
    }
    assert!(app.point("reader")["table"]["victory"].is_null());
    app.point("standing:0");
    app.activate("reader", 0);
    let reader = app.observe();
    assert!(reader["table"]["victory"].is_null());
    assert!(reader["table"]["standings"].as_array().unwrap().is_empty());
    assert!(
        !reader["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"].as_str().unwrap().starts_with("standing:"))
    );
    app.capture("reader-without-score-panels");
    app.activate("close", 0);
    // Real authorized transfers and Terror grow the contribution list and
    // update both factions' margins without reopening a stale disclosure.
    for action in [
        "terror",
        "transfer",
        "transfer:3:1",
        "amount:2",
        "authorize",
        "terror:santiago",
        "terror:oriente",
    ] {
        if action == "amount:2" {
            app.choose("amount:1");
            app.choose("amount:2");
        } else {
            app.choose(action);
        }
    }
    let m26 = app.point("standing:1");
    assert_eq!(
        m26["table"]["victory"]["formula"],
        "9 Opposition + 1 Base − 15 = -5"
    );
    assert!(
        m26["table"]["victory"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r[0] == "Santiago de Cuba" && r[1] == "1 × 1 = 1")
    );
    assert_victory_geometry(&m26);
    app.capture("largest-live-opposition-breakdown");
    let syndicate = app.point("margin:3");
    assert_eq!(
        syndicate["table"]["victory"]["formula"],
        "Lower of -4 and -17 = -17"
    );
    app.choose("remove");
    app.remove("sierra-maestra:base", 1);
    app.choose("done-removal");
    let removed = app.point("standing:1");
    assert_eq!(
        removed["table"]["victory"]["formula"],
        "9 Opposition + 0 Bases − 15 = -6"
    );
    assert!(
        removed["table"]["victory"]["footer"]
            .as_str()
            .unwrap()
            .contains("4 Base")
    );
    assert_victory_geometry(&removed);
    app.capture("removal-updates-victory-and-supply");
    app.quit();
}

fn assert_victory_geometry(view: &Value) {
    let rect = |v: &Value| -> [f64; 4] { std::array::from_fn(|i| v[i].as_f64().unwrap()) };
    let tag = rect(&view["table"]["victory"]["bounds"]);
    assert!(tag[0] >= 0. && tag[1] >= 112. && tag[2] <= 1920. && tag[3] <= 960.);
    let mut texts = view["table"]["victory_text"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| rect(&t["bounds"]))
        .collect::<Vec<_>>();
    assert!(texts.len() >= 7);
    texts.sort_by(|a, b| a[1].total_cmp(&b[1]));
    for text in &texts {
        assert!(
            text[0] >= tag[0] && text[1] >= tag[1] && text[2] <= tag[2] && text[3] <= tag[3],
            "text outside victory tag: {view}"
        );
    }
    for (index, a) in texts.iter().enumerate() {
        for b in &texts[index + 1..] {
            assert!(
                a[2] <= b[0] || a[0] >= b[2] || a[3] <= b[1] || a[1] >= b[3],
                "victory text overlaps: {view}"
            );
        }
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_own_removal_restart_obligation_and_all_pieces_finish() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-15".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("removal-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Removal native evidence: {}", app.root.display());
    app.launch();
    app.choose("terror");
    app.capture("before-removal");
    // Removal mode is saved before it is shown, and a restart resumes it.
    app.choose("remove");
    app.capture("removal-mode");
    let mode = app.observe();
    assert_eq!(mode["observation"]["map_piece_count"], 29);
    assert_eq!(mode["observation"]["removal"], "Selecting");
    std::fs::copy(&app.save, app.root.join("removal-mode-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], mode["observation"]);
    // Done with nothing removed removes nothing.
    app.choose("done-removal");
    assert_eq!(app.observe()["observation"]["map_piece_count"], 29);
    app.choose("remove");
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.remove("santiago:underground", 1);
    let removed = app.observe();
    assert_eq!(removed["observation"]["map_piece_count"], 28);
    assert_eq!(removed["observation"]["removal"], "Selecting");
    assert!(
        removed["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("Available")
    );
    app.capture("pre-kidnap-removal");
    // The same removal at the stale revision is refused and changes nothing.
    app.request(
        json!({"op":"activate","revision":revision,"component":"popover:commit"}),
        false,
    );
    assert_eq!(app.observe()["observation"], removed["observation"]);
    std::fs::copy(&app.save, app.root.join("removed-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], removed["observation"]);
    app.choose("done-removal");
    // A real giver funds the remaining Guerrilla after Kidnap was defeated.
    for action in [
        "transfer",
        "transfer:2:1",
        "amount:1",
        "authorize",
        "terror:santiago",
        "remove",
    ] {
        app.choose(action);
    }
    app.remove("santiago:active", 1);
    app.remove("sierra-maestra:base", 1);
    app.choose("done-removal");
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("standing:1", rev);
    app.capture("base-and-active-removed");
    assert_eq!(app.observe()["observation"]["standings"][1]["score"], 7);
    app.choose("remove");
    app.remove("oriente:underground", 1);
    app.choose("done-removal");
    app.choose("end-terror");
    app.capture("all-pieces-after-terror-finished");
    assert_eq!(app.observe()["observation"]["map_piece_count"], 25);
    app.quit();

    // A fresh zero-space path removes everything and still finishes; Santiago's two
    // Guerrillas go in one decision.
    app.save = app.root.join("zero-space.json");
    app.launch();
    app.choose("terror");
    app.choose("remove");
    app.remove("sierra-maestra:base", 1);
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("standing:1", rev);
    assert_eq!(app.observe()["observation"]["standings"][1]["score"], 6);
    app.capture("base-removed-standing-six");
    app.remove("santiago:underground", 2);
    app.remove("oriente:underground", 1);
    app.choose("done-removal");
    app.choose("end-terror");
    let zero = app.observe();
    assert_eq!(zero["observation"]["map_piece_count"], 25);
    assert_eq!(zero["observation"]["available_piece_count"], 61);
    assert_eq!(zero["observation"]["standings"][1]["score"], 6);
    assert_eq!(zero["observation"]["resources"], json!([6, 0, 5, 15]));
    app.capture("zero-space-all-removed-finished");
    app.quit();

    app.save = app.root.join("owed.json");
    app.launch();
    app.choose("terror");
    for action in ["kidnap", "kidnap:roll", "remove"] {
        app.choose(action);
    }
    app.remove("santiago:underground", 1);
    // The last Santiago Underground Guerrilla is protected: a click on it opens
    // nothing and changes nothing.
    let owed = app.observe();
    let saved = std::fs::read(&app.save).unwrap();
    let rev = owed["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("piece:{}", piece("santiago:underground")), rev);
    assert_eq!(app.observe()["observation"], owed["observation"]);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("owed-removal-protected");
    std::fs::copy(&app.save, app.root.join("owed-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], owed["observation"]);
    app.remove("sierra-maestra:base", 1);
    app.remove("oriente:underground", 1);
    app.choose("done-removal");
    app.choose("terror:santiago");
    app.choose("remove");
    app.remove("santiago:active", 1);
    app.choose("done-removal");
    let v = app.observe();
    assert_eq!(v["observation"]["map_piece_count"], 25);
    assert_eq!(v["observation"]["standings"][1]["score"], 7);
    let amount = v["observation"]["resources"][1].as_u64().unwrap();
    app.choose("transfer");
    app.choose("transfer:1:2");
    for n in 1..=amount {
        app.choose(&format!("amount:{n}"));
    }
    app.choose("authorize");
    app.choose("end-terror");
    assert_eq!(app.observe()["observation"]["resources"][1], 0);
    assert!(
        app.observe()["observation"]["next_options"]
            .as_array()
            .unwrap()
            .contains(&json!("Event"))
    );
    app.capture("owed-fulfilled-all-removed-finished");
    app.quit();

    app.save = app.root.join("split.json");
    app.launch();
    app.choose("terror");
    for action in [
        "transfer",
        "transfer:2:1",
        "amount:1",
        "amount:2",
        "authorize",
        "terror:santiago",
        "terror:oriente",
        "remove",
    ] {
        app.choose(action);
    }
    app.capture("split-active-underground-removal");
    app.remove("santiago:active", 1);
    let split = app.observe();
    assert_eq!(
        split["observation"]["spaces"][2]["pieces"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["state"],
        "Underground"
    );
    app.capture("active-removed-underground-remains");
    app.remove("santiago:underground", 1);
    app.remove("oriente:active", 1);
    app.remove("sierra-maestra:base", 1);
    app.choose("done-removal");
    app.choose("end-terror");
    assert_eq!(app.observe()["observation"]["standings"][1]["score"], 9);
    assert_eq!(app.observe()["observation"]["map_piece_count"], 25);
    app.capture("both-terrors-remain-after-all-removal");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_kidnap_owed_terror_restart_and_next_permissions() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-14".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("terror-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Terror native evidence: {}", app.root.display());
    app.launch();
    app.choose("terror");
    app.capture("before-roll");
    app.choose("kidnap");
    app.choose("kidnap:roll");
    let rolled = app.observe();
    assert_eq!(rolled["observation"]["owed_terror"], "santiago");
    assert!(
        rolled["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("rolled")
    );
    app.capture("rolled-owed");
    let saved_roll = std::fs::read(&app.save).unwrap();
    std::fs::write(app.root.join("rolled-valid.json"), &saved_roll).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved_roll);
    app.request(
        json!({"op":"activate","revision":1,"choice":"1:kidnap"}),
        false,
    );
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    let rev = rolled["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("{rev}:end-terror"), rev);
    assert!(
        app.observe()["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("owed")
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), saved_roll);
    app.activate("reader", rev);
    app.activate("close", rev);
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    app.choose("terror:santiago");
    let after = app.observe();
    assert_eq!(after["observation"]["standings"][1]["score"], 8);
    assert_eq!(after["observation"]["owed_terror"], Value::Null);
    assert!(
        after["table"]["standings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["visible"] == true)
    );
    app.activate("space:santiago", 4);
    app.capture("santiago-after-terror");
    std::fs::copy(&app.save, app.root.join("terror-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], after["observation"]);
    app.request(
        json!({"op":"activate","revision":3,"choice":"3:terror:santiago"}),
        false,
    );
    assert_eq!(app.observe()["observation"], after["observation"]);
    app.activate("standing:1", 4);
    app.capture("santiago-standing");
    // Real Syndicate donation restores Oriente even if the real roll was 1.
    app.choose("transfer");
    app.choose("transfer:3:1");
    app.choose("amount:1");
    app.choose("authorize");
    app.capture("donated-continuation");
    app.choose("terror:oriente");
    let both = app.observe();
    assert_eq!(both["observation"]["standings"][1]["score"], 10);
    app.capture("both-terrors");
    app.choose("end-terror");
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["next_options"],
        json!(["Pass (+1 if executed later)", "Limited Operation", "Event"])
    );
    assert_eq!(
        finished["observation"]["eligible"],
        json!([true, true, true, true])
    );
    app.capture("kidnap-finished");
    app.quit();
}

/// #63: Kidnap opened, cancelled, opened again, restarted while open, then rolled,
/// through the panel's real buttons. Opening and cancelling leave the saved match and
/// its randomness unchanged; the roll commits once and survives restart.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_kidnap_opens_cancels_and_rolls_through_the_panel() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-63".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("kidnap-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Kidnap native evidence: {}", app.root.display());
    let saved = |app: &Native| -> Value {
        serde_json::from_slice(&std::fs::read(&app.save).unwrap()).unwrap()
    };
    // Rests the pointer on a playable choice's button and returns its card.
    let card = |app: &mut Native, action: &str, capture: &str| -> String {
        let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
        let id = format!("{revision}:{action}");
        app.request(json!({"op":"point","revision":revision,"choice":id}), true);
        app.request(json!({"op":"settle"}), true);
        let view = app.observe();
        assert_eq!(view["table"]["preview"]["choice"], id, "{view}");
        app.capture(capture);
        view["table"]["preview"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let panel = |view: &Value, needle: &str| -> bool {
        view["table"]["inline_text"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| {
                t["text"]
                    .as_str()
                    .unwrap()
                    .replace('\n', " ")
                    .contains(needle)
            })
    };
    let ids = |view: &Value| -> Vec<String> {
        view["observation"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                c["id"]
                    .as_str()
                    .unwrap()
                    .split_once(':')
                    .unwrap()
                    .1
                    .to_owned()
            })
            .collect()
    };
    app.launch();
    app.choose("terror");
    let before = app.observe();
    let random = saved(&app)["payload"]["random"].clone();
    let stakes = card(&mut app, "kidnap", "kidnap-open-preview");
    for fact in [
        "Target: Government in Santiago de Cuba (R 4.3.3), which has 6",
        "Possible proceeds: 1–6 Resources · guaranteed minimum 1",
        "accepted commitment policy",
        "card 17's Event",
        "Opening rolls nothing",
    ] {
        assert!(stakes.contains(fact), "{fact}: {stakes}");
    }

    // Open.
    app.choose("kidnap");
    let opened = app.observe();
    assert_eq!(opened["observation"]["kidnap_open"], true);
    assert_eq!(opened["observation"]["kidnap"], Value::Null);
    assert_eq!(opened["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(ids(&opened), ["kidnap:roll", "kidnap:cancel"]);
    assert!(
        opened["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("nothing rolled")
    );
    assert!(panel(&opened, "guaranteed minimum 1"), "{opened}");
    assert_eq!(saved(&app)["payload"]["random"], random);
    app.capture("kidnap-opened");
    let roll = card(&mut app, "kidnap:roll", "kidnap-roll-preview");
    assert!(roll.contains("guaranteed minimum 1"), "{roll}");
    assert!(roll.contains("Becomes unavailable: End Terror"), "{roll}");

    // Cancel: nothing rolled, nothing changed, and going back is still offered.
    app.choose("kidnap:cancel");
    let cancelled = app.observe();
    assert_eq!(cancelled["observation"]["kidnap_open"], false);
    assert_eq!(
        cancelled["observation"]["resources"],
        before["observation"]["resources"]
    );
    assert_eq!(
        cancelled["observation"]["spaces"],
        before["observation"]["spaces"]
    );
    assert!(ids(&cancelled).contains(&"back-to-choice".to_owned()));
    assert_eq!(saved(&app)["payload"]["random"], random);
    assert!(
        cancelled["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("cancelled")
    );
    app.capture("kidnap-cancelled");

    // Open again and restart while open.
    app.choose("kidnap");
    let reopened = app.observe();
    let open_bytes = std::fs::read(&app.save).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], reopened["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), open_bytes);
    app.capture("kidnap-open-after-restart");

    // Roll through its button.
    let revision = reopened["observation"]["revision"].as_u64().unwrap();
    app.choose("kidnap:roll");
    let rolled = app.observe();
    let die = rolled["observation"]["kidnap"]["die"].as_u64().unwrap();
    assert!((1..=6).contains(&die));
    assert_eq!(rolled["observation"]["kidnap"]["proceeds"], die);
    assert_eq!(
        rolled["observation"]["resources"],
        json!([6 - die, die, 5, 15])
    );
    assert_eq!(rolled["observation"]["owed_terror"], "santiago");
    assert_ne!(saved(&app)["payload"]["random"], random);
    assert!(!ids(&rolled).contains(&"back-to-choice".to_owned()));
    assert!(
        rolled["table"]["notice"]
            .as_str()
            .unwrap()
            .contains(&format!("Kidnap rolled {die}"))
    );
    assert!(panel(&rolled, &format!("Kidnap rolled {die}")), "{rolled}");
    app.capture("kidnap-rolled");
    let rolled_bytes = std::fs::read(&app.save).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:kidnap:roll")}),
        false,
    );
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), rolled_bytes);
    app.capture("kidnap-rolled-after-restart");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_donations_enable_both_orders_without_or_with_late_kidnap() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-14".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("donated-terror-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    eprintln!("Donated Terror native evidence: {}", root.display());
    for (name, order, kidnap) in [
        ("no-santiago-first", ["santiago", "oriente"], false),
        ("no-oriente-first", ["oriente", "santiago"], false),
        ("late-santiago-first", ["santiago", "oriente"], true),
        ("late-oriente-first", ["oriente", "santiago"], true),
    ] {
        let directory = root.join(name);
        std::fs::create_dir_all(&directory).unwrap();
        let mut app = Native {
            cards: cards.clone(),
            socket: socket_dir.path().join("seat.sock"),
            save: directory.join("task.json"),
            root: directory,
            step: 0,
        };
        app.launch();
        app.choose("terror");
        app.choose("transfer");
        app.choose("transfer:2:1");
        app.choose("amount:1");
        app.choose("amount:2");
        app.capture("donor-authorization");
        app.choose("authorize");
        assert_eq!(
            app.observe()["observation"]["resources"],
            json!([6, 2, 3, 15])
        );
        app.choose(&format!("terror:{}", order[0]));
        app.capture("first-terror");
        app.choose(&format!("terror:{}", order[1]));
        let both = app.observe();
        assert_eq!(both["observation"]["resources"], json!([6, 0, 3, 15]));
        assert_eq!(both["observation"]["standings"][1]["score"], 10);
        app.capture("both-before-kidnap-or-finish");
        if kidnap {
            app.choose("kidnap");
            app.choose("kidnap:roll");
            let roll = app.observe();
            let die = roll["observation"]["kidnap"]["die"].as_u64().unwrap();
            assert!((1..=6).contains(&die));
            let expected = [
                [5, 1, 3, 15],
                [4, 2, 3, 15],
                [3, 3, 3, 15],
                [2, 4, 3, 15],
                [1, 5, 3, 15],
                [0, 6, 3, 15],
            ][die as usize - 1];
            assert_eq!(roll["observation"]["resources"], json!(expected));
            assert_eq!(roll["observation"]["owed_terror"], Value::Null);
            app.capture("late-kidnap");
        }
        app.choose("end-terror");
        let finished = app.observe();
        assert_eq!(
            finished["observation"]["next_options"]
                .as_array()
                .unwrap()
                .contains(&json!("Event")),
            kidnap
        );
        assert!(
            finished["table"]["standings"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["visible"] == true)
        );
        app.capture("finished");
        app.quit();
    }
}
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_controls_hover_camera_finish_and_process_restarts() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS to the accepted card directory")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-12"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("native-")
        .tempdir_in(&evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Native evidence: {}", app.root.display());
    app.launch();
    let initial = app.observe();
    assert_eq!(initial["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(initial["observation"]["revision"], 0);
    let original = std::fs::read(&app.save).unwrap();
    app.capture("initial");
    let mut water_frames = vec![
        initial["table"]["water_frame"]
            .as_u64()
            .expect("authored water admitted"),
    ];
    for (index, faction) in ["Government", "M26", "Directorio", "Syndicate"]
        .into_iter()
        .enumerate()
    {
        app.request(
            json!({"op":"point","revision":0,"component":format!("standing:{index}")}),
            true,
        );
        let settled = app.request(json!({"op":"settle"}), true);
        assert_eq!(settled["table"]["hover"], faction);
        water_frames.push(settled["table"]["water_frame"].as_u64().unwrap());
        app.capture(&format!("hover-{index}"));
    }
    // Press/release steps advance the same clock as desktop playback. Home is
    // idempotent here; cross a full loop without inventing a time-control API.
    for _ in 0..125 {
        let stepped = app.request(json!({"op":"key","revision":0,"key":"Home"}), true);
        assert_eq!(stepped["table"]["settled"], true);
        water_frames.push(stepped["table"]["water_frame"].as_u64().unwrap());
    }
    assert!(water_frames.iter().all(|frame| *frame < 24));
    assert!(water_frames.windows(2).any(|frames| frames[1] > frames[0]));
    assert!(
        water_frames.windows(2).any(|frames| frames[1] < frames[0]),
        "authored loop must wrap: {water_frames:?}"
    );
    // Check rendered sea pixels too, not just the selected frame index. This
    // rectangle stays left of the island at home and excludes UI details.
    let first = image::open(app.root.join("initial.png")).unwrap().to_rgb8();
    let later = image::open(app.root.join("hover-0.png")).unwrap().to_rgb8();
    let changed = (70..180)
        .flat_map(|x| (420..700).map(move |y| (x, y)))
        .filter(|&(x, y)| first.get_pixel(x, y) != later.get_pixel(x, y))
        .count();
    assert!(
        changed > 20,
        "water texture must visibly advance; changed pixels: {changed}"
    );
    app.activate("standing:0", 0);
    assert_eq!(app.observe()["observation"]["revision"], 0);
    for id in [
        "havana",
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
        app.activate(&format!("space:{id}"), 0);
        assert_eq!(app.observe()["observation"]["revision"], 0);
    }
    app.capture("raised-ec-picking");
    app.point("reader");
    let home = app.observe()["table"]["camera"].clone();
    for _ in 0..24 {
        app.request(json!({"op":"key","key":"ArrowLeft","revision":0}), true);
        app.request(json!({"op":"key","key":"KeyA","revision":0}), true);
    }
    for _ in 0..8 {
        app.request(json!({"op":"key","key":"Equal","revision":0}), true);
    }
    assert_ne!(app.observe()["table"]["camera"], home);
    app.capture("camera-moved");
    let moved = image::open(app.root.join("camera-moved.png"))
        .unwrap()
        .to_rgb8();
    assert_eq!(
        image::imageops::crop_imm(&moved, 1380, 280, 500, 660).to_image(),
        image::imageops::crop_imm(&first, 1380, 280, 500, 660).to_image(),
        "projected count labels must stay beneath opaque decision chrome during pan/orbit/zoom"
    );
    app.request(json!({"op":"key","key":"Home","revision":0}), true);
    assert_eq!(app.observe()["table"]["camera"], home);
    app.activate("reader", 0);
    let reading = app.observe();
    let standings = reading["table"]["standings"].as_array().unwrap();
    assert!(standings.is_empty());
    app.capture("reader");
    let before = app.observe()["table"]["camera"].clone();
    app.request(json!({"op":"key","key":"KeyW","revision":0}), true);
    assert_eq!(app.observe()["table"]["camera"], before);
    app.activate("close", 0);
    assert_eq!(
        std::fs::read(&app.save).unwrap(),
        original,
        "Disclosure, camera and cancellation must not write a match effect"
    );
    app.quit();
    app.launch();
    let resumed = app.observe();
    assert_eq!(resumed["observation"], initial["observation"]);
    app.capture("initial-restarted");
    app.choose("terror");
    // The record before Finish, which `.previous` keeps for recovery.
    let chosen = app.observe();
    app.request(
        json!({"op":"activate","revision":1,"choice":"1:end-terror"}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    let finished = app.observe();
    assert_eq!(finished["observation"]["revision"], 2);
    assert_eq!(finished["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(finished["observation"]["owner"], "Directorio");
    assert_eq!(
        finished["observation"]["next_options"],
        json!(["Pass (+1 if executed later)", "Limited Operation"])
    );
    app.capture("finished");
    let valid = std::fs::read(&app.save).unwrap();
    app.request(
        json!({"op":"activate","revision":1,"choice":"1:end-terror"}),
        false,
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), valid);
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], finished["observation"]);
    app.capture("finished-restarted");
    app.quit();
    // Missing required input must fail before touching an existing valid match.
    let missing = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            Path::new("/definitely-missing-cuba-cards")
                .to_str()
                .unwrap(),
            "--save",
            app.save.to_str().unwrap(),
            "--agent-seat",
        ])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert_eq!(std::fs::read(&app.save).unwrap(), valid);
    std::fs::write(app.root.join("missing-input.stderr"), missing.stderr).unwrap();
    let changed_cards = tempfile::tempdir().unwrap();
    for name in ["card-17.png", "card-18.png"] {
        std::fs::copy(app.cards.join(name), changed_cards.path().join(name)).unwrap();
    }
    let mut card = std::fs::read(changed_cards.path().join("card-17.png")).unwrap();
    card[100] ^= 1;
    std::fs::write(changed_cards.path().join("card-17.png"), card).unwrap();
    let changed_art = tempfile::tempdir().unwrap();
    for line in cuba_libre_unfolded::assets::MANIFEST
        .lines()
        .chain(cuba_libre_unfolded::assets::STATUS_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::VICTORY_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::LAYOUT_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::TERRAIN_MANIFEST.lines())
    {
        let (_, path) = line.split_once("  ").unwrap();
        let relative = path.strip_prefix("art/atlas/").unwrap();
        let destination = changed_art.path().join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(path, destination).unwrap();
    }
    std::fs::write(changed_art.path().join("atlas-map.glb"), b"changed map").unwrap();
    for (name, cards, art) in [
        ("changed-card", changed_cards.path(), Path::new("art/atlas")),
        ("changed-art", app.cards.as_path(), changed_art.path()),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
            .args([
                "--cards",
                cards.to_str().unwrap(),
                "--art",
                art.to_str().unwrap(),
                "--save",
                app.save.to_str().unwrap(),
                "--agent-seat",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert_eq!(std::fs::read(&app.save).unwrap(), valid);
        std::fs::write(app.root.join(format!("{name}.stderr")), output.stderr).unwrap();
    }
    std::fs::copy(
        "art/atlas/atlas-map.glb",
        changed_art.path().join("atlas-map.glb"),
    )
    .unwrap();
    for asset in [
        "atlas-control-uncontrolled.png",
        "atlas-victory-icon-government.png",
        "atlas-victory-ui.png",
        "atlas-layout-card.png",
        "atlas-terrain-map.glb",
        "terrain-labels.json",
        "atlas-terrain-label-havana.png",
    ] {
        let status_path = changed_art.path().join(asset);
        for missing in [false, true] {
            if missing {
                std::fs::remove_file(&status_path).unwrap();
            } else {
                std::fs::write(&status_path, b"changed presentation").unwrap();
            }
            let refused = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
                .args([
                    "--cards",
                    app.cards.to_str().unwrap(),
                    "--art",
                    changed_art.path().to_str().unwrap(),
                    "--save",
                    app.save.to_str().unwrap(),
                    "--agent-seat",
                ])
                .output()
                .unwrap();
            assert!(!refused.status.success());
            assert!(String::from_utf8_lossy(&refused.stderr).contains(asset));
            assert_eq!(std::fs::read(&app.save).unwrap(), valid);
            std::fs::write(
                app.root.join(format!("{asset}-missing-{missing}.stderr")),
                refused.stderr,
            )
            .unwrap();
        }
        std::fs::copy(Path::new("art/atlas").join(asset), &status_path).unwrap();
    }
    let original_path = app.save.clone();
    let previous =
        std::fs::read(unfolded_record_replacement::previous_path(&original_path)).unwrap();
    std::fs::write(&original_path, b"interrupted record").unwrap();
    let refused = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--save",
            original_path.to_str().unwrap(),
            "--agent-seat",
        ])
        .output()
        .unwrap();
    assert!(!refused.status.success());
    std::fs::write(app.root.join("corrupt-main.stderr"), refused.stderr).unwrap();
    assert_eq!(
        std::fs::read(&original_path).unwrap(),
        b"interrupted record"
    );
    assert_eq!(
        std::fs::read(unfolded_record_replacement::previous_path(&original_path)).unwrap(),
        previous
    );
    app.save = app.root.join("recovered.json");
    let recovered = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--save",
            app.save.to_str().unwrap(),
            "--recover-from",
            original_path.to_str().unwrap(),
            "--agent-seat",
            "--socket",
            app.socket.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        recovered.status.success(),
        "recovery: {}",
        String::from_utf8_lossy(&recovered.stderr)
    );
    std::fs::write(app.root.join("recovery-launch.jsonl"), recovered.stdout).unwrap();
    assert_eq!(app.observe()["observation"], chosen["observation"]);
    app.capture("explicit-recovery");
    app.quit();
    assert_eq!(
        std::fs::read(&original_path).unwrap(),
        b"interrupted record"
    );
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained transfer captures"]
fn native_giver_transfers_all_directions_and_interrupted_authorization() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-13"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("transfers-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Transfer evidence: {}", app.root.display());
    app.launch();
    app.choose("terror");
    app.choose("transfer");
    app.capture("directions");
    app.choose("transfer:3:1");
    assert_eq!(app.observe()["observation"]["owner"], "Syndicate");
    app.choose("amount:1");
    app.capture("giver-authorization");
    let pending = app.observe()["observation"].clone();
    let saved = std::fs::read(&app.save).unwrap();
    app.choose("wait");
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    // Each request reconnects; quitting and relaunching additionally crosses processes.
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], pending);
    app.capture("giver-restarted");
    let revision = pending["revision"].as_u64().unwrap();
    app.choose("authorize");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 1, 5, 14])
    );
    let after = app.observe()["observation"].clone();
    app.capture("accepted-balances");
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], after);
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:authorize")}),
        false,
    );
    assert_eq!(app.observe()["observation"], after);
    let revision = after["revision"].as_u64().unwrap();
    app.activate("standing:3", revision);
    app.capture("syndicate-contributions");
    app.point("reader");
    for (pair, expected) in [
        ("1:3", [6, 0, 5, 15]),
        ("0:1", [5, 1, 5, 15]),
        ("1:0", [6, 0, 5, 15]),
        ("2:1", [6, 1, 4, 15]),
        ("1:2", [6, 0, 5, 15]),
        ("3:1", [6, 1, 5, 14]),
    ] {
        app.choose("transfer");
        app.choose(&format!("transfer:{pair}"));
        app.choose("amount:1");
        app.choose("authorize");
        assert_eq!(app.observe()["observation"]["resources"], json!(expected));
    }
    app.choose("transfer");
    app.choose("transfer:0:1");
    app.choose("amount:1");
    app.capture("decline-choice");
    app.choose("decline");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 1, 5, 14])
    );
    app.choose("transfer");
    app.choose("transfer:1:0");
    app.choose("authorize");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 1, 5, 14])
    );
    app.choose("end-terror");
    assert_eq!(app.observe()["observation"]["finished"], true);
    app.capture("transfers-finished");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_hover_status_is_transient_without_region_inspection() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-16".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("hover-status-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Hover status evidence: {}", app.root.display());
    app.launch();
    assert!(app.observe()["table"]["status"].is_null());
    app.capture("initial-no-status");
    app.request(
        json!({"op":"point","revision":0,"component":"space:matanzas"}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    let view = app.observe();
    assert_eq!(view["table"]["status"]["space"], "matanzas");
    assert_eq!(view["table"]["status"]["control"], "Uncontrolled");
    assert_eq!(view["table"]["status"]["opinion"], "Passive Opposition");
    assert_eq!(view["table"]["status"]["contribution"], "×1 · 26July +1");
    app.capture("matanzas-hover");
    app.activate("space:matanzas", 0);
    assert_eq!(app.observe()["observation"]["revision"], 0);
    app.request(
        json!({"op":"point","revision":0,"component":"reader"}),
        true,
    );
    assert!(app.observe()["table"]["status"].is_null());
    // Independent prepared-position expectations from R1.6/1.7 and the
    // accepted tactical handoff, including mixed Control/opinion cases.
    for (id, control, opinion, contribution) in [
        (
            "havana",
            "Government",
            "Active Support",
            "×2 · Government +12",
        ),
        (
            "camaguey-city",
            "Government",
            "Passive Support",
            "×1 · Government +1",
        ),
        ("santiago", "Government", "Neutral", "0 Support/Opposition"),
        (
            "pinar-del-rio",
            "Syndicate",
            "Active Support",
            "×2 · Government +2",
        ),
        (
            "la-habana",
            "Syndicate",
            "Passive Support",
            "×1 · Government +1",
        ),
        (
            "matanzas",
            "Uncontrolled",
            "Passive Opposition",
            "×1 · 26July +1",
        ),
        (
            "las-villas",
            "Government",
            "Neutral",
            "0 Support/Opposition",
        ),
        (
            "camaguey-province",
            "Directorio",
            "Passive Opposition",
            "×1 · 26July +1",
        ),
        ("oriente", "26July", "Passive Opposition", "×1 · 26July +2"),
        (
            "sierra-maestra",
            "26July",
            "Active Opposition",
            "×2 · 26July +2",
        ),
    ] {
        app.request(
            json!({"op":"point","revision":0,"component":format!("space:{id}")}),
            true,
        );
        app.request(json!({"op":"settle"}), true);
        let status = app.observe()["table"]["status"].clone();
        assert_eq!(status["space"], id);
        assert_eq!(status["control"], control);
        assert_eq!(status["opinion"], opinion);
        assert_eq!(status["contribution"], contribution);
        assert_status_geometry(&status);
        let viewport = status["viewport"].as_array().unwrap();
        for occupied in status["occupied_bounds"].as_array().unwrap() {
            for (axis, bound) in [(0, 0), (1, 1)] {
                assert!(
                    occupied[axis].as_f64().unwrap() >= viewport[bound].as_f64().unwrap(),
                    "home framing clips island: {status}"
                );
                assert!(
                    occupied[axis + 2].as_f64().unwrap() <= viewport[bound + 2].as_f64().unwrap(),
                    "home framing clips island: {status}"
                );
            }
        }
        app.capture(&format!("status-{id}"));
    }
    for id in [
        "ec-pinar-la-habana",
        "ec-las-villas-camaguey",
        "ec-oriente-sierra",
    ] {
        app.request(
            json!({"op":"point","revision":0,"component":format!("space:{id}")}),
            true,
        );
        app.request(json!({"op":"settle"}), true);
        assert!(app.observe()["table"]["status"].is_null());
    }
    for prefix in ["piece:havana ", "count:havana "] {
        let view = app.observe();
        let target = view["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|c| c["name"].as_str())
            .find(|name| name.starts_with(prefix))
            .unwrap();
        assert_eq!(app.point(target)["table"]["status"]["space"], "havana");
    }
    app.activate("reader", 0);
    assert!(app.observe()["table"]["status"].is_null());
    app.capture("reader-suppresses-status");
    app.activate("close", 0);
    for key in ["ArrowLeft", "Equal", "Minus"] {
        for _ in 0..5 {
            app.request(json!({"op":"key","revision":0,"key":key}), true);
        }
        app.request(json!({"op":"settle"}), true);
        let shown = app.point("space:havana")["table"]["status"].clone();
        assert_eq!(shown["space"], "havana");
        assert_status_geometry(&shown);
        app.capture(&format!("havana-{key}"));
    }
    app.request(json!({"op":"key","revision":0,"key":"Home"}), true);
    app.request(json!({"op":"settle"}), true);
    // Donated Resources exercise the actual Terror path without randomness.
    for action in [
        "terror",
        "transfer",
        "transfer:3:1",
        "amount:1",
        "authorize",
        "terror:oriente",
    ] {
        app.choose(action);
    }
    let status = app.point("space:oriente")["table"]["status"].clone();
    assert_eq!(status["opinion"], "Active Opposition");
    assert_eq!(status["contribution"], "×2 · 26July +4");
    app.capture("oriente-after-terror");
    app.choose("remove");
    app.remove("oriente:active", 1);
    app.choose("done-removal");
    let status = app.point("space:oriente")["table"]["status"].clone();
    assert_eq!(status["control"], "Uncontrolled");
    assert_eq!(status["opinion"], "Active Opposition");
    app.capture("oriente-empty-control");
    let standing = app.point("standing:1");
    assert!(standing["table"]["status"].is_null());
    assert!(
        standing["table"]["victory"]["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r[0] == "Oriente" && r[1] == "2 × 2 = 4")
    );
    app.capture("standing-keeps-calculation");
    let final_state = app.observe()["observation"].clone();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], final_state);
    app.quit();
    if let Some(legacy) = std::env::var_os("CUBA_COMPAT_SAVE") {
        let legacy = PathBuf::from(legacy);
        let original = std::fs::read(&legacy).unwrap();
        let record: Value = serde_json::from_slice(&original).unwrap();
        // Read-only acquisition: the owner's record is never an app save path.
        app.save = app.root.join("legacy-copy.json");
        std::fs::write(&app.save, &original).unwrap();
        app.launch();
        let resumed = app.observe();
        assert_eq!(
            resumed["observation"]["revision"],
            record["payload"]["revision"]
        );
        assert_eq!(
            resumed["observation"]["resources"],
            record["payload"]["resources"]
        );
        assert_eq!(
            resumed["observation"]["spaces"],
            record["payload"]["spaces"]
        );
        app.point("space:havana");
        app.capture("legacy-save-resumed");
        app.quit();
        app.launch();
        assert_eq!(app.observe()["observation"], resumed["observation"]);
        app.quit();
        assert_eq!(std::fs::read(&app.save).unwrap(), original);
        assert_eq!(std::fs::read(&legacy).unwrap(), original);
    }
}

fn assert_status_geometry(status: &Value) {
    let rect =
        |key: &str| -> [f64; 4] { std::array::from_fn(|i| status[key][i].as_f64().unwrap()) };
    let tag = rect("bounds");
    let region = rect("region_bounds");
    let viewport = rect("viewport");
    assert!(
        tag[0] >= viewport[0]
            && tag[1] >= viewport[1]
            && tag[2] <= viewport[2]
            && tag[3] <= viewport[3]
    );
    assert!(
        tag[2] < region[0] || tag[0] > region[2] || tag[3] < region[1] || tag[1] > region[3],
        "tag overlaps its region: {status}"
    );
    for other in status["occupied_bounds"].as_array().unwrap() {
        let r: [f64; 4] = std::array::from_fn(|i| other[i].as_f64().unwrap());
        assert!(
            tag[2] < r[0] || tag[0] > r[2] || tag[3] < r[1] || tag[1] > r[3],
            "tag covers a region or piece label: {status}"
        );
    }
    let dx = status["leader"][0][0].as_f64().unwrap() - status["leader"][1][0].as_f64().unwrap();
    let dy = status["leader"][0][1].as_f64().unwrap() - status["leader"][1][1].as_f64().unwrap();
    assert!(
        dx.hypot(dy) >= 15.,
        "leader needs visible separation: {status}"
    );
}

/// Pixels that differ inside a token's crop of two same-size captures.
fn token_changes(first: &Path, second: &Path, at: [f64; 2]) -> usize {
    let a = image::open(first).unwrap().to_rgb8();
    let b = image::open(second).unwrap().to_rgb8();
    assert_eq!(a.dimensions(), b.dimensions());
    // The bell rises above its base anchor; keep the crop on the token.
    let (x0, y0) = ((at[0] - 45.).max(0.) as u32, (at[1] - 70.).max(0.) as u32);
    (x0..(x0 + 90).min(a.width()))
        .flat_map(|x| (y0..(y0 + 100).min(a.height())).map(move |y| (x, y)))
        .filter(|&(x, y)| a.get_pixel(x, y) != b.get_pixel(x, y))
        .count()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_terror_bells_loop_seamlessly_out_of_step_and_hover_their_space() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-41".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("terror-bell-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Terror bell evidence: {}", app.root.display());
    app.launch();
    assert_eq!(app.observe()["table"]["terror"], json!([]));
    app.choose("terror");
    app.choose("transfer");
    app.choose("transfer:2:1");
    app.choose("amount:1");
    app.choose("amount:2");
    app.choose("authorize");
    app.choose("terror:santiago");
    app.choose("terror:oriente");
    let start = app.observe();
    assert_eq!(start["table"]["settled"], true);
    let revision = start["observation"]["revision"].as_u64().unwrap();
    let seeks = |view: &Value| {
        let bells = view["table"]["terror"].as_array().unwrap();
        assert_eq!(
            bells.iter().map(|b| &b["space"]).collect::<Vec<_>>(),
            [&json!("oriente"), &json!("santiago")],
            "one playing bell per Terror space"
        );
        bells
            .iter()
            .map(|b| b["seek"].as_f64().unwrap())
            .collect::<Vec<_>>()
    };
    let initial = seeks(&start);
    assert!(initial.iter().all(|s| (0.0..3.6).contains(s)));
    assert!(
        (initial[0] - initial[1]).abs() > 0.3,
        "neighbouring bells must not swing in unison: {initial:?}"
    );
    let position = |view: &Value, name: &str| {
        let control = view["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("{name} is on screen"));
        [
            control["position"][0].as_f64().unwrap(),
            control["position"][1].as_f64().unwrap(),
        ]
    };
    let bells = [
        position(&start, "piece:santiago Terror"),
        position(&start, "piece:oriente Terror"),
    ];
    app.capture("loop-start");
    // Each key press steps the seat's manual clock twice (2 × 16,666,667 ns), so
    // 54 presses are half the 3.6 s authored loop and 108 are one whole loop.
    let press = |app: &mut Native, times: usize| {
        for _ in 0..times {
            let pressed = app.request(json!({"op":"key","revision":revision,"key":"Home"}), true);
            assert_eq!(
                pressed["table"]["settled"], true,
                "the loop never holds the table"
            );
        }
    };
    press(&mut app, 27);
    app.capture("loop-quarter");
    press(&mut app, 27);
    app.capture("loop-half");
    press(&mut app, 54);
    app.capture("loop-whole");
    let whole = seeks(&app.observe());
    for (a, b) in initial.iter().zip(&whole) {
        assert!(
            (a - b).abs() < 1e-4,
            "one loop returns each bell: {initial:?} {whole:?}"
        );
    }
    for at in bells {
        // A swing is symmetric, so a bell caught mid-swing looks the same half a
        // loop later; a quarter loop later it is at an end. Either pair shows the
        // swing, whatever the bell's phase.
        let swing = [
            token_changes(
                &app.root.join("loop-start.png"),
                &app.root.join("loop-half.png"),
                at,
            ),
            token_changes(
                &app.root.join("loop-start.png"),
                &app.root.join("loop-quarter.png"),
                at,
            ),
        ];
        assert!(
            swing.iter().any(|n| *n > 20),
            "the bell visibly swings: {swing:?} changed pixels"
        );
        let seam = token_changes(
            &app.root.join("loop-start.png"),
            &app.root.join("loop-whole.png"),
            at,
        );
        assert_eq!(seam, 0, "one whole loop renders the bell identically");
    }
    for (space, name) in [
        ("santiago", "piece:santiago Terror"),
        ("oriente", "piece:oriente Terror"),
        ("santiago", "count:santiago Terror"),
    ] {
        let hovered = app.point(name);
        assert_eq!(hovered["table"]["status"]["space"], space, "{name}");
    }
    app.capture("hover-bell");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_top_display_matches_layout_and_follows_the_economy() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-46".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("top-display-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards,
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("task.json"),
        root,
        step: 0,
    };
    eprintln!("Top display native evidence: {}", app.root.display());
    // 3840x2160 is 16:9 like the owner's 6144x3456 display that the layout frames.
    app.launch_size("3840x2160");
    let rest = app.observe();
    let layout: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-top-display.layout.json")).unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    let parts = rest["table"]["top_parts"].as_array().unwrap().clone();
    let part = |parts: &[Value], name: &str| {
        parts
            .iter()
            .find(|p| p["part"] == name)
            .unwrap_or_else(|| panic!("native top display has no {name}"))
            .clone()
    };
    // Images on all four edges; text on left, top and right, since its line
    // height is the renderer's rather than the layout's.
    for (group, edges) in [("parts", 4), ("text", 3)] {
        for (name, rect) in layout[group].as_object().unwrap() {
            let r: Vec<f64> = rect
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() * ratio)
                .collect();
            let expected = [r[0], r[1], r[0] + r[2], r[1] + r[3]];
            let native = rect_of(&part(&parts, name)["bounds"]);
            for (n, e) in native.iter().zip(expected).take(edges) {
                assert!(
                    (n - e).abs() <= 2.,
                    "{name}: native {native:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    }
    let ledger = |view: &Value| -> Vec<String> {
        let parts = view["table"]["top_parts"].as_array().unwrap();
        (0..4)
            .map(|row| {
                part(parts, &format!("ledger-row-{row}-value"))["text"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    };
    assert_eq!(ledger(&rest), ["6", "0", "5", "15"]);
    assert_eq!(part(&parts, "aid-value")["text"], "15");
    app.capture("rest");
    let viewport = rect_of(&json!([0., 0., 3840., 2160.]));
    for (component, title) in [
        ("aid", "Aid"),
        ("us-alliance", "US Alliance"),
        ("resources", "Resources"),
        ("cash", "Cash"),
    ] {
        let shown = app.point(component);
        assert_eq!(shown["table"]["hover"], component);
        assert_eq!(shown["table"]["top"]["item"], component);
        assert_eq!(shown["table"]["top"]["title"], title);
        assert!(shown["table"]["victory"].is_null());
        let plate = rect_of(&shown["table"]["top"]["bounds"]);
        assert!(plate[0] >= viewport[0] && plate[2] <= viewport[2] && plate[3] <= viewport[3]);
        for line in shown["table"]["top_text"].as_array().unwrap() {
            let text = rect_of(&line["bounds"]);
            assert!(
                text[0] >= plate[0] && text[2] <= plate[2] && text[3] <= plate[3],
                "{component} tag line {line} leaves its plate {plate:?}"
            );
        }
        if component == "resources" {
            // The Resources tag opens beside the ledger and leaves Cash visible.
            for slot in 0..4 {
                let cash = rect_of(&part(&parts, &format!("cash-slot-{slot}"))["bounds"]);
                assert!(
                    plate[3] <= cash[1] || plate[2] <= cash[0],
                    "covers cash slot {slot}"
                );
            }
        }
        app.capture(&format!("tag-{component}"));
    }
    // Every Resources change reaches the ledger at once.
    app.choose("terror");
    app.choose("transfer");
    app.choose("transfer:2:1");
    app.choose("amount:1");
    app.choose("amount:2");
    app.choose("authorize");
    assert_eq!(ledger(&app.observe()), ["6", "2", "3", "15"]);
    app.capture("after-transfer");
    app.choose("terror:santiago");
    app.choose("terror:oriente");
    assert_eq!(ledger(&app.observe()), ["6", "0", "3", "15"]);
    app.capture("after-terror");
    app.choose("kidnap");
    app.choose("kidnap:roll");
    let roll = app.observe();
    let resources: Vec<String> = roll["observation"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.to_string())
        .collect();
    assert_ne!(resources[0], "6", "a Kidnap roll takes at least 1");
    assert_eq!(ledger(&roll), resources);
    app.capture("after-kidnap");
    let revision = roll["observation"]["revision"].as_u64().unwrap();
    app.activate("reader", revision);
    let reader = app.observe();
    assert!(reader["table"]["top"].is_null());
    assert!(reader["table"]["top_parts"].as_array().unwrap().is_empty());
    app.capture("reader-without-title");
    app.quit();
}

fn rect_of(value: &Value) -> [f64; 4] {
    let v: Vec<f64> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_f64().unwrap())
        .collect();
    [v[0], v[1], v[2], v[3]]
}

/// Checks every drawn text that carries inline emblems (#60) and returns how many
/// emblems it checked. Each emblem is 0.9× its type size, lies inside its text's
/// bounds, sits on its line's baseline (the commonest bottom of the letters beside
/// it), and keeps text glued to it ("[S].") on its line.
fn assert_inline_emblems(view: &Value) -> usize {
    let rect = |v: &Value| -> [f64; 4] { std::array::from_fn(|i| v[i].as_f64().unwrap()) };
    let viewport = view["table"]["viewport"].as_array().unwrap();
    let (width, height) = (viewport[0].as_f64().unwrap(), viewport[1].as_f64().unwrap());
    let mut checked = 0;
    for text in view["table"]["inline_text"].as_array().unwrap() {
        let bounds = rect(&text["bounds"]);
        let size = text["font_size"].as_f64().unwrap();
        assert!(
            bounds[0] >= 0. && bounds[1] >= 0. && bounds[2] <= width && bounds[3] <= height,
            "text leaves the window: {text}"
        );
        for emblem in text["emblems"].as_array().unwrap() {
            let at = rect(&emblem["bounds"]);
            assert!(
                at[0] >= bounds[0] - 0.5
                    && at[1] >= bounds[1] - 0.5
                    && at[2] <= bounds[2] + 0.5
                    && at[3] <= bounds[3] + 0.5,
                "emblem clipped by its text: {text}"
            );
            assert!(
                ((at[3] - at[1]) - 0.9 * size).abs() < 0.5,
                "emblem is not 0.9× its type: {text}"
            );
            let baseline = emblem["baseline"]
                .as_f64()
                .expect("letters beside the emblem");
            assert!(
                (at[3] - baseline).abs() <= 1.5,
                "emblem off its baseline: {text}"
            );
            assert_eq!(
                emblem["glued_text_wrapped"], false,
                "glued text wrapped away: {text}"
            );
            checked += 1;
        }
    }
    checked
}

/// The emblems drawn in the text whose words are exactly `words`.
fn emblems_in(view: &Value, words: &str) -> Vec<String> {
    let texts = view["table"]["inline_text"].as_array().unwrap();
    let found: Vec<_> = texts.iter().filter(|t| t["text"] == words).collect();
    assert_eq!(found.len(), 1, "one drawn text {words:?}: {texts:?}");
    found[0]["emblems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["faction"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_inline_emblems_sit_on_their_baseline_inside_their_text() {
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-60".into());
    std::fs::create_dir_all(&evidence).unwrap();
    for size in ["1920x1280", "4096x2160"] {
        let root = tempfile::Builder::new()
            .prefix(&format!("inline-{size}-"))
            .tempdir_in(&evidence)
            .unwrap()
            .keep()
            .canonicalize()
            .unwrap();
        let socket_dir = tempfile::tempdir().unwrap();
        let mut app = Native {
            cards: cards.clone(),
            socket: socket_dir.path().join("seat.sock"),
            save: root.join("task.json"),
            root,
            step: 0,
        };
        eprintln!("Inline emblem evidence: {}", app.root.display());
        app.launch_size(size);
        let mut checked = 0;

        // The action choice (#69): a briefing names each faction in full at its first
        // mention, then by emblem, and an unsupported option's explanation shows in
        // the session notice.
        app.activate("option:rally", 0);
        let open = app.observe();
        checked += assert_inline_emblems(&open);
        let briefed: Vec<String> = open["table"]["inline_text"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|t| t["emblems"].as_array().unwrap().clone())
            .map(|e| e["faction"].as_str().unwrap().to_owned())
            .collect();
        for faction in ["26July", "Directorio"] {
            assert!(
                briefed.contains(&faction.to_owned()),
                "{faction}: {briefed:?}"
            );
        }
        app.capture("action-choice");
        app.activate("choice-commit", 0);
        let refused = app.observe();
        checked += assert_inline_emblems(&refused);
        let notice = refused["table"]["notice"].as_str().unwrap().to_owned();
        assert!(notice.contains("26July Guerrillas"));
        let session = |view: &Value, words: &str| -> Vec<String> {
            let texts = view["table"]["inline_text"].as_array().unwrap();
            let found: Vec<_> = texts
                .iter()
                .filter(|t| t["text"].as_str().unwrap().ends_with(words))
                .collect();
            assert_eq!(
                found.len(),
                1,
                "one session text ending {words:?}: {texts:?}"
            );
            found[0]["emblems"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["faction"].as_str().unwrap().to_owned())
                .collect()
        };
        assert!(session(&refused, &notice).contains(&"26July".to_owned()));
        app.capture("rally-explanation");
        app.activate("option:rally", 0);
        app.choose("terror");

        // Panel text and a red primary button.
        let initial = app.observe();
        checked += assert_inline_emblems(&initial);
        assert_eq!(
            emblems_in(&initial, "Kidnap · Santiago · Government"),
            ["Government"]
        );
        // The panel heading introduces the deciding faction, so it keeps its name.
        assert!(
            initial["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| !t["text"].as_str().unwrap().starts_with("26July · "))
        );
        app.capture("initial");

        // A tag note names Government by emblem; Resources rows keep faction names.
        let aid = app.point("aid");
        checked += assert_inline_emblems(&aid);
        assert!(
            aid["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["text"]
                    .as_str()
                    .unwrap()
                    .contains("added to Government Resources"))
        );
        app.capture("aid-tag");
        let resources = app.point("resources");
        checked += assert_inline_emblems(&resources);
        let rows = resources["table"]["top_text"].as_array().unwrap();
        for name in ["Government", "26July", "Directorio", "Syndicate"] {
            assert!(rows.iter().any(|t| t["text"] == name), "row {name}");
            assert!(
                resources["table"]["inline_text"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|t| t["text"] != name),
                "row label {name} drawn as an emblem"
            );
        }
        app.capture("resources-tag");

        // A region tag's contribution line.
        let status = app.point("space:matanzas");
        checked += assert_inline_emblems(&status);
        assert_eq!(status["table"]["status"]["opinion"], "Passive Opposition");
        let contribution = status["table"]["status"]["contribution"].as_str().unwrap();
        assert_eq!(emblems_in(&status, contribution), ["26July"]);
        app.capture("status-tag");

        // Direction buttons are red: 26July keeps its name, the other giver is an emblem.
        // Observations keep every name.
        app.choose("transfer");
        let directions = app.observe();
        checked += assert_inline_emblems(&directions);
        for (label, drawn) in [
            ("Government → 26July", "Government"),
            ("26July → Directorio", "Directorio"),
            ("Syndicate → 26July", "Syndicate"),
        ] {
            assert!(
                directions["observation"]["choices"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["label"] == label),
                "observation keeps names: {label}"
            );
            assert_eq!(emblems_in(&directions, label), [drawn]);
        }
        app.capture("transfer-directions");

        // Paper panel text draws 26July's emblem too.
        app.choose("transfer:3:1");
        app.choose("amount:1");
        let authorizing = app.observe();
        checked += assert_inline_emblems(&authorizing);
        assert!(
            authorizing["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |t| t["text"].as_str().unwrap().starts_with("Giver: Syndicate")
                        && t["emblems"].as_array().unwrap().len() == 4
                )
        );
        app.capture("transfer-authorization");

        // The session notice: "[S] gave 1 to [26]. Saved." keeps its full stop.
        app.choose("authorize");
        let given = app.observe();
        checked += assert_inline_emblems(&given);
        assert_eq!(
            given["table"]["notice"],
            "Syndicate gave 1 to 26July. Saved."
        );
        app.capture("after-authorize");

        // Kidnap's record and notice.
        app.choose("kidnap");
        app.choose("kidnap:roll");
        let kidnap = app.observe();
        checked += assert_inline_emblems(&kidnap);
        assert!(
            kidnap["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["text"]
                    .as_str()
                    .unwrap()
                    .contains("(Government → 26July)"))
        );
        app.capture("kidnap");

        // The reader's Eligibility line.
        let revision = kidnap["observation"]["revision"].as_u64().unwrap();
        app.activate("reader", revision);
        let reader = app.observe();
        checked += assert_inline_emblems(&reader);
        assert!(
            reader["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["text"]
                    .as_str()
                    .unwrap()
                    .contains("Eligible: Government, 26July")
                    && t["emblems"].as_array().unwrap().len() >= 4)
        );
        app.capture("reader");
        app.activate("close", revision);
        app.quit();

        // Pass, on a fresh save: its notice and Directorio's options name factions by emblem.
        app.save = app.root.join("pass.json");
        app.launch_size(size);
        app.choose("pass");
        let passed = app.observe();
        checked += assert_inline_emblems(&passed);
        let notice = passed["table"]["notice"].as_str().unwrap().to_owned();
        assert!(notice.starts_with("26July passed"), "{notice}");
        assert_eq!(session(&passed, &notice), ["26July", "Directorio"]);
        assert!(
            passed["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t["text"]
                    .as_str()
                    .unwrap()
                    .starts_with("Directorio is now 1st Eligible"))
        );
        app.capture("after-pass");
        assert!(checked >= 20, "checked {checked} emblems");
        app.quit();
    }
}

/// #61: the action choice through the real controls. Pass ends the task; Terror,
/// a pending request and going back return to the choice before any effect.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained action-choice captures"]
fn native_action_choice_pass_and_back_before_the_first_effect() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-61"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("action-choice-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("pass.json"),
        root,
        step: 0,
    };
    eprintln!("Action choice evidence: {}", app.root.display());
    let ledger = |view: &Value| -> Vec<String> {
        (0..4)
            .map(|row| {
                view["table"]["top_parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["part"] == format!("ledger-row-{row}-value"))
                    .unwrap()["text"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect()
    };
    let named = |view: &Value, name: &str| {
        view["table"]["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .cloned()
    };
    app.launch();
    let open = app.observe();
    assert_eq!(open["observation"]["action"], "Open");
    assert_eq!(open["observation"]["executing"], Value::Null);
    let choices = open["observation"]["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 11);
    // Every listed option is a real row (#69); only the commit submits, and only for a
    // pinned playable option.
    for choice in choices {
        let id = choice["id"].as_str().unwrap();
        let row = format!("option:{}", id.split_once(':').unwrap().1);
        let control = named(&open, &row).unwrap_or_else(|| panic!("{row} not drawn"));
        assert_eq!(control["choice"], false, "{row}");
    }
    assert!(named(&open, "choice-commit").is_some());
    app.capture("action-choice");
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("option:rally", 0);
    assert_eq!(app.observe()["table"]["choice"]["pinned"], "rally");
    app.activate("choice-commit", 0);
    let refused = app.observe();
    assert!(
        refused["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("not in this task")
    );
    assert_eq!(refused["observation"], open["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("rally-not-in-this-task");
    app.activate("option:rally", 0);
    app.choose("pass");
    let passed = app.observe();
    assert_eq!(passed["observation"]["action"], "Pass");
    assert_eq!(passed["observation"]["resources"], json!([6, 1, 5, 15]));
    assert_eq!(passed["observation"]["owner"], "Directorio");
    assert_eq!(passed["observation"]["eligibility"]["first"], "Directorio");
    assert_eq!(passed["observation"]["eligibility"]["second"], "Government");
    assert_eq!(passed["observation"]["eligibility"]["m26_box"], "Pass");
    assert_eq!(
        passed["observation"]["next_options"],
        json!([
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)",
            "Event"
        ])
    );
    assert_eq!(ledger(&passed), ["6", "1", "5", "15"]);
    assert!(named(&passed, "1:transfer").is_none());
    app.capture("after-pass");
    let valid = std::fs::read(&app.save).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], passed["observation"]);
    app.request(
        json!({"op":"activate","revision":0,"choice":"0:pass"}),
        false,
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), valid);
    app.capture("after-pass-restarted");
    app.quit();

    app.save = app.root.join("back.json");
    app.launch();
    app.choose("terror");
    let terror = app.observe();
    assert_eq!(terror["observation"]["executing"], "M26");
    assert!(named(&terror, "1:transfer").is_some());
    assert!(named(&terror, "1:back-to-choice").is_some());
    app.capture("terror-chosen");
    for action in ["transfer", "transfer:3:1", "amount:1"] {
        app.choose(action);
    }
    let pending = app.observe();
    assert_eq!(pending["observation"]["owner"], "Syndicate");
    app.capture("pending-request-with-withdrawal");
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], pending["observation"]);
    app.choose("back-to-choice");
    let back = app.observe();
    assert_eq!(back["observation"]["action"], "Open");
    assert_eq!(back["observation"]["transfer"], Value::Null);
    assert_eq!(back["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(back["observation"]["revision"], 5);
    assert_eq!(ledger(&back), ["6", "0", "5", "15"]);
    assert!(
        back["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("action choice")
    );
    app.capture("back-at-action-choice");
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], back["observation"]);
    app.choose("pass");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 1, 5, 15])
    );
    app.capture("pass-after-going-back");
    app.quit();
}

/// #62: each irreversible step's preview and each disabled Terror target's reason and
/// remedies, as the real panel shows them while the pointer rests on a button. The
/// captures cover the panel before and after Terror and End Terror's preview with no
/// Terror, after Terror, and after Kidnap.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained preview captures"]
fn native_previews_disabled_targets_and_end_terror() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-62"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("previews-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("previews.json"),
        root,
        step: 0,
    };
    eprintln!("Preview evidence: {}", app.root.display());
    // Rests the pointer on a choice's button and returns the card it opens.
    let card = |app: &mut Native, action: &str, capture: &str| -> String {
        let view = app.observe();
        let revision = view["observation"]["revision"].as_u64().unwrap();
        let id = format!("{revision}:{action}");
        // A disabled choice's button is a plain component; a playable one is a choice.
        let playable = view["observation"]["choices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == id && c["supported"] == true);
        let key = if playable { "choice" } else { "component" };
        app.request(json!({"op":"point","revision":revision,(key):id}), true);
        app.request(json!({"op":"settle"}), true);
        let view = app.observe();
        let shown = &view["table"]["preview"];
        assert_eq!(shown["choice"], format!("{revision}:{action}"), "{view}");
        app.capture(capture);
        let lines: Vec<_> = shown["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap().to_owned())
            .collect();
        lines.join(" ")
    };
    app.launch();
    // At the action choice Pass's preview is its pinned briefing (#69).
    app.activate("option:pass", 0);
    let pass = app.observe()["table"]["briefing"].to_string();
    assert!(pass.contains("+1 Resource (0 → 1)"), "{pass}");
    assert!(pass.contains("Directorio becomes 1st Eligible"), "{pass}");
    app.capture("pass-preview");
    app.activate("option:pass", 0);
    app.choose("terror");
    app.capture("panel-before-terror");
    let before = app.observe();
    let names: Vec<_> = before["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_owned())
        .collect();
    // Every listed choice, the disabled targets included, is a real control.
    for name in &names {
        assert!(
            before["table"]["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| &c["name"] == name),
            "{name}"
        );
    }
    let santiago = card(&mut app, "terror:santiago", "santiago-disabled");
    assert!(
        santiago.contains("In this task: Kidnap here first · Request a Resource transfer"),
        "{santiago}"
    );
    let zero = card(&mut app, "end-terror", "end-terror-zero-space");
    for fact in [
        "No Terror resolved",
        "no Pass income",
        "Back to the action choice",
    ] {
        assert!(zero.contains(fact), "{fact}: {zero}");
    }
    let saved = std::fs::read(&app.save).unwrap();
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("{revision}:terror:santiago"), revision);
    assert!(
        app.observe()["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("Kidnap here first")
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    for step in ["transfer", "transfer:2:1", "amount:1", "authorize"] {
        app.choose(step);
    }
    let funded = card(&mut app, "terror:santiago", "santiago-preview");
    assert!(funded.contains("26July Resources: 1 → 0"), "{funded}");
    assert!(
        funded.contains("26July victory standing 7 → 8 (margin -8 → -7)"),
        "{funded}"
    );
    assert!(funded.contains("Neutral → Passive Opposition"), "{funded}");
    app.choose("terror:santiago");
    app.capture("panel-after-terror");
    let after = app.observe();
    assert_eq!(after["observation"]["standings"][1]["score"], 8);
    let only = card(&mut app, "end-terror", "end-terror-after-terror");
    assert!(only.contains("Records: Operation Only"), "{only}");
    assert!(!only.contains("No Terror resolved"), "{only}");
    app.choose("kidnap");
    app.choose("kidnap:roll");
    let special = card(&mut app, "end-terror", "end-terror-after-kidnap");
    assert!(
        special.contains("Records: Operation with Special Activity"),
        "{special}"
    );
    assert!(
        special.contains("All 26July Guerrillas Active. 26July Ineligible through next card."),
        "{special}"
    );
    app.choose("end-terror");
    app.capture("ended");
    assert_eq!(
        app.observe()["observation"]["eligibility"]["m26_box"],
        "OperationWithSpecialActivity"
    );
    app.quit();
}

/// One space's targeting state, as the table reports it.
fn target_state(view: &Value, space: &str) -> String {
    view["table"]["map_targets"]
        .as_array()
        .unwrap_or_else(|| panic!("not targeting: {}", view["table"]["map_targets"]))
        .iter()
        .find(|t| t["space"] == space)
        .unwrap()["state"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The map states agree with the observation: a space is selectable when one of its
/// actions is supported, blocked when none is but one names a remedy, else not
/// involved; and the visible highlights are exactly the selected graphics for those
/// states, a veil applying only when all of its spaces are not involved.
fn assert_map_agrees(view: &Value) {
    let choices = view["observation"]["choices"].as_array().unwrap();
    let revision = view["observation"]["revision"].as_u64().unwrap();
    let mut expected_files = vec![];
    for space in ["santiago", "oriente"] {
        let mut here = vec![format!("{revision}:terror:{space}")];
        if space == "santiago" {
            here.push(format!("{revision}:kidnap"));
        }
        let here: Vec<&Value> = choices
            .iter()
            .filter(|c| here.iter().any(|id| c["id"] == id.as_str()))
            .collect();
        let state = if here.iter().any(|c| c["supported"] == true) {
            "selectable"
        } else if here
            .iter()
            .any(|c| !c["remedies"].as_array().unwrap().is_empty())
        {
            "blocked"
        } else {
            "not-involved"
        };
        assert_eq!(target_state(view, space), state, "{space}: {view}");
        // #73's veils dim Santiago and Oriente too once nothing is left there.
        expected_files.push(if state == "not-involved" {
            format!("atlas-highlight-veil-{space}.png")
        } else {
            format!("atlas-highlight-{state}-{space}.png")
        });
    }
    for target in view["table"]["map_targets"].as_array().unwrap() {
        let space = target["space"].as_str().unwrap();
        if space != "santiago" && space != "oriente" {
            assert_eq!(target["state"], "not-involved", "{space}");
        }
    }
    expected_files.extend(
        [
            "atlas-highlight-veil-cities.png",
            "atlas-highlight-veil-ecs.png",
            "atlas-highlight-veil-provinces.png",
            "atlas-highlight-veil-sierra-maestra.png",
        ]
        .map(String::from),
    );
    expected_files.sort();
    let shown: Vec<String> = view["table"]["highlights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(shown, expected_files);
}

/// Every popover row agrees with its choice: the same supported state, reason and
/// remedies the observation gives.
fn assert_popover_agrees(view: &Value) {
    let popover = &view["table"]["popover"];
    let choices = view["observation"]["choices"].as_array().unwrap();
    for row in popover["card"]["rows"].as_array().unwrap() {
        let Some(id) = row["choice"].as_str() else {
            continue;
        };
        let choice = choices.iter().find(|c| c["id"] == id).unwrap();
        assert_eq!(
            row["mark"] == "selectable",
            choice["supported"] == true,
            "{row}"
        );
        assert_eq!(row["remedies"], choice["remedies"], "{row}");
        if choice["supported"] == false {
            let reason = row["lines"][0][1].as_str().unwrap();
            let explanation = choice["explanation"].as_str().unwrap();
            // A leading rule reference moves to the end, as the layout's reason line has it.
            let expected = match explanation.split_once(". ") {
                Some((rule, rest)) if rule.starts_with("R ") => format!("{rest} {rule}"),
                _ => explanation.to_owned(),
            };
            assert_eq!(reason, expected, "{row}");
        }
    }
}

fn popover_row<'a>(view: &'a Value, head: &str) -> &'a Value {
    view["table"]["popover"]["card"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["head"] == head)
        .unwrap_or_else(|| panic!("no {head} row: {}", view["table"]["popover"]))
}

/// The card is the layout's 340 px wide (at this 1:1 reference scale), beside its
/// anchor rather than over it, inside the window and clear of the Operation panel.
fn assert_popover_placed(view: &Value) {
    let popover = &view["table"]["popover"];
    let [left, top, right, bottom] = rect_of(&popover["bounds"]);
    let anchor = popover["anchor"].as_array().unwrap();
    let ax = anchor[0].as_f64().unwrap();
    assert!((right - left - 340.).abs() <= 2., "{popover}");
    assert!(left >= ax + 38. || right <= ax - 38., "{popover}");
    let map = rect_of(&view["table"]["map_viewport"]);
    assert!(top >= map[1] && bottom <= map[3], "{popover}");
    assert!(right <= 1380., "{popover}");
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained map-targeting captures"]
fn native_map_targets_and_space_popovers_follow_the_rules() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-72"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("targets-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("targets.json"),
        root,
        step: 0,
    };
    eprintln!("Map targeting evidence: {}", app.root.display());
    // A region click outside a targeting state changes nothing at all.
    let inert = |app: &mut Native, space: &str| {
        let before = app.observe();
        let saved = std::fs::read(&app.save).unwrap();
        let revision = before["observation"]["revision"].as_u64().unwrap();
        app.activate(&format!("space:{space}"), revision);
        let after = app.observe();
        assert_eq!(after["observation"], before["observation"]);
        assert_eq!(after["table"]["popover"], Value::Null);
        assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    };
    app.launch();
    let open = app.observe();
    assert_eq!(open["table"]["map_targets"], Value::Null);
    assert_eq!(open["table"]["highlights"], json!([]));
    inert(&mut app, "santiago");
    app.capture("action-choice-click-does-nothing");

    // Terror at 0 Resources: Santiago offers Kidnap, Oriente waits for a transfer.
    app.choose("terror");
    let zero = app.observe();
    assert_map_agrees(&zero);
    assert_eq!(target_state(&zero, "santiago"), "selectable");
    assert_eq!(target_state(&zero, "oriente"), "blocked");
    app.capture("terror-zero-resources");
    // Hovering the blocked space says what is missing and what would unblock it.
    let hover = app.point("space:oriente");
    assert_eq!(hover["table"]["popover"]["space"], "oriente", "{hover}");
    assert_eq!(hover["table"]["popover"]["pinned"], false);
    let terror = popover_row(&hover, "Terror");
    assert_eq!(terror["mark"], "not-yet");
    assert_eq!(terror["remedies"], json!(["Request a Resource transfer"]));
    assert_eq!(
        terror["lines"][0][1],
        "Cost: 1 Resource per space (26July has 0). R 3.3.4"
    );
    assert_popover_agrees(&hover);
    assert_popover_placed(&hover);
    // The region's own status tag keeps working beside it.
    assert_eq!(hover["table"]["status"]["space"], "oriente");
    app.capture("oriente-blocked-hover-zero-resources");
    let away = app.point("standing:0");
    assert_eq!(away["table"]["popover"], Value::Null);

    // Clicking Santiago keeps its popover open: Kidnap chosen, its stakes previewed
    // before the only acting button; Terror not yet, with both remedies.
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("space:santiago", 1);
    let santiago = app.observe();
    assert_eq!(santiago["observation"]["revision"], 1);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    assert_eq!(santiago["table"]["popover"]["pinned"], true);
    let kidnap = popover_row(&santiago, "Kidnap · from Government");
    assert_eq!(kidnap["chosen"], true);
    assert_eq!(kidnap["button"], "Open Kidnap · nothing rolled yet");
    let stakes = kidnap["lines"].to_string();
    assert!(
        stakes.contains("The roll takes 1–6 Resources from Government (has 6); at least 1."),
        "{stakes}"
    );
    assert_eq!(
        popover_row(&santiago, "Terror")["remedies"],
        json!(["Kidnap here first", "Request a Resource transfer"])
    );
    assert_popover_agrees(&santiago);
    assert_popover_placed(&santiago);
    app.capture("santiago-popover-zero-resources");
    // Opening Kidnap from the popover: it rolls nothing and pauses map targeting.
    app.activate("popover:commit", 1);
    let opened = app.observe();
    assert_eq!(opened["observation"]["revision"], 2);
    assert_eq!(opened["observation"]["kidnap_open"], true);
    assert_eq!(opened["observation"]["kidnap"], Value::Null);
    assert_eq!(opened["table"]["popover"], Value::Null);
    assert_eq!(opened["table"]["map_targets"], Value::Null);
    app.capture("kidnap-opened-from-popover");
    inert(&mut app, "santiago");
    app.choose("kidnap:cancel");

    // After a donation both targets are selectable; Terror from the map popover.
    for step in ["transfer", "transfer:3:1", "amount:1", "authorize"] {
        app.choose(step);
    }
    let funded = app.observe();
    assert_map_agrees(&funded);
    assert_eq!(target_state(&funded, "oriente"), "selectable");
    app.capture("donated-both-selectable");
    let revision = funded["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", revision);
    let terror = app.observe();
    let row = popover_row(&terror, "Terror");
    assert_eq!(row["chosen"], true);
    let preview = row["lines"].to_string();
    for fact in [
        "Pay 1 Resource: 26July 1 → 0.",
        "Flip 1 Underground Guerrilla there to Active.",
        "Place a Terror marker; Neutral → Passive Opposition: 26July 7 → 8 (margin -8 → -7).",
    ] {
        assert!(preview.contains(fact), "{fact}: {preview}");
    }
    app.capture("santiago-terror-preview");
    // Choosing the other row only changes the preview.
    app.activate("popover:row:kidnap", revision);
    let switched = app.observe();
    assert_eq!(switched["observation"]["revision"], revision);
    assert_eq!(
        popover_row(&switched, "Kidnap · from Government")["chosen"],
        true
    );
    app.activate("popover:row:terror:santiago", revision);
    app.activate("popover:commit", revision);
    let after = app.observe();
    assert_eq!(after["observation"]["standings"][1]["score"], 8);
    assert_eq!(after["observation"]["resources"][1], 0);
    assert_eq!(after["table"]["popover"], Value::Null);
    // Oriente is blocked again; Santiago still offers Kidnap.
    assert_map_agrees(&after);
    assert_eq!(target_state(&after, "oriente"), "blocked");
    assert_eq!(target_state(&after, "santiago"), "selectable");
    app.capture("after-santiago-terror");
    let hover = app.point("space:oriente");
    assert_eq!(hover["table"]["popover"]["space"], "oriente");
    assert_popover_agrees(&hover);
    app.capture("oriente-blocked-hover-after-donation");
    // A resolved Terror stays in Santiago's popover as history.
    let revision = after["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", revision);
    let history = app.observe();
    assert_eq!(popover_row(&history, "Terror")["mark"], "done");
    app.capture("santiago-terror-done");
    app.activate("space:santiago", revision);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);

    // Terror from the panel after another donation.
    for step in ["transfer", "transfer:3:1", "amount:1", "authorize"] {
        app.choose(step);
    }
    app.choose("terror:oriente");
    let panel = app.observe();
    assert_eq!(panel["observation"]["standings"][1]["score"], 10);
    assert_map_agrees(&panel);
    assert_eq!(target_state(&panel, "oriente"), "not-involved");
    app.capture("after-oriente-terror-from-panel");

    // Removal mode pauses space targeting; a removal can leave a space with no remedy.
    app.choose("remove");
    inert(&mut app, "santiago");
    app.remove("santiago:active", 1);
    app.choose("done-removal");
    let removed = app.observe();
    // 1 Guerrilla does not exceed 1 Police: Kidnap is gone, Terror there resolved.
    assert_map_agrees(&removed);
    assert_eq!(target_state(&removed, "santiago"), "not-involved");
    inert(&mut app, "santiago");
    app.capture("after-removal");

    app.choose("end-terror");
    assert_eq!(app.observe()["table"]["map_targets"], Value::Null);
    inert(&mut app, "oriente");
    app.capture("ended");
    app.quit();

    // A changed highlight refuses at launch, before any save is touched.
    let changed_art = tempfile::tempdir().unwrap();
    for line in cuba_libre_unfolded::assets::MANIFEST
        .lines()
        .chain(cuba_libre_unfolded::assets::STATUS_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::VICTORY_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::LAYOUT_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::TERRAIN_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::TERROR_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::TOP_MANIFEST.lines())
        .chain(cuba_libre_unfolded::assets::HIGHLIGHT_MANIFEST.lines())
    {
        let (_, path) = line.split_once("  ").unwrap();
        let destination = changed_art
            .path()
            .join(path.strip_prefix("art/atlas/").unwrap());
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(path, destination).unwrap();
    }
    let highlight = changed_art
        .path()
        .join("atlas-highlight-blocked-oriente.png");
    let mut bytes = std::fs::read(&highlight).unwrap();
    bytes[100] ^= 1;
    std::fs::write(&highlight, bytes).unwrap();
    let saved = std::fs::read(&app.save).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
        .args([
            "--cards",
            app.cards.to_str().unwrap(),
            "--art",
            changed_art.path().to_str().unwrap(),
            "--save",
            app.save.to_str().unwrap(),
            "--agent-seat",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("atlas-highlight-blocked-oriente.png"),
        "{stderr}"
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    std::fs::write(app.root.join("changed-highlight.stderr"), output.stderr).unwrap();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; compare with art/proposals/issue-66/hf-a-survey-ink-16x9.jpg"]
fn native_map_targets_at_the_owner_s_aspect() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-72"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("targets-16x9-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("targets.json"),
        root,
        step: 0,
    };
    eprintln!("Map targeting 16:9 evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app.choose("terror");
    let view = app.observe();
    assert_map_agrees(&view);
    app.capture("terror-zero-3840x2160");
    app.activate("space:santiago", 1);
    let view = app.observe();
    assert_eq!(view["table"]["popover"]["pinned"], true);
    // The card keeps the layout's proportions at this scale: 340 reference pixels.
    let [left, _, right, _] = rect_of(&view["table"]["popover"]["bounds"]);
    let scale = (3840f64 / 1920.).min(2160. / 1280.);
    assert!(
        (right - left - 340. * scale).abs() <= 2.,
        "{}",
        view["table"]["popover"]
    );
    app.capture("santiago-popover-3840x2160");
    app.quit();
}

/// #69: the selected AC-B action choice. Native placement matches the FigCraft layout
/// record within 2 px at 3840×2160 (16:9, like the owner's 6144×3456 frame, ×0.625).
/// Every option, its briefing, the pin, the commit, the BRIEFING ribbon and its tabs
/// are reached by point, activate and key. Briefing values agree with the observation
/// before and after a transfer, a Terror and a removal, and no briefing recommends a
/// move.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained action-choice captures"]
fn native_action_choice_matches_layout_and_briefs_every_option() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-69"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("ac-b-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("ac-b.json"),
        root,
        step: 0,
    };
    eprintln!("AC-B evidence: {}", app.root.display());
    let layout: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-action-choice.layout.json")).unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    let part = |view: &Value, name: &str| -> [f64; 4] {
        let found = view["table"]["choice_parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["part"] == name)
            .unwrap_or_else(|| panic!("no native part {name}"))
            .clone();
        rect_of(&found["bounds"])
    };
    // Each group's edges: left, top, right, bottom, or the horizontal centre (4).
    let compare = |view: &Value, group: &Value, edges: &[usize]| {
        for (name, r) in group.as_object().unwrap() {
            let r: Vec<f64> = r
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() * ratio)
                .collect();
            let expected = [r[0], r[1], r[0] + r[2], r[1] + r[3], r[0] + r[2] / 2.];
            let n = part(view, name);
            let native = [n[0], n[1], n[2], n[3], (n[0] + n[2]) / 2.];
            for &edge in edges {
                assert!(
                    (native[edge] - expected[edge]).abs() <= 2.,
                    "{name} edge {edge}: native {n:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    };
    let briefing = |view: &Value| -> String {
        let shown = &view["table"]["briefing"];
        assert!(shown.is_object(), "no briefing shown: {}", view["table"]);
        shown["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                format!(
                    "{}: {}",
                    e["label"].as_str().unwrap(),
                    e["text"].as_str().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let no_advice = |words: &str| {
        let lower = words.to_lowercase();
        for advice in [
            "should",
            "recommend",
            "best",
            "better",
            "consider",
            "suggest",
            "you ",
        ] {
            assert!(!lower.contains(advice), "{advice}: {words}");
        }
    };
    let key = |app: &mut Native, key: &str, focused: Option<&str>| -> Value {
        let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
        let mut request = json!({"op":"key","revision":revision,"key":key});
        if let Some(focused) = focused {
            request["focused"] = json!(focused);
        }
        app.request(request, true);
        app.request(json!({"op":"settle"}), true);
        app.observe()
    };
    app.launch_size("3840x2160");
    let open = app.observe();
    assert_eq!(open["observation"]["action"], "Open");
    assert_eq!(open["table"]["briefing"], Value::Null);
    let commit = |view: &Value| -> Value {
        view["table"]["choice_parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["part"] == "choice-commit-label")
            .unwrap()["text"]
            .clone()
    };
    assert_eq!(commit(&open), "Click an option to choose it");

    // Hover: every listed option opens its briefing, unpinned, without changing the
    // match; none recommends a move.
    let options: Vec<String> = open["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            c["id"]
                .as_str()
                .unwrap()
                .split_once(':')
                .unwrap()
                .1
                .to_owned()
        })
        .collect();
    assert_eq!(options.len(), 11);
    for option in &options {
        let view = app.point(&format!("option:{option}"));
        let view = if view["table"]["briefing"].is_object() {
            view
        } else {
            app.observe()
        };
        assert_eq!(
            view["table"]["briefing"]["option"],
            option.as_str(),
            "{option}"
        );
        assert_eq!(view["table"]["briefing"]["pinned"], false);
        let words = briefing(&view);
        assert!(words.contains("26July has 0"), "{option}: {words}");
        no_advice(&words);
        assert_eq!(view["observation"], open["observation"]);
        if option == "pass" || option == "rally" {
            app.capture(&format!("hover-{}", option.replace(':', "-")));
        }
    }
    // Pass keeps its exact numbers (R 2.3.3).
    app.activate("option:pass", 0);
    let pass = app.observe();
    assert_eq!(pass["table"]["choice"]["pinned"], "pass");
    assert!(briefing(&pass).contains("RESULT: +1 Resource (0 → 1). 26July stays Eligible."));
    assert_eq!(commit(&pass), "Pass · +1 Resource");
    app.capture("pass-pinned");

    // Terror pinned is the layout's 16:9 frame.
    app.activate("option:terror", 0);
    let pinned = app.observe();
    assert_eq!(pinned["table"]["briefing"]["option"], "terror");
    assert_eq!(pinned["table"]["briefing"]["pinned"], true);
    assert_eq!(commit(&pinned), "Choose Terror");
    compare(&pinned, &layout["parts"], &[0, 1, 2, 3]);
    compare(&pinned, &layout["stretched"], &[0, 1, 2]);
    compare(&pinned, &layout["text"], &[0, 1]);
    compare(&pinned, &layout["right_text"], &[1, 2]);
    compare(&pinned, &layout["centered_text"], &[1, 4]);
    let terror = briefing(&pinned);
    for fact in [
        "COST: 1 Resource per City or Province · 26July has 0.",
        "NEXT: Directorio, 2nd Eligible: Pass or a Limited Operation; after Kidnap, also card 17's Event",
    ] {
        assert!(terror.contains(fact), "{fact}: {terror}");
    }
    app.capture("terror-pinned");

    // Keyboard: Tab focuses Pass and shows its briefing; Enter pins it; Escape unpins.
    let focused = key(&mut app, "Tab", None);
    assert_eq!(focused["table"]["choice"]["focus"], "option:pass");
    assert_eq!(focused["table"]["briefing"]["option"], "pass");
    let pinned_pass = key(&mut app, "Enter", Some("option:pass"));
    assert_eq!(pinned_pass["table"]["choice"]["pinned"], "pass");
    app.capture("keyboard-pass-pinned");
    let unpinned = key(&mut app, "Escape", None);
    assert_eq!(unpinned["table"]["choice"]["pinned"], Value::Null);
    // Tab on to Terror, pin it, then to the commit, which Enter presses.
    for _ in 0..9 {
        key(&mut app, "Tab", None);
    }
    let on_terror = key(&mut app, "Enter", Some("option:terror"));
    assert_eq!(on_terror["table"]["choice"]["pinned"], "terror");
    key(&mut app, "Tab", None);
    let on_commit = key(&mut app, "Tab", None);
    assert_eq!(on_commit["table"]["choice"]["focus"], "0:terror");
    let chosen = key(&mut app, "Enter", Some("0:terror"));
    assert_eq!(chosen["observation"]["action"], "Terror");
    assert_eq!(chosen["observation"]["executing"], "M26");
    assert_eq!(chosen["table"]["briefing"], Value::Null);

    // During Terror: the ribbon reopens the briefings, with Kidnap and Terror tabs.
    let ribbon = part(&chosen, "briefing-ribbon");
    let expected = &layout["execution"]["parts"]["briefing-ribbon"];
    assert!((ribbon[0] - expected[0].as_f64().unwrap() * ratio).abs() <= 2.);
    app.activate("briefing-ribbon", 1);
    let reopened = app.observe();
    assert_eq!(reopened["table"]["briefing"]["option"], "terror");
    assert_eq!(
        reopened["table"]["briefing"]["tabs"],
        json!(["terror:kidnap", "terror"])
    );
    compare(&reopened, &layout["execution"]["parts"], &[0, 1, 2, 3]);
    compare(&reopened, &layout["execution"]["text"], &[0, 1]);
    app.capture("ribbon-terror");
    app.activate("briefing-tab:kidnap", 1);
    let kidnap = app.observe();
    assert_eq!(kidnap["table"]["briefing"]["option"], "terror:kidnap");
    let words = briefing(&kidnap);
    assert!(
        words.contains("RESULT: Proceeds 1–6 Resources while Government has 6; at least 1."),
        "{words}"
    );
    no_advice(&words);
    app.capture("ribbon-kidnap");
    // The tag closes and reopens from the ribbon by key, too.
    let closed = key(&mut app, "Escape", None);
    assert_eq!(closed["table"]["briefing"], Value::Null);
    assert_eq!(
        key(&mut app, "Tab", None)["table"]["choice"]["focus"],
        "briefing-ribbon"
    );
    let opened = key(&mut app, "Enter", Some("briefing-ribbon"));
    assert_eq!(opened["table"]["briefing"]["option"], "terror:kidnap");
    key(&mut app, "Tab", None);
    let on_tab = key(&mut app, "Tab", None);
    assert_eq!(on_tab["table"]["choice"]["focus"], "briefing-tab:terror");
    let terror_tab = key(&mut app, "Enter", Some("briefing-tab:terror"));
    assert_eq!(terror_tab["table"]["briefing"]["option"], "terror");

    // Agreement after a transfer: Government gives 2 (6 → 4, 26July 0 → 2).
    for action in [
        "transfer",
        "transfer:0:1",
        "amount:1",
        "amount:2",
        "authorize",
    ] {
        app.choose(action);
    }
    let after = app.observe();
    assert_eq!(after["observation"]["resources"], json!([4, 2, 5, 15]));
    assert!(
        briefing(&after).contains("26July has 2."),
        "{}",
        briefing(&after)
    );
    let rev = after["observation"]["revision"].as_u64().unwrap();
    app.activate("briefing-tab:kidnap", rev);
    let words = briefing(&app.observe());
    assert!(words.contains("while Government has 4;"), "{words}");
    app.capture("after-transfer-kidnap");
    // After a Terror: 26July paid 1 (2 → 1).
    app.choose("terror:oriente");
    let after = app.observe();
    assert_eq!(after["observation"]["resources"][1], 1);
    app.activate(
        "briefing-tab:terror",
        after["observation"]["revision"].as_u64().unwrap(),
    );
    let words = briefing(&app.observe());
    assert!(words.contains("26July has 1."), "{words}");
    app.capture("after-terror");
    // After removing a Santiago Guerrilla, Kidnap is no longer possible there: 1
    // Guerrilla does not outnumber 1 Police (R 4.3.3).
    // #73: removal is on the map; the open tag covers Santiago, so Escape closes it
    // first, and the ribbon reopens it afterwards.
    app.choose("remove");
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.request(json!({"op":"key","revision":rev,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    app.remove("santiago:underground", 1);
    app.choose("done-removal");
    let after = app.observe();
    let rev = after["observation"]["revision"].as_u64().unwrap();
    if after["table"]["choice"]["ribbon"] != true {
        app.activate("briefing-ribbon", rev);
    }
    app.activate("briefing-tab:kidnap", rev);
    let view = app.observe();
    let words = briefing(&view);
    let kidnap = view["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str().unwrap().ends_with(":kidnap"))
        .unwrap()
        .clone();
    assert_eq!(kidnap["supported"], false);
    let reason = kidnap["explanation"].as_str().unwrap();
    assert!(reason.starts_with("1 Guerrillas (Active + Underground) vs 1 Police; must outnumber."));
    assert!(
        words.contains("RESULT: Unavailable now. 1 Guerrillas (Active + Underground) vs 1 Police; must outnumber."),
        "{words}"
    );
    assert!(!words.contains("Santiago"), "{words}");
    app.capture("after-removal-kidnap");
    app.quit();
}

/// The owner's clash rule (#58, carried to #69 and #72): while a space popover is
/// open it wins over the AC-B briefing tag; opening the popover closes the tag, which
/// reopens from the BRIEFING ribbon (checked in the AC-B test above). Neither changes
/// the match. At 3840×2160 the open tag covers Santiago, so the popover is opened on
/// Oriente (blocked at 0 Resources). The test ends with Oriente's popover kept open:
/// at this size it never reports itself placed, so the seat refuses later input (a
/// #72 defect reported by the merge of #69).
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained clash captures"]
fn native_space_popover_closes_the_briefing_tag() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-69"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("clash-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS")
            .expect("Set CUBA_TEST_CARDS")
            .into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("clash.json"),
        root,
        step: 0,
    };
    eprintln!("Clash rule evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app.choose("terror");
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("briefing-ribbon", rev);
    let tag = app.observe();
    assert_eq!(tag["table"]["briefing"]["option"], "terror");
    assert_eq!(tag["table"]["choice"]["ribbon"], true);
    app.capture("clash-tag-open");
    // A click through the open tag reaches nothing on the map.
    let covered = app.request(
        json!({"op":"activate","revision":rev,"component":"space:santiago"}),
        false,
    );
    assert!(
        covered["error"]
            .as_str()
            .unwrap()
            .contains("not component space:santiago"),
        "{}",
        covered["error"]
    );
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("space:oriente", rev);
    let popover = app.observe();
    assert_eq!(popover["table"]["popover"]["pinned"], true);
    assert_eq!(popover["table"]["popover"]["space"], "oriente");
    assert_eq!(popover["table"]["briefing"], Value::Null);
    assert_eq!(popover["table"]["choice"]["ribbon"], false);
    assert_eq!(popover["observation"]["revision"], rev);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("clash-popover-closes-tag");
    // Clicking Oriente again closes its popover (#73 made it settle at this size).
    app.activate("space:oriente", rev);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    app.activate("space:oriente", rev);
    assert_eq!(app.observe()["table"]["popover"]["space"], "oriente");

    // Orchestrator implementation judgement (#58 carry-forward): whichever opened
    // last wins, and the tag closes by Escape or a click on the map outside it.
    // Opening the tag from the ribbon closes the open popover.
    app.activate("briefing-ribbon", rev);
    let reopened = app.observe();
    assert_eq!(reopened["table"]["choice"]["ribbon"], true);
    assert_eq!(reopened["table"]["popover"], Value::Null);
    app.capture("clash-tag-closes-popover");
    // Escape closes the tag; the targets under it are clickable again.
    app.request(json!({"op":"key","revision":rev,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    let escaped = app.observe();
    assert_eq!(escaped["table"]["choice"]["ribbon"], false);
    assert_eq!(escaped["table"]["briefing"], Value::Null);
    app.activate("space:santiago", rev);
    assert_eq!(app.observe()["table"]["popover"]["space"], "santiago");
    // A click on the map outside the tag closes it too, and changes nothing else.
    app.activate("briefing-ribbon", rev);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    app.activate("space:camaguey-city", rev);
    let outside = app.observe();
    assert_eq!(outside["table"]["choice"]["ribbon"], false);
    assert_eq!(outside["table"]["briefing"], Value::Null);
    assert_eq!(outside["table"]["popover"], Value::Null);
    assert_eq!(outside["observation"], popover["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.activate("space:santiago", rev);
    assert_eq!(app.observe()["table"]["popover"]["space"], "santiago");
    app.capture("clash-map-click-closed-tag");
    app.quit();
}

/// Rows of the shown piece popover as one text, for checking previewed facts.
fn popover_text(view: &Value) -> String {
    view["table"]["popover"]["card"]["rows"][0]["lines"]
        .as_array()
        .unwrap_or_else(|| panic!("no piece popover: {}", view["table"]["popover"]))
        .iter()
        .map(|l| l[1].as_str().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Removal mode's map agrees with the authority: a marker under each 26July group in
/// the state its choices give, and the veil on exactly the spaces holding no 26July
/// piece; no space rule shows.
fn assert_removal_map_agrees(view: &Value) {
    let removal = &view["table"]["removal"];
    let choices = view["observation"]["choices"].as_array().unwrap();
    let mut expected = vec![];
    for piece in removal["pieces"].as_array().unwrap() {
        let supported = piece["choices"].as_array().unwrap().iter().any(|id| {
            choices
                .iter()
                .any(|c| c["id"] == *id && c["supported"] == true)
        });
        let mark = if supported { "removable" } else { "protected" };
        assert_eq!(piece["mark"], mark, "{piece}");
        expected.push(json!({
            "piece": format!("{}:{}", piece["space"].as_str().unwrap(), piece["group"].as_str().unwrap()),
            "file": format!("atlas-piece-{mark}.png"),
        }));
    }
    let mut shown: Vec<Value> = view["table"]["markers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            // Each marker is on screen, under its piece.
            assert!(m["at"].is_array(), "{m}");
            json!({"piece": m["piece"], "file": m["file"]})
        })
        .collect();
    let key = |v: &Value| v["piece"].to_string();
    expected.sort_by_key(key);
    shown.sort_by_key(key);
    assert_eq!(shown, expected);
    let highlights: Vec<&str> = view["table"]["highlights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h.as_str().unwrap())
        .collect();
    assert!(
        highlights.iter().all(|h| h.contains("-veil-")),
        "{highlights:?}"
    );
    let dimmed = removal["dimmed"].as_array().unwrap();
    for (space, file) in [
        ("santiago", "atlas-highlight-veil-santiago.png"),
        ("oriente", "atlas-highlight-veil-oriente.png"),
        ("sierra-maestra", "atlas-highlight-veil-sierra-maestra.png"),
    ] {
        assert_eq!(
            highlights.contains(&file),
            dimmed.iter().any(|d| d == space),
            "{space}: {highlights:?}"
        );
    }
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained removal-mode captures against art/proposals/issue-66/hf-a-survey-ink-states.jpg"]
fn native_removal_mode_marks_pieces_and_removes_through_the_piece_popover() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-73"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("removal-mode-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let socket_dir = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat.sock"),
        save: root.join("removal.json"),
        root,
        step: 0,
    };
    eprintln!("Removal mode evidence: {}", app.root.display());
    app.launch();
    app.choose("terror");
    app.choose("remove");
    // Entering: each own group is marked removable; every other space is dimmed.
    let mode = app.observe();
    assert_removal_map_agrees(&mode);
    assert_eq!(mode["table"]["markers"].as_array().unwrap().len(), 3);
    assert_eq!(mode["table"]["map_targets"], Value::Null);
    // The panel offers only Done; the removals are on the map.
    let panel: Vec<&str> = mode["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["choice"] == true)
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(panel.iter().all(|c| !c.contains(":remove:")), "{panel:?}");
    assert!(
        panel.iter().any(|c| c.ends_with(":done-removal")),
        "{panel:?}"
    );
    app.capture("removal-mode");

    // Hovering a removable piece shows its popover; a click keeps it open.
    let hovered = app.point(&format!("piece:{}", piece("santiago:underground")));
    assert_eq!(hovered["table"]["popover"]["kind"], "piece");
    assert_eq!(hovered["table"]["popover"]["pinned"], false);
    let revision = mode["observation"]["revision"].as_u64().unwrap();
    app.activate(
        &format!("piece:{}", piece("santiago:underground")),
        revision,
    );
    let santiago = app.observe();
    let card = &santiago["table"]["popover"]["card"];
    assert_eq!(santiago["table"]["popover"]["pinned"], true);
    assert_eq!(card["title"], "26July Guerrilla · Underground");
    assert_eq!(card["stepper"], json!({"value": 1, "max": 2, "count": 2}));
    // Removing 1 before Kidnap: 1 Guerrilla does not exceed 1 Police (R 4.3.3).
    let text = popover_text(&santiago);
    assert!(text.contains("Then unavailable: Kidnap"), "{text}");
    assert!(text.contains("Control: unchanged (Government)."), "{text}");
    assert_popover_placed(&santiago);
    assert!(assert_inline_emblems(&santiago) > 0);
    assert!(
        emblems_in(&santiago, "Control: unchanged (Government). Then unavailable: Kidnap, going back to the action choice.")
            .contains(&"Government".to_owned())
    );
    app.capture("santiago-popover");

    // A quantity greater than one: the stepper chooses the authority's choice for 2.
    app.activate("popover:quantity:plus", revision);
    let two = app.observe();
    assert_eq!(two["table"]["popover"]["card"]["stepper"]["value"], 2);
    assert_eq!(
        two["table"]["popover"]["card"]["rows"][0]["button"],
        "Remove 2 Guerrillas"
    );
    assert!(popover_text(&two).contains("2 Underground Guerrillas return to Available."));
    app.capture("santiago-popover-two");
    // Back to 1, and Cancel closes it with nothing decided.
    app.activate("popover:quantity:minus", revision);
    assert_eq!(
        app.observe()["table"]["popover"]["card"]["stepper"]["value"],
        1
    );
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("popover:cancel", revision);
    let cancelled = app.observe();
    assert_eq!(cancelled["table"]["popover"], Value::Null);
    assert_eq!(cancelled["observation"], mode["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);

    // Removing a Santiago Guerrilla: Kidnap becomes unavailable, the mode stays open.
    app.remove("santiago:underground", 1);
    let one = app.observe();
    assert_eq!(one["observation"]["removal"], "Selecting");
    assert_eq!(one["observation"]["map_piece_count"], 28);
    assert_eq!(one["table"]["popover"], Value::Null);
    assert_removal_map_agrees(&one);
    app.capture("after-santiago-removal");
    // Restart in removal mode after a removal: the same state, once.
    app.quit();
    app.launch();
    let resumed = app.observe();
    assert_eq!(resumed["observation"], one["observation"]);
    assert_removal_map_agrees(&resumed);

    // Removing the Base: 26July 7 → 6, Sierra Maestra uncontrolled.
    let revision = resumed["observation"]["revision"].as_u64().unwrap();
    app.point(&format!("piece:{}", piece("sierra-maestra:base")));
    app.activate(&format!("piece:{}", piece("sierra-maestra:base")), revision);
    let base = app.observe();
    assert_eq!(base["table"]["popover"]["card"]["title"], "26July Base");
    assert_eq!(base["table"]["popover"]["card"]["stepper"], Value::Null);
    let text = popover_text(&base);
    assert!(
        text.contains("Control of Sierra Maestra: 26July → none."),
        "{text}"
    );
    assert!(text.contains("26July 7 → 6 (margin -8 → -9)"), "{text}");
    app.capture("base-popover");
    app.activate("popover:commit", revision);
    app.request(json!({"op":"settle"}), true);
    let after = app.observe();
    assert_eq!(after["observation"]["standings"][1]["score"], 6);
    // Sierra Maestra now holds no 26July piece, so it dims.
    assert_removal_map_agrees(&after);
    assert!(
        after["table"]["highlights"]
            .as_array()
            .unwrap()
            .contains(&json!("atlas-highlight-veil-sierra-maestra.png"))
    );
    app.capture("after-base-removal");

    // Done leaves the mode: the markers go and Terror targeting returns.
    app.request(json!({"op":"settle"}), true);
    app.choose("done-removal");
    let done = app.observe();
    assert_eq!(done["observation"]["removal"], Value::Null);
    assert_eq!(done["table"]["removal"], Value::Null);
    assert_eq!(done["table"]["markers"], json!([]));
    assert_map_agrees(&done);
    let kidnap = done["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str().unwrap().ends_with(":kidnap"))
        .unwrap()
        .clone();
    assert_eq!(kidnap["supported"], false);
    app.capture("done");
    app.quit();

    // The protected piece: after the Kidnap roll owes Santiago's Terror, one of the
    // two may go; the last shows its reason on hover, and a click opens nothing.
    app.save = app.root.join("protected.json");
    app.launch();
    for action in ["terror", "kidnap", "kidnap:roll", "remove"] {
        app.choose(action);
    }
    let owed = app.observe();
    let revision = owed["observation"]["revision"].as_u64().unwrap();
    app.activate(
        &format!("piece:{}", piece("santiago:underground")),
        revision,
    );
    let limited = app.observe();
    assert_eq!(
        limited["table"]["popover"]["card"]["stepper"],
        json!({"value": 1, "max": 1, "count": 2})
    );
    assert!(popover_text(&limited).contains("At most 1: the other is protected"));
    let controls: Vec<&str> = limited["table"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(!controls.contains(&"popover:quantity:plus"), "{controls:?}");
    app.capture("owed-stack-at-most-one");
    app.activate("popover:commit", revision);
    app.request(json!({"op":"settle"}), true);
    let last = app.observe();
    assert_removal_map_agrees(&last);
    let marker = last["table"]["markers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["piece"] == "santiago:underground")
        .unwrap()
        .clone();
    assert_eq!(marker["file"], "atlas-piece-protected.png");
    let tag = app.point(&format!("piece:{}", piece("santiago:underground")));
    assert_eq!(tag["table"]["popover"]["kind"], "protected");
    assert_eq!(
        tag["table"]["popover"]["card"]["title"],
        "Protected · 26July Guerrilla"
    );
    assert!(popover_text(&tag).contains("accepted commitment policy"));
    app.capture("protected-tag");
    let saved = std::fs::read(&app.save).unwrap();
    let revision = last["observation"]["revision"].as_u64().unwrap();
    app.activate(
        &format!("piece:{}", piece("santiago:underground")),
        revision,
    );
    let clicked = app.observe();
    assert_eq!(clicked["observation"], last["observation"]);
    assert_eq!(clicked["table"]["popover"], Value::Null);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.quit();

    // At the owner's aspect, for comparison with the selected design's states.
    app.save = app.root.join("owner-aspect.json");
    app.launch_size("3840x2160");
    app.choose("terror");
    app.choose("remove");
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(
        &format!("piece:{}", piece("santiago:underground")),
        revision,
    );
    let wide = app.observe();
    let [left, _, right, _] = rect_of(&wide["table"]["popover"]["bounds"]);
    let scale = (3840f64 / 1920.).min(2160. / 1280.);
    assert!(
        (right - left - 340. * scale).abs() <= 2.,
        "{}",
        wide["table"]["popover"]
    );
    app.capture("removal-popover-3840x2160");
    app.quit();
}

/// The EL-A eligibility indicator (#70) in each state the task reaches: the opening
/// (unchanged all through Terror), after Pass, after Operation Only and after Operation
/// with Special Activity. At 3840×2160 every drawn part is compared with the layout
/// record from `atlas-eligibility.figcraft` within 2 px, the drawn state agrees with the
/// rules authority's Eligibility observation, nobody is shown Ineligible, and every
/// box's hover tag is reachable through the seat and dismisses when the pointer leaves.
/// Crops of the panel are retained for inspection against the layout's exports.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained eligibility captures"]
fn native_eligibility_indicator_matches_layout_in_every_state() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-70"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("el-a-")
        .tempdir_in(evidence)
        .unwrap()
        .keep()
        .canonicalize()
        .unwrap();
    let cards: PathBuf = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    eprintln!("EL-A evidence: {}", root.display());
    let layout: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-eligibility.layout.json")).unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    let scaled =
        |r: &Value| -> [f64; 4] { std::array::from_fn(|i| r[i].as_f64().unwrap() * ratio) };
    let part = |view: &Value, name: &str| -> [f64; 4] {
        let found = view["table"]["eligibility"]["parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["part"] == name)
            .unwrap_or_else(|| panic!("no native part {name}: {}", view["table"]["eligibility"]))
            .clone();
        rect_of(&found["bounds"])
    };
    // Each group's edges: left, top, right, bottom, or the horizontal centre (4).
    let compare = |view: &Value, group: &Value, edges: &[usize], shift: [f64; 2]| {
        for (name, r) in group.as_object().unwrap() {
            let [x, y, w, h] = scaled(r);
            let (x, y) = (x + shift[0], y + shift[1]);
            let expected = [x, y, x + w, y + h, x + w / 2.];
            let n = part(view, name);
            let native = [n[0], n[1], n[2], n[3], (n[0] + n[2]) / 2.];
            for &edge in edges {
                assert!(
                    (native[edge] - expected[edge]).abs() <= 2.,
                    "{name} edge {edge}: native {n:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    };
    let compare_state = |view: &Value, state: &str| {
        let record = &layout["states"][state];
        assert!(record.is_object(), "no layout state {state}");
        compare(view, &record["images"], &[0, 1, 2, 3], [0., 0.]);
        compare(view, &record["text"], &[0, 1], [0., 0.]);
        compare(view, &record["centered_text"], &[1, 4], [0., 0.]);
    };
    // The drawn track agrees with the rules authority's record, and nobody is
    // Ineligible inside the task (R 2.3.7).
    let agree = |view: &Value| {
        let e = &view["observation"]["eligibility"];
        let track = &view["table"]["eligibility"]["track"];
        assert_eq!(e["eligible"], json!([true, true, true, true]));
        let rows = track["rows"].as_array().unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| r["faction"].clone())
                .collect::<Vec<_>>(),
            e["order"].as_array().unwrap().clone()
        );
        for row in rows {
            assert_eq!(row["eligible"], true, "{row}");
            let role = if row["faction"] == e["first"] {
                json!("1st")
            } else if row["faction"] == e["second"] {
                json!("2nd")
            } else {
                Value::Null
            };
            assert_eq!(row["role"], role, "{row}");
            assert_eq!(
                row["acted"],
                json!(row["faction"] == "M26" && !e["m26_box"].is_null())
            );
            let name = row["faction"].as_str().unwrap();
            let key = if name == "M26" {
                "26july".to_owned()
            } else {
                name.to_lowercase()
            };
            assert!(
                view["table"]["eligibility"]["parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["part"] == format!("eligibility-row:{key}:eligible")),
                "{key} is not marked Eligible"
            );
        }
        let occupied = match e["m26_box"].as_str() {
            None => Value::Null,
            Some("Pass") => json!("eligibility:pass"),
            Some("OperationOnly") => json!("eligibility:operation-only"),
            Some(_) => json!("eligibility:operation-special-activity"),
        };
        assert_eq!(track["occupied"], occupied);
        assert_eq!(track["ineligible"], json!([]));
    };
    let boxes = [
        "eligibility:pass",
        "eligibility:operation-only",
        "eligibility:operation-special-activity",
    ];
    // Hovering each box opens its tag beside the panel, at the layout's offset from
    // that box, without changing the match; pointing away dismisses it.
    let hover_all = |app: &mut Native, state: &str| {
        let before = app.observe();
        let tag = &layout["tag"];
        let anchor = scaled(
            &layout["states"]["operation-special-activity"]["images"]["eligibility:operation-special-activity"],
        );
        for name in boxes {
            let view = app.point(name);
            let view = if view["table"]["eligibility"]["tag"].is_object() {
                view
            } else {
                app.observe()
            };
            let shown = &view["table"]["eligibility"]["tag"];
            assert_eq!(shown["slot"], name, "{state}: {}", view["table"]);
            let occupied = before["table"]["eligibility"]["track"]["occupied"] == name;
            assert_eq!(shown["occupied"], occupied);
            assert!(
                shown["note"]
                    .as_str()
                    .unwrap()
                    .contains("Nobody becomes Ineligible during this card")
            );
            let target = part(&view, name);
            let shift = [0., target[1] - anchor[1]];
            compare(&view, &tag["stretched"], &[0, 1, 2], shift);
            compare(&view, &tag["images"], &[0, 1, 2, 3], shift);
            compare(&view, &tag["text"], &[0, 1], shift);
            compare(&view, &tag["right_text"], &[1, 2], shift);
            compare(&view, &tag["note"], &[0], shift);
            let plate = part(&view, "eligibility-tag");
            let note = part(&view, "eligibility-tag-note");
            assert!(note[3] <= plate[3], "the tag's plate holds its note");
            assert_eq!(view["observation"], before["observation"]);
            if occupied || state == "open" {
                app.capture(&format!(
                    "{state}-hover-{}",
                    name.trim_start_matches("eligibility:")
                ));
            }
        }
        // The tag dismisses like the other hover tags when the pointer leaves.
        let away = app.point("reader");
        assert!(away["table"]["eligibility"]["tag"].is_null());
        assert_eq!(app.observe()["table"]["eligibility"]["tag"], Value::Null);
    };
    let crop = |app: &Native, capture: &str, name: &str| {
        let image = image::open(app.root.join(format!("{capture}.png")))
            .unwrap()
            .to_rgb8();
        let [x, y, w, h] = scaled(&layout["crop"]);
        image::imageops::crop_imm(&image, x as u32, y as u32, w as u32, h as u32)
            .to_image()
            .save(app.root.join(format!("{name}.png")))
            .unwrap();
    };
    let session = |name: &str| {
        let socket_dir = tempfile::tempdir().unwrap();
        let app = Native {
            cards: cards.clone(),
            socket: socket_dir.path().join("seat.sock"),
            save: root.join(format!("{name}.json")),
            root: root.join(name),
            step: 0,
        };
        std::fs::create_dir_all(&app.root).unwrap();
        (app, socket_dir)
    };

    // The opening, and all through Terror until End Terror.
    let (mut app, _socket) = session("opening");
    app.launch_size("3840x2160");
    let open = app.observe();
    assert_eq!(open["observation"]["action"], "Open");
    agree(&open);
    compare_state(&open, "open");
    app.capture("open");
    crop(&app, "open", "panel-open");
    hover_all(&mut app, "open");
    app.choose("terror");
    let terror = app.observe();
    agree(&terror);
    compare_state(&terror, "open");
    app.capture("during-terror");
    app.choose("end-terror");
    let only = app.observe();
    assert_eq!(
        only["observation"]["eligibility"]["m26_box"],
        "OperationOnly"
    );
    agree(&only);
    compare_state(&only, "operation-only");
    app.capture("operation-only");
    crop(&app, "operation-only", "panel-operation-only");
    hover_all(&mut app, "operation-only");
    app.quit();

    // After Pass: Directorio 1st, Government 2nd, 26July in the Pass box (R 2.3.3).
    let (mut app, _socket) = session("pass");
    app.launch_size("3840x2160");
    app.choose("pass");
    let pass = app.observe();
    assert_eq!(pass["observation"]["eligibility"]["first"], "Directorio");
    agree(&pass);
    compare_state(&pass, "pass");
    app.capture("pass");
    crop(&app, "pass", "panel-pass");
    hover_all(&mut app, "pass");
    // A restart shows the same recorded state.
    app.quit();
    app.launch_size("3840x2160");
    let restarted = app.observe();
    assert_eq!(restarted["observation"], pass["observation"]);
    assert_eq!(
        restarted["table"]["eligibility"]["track"],
        pass["table"]["eligibility"]["track"]
    );
    app.quit();

    // Operation with Special Activity: the layout's 16:9 frame, with its box's tag.
    let (mut app, _socket) = session("special-activity");
    app.launch_size("3840x2160");
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
    ] {
        app.choose(action);
        agree(&app.observe());
    }
    let special = app.observe();
    assert_eq!(
        special["observation"]["eligibility"]["m26_box"],
        "OperationWithSpecialActivity"
    );
    compare_state(&special, "operation-special-activity");
    hover_all(&mut app, "operation-special-activity");
    let view = app.point("eligibility:operation-special-activity");
    assert!(view["table"]["eligibility"]["tag"].is_object());
    app.capture("operation-special-activity");
    crop(
        &app,
        "operation-special-activity",
        "panel-operation-special-activity",
    );
    app.quit();

    // At 3:2, the seat's default, the camera frames Cuba wider and the track covers
    // Pinar del Río's pieces (foundation 4 requires only the 16:9 frame; #70 records
    // this as an open limit). The capture is retained for that record.
    let (mut app, _socket) = session("three-two");
    app.launch();
    let view = app.observe();
    agree(&view);
    assert_eq!(view["table"]["viewport"], json!([1920., 1280.]));
    app.capture("opening-1920x1280");
    app.quit();
}
