//! Matched Santiago positions worked from R 3.1, 3.2.4, 6.3.1 and #130.
use cuba_libre_unfolded::{
    AssaultPreview, Choice, Faction, Game, Kind, Observation, State,
    reports::Reports,
    targets::{SpaceState, space_targets},
};

/// The plans the open Assault offers in `space`, as the authority states their space.
fn plans<'a>(view: &'a Observation, space: &str) -> Vec<&'a Choice> {
    view.choices
        .iter()
        .filter(|c| c.site.as_ref().is_some_and(|site| site.space == space))
        .collect()
}

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap_or_else(|error| panic!("{action}: {error}"));
}

#[test]
fn funded_santiago_assault_returns_only_active_guerrillas_then_hands_off() {
    let mut funded = crate::terror_chosen(Game::opening_with_seed(2));
    for action in ["kidnap", "kidnap:roll", "terror:santiago", "transfer"] {
        choose(&mut funded, action);
    }
    let transfer = funded.observe();
    assert_eq!(transfer.resources, [1, 4, 5, 15]);
    assert_eq!(transfer.owner, Faction::M26);
    choose(&mut funded, "give:0:1");
    choose(&mut funded, "end-terror");
    assert_eq!(funded.observe().resources, [2, 3, 5, 15]);
    let random = funded.random_state();

    for (event, steps, returned, underground) in [
        ("event:shaded", ["resolve-shaded", ""], 2, 0),
        (
            "event:unshaded",
            ["rally:beneficiary:26july", "rally:sierra-maestra:place:0"],
            1,
            1,
        ),
    ] {
        let mut game = funded.clone();
        choose(&mut game, event);
        for step in steps.into_iter().filter(|step| !step.is_empty()) {
            choose(&mut game, step);
        }
        choose(&mut game, "advance-card");
        choose(&mut game, "government:assault");
        let pending = game.observe();
        assert_eq!(pending.executing, Some(Faction::Government));
        assert_eq!(pending.resources, [2, 3, 5, 15]);
        let payment = pending
            .choices
            .iter()
            .find(|choice| choice.action() == "assault:santiago:26july")
            .unwrap();
        assert!(payment.explanation.contains("2 Troops + 1 Police = 3"));
        let preview = payment.preview.as_ref().unwrap();
        assert_eq!(preview.resources[0].from, 2);
        assert_eq!(preview.resources[0].to, 0);
        assert_eq!(preview.removed.len(), 1);
        assert_eq!(
            (
                preview.removed[0].kind,
                preview.removed[0].state,
                preview.removed[0].count
            ),
            (Kind::Guerrilla, State::Active, returned)
        );
        assert!(
            game.submit(
                pending.owner,
                pending.revision,
                &format!(
                    "{}:assault:santiago:26july:{}",
                    pending.revision,
                    returned + 1
                )
            )
            .is_err()
        );
        for rejected in [0, returned - 1] {
            assert!(
                game.submit(
                    pending.owner,
                    pending.revision,
                    &format!("{}:assault:santiago:26july:{rejected}", pending.revision),
                )
                .is_err()
            );
        }
        assert_eq!(game.observe(), pending);
        choose(&mut game, payment.action());
        let paid = game.observe();
        assert_eq!(paid.resources, [0, 3, 5, 15]);
        assert_eq!(paid.executing, Some(Faction::Government));
        assert_eq!(
            paid.map_piece_count,
            pending.map_piece_count - u16::from(returned)
        );
        assert_eq!(
            paid.available_piece_count,
            pending.available_piece_count + u16::from(returned)
        );
        let available_m26 = |view: &cuba_libre_unfolded::Observation| {
            view.available
                .iter()
                .find(|p| p.faction == Faction::M26 && p.kind == Kind::Guerrilla)
                .unwrap()
                .count
        };
        assert_eq!(available_m26(&paid), available_m26(&pending) + returned);
        let santiago = paid.spaces.iter().find(|s| s.id == "santiago").unwrap();
        assert_eq!(santiago.control(), Some(Faction::Government));
        assert_eq!(
            santiago
                .pieces
                .iter()
                .filter(|p| p.faction == Faction::M26)
                .map(|p| (p.kind, p.state, p.count))
                .collect::<Vec<_>>(),
            if underground == 0 {
                vec![]
            } else {
                vec![(Kind::Guerrilla, State::Underground, 1)]
            }
        );
        assert_eq!(paid.standings, pending.standings);
        assert!(
            game.submit(pending.owner, pending.revision, &payment.id)
                .is_err()
        );
        assert!(
            game.submit(
                paid.owner,
                paid.revision,
                &format!("{}:{}", paid.revision, payment.action())
            )
            .is_err()
        );
        assert_eq!(game.observe(), paid);

        choose(&mut game, "end-assault");
        let handoff = game.observe();
        assert_eq!(handoff.owner, Faction::Syndicate);
        assert_eq!(handoff.executing, None);
        let next = handoff.next.unwrap();
        assert_eq!(next.role, "2nd Eligible");
        assert_eq!(next.options, ["Pass", "Limited Operation"]);
        assert_eq!(
            handoff
                .choices
                .iter()
                .map(|c| c.action())
                .collect::<Vec<_>>(),
            ["syndicate:pass", "syndicate:limited-operation"]
        );
        assert_eq!(handoff.resources, paid.resources);
        assert_eq!(handoff.spaces, paid.spaces);
        assert_eq!(game.random_state(), random);
    }
}

