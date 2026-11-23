//! #144: what the two pile tags read, from the rules model. Every expectation here was
//! worked by hand, before the model, from the March 2019 Living Rules (SHA-256
//! `8152cb00…6792`), the third-printing playbook's card list (p. 36, SHA-256
//! `dcaa5afd…e8`) and the supplied faces:
//!
//! - **Deck Preparation** (back page): "Separate out the 4 Propaganda cards and shuffle
//!   the 48 Event cards. … Split remaining Event cards into 4 roughly (or, if desired,
//!   exactly) equal piles. Shuffle 1 Propaganda card into each pile and stack 1 pile
//!   onto the other to form a face-down draw deck." With exactly equal piles (the local
//!   policy this ticket records) each pile is 12 Events and 1 Propaganda card: the deck's
//!   cards 1–13 are the first pile, 14–26 the second, 27–39 the third and 40–52 the
//!   fourth, and the n-th Propaganda card is somewhere in the n-th pile.
//! - **R 2.2, 2.3.8:** the played pile holds every card played, the current card on top;
//!   the draw deck holds the rest, the preview face up on top. All played cards and the
//!   deck's number are open; the order under the preview is not.
//! - **R 2.4, 2.4.1:** a Propaganda card brings a Propaganda Round; the fourth is final.
//!
//! **The general rule.** Count deck positions from the start of the game, so the played
//! pile is positions 1..=P and the preview is P + 1. With k Propaganda cards played, the
//! next one is the (k + 1)-th, which lies in pile k + 1: positions 13k + 1 ..= 13k + 13,
//! of which only those after P are still to come. Counted from the deck's top (the
//! preview is the 1st card), it is between max(P + 1, 13k + 1) − P and 13(k + 1) − P.
//! The preview is face up, so it narrows the figure: a Propaganda preview is the next
//! card; an Event preview cannot be, so the earliest is then the 2nd card.
//!
//! **The exceptions, checked against the rule.** If the pile the preview lies in still
//! holds its Propaganda card (k is that pile's number less one), the figure runs to the
//! end of that pile. If its Propaganda card is already played (k is that pile's
//! number), the figure is the whole of the next pile. A preview that starts a new pile
//! has that pile's 13 cards ahead. The worked positions below agree with the rule:
//!
//! | Position | P | k | Preview | Hand count | Earliest–latest |
//! |---|---|---|---|---|---|
//! | Prepared deck, card 17 | 18 | 1 | 18, an Event | pile 2 is 14–26; 19–26 are left, 8 cards with the preview | 2–8 |
//! | Prepared deck, card 18 | 19 | 1 | 19, an Event | 20–26 left, 7 with the preview | 2–7 |
//! | Pile 2's Propaganda played as its first card, 2 cards into pile 2 | 15 | 2 | an Event | 16–26 are pile 2's last 11 Events; pile 3 is 27–39, the 12th to 24th | 12–24 |
//! | Preview is a Propaganda card (prepared deck, card 24 current) | 25 | 1 | 50 | it is the next card | 1–1 |
//! | Same pile, preview a Propaganda card early | 20 | 1 | Propaganda | the next card | 1–1 |
//! | Current card is the last of pile 1 (49 in the prepared deck) | 13 | 1 | 13, the first card of pile 2 | 14–26, the 1st to 13th; the preview is an Event | 2–13 |
//! | Current card is the first card of pile 2 | 14 | 1 | an Event | 15–26, 12 cards with the preview | 2–12 |
//! | First card of the game, an Event | 1 | 0 | an Event | 2–13, 12 cards with the preview | 2–12 |
//! | First card of the game is Propaganda | 1 | 1 | an Event | pile 2 is 14–26, the 13th to 25th | 13–25 |
//! | All four played | 52 | 4 | none | no Propaganda card is left | none |
//!
//! In the prepared deck each Propaganda card is its pile's last, so the figure's latest
//! card is where the next one actually lies: 50 is the 8th card on card 17 and the 7th
//! on card 18.
use cuba_libre_unfolded::{Game, NextPropaganda, next_propaganda};

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
    let mut game = Game::prepared_with_seed(2);
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
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

