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
        // During Terror a step is committed by pinning its row and then pressing the
        // commit at the panel's foot (EX-A, #84). Opening a pending request's decision
        // is the commit's default, so a pinned withdrawal is unpinned first.
        let terror =
            view["observation"]["action"] == "Terror" && view["observation"]["finished"] == false;
        let pinned = view["table"]["choice"]["pinned"].clone();
        if terror && is_step(action) && pinned != action {
            self.activate(&format!("step:{action}"), revision);
            let view = self.observe();
            assert_eq!(view["table"]["choice"]["pinned"], action, "pinned");
        } else if terror && action == "reopen" && pinned.is_string() && pinned != "transfer" {
            self.activate(&format!("step:{}", pinned.as_str().unwrap()), revision);
        }
        // A transfer's amount and, while composing, its counterpart and direction are
        // set in the modal before its button submits them (#71).
        let parts: Vec<&str> = action.split(':').collect();
        let amount = match parts[..] {
            [verb @ ("give" | "ask"), other, n] => {
                let key = ["government", "26july", "directorio", "syndicate"]
                    [other.parse::<usize>().unwrap()];
                self.activate(&format!("transfer:tab:{key}"), revision);
                self.activate(&format!("transfer:direction:{verb}"), revision);
                Some(n)
            }
            ["authorize", n] => Some(n),
            _ => None,
        };
        if let Some(n) = amount {
            self.type_amount(n);
        }
        self.request(
            json!({"op":"activate","revision":revision,"choice":format!("{revision}:{action}")}),
            true,
        );
        self.request(json!({"op":"settle"}), true);
    }
    /// Rests the pointer on a step's row during Terror and returns the tag it opens
    /// (EX-A, #84): its title, each entry with its remedy chips, and its rules line.
    fn tag(&mut self, step: &str, capture: &str) -> String {
        self.point(&format!("step:{step}"));
        let view = self.observe();
        let shown = &view["table"]["briefing"];
        assert_eq!(shown["option"], step, "{}", view["table"]);
        self.capture(capture);
        tag_text(shown)
    }
    /// Pins a step that cannot be taken now and presses the disabled commit; returns
    /// the refusal it reports. Nothing is submitted.
    fn refuse(&mut self, step: &str) -> String {
        let view = self.observe();
        let revision = view["observation"]["revision"].as_u64().unwrap();
        if view["table"]["choice"]["pinned"] != step {
            self.activate(&format!("step:{step}"), revision);
        }
        self.activate("execution-commit", revision);
        self.observe()["table"]["notice"]
            .as_str()
            .unwrap()
            .to_owned()
    }
    /// Types an amount into the transfer modal's field: clears it, then the digits.
    fn type_amount(&mut self, digits: &str) {
        let revision = self.observe()["observation"]["revision"].as_u64().unwrap();
        for _ in 0..2 {
            self.request(
                json!({"op":"key","key":"Backspace","revision":revision}),
                true,
            );
        }
        for digit in digits.chars() {
            self.request(
                json!({"op":"key","key":digit.to_string(),"revision":revision}),
                true,
            );
        }
        self.request(json!({"op":"settle"}), true);
        assert_eq!(
            self.observe()["table"]["transfer"]["state"]["entry"],
            digits,
            "typed amount"
        );
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
    /// Step the seat's clock `steps` steps of 16,666,667 ns with no input. The reply
    /// takes every step asked for and leaves the table settled.
    fn advance(&mut self, steps: u32) -> Value {
        let advanced = self.request(json!({"op":"advance","steps":steps}), true);
        assert_eq!(advanced["steps"], steps, "{advanced}");
        assert_eq!(
            advanced["table"]["settled"], true,
            "advancing never holds the table: {advanced}"
        );
        advanced
    }
    fn quit(&mut self) {
        self.request(json!({"op":"quit"}), true);
    }
}
/// A step of the EX-A execution panel, which commits only after its row is pinned.
fn is_step(action: &str) -> bool {
    action.starts_with("terror:")
        || matches!(
            action,
            "kidnap" | "transfer" | "remove" | "end-terror" | "back-to-choice"
        )
}
/// A shown tag's words: title, entries (with their chips) and rules line.
fn tag_text(shown: &Value) -> String {
    let mut words = vec![shown["title"].as_str().unwrap_or_default().to_owned()];
    for entry in shown["entries"].as_array().into_iter().flatten() {
        let chips = entry["chips"]
            .as_array()
            .map(|c| {
                c.iter()
                    .map(|c| c.as_str().unwrap())
                    .collect::<Vec<_>>()
                    .join(" · ")
            })
            .unwrap_or_default();
        words.push(format!(
            "{}: {}{}",
            entry["label"].as_str().unwrap(),
            entry["text"].as_str().unwrap().replace('\n', " "),
            chips
        ));
    }
    words.push(shown["rules"].as_str().unwrap_or_default().to_owned());
    words.join(" | ")
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
        "ask:3:2",
        "authorize:2",
        "terror:santiago",
        "terror:oriente",
    ] {
        app.choose(action);
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
        "ask:2:1",
        "authorize:1",
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
    app.choose("kidnap");
    app.roll();
    app.choose("remove");
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
    app.choose(&format!("give:2:{amount}"));
    app.choose("end-terror");
    assert_eq!(app.observe()["observation"]["resources"][1], 0);
    assert!(
        app.observe()["observation"]["next"]["options"]
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
        "ask:2:2",
        "authorize:2",
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
    let refusal = app.refuse("end-terror");
    assert!(refusal.contains("owed"), "{refusal}");
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
    // A real Syndicate Resource transfer restores Oriente even if the real roll was 1.
    app.choose("transfer");
    app.choose("ask:3:1");
    app.choose("authorize:1");
    app.capture("donated-continuation");
    app.choose("terror:oriente");
    let both = app.observe();
    assert_eq!(both["observation"]["standings"][1]["score"], 10);
    app.capture("both-terrors");
    app.choose("end-terror");
    let finished = app.observe();
    assert_eq!(
        finished["observation"]["next"]["options"],
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
    // Rests the pointer on a step's row and returns its tag (EX-A, #84).
    let card = |app: &mut Native, step: &str, capture: &str| -> String { app.tag(step, capture) };
    // The pinned tag's words, as the seat reports them.
    let panel = |view: &Value, needle: &str| -> bool {
        tag_text(&view["table"]["briefing"]).contains(needle)
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
        "TARGET: Government in Santiago de Cuba, which has 6 Resources",
        "POSSIBLE PROCEEDS: 1–6 Resources · guaranteed minimum 1",
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
    // Opened, the Kidnap step holds the roll's stakes.
    assert_eq!(opened["table"]["choice"]["pinned"], "kidnap");
    let roll = card(&mut app, "kidnap", "kidnap-roll-preview");
    assert!(roll.contains("guaranteed minimum 1"), "{roll}");
    assert!(
        roll.contains("End Terror is refused until it resolves"),
        "{roll}"
    );

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
    let pill = rolled["table"]["choice_parts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["part"] == "step:kidnap-pill-label")
        .map(|p| p["text"].clone());
    assert_eq!(pill, Some(json!(format!("ROLLED {die}"))), "{rolled}");
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
fn native_resource_transfers_enable_both_orders_without_or_with_late_kidnap() {
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
    eprintln!(
        "Terror after Resource transfers, native evidence: {}",
        root.display()
    );
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
        app.choose("ask:2:2");
        app.capture("donor-authorization");
        app.choose("authorize:2");
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
            finished["observation"]["next"]["options"]
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
    // The seat's steps advance the same clock as desktop playback. The loop is
    // 24 frames at 6 a second, 240 steps; sample it every 2 steps for 250.
    for _ in 0..125 {
        let stepped = app.advance(2);
        water_frames.push(stepped["table"]["water_frame"].as_u64().unwrap());
    }
    assert!(water_frames.iter().all(|frame| *frame < 24));
    assert!(water_frames.windows(2).any(|frames| frames[1] > frames[0]));
    assert!(
        water_frames.windows(2).any(|frames| frames[1] < frames[0]),
        "authored loop must wrap: {water_frames:?}"
    );
    // Check rendered sea pixels too, not just the selected frame index. This
    // rectangle is open sea south-west of the island at home, below the eligibility
    // indicator (#70), and excludes UI details.
    let first = image::open(app.root.join("initial.png")).unwrap().to_rgb8();
    let later = image::open(app.root.join("hover-0.png")).unwrap().to_rgb8();
    let changed = (70..600)
        .flat_map(|x| (580..850).map(move |y| (x, y)))
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
    app.choose("end-terror");
    let finished = app.observe();
    assert_eq!(finished["observation"]["revision"], 2);
    assert_eq!(finished["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(finished["observation"]["owner"], "Directorio");
    assert_eq!(
        finished["observation"]["next"]["options"],
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
fn native_transfer_modal_gives_asks_and_waits_for_the_giver() {
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
    // #71: the TM-B transfer modal. Expected balances are worked from the prepared
    // position (Government 6, 26July 0, Directorio 5, Syndicate 15) and R 1.5.1.
    app.choose("transfer");
    let opened = app.observe();
    let modal = |view: &Value| view["table"]["transfer"]["state"].clone();
    assert_eq!(modal(&opened)["shown"], true);
    assert_eq!(modal(&opened)["label"], "Choose a counterpart");
    app.capture("opened");
    // The standings stay visible above the scrim; the map and camera take no input.
    for standing in opened["table"]["standings"].as_array().unwrap() {
        assert_eq!(standing["visible"], true, "{standing}");
    }
    let revision = opened["observation"]["revision"].as_u64().unwrap();
    let camera = opened["table"]["camera"].clone();
    app.request(
        json!({"op":"point","revision":revision,"component":"space:santiago"}),
        false,
    );
    app.request(json!({"op":"key","key":"KeyW","revision":revision}), true);
    app.request(json!({"op":"settle"}), true);
    assert_eq!(app.observe()["table"]["camera"], camera, "camera moved");
    // Ask Syndicate: the slider's track spans its 15; a click at its middle sets 8.
    app.activate("transfer:tab:syndicate", revision);
    app.activate("transfer:direction:ask", revision);
    app.activate("transfer:slider", revision);
    assert_eq!(modal(&app.observe())["amount"], 8);
    app.activate("transfer:quick:2", revision);
    let asking = app.observe();
    assert_eq!(modal(&asking)["label"], "Send request: 2 from Syndicate");
    assert_eq!(modal(&asking)["limit"], 15);
    app.capture("ask-syndicate");
    app.choose("ask:3:2");
    let deciding = app.observe();
    assert_eq!(deciding["observation"]["owner"], "Syndicate");
    assert_eq!(deciding["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(modal(&deciding)["entry"], "2", "prefilled with the request");
    assert_eq!(modal(&deciding)["label"], "Syndicate authorizes 2");
    app.capture("giver-decision");
    // Wait: the modal closes and the request stays pending. That it was set aside is
    // saved (#75), so a restart shows the request exactly as it was left.
    let pending = deciding["observation"].clone();
    let saved = std::fs::read(&app.save).unwrap();
    app.choose("wait");
    let waiting = app.observe();
    assert_ne!(std::fs::read(&app.save).unwrap(), saved);
    assert_eq!(
        waiting["observation"]["transfer"]["Authorizing"]["waited"],
        true
    );
    assert_eq!(waiting["observation"]["resources"], pending["resources"]);
    assert_eq!(modal(&waiting)["shown"], false);
    assert_eq!(modal(&waiting)["pending"], true);
    app.capture("after-wait");
    // Each request reconnects; quitting and relaunching additionally crosses processes.
    app.quit();
    app.launch();
    let restarted = app.observe();
    assert_eq!(restarted["observation"], waiting["observation"]);
    assert_eq!(modal(&restarted)["shown"], false);
    assert_eq!(modal(&restarted)["pending"], true);
    app.capture("pending-restarted");
    // Open in the top band shows the giver's decision again, prefilled as asked.
    app.choose("reopen");
    let reopened = app.observe();
    assert_eq!(modal(&reopened)["shown"], true);
    assert_eq!(modal(&reopened)["pending"], false);
    assert_eq!(modal(&reopened)["entry"], "2");
    assert_eq!(
        reopened["observation"]["transfer"]["Authorizing"]["amount"],
        2
    );
    app.capture("giver-restarted");
    // The giver authorizes more than was asked.
    let revision = reopened["observation"]["revision"].as_u64().unwrap();
    app.choose("authorize:5");
    let after = app.observe()["observation"].clone();
    assert_eq!(after["resources"], json!([6, 5, 5, 10]));
    app.capture("accepted-balances");
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], after);
    app.request(
        json!({"op":"activate","revision":revision,"choice":format!("{revision}:authorize:5")}),
        false,
    );
    assert_eq!(
        app.observe()["observation"],
        after,
        "no duplicated transfer"
    );
    // A declined request moves nothing.
    app.choose("transfer");
    app.choose("ask:0:1");
    app.capture("decline-choice");
    app.choose("decline");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 5, 5, 10])
    );
    // 26July's own Resource transfer moves at once; more than it holds is refused in
    // the modal.
    app.choose("transfer");
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("transfer:tab:directorio", revision);
    app.activate("transfer:direction:give", revision);
    app.type_amount("9");
    let refused = app.observe();
    assert_eq!(
        modal(&refused)["refusal"],
        "9 is more than 5: 26July has 5 Resources."
    );
    assert_eq!(modal(&refused)["commit"], Value::Null);
    app.capture("give-refused");
    app.choose("give:2:1");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 4, 6, 10])
    );
    // Zero and repeated transfers.
    app.choose("transfer");
    app.choose("give:3:0");
    app.choose("transfer");
    app.choose("give:3:1");
    assert_eq!(
        app.observe()["observation"]["resources"],
        json!([6, 3, 6, 11])
    );
    // Ending Terror withdraws a pending request; nothing moves.
    app.choose("transfer");
    app.choose("ask:3:4");
    app.choose("wait");
    app.choose("end-terror");
    let ended = app.observe();
    assert_eq!(ended["observation"]["finished"], true);
    assert_eq!(ended["observation"]["transfer"], Value::Null);
    assert_eq!(ended["observation"]["resources"], json!([6, 3, 6, 11]));
    assert_eq!(modal(&ended)["shown"], false);
    app.capture("transfers-finished");
    app.quit();
}

/// #71: the TM-B modal at the owner's 16:9 aspect against its FigCraft layout record,
/// in the 16:9 frame's state (26July asks Syndicate for 2). 3840x2160 is the frame
/// (6144x3456) x 0.625. Plates are compared on all four edges; parts that hug text
/// (buttons, quick values, the toggle) on their top and bottom only, since Bevy's text
/// metrics differ from FigCraft's. The amount field is right-aligned beside the quick
/// values, so its left edge follows their text: its top, bottom and width are compared.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained capture"]
fn native_transfer_modal_matches_layout() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-71"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("transfer-layout-")
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
    eprintln!("Transfer layout evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app.choose("terror");
    app.choose("transfer");
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("transfer:tab:syndicate", revision);
    app.activate("transfer:direction:ask", revision);
    app.type_amount("2");
    let view = app.observe();
    app.capture("ask-syndicate-3840");
    let record: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-transfer.layout.json")).unwrap();
    let k = 3840. / 6144.;
    let parts = view["table"]["transfer"]["parts"].as_array().unwrap();
    let hugging = [
        "direction-toggle",
        "transfer:quick:1",
        "transfer:quick:2",
        "transfer:quick:max",
        "button:back",
        "button:primary",
    ];
    let mut report = vec![];
    for (name, bounds) in record["parts"].as_object().unwrap() {
        let b: Vec<f64> = bounds
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() * k)
            .collect();
        let want = [b[0], b[1], b[0] + b[2], b[1] + b[3]];
        let got = rect_of(
            &parts
                .iter()
                .find(|p| p["part"] == name.as_str())
                .unwrap_or_else(|| panic!("{name} not drawn"))["bounds"],
        );
        let edges: &[usize] = if hugging.contains(&name.as_str()) {
            &[1, 3]
        } else {
            &[0, 1, 2, 3]
        };
        let edges: &[usize] = if name == "number-field" {
            &[1, 3]
        } else {
            edges
        };
        let mut worst = edges
            .iter()
            .map(|&i| (got[i] - want[i]).abs())
            .fold(0., f64::max);
        if name == "number-field" {
            worst = worst.max(((got[2] - got[0]) - (want[2] - want[0])).abs());
        }
        report.push((worst, format!("{name}: {worst:.1} px {got:?} vs {want:?}")));
    }
    let text: Vec<_> = report.iter().map(|(_, line)| line.clone()).collect();
    std::fs::write(app.root.join("layout-deltas.txt"), text.join("\n")).unwrap();
    for (worst, line) in &report {
        assert!(*worst <= 4., "{line}");
    }
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
    // Resource transfers exercise the actual Terror path without randomness.
    for action in [
        "terror",
        "transfer",
        "ask:3:1",
        "authorize:1",
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
    app.choose("ask:2:2");
    app.choose("authorize:2");
    app.choose("terror:santiago");
    app.choose("terror:oriente");
    let start = app.observe();
    assert_eq!(start["table"]["settled"], true);
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
    // The authored loop is 3.6 s: 216 steps of 16,666,667 ns. Advance 2 steps at a
    // time (and an odd step alone), so each request shows the loop never holds the
    // table.
    let step = |app: &mut Native, steps: usize| {
        for _ in 0..steps / 2 {
            app.advance(2);
        }
        if steps % 2 == 1 {
            app.advance(1);
        }
    };
    step(&mut app, 216);
    app.capture("loop-whole");
    let whole = seeks(&app.observe());
    for (a, b) in initial.iter().zip(&whole) {
        assert!(
            (a - b).abs() < 1e-4,
            "one loop returns each bell: {initial:?} {whole:?}"
        );
    }
    for at in bells {
        let seam = token_changes(
            &app.root.join("loop-start.png"),
            &app.root.join("loop-whole.png"),
            at,
        );
        assert_eq!(seam, 0, "one whole loop renders the bell identically");
    }
    // The swing, independent of the run's phase: the bell swings four times per loop
    // (art/atlas/TERROR.md), a 0.9 s swing. Each bell in turn is driven through the
    // manual clock, with `advance`, to the fixed seek times 0, 0.225, 0.45 and 0.675 s,
    // a quarter swing apart (to within half a step, 1/120 s). Whatever the clip's own
    // phase, two of those four poses lie at least √2 times the amplitude (34–50°) apart.
    let mut reached = vec![];
    for (index, (at, name)) in bells.into_iter().zip(["santiago", "oriente"]).enumerate() {
        // `seeks` lists Oriente then Santiago; `bells` lists Santiago then Oriente.
        let bell = 1 - index;
        let mut shots = vec![];
        for target in [0., 0.225, 0.45, 0.675] {
            let now = seeks(&app.observe())[bell];
            let steps = ((target - now).rem_euclid(3.6) * 60.).round() as usize % 216;
            step(&mut app, steps);
            let seek = seeks(&app.observe())[bell];
            let off = (seek - target).rem_euclid(3.6);
            assert!(
                off.min(3.6 - off) <= 1. / 120. + 1e-3,
                "{name} bell driven to {target} s, at {seek} s"
            );
            let shot = format!("swing-{name}-{target}");
            app.capture(&shot);
            reached.push(format!("{name} {target} s: seek {seek:.4} s"));
            shots.push(app.root.join(format!("{shot}.png")));
        }
        let mut swing = vec![];
        for a in 0..shots.len() {
            for b in a + 1..shots.len() {
                swing.push(token_changes(&shots[a], &shots[b], at));
            }
        }
        assert!(
            swing.iter().any(|n| *n > 20),
            "the {name} bell visibly swings: {swing:?} changed pixels"
        );
        reached.push(format!("{name} changed pixels by pair: {swing:?}"));
    }
    std::fs::write(app.root.join("swing-seeks.txt"), reached.join("\n")).unwrap();
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
    app.choose("ask:2:2");
    app.choose("authorize:2");
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
        // A step's tag is a preview: Kidnap's names Government by emblem after its
        // first mention, as a briefing does (EX-A, #84).
        let revision = initial["observation"]["revision"].as_u64().unwrap();
        app.activate("step:kidnap", revision);
        let tagged = app.observe();
        checked += assert_inline_emblems(&tagged);
        assert_eq!(
            emblems_in(
                &tagged,
                "Government in Santiago de Cuba, which has 6 Resources"
            ),
            ["Government"]
        );
        app.capture("kidnap-tag");
        app.activate("step:kidnap", revision);
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

        // The transfer modal (TM-B, #71): its running text and buttons name factions by
        // emblem ("Ask [D] for Resources", story 60), while its headings, labels and
        // account names keep them, and so does 26July on the red primary button.
        // Observations keep every name.
        app.choose("transfer");
        let directions = app.observe();
        checked += assert_inline_emblems(&directions);
        assert_eq!(
            emblems_in(&directions, "Open while 26July executes Terror  ·  R 1.5.1"),
            ["26July"]
        );
        let revision = directions["observation"]["revision"].as_u64().unwrap();
        app.activate("transfer:tab:syndicate", revision);
        app.activate("transfer:direction:ask", revision);
        app.activate("transfer:quick:1", revision);
        let asking = app.observe();
        checked += assert_inline_emblems(&asking);
        for (words, drawn) in [
            ("Send request: 1 from Syndicate", vec!["Syndicate"]),
            (
                "Syndicate would give;\nSyndicate decides the amount",
                vec!["Syndicate", "Syndicate"],
            ),
            (
                "Nothing moves until Syndicate authorizes.",
                vec!["Syndicate"],
            ),
        ] {
            assert_eq!(emblems_in(&asking, words), drawn, "{words}");
        }
        assert!(
            asking["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |t| t["text"].as_str().unwrap().contains("Syndicate's balance")
                        && t["emblems"].as_array().unwrap().len() == 1
                ),
            "the binding limit names Syndicate by emblem"
        );
        // Account names and headings keep the names.
        for label in ["26July", "Syndicate", "COUNTERPART"] {
            assert!(
                asking["table"]["inline_text"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|t| t["text"] != label),
                "label {label} drawn as an emblem"
            );
        }
        app.capture("transfer-ask-syndicate");
        assert!(
            directions["observation"]["choices"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["label"] == "Ask Syndicate for 1"),
            "observation keeps names"
        );
        assert!(
            directions["table"]["transfer"]["parts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["text"] == "RESOURCE TRANSFER")
        );
        app.capture("transfer-directions");

        // The panel beneath names the giver by emblem. In Syndicate's decision the green
        // header note draws 26July's emblem, and the green primary button keeps
        // Syndicate's name, whose green emblem would vanish on it (#60's rule).
        app.choose("ask:3:1");
        let authorizing = app.observe();
        checked += assert_inline_emblems(&authorizing);
        assert_eq!(
            emblems_in(&authorizing, "Transfer request from 26July  ·  R 1.5.1"),
            ["26July"]
        );
        assert!(
            authorizing["table"]["inline_text"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| t["text"] != "Syndicate authorizes 1"),
            "Syndicate drawn as an emblem on its own green"
        );
        assert!(
            authorizing["table"]["transfer"]["parts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["part"] == "button:primary")
        );
        // The panel's transfer step beneath the modal names the giver by emblem.
        assert_eq!(
            emblems_in(&authorizing, "pending · Syndicate decides"),
            ["Syndicate"]
        );
        app.capture("transfer-authorization");

        // The session notice: "[S] gave 1 to [26]. Saved." keeps its full stop.
        app.choose("authorize:1");
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
                .any(|t| t["text"].as_str().unwrap().contains("Government pays")
                    && t["emblems"].as_array().unwrap().len() >= 2)
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
                    .starts_with("Directorio, 1st Eligible, may:"))
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
        passed["observation"]["next"]["options"],
        json!([
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)",
            "Event"
        ])
    );
    assert_eq!(ledger(&passed), ["6", "1", "5", "15"]);
    assert!(named(&passed, "step:transfer").is_none());
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
    assert!(named(&terror, "step:transfer").is_some());
    assert!(named(&terror, "step:back-to-choice").is_some());
    app.capture("terror-chosen");
    for action in ["transfer", "ask:3:1"] {
        app.choose(action);
    }
    let pending = app.observe();
    assert_eq!(pending["observation"]["owner"], "Syndicate");
    // The giver's decision covers the panel; after Wait, 26July's withdrawals show.
    app.choose("wait");
    let waited = app.observe();
    app.capture("pending-request-with-withdrawal");
    app.quit();
    app.launch();
    // The request comes back as it was left, set aside (#75).
    let restarted = app.observe();
    assert_eq!(restarted["observation"], waited["observation"]);
    assert_eq!(restarted["table"]["transfer"]["state"]["shown"], false);
    assert_eq!(restarted["table"]["transfer"]["state"]["pending"], true);
    assert_eq!(
        restarted["observation"]["resources"],
        pending["observation"]["resources"]
    );
    app.choose("back-to-choice");
    let back = app.observe();
    assert_eq!(back["observation"]["action"], "Open");
    assert_eq!(back["observation"]["transfer"], Value::Null);
    assert_eq!(back["observation"]["resources"], json!([6, 0, 5, 15]));
    // Terror, transfer, ask, Wait and back: five decisions from revision 0.
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
    // Rests the pointer on a step's row and returns the tag it opens (EX-A, #84).
    let card = |app: &mut Native, step: &str, capture: &str| -> String { app.tag(step, capture) };
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
        .map(|c| {
            format!(
                "step:{}",
                c["id"].as_str().unwrap().split_once(':').unwrap().1
            )
        })
        .collect();
    // Every listed step, the disabled targets included, is a real control.
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
        santiago.contains("IN THIS TASK: Kidnap here first · Request a Resource transfer"),
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
    let refusal = app.refuse("terror:santiago");
    assert!(refusal.contains("Kidnap here first"), "{refusal}");
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    for step in ["transfer", "ask:2:1", "authorize:1"] {
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
    assert!(only.contains("RECORDS: Operation Only"), "{only}");
    assert!(!only.contains("No Terror resolved"), "{only}");
    app.choose("kidnap");
    app.roll();
    let special = card(&mut app, "end-terror", "end-terror-after-kidnap");
    assert!(
        special.contains("RECORDS: Operation with Special Activity"),
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
/// faces, from #8's seeds whose rolls are known (`tests/integration/kidnap.rs`), the tray in the
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
    // 16:9, like the owner's 6144x3456 display that the AC-B die-presented frame lays
    // out (frame C's tray composed into the AC-B panel).
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
        // While the die is presented nothing on the map is targeted or marked, no
        // popover is open and the transfer modal is closed (#72, #73, #71).
        assert!(first["table"]["map_targets"].is_null(), "{first}");
        assert!(first["table"]["removal"].is_null(), "{first}");
        assert!(first["table"]["popover"].is_null(), "{first}");
        assert_eq!(first["table"]["transfer"]["state"]["shown"], false);
        let tray = rect_of(&shown["viewport"]);
        if die == 1 {
            // The tray matches the die-presented frame on all four edges, within 2 px.
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
        // The landed top face keeps #67's legibility floor, "about 70 px" at 1920x1080,
        // made explicit as 69.5 px (`die::FACE_FLOOR`); that is 139 px here (the layout
        // scale is twice that of 1920x1080).
        let face_px = shown["top_face"].as_f64().unwrap();
        eprintln!("face {die}: landed top face {face_px:.1} px at 3840x2160");
        assert!(face_px >= 139., "landed face {face_px} px at 3840x2160");
        let part = |name: &str| -> [f64; 4] {
            rect_of(
                &landed["table"]["choice_parts"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["part"] == name)
                    .unwrap_or_else(|| panic!("no part {name}"))["bounds"],
            )
        };
        let apart =
            |a: [f64; 4], b: [f64; 4]| a[2] <= b[0] || b[2] <= a[0] || a[3] <= b[1] || b[3] <= a[1];
        let ribbon = part("briefing-ribbon");
        let next = part("kidnap-die:continue");
        assert!(
            apart(ribbon, tray),
            "ribbon {ribbon:?} overlaps tray {tray:?}"
        );
        assert!(
            apart(next, tray),
            "Continue {next:?} overlaps tray {tray:?}"
        );
        for t in shown["text"].as_array().unwrap() {
            let bounds = rect_of(&t["bounds"]);
            assert!(apart(bounds, ribbon), "{t} overlaps the ribbon");
            if t["part"] != "panel title" {
                assert!(
                    bounds[1] >= tray[3] - 0.5 && bounds[3] <= next[1] + 0.5,
                    "{t}"
                );
            }
        }
        if die == 1 {
            // Continue and the ribbon match the frame on all four edges, within 2 px.
            for (name, native) in [("kidnap-die:continue", next), ("briefing-ribbon", ribbon)] {
                let expected = scaled(&layout["parts"][name]);
                for (n, e) in native.iter().zip(expected) {
                    assert!(
                        (n - e).abs() <= 2.,
                        "{name}: native {native:?} differs from layout {expected:?}"
                    );
                }
            }
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
            let picked = app.observe();
            assert_eq!(picked["observation"], landed["observation"]);
            assert_eq!(picked["table"]["die"]["landed"], true);
            assert!(picked["table"]["popover"].is_null(), "{picked}");
        }
        if die == 4 {
            // The BRIEFING ribbon still opens the briefing tag beside the panel, and
            // closes it again; the die stays presented and nothing is decided.
            let revision = landed["observation"]["revision"].as_u64().unwrap();
            app.activate("briefing-ribbon", revision);
            let tag = app.observe();
            assert_eq!(tag["table"]["choice"]["ribbon"], true, "{tag}");
            assert_eq!(tag["table"]["die"]["landed"], true);
            assert_eq!(tag["observation"], landed["observation"]);
            app.capture(&format!("face-{die}-briefing"));
            app.activate("briefing-ribbon", revision);
            assert_eq!(app.observe()["table"]["choice"]["ribbon"], false);
        }
        // Continue returns the panel to Terror: by Enter on the focused control once,
        // otherwise by its button.
        let revision = landed["observation"]["revision"].as_u64().unwrap();
        if die == 1 {
            app.request(
                json!({"op":"key","revision":revision,"key":"Enter","focused":"kidnap-die:continue"}),
                true,
            );
            app.request(json!({"op":"settle"}), true);
        } else {
            app.activate("kidnap-die:continue", revision);
        }
        let continued = app.observe();
        assert!(continued["table"]["die"].is_null(), "{continued}");
        app.capture(&format!("face-{die}-continued"));
        assert_eq!(continued["observation"], landed["observation"]);
        assert_eq!(std::fs::read(&app.save).unwrap(), committed);
        app.choose("terror:santiago");
        assert_eq!(app.observe()["observation"]["owed_terror"], Value::Null);
        app.quit();
    }
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
    // Hovering the blocked space shows its status tag and opens no popover (#58: a
    // click opens it).
    let hover = app.point("space:oriente");
    assert_eq!(hover["table"]["popover"], Value::Null, "{hover}");
    assert_eq!(hover["table"]["status"]["space"], "oriente");
    app.capture("oriente-blocked-hover-zero-resources");
    // Clicking it says what is missing and what would unblock it.
    let saved = std::fs::read(&app.save).unwrap();
    app.activate("space:oriente", 1);
    let blocked = app.observe();
    assert_eq!(blocked["table"]["popover"]["space"], "oriente", "{blocked}");
    assert_eq!(blocked["table"]["popover"]["pinned"], true);
    let terror = popover_row(&blocked, "Terror");
    assert_eq!(terror["mark"], "not-yet");
    assert_eq!(terror["remedies"], json!(["Request a Resource transfer"]));
    assert_eq!(
        terror["lines"][0][1],
        "Cost: 1 Resource per space (26July has 0). R 3.3.4"
    );
    assert_popover_agrees(&blocked);
    assert_popover_placed(&blocked);
    assert_eq!(blocked["observation"]["revision"], 1);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("oriente-blocked-popover-zero-resources");
    app.activate("space:oriente", 1);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    let away = app.point("standing:0");
    assert_eq!(away["table"]["popover"], Value::Null);

    // Clicking Santiago opens its popover: Kidnap chosen, its stakes previewed
    // before the only acting button; Terror not yet, with both remedies.
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
    // The open Kidnap step's tag is pinned over the map's east (EX-A, #84); Escape
    // closes it, as a player would, before Santiago is clicked.
    assert_eq!(opened["table"]["choice"]["pinned"], "kidnap");
    app.request(json!({"op":"key","revision":2,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    inert(&mut app, "santiago");
    app.choose("kidnap:cancel");

    // After a Resource transfer both targets are selectable; Terror from the map
    // popover.
    for step in ["transfer", "ask:3:1", "authorize:1"] {
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
    let revision = after["observation"]["revision"].as_u64().unwrap();
    let hover = app.point("space:oriente");
    assert_eq!(hover["table"]["popover"], Value::Null);
    app.activate("space:oriente", revision);
    let blocked = app.observe();
    assert_eq!(blocked["table"]["popover"]["space"], "oriente");
    assert_popover_agrees(&blocked);
    app.capture("oriente-blocked-hover-after-donation");
    app.activate("space:oriente", revision);
    // A resolved Terror stays in Santiago's popover as history.
    app.activate("space:santiago", revision);
    let history = app.observe();
    assert_eq!(popover_row(&history, "Terror")["mark"], "done");
    app.capture("santiago-terror-done");
    app.activate("space:santiago", revision);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);

    // Terror from the panel after another Resource transfer.
    for step in ["transfer", "ask:3:1", "authorize:1"] {
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

/// The space popover against its layout record (`art/atlas/atlas-popover.layout.json`,
/// the HF-A master A-P1): Santiago at revision 1, Kidnap chosen and Terror not yet, at
/// 3840x2160 (the reference x 1.6875), within the 4 px of the transfer modal's check.
/// Parts that hold wrapped text are compared on the edges the text does not move,
/// because Bevy's Lato metrics differ from FigCraft's: the card and the chosen row on
/// left, top and right; the button on left, right and height; the second divider on
/// left, right and height; the other row and the footer on left and right.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; inspect retained screenshots separately"]
fn native_space_popover_matches_layout() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/review-fix-2"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("popover-layout-")
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
    eprintln!("Popover layout evidence: {}", app.root.display());
    app.launch_size("3840x2160");
    app.choose("terror");
    app.activate("space:santiago", 1);
    let view = app.observe();
    app.capture("santiago-popover-3840");
    let popover = &view["table"]["popover"];
    assert_eq!(popover["pinned"], true, "{popover}");
    let record: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-popover.layout.json")).unwrap();
    let s = (3840f64 / 1920.).min(2160. / 1280.);
    let parts = popover["parts"].as_array().unwrap();
    let drawn = |name: &str| {
        let found: Vec<_> = parts.iter().filter(|p| p["part"] == name).collect();
        assert_eq!(found.len(), 1, "one {name}: {parts:?}");
        rect_of(&found[0]["bounds"])
    };
    let card = drawn("popover-card");
    let mut report = vec![];
    for (name, bounds) in record["parts"].as_object().unwrap() {
        let b: Vec<f64> = bounds
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() * s)
            .collect();
        let want = [
            card[0] + b[0],
            card[1] + b[1],
            card[0] + b[0] + b[2],
            card[1] + b[1] + b[3],
        ];
        let got = drawn(name);
        let edges: &[usize] = match name.as_str() {
            "popover-card" | "popover-row-chosen" => &[0, 1, 2],
            "popover-row" | "popover-footer" => &[0, 2],
            "popover-commit" | "popover-divider:1" => &[0, 2],
            _ => &[0, 1, 2, 3],
        };
        let mut worst = edges
            .iter()
            .map(|&i| (got[i] - want[i]).abs())
            .fold(0., f64::max);
        if matches!(name.as_str(), "popover-commit" | "popover-divider:1") {
            worst = worst.max(((got[3] - got[1]) - (want[3] - want[1])).abs());
        }
        report.push((worst, format!("{name}: {worst:.1} px {got:?} vs {want:?}")));
    }
    let text: Vec<_> = report.iter().map(|(_, line)| line.clone()).collect();
    std::fs::write(app.root.join("layout-deltas.txt"), text.join("\n")).unwrap();
    for (worst, line) in &report {
        assert!(*worst <= 4., "{line}");
    }
    app.quit();
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
    // Tab passes the execution panel's steps and commit before the ribbon (#84).
    let mut focus = Value::Null;
    for _ in 0..12 {
        focus = key(&mut app, "Tab", None)["table"]["choice"]["focus"].clone();
        if focus == "briefing-ribbon" {
            break;
        }
    }
    assert_eq!(focus, "briefing-ribbon");
    let opened = key(&mut app, "Enter", Some("briefing-ribbon"));
    assert_eq!(opened["table"]["briefing"]["option"], "terror:kidnap");
    key(&mut app, "Tab", None);
    let on_tab = key(&mut app, "Tab", None);
    assert_eq!(on_tab["table"]["choice"]["focus"], "briefing-tab:terror");
    let terror_tab = key(&mut app, "Enter", Some("briefing-tab:terror"));
    assert_eq!(terror_tab["table"]["briefing"]["option"], "terror");

    // Agreement after a transfer: Government gives 2 (6 → 4, 26July 0 → 2).
    // Pinning a step takes the tag's place from the ribbon's (EX-A, #84), so the
    // ribbon reopens it afterwards.
    for action in ["transfer", "ask:0:2", "authorize:2"] {
        app.choose(action);
    }
    let rev = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate("briefing-ribbon", rev);
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
    if after["table"]["choice"]["ribbon"] != true {
        app.activate(
            "briefing-ribbon",
            after["observation"]["revision"].as_u64().unwrap(),
        );
    }
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
    assert!(reason.starts_with("1 Underground Guerrilla vs 1 Police; must outnumber."));
    assert!(
        words.contains(
            "RESULT: Unavailable now. 1 Underground Guerrilla vs 1 Police; must outnumber."
        ),
        "{words}"
    );
    assert!(!words.contains("Santiago"), "{words}");
    app.capture("after-removal-kidnap");
    app.quit();
}
/// #84: the EX-A execution panel against its FigCraft layout record at 3840×2160,
/// within the 2 px the other layout comparisons use, and every step's tag before its
/// committing click: Terror targets from the panel, Kidnap opened (Roll and Cancel),
/// End Terror's previews, the clash rule with a space popover, Escape and an outside
/// click, the keyboard, and the ended panel.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect retained execution-panel captures"]
fn native_execution_panel_matches_layout_and_previews_every_step() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-84"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("ex-a-")
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
        save: root.join("ex-a.json"),
        root,
        step: 0,
    };
    eprintln!("EX-A evidence: {}", app.root.display());
    let layout: Value =
        serde_json::from_str(include_str!("../art/atlas/atlas-execution.layout.json")).unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    let parts =
        |view: &Value| -> Vec<Value> { view["table"]["choice_parts"].as_array().unwrap().clone() };
    let part = |view: &Value, name: &str| -> Value {
        parts(view)
            .into_iter()
            .find(|p| p["part"] == name)
            .unwrap_or_else(|| panic!("no native part {name}: {}", view["table"]["choice_parts"]))
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
            let n = rect_of(&part(view, name)["bounds"]);
            let native = [n[0], n[1], n[2], n[3], (n[0] + n[2]) / 2.];
            for &edge in edges {
                assert!(
                    (native[edge] - expected[edge]).abs() <= 2.,
                    "{name} edge {edge}: native {n:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    };
    let words = |view: &Value, name: &str| -> String {
        part(view, name)["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
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
    let rev = |view: &Value| view["observation"]["revision"].as_u64().unwrap();
    app.launch_size("3840x2160");
    app.choose("terror");

    // Terror chosen at 0 Resources (EX-A state 1): every step is listed and named for
    // the seat; both targets are blocked with remedies; nothing is pinned.
    let chosen = app.observe();
    for step in [
        "terror:santiago",
        "terror:oriente",
        "kidnap",
        "transfer",
        "remove",
        "end-terror",
        "back-to-choice",
    ] {
        assert!(
            chosen["table"]["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["name"] == format!("step:{step}")),
            "step:{step}"
        );
    }
    assert_eq!(
        words(&chosen, "step:terror:santiago-pill-label"),
        "NOT YET · 26JULY HAS 0"
    );
    assert_eq!(
        words(&chosen, "execution-commit-label"),
        "Click a step to choose it"
    );
    assert_eq!(chosen["table"]["briefing"], Value::Null);
    app.capture("terror-chosen");
    let blocked = app.tag("terror:oriente", "oriente-blocked-tag");
    assert!(
        blocked.contains("NOT YET: Cost: 1 Resource per space (26July has 0)."),
        "{blocked}"
    );
    assert!(
        blocked.contains("IN THIS TASK: Request a Resource transfer"),
        "{blocked}"
    );

    // Funded by a Syndicate transfer, Oriente's Terror from the panel: its tag shows
    // the authority's preview before the commit (Oriente Pop 2, Passive → Active
    // Opposition: standing 7 → 9, margin −8 → −6).
    for action in ["transfer", "ask:3:1", "authorize:1"] {
        app.choose(action);
    }
    let oriente = app.tag("terror:oriente", "oriente-preview");
    for fact in [
        "COST: 26July Resources: 1 → 0",
        "Oriente: Passive Opposition → Active Opposition",
        "26July victory standing 7 → 9 (margin -8 → -6)",
    ] {
        assert!(oriente.contains(fact), "{fact}: {oriente}");
    }
    app.choose("terror:oriente");
    let paid = app.observe();
    assert_eq!(paid["observation"]["resources"], json!([6, 0, 5, 14]));
    assert_eq!(
        words(&paid, "step:terror:oriente-pill-label"),
        "DONE · PAID 1"
    );

    // Kidnap opened (EX-A state 3): its stakes stay pinned on the held step; Cancel
    // and Roll take the foot; the steps that wait are dimmed.
    app.choose("kidnap");
    let opened = app.observe();
    assert_eq!(opened["table"]["choice"]["pinned"], "kidnap");
    assert_eq!(opened["table"]["briefing"]["option"], "kidnap");
    assert_eq!(
        words(&opened, "step:kidnap-pill-label"),
        "OPEN · NOTHING ROLLED"
    );
    compare(&opened, &layout["kidnap"]["images"], &[0, 1, 2, 3]);
    compare(&opened, &layout["kidnap"]["text"], &[0, 1]);
    compare(&opened, &layout["kidnap"]["right"], &[1, 2, 3]);
    compare(&opened, &layout["kidnap"]["centered_text"], &[1, 4]);
    app.capture("kidnap-opened");
    // Escape closes the tag; its row reopens it.
    let closed = key(&mut app, "Escape", None);
    assert_eq!(closed["table"]["briefing"], Value::Null);
    app.activate("step:kidnap", rev(&closed));
    assert_eq!(app.observe()["table"]["briefing"]["option"], "kidnap");
    app.roll();

    // Santiago owed and pinned: the layout's 16:9 frame (revision 14 of #75's run).
    let rolled = app.observe();
    let die = rolled["observation"]["kidnap"]["die"].as_u64().unwrap();
    assert_eq!(
        words(&rolled, "step:kidnap-pill-label"),
        format!("ROLLED {die}")
    );
    assert_eq!(
        words(&rolled, "step:back-to-choice-pill-label"),
        "NO LONGER OFFERED"
    );
    assert_eq!(
        words(&rolled, "step:end-terror-pill-label"),
        "NOT YET · SANTIAGO OWED"
    );
    let refusal = app.refuse("end-terror");
    assert!(refusal.contains("Resolve Santiago's Terror"), "{refusal}");
    app.activate("step:terror:santiago", rev(&rolled));
    let pinned = app.observe();
    assert_eq!(pinned["table"]["briefing"]["option"], "terror:santiago");
    assert_eq!(pinned["table"]["briefing"]["pinned"], true);
    assert_eq!(
        words(&pinned, "execution-commit-label"),
        "Terror in Santiago de Cuba · pay 1 Resource"
    );
    assert_eq!(
        words(&pinned, "step:terror:santiago-pill-label"),
        "OWED · 1 RESOURCE"
    );
    let santiago = tag_text(&pinned["table"]["briefing"]);
    let cost = format!("COST: 26July Resources: {die} → {}", die - 1);
    for fact in [
        cost.as_str(),
        "Santiago de Cuba: Neutral → Passive Opposition",
        "26July victory standing 9 → 10 (margin -6 → -5)",
        "OWED: Santiago Terror owed",
    ] {
        assert!(santiago.contains(fact), "{fact}: {santiago}");
    }
    compare(&pinned, &layout["parts"], &[0, 1, 2, 3]);
    compare(&pinned, &layout["pills"], &[1, 2, 3]);
    compare(&pinned, &layout["stretched"], &[0, 1, 2]);
    compare(&pinned, &layout["text"], &[0, 1]);
    compare(&pinned, &layout["right_text"], &[1, 2]);
    compare(&pinned, &layout["centered_text"], &[1, 4]);
    app.capture("santiago-pinned-16x9");

    // The clash rule: a space popover wins over the step's tag, and the tag reopens
    // from its row, which closes the popover.
    app.activate("space:santiago", rev(&pinned));
    let popover = app.observe();
    assert_eq!(
        popover["table"]["popover"]["space"], "santiago",
        "{}",
        popover["table"]
    );
    assert_eq!(popover["table"]["briefing"], Value::Null);
    assert_eq!(popover["table"]["choice"]["pinned"], Value::Null);
    app.capture("popover-wins");
    app.activate("step:terror:santiago", rev(&pinned));
    let reopened = app.observe();
    assert_eq!(reopened["table"]["popover"], Value::Null);
    assert_eq!(reopened["table"]["briefing"]["option"], "terror:santiago");
    // A click on the map outside the tag closes it.
    app.activate("space:havana", rev(&pinned));
    let outside = app.observe();
    assert_eq!(outside["table"]["briefing"], Value::Null);
    assert_eq!(outside["observation"], pinned["observation"]);

    // The keyboard: Tab focuses the first step and shows its tag; Enter pins it; Tab
    // reaches the commit, which Enter presses.
    let focused = key(&mut app, "Tab", None);
    assert_eq!(focused["table"]["choice"]["focus"], "step:terror:santiago");
    assert_eq!(focused["table"]["briefing"]["option"], "terror:santiago");
    let pinned_key = key(&mut app, "Enter", Some("step:terror:santiago"));
    assert_eq!(pinned_key["table"]["choice"]["pinned"], "terror:santiago");
    let commit = format!("{}:terror:santiago", rev(&pinned_key));
    let mut view = pinned_key;
    for _ in 0..10 {
        if view["table"]["choice"]["focus"] == commit.as_str() {
            break;
        }
        view = key(&mut app, "Tab", None);
    }
    assert_eq!(view["table"]["choice"]["focus"], commit.as_str());
    let paid = key(&mut app, "Enter", Some(&commit));
    assert_eq!(paid["observation"]["owed_terror"], Value::Null);
    assert_eq!(paid["observation"]["resources"][1], die - 1);
    assert_eq!(
        words(&paid, "step:terror:santiago-pill-label"),
        "DONE · PAID 1"
    );

    // End Terror after Kidnap (EX-A state 7), then the ended panel.
    let end = app.tag("end-terror", "end-terror-after-kidnap");
    for fact in [
        "RECORDS: Operation with Special Activity (R 2.3.4)",
        "EVENT FOR DIRECTORIO",
        "Ineligible after this card",
    ] {
        assert!(end.contains(fact), "{fact}: {end}");
    }
    app.choose("end-terror");
    let ended = app.observe();
    assert_eq!(ended["observation"]["finished"], true);
    assert_eq!(words(&ended, "execution-heading"), "26July · Terror ended");
    compare(&ended, &layout["ended"]["images"], &[0, 1, 2, 3]);
    compare(&ended, &layout["ended"]["text"], &[0, 1]);
    assert!(
        words(&ended, "ended-value:RESULT").contains(&format!("Kidnap rolled {die}")),
        "{}",
        words(&ended, "ended-value:RESULT")
    );
    app.capture("ended");
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
    // Escape closes an open space popover as it closes the tag (#75), deciding nothing.
    app.request(json!({"op":"key","revision":rev,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    let escaped = app.observe();
    assert_eq!(escaped["table"]["popover"], Value::Null);
    assert_eq!(escaped["observation"], popover["observation"]);
    assert_eq!(std::fs::read(&app.save).unwrap(), saved);
    app.capture("clash-escape-closed-popover");
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

    // Hovering a removable piece opens no popover (#58 story 49); a click opens it.
    let hovered = app.point(&format!("piece:{}", piece("santiago:underground")));
    assert_eq!(hovered["table"]["popover"], Value::Null, "{hovered}");
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
    // Escape closes a piece popover as Cancel does (#75), deciding nothing.
    app.activate(
        &format!("piece:{}", piece("santiago:underground")),
        revision,
    );
    assert_eq!(app.observe()["table"]["popover"]["pinned"], true);
    app.request(json!({"op":"key","revision":revision,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
    let escaped = app.observe();
    assert_eq!(escaped["table"]["popover"], Value::Null);
    assert_eq!(escaped["observation"], mode["observation"]);
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
    for action in ["terror", "kidnap"] {
        app.choose(action);
    }
    app.roll();
    app.choose("remove");
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
        // The roll's die is presented, then closed with Continue (#74).
        if action == "kidnap:roll" {
            app.roll();
        } else {
            app.choose(action);
        }
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

/// #58 review fix 3 and final sweep: an early Kidnap owes Santiago's Terror, then a
/// Transfer request waits on Directorio. End Terror would withdraw the request, but
/// the authority refuses it, so the row, its tag, the commit and the request's tag all
/// say not yet with the authority's reason and its remedy, timed "once Directorio has
/// decided", and pressing the commit submits nothing. Then the remedy is taken, and
/// the Terror and End Terror notices, both ended panels and the Pass notice state the
/// authority's record.
#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained captures"]
fn native_refused_end_terror_while_a_request_waits_and_the_ended_panels() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/review-fix-3"));
    std::fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("refused-end-")
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
        save: root.join("refused-end.json"),
        root: root.clone(),
        step: 0,
    };
    eprintln!("Review fix 3 evidence: {}", app.root.display());
    let words = |view: &Value, name: &str| -> String {
        view["table"]["choice_parts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["part"] == name)
            .and_then(|p| p["text"].as_str())
            .unwrap_or_default()
            .to_owned()
    };
    app.launch_size("3840x2160");
    app.choose("terror");
    app.choose("kidnap");
    app.roll();
    app.choose("transfer");
    app.choose("ask:2:1");
    app.choose("wait");
    let pending = app.observe();
    assert_eq!(pending["observation"]["owed_terror"], "santiago");
    let end = pending["observation"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str().unwrap().ends_with(":end-terror"))
        .unwrap()
        .clone();
    assert_eq!(end["supported"], false, "{end}");
    assert_eq!(
        words(&pending, "step:end-terror-pill-label"),
        "NOT YET · SANTIAGO OWED"
    );
    let remedy = "Resolve Santiago's Terror once Directorio has decided";
    let tag = app.tag("end-terror", "refused-end-terror-tag");
    assert!(tag.contains("Santiago Terror is owed"), "{tag}");
    assert!(tag.contains(remedy), "{tag}");
    let revision = pending["observation"]["revision"].as_u64().unwrap();
    let refusal = app.refuse("end-terror");
    assert!(refusal.contains("Santiago Terror is owed"), "{refusal}");
    assert!(
        refusal.contains(&format!("In this task: {remedy}.")),
        "{refusal}"
    );
    let refused = app.observe();
    assert_eq!(
        refused["observation"]["revision"], revision,
        "nothing submitted"
    );
    assert!(refused["observation"]["transfer"].is_object());
    assert_eq!(
        words(&refused, "execution-commit-label"),
        "End Terror is not available yet"
    );
    app.capture("refused-end-terror-pinned");
    app.activate("step:end-terror", revision);
    let request = app.tag("transfer", "pending-request-tag");
    assert!(!request.contains("may withdraw"), "{request}");
    assert!(
        request.contains(&format!("In this task: {remedy}.")),
        "{request}"
    );
    // The remedy: Directorio decides, then Santiago's Terror; End Terror then records
    // the Operation with its Special Activity.
    app.choose("reopen");
    app.choose("decline");
    app.choose("terror:santiago");
    let resolved = app.observe();
    assert_eq!(
        resolved["table"]["notice"],
        "Terror saved in Santiago de Cuba: paid 1 Resource, flipped 1 Underground Guerrilla to Active, placed a Terror marker and shifted Neutral → Passive Opposition."
    );
    app.capture("terror-notice");
    app.choose("end-terror");
    let ended = app.observe();
    assert_eq!(
        ended["table"]["notice"],
        "Terror ended: Operation with Special Activity recorded. Saved."
    );
    let next = &ended["observation"]["next"];
    assert_eq!(
        (&next["faction"], &next["role"]),
        (&json!("Directorio"), &json!("2nd Eligible"))
    );
    assert_eq!(
        ended["observation"]["eligibility"]["m26_box"],
        "OperationWithSpecialActivity"
    );
    assert_eq!(ended["observation"]["eligibility"]["m26_next_card"], false);
    assert_eq!(
        words(&ended, "execution-subtitle").lines().next(),
        Some("Operation with Special Activity recorded (R 2.3.4)")
    );
    assert_eq!(
        words(&ended, "ended-value:ELIGIBILITY"),
        "26July becomes Ineligible after this card (R 2.3.7)."
    );
    assert!(
        words(&ended, "ended-value:NEXT · SHOWN, NOT EXECUTED")
            .starts_with("Directorio, 2nd Eligible, may:")
    );
    app.capture("ended-special-activity");
    app.quit();

    // A Pass: the authority's recorded Resources and Eligibility.
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        socket: socket_dir.path().join("seat-pass.sock"),
        save: root.join("pass.json"),
        root,
        step: 100,
    };
    app.launch_size("3840x2160");
    app.choose("pass");
    let passed = app.observe();
    assert_eq!(
        passed["table"]["notice"],
        "26July passed: +1 Resource and stays Eligible. Directorio is 1st Eligible. Saved."
    );
    let change = &passed["observation"]["recorded_resources"][0];
    assert_eq!(
        words(&passed, "ended-value:RESULT").lines().next(),
        Some(format!("26July Resources: {} → {}", change["from"], change["to"]).as_str())
    );
    assert_eq!(passed["observation"]["eligibility"]["m26_next_card"], true);
    assert_eq!(
        words(&passed, "ended-value:ELIGIBILITY"),
        "26July stays Eligible (R 2.3.3)."
    );
    assert!(
        words(&passed, "ended-value:NEXT · SHOWN, NOT EXECUTED")
            .starts_with("Directorio, 1st Eligible, may:")
    );
    app.capture("ended-pass");
    app.quit();
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and a working Vulkan renderer; opens no desktop window"]
fn native_right_drag_orbits_and_the_wheel_zooms_the_table_camera() {
    let cards = std::env::var_os("CUBA_TEST_CARDS")
        .expect("Set CUBA_TEST_CARDS to the accepted card directory")
        .into();
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-79"));
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
    let camera = |view: &Value, field: &str| view["table"]["camera"][field].as_f64().unwrap();
    // Resting on a space starts its status dwell, during which the table takes no input:
    // point and settle first, as a player's pointer would come to rest.
    let home = app.point("space:las-villas");

    // The seat presses the right button on the map, moves the pointer by (120, 30)
    // logical pixels while it is held and releases it: the camera's own scene drag.
    let dragged = app.request(
        json!({"op":"drag","revision":0,"component":"space:las-villas","button":"right","x":120,"y":30}),
        true,
    );
    assert_eq!(dragged["status"], "dragged", "{dragged}");
    let orbited = app.request(json!({"op":"settle"}), true);
    // 120 * 0.006 radians of yaw and 30 * 0.004 of pitch.
    assert!(
        (camera(&orbited, "yaw") - (camera(&home, "yaw") - 0.72)).abs() < 0.0001,
        "{orbited}"
    );
    assert!(
        (camera(&orbited, "pitch") - (camera(&home, "pitch") + 0.12)).abs() < 0.0001,
        "{orbited}"
    );
    assert_eq!(camera(&orbited, "distance"), camera(&home, "distance"));
    app.capture("orbited");

    // Two wheel lines in: 0.95 squared of the distance.
    app.point("space:las-villas");
    let scrolled = app.request(
        json!({"op":"scroll","revision":0,"component":"space:las-villas","y":2}),
        true,
    );
    assert_eq!(scrolled["status"], "scrolled", "{scrolled}");
    let zoomed = app.request(json!({"op":"settle"}), true);
    assert!(
        (camera(&zoomed, "distance") - camera(&home, "distance") * 0.9025).abs() < 0.001,
        "{zoomed}"
    );
    assert_eq!(camera(&zoomed, "yaw"), camera(&orbited, "yaw"));
    app.capture("zoomed");

    app.request(json!({"op":"key","key":"Home","revision":0}), true);
    assert_eq!(app.observe()["table"]["camera"], home["table"]["camera"]);
    app.quit();
}