#[test]
fn havana_assault_closes_the_targeted_casino_without_returning_it() {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:4",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:assault",
    ] {
        choose(&mut game, action);
    }
    let before = game.observe();
    assert_eq!(before.resources, [5, 0, 5, 15]);
    // #169: Havana's plans are offered with every other space's. Directorio's two
    // Guerrillas are Underground, so no plan targets it (#166's playtest): a row that
    // named it would say Assault affects it. Every row says why its pieces stay.
    let havana = plans(&before, "havana");
    assert_eq!(
        havana.iter().map(|c| c.action()).collect::<Vec<_>>(),
        ["assault:havana:syndicate", "assault:havana:none",]
    );
    let reason = "Directorio: 0 Active, 2 Underground. Assault cannot remove its Underground Guerrillas until they are Activated.";
    for row in &havana {
        assert!(row.explanation.contains(reason), "{}", row.explanation);
    }
    let gone = format!("{}:assault:havana:directorio+syndicate", before.revision);
    assert!(
        game.clone()
            .submit(before.owner, before.revision, &gone)
            .is_err()
    );
    let nothing = havana[1];
    let zero = nothing.preview.as_ref().unwrap();
    assert!(zero.flips.is_empty() && zero.removed.is_empty());
    assert_eq!(zero.resources[0].to, 3);
    let target = havana[0];
    assert!(target.explanation.contains("6 Troops + 4 Police = 10"));
    let preview = target.preview.as_ref().unwrap();
    assert_eq!(preview.resources[0].from, 5);
    assert_eq!(preview.resources[0].to, 3);
    assert!(preview.removed.is_empty());
    let effect = &preview.assault.as_ref().unwrap().effect;
    assert!(effect.contains("Close 1 Casino in place"), "{effect}");
    assert!(!effect.contains("to Available"), "{effect}");
    assert!(
        preview
            .notes
            .iter()
            .all(|note| !note.contains("to Available"))
    );
    let flip = &preview.flips[0];
    assert_eq!(
        (
            &*flip.space,
            flip.faction,
            flip.kind,
            flip.from,
            flip.to,
            flip.count
        ),
        (
            "havana",
            Faction::Syndicate,
            Kind::Casino,
            State::Open,
            State::Closed,
            1
        ),
    );

    for action in [
        "assault:havana:directorio",
        "assault:havana:syndicate:0",
        "assault:havana:syndicate:2",
    ] {
        assert!(
            game.submit(
                before.owner,
                before.revision,
                &format!("{}:{action}", before.revision)
            )
            .is_err()
        );
    }
    assert_eq!(game.observe(), before);
    choose(&mut game, target.action());
    let after = game.observe();
    assert_eq!(after.resources, [3, 0, 5, 15]);
    assert_eq!(after.map_piece_count, before.map_piece_count);
    assert_eq!(after.available, before.available);
    let havana = &after.spaces[0];
    assert_eq!(havana.control(), Some(Faction::Government));
    assert_eq!(
        havana
            .pieces
            .iter()
            .find(|p| p.kind == Kind::Casino)
            .unwrap()
            .state,
        State::Closed,
    );
    assert_eq!(
        havana
            .pieces
            .iter()
            .find(|p| p.faction == Faction::Directorio)
            .unwrap()
            .count,
        2,
    );
    let standing = &after.standings[Faction::Syndicate.index()];
    assert_eq!(standing.score, 2);
    assert_eq!(standing.margin, -15);
}

