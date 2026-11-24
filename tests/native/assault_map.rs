//! #169: Government chooses Assault's space on the map and nowhere else. The map marks
//! every space for the open Assault, a click opens the space popover with that space's
//! plans, and the popover's one button pays and resolves. The panel keeps what is not
//! a place: the cost, the record of the spaces paid, Resource transfer, Remove own
//! pieces and Finish Assault. Played at the owner's display by space picks alone.
//!
//! Expected values are worked from R 3.2.4 on the accepted tactical handoff's position
//! (`docs/research/tactical-slice-handoff.md`) after this path's decisions: seed 2's
//! Kidnap roll takes 5 of Government's 6 Resources, Santiago's Terror flips one of
//! 26July's two Guerrillas there to Active, and the shaded Event makes every 26July
//! Guerrilla Active. A space costs 2 Resources at a Firm US Alliance (R 6.3.1).
//! - **Havana**, City: 6 Troops + 4 Police = 10. Directorio's 2 Guerrillas are
//!   Underground; Syndicate has no Guerrillas, so its open Casino would close (R 1.4.5).
//! - **Santiago de Cuba**, City: 2 Troops + 1 Police = 3. Both Active 26July
//!   Guerrillas would return.
//! - **The other 11 spaces:** no Government cube stands with an enemy piece (Las
//!   Villas has 3 Troops and no enemy; Camagüey City 1 Troops + 2 Police and no enemy),
//!   so Assault would remove nothing there.
use super::*;
use cuba_libre_unfolded::save::Session;
use std::fs;
use unfolded_record_replacement::previous_path;

const OTHERS: [&str; 11] = [
    "camaguey-city",
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
];

/// Card 18 at Government's opening decision with Government at 1 Resource, through
/// the same checked and durable submission the seat uses.
fn short_of_the_cost(path: &Path) {
    let mut session = Session::create_seeded(path, 2).unwrap();
    for action in [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
    ] {
        let view = session.observe();
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        session.submit(owner, view.revision, &id).unwrap();
    }
    assert_eq!(session.observe().resources, [1, 4, 5, 15]);
}

/// Captures the window and keeps two 1:1 crops for inspection at the owner's display
/// size: the decision panel, and the map around the open popover or, with none open,
/// the island.
fn shot(app: &mut Native, name: &str) {
    app.capture(name);
    let capture = image::open(app.root.join(format!("{name}.png"))).unwrap();
    capture
        .crop_imm(2880, 400, 960, 1200)
        .save(app.root.join(format!("{name}-panel.png")))
        .unwrap();
    let popover = app.observe()["table"]["popover"]["bounds"].clone();
    let (x, y) = match popover.as_array() {
        Some(_) => {
            let [left, top, right, bottom] = rect_of(&popover);
            (
                ((left + right) / 2. - 800.).clamp(0., 2240.) as u32,
                ((top + bottom) / 2. - 500.).clamp(0., 1160.) as u32,
            )
        }
        None => (700, 500),
    };
    capture
        .crop_imm(x, y, 1600, 1000)
        .save(app.root.join(format!("{name}-map.png")))
        .unwrap();
}

fn pin(app: &mut Native, action: &str) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.activate(&format!("option:{action}"), revision);
}

fn escape(app: &mut Native) {
    let revision = app.observe()["observation"]["revision"].as_u64().unwrap();
    app.request(json!({"op":"key","revision":revision,"key":"Escape"}), true);
    app.request(json!({"op":"settle"}), true);
}

fn records(app: &Native) -> (Vec<u8>, Vec<u8>) {
    (
        fs::read(&app.save).unwrap(),
        fs::read(previous_path(&app.save)).unwrap(),
    )
}

