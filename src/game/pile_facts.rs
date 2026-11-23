//! What the two pile tags read (#144; R 2.2, 2.3.8, 2.4.1, Deck Preparation): every card
//! in the played pile, most recent first, and the draw deck's public facts, among them
//! how soon the next Propaganda card can come.
//!
//! Everything here is derived from public facts alone: the played cards, the two
//! counts, the Propaganda cards played and the preview, which lies face up. Nothing
//! here is given the order of the cards under the preview.
use super::cards::{DECK_CARDS, PROPAGANDA_CARDS, is_propaganda};
use super::titles::title;
use serde::Serialize;

/// The cards in each pile of a deck prepared with exactly equal piles: 12 Events and
/// one Propaganda card (Deck Preparation; the local policy "Next Propaganda assumes
/// equal piles").
const PILE: u8 = DECK_CARDS / PROPAGANDA_CARDS;

/// One card in the played pile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayedCard {
    pub number: u8,
    pub title: &'static str,
    /// One of the four Propaganda cards (R 2.4).
    pub propaganda: bool,
    /// The current card, on top of the pile.
    pub current: bool,
}

/// The played pile's tag's words, worded once here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayedWords {
    pub cards: String,
    pub propaganda: String,
    /// One line for each card, most recent first.
    pub rows: Vec<String>,
    pub note: &'static str,
}

/// Every card played, most recent first, the current card among them (R 2.2: "All
/// played cards … are open to inspection").
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayedPile {
    pub cards: Vec<PlayedCard>,
    pub words: PlayedWords,
}

/// Where the next Propaganda card can lie, counted from the draw deck's top: the
/// preview is the 1st card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct NextPropaganda {
    pub earliest: u8,
    pub latest: u8,
}

/// The draw deck's tag's words, worded once here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeckWords {
    pub cards: String,
    pub face_down: String,
    pub propaganda_left: String,
    pub next_propaganda: String,
    /// What the figure assumes and why it says what it says.
    pub note: String,
}

/// The draw deck's public facts (R 2.2, 2.3.8): its cards under the face-up preview,
/// the Propaganda cards still in it and how soon the next can come.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeckFacts {
    pub face_down: u8,
    pub propaganda_left: u8,
    /// None once all four Propaganda cards are played.
    pub next_propaganda: Option<NextPropaganda>,
    pub words: DeckWords,
}

/// The earliest and latest place of the next Propaganda card, counted from the deck's
/// top, from the number of cards played (the current card among them), the
/// Propaganda cards among those, and whether the face-up preview is one.
///
/// It assumes the deck was prepared with exactly equal piles of 12 Events, each with
/// one Propaganda card, so the n-th Propaganda card lies in the n-th pile of 13. With
/// `propaganda` of them played the next lies in pile `propaganda + 1`; of that pile's
/// places only those after the played cards are still to come. An Event preview
/// cannot be it, and a Propaganda preview is it. None when all four are played, or
/// when the counts do not fit equal piles.
pub fn next_propaganda(played: u8, propaganda: u8, preview: bool) -> Option<NextPropaganda> {
    if propaganda >= PROPAGANDA_CARDS {
        return None;
    }
    let first = (PILE * propaganda + 1).max(played + 1) - played;
    let last = PILE * (propaganda + 1);
    if last <= played {
        return None;
    }
    let last = last - played;
    match (preview, first) {
        (true, 1) => Some(NextPropaganda {
            earliest: 1,
            latest: 1,
        }),
        (true, _) => None,
        (false, 1) if last == 1 => None,
        (false, 1) => Some(NextPropaganda {
            earliest: 2,
            latest: last,
        }),
        (false, _) => Some(NextPropaganda {
            earliest: first,
            latest: last,
        }),
    }
}

fn ordinal(n: u8) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

const PREPARATION: &str =
    "Deck Preparation shuffles one Propaganda card into each of four piles of Events.";