#[test]
fn all_thirteen_spaces_allow_inspection_and_pay_once_even_when_no_pieces_can_be_affected() {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:4",
        "end-terror",
        "event:unshaded",
        "rally:beneficiary:26july",
        "rally:sierra-maestra:place:4",
        "advance-card",
        "government:assault",
    ] {
        choose(&mut game, action);
    }
    let original = game.observe();
    assert!(
        original
            .choices
            .iter()
            .any(|c| c.action() == "end-assault" && c.supported)
    );

    // Every space's plans are offered at once (#169): looking at one costs no
    // decision, and each plan carries the authority's preview.
    for space in &original.spaces {
        let offered = plans(&original, &space.id);
        assert!(!offered.is_empty(), "{}", space.id);
        assert!(offered.iter().all(|c| c.preview.is_some()), "{}", space.id);
        let arithmetic = match space.id.as_str() {
            "las-villas" => "3 Troops ÷ 2 = 1",
            "pinar-del-rio" => "= 0 in Forest",
            "la-habana" => "= 0 in Grassland",
            "ec-pinar-la-habana" => "= 0 in an Economic Center",
            _ => "",
        };
        assert!(offered[0].explanation.contains(arithmetic), "{}", space.id);
    }

    for (space, balance) in [("sierra-maestra", 3), ("ec-oriente-sierra", 1)] {
        let pending = game.observe();
        // Sierra Maestra's Base stands behind 4 Underground Guerrillas, so no plan
        // there removes anything: the space has the one plan that removes nothing.
        let offered = plans(&pending, space);
        let action = format!("assault:{space}:none");
        assert_eq!(
            offered.iter().map(|c| c.action()).collect::<Vec<_>>(),
            [action.as_str()]
        );
        let choice = offered[0];
        let preview = choice.preview.as_ref().unwrap();
        assert_eq!(preview.resources[0].to, balance);
        assert!(preview.removed.is_empty() && preview.flips.is_empty());
        if space == "sierra-maestra" {
            assert!(choice.explanation.contains("4 Underground"));
            assert!(choice.explanation.contains("protect their faction's Bases"));
        }
        for illegal in [format!("{action}:1"), format!("assault:{space}:26july")] {
            let illegal = format!("{}:{illegal}", pending.revision);
            assert!(
                game.submit(pending.owner, pending.revision, &illegal)
                    .is_err()
            );
        }
        choose(&mut game, &action);
        let settled = game.observe();
        assert_eq!(settled.resources[0], balance);
        assert_eq!(settled.spaces, original.spaces);
        assert_eq!(settled.available, original.available);
        assert!(plans(&settled, space).is_empty());
    }

    let pending = game.observe();
    let payment = pending
        .choices
        .iter()
        .find(|c| c.action() == "assault:havana:syndicate")
        .unwrap();
    assert!(!payment.supported);
    assert!(payment.preview.is_none());
    let error = game
        .submit(pending.owner, pending.revision, &payment.id)
        .unwrap_err();
    assert!(error.contains("Government has 1 Resource and cannot pay 2 Resources"));
    assert_eq!(game.observe(), pending);
    choose(&mut game, "end-assault");
    assert_eq!(game.observe().resources[0], 1);
    assert_eq!(game.observe().owner, Faction::Syndicate);
}

// ---- #169: Assault's space is chosen on the map ----------------------------------
//
// Expected values are worked by hand from R 3.2.4 on the accepted tactical handoff's
// position (`docs/research/tactical-slice-handoff.md`, "Starting position and
// boundary") and the decisions each path takes, not read from the implementation:
// - **Capacity:** 1 piece for each Troops cube; in a City or economic center also 1
//   for each Police; in Mountain 1 for every 2 Troops, rounded down.
// - **What it can affect:** Active Guerrillas; a targeted faction's Bases (Casinos
//   close, R 1.4.5) only once that faction has no Guerrillas in the space.
// - **Cost:** 2 Resources a space at a Firm US Alliance (R 6.3.1).

/// Seed 2's Kidnap roll is 5 (Government 6 → 1, 26July 0 → 5); Santiago's Terror then
/// costs 26July 1 and flips one of its two Underground Guerrillas there to Active.
const TERROR: [&str; 3] = ["kidnap", "kidnap:roll", "terror:santiago"];

/// A space's pieces in the handoff's notation: T Troops, P Police, U Underground and
/// A Active Guerrillas, B Base, C open Casino, X closed Casino, by faction.
fn pieces(view: &Observation, space: &str) -> String {
    let space = view.spaces.iter().find(|s| s.id == space).unwrap();
    let mut factions = vec![];
    for (faction, letter) in [
        (Faction::Government, "G"),
        (Faction::M26, "M"),
        (Faction::Directorio, "D"),
        (Faction::Syndicate, "S"),
    ] {
        let mut groups = vec![];
        for (kind, state, mark) in [
            (Kind::Troops, State::Ordinary, "T"),
            (Kind::Police, State::Ordinary, "P"),
            (Kind::Guerrilla, State::Active, "A"),
            (Kind::Guerrilla, State::Underground, "U"),
            (Kind::Base, State::Ordinary, "B"),
            (Kind::Casino, State::Open, "C"),
            (Kind::Casino, State::Closed, "X"),
        ] {
            let count: u8 = space
                .pieces
                .iter()
                .filter(|p| p.faction == faction && p.kind == kind && p.state == state)
                .map(|p| p.count)
                .sum();
            if count > 0 {
                groups.push(format!("{count}{mark}"));
            }
        }
        if !groups.is_empty() {
            factions.push(format!("{letter}:{}", groups.join(",")));
        }
    }
    factions.join(" ")
}

/// One space of a worked position: its pieces, the state R 3.2.4 gives it, and the
/// plans that remove or closes something there (the plan that removes nothing is
/// offered besides, in every unpaid space).
type Worked = (
    &'static str,
    &'static str,
    SpaceState,
    &'static [&'static str],
);