/// Every space's state on the map and the one graphic drawn for it: its rule when
/// selectable, its dashed rule when blocked, and its veil when Assault would have no
/// effect there or the space is not involved.
fn assert_map(view: &Value, expected: &[(&str, &str)]) {
    let targets = view["table"]["map_targets"]
        .as_array()
        .unwrap_or_else(|| panic!("not targeting: {}", view["table"]["map_targets"]));
    assert_eq!(targets.len(), 13);
    let choices = view["observation"]["choices"].as_array().unwrap();
    let mut files = vec![];
    for target in targets {
        let space = target["space"].as_str().unwrap();
        let state = target["state"].as_str().unwrap();
        let (_, wanted) = expected
            .iter()
            .find(|(id, _)| *id == space)
            .unwrap_or_else(|| panic!("no expectation for {space}"));
        assert_eq!(state, *wanted, "{space}");
        // The seat reads the choices behind each state, and each states its space.
        let behind: Vec<&Value> = choices
            .iter()
            .filter(|c| c["site"]["space"] == space)
            .collect();
        assert_eq!(
            target["choices"].as_array().unwrap().len(),
            behind.len(),
            "{space}"
        );
        assert_eq!(behind.is_empty(), state == "not-involved", "{space}");
        files.push(match state {
            "selectable" | "blocked" => format!("atlas-highlight-{state}-{space}.png"),
            _ => format!("atlas-highlight-veil-{space}.png"),
        });
    }
    files.sort();
    let shown: Vec<String> = view["table"]["highlights"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(shown, files);
}

fn states<'a>(havana: &'a str, santiago: &'a str, matanzas: &'a str) -> Vec<(&'a str, &'a str)> {
    let mut all = vec![("havana", havana), ("santiago", santiago)];
    all.extend(OTHERS.map(|space| {
        (
            space,
            if space == "matanzas" {
                matanzas
            } else {
                "no-effect"
            },
        )
    }));
    all
}

