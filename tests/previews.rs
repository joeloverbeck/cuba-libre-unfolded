//! #62: every irreversible step carries a preview computed by the rules authority, a
//! Terror target that cannot be acted on now is shown disabled with its reason and
//! in-task remedies, and End Terror replaces both Finish choices.
//!
//! Expected values come from the accepted deacd635 tactical handoff's worked
//! arithmetic and the rules, never from the engine: 26July's standing is Opposition
//! `Matanzas 1 + Camagüey Province 1 + Oriente 2 + Sierra Maestra 2 = 6` plus 1 Base,
//! 7, margin `7 − 15 = −8` (handoff "standings"). Santiago Terror gives 8/−7 (Pop 1,
//! Neutral → Passive Opposition); Oriente Terror adds 2 (Pop 2, Passive → Active
//! Opposition). Removing the Base costs 1 and leaves Sierra Maestra uncontrolled;
//! removing a Santiago Guerrilla leaves 1, which does not exceed 1 Police (R 4.3.3).
use cuba_libre_unfolded::{Choice, Faction, Game, Kind, Preview, SequenceBox, State};

const SHADED: &str = "All 26July Guerrillas Active. 26July Ineligible through next card.";

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    let id = format!("{}:{action}", view.revision);
    let owner = view
        .choices
        .iter()
        .find(|c| c.id == id)
        .map_or(view.owner, |c| c.owner);
    game.submit(owner, view.revision, &id)
        .unwrap_or_else(|e| panic!("{id}: {e}"));
}

fn transfer(game: &mut Game, giver: Faction, recipient: Faction, amount: u8) {
    choose(game, "transfer");
    if giver == Faction::M26 {
        choose(game, &format!("give:{}:{amount}", recipient.index()));
    } else {
        choose(game, &format!("ask:{}:{amount}", giver.index()));
        choose(game, &format!("authorize:{amount}"));
    }
}

fn remove(game: &mut Game, target: &str) {
    choose(game, "remove");
    choose(game, &format!("remove:{target}:1"));
    choose(game, "done-removal");
}

fn choice(game: &Game, action: &str) -> Choice {
    let view = game.observe();
    let id = format!("{}:{action}", view.revision);
    view.choices
        .into_iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("{id} is not listed"))
}

fn preview(game: &Game, action: &str) -> Preview {
    choice(game, action)
        .preview
        .unwrap_or_else(|| panic!("{action} has no preview"))
}

/// (score from, score to, margin from, margin to) for one faction, if it changes.
fn standing(preview: &Preview, faction: Faction) -> Option<[i16; 4]> {
    preview
        .standings
        .iter()
        .find(|s| s.faction == faction)
        .map(|s| [s.score_from, s.score_to, s.margin_from, s.margin_to])
}

fn unavailable(preview: &Preview) -> Vec<&str> {
    preview
        .unavailable
        .iter()
        .map(|u| u.choice.as_str())
        .collect()
}

fn funded() -> Game {
    let mut game = Game::prepared();
    transfer(&mut game, Faction::Directorio, Faction::M26, 1);
    game
}

#[test]
fn santiago_terror_previews_its_cost_flip_shift_and_standing_7_to_8() {
    let game = funded();
    let p = preview(&game, "terror:santiago");
    let resources: Vec<_> = p
        .resources
        .iter()
        .map(|r| (r.faction, r.from, r.to))
        .collect();
    assert_eq!(resources, [(Faction::M26, 1, 0)]);
    let flips: Vec<_> = p
        .flips
        .iter()
        .map(|f| (f.space.as_str(), f.faction, f.kind, f.from, f.to, f.count))
        .collect();
    assert_eq!(
        flips,
        [(
            "santiago",
            Faction::M26,
            Kind::Guerrilla,
            State::Underground,
            State::Active,
            1
        )]
    );
    assert_eq!(p.terror, ["santiago"]);
    let shifts: Vec<_> = p
        .support
        .iter()
        .map(|s| (s.space.as_str(), s.from, s.to, s.label.as_str()))
        .collect();
    assert_eq!(
        shifts,
        [("santiago", 0, -1, "Neutral → Passive Opposition")]
    );
    // Activation keeps the piece on the map, so Government still Controls Santiago.
    assert!(p.control.is_empty(), "{:?}", p.control);
    assert!(p.removed.is_empty());
    assert_eq!(standing(&p, Faction::M26), Some([7, 8, -8, -7]));
    assert_eq!(p.standings.len(), 1, "{:?}", p.standings);
    // The last Resource pays for it, so Oriente can no longer be afforded.
    assert_eq!(unavailable(&p), ["terror:oriente"]);
    assert_eq!((p.records, &p.next), (None, &None));
}