/// Checks every space of `view` against its worked row: the pieces the row was worked
/// from, the map state, and the plans offered.
fn assert_worked(view: &Observation, worked: &[Worked; 13], supported: bool) {
    let targets = space_targets(view).expect("an open Assault targets the map");
    assert_eq!(targets.len(), 13);
    for ((id, expected_pieces, state, effective), target) in worked.iter().zip(&targets) {
        assert_eq!(target.space, *id);
        assert_eq!(pieces(view, id), *expected_pieces, "{id}");
        assert_eq!(target.state, *state, "{id}");
        let offered = plans(view, id);
        let mut expected: Vec<String> = effective
            .iter()
            .map(|plan| format!("assault:{id}:{plan}"))
            .collect();
        if *state != SpaceState::NotInvolved {
            expected.push(format!("assault:{id}:none"));
        }
        assert_eq!(
            offered.iter().map(|c| c.action()).collect::<Vec<_>>(),
            expected,
            "{id}"
        );
        assert_eq!(
            target.choices,
            offered.iter().map(|c| c.id.clone()).collect::<Vec<_>>()
        );
        for choice in &offered {
            let site = choice.site.as_ref().unwrap();
            // The authority states whether each plan removes or closes a piece.
            assert_eq!(
                site.effect,
                !choice.action().ends_with(":none"),
                "{}",
                choice.id
            );
            assert_eq!(choice.supported, supported, "{}", choice.id);
            assert_eq!(choice.preview.is_some(), supported, "{}", choice.id);
        }
    }
}

const NO_EFFECT: SpaceState = SpaceState::NoEffect;

/// The 11 spaces where no handoff continuation lets Assault affect a piece, with the
/// arithmetic for each. `oriente` and `sierra` are 26July's pieces there on this path.
fn quiet(oriente: &'static str, sierra: &'static str) -> [Worked; 11] {
    [
        // City: 1 Troops + 2 Police = 3, and no enemy piece.
        ("camaguey-city", "G:1T,2P", NO_EFFECT, &[]),
        // Forest: 0 Troops = 0.
        ("pinar-del-rio", "S:1C", NO_EFFECT, &[]),
        // Grassland: 0 Troops = 0.
        ("la-habana", "S:1C", NO_EFFECT, &[]),
        ("matanzas", "", NO_EFFECT, &[]),
        // Mountain: 3 Troops ÷ 2 = 1, and no enemy piece.
        ("las-villas", "G:3T", NO_EFFECT, &[]),
        // Forest: 0 Troops = 0.
        ("camaguey-province", "D:1U", NO_EFFECT, &[]),
        ("oriente", oriente, NO_EFFECT, &[]),
        // Mountain: 0 Troops ÷ 2 = 0.
        ("sierra-maestra", sierra, NO_EFFECT, &[]),
        ("ec-pinar-la-habana", "", NO_EFFECT, &[]),
        ("ec-las-villas-camaguey", "", NO_EFFECT, &[]),
        ("ec-oriente-sierra", "", NO_EFFECT, &[]),
    ]
}

/// A worked position: Havana, Santiago and the 11 quiet spaces, in the map's order.
fn worked(
    havana: Worked,
    santiago: Worked,
    oriente: &'static str,
    sierra: &'static str,
) -> [Worked; 13] {
    let quiet = quiet(oriente, sierra);
    let mut all = vec![havana, quiet[0], santiago];
    all.extend_from_slice(&quiet[1..]);
    all.try_into().unwrap()
}

/// Card 18 with Assault opened, after `event` on card 17 and `give` Resources from
/// 26July to Government during Terror.
fn assault_after(give: u8, event: &[&str]) -> Game {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in TERROR {
        choose(&mut game, action);
    }
    if give > 0 {
        choose(&mut game, "transfer");
        choose(&mut game, &format!("give:0:{give}"));
    }
    choose(&mut game, "end-terror");
    for action in event {
        choose(&mut game, action);
    }
    choose(&mut game, "advance-card");
    choose(&mut game, "government:assault");
    game
}

const SHADED: [&str; 2] = ["event:shaded", "resolve-shaded"];
const RALLY: [&str; 3] = [
    "event:unshaded",
    "rally:beneficiary:26july",
    "rally:sierra-maestra:place:4",
];

#[test]
fn syndicate_pass_finishes_card_18_after_havana_assault() {
    let mut game = assault_after(4, &SHADED);
    choose(&mut game, "assault:havana:syndicate");
    choose(&mut game, "end-assault");
    let before = game.observe();
    assert_eq!(before.owner, Faction::Syndicate);
    assert_eq!(before.next.as_ref().unwrap().role, "2nd Eligible");
    assert_eq!(before.resources[Faction::Syndicate.index()], 15);
    assert!(before.eligibility.event_restriction.is_some());
    let mut reports = Reports::new(&before);

    let pass = before
        .choices
        .iter()
        .find(|choice| choice.action() == "syndicate:pass")
        .unwrap();
    assert!(pass.supported);
    let balance = &pass.preview.as_ref().unwrap().resources[0];
    assert_eq!(
        (balance.faction, balance.from, balance.to),
        (Faction::Syndicate, 15, 16)
    );
    choose(&mut game, pass.action());

    let after = game.observe();
    reports.record(&after);
    let reports = reports.all();
    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert_eq!(report.headline, "Syndicate Passes");
    assert!(report.complete);
    assert_eq!(report.recorded, Some("Pass"));
    assert_eq!(after.resources, [3, 0, 5, 16]);
    assert!(after.passed[Faction::Syndicate.index()]);
    assert_eq!(after.standings[Faction::Syndicate.index()].score, 2);
    assert_eq!(
        after
            .available
            .iter()
            .find(|p| p.kind == Kind::Casino)
            .unwrap()
            .count,
        7
    );
    assert_eq!(after.spaces, before.spaces);
    assert_eq!(after.available, before.available);
    assert_eq!(after.eligible, [false, true, true, true]);
    assert_eq!(
        after
            .eligibility
            .next_card
            .unwrap()
            .map(|entry| entry.eligible),
        [false, true, true, true]
    );
    assert!(after.eligibility.event_restriction.is_none());
    assert_eq!((after.current_card, after.preview_card), (18, 19));
    assert_eq!(after.deck, before.deck);
    assert_eq!(after.played_pile, before.played_pile);
    assert!(after.capabilities.is_empty());
    assert!(after.next.is_none() && after.choices.is_empty());

    assert!(
        game.submit(before.owner, before.revision, &pass.id)
            .is_err()
    );
    assert!(
        game.submit(
            Faction::Syndicate,
            after.revision,
            &format!("{}:syndicate:pass", after.revision),
        )
        .is_err()
    );
    assert_eq!(game.observe(), after);
}

