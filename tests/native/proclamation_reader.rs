use super::{faction_seats, *};
use cuba_libre_unfolded::save::Session;
use faction_seats::{DIRECTORIO, M26, SYNDICATE, holding, seat};
use std::time::{Duration, Instant};

const RETURN: &str = "proclamation:return";
const INBOX: &str = "proclamations";

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_reads_proclamations_in_order_over_the_pending_giver_choice() {
    let mut app = pending_giver("proclamation-reader-");
    let m26 = app.seat_agent(M26, "26july");
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    app.key("Tab");
    app.key("Backspace");
    let before = app.key("1");
    let pending = before["observation"].clone();
    let revision = app.revision();
    assert_eq!(pending["owner"], "Syndicate", "{pending}");
    assert_eq!(
        before["table"]["transfer"]["state"]["focus"],
        "transfer:field"
    );
    app.capture("pending-giver-before-reading");

    let first = "Syndicate, the doors of the Havana Casino are chained. Two of your Resources will help the house reopen. In return, when you ask for a contribution on a later card, we will hear you first. Our word is public. Government can read it, and so can Directorio. Give 1 if two is too dear, or give nothing. Either way, when the lights come on, Havana will remember who brought the key. Ask us again at dusk.";
    assert_eq!(first.chars().count(), 400);
    let second = concat!(
        "A public promise is worth hearing.\n",
        "We saw the Casino's doors close.\n",
        "We were there the night before.\n",
        "Havana should remember.\n",
        "We saw no contract.\n",
        "We heard the offer.\n",
        "We can repeat the words.\n",
        "We can witness an answer.\n",
        "The giver still decides.",
    );
    assert!(second.chars().count() < 400);
    publish(&mut app, &m26, revision, first);
    let view = app.settle();
    let reader = &view["table"]["proclamations"];
    assert_eq!(reader["open"], 1, "{reader}");
    assert_eq!(reader["documents"][0]["author"], "M26");
    assert_eq!(reader["documents"][0]["text"], first);
    assert_eq!(reader["documents"][0]["read"], false);
    app.capture("first-broadsheet-over-giver");

    publish(&mut app, &directorio, revision, second);
    let queued = app.settle();
    assert_eq!(queued["table"]["proclamations"]["open"], 1);
    assert_eq!(queued["table"]["proclamations"]["unread"], 2);
    assert_eq!(queued["observation"], pending);

    app.activate(RETURN, revision);
    let between = app.settle();
    assert_eq!(between["table"]["proclamations"]["open"], Value::Null);
    assert_eq!(between["table"]["proclamations"]["unread"], 1);
    assert_eq!(between["table"]["transfer"], before["table"]["transfer"]);
    assert_eq!(between["observation"], pending);

    // The last arrival can stay unread while the ordinary table is available.
    app.activate(INBOX, revision);
    let next = app.settle();
    let reader = &next["table"]["proclamations"];
    assert_eq!(reader["open"], 2, "{reader}");
    assert_eq!(reader["documents"][0]["read"], true);
    assert_eq!(reader["documents"][1]["author"], "Directorio");
    assert_eq!(reader["documents"][1]["text"], second);
    assert_eq!(reader["documents"][1]["read"], false);
    app.capture("second-broadsheet-over-giver");
    let scroll = app.request(
        json!({"op":"scroll","revision":revision,"component":RETURN,"y":-8.}),
        true,
    );
    assert!(scroll["table"]["proclamations"]["scroll"].as_f64().unwrap() > 0.);
    app.capture("second-broadsheet-last-line");

    app.request(
        json!({"op":"key","key":"Enter","revision":revision,"focused":RETURN}),
        true,
    );
    let restored = app.settle();
    assert_eq!(restored["table"]["proclamations"]["open"], Value::Null);
    assert_eq!(restored["table"]["proclamations"]["unread"], 0);
    assert_eq!(restored["table"]["transfer"], before["table"]["transfer"]);
    assert_eq!(restored["observation"], pending);
    app.capture("pending-giver-focus-restored");

    app.activate(INBOX, revision);
    let reread = app.settle();
    assert_eq!(reread["table"]["proclamations"]["open"], 1);
    assert_eq!(
        reread["table"]["proclamations"]["documents"][0]["read"],
        true
    );
    assert!(reread["table"]["newspaper"]["open"].is_null());
    assert_eq!(app.key("ArrowRight")["table"]["proclamations"]["open"], 2);
    assert_eq!(app.key("ArrowLeft")["table"]["proclamations"]["open"], 1);
    app.key("Escape");
    assert_eq!(app.observe()["observation"], pending);
    let keyboard = app.key("KeyR");
    assert_eq!(keyboard["table"]["proclamations"]["open"], 1);
    assert_eq!(app.key("Escape")["observation"], pending);
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_rechecks_a_givers_held_choice_after_reading() {
    for take_back in [false, true] {
        let prefix = if take_back {
            "proclamation-reclaimed-"
        } else {
            "proclamation-held-"
        };
        let mut app = pending_giver(prefix);
        let giver = app.seat_agent(SYNDICATE, "syndicate");
        let before = app.settle();
        assert!(holding(&before).is_empty());
        let position = before["observation"].clone();
        assert_eq!(position["resources"], json!([1, 4, 5, 15]));
        let revision = app.revision();
        let revision_arg = revision.to_string();
        let id = format!("{revision}:authorize:2");

        let mut offered = app.player(
            "same-frame-publication",
            &[
                "publish",
                &giver,
                &revision_arg,
                "17",
                "We offer two Resources.",
            ],
        );
        // Observe does not drive a frame. Wait until publication is the first
        // request queued for the table, then let the second client reach the same
        // inbox before running one frame.
        let deadline = Instant::now() + Duration::from_secs(10);
        while app.observe()["pending"] != json!(["faction seats"]) {
            assert!(
                Instant::now() < deadline,
                "publication did not reach the table"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut selected = app.player("same-frame-choice", &["choose", &giver, &revision_arg, &id]);
        std::thread::sleep(Duration::from_millis(200));
        let held = app.request(json!({"op":"advance","steps":1}), true);
        assert_eq!(held["observation"], position);
        assert_eq!(held["table"]["proclamations"]["open"], 1);
        assert_eq!(seat(&held, "Syndicate")["holds_choice"], true);
        assert_eq!(holding(&app.settle()), ["a proclamation"]);
        let (code, publication) = app.answered(&mut offered);
        assert_eq!((code, &publication["status"]), (0, &json!("published")));
        let (code, submission) = app.answered(&mut selected);
        assert_eq!((code, &submission["status"]), (0, &json!("held")));
        app.capture("giver-choice-held-by-publication");

        if take_back {
            // Seat custody is still available during reading. Reclaiming it ends
            // the held identity and withdraws its queued authorization.
            let reclaimed = app.key(SYNDICATE);
            assert_eq!(reclaimed["table"]["proclamations"]["open"], 1);
            assert_eq!(seat(&reclaimed, "Syndicate")["holds_choice"], false);
            assert_eq!(reclaimed["observation"], position);
            app.key("Escape");
            let restored = app.settle();
            assert_eq!(restored["observation"], position);
            assert_eq!(restored["table"]["transfer"]["state"]["shown"], true);
            let (_, ended) = app.ask(&["choose", &giver, &revision_arg, &id]);
            assert_eq!(ended["status"], "closed", "{ended}");
            assert_eq!(app.settle()["observation"], position);
            app.capture("reclaimed-giver-choice-still-pending");
        } else {
            app.key("Escape");
            let applied = app.settle();
            assert_eq!(applied["observation"]["revision"], revision + 1);
            assert_eq!(applied["observation"]["resources"], json!([1, 6, 5, 13]));
            assert_eq!(seat(&applied, "Syndicate")["holds_choice"], false);
            let (_, repeated) = app.ask(&["choose", &giver, &revision_arg, &id]);
            assert_eq!(repeated["status"], "refused", "{repeated}");
            assert_eq!(app.settle()["observation"], applied["observation"]);
            app.capture("held-authorization-applied-once");
        }
        app.quit();
    }
}

fn pending_giver(prefix: &str) -> NativeRun {
    let mut app = super::available_forces::unlaunched(prefix, "issue-231");
    let mut seed = Session::create_seeded(&app.save, 2).unwrap();
    for action in ["terror", "kidnap", "kidnap:roll", "terror:santiago"] {
        let view = seed.observe();
        seed.submit(
            view.owner,
            view.revision,
            &format!("{}:{action}", view.revision),
        )
        .unwrap();
    }
    drop(seed);

    app.launch_size("3840x2160");
    app.choose("transfer");
    app.choose("ask:3:2");
    app
}

fn publish(app: &mut Native, player: &str, revision: u64, text: &str) {
    let (code, answer) = app.ask(&["publish", player, &revision.to_string(), "17", text]);
    assert_eq!(
        (code, &answer["status"]),
        (0, &json!("published")),
        "{answer}"
    );
}