#[test]
fn oriente_terror_previews_a_shift_to_active_opposition_and_standing_plus_2() {
    let p = preview(&funded(), "terror:oriente");
    assert_eq!(standing(&p, Faction::M26), Some([7, 9, -8, -6]));
    let shifts: Vec<_> = p
        .support
        .iter()
        .map(|s| (s.space.as_str(), s.label.as_str()))
        .collect();
    assert_eq!(
        shifts,
        [("oriente", "Passive Opposition → Active Opposition")]
    );
    assert_eq!(p.flips.len(), 1);
    assert_eq!(p.flips[0].space, "oriente");
}

#[test]
fn removing_a_santiago_guerrilla_before_kidnap_previews_kidnap_becoming_unavailable() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let p = preview(&game, "remove:santiago:underground:1");
    let removed: Vec<_> = p
        .removed
        .iter()
        .map(|r| (r.space.as_str(), r.faction, r.kind, r.state, r.count))
        .collect();
    assert_eq!(
        removed,
        [(
            "santiago",
            Faction::M26,
            Kind::Guerrilla,
            State::Underground,
            1
        )]
    );
    // Government 3 against 26July 1 still Controls; no Opposition changes.
    assert!(p.control.is_empty());
    assert!(p.standings.is_empty());
    let kidnap = p.unavailable.iter().find(|u| u.choice == "kidnap").unwrap();
    assert!(kidnap.reason.contains("1 Police"), "{}", kidnap.reason);
    // A removal is a real effect, so going back to the action choice ends too.
    assert!(unavailable(&p).contains(&"back-to-choice"));
    // Removal mode stays open after it; the consequences are the same.
    choose(&mut game, "remove:santiago:underground:1");
    assert_eq!(
        game.observe().removal,
        Some(cuba_libre_unfolded::Removal::Selecting)
    );
}

#[test]
fn removing_the_base_previews_standing_minus_1_and_sierra_maestra_uncontrolled() {
    let mut game = Game::prepared();
    choose(&mut game, "remove");
    let p = preview(&game, "remove:sierra-maestra:base:1");
    assert_eq!(standing(&p, Faction::M26), Some([7, 6, -8, -9]));
    let control: Vec<_> = p
        .control
        .iter()
        .map(|c| (c.space.as_str(), c.from, c.to))
        .collect();
    assert_eq!(control, [("sierra-maestra", Some(Faction::M26), None)]);
    assert_eq!(p.removed.len(), 1);
    assert_eq!(p.removed[0].kind, Kind::Base);
}

fn target(game: &Game, space: &str) -> Choice {
    choice(game, &format!("terror:{space}"))
}

#[test]
fn at_0_resources_both_targets_are_disabled_with_their_in_task_remedies() {
    let game = Game::prepared();
    let santiago = target(&game, "santiago");
    assert!(!santiago.supported);
    assert!(
        santiago
            .explanation
            .contains("Cost: 1 Resource per space (26July has 0)"),
        "{}",
        santiago.explanation
    );
    assert_eq!(
        santiago.remedies,
        ["Kidnap here first", "Request a Resource transfer"]
    );
    let oriente = target(&game, "oriente");
    assert!(!oriente.supported);
    assert_eq!(oriente.remedies, ["Request a Resource transfer"]);
    // A disabled target is refused with its reason and changes nothing.
    let mut refused = game.clone();
    let error = refused
        .submit(Faction::M26, 0, "0:terror:santiago")
        .unwrap_err();
    assert_eq!(error, santiago.explanation);
    assert_eq!(refused, game);
}

#[test]
fn after_a_transfer_funds_one_terror_the_other_target_is_disabled_again() {
    let mut game = funded();
    assert!(target(&game, "santiago").supported);
    assert!(target(&game, "oriente").supported);
    assert!(target(&game, "santiago").remedies.is_empty());
    choose(&mut game, "terror:oriente");
    let santiago = target(&game, "santiago");
    assert!(!santiago.supported);
    assert_eq!(
        santiago.remedies,
        ["Kidnap here first", "Request a Resource transfer"]
    );
    let oriente = target(&game, "oriente");
    assert!(!oriente.supported);
    assert!(
        oriente.explanation.contains("already"),
        "{}",
        oriente.explanation
    );
    assert!(oriente.remedies.is_empty());
}

#[test]
fn kidnap_is_no_remedy_when_government_has_nothing_to_take() {
    // Government gives 26July all 6, and 26July gives them on to Directorio: Kidnap
    // would take nothing and cannot guarantee Santiago's payment.
    let mut game = Game::prepared();
    transfer(&mut game, Faction::Government, Faction::M26, 6);
    transfer(&mut game, Faction::M26, Faction::Directorio, 6);
    assert_eq!(game.observe().resources, [0, 0, 11, 15]);
    assert_eq!(
        target(&game, "santiago").remedies,
        ["Request a Resource transfer"]
    );
}