/// The third-printing playbook's card list (p. 36), transcribed from the playbook. The
/// model's titles are transcribed from the supplied faces; the two must agree.
const PLAYBOOK: [&str; 52] = [
    "Armored Cars",
    "Guantánamo Bay",
    "Eulogio Cantillo",
    "S.I.M.",
    "Rolando Masferrer",
    "Sánchez Mosquera",
    "Election",
    "General Strike",
    "Coup",
    "MAP",
    "Batista Flees",
    "BRAC",
    "El Che",
    "Operation Fisherman",
    "Come Comrades!",
    "Larrazábal",
    "Alberto Bayo",
    "Pact of Caracas",
    "Sierra Maestra Manifesto",
    "The Twelve",
    "Fangio",
    "Raúl",
    "Radio Rebelde",
    "Vilma Espín",
    "Escapade",
    "Rodríguez Loeches",
    "Echeverría",
    "Morgan",
    "Fauré Chomón",
    "The Guerrilla Life",
    "Escopeteros",
    "Resistencia Cívica",
    "Carlos Prío",
    "US Speaking Tour",
    "Defections",
    "Eloy Gutiérrez Menoyo",
    "Herbert Matthews",
    "Meyer Lansky",
    "Turismo",
    "Ambassador Smith",
    "Fat Butcher",
    "Llano",
    "Mafia Offensive",
    "Rebel Air Force",
    "Anastasia",
    "Sinatra",
    "Pact of Miami",
    "Santo Trafficante Jr",
    "Propaganda!",
    "Propaganda!",
    "Propaganda!",
    "Propaganda!",
];

#[test]
fn every_title_agrees_with_the_playbook_s_card_list() {
    for (number, title) in (1..=52).zip(PLAYBOOK) {
        assert_eq!(cuba_libre_unfolded::title(number), title, "card {number}");
    }
}

fn between(earliest: u8, latest: u8) -> Option<NextPropaganda> {
    Some(NextPropaganda { earliest, latest })
}

#[test]
fn the_next_propaganda_card_follows_the_hand_worked_positions() {
    // (P, k, preview is Propaganda) → earliest and latest, from the table above.
    let cases = [
        ("prepared deck, card 17", 18, 1, false, between(2, 8)),
        ("prepared deck, card 18", 19, 1, false, between(2, 7)),
        (
            "current pile's Propaganda played",
            15,
            2,
            false,
            between(12, 24),
        ),
        ("preview is 50 on card 24", 25, 1, true, between(1, 1)),
        ("preview is Propaganda early", 20, 1, true, between(1, 1)),
        ("preview starts pile 2", 13, 1, false, between(2, 13)),
        ("current card starts pile 2", 14, 1, false, between(2, 12)),
        ("first card, an Event", 1, 0, false, between(2, 12)),
        ("first card, Propaganda", 1, 1, false, between(13, 25)),
        ("all four played", 52, 4, false, None),
    ];
    for (name, played, propaganda, preview, expected) in cases {
        assert_eq!(
            next_propaganda(played, propaganda, preview),
            expected,
            "{name}"
        );
    }
}

