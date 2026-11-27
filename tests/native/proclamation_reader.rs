use super::{faction_seats, *};
use cuba_libre_unfolded::save::Session;
use faction_seats::{DIRECTORIO, GOVERNMENT, M26, SYNDICATE, holding, seat};
use std::fs;
use std::time::{Duration, Instant};
use unfolded_record_replacement::previous_path;

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
    let covered = app.request(
        json!({"op":"activate","revision":revision,"component":"transfer:field"}),
        false,
    );
    assert_eq!(covered["status"], "missed");
    assert_eq!(covered["observation"], pending);
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
    app.capture("archive-entry-with-unread-proclamation");

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

    let government = app.seat_agent(GOVERNMENT, "government");
    publish(&mut app, &government, revision, "The giver may answer.");
    app.activate(RETURN, revision);
    app.activate(INBOX, revision);
    app.activate("proclamation:next", revision);
    let middle = app.observe();
    assert_eq!(middle["table"]["proclamations"]["open"], 2);
    assert_eq!(middle["observation"], pending);
    app.capture("archive-middle-with-both-controls");
    app.activate("proclamation:previous", revision);
    assert_eq!(app.observe()["table"]["proclamations"]["open"], 1);
    app.quit();
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
            app.activate("seat:syndicate:take-back", revision);
            let reclaimed = app.observe();
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
            // A missing main record makes the shared replacement refuse. The
            // published words and held choice must survive the failed click.
            let saved = app.save_bytes();
            let previous = fs::read(previous_path(&app.save)).unwrap();
            let preserved = app
                .root
                .join("preserved-during-failed-acknowledgement.json");
            fs::rename(&app.save, &preserved).unwrap();
            app.activate(RETURN, revision);
            fs::rename(&preserved, &app.save).unwrap();
            let failed = app.observe();
            assert_eq!(failed["observation"], position);
            let reader = &failed["table"]["proclamations"];
            assert_eq!(reader["open"], 1);
            assert_eq!(reader["unread"], 1);
            assert_eq!(reader["documents"][0]["read"], false);
            assert!(reader["save_error"].is_string(), "{reader}");
            assert_eq!(seat(&failed, "Syndicate")["holds_choice"], true);
            assert_eq!(holding(&failed), ["a proclamation"]);
            assert_eq!(app.save_bytes(), saved);
            assert_eq!(fs::read(previous_path(&app.save)).unwrap(), previous);
            app.capture("failed-acknowledgement-keeps-choice-held");

            app.key("Escape");
            let applied = app.settle();
            assert!(applied["table"]["proclamations"]["save_error"].is_null());
            assert_eq!(applied["table"]["proclamations"]["unread"], 0);
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

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan"]
fn native_resume_and_recovery_preserve_proclamations_and_the_pending_giver() {
    let mut app = pending_giver_in("proclamation-resume-", "issue-232");
    let m26 = app.seat_agent(M26, "26july");
    let directorio = app.seat_agent(DIRECTORIO, "directorio");
    let revision = app.revision();
    let first = "Authorize 2 Resources. 26July will hear your request on the next card.";
    let second = "Directorio heard the offer.\nThe choice remains with Syndicate.";
    publish(&mut app, &m26, revision, first);
    publish(&mut app, &directorio, revision, second);
    app.activate(RETURN, revision);
    app.choose("wait");

    let before = app.observe();
    let pending = &before["observation"];
    let request = &pending["transfer"]["Authorizing"];
    assert_eq!(pending["owner"], "Syndicate");
    assert_eq!(pending["resources"], json!([1, 4, 5, 15]));
    assert_eq!(request["amount"], 2, "{request}");
    assert_eq!(request["waited"], true);
    let documents = json!([
        {"number":1,"card":17,"author":"M26","text":first,"read":true},
        {"number":2,"card":17,"author":"Directorio","text":second,"read":false}
    ]);
    let reader =
        json!({"documents":documents,"open":null,"scroll":0.0,"unread":1,"save_error":null});
    assert_eq!(before["table"]["proclamations"], reader);
    app.capture("before-save-quit-with-unread-proclamation");
    let waiting_save = app.save_bytes();
    let before_quit_previous = fs::read(previous_path(&app.save)).unwrap();

    let resumed = super::save_quit::save_quit_and_resume(&mut app, false);
    assert_eq!(resumed["observation"], *pending);
    assert_eq!(resumed["table"]["proclamations"], reader);
    assert_eq!(app.save_bytes(), waiting_save);
    assert_eq!(
        fs::read(previous_path(&app.save)).unwrap(),
        before_quit_previous
    );
    assert_allowances_and_reset_seats(&mut app, &resumed, first, second);
    app.capture("reopened-pending-giver-with-unread-proclamation");

    let revision = app.revision();
    app.activate(INBOX, revision);
    let unread = app.observe();
    assert_eq!(unread["table"]["proclamations"]["open"], 2);
    let reread = app.key("ArrowLeft");
    assert_eq!(reread["table"]["proclamations"]["open"], 1);
    assert_eq!(
        reread["table"]["proclamations"]["documents"][0]["text"],
        first
    );
    assert_eq!(reread["observation"], *pending);
    app.capture("reread-original-offer-before-authorizing");
    let both_read = app.save_bytes();
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), waiting_save);
    app.key("Escape");
    assert_eq!(app.save_bytes(), both_read, "rereading replaced the save");
    assert_eq!(fs::read(previous_path(&app.save)).unwrap(), waiting_save);
    let mut reader_after_reading = reader;
    reader_after_reading["unread"] = 0.into();
    reader_after_reading["documents"][1]["read"] = true.into();

    // This particular previous record still has one unread document, even
    // though the source now records both. Recover it without changing either.
    app.quit();
    let source =
        super::construct_transfers::launch_recovered(&mut app, "recovered-one-unread.json");
    let earlier = app.observe();
    assert_eq!(earlier["observation"], *pending);
    assert_eq!(
        earlier["table"]["proclamations"],
        before["table"]["proclamations"]
    );
    assert_eq!(app.save_bytes(), waiting_save);
    assert_eq!(fs::read(&source).unwrap(), both_read);
    assert_eq!(fs::read(previous_path(&source)).unwrap(), waiting_save);
    app.capture("recovered-record-with-one-unread-proclamation");
    app.quit();
    app.save = source;
    app.launch_size("3840x2160");
    assert_eq!(app.save_bytes(), both_read);

    app.choose("reopen");
    let before_authorizing = app.observe();
    assert_eq!(
        before_authorizing["observation"]["transfer"]["Authorizing"]["amount"],
        2
    );
    assert_eq!(
        before_authorizing["table"]["proclamations"],
        reader_after_reading
    );
    let before_authorizing_save = app.save_bytes();
    let giver_revision = app.revision();
    let authorization = format!("{giver_revision}:authorize:2");
    app.choose("authorize:2");
    let applied = app.observe();
    assert_eq!(applied["observation"]["revision"], giver_revision + 1);
    assert!(applied["observation"]["transfer"].is_null());
    assert_eq!(applied["observation"]["resources"], json!([1, 6, 5, 13]));
    assert_eq!(applied["table"]["proclamations"], reader_after_reading);
    app.capture("authorized-once-after-reopen");
    let committed = app.save_bytes();
    assert_eq!(
        fs::read(previous_path(&app.save)).unwrap(),
        before_authorizing_save
    );
    app.quit();

    app.launch_size("3840x2160");
    let committed_again = app.observe();
    assert_eq!(committed_again["observation"], applied["observation"]);
    assert_eq!(
        committed_again["table"]["proclamations"],
        reader_after_reading
    );
    app.request(
        json!({"op":"activate","revision":giver_revision,"choice":authorization}),
        false,
    );
    assert_allowances_and_reset_seats(&mut app, &committed_again, first, second);
    assert_eq!(app.save_bytes(), committed);
    app.quit();

    let source = super::construct_transfers::launch_recovered(&mut app, "recovered.json");
    let recovered = app.observe();
    assert_eq!(recovered["observation"], before_authorizing["observation"]);
    assert_eq!(recovered["table"]["proclamations"], reader_after_reading);
    assert_eq!(app.save_bytes(), before_authorizing_save);
    assert_allowances_and_reset_seats(&mut app, &recovered, first, second);
    app.capture("recovered-exact-giver-decision");
    app.choose("authorize:2");
    assert_eq!(app.save_bytes(), committed);
    assert_eq!(fs::read(&source).unwrap(), committed);
    assert_eq!(
        fs::read(previous_path(&source)).unwrap(),
        before_authorizing_save
    );

    app.quit();
}

fn assert_allowances_and_reset_seats(app: &mut Native, view: &Value, first: &str, second: &str) {
    for (faction, key, name, can_publish) in [
        ("Government", faction_seats::GOVERNMENT, "government", true),
        ("M26", M26, "26july", false),
        ("Directorio", DIRECTORIO, "directorio", false),
        ("Syndicate", SYNDICATE, "syndicate", true),
    ] {
        assert_eq!(seat(view, faction)["state"], "human");
        let player = app.seat_agent(key, name);
        let (code, seen) = app.ask(&["observe", &player]);
        assert_eq!(code, 0, "{seen}");
        assert_eq!(seen["view"]["proclamations"]["can_publish"], can_publish);
        assert_eq!(
            seen["view"]["proclamations"]["published"],
            json!([
                {"card":17,"author":"M26","text":first},
                {"card":17,"author":"Directorio","text":second}
            ])
        );
        app.key(key);
    }
}

fn pending_giver(prefix: &str) -> NativeRun {
    pending_giver_in(prefix, "issue-231")
}

fn pending_giver_in(prefix: &str, issue: &str) -> NativeRun {
    let mut app = super::available_forces::unlaunched(prefix, issue);
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
