//! #139: what the card tag reads about the current and the preview card, from the
//! rules model. Every expectation here was worked before the model from the March
//! 2019 Living Rules and the supplied faces of cards 17, 18 and 19:
//!
//! - Faces: 17 *Alberto Bayo* and 18 *Pact of Caracas* print 26July, DR, Government,
//!   Syndicate; 19 *Sierra Maestra Manifesto* prints 26July, DR, Syndicate, Government.
//!   17 has an unshaded and a shaded text (dual use, R 5.2). 18 has one text in an
//!   "Insurgent Capability" box (R 5.3). 19 has one text and no lasting-effect box.
//! - R 2.3.2–2.3.4: the Event is an option of the 1st Eligible, and of the 2nd only after
//!   the 1st's Operation with a Special Activity. After an Operation Only the 2nd may
//!   take only a Limited Operation. R 2.3.3: a Pass hands the 1st Eligible's role to the
//!   next Eligible faction in card order.
//! - R 2.3.8: the preview is the draw deck's top card, played after the current card.
use cuba_libre_unfolded::{
    CardFacts, EventKind, EventStatus, EventText, Faction, Game, LastingEffect, Role,
};

use Faction::{Directorio, Government, M26, Syndicate};

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

fn current(game: &Game) -> CardFacts {
    game.observe().cards.current
}
fn preview(game: &Game) -> CardFacts {
    game.observe().cards.preview
}

/// 26July's Terror with Kidnap, funded as #130 funds it (seed 2), up to the moment
/// Directorio decides as 2nd Eligible.
fn handoff() -> Game {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in ["kidnap", "kidnap:roll", "terror:santiago", "transfer"] {
        choose(&mut game, action);
    }
    choose(&mut game, "give:0:1");
    choose(&mut game, "end-terror");
    game
}

#[test]
fn card_17_states_its_face_and_the_opening_option() {
    let game = Game::opening();
    let card = current(&game);
    assert_eq!((card.number, card.title), (17, "Alberto Bayo"));
    assert_eq!(card.order, [M26, Directorio, Government, Syndicate]);
    assert_eq!(card.kind, EventKind::DualUse);
    assert_eq!(card.lasting, None);
    assert_eq!(
        card.status,
        EventStatus::Open {
            faction: M26,
            role: Role::First
        }
    );
    assert_eq!(card.rules, ["R 2.3.4", "R 5.2"]);
    assert_eq!(
        card.words.event,
        "Dual-use: the executing faction chooses unshaded or shaded"
    );
    assert_eq!(card.words.status, "Event open to 26July as 1st Eligible");
    assert_eq!(card.words.lasting, "None");
    assert_eq!(card.words.rules, "R 2.3.4 · R 5.2");
}

#[test]
fn card_18_in_preview_is_a_capability_not_yet_open() {
    for game in [Game::opening(), handoff()] {
        let card = preview(&game);
        assert_eq!((card.number, card.title), (18, "Pact of Caracas"));
        assert_eq!(card.order, [M26, Directorio, Government, Syndicate]);
        assert_eq!(card.kind, EventKind::Single);
        assert_eq!(
            card.lasting,
            Some(LastingEffect::InsurgentCapability { in_effect: false })
        );
        assert_eq!(card.status, EventStatus::Preview { after: 17 });
        assert_eq!(card.rules, ["R 2.3.8", "R 5.1", "R 5.3"]);
        assert_eq!(
            card.words.event,
            "Single text: the executing faction carries it out"
        );
        assert_eq!(
            card.words.status,
            "Event not yet open: this card is played after card 17"
        );
        assert_eq!(
            card.words.lasting,
            "Insurgent Capability · not in effect until executed"
        );
        assert_eq!(card.words.rules, "R 2.3.8 · R 5.1 · R 5.3");
    }
}

#[test]
fn a_pass_opens_card_17_to_directorio_as_1st_eligible() {
    let mut game = Game::opening();
    choose(&mut game, "pass");
    let card = current(&game);
    assert_eq!(
        card.status,
        EventStatus::Open {
            faction: Directorio,
            role: Role::First
        }
    );
    assert_eq!(
        card.words.status,
        "Event open to Directorio as 1st Eligible"
    );
}

#[test]
fn during_terror_the_event_waits_on_26july_s_operation() {
    let mut game = Game::opening();
    choose(&mut game, "terror");
    let waiting = current(&game);
    assert_eq!(
        waiting.status,
        EventStatus::AwaitingOperation {
            executing: M26,
            second: Directorio,
            special_activity: false
        }
    );
    assert_eq!(
        waiting.words.status,
        "Event not yet open: Directorio may execute it if 26July's Operation includes a Special Activity"
    );

    let mut kidnapped = crate::terror_chosen(Game::opening_with_seed(2));
    choose(&mut kidnapped, "kidnap");
    choose(&mut kidnapped, "kidnap:roll");
    let card = current(&kidnapped);
    assert_eq!(
        card.status,
        EventStatus::AwaitingOperation {
            executing: M26,
            second: Directorio,
            special_activity: true
        }
    );
    assert_eq!(
        card.words.status,
        "Event not yet open: Directorio may execute it once 26July ends its Operation"
    );
}