#[test]
fn a_target_lost_to_removal_has_no_remedy() {
    let mut game = Game::prepared();
    remove(&mut game, "santiago:underground");
    remove(&mut game, "santiago:underground");
    remove(&mut game, "oriente:underground");
    for space in ["santiago", "oriente"] {
        let target = target(&game, space);
        assert!(!target.supported, "{space}");
        assert!(
            target
                .explanation
                .contains("no Underground 26July Guerrilla"),
            "{}",
            target.explanation
        );
        assert!(target.remedies.is_empty(), "{space}: {:?}", target.remedies);
    }
}

#[test]
fn while_santiago_terror_is_owed_oriente_is_disabled_by_the_reserve() {
    let mut game = Game::prepared_with_seed(19); // actual shared roll 1: 26July 0 → 1
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    assert_eq!(game.observe().resources[1], 1);
    assert!(target(&game, "santiago").supported);
    let oriente = target(&game, "oriente");
    assert!(!oriente.supported);
    assert!(
        oriente.explanation.contains("accepted commitment policy"),
        "{}",
        oriente.explanation
    );
    assert_eq!(oriente.remedies, ["Request a Resource transfer"]);
    let end = choice(&game, "end-terror");
    assert!(!end.supported);
    assert!(
        end.explanation.contains("accepted commitment policy"),
        "{}",
        end.explanation
    );
    assert_eq!(end.preview, None);
}

/// #75: the refused End Terror names the in-task step that makes it available, so
/// the panel never says nothing here can (#58 story 53). The commitment policy keeps
/// Santiago's payment and Underground Guerrilla, so that step is always offered.
#[test]
fn while_santiago_terror_is_owed_end_terror_names_resolving_it_as_the_remedy() {
    let mut game = Game::prepared_with_seed(19); // actual shared roll 1: 26July 0 → 1
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    let end = choice(&game, "end-terror");
    assert!(!end.supported);
    assert_eq!(end.remedies, ["Resolve Santiago's Terror"]);
    assert!(target(&game, "santiago").supported);
    // The same refusal, offered as 26July's withdrawal beside a pending request.
    transfer_request(&mut game, Faction::Syndicate, 1);
    let withdraw = choice(&game, "end-terror");
    assert!(!withdraw.supported);
    assert_eq!(withdraw.remedies, ["Resolve Santiago's Terror"]);
    choose(&mut game, "decline");
    choose(&mut game, "terror:santiago");
    let end = choice(&game, "end-terror");
    assert!(end.supported);
    assert!(end.remedies.is_empty());
}

fn transfer_request(game: &mut Game, giver: Faction, amount: u8) {
    choose(game, "transfer");
    choose(game, &format!("ask:{}:{amount}", giver.index()));
}