#[test]
fn limited_construct_pays_once_and_distinguishes_the_two_havana_plans() {
    let mut game = assault_after(4, &SHADED);
    choose(&mut game, "assault:havana:syndicate");
    choose(&mut game, "end-assault");
    let handoff = game.observe();
    let casino_counts = |view: &Observation| {
        let havana = view
            .spaces
            .iter()
            .find(|space| space.id == "havana")
            .unwrap();
        let in_havana = |state| {
            havana
                .pieces
                .iter()
                .filter(|piece| piece.kind == Kind::Casino && piece.state == state)
                .map(|piece| piece.count)
                .sum::<u8>()
        };
        assert_eq!(havana.control(), Some(Faction::Government));
        (
            in_havana(State::Open),
            in_havana(State::Closed),
            view.standings[Faction::Syndicate.index()].score,
            view.available
                .iter()
                .find(|piece| piece.kind == Kind::Casino)
                .unwrap()
                .count,
        )
    };
    assert_eq!(handoff.resources[3], 15);
    assert_eq!(casino_counts(&handoff), (0, 1, 2, 7));

    choose(&mut game, "syndicate:limited-operation");
    let open = game.observe();
    assert_eq!(open.executing, Some(Faction::Syndicate));
    let finish = open
        .choices
        .iter()
        .find(|c| c.action() == "end-construct")
        .unwrap();
    assert!(!finish.supported);
    assert!(game.submit(open.owner, open.revision, &finish.id).is_err());
    assert_eq!(game.observe(), open);
    let offered: Vec<_> = plans(&open, "havana")
        .into_iter()
        .filter(|c| c.supported)
        .collect();
    assert_eq!(
        offered.iter().map(|c| c.action()).collect::<Vec<_>>(),
        ["construct:havana:open", "construct:havana:place"],
    );

    for (plan, expected) in offered.into_iter().zip([(1, 0, 3, 7), (0, 2, 2, 6)]) {
        let mut branch = game.clone();
        let price = &plan.preview.as_ref().unwrap().resources[0];
        assert_eq!(
            (price.faction, price.from, price.to),
            (Faction::Syndicate, 15, 10)
        );
        choose(&mut branch, plan.action());
        let paid = branch.observe();
        assert_eq!(paid.resources, [3, 0, 5, 10]);
        assert_eq!(casino_counts(&paid), expected);
        assert!(
            paid.choices
                .iter()
                .all(|c| c.space().is_none() || !c.supported)
        );
        let refusal = branch
            .submit(
                Faction::Syndicate,
                paid.revision,
                &format!("{}:{}", paid.revision, plan.action()),
            )
            .unwrap_err();
        assert!(refusal.contains("one space"), "{refusal}");
        assert_eq!(branch.observe(), paid);
        let another_space = format!("{}:construct:matanzas:place", paid.revision);
        let refusal = branch
            .submit(paid.owner, paid.revision, &another_space)
            .unwrap_err();
        assert!(refusal.contains("one space"), "{refusal}");
        let leave = format!("{}:back-to-choice", paid.revision);
        assert!(branch.submit(paid.owner, paid.revision, &leave).is_err());
        assert_eq!(branch.observe(), paid);
        choose(&mut branch, "end-construct");
        let finished = branch.observe();
        assert_eq!(finished.eligible, [false, true, true, false]);
        assert_eq!((finished.current_card, finished.preview_card), (18, 19));
        assert_eq!(finished.resources, paid.resources);
        assert_eq!(casino_counts(&finished), expected);
        assert!(finished.next.is_none() && finished.choices.is_empty());
    }
}

