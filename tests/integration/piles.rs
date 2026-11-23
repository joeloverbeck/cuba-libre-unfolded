//! #143: the numbers on the two piles and the Propaganda pips, from the rules model.
//! Every expectation here was worked by hand, before the model, from the March 2019
//! Living Rules and the prepared deck:
//!
//! - Deck Preparation: 48 Event cards and 4 Propaganda cards, 52 in all, as four piles
//!   of 12 Events with one Propaganda card each. The playbook's card list numbers the
//!   Propaganda cards 49–52.
//! - The prepared deck (`docs/research/tactical-slice-handoff.md`): Events 1–12 then
//!   Propaganda 49; 13–24 then 50; 25–36 then 51; 37–48 then 52.
//! - R 2.2: the first card goes onto the played pile and the next is revealed on the
//!   draw deck, where it stays until it is played (R 2.3.8). Every card is therefore
//!   in one of the two piles, and "all played cards and the number of cards in the
//!   draw deck are open to inspection".
//!
//! Card 17 is the 18th card of that order (12 Events, card 49, then 13–17), so on card
//! 17 the played pile holds 18 cards with it, and the draw deck the other 34 with the
//! preview, card 18, on top: 18–24 (7), 50, 25–36 (12), 51, 37–48 (12), 52. Only card
//! 49 of the four Propaganda cards is among the played. Playing card 18 (R 2.3.8)
//! moves one Event across: 19 and 33, still one Propaganda card played.
use cuba_libre_unfolded::Game;
use serde_json::Value;

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

/// Card 17 played to its end and card 18 made current (R 2.3.8).
fn advanced() -> Game {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in ["kidnap", "kidnap:roll", "terror:santiago", "transfer"] {
        choose(&mut game, action);
    }
    for action in [
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
    ] {
        choose(&mut game, action);
    }
    game
}

#[test]
fn on_card_17_the_piles_read_18_and_34_with_one_propaganda_card_played() {
    let view = Game::opening().observe();
    assert_eq!((view.current_card, view.preview_card), (17, 18));
    assert_eq!(view.piles.played, 18, "the played pile counts card 17");
    assert_eq!(view.piles.deck, 34, "the draw deck counts the preview");
    assert_eq!(view.piles.played + view.piles.deck, 52);
    assert_eq!(view.piles.propaganda_played, 1);
    assert_eq!(view.piles.propaganda_cards, 4);
}

#[test]
fn playing_card_18_moves_one_card_across() {
    let view = advanced().observe();
    assert_eq!((view.current_card, view.preview_card), (18, 19));
    assert_eq!(view.piles.played, 19);
    assert_eq!(view.piles.deck, 33);
    assert_eq!(view.piles.played + view.piles.deck, 52);
    // Card 18 is an Event, so the Propaganda cards played stay at one.
    assert_eq!(view.piles.propaganda_played, 1);
    assert_eq!(view.piles.propaganda_cards, 4);
}

#[test]
fn the_counts_hold_at_every_step_of_card_17() {
    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    let steps = [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
    ];
    for action in steps {
        choose(&mut game, action);
        let piles = game.observe().piles;
        assert_eq!(
            (piles.played, piles.deck, piles.propaganda_played),
            (18, 34, 1),
            "after {action}"
        );
    }
}

/// Every array of whole numbers anywhere in a value.
fn number_lists(value: &Value, found: &mut Vec<Vec<u64>>) {
    match value {
        Value::Array(items) => {
            if !items.is_empty() && items.iter().all(Value::is_u64) {
                found.push(items.iter().map(|n| n.as_u64().unwrap()).collect());
            }
            for item in items {
                number_lists(item, found);
            }
        }
        Value::Object(fields) => {
            for field in fields.values() {
                number_lists(field, found);
            }
        }
        _ => {}
    }
}

/// R 2.2 opens the played cards and the deck's number to inspection, and R 2.3.8 one
/// card ahead. The order of the cards under the preview stays closed: by the prepared
/// deck they are 19, 20, 21, … on card 17 and 20, 21, 22, … on card 18.
#[test]
fn the_order_of_the_face_down_cards_is_not_observed() {
    for (game, first_hidden) in [(Game::opening(), 19), (advanced(), 20)] {
        let view = serde_json::to_value(game.observe()).unwrap();
        let fields = view.as_object().unwrap();
        for name in fields.keys() {
            assert!(
                !name.contains("deck_order") && name != "draw_deck",
                "{name} would carry the deck"
            );
        }
        let mut lists = Vec::new();
        number_lists(&view, &mut lists);
        assert!(!lists.is_empty());
        for list in &lists {
            assert!(
                !list
                    .windows(2)
                    .any(|pair| pair == [first_hidden, first_hidden + 1]),
                "{list:?} follows the face-down order"
            );
        }
        // The played cards are public, and the list ends before the first hidden one.
        let played: Vec<u64> = view["played_cards"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap())
            .collect();
        assert!(
            played
                .iter()
                .all(|card| *card < first_hidden - 1 || *card == 49)
        );
    }
}

/// Every string anywhere in a value, and every whole number held by a field whose name
/// says it holds a card, with the field's name.
fn words_and_cards<'a>(
    value: &'a Value,
    key: &str,
    words: &mut Vec<&'a str>,
    cards: &mut Vec<(String, u64)>,
) {
    match value {
        Value::String(text) => words.push(text),
        Value::Number(n) if key.contains("card") || key == "number" => {
            if let Some(n) = n.as_u64() {
                cards.push((key.to_owned(), n));
            }
        }
        Value::Array(items) => {
            for item in items {
                words_and_cards(item, key, words, cards);
            }
        }
        Value::Object(fields) => {
            for (name, field) in fields {
                words_and_cards(field, name, words, cards);
            }
        }
        _ => {}
    }
}

/// Story 40 and the spec's "Counts after the advance": the face-down order appears
/// nowhere in the observation. By the prepared deck the card under the preview is
/// 19, *Sierra Maestra Manifesto*, on card 17 and 20, *The Twelve*, on card 18 (the
/// playbook's card list, p. 36). Neither may be named or numbered anywhere, also not
/// in a choice's label or explanation; checked at the opening, when the advance itself
/// is offered, and after it.
#[test]
fn the_card_under_the_preview_is_named_nowhere_in_the_observation() {
    let mut offered = crate::terror_chosen(Game::opening_with_seed(2));
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
    ] {
        choose(&mut offered, action);
    }
    let advance = format!("{}:advance-card", offered.observe().revision);
    assert!(
        offered.observe().choices.iter().any(|c| c.id == advance),
        "the advance is offered"
    );
    for (game, hidden, title) in [
        (Game::opening(), 19, "Sierra Maestra Manifesto"),
        (offered, 19, "Sierra Maestra Manifesto"),
        (advanced(), 20, "The Twelve"),
    ] {
        let view = serde_json::to_value(game.observe()).unwrap();
        let (mut words, mut cards) = (Vec::new(), Vec::new());
        words_and_cards(&view, "", &mut words, &mut cards);
        assert!(!words.is_empty() && !cards.is_empty());
        let number = hidden.to_string();
        for text in words {
            assert!(!text.contains(title), "{text:?} names card {hidden}");
            assert!(
                !text
                    .split(|c: char| !c.is_ascii_digit())
                    .any(|digits| digits == number),
                "{text:?} numbers card {hidden}"
            );
        }
        for (name, card) in cards {
            assert_ne!(card, hidden, "{name} holds card {hidden}");
        }
    }
}