#[test]
fn on_card_17_the_played_pile_lists_18_cards_most_recent_first() {
    let view = Game::opening().observe();
    let pile = &view.played_pile;
    let numbers: Vec<u8> = pile.cards.iter().map(|c| c.number).collect();
    // The prepared deck's first 18 cards, 1–12, 49, 13–17, read from the top down.
    assert_eq!(
        numbers,
        [
            17, 16, 15, 14, 13, 49, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
        ]
    );
    for card in &pile.cards {
        assert_eq!(card.title, PLAYBOOK[usize::from(card.number) - 1]);
        assert_eq!(card.propaganda, card.number == 49, "card {}", card.number);
        assert_eq!(card.current, card.number == 17, "card {}", card.number);
    }
    assert_eq!(pile.words.cards, "18 · current card on top");
    assert_eq!(pile.words.propaganda, "1 of 4 played");
    assert_eq!(pile.words.rows[0], "17 · Alberto Bayo · current");
    assert_eq!(pile.words.rows[5], "49 · Propaganda!");
    assert_eq!(pile.words.rows[17], "1 · Armored Cars");
    assert_eq!(pile.words.rows.len(), 18);
}

#[test]
fn after_the_advance_the_played_pile_lists_19_cards() {
    let view = advanced().observe();
    let pile = &view.played_pile;
    let numbers: Vec<u8> = pile.cards.iter().map(|c| c.number).collect();
    assert_eq!(
        numbers,
        [
            18, 17, 16, 15, 14, 13, 49, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
        ]
    );
    assert!(pile.cards[0].current && !pile.cards[1].current);
    assert_eq!(pile.words.cards, "19 · current card on top");
    assert_eq!(pile.words.rows[0], "18 · Pact of Caracas · current");
    assert_eq!(pile.words.rows[1], "17 · Alberto Bayo");
}

#[test]
fn on_card_17_the_deck_reads_34_33_and_three_propaganda_cards_within_8() {
    let view = Game::opening().observe();
    let deck = &view.deck;
    assert_eq!(view.piles.deck, 34);
    assert_eq!(deck.face_down, 33);
    assert_eq!(deck.propaganda_left, 3);
    assert_eq!(deck.next_propaganda, between(2, 8));
    assert_eq!(deck.words.cards, "34 · counts the preview");
    assert_eq!(deck.words.face_down, "33");
    assert_eq!(deck.words.propaganda_left, "3 of 4");
    assert_eq!(deck.words.next_propaganda, "within the next 8 cards");
    assert!(
        deck.words
            .note
            .contains("assumes the deck was prepared with exactly equal piles of 12"),
        "{}",
        deck.words.note
    );
    assert!(
        deck.words
            .note
            .ends_with("so the current pile has 8 cards left.")
    );
}

#[test]
fn after_the_advance_the_deck_reads_33_32_and_within_7() {
    let view = advanced().observe();
    let deck = &view.deck;
    assert_eq!(view.piles.deck, 33);
    assert_eq!(deck.face_down, 32);
    assert_eq!(deck.propaganda_left, 3);
    assert_eq!(deck.next_propaganda, between(2, 7));
    assert_eq!(deck.words.cards, "33 · counts the preview");
    assert_eq!(deck.words.next_propaganda, "within the next 7 cards");
    assert!(
        deck.words
            .note
            .ends_with("so the current pile has 7 cards left.")
    );
}

#[test]
fn the_figure_s_words_name_each_hand_worked_case() {
    use cuba_libre_unfolded::next_propaganda_words as words;
    // Within the current pile: the sheet's own line.
    assert_eq!(words(18, 1, false).0, "within the next 8 cards");
    // The current pile's Propaganda card already played: the whole of the next pile.
    let (figure, note) = words(15, 2, false);
    assert_eq!(figure, "the 12th to the 24th card");
    assert!(
        note.contains("the current pile's Propaganda card is played"),
        "{note}"
    );
    // The preview is a Propaganda card.
    let (figure, note) = words(25, 1, true);
    assert_eq!(figure, "the preview, next");
    assert!(note.contains("The preview is a Propaganda card"), "{note}");
    // The preview starts a pile.
    let (figure, note) = words(13, 1, false);
    assert_eq!(figure, "within the next 13 cards");
    assert!(
        note.ends_with("so the preview starts a new pile of 13 cards."),
        "{note}"
    );
    // None left.
    assert_eq!(words(52, 4, false).0, "none: all four played");
}