#[test]
fn an_open_assault_offers_every_unpaid_space_s_plans_at_once_and_no_menu() {
    let game = assault_after(4, &RALLY);
    let view = game.observe();
    assert_eq!(view.resources, [5, 0, 5, 15]);
    assert_eq!(
        view.choices.iter().map(|c| c.action()).collect::<Vec<_>>(),
        [
            "assault:havana:syndicate",
            "assault:havana:none",
            "assault:camaguey-city:none",
            "assault:santiago:26july",
            "assault:santiago:none",
            "assault:pinar-del-rio:none",
            "assault:la-habana:none",
            "assault:matanzas:none",
            "assault:las-villas:none",
            "assault:camaguey-province:none",
            "assault:oriente:none",
            "assault:sierra-maestra:none",
            "assault:ec-pinar-la-habana:none",
            "assault:ec-las-villas-camaguey:none",
            "assault:ec-oriente-sierra:none",
            "end-assault",
            "transfer",
            "remove",
        ]
    );
    // No area, space-list or Back choice exists, and none is accepted.
    for gone in [
        "assault:areas:cities",
        "assault:areas:provinces",
        "assault:areas:economic-centers",
        "assault:space:santiago",
        "assault:back",
    ] {
        let mut probe = game.clone();
        assert!(
            probe
                .submit(
                    view.owner,
                    view.revision,
                    &format!("{}:{gone}", view.revision)
                )
                .is_err(),
            "{gone}"
        );
        assert_eq!(probe, game);
    }
    // The match stores no browse position: an open Assault records only what was
    // paid and whether it finished.
    let saved = serde_json::to_value(&game).unwrap();
    assert_eq!(
        saved["assault"],
        serde_json::json!({ "paid": [], "finished": false })
    );
}

#[test]
fn each_of_the_thirteen_spaces_has_the_state_r_3_2_4_gives_its_pieces() {
    // After the unshaded Event's free Rally (4 Guerrillas placed in Sierra Maestra),
    // Government has 5 Resources: 6 − 5 (Kidnap) + 4 (given).
    // - Havana, City: 6 Troops + 4 Police = 10. Directorio's 2 Guerrillas are
    //   Underground; Syndicate has no Guerrillas there, so its open Casino closes.
    // - Santiago, City: 2 Troops + 1 Police = 3. 26July has 1 Active Guerrilla (the
    //   one Terror flipped) and 1 Underground: the Active one is removed.
    let havana: Worked = (
        "havana",
        "G:6T,4P D:2U S:1C",
        SpaceState::Selectable,
        &["syndicate"],
    );
    let rally = assault_after(4, &RALLY);
    assert_eq!(rally.observe().resources, [5, 0, 5, 15]);
    assert_worked(
        &rally.observe(),
        &worked(
            havana,
            (
                "santiago",
                "G:2T,1P M:1A,1U",
                SpaceState::Selectable,
                &["26july"],
            ),
            "M:1U",
            "M:4U,1B",
        ),
        true,
    );

    // After the shaded Event every 26July Guerrilla is Active. Santiago's 2 are both
    // removed (capacity 3). Oriente's is Active too, but Oriente holds no Troops:
    // 0 Troops = 0 in Forest.
    let shaded = assault_after(4, &SHADED);
    let santiago: Worked = (
        "santiago",
        "G:2T,1P M:2A",
        SpaceState::Selectable,
        &["26july"],
    );
    assert_worked(
        &shaded.observe(),
        &worked(havana, santiago, "M:1A", "M:1B"),
        true,
    );

    // With nothing given, Government has 1 Resource and a space costs 2: Havana and
    // Santiago are blocked, and a Resource transfer is the remedy. The other 11 would
    // remove nothing, so they stay no effect.
    let mut short = assault_after(0, &SHADED);
    let view = short.observe();
    assert_eq!(view.resources, [1, 4, 5, 15]);
    let blocked = |space: Worked| (space.0, space.1, SpaceState::Blocked, space.3);
    assert_worked(
        &view,
        &worked(blocked(havana), blocked(santiago), "M:1A", "M:1B"),
        false,
    );
    for choice in view.choices.iter().filter(|c| c.site.is_some()) {
        assert!(
            choice
                .explanation
                .starts_with("Government has 1 Resource and cannot pay 2 Resources (R 6.3.1)."),
            "{}",
            choice.explanation
        );
        assert_eq!(choice.remedies, ["Request a Resource transfer"]);
    }

    // A Resource transfer under way suspends the targeting, and the transfer turns
    // the blocked spaces selectable at once: 26July gives 1, Government has 2.
    choose(&mut short, "transfer");
    assert_eq!(space_targets(&short.observe()), None);
    choose(&mut short, "ask:1:1");
    assert_eq!(space_targets(&short.observe()), None);
    choose(&mut short, "authorize:1");
    let funded = short.observe();
    assert_eq!(funded.resources, [2, 3, 5, 15]);
    assert_worked(&funded, &worked(havana, santiago, "M:1A", "M:1B"), true);

    // Paying resolves Santiago at once and takes its choices: the space is not
    // involved for the rest of the Operation (R 3.1). Government is at 0 again, so
    // Havana is blocked.
    choose(&mut short, "assault:santiago:26july");
    let paid = short.observe();
    assert_eq!(paid.resources, [0, 3, 5, 15]);
    assert_worked(
        &paid,
        &worked(
            blocked(havana),
            ("santiago", "G:2T,1P", SpaceState::NotInvolved, &[]),
            "M:1A",
            "M:1B",
        ),
        false,
    );
}

