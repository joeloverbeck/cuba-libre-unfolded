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
    fn choose(&mut self, action: &str) {
        let view = self.observe();
        let revision = view["observation"]["revision"].as_u64().unwrap();
        self.request(
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:{action}")}),
            true,
        );
        self.request(json!({"op":"settle"}), true);
    }
    /// Rolls the opened Kidnap, lets the die land, and closes it with Continue so the
    /// panel offers the Terror choices again (#74).
    fn roll(&mut self) {
        self.choose("kidnap:roll");
        let revision = self.observe()["observation"]["revision"].as_u64().unwrap();
        self.activate("kidnap-die:continue", revision);
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
    for action in ["remove", "remove:sierra-maestra:base", "confirm-removal"] {
        app.choose(action);
    }
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
fn native_own_removal_confirmation_restart_obligation_and_all_pieces_finish() {
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
    app.choose("remove");
    app.choose("remove:santiago:underground");
    app.capture("pending-confirmation");
    let pending = app.observe();
    assert_eq!(pending["observation"]["map_piece_count"], 29);
    std::fs::copy(&app.save, app.root.join("pending-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], pending["observation"]);
    app.choose("cancel-removal");
    assert_eq!(app.observe()["observation"]["map_piece_count"], 29);
    app.choose("remove");
    app.choose("remove:santiago:underground");
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.choose("confirm-removal");
    let removed = app.observe();
    assert_eq!(removed["observation"]["map_piece_count"], 28);
    assert!(
        removed["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("Available")
    );
    app.capture("pre-kidnap-removal");
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:confirm-removal")}),
        false,
    );
    assert_eq!(app.observe()["observation"], removed["observation"]);
    std::fs::copy(&app.save, app.root.join("removed-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], removed["observation"]);
    // A real giver funds the remaining Guerrilla after Kidnap was defeated.
    for action in [
        "transfer",
        "transfer:2:1",
        "amount:1",
        "authorize",
        "terror:santiago",
        "remove",
        "remove:santiago:active",
        "confirm-removal",
        "remove",
        "remove:sierra-maestra:base",
        "confirm-removal",
    ] {
        app.choose(action);
    }
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("standing:1", rev);
    app.capture("base-and-active-removed");
    assert_eq!(app.observe()["observation"]["standings"][1]["score"], 7);
    for action in [
        "remove",
        "remove:oriente:underground",
        "confirm-removal",
        "end-terror",
    ] {
        app.choose(action);
    }
    app.capture("all-pieces-after-terror-finished");
    assert_eq!(app.observe()["observation"]["map_piece_count"], 25);
    app.quit();

    // A fresh zero-space path removes everything and still finishes.
    app.save = app.root.join("zero-space.json");
    app.launch();
    app.choose("terror");
    for target in [
        "sierra-maestra:base",
        "santiago:underground",
        "santiago:underground",
        "oriente:underground",
    ] {
        app.choose("remove");
        app.choose(&format!("remove:{target}"));
        app.choose("confirm-removal");
        if target == "sierra-maestra:base" {
            let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
            app.activate("standing:1", rev);
            assert_eq!(app.observe()["observation"]["standings"][1]["score"], 6);
            app.capture("base-removed-standing-six");
        }
    }
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
    app.choose("kidnap");
    app.roll();
    for action in [
        "remove",
        "remove:santiago:underground",
        "confirm-removal",
        "remove",
    ] {
        app.choose(action);
    }
    let owed = app.observe();
    let rev = owed["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("{rev}:remove:santiago:underground"), rev);
    assert!(
        app.observe()["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("owed")
    );
    assert_eq!(app.observe()["observation"], owed["observation"]);
    app.capture("owed-removal-refused");
    std::fs::copy(&app.save, app.root.join("owed-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], owed["observation"]);
    for action in [
        "remove:sierra-maestra:base",
        "confirm-removal",
        "remove",
        "remove:oriente:underground",
        "confirm-removal",
        "terror:santiago",
        "remove",
        "remove:santiago:active",
        "confirm-removal",
    ] {
        app.choose(action);
    }
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
    for action in ["remove:santiago:active", "confirm-removal"] {
        app.choose(action);
    }
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
    for action in [
        "remove",
        "remove:santiago:underground",
        "confirm-removal",
        "remove",
        "remove:oriente:active",
        "confirm-removal",
        "remove",
        "remove:sierra-maestra:base",
        "confirm-removal",
        "end-terror",
    ] {
        app.choose(action);
    }
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
    app.roll();
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
    // The restart shows the landed die again (#74); Continue saves nothing.
    let rev = rolled["observation"]["revision"].as_u64().unwrap();
    assert_eq!(app.observe()["table"]["die"]["landed"], true);
    app.activate("kidnap-die:continue", rev);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved_roll);
    app.request(
        json!({"op":"activate","revision":1,"choice":"1:kidnap"}),
        false,
    );
    assert_eq!(app.observe()["observation"], rolled["observation"]);
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
    app.roll();
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
            app.roll();
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
    for action in ["remove", "remove:oriente:active", "confirm-removal"] {
        app.choose(action);
    }
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
    press(&mut app, 54);
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
        let half = token_changes(
            &app.root.join("loop-start.png"),
            &app.root.join("loop-half.png"),
            at,
        );
        assert!(half > 20, "the bell visibly swings: {half} changed pixels");
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
    app.roll();
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

        // The action choice (#61): card 17's order and 26July's Resources by emblem,
        // and an unsupported option's explanation in the session notice.
        let open = app.observe();
        checked += assert_inline_emblems(&open);
        let order = open["table"]["inline_text"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["text"].as_str().unwrap().starts_with("Order: "))
            .expect("the order line");
        let mut drawn: Vec<_> = order["emblems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["faction"].as_str().unwrap())
            .collect();
        assert_eq!(
            drawn.len(),
            5,
            "four in order, then 26July's Resources: {order}"
        );
        drawn.sort_unstable();
        drawn.dedup();
        assert_eq!(drawn.len(), 4);
        app.capture("action-choice");
        app.activate("0:rally", 0);
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
        app.roll();
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
    // Every listed option is a real control; only Pass and Terror are playable.
    for choice in choices {
        let id = choice["id"].as_str().unwrap();
        let control = named(&open, id).unwrap_or_else(|| panic!("{id} not drawn"));
        assert_eq!(control["choice"], choice["supported"], "{id}");
    }
    app.capture("action-choice");
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("0:rally", 0);
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
    let pass = card(&mut app, "pass", "pass-preview");
    assert!(pass.contains("26July Resources: 0 → 1"), "{pass}");
    assert!(pass.contains("Directorio, 1st Eligible"), "{pass}");
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
    app.roll();
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

/// Pixels that differ between two captures inside `rect` ([left, top, right, bottom]).
fn changes_within(first: &Path, second: &Path, rect: [f64; 4]) -> usize {
    let a = image::open(first).unwrap().to_rgb8();
    let b = image::open(second).unwrap().to_rgb8();
    assert_eq!(a.dimensions(), b.dimensions());
    (rect[0] as u32..rect[2] as u32)
        .flat_map(|x| (rect[1] as u32..rect[3] as u32).map(move |y| (x, y)))
        .filter(|&(x, y)| a.get_pixel(x, y) != b.get_pixel(x, y))
        .count()
}

/// #74: the Kidnap die lands on the committed face (ADR 0001). For each of the six
/// faces, from #8's seeds whose rolls are known (`tests/kidnap.rs`), the tray in the
/// operation panel matches FigCraft frame C, the die advances with the manual clock and
/// stays unsettled until it lands, the ledger keeps the balances before the roll until
/// then, and the landed die shows the committed face with the result. A restart during
/// the tumble shows the landed die and replays nothing.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_kidnap_die_lands_on_each_face_and_restarts_landed() {
    use cuba_libre_unfolded::save::Session;
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| "local/verification/issue-74".into());
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("die-")
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
    eprintln!("Kidnap die native evidence: {}", app.root.display());
    // 16:9, like the owner's 6144x3456 display that FigCraft frame C lays out.
    let size = "3840x2160";
    let layout: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-kidnap-tray.layout.json")).unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    let scaled = |rect: &Value| -> [f64; 4] {
        let r: Vec<f64> = rect
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() * ratio)
            .collect();
        [r[0], r[1], r[0] + r[2], r[1] + r[3]]
    };
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
    let pending = |view: &Value| -> Vec<String> {
        view["table"]["pending"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_owned())
            .collect()
    };
    let rate = 0.5;
    let step = 1. / 60.;
    for (seed, die) in [(19u64, 1u64), (0, 2), (5, 3), (3, 4), (2, 5), (1, 6)] {
        // A new save with this seed: created by the library as the app would, then
        // resealed with the known random state.
        app.save = app.root.join(format!("face-{die}.json"));
        drop(Session::open(&app.save, None).unwrap());
        let mut record: Value = serde_json::from_slice(&std::fs::read(&app.save).unwrap()).unwrap();
        record["payload"]["random"] = seed.into();
        let identity = serde_json::from_value(record["identity"].clone()).unwrap();
        std::fs::write(
            &app.save,
            unfolded_record_replacement::save::seal(&identity, &record["payload"]),
        )
        .unwrap();
        app.launch_size(size);
        app.choose("terror");
        app.choose("kidnap");
        let opened = app.observe();
        assert_eq!(ledger(&opened), ["6", "0", "5", "15"]);
        let revision = opened["observation"]["revision"].as_u64().unwrap();
        app.request(
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:kidnap:roll")}),
            true,
        );
        // The result is committed and saved before the die moves.
        let committed = std::fs::read(&app.save).unwrap();
        let saved: Value = serde_json::from_slice(&committed).unwrap();
        assert_eq!(saved["payload"]["kidnap"]["die"], die);
        assert_eq!(saved["payload"]["kidnap"]["revision"], revision + 1);
        let first = app.request(json!({"op":"settle","steps":16}), true);
        assert_eq!(first["status"], "unsettled", "{first}");
        let first = app.observe();
        let shown = &first["table"]["die"];
        assert_eq!(shown["face"], die);
        assert_eq!(shown["revision"], revision + 1);
        assert_eq!(shown["landed"], false, "{shown}");
        assert!(pending(&first).contains(&"Kidnap die landing".to_owned()));
        assert_eq!(first["table"]["settled"], false);
        assert_eq!(first["observation"]["kidnap"]["die"], die);
        // In the air, the ledger keeps the balances before the roll; the standings stay.
        assert_eq!(ledger(&first), ["6", "0", "5", "15"]);
        assert!(
            first["table"]["standings"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["visible"] == true)
        );
        let tray = rect_of(&shown["viewport"]);
        if die == 1 {
            // The tray matches frame C on all four edges, within 2 px.
            let expected = scaled(&layout["parts"]["kidnap-tray viewport"]);
            for (n, e) in tray.iter().zip(expected) {
                assert!(
                    (n - e).abs() <= 2.,
                    "tray {tray:?} differs from layout {expected:?}"
                );
            }
        }
        app.capture(&format!("face-{die}-air-a"));
        // The manual clock moves the die: six steps advance the clip by 6/60 × 0.5 s.
        app.request(json!({"op":"settle","steps":6}), true);
        let second = app.observe();
        let advanced =
            second["table"]["die"]["seek"].as_f64().unwrap() - shown["seek"].as_f64().unwrap();
        assert!((advanced - 6. * step * rate).abs() < 1e-4, "{advanced}");
        app.capture(&format!("face-{die}-air-b"));
        let moved = changes_within(
            &app.root.join(format!("face-{die}-air-a.png")),
            &app.root.join(format!("face-{die}-air-b.png")),
            tray,
        );
        assert!(moved > 200, "the tumble moved {moved} pixels in the tray");
        if die == 3 {
            // Restart during the tumble: the landed die, never a replay or a reroll.
            let bytes = std::fs::read(&app.save).unwrap();
            app.quit();
            app.launch_size(size);
            let restarted = app.observe();
            assert_eq!(restarted["table"]["die"]["landed"], true, "{restarted}");
            assert_eq!(
                restarted["table"]["die"]["seek"],
                restarted["table"]["die"]["duration"]
            );
            assert_eq!(restarted["observation"]["kidnap"]["die"], die);
            assert_eq!(std::fs::read(&app.save).unwrap(), bytes);
            app.capture(&format!("face-{die}-restarted"));
        }
        app.request(json!({"op":"settle"}), true);
        let landed = app.observe();
        assert_eq!(landed["table"]["settled"], true, "{landed}");
        let shown = &landed["table"]["die"];
        assert_eq!(shown["landed"], true);
        assert_eq!(shown["face"], die);
        assert!(shown["seek"].as_f64().unwrap() >= shown["landing"].as_f64().unwrap());
        // On landing: the result names the faction, and the ledger shows the transfer.
        let after = [6 - die, die, 5, 15].map(|v| v.to_string());
        assert_eq!(ledger(&landed), after);
        assert_eq!(
            landed["observation"]["resources"],
            json!([6 - die, die, 5, 15])
        );
        let texts = shown["text"].as_array().unwrap();
        let text = |part: &str| texts.iter().find(|t| t["part"] == part).unwrap().clone();
        assert_eq!(
            text("kidnap-result title")["text"],
            format!("26July rolls {die}")
        );
        assert_eq!(
            text("kidnap-result detail")["text"],
            format!(
                "Kidnap: Government → 26July, {die} Resource{}",
                if die == 1 { "" } else { "s" }
            )
        );
        if die == 1 {
            // Text on left and top, within 2 px.
            for (part, rect) in layout["text"].as_object().unwrap() {
                let expected = scaled(rect);
                let native = rect_of(&text(part)["bounds"]);
                for (n, e) in native.iter().zip(expected).take(2) {
                    assert!(
                        (n - e).abs() <= 2.,
                        "{part}: native {native:?} differs from layout {expected:?}"
                    );
                }
            }
        }
        assert!(
            landed["table"]["notice"]
                .as_str()
                .unwrap()
                .contains(&format!("Kidnap rolled {die}"))
        );
        app.capture(&format!("face-{die}-landed"));
        // No presentation timing is saved: the record is the one the commit wrote.
        assert_eq!(std::fs::read(&app.save).unwrap(), committed);
        if die == 2 {
            // The tray never takes a region pick: the seat's pick of Santiago, beside
            // which the panel shows the die, reaches the region and changes nothing.
            let revision = landed["observation"]["revision"].as_u64().unwrap();
            app.activate("space:santiago", revision);
            assert_eq!(app.observe()["observation"], landed["observation"]);
            assert_eq!(app.observe()["table"]["die"]["landed"], true);
        }
        // Continue returns the panel to Terror: by Enter on the focused control once,
        // otherwise by its button.
        let revision = landed["observation"]["revision"].as_u64().unwrap();
        if die == 1 {
            app.request(
                json!({"op":"key","revision":revision,"key":"Enter","focused":"Continue Terror"}),
                true,
            );
            app.request(json!({"op":"settle"}), true);
        } else {
            app.activate("kidnap-die:continue", revision);
        }
        let continued = app.observe();
        assert!(continued["table"]["die"].is_null(), "{continued}");
        assert_eq!(continued["observation"], landed["observation"]);
        assert_eq!(std::fs::read(&app.save).unwrap(), committed);
        app.choose("terror:santiago");
        assert_eq!(app.observe()["observation"]["owed_terror"], Value::Null);
        app.quit();
    }
}