fn rows(view: &Value) -> Vec<(String, String, bool, Option<String>)> {
    view["table"]["popover"]["card"]["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("no popover: {}", view["table"]["popover"]))
        .iter()
        .map(|r| {
            (
                r["head"].as_str().unwrap().to_owned(),
                r["mark"].as_str().unwrap().to_owned(),
                r["chosen"].as_bool().unwrap(),
                r["button"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

fn row_text(view: &Value, index: usize) -> String {
    view["table"]["popover"]["card"]["rows"][index]["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l[1].as_str().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn part_text(view: &Value, name: &str) -> String {
    choice_part(view, name)["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn has_part(view: &Value, name: &str) -> bool {
    view["table"]["choice_parts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["part"] == name)
}

#[test]
#[ignore = "requires CUBA_TEST_CARDS and Vulkan; inspect the retained Assault map captures at 3840x2160"]
fn native_assault_is_chosen_on_the_map_by_space_picks_alone() {
    let evidence = std::env::var_os("CUBA_NATIVE_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("local/verification/issue-169"));
    fs::create_dir_all(&evidence).unwrap();
    let root = tempfile::Builder::new()
        .prefix("assault-map-")
        .tempdir_in(evidence.canonicalize().unwrap())
        .unwrap()
        .keep();
    let socket = tempfile::tempdir().unwrap();
    let mut app = Native {
        cards: std::env::var_os("CUBA_TEST_CARDS").unwrap().into(),
        save: root.join("task.json"),
        socket: socket.path().join("seat.sock"),
        root,
        step: 0,
    };
    eprintln!("Assault on the map evidence: {}", app.root.display());
    short_of_the_cost(&app.save);
    let layout: Value = serde_json::from_str(include_str!(
        "../../art/atlas/atlas-action-choice.layout.json"
    ))
    .unwrap();
    let ratio = 3840. / layout["frame"][0].as_f64().unwrap();
    // A part's native bounds against its layout record entry, on the given edges
    // (left, top, right, bottom), within 2 px at 3840×2160.
    let part =
        |view: &Value, name: &str| -> [f64; 4] { rect_of(&choice_part(view, name)["bounds"]) };
    let compare = |view: &Value, group: &Value, edges: &[usize]| {
        for (name, r) in group.as_object().unwrap() {
            let r: Vec<f64> = r
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() * ratio)
                .collect();
            let expected = [r[0], r[1], r[0] + r[2], r[1] + r[3]];
            let native = part(view, name);
            for &edge in edges {
                assert!(
                    (native[edge] - expected[edge]).abs() <= 2.,
                    "{name} edge {edge}: native {native:?} differs from layout {expected:?} by more than 2 px"
                );
            }
        }
    };

    app.launch_size("3840x2160");
    pin(&mut app, "government:assault");
    app.choose("government:assault");
    escape(&mut app);
    app.point("area:propaganda");

    // 1. Government has 1 Resource: Havana and Santiago are blocked, and the other 11
    //    spaces, where Assault would remove nothing, are veiled and stay choosable.
    let opened = app.observe();
    assert_eq!(opened["observation"]["resources"], json!([1, 4, 5, 15]));
    assert_eq!(opened["observation"]["executing"], "Government");
    assert_map(&opened, &states("blocked", "blocked", "no-effect"));
    assert_eq!(opened["table"]["popover"], Value::Null);
    assert_eq!(part_text(&opened, "choice-heading"), "Government · Assault");
    assert_eq!(
        part_text(&opened, "choice-subtitle"),
        "1 Resource · Firm US Alliance\n2 Resources per space"
    );
    assert_eq!(
        part_text(&opened, "assault-map-line"),
        "Choose a space on the map; its popover pays and resolves."
    );
    // The panel lists no place: its rows are the three steps that select no space.
    let options: Vec<String> = opened["table"]["choice_parts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["part"].as_str())
        .filter(|p| p.starts_with("option:") && !p.ends_with("-label"))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        options,
        ["option:end-assault", "option:remove", "option:transfer"]
    );
    assert!(
        opened["table"]["choice_parts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| !p["part"].as_str().unwrap().starts_with("assault-record:"))
    );
    // With nothing paid the rows stand right under the map line, as the layout
    // record's "nothing paid" panel has them.
    compare(
        &opened,
        &layout["assault"]["opened"]["parts"],
        &[0, 1, 2, 3],
    );
    compare(&opened, &layout["assault"]["opened"]["text"], &[0, 1]);
    shot(&mut app, "1-opened-blocked");

    // 2. A blocked space's popover: each plan with the authority's reason and the
    //    Resource transfer that would open it, and no button. Looking saves nothing.
    let saved = records(&app);
    let revision = opened["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", revision);
    let blocked = app.observe();
    assert_eq!(blocked["observation"], opened["observation"]);
    assert_eq!(records(&app), saved);
    assert_eq!(
        blocked["table"]["popover"]["card"]["title"],
        "Santiago de Cuba"
    );
    assert_eq!(
        rows(&blocked),
        [
            (
                "Target 26July".to_owned(),
                "not-yet".to_owned(),
                false,
                None
            ),
            (
                "Removes nothing".to_owned(),
                "not-yet".to_owned(),
                false,
                None
            ),
        ]
    );
    assert!(
        row_text(&blocked, 0)
            .starts_with("Government has 1 Resource and cannot pay 2 Resources (R 6.3.1)."),
        "{}",
        row_text(&blocked, 0)
    );
    assert_eq!(
        blocked["table"]["popover"]["card"]["rows"][0]["remedies"],
        json!(["Request a Resource transfer"])
    );
    shot(&mut app, "2-santiago-blocked-popover");
    escape(&mut app);

    // 3. A Resource transfer suspends the targeting while it is under way, and turns
    //    the blocked spaces selectable at once: 26July gives 3, Government has 4.
    pin(&mut app, "transfer");
    app.choose("transfer");
    assert_eq!(app.observe()["table"]["map_targets"], Value::Null);
    app.choose("ask:1:3");
    assert_eq!(app.observe()["table"]["map_targets"], Value::Null);
    app.choose("authorize:3");
    app.point("area:propaganda");
    let funded = app.observe();
    assert_eq!(funded["observation"]["resources"], json!([4, 1, 5, 15]));
    assert_map(&funded, &states("selectable", "selectable", "no-effect"));
    shot(&mut app, "3-funded-selectable");

    // 4. Santiago's popover: the plan that removes something first and chosen, with
    //    the authority's preview and the one button; the plan that removes nothing last.
    let saved = records(&app);
    let revision = funded["observation"]["revision"].as_u64().unwrap();
    app.activate("space:santiago", revision);
    let open = app.observe();
    assert_eq!(open["observation"], funded["observation"]);
    assert_eq!(records(&app), saved);
    assert_eq!(
        open["table"]["popover"]["card"]["subtitle"],
        "City · Government executing Assault · Government has 4 Resources"
    );
    assert_eq!(
        rows(&open),
        [
            (
                "Target 26July".to_owned(),
                "selectable".to_owned(),
                true,
                Some("Pay 2 · Assault in Santiago de Cuba".to_owned())
            ),
            (
                "Removes nothing".to_owned(),
                "selectable".to_owned(),
                false,
                None
            ),
        ]
    );
    let preview = row_text(&open, 0);
    for fact in [
        "2 Troops + 1 Police = 3 in a City (R 3.2.4).",
        "Pay 2 Resources: Government 4 → 2.",
        "26July: 2 Active, 0 Underground.",
        "R 3.2.4: Return 2 26July Active Guerrillas to Available Forces.",
    ] {
        assert!(preview.contains(fact), "{fact}: {preview}");
    }
    shot(&mut app, "4-santiago-popover");

    // The plan that removes nothing says so on its button; choosing a row decides
    // nothing.
    app.activate("popover:row:assault:santiago:none", revision);
    let nothing = app.observe();
    assert_eq!(nothing["observation"], funded["observation"]);
    assert_eq!(
        rows(&nothing)[1],
        (
            "Removes nothing".to_owned(),
            "selectable".to_owned(),
            true,
            Some("Pay 2 · Assault removes nothing".to_owned())
        )
    );
    assert!(row_text(&nothing, 1).contains("R 3.2.4: No pieces affected."));
    shot(&mut app, "5-santiago-removes-nothing-row");
    app.activate("popover:row:assault:santiago:26july", revision);

    // 5. The button pays and resolves at once: 4 − 2 = 2, and both Active 26July
    //    Guerrillas return to Available Forces.
    app.activate("popover:commit", revision);
    app.request(json!({"op":"settle"}), true);
    app.point("area:propaganda");
    let paid = app.observe();
    let after = &paid["observation"];
    assert_eq!(after["revision"].as_u64().unwrap(), revision + 1);
    assert_eq!(after["resources"], json!([2, 1, 5, 15]));
    let santiago = after["spaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == "santiago")
        .unwrap();
    assert!(
        santiago["pieces"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["faction"] != "M26")
    );
    assert_eq!(
        after["available_piece_count"].as_u64().unwrap(),
        funded["observation"]["available_piece_count"]
            .as_u64()
            .unwrap()
            + 2
    );
    // The paid space is not involved: its veil is drawn and a click opens nothing.
    assert_map(&paid, &states("selectable", "not-involved", "no-effect"));
    assert_eq!(paid["table"]["popover"], Value::Null);
    app.activate("space:santiago", revision + 1);
    assert_eq!(app.observe()["table"]["popover"], Value::Null);
    // The panel's record line, under the cost lines.
    assert_eq!(
        part_text(&paid, "choice-subtitle"),
        "2 Resources · Firm US Alliance\n2 Resources per space"
    );
    assert_eq!(
        part_text(&paid, "assault-record:santiago").replace('\n', " "),
        "Santiago de Cuba · paid 2 Resources · returned 2 26July Guerrillas"
    );
    shot(&mut app, "6-santiago-paid");

    // 6. A no-effect space stays choosable: Matanzas is empty Grassland (0 Troops =
    //    0). Its popover holds the one plan, which pays 2 and changes nothing else.
    app.activate("space:matanzas", revision + 1);
    let quiet = app.observe();
    assert_eq!(
        rows(&quiet),
        [(
            "Removes nothing".to_owned(),
            "selectable".to_owned(),
            true,
            Some("Pay 2 · Assault removes nothing".to_owned())
        )]
    );
    let preview = row_text(&quiet, 0);
    for fact in [
        "0 Troops = 0 in Grassland. 0 Police do not count (R 3.2.4).",
        "Pay 2 Resources: Government 2 → 0.",
        "No enemy pieces are there.",
        "R 3.2.4: No pieces affected.",
    ] {
        assert!(preview.contains(fact), "{fact}: {preview}");
    }
    shot(&mut app, "7-matanzas-popover");
    app.activate("popover:commit", revision + 1);
    app.request(json!({"op":"settle"}), true);
    app.point("area:propaganda");
    let second = app.observe();
    let settled = &second["observation"];
    assert_eq!(settled["resources"], json!([0, 1, 5, 15]));
    assert_eq!(settled["spaces"], after["spaces"]);
    assert_eq!(settled["available"], after["available"]);
    assert_eq!(settled["standings"], after["standings"]);
    // Government is at 0 again: Havana is blocked. Both paid spaces are not involved.
    assert_map(&second, &states("blocked", "not-involved", "not-involved"));
    assert_eq!(
        part_text(&second, "assault-record:matanzas"),
        "Matanzas · paid 2 Resources · removed nothing"
    );
    shot(&mut app, "8-matanzas-paid");

    // 7. The panel matches its layout record's Assault frame, which shows this state:
    //    the map line, the two records in the order paid, and the three rows under them.
    compare(&second, &layout["assault"]["parts"], &[0, 1, 2, 3]);
    compare(&second, &layout["assault"]["text"], &[0, 1]);
    // No record wraps beyond the lines the panel planned for it: each record here
    // is one line, and its drawn height follows.
    let line = layout["assault"]["record_line"].as_f64().unwrap() * ratio;
    for (name, lines) in [
        ("assault-record:santiago", 1.),
        ("assault-record:matanzas", 1.),
    ] {
        let [_, top, _, bottom] = part(&second, name);
        assert!(
            ((bottom - top) - lines * line).abs() <= 2.,
            "{name}: {} px tall, {lines} lines of {line}",
            bottom - top
        );
    }
    // The records sit in the order paid, above the rows.
    let top = |name: &str| part(&second, name)[1];
    assert!(top("assault-map-line") < top("assault-record:santiago"));
    assert!(top("assault-record:santiago") < top("assault-record:matanzas"));
    assert!(top("assault-record:matanzas") < top("option:end-assault"));

    // 8. A restart resumes the Assault with its paid spaces.
    let resumed_from = records(&app);
    app.quit();
    app.launch_size("3840x2160");
    app.point("area:propaganda");
    let resumed = app.observe();
    assert_eq!(resumed["observation"], *settled);
    assert_eq!(records(&app), resumed_from);
    assert_map(&resumed, &states("blocked", "not-involved", "not-involved"));
    assert_eq!(
        part_text(&resumed, "assault-record:santiago").replace('\n', " "),
        "Santiago de Cuba · paid 2 Resources · returned 2 26July Guerrillas"
    );
    assert!(has_part(&resumed, "assault-record:matanzas"));
    shot(&mut app, "9-resumed");

    // 9. Finish Assault stays a row of the panel.
    pin(&mut app, "end-assault");
    app.choose("end-assault");
    let handoff = app.observe();
    assert_eq!(handoff["observation"]["owner"], "Syndicate");
    assert_eq!(handoff["observation"]["next"]["role"], "2nd Eligible");
    assert_eq!(handoff["table"]["map_targets"], Value::Null);
    shot(&mut app, "10-finished");
    app.quit();
}