#[test]
fn the_map_states_follow_a_voluntary_removal_at_once() {
    let mut game = assault_after(4, &SHADED);
    // Removal mode suspends the targeting, as it does during Terror.
    choose(&mut game, "remove");
    assert_eq!(space_targets(&game.observe()), None);
    // Santiago, City: 2 Troops + 1 Police = 3. Without them, 0 + 0 = 0, and the 2
    // Active Guerrillas stay.
    choose(&mut game, "remove:santiago:troops:2");
    choose(&mut game, "remove:santiago:police:1");
    choose(&mut game, "done-removal");
    let view = game.observe();
    let santiago = space_targets(&view)
        .unwrap()
        .into_iter()
        .find(|t| t.space == "santiago")
        .unwrap();
    assert_eq!(pieces(&view, "santiago"), "M:2A");
    assert_eq!(santiago.state, SpaceState::NoEffect);
    let only = plans(&view, "santiago");
    assert_eq!(only.len(), 1);
    assert_eq!(only[0].action(), "assault:santiago:none");
    assert!(
        only[0]
            .explanation
            .contains("0 Troops + 0 Police = 0 in a City")
    );
}

#[test]
fn the_plan_that_removes_nothing_costs_the_space_s_price_and_changes_nothing_else() {
    let mut game = assault_after(4, &SHADED);
    let before = game.observe();
    assert_eq!(before.resources, [5, 0, 5, 15]);
    // Santiago holds 2 Active 26July Guerrillas that another plan would remove; the
    // plan that removes nothing leaves them. Matanzas is empty.
    for (space, balance) in [("santiago", 3), ("matanzas", 1)] {
        let pending = game.observe();
        let action = format!("assault:{space}:none");
        let choice = pending
            .choices
            .iter()
            .find(|c| c.action() == action)
            .unwrap();
        assert_eq!(choice.label, "Removes nothing");
        assert_eq!(
            choice.site,
            Some(cuba_libre_unfolded::Site {
                space: space.into(),
                effect: false,
                unpayable: false
            })
        );
        let preview = choice.preview.as_ref().unwrap();
        assert_eq!(preview.resources.len(), 1);
        assert_eq!(
            (preview.resources[0].faction, preview.resources[0].to),
            (Faction::Government, balance)
        );
        assert!(preview.removed.is_empty() && preview.flips.is_empty());
        assert!(preview.control.is_empty() && preview.standings.is_empty());
        assert_eq!(
            preview.assault.as_ref().unwrap().effect,
            "R 3.2.4: No pieces affected."
        );
        choose(&mut game, &action);
        let after = game.observe();
        assert_eq!(after.resources, [balance, 0, 5, 15]);
        assert_eq!(after.spaces, before.spaces);
        assert_eq!(after.available, before.available);
        assert_eq!(after.standings, before.standings);
        assert_eq!(after.executing, Some(Faction::Government));
        // The paid space is not involved, whatever stands there (R 3.1).
        let target = space_targets(&after)
            .unwrap()
            .into_iter()
            .find(|t| t.space == space)
            .unwrap();
        assert_eq!(target.state, SpaceState::NotInvolved);
        assert!(target.choices.is_empty());
    }
}

#[test]
fn each_paid_space_is_recorded_in_the_order_paid_with_what_it_removed_or_closed() {
    let mut game = assault_after(4, &SHADED);
    assert!(game.observe().assault_paid.is_empty());
    // Santiago first, though Havana comes first on the map: 2 Active 26July
    // Guerrillas return. Then Havana: Syndicate's Casino closes (R 1.4.5).
    choose(&mut game, "assault:santiago:26july");
    choose(&mut game, "assault:havana:syndicate");
    let view = game.observe();
    assert_eq!(view.resources, [1, 0, 5, 15]);
    assert_eq!(
        view.assault_paid
            .iter()
            .map(|paid| (
                paid.space.as_str(),
                paid.paid,
                paid.returned,
                paid.summary()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "santiago",
                2,
                2,
                "Santiago de Cuba · paid 2 Resources · returned 2 26July Guerrillas".to_owned()
            ),
            (
                "havana",
                2,
                0,
                "Havana · paid 2 Resources · closed 1 Casino".to_owned()
            ),
        ]
    );
    // A third space cannot be paid with 1 Resource; the record keeps both.
    let matanzas = view
        .choices
        .iter()
        .find(|c| c.action() == "assault:matanzas:none")
        .unwrap();
    assert!(!matanzas.supported);
    // The preview of the second payment said that no other space could be paid after it.
    let mut again = assault_after(4, &SHADED);
    choose(&mut again, "assault:santiago:26july");
    let havana = again.observe();
    let preview = havana
        .choices
        .iter()
        .find(|c| c.action() == "assault:havana:syndicate")
        .unwrap()
        .preview
        .clone()
        .unwrap();
    // Each part is stated by name, so none depends on the notes' order.
    assert_eq!(
        preview.assault,
        Some(AssaultPreview {
            arithmetic: "6 Troops + 4 Police = 10 in a City (R 3.2.4).".into(),
            affects: "Syndicate: 0 Active, 0 Underground, 1 open Casino. Directorio: 0 Active, 2 Underground. Assault cannot remove its Underground Guerrillas until they are Activated.".into(),
            effect: "R 3.2.4: Close 1 Casino in place (R 1.4.5).".into(),
        })
    );
    assert_eq!(
        preview.notes,
        [
            "Government would have 1 Resource: another space would need a Resource transfer first (R 6.3.1)."
        ]
    );
    assert!(preview.unavailable.is_empty());
    // The record survives the ending.
    choose(&mut game, "end-assault");
    assert_eq!(game.observe().assault_paid, view.assault_paid);
}