const ASSUMPTION: &str =
    "Local policy: the figure assumes the deck was prepared with exactly equal piles of 12";

/// The deck tag's figure and its note, for the same facts as [`next_propaganda`].
pub fn next_propaganda_words(played: u8, propaganda: u8, preview: bool) -> (String, String) {
    let Some(next) = next_propaganda(played, propaganda, preview) else {
        return if propaganda >= PROPAGANDA_CARDS {
            (
                "none: all four played".into(),
                format!("{PREPARATION} All four Propaganda cards have been played."),
            )
        } else {
            (
                "not known".into(),
                format!("{PREPARATION} These counts do not fit a deck of equal piles."),
            )
        };
    };
    if preview {
        return (
            "the preview, next".into(),
            format!("{PREPARATION} The preview is a Propaganda card, so it comes next."),
        );
    }
    // The cards left in the pile the preview lies in, the preview among them.
    let left = PILE - (played % PILE);
    let cards = |n: u8| {
        if n == 1 {
            "1 card".to_string()
        } else {
            format!("{n} cards")
        }
    };
    // The pile the preview lies in has had its Propaganda card played: the next is in
    // the pile after it.
    let next_pile = PILE * propaganda > played;
    let (figure, why) = if next_pile {
        (
            format!(
                "the {} to the {} card",
                ordinal(next.earliest),
                ordinal(next.latest)
            ),
            format!(
                "so the current pile's Propaganda card is played, and the next is in the pile after its {} left",
                cards(left)
            ),
        )
    } else if left == PILE {
        (
            format!("within the next {}", cards(next.latest)),
            format!("so the preview starts a new pile of {PILE} cards"),
        )
    } else {
        (
            format!("within the next {}", cards(next.latest)),
            format!("so the current pile has {} left", cards(left)),
        )
    };
    (figure, format!("{PREPARATION} {ASSUMPTION}, {why}."))
}

/// The played pile from the cards played before the current one, oldest first, and
/// the current card.
pub(crate) fn played_pile(played_before: &[u8], current: u8) -> PlayedPile {
    let cards: Vec<PlayedCard> = played_before
        .iter()
        .copied()
        .chain([current])
        .rev()
        .map(|number| PlayedCard {
            number,
            title: title(number),
            propaganda: is_propaganda(number),
            current: number == current,
        })
        .collect();
    let propaganda = cards.iter().filter(|c| c.propaganda).count();
    PlayedPile {
        words: PlayedWords {
            cards: format!("{} · current card on top", cards.len()),
            propaganda: format!("{propaganda} of {PROPAGANDA_CARDS} played"),
            rows: cards
                .iter()
                .map(|c| {
                    let row = format!("{} · {}", c.number, c.title);
                    if c.current {
                        format!("{row} · current")
                    } else {
                        row
                    }
                })
                .collect(),
            note: "The pile is open to inspection (R 2.2). It shows which Events are gone and how many Propaganda cards have been played.",
        },
        cards,
    }
}

/// The draw deck's facts from the played pile, the number of cards in the deck (the
/// preview among them) and the preview, the only one of them face up.
pub(crate) fn deck_facts(played: &PlayedPile, deck: u8, preview: u8) -> DeckFacts {
    let count = played.cards.len() as u8;
    let propaganda = played.cards.iter().filter(|c| c.propaganda).count() as u8;
    let preview = is_propaganda(preview);
    let (figure, note) = next_propaganda_words(count, propaganda, preview);
    let face_down = deck.saturating_sub(1);
    let propaganda_left = PROPAGANDA_CARDS - propaganda;
    DeckFacts {
        face_down,
        propaganda_left,
        next_propaganda: next_propaganda(count, propaganda, preview),
        words: DeckWords {
            cards: format!("{deck} · counts the preview"),
            face_down: face_down.to_string(),
            propaganda_left: format!("{propaganda_left} of {PROPAGANDA_CARDS}"),
            next_propaganda: figure,
            note,
        },
    }
}