#[test]
fn an_operation_only_closes_the_event_for_the_card() {
    let mut game = crate::terror_chosen(Game::opening());
    choose(&mut game, "end-terror");
    let card = current(&game);
    assert_eq!(
        card.status,
        EventStatus::Closed {
            executed: M26,
            second: Some(Directorio)
        }
    );
    assert_eq!(
        card.words.status,
        "Event not open this card: after 26July's Operation Only, Directorio may take only a Limited Operation"
    );
}

#[test]
fn the_event_opens_to_directorio_then_runs_and_resolves_either_text() {
    let open = handoff();
    let card = current(&open);
    assert_eq!(
        card.status,
        EventStatus::Open {
            faction: Directorio,
            role: Role::Second
        }
    );
    assert_eq!(
        card.words.status,
        "Event open to Directorio as 2nd Eligible"
    );

    let mut shaded = open.clone();
    choose(&mut shaded, "event:shaded");
    let card = current(&shaded);
    assert_eq!(
        card.status,
        EventStatus::InProgress {
            faction: Directorio,
            text: EventText::Shaded
        }
    );
    assert_eq!(
        card.words.status,
        "Event in progress: Directorio is executing the shaded text"
    );
    choose(&mut shaded, "resolve-shaded");
    let card = current(&shaded);
    assert_eq!(
        card.status,
        EventStatus::Resolved {
            faction: Directorio,
            text: EventText::Shaded
        }
    );
    assert_eq!(
        card.words.status,
        "Event resolved: Directorio executed the shaded text"
    );
    assert_eq!(card.rules, ["R 2.3.4", "R 5.2"]);

    let mut unshaded = open;
    choose(&mut unshaded, "event:unshaded");
    let expected = EventStatus::InProgress {
        faction: Directorio,
        text: EventText::Unshaded,
    };
    assert_eq!(current(&unshaded).status, expected);
    choose(&mut unshaded, "rally:beneficiary:26july");
    assert_eq!(current(&unshaded).status, expected);
    assert_eq!(
        current(&unshaded).words.status,
        "Event in progress: Directorio is executing the unshaded text"
    );
    choose(&mut unshaded, "rally:sierra-maestra:place:0");
    let card = current(&unshaded);
    assert_eq!(
        card.status,
        EventStatus::Resolved {
            faction: Directorio,
            text: EventText::Unshaded
        }
    );
    assert_eq!(
        card.words.status,
        "Event resolved: Directorio executed the unshaded text"
    );
    // The preview is still card 18 until the advance.
    assert_eq!(
        preview(&unshaded).status,
        EventStatus::Preview { after: 17 }
    );
}

#[test]
fn after_the_advance_card_18_opens_to_government_and_19_is_previewed() {
    let mut game = handoff();
    choose(&mut game, "event:shaded");
    choose(&mut game, "resolve-shaded");
    choose(&mut game, "advance-card");

    let card = current(&game);
    assert_eq!((card.number, card.title), (18, "Pact of Caracas"));
    // 26July executed an Operation and Directorio the Event, so both are Ineligible
    // (R 2.3.7); Government is the leftmost Eligible faction.
    assert_eq!(
        card.status,
        EventStatus::Open {
            faction: Government,
            role: Role::First
        }
    );
    assert_eq!(
        card.words.status,
        "Event open to Government as 1st Eligible"
    );
    assert_eq!(card.rules, ["R 2.3.4", "R 5.1", "R 5.3"]);
    assert_eq!(
        card.words.lasting,
        "Insurgent Capability · not in effect until executed"
    );

    let next = preview(&game);
    assert_eq!((next.number, next.title), (19, "Sierra Maestra Manifesto"));
    assert_eq!(next.order, [M26, Directorio, Syndicate, Government]);
    assert_eq!(next.kind, EventKind::Single);
    assert_eq!(next.lasting, None);
    assert_eq!(next.status, EventStatus::Preview { after: 18 });
    assert_eq!(next.rules, ["R 2.3.8", "R 5.1"]);
    assert_eq!(
        next.words.status,
        "Event not yet open: this card is played after card 18"
    );
    assert_eq!(next.words.lasting, "None");

    let mut passed = game.clone();
    choose(&mut passed, "government:pass");
    let card = current(&passed);
    assert_eq!(
        card.status,
        EventStatus::Open {
            faction: Syndicate,
            role: Role::First
        }
    );
    assert_eq!(card.words.status, "Event open to Syndicate as 1st Eligible");

    let mut assault = game;
    choose(&mut assault, "government:assault");
    let card = current(&assault);
    let closed = EventStatus::Closed {
        executed: Government,
        second: Some(Syndicate),
    };
    assert_eq!(card.status, closed);
    assert_eq!(
        card.words.status,
        "Event not open this card: after Government's Operation Only, Syndicate may take only a Limited Operation"
    );
    choose(&mut assault, "end-assault");
    assert_eq!(current(&assault).status, closed);
}
