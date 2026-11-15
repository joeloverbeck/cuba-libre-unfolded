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
        let output = Command::new(env!("CARGO_BIN_EXE_cuba-libre"))
            .args([
                "--cards",
                self.cards.to_str().unwrap(),
                "--save",
                self.save.to_str().unwrap(),
                "--agent-seat",
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
    app.capture("before-roll");
    app.choose("kidnap");
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
        json!({"op":"activate","revision":0,"choice":"0:kidnap"}),
        false,
    );
    assert_eq!(app.observe()["observation"], rolled["observation"]);
    let rev = rolled["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("{rev}:finish"), rev);
    assert!(
        app.observe()["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("owed")
    );
    assert_eq!(std::fs::read(&app.save).unwrap(), saved_roll);
    app.activate("cancel", rev);
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
    app.activate("space:santiago", 2);
    app.capture("santiago-after-terror");
    std::fs::copy(&app.save, app.root.join("terror-valid.json")).unwrap();
    app.quit();
    app.launch();
    assert_eq!(app.observe()["observation"], after["observation"]);
    app.request(
        json!({"op":"activate","revision":1,"choice":"1:terror:santiago"}),
        false,
    );
    assert_eq!(app.observe()["observation"], after["observation"]);
    app.activate("standing:1", 2);
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
    assert!(
        both["table"]["details_height"].as_f64().unwrap() <= 190.,
        "contributions must fit above the footer: {}",
        both["table"]["details_height"]
    );
    app.capture("both-terrors");
    app.choose("finish");
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
        app.choose("finish");
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
    assert_eq!(app.observe()["table"]["pinned"], "Government");
    app.activate("close", 0);
    app.activate("0:remove", 0);
    let disclosed = app.observe();
    assert!(
        disclosed["table"]["notice"]
            .as_str()
            .unwrap()
            .contains("Incomplete")
    );
    assert_eq!(disclosed["observation"], initial["observation"]);
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
        assert_eq!(app.observe()["table"]["inspected"], id);
    }
    app.capture("raised-ec-inspection");
    app.activate("cancel", 0);
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
    assert_eq!(standings.len(), 4);
    assert!(standings.iter().all(|standing| standing["visible"] == true));
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
    app.request(
        json!({"op":"activate","revision":0,"choice":"0:finish"}),
        true,
    );
    app.request(json!({"op":"settle"}), true);
    let finished = app.observe();
    assert_eq!(finished["observation"]["revision"], 1);
    assert_eq!(finished["observation"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(finished["observation"]["owner"], "Directorio");
    assert_eq!(
        finished["observation"]["next_options"],
        json!(["Pass (+1 if executed later)", "Limited Operation"])
    );
    app.capture("finished");
    let valid = std::fs::read(&app.save).unwrap();
    app.request(
        json!({"op":"activate","revision":0,"choice":"0:finish"}),
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
    for line in cuba_libre_unfolded::assets::MANIFEST.lines() {
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
    let original_path = app.save.clone();
    let previous = std::fs::read(original_path.with_file_name("task.json.previous")).unwrap();
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
        std::fs::read(original_path.with_file_name("task.json.previous")).unwrap(),
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
    assert_eq!(app.observe()["observation"], initial["observation"]);
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
    app.activate("close", revision);
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
    app.choose("finish");
    assert_eq!(app.observe()["observation"]["finished"], true);
    app.capture("transfers-finished");
    app.quit();
}