#[test]
fn end_terror_records_operation_only_and_directorio_may_pass_or_take_a_limited_operation() {
    let mut game = funded();
    choose(&mut game, "terror:oriente");
    let end = choice(&game, "end-terror");
    assert_eq!(end.label, "End Terror");
    let p = end.preview.unwrap();
    assert_eq!(p.records, Some(SequenceBox::OperationOnly));
    let next = p.next.as_ref().unwrap();
    assert_eq!(
        (next.faction, next.role.as_str()),
        (Faction::Directorio, "2nd Eligible")
    );
    assert_eq!(
        next.options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    assert!(!p.notes.join(" ").contains(SHADED));
    assert!(p.resources.is_empty() && p.standings.is_empty());
    choose(&mut game, "end-terror");
    let view = game.observe();
    assert_eq!(view.eligibility.m26_box, p.records);
    assert_eq!(view.next_options, next.options);
}

#[test]
fn end_terror_after_kidnap_records_the_special_activity_and_names_card_17_shaded() {
    let mut game = Game::prepared_with_seed(5); // actual shared roll 3
    choose(&mut game, "kidnap");
    choose(&mut game, "kidnap:roll");
    choose(&mut game, "terror:santiago");
    let p = preview(&game, "end-terror");
    assert_eq!(p.records, Some(SequenceBox::OperationWithSpecialActivity));
    let next = p.next.as_ref().unwrap();
    assert_eq!(
        next.options,
        ["Pass (+1 if executed later)", "Limited Operation", "Event"]
    );
    let notes = p.notes.join(" ");
    assert!(notes.contains(SHADED), "{notes}");
    assert!(notes.contains("R 2.3.4, 5.2"), "{notes}");
    choose(&mut game, "end-terror");
    assert_eq!(game.observe().eligibility.m26_box, p.records);
}

#[test]
fn zero_space_end_terror_warns_and_offers_going_back_only_before_an_effect() {
    let game = Game::prepared();
    let p = preview(&game, "end-terror");
    assert_eq!(p.records, Some(SequenceBox::OperationOnly));
    let notes = p.notes.join(" ");
    for fact in [
        "No Terror resolved",
        "no effect",
        "no Pass income",
        "26July becomes Ineligible after this card",
    ] {
        assert!(notes.contains(fact), "{fact}: {notes}");
    }
    assert!(notes.contains("Back to the action choice"), "{notes}");
    assert!(choice(&game, "back-to-choice").supported);
    assert!(p.resources.is_empty() && p.standings.is_empty() && p.terror.is_empty());
    // After a removal something has happened: the warning stays, going back does not.
    let mut removed = Game::prepared();
    remove(&mut removed, "oriente:underground");
    let notes = preview(&removed, "end-terror").notes.join(" ");
    assert!(notes.contains("No Terror resolved"), "{notes}");
    assert!(!notes.contains("Back to the action choice"), "{notes}");
}

#[test]
fn pass_previews_one_resource_and_directorio_first_eligible() {
    let p = preview(&Game::opening(), "pass");
    let resources: Vec<_> = p
        .resources
        .iter()
        .map(|r| (r.faction, r.from, r.to))
        .collect();
    assert_eq!(resources, [(Faction::M26, 0, 1)]);
    assert_eq!(p.records, Some(SequenceBox::Pass));
    let next = p.next.unwrap();
    assert_eq!(
        (next.faction, next.role.as_str()),
        (Faction::Directorio, "1st Eligible")
    );
    assert_eq!(
        next.options,
        [
            "Pass (+1 if executed later)",
            "Operation (with or without a Special Activity)",
            "Event"
        ]
    );
    assert!(p.notes.join(" ").contains("stays Eligible"));
}

#[test]
fn no_old_finish_labels_remain() {
    for game in [Game::prepared(), funded()] {
        for c in game.observe().choices {
            assert!(!c.label.contains("Finish"), "{}", c.label);
            assert!(!c.label.contains("Full Terror"), "{}", c.label);
        }
    }
}

#[test]
fn reading_every_preview_changes_neither_the_state_nor_the_randomness() {
    let mut owed = Game::prepared_with_seed(19);
    choose(&mut owed, "kidnap");
    choose(&mut owed, "kidnap:roll");
    let mut removing = Game::prepared();
    choose(&mut removing, "remove");
    let mut confirming = removing.clone();
    choose(&mut confirming, "remove:santiago:underground:1");
    let mut opened = Game::prepared();
    choose(&mut opened, "kidnap");
    for game in [
        Game::opening(),
        Game::prepared(),
        funded(),
        owed,
        removing,
        confirming,
        opened,
    ] {
        let before = game.clone();
        let random = game.random_state();
        let view = game.observe();
        assert!(view.choices.iter().any(|c| c.preview.is_some()));
        assert_eq!(game, before);
        assert_eq!(game.random_state(), random);
        // Kidnap's roll is never applied to a copy (#63): opening it and its roll show
        // only the stakes, never a Resource result.
        assert!(
            view.choices
                .iter()
                .filter(|c| c.id.ends_with(":kidnap") || c.id.ends_with(":kidnap:roll"))
                .filter_map(|c| c.preview.as_ref())
                .all(|p| p.kidnap.is_some() && p.resources.is_empty())
        );
    }
}

#[test]
fn committing_a_previewed_step_produces_the_previewed_resources_and_standings() {
    let game = funded();
    for action in ["terror:santiago", "terror:oriente", "end-terror"] {
        let p = preview(&game, action);
        let mut committed = game.clone();
        choose(&mut committed, action);
        let view = committed.observe();
        for r in &p.resources {
            assert_eq!(view.resources[r.faction.index()], r.to, "{action}");
        }
        for s in &p.standings {
            let actual = &view.standings[s.faction.index()];
            assert_eq!((actual.score, actual.margin), (s.score_to, s.margin_to));
        }
    }
}

/// #84: the observation says whether the first effect has happened, so a step that
/// waits on an open Kidnap can say whether going back is still possible afterwards.
/// Opening Kidnap or the transfer, or asking, is no effect; an authorized transfer is
/// (local policy, #58).
#[test]
fn the_observation_reports_the_first_effect() {
    let mut game = Game::prepared();
    assert!(!game.observe().effected);
    choose(&mut game, "kidnap");
    assert!(!game.observe().effected, "opening Kidnap rolls nothing");
    choose(&mut game, "kidnap:cancel");
    choose(&mut game, "transfer");
    choose(&mut game, &format!("ask:{}:1", Faction::Directorio.index()));
    assert!(!game.observe().effected, "a request moves nothing");
    choose(&mut game, "authorize:1");
    assert!(
        game.observe().effected,
        "an authorized transfer is an effect"
    );
    choose(&mut game, "terror:santiago");
    assert!(game.observe().effected);
}