/// The reason the rules authority gives for each space, in the map's order, or `None`.
fn reasons(view: &Observation) -> Vec<(String, Option<String>)> {
    view.spaces
        .iter()
        .map(|space| {
            let reason = view
                .space_reasons
                .iter()
                .find(|r| r.space == space.id)
                .map(|r| r.reason.clone());
            (space.id.clone(), reason)
        })
        .collect()
}

/// The 13 spaces' expected reasons, in the map's order: Havana and Santiago, then the
/// 11 spaces where Assault would remove nothing.
fn expected_reasons(havana: Option<&str>, santiago: Option<&str>) -> Vec<(String, Option<String>)> {
    // Worked from R 3.2.4 on the pieces `quiet` lists, independently of the plans:
    // - no enemy piece in Camagüey City (1 Troops, 2 Police), Matanzas, Las Villas
    //   (3 Troops) or any of the three Economic Centers;
    // - no Troops in Pinar del Río (Syndicate's open Casino) and La Habana (Forest and
    //   Grassland count Troops only, and no Police stands there either), Camagüey
    //   (Directorio's Underground Guerrilla), Oriente and Sierra Maestra (26July's
    //   pieces; Sierra Maestra is Mountain, where 0 Troops ÷ 2 = 0).
    let enemy = Some("No enemy pieces (R 3.2.4)");
    let troops = Some("No Troops (R 3.2.4)");
    [
        ("havana", havana),
        ("camaguey-city", enemy),
        ("santiago", santiago),
        ("pinar-del-rio", troops),
        ("la-habana", troops),
        ("matanzas", enemy),
        ("las-villas", enemy),
        ("camaguey-province", troops),
        ("oriente", troops),
        ("sierra-maestra", troops),
        ("ec-pinar-la-habana", enemy),
        ("ec-las-villas-camaguey", enemy),
        ("ec-oriente-sierra", enemy),
    ]
    .into_iter()
    .map(|(space, reason)| (space.to_owned(), reason.map(str::to_owned)))
    .collect()
}

/// #170: the space tag's line is the rules authority's reason for each space's state,
/// checked for all 13 spaces in each of the handoff's Assault positions. A selectable
/// space says what its best plan would do (#180); a blocked one names the Resource transfer that would open it
/// (R 6.3.1, 1.5.1); a paid one says what it paid (R 3.1); every other one says why
/// R 3.2.4 would remove nothing there.
#[test]
fn each_of_the_thirteen_spaces_has_the_reason_r_3_2_4_gives_its_state() {
    // Government has 5 Resources, so Havana and Santiago are selectable. Havana's open
    // Casino would close (its capacity is 10 and Directorio's Guerrillas are
    // Underground). Santiago's capacity is 3: the shaded Event leaves its 2 Guerrillas
    // Active, and without it Terror has flipped 1 of them.
    let casino = Some("Can close 1 Casino");
    let both = Some("Can remove 2 Guerrillas");
    for (event, santiago) in [
        (&RALLY[..], Some("Can remove 1 Guerrilla")),
        (&SHADED[..], both),
    ] {
        let view = assault_after(4, event).observe();
        assert_eq!(
            reasons(&view),
            expected_reasons(casino, santiago),
            "{event:?}"
        );
    }
    // With 1 Resource Government cannot pay 2, and 26July, Directorio and Syndicate
    // have Resources to give.
    let transfer = Some("A Resource transfer opens it");
    let mut short = assault_after(0, &SHADED);
    assert_eq!(
        reasons(&short.observe()),
        expected_reasons(transfer, transfer)
    );
    // No targeting while the transfer is under way, so no reason either.
    choose(&mut short, "transfer");
    assert!(short.observe().space_reasons.is_empty());
    choose(&mut short, "ask:1:1");
    choose(&mut short, "authorize:1");
    assert_eq!(reasons(&short.observe()), expected_reasons(casino, both));
    // Santiago paid 2 (Firm, R 6.3.1); Government is at 0 again.
    choose(&mut short, "assault:santiago:26july");
    assert_eq!(
        reasons(&short.observe()),
        expected_reasons(transfer, Some("Assaulted · paid 2 Resources"))
    );
    // Once the Assault is finished, no choice acts on a space.
    choose(&mut short, "end-assault");
    assert!(short.observe().space_reasons.is_empty());
    // Every line fits the tag's one line (#170's measured budget).
    for (_, reason) in reasons(&assault_after(0, &SHADED).observe()) {
        assert!(reason.is_none_or(|r| r.chars().count() <= 33));
    }
}
