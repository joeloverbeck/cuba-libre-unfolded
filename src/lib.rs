//! The accepted prepared Cuba Libre tactical task; not a full game.
pub mod game;
pub use game::*;
pub mod assets;
pub mod save;
pub mod seat_brief;
#[cfg(feature = "tabletop")]
pub mod tabletop;
pub mod targets;

/// `game`, an opening position, after 26July chooses Terror (R 2.3.4) as a player
/// does: submitted at match revision 0, so the position is at revision 1.
#[cfg(test)]
pub(crate) fn terror_chosen(mut game: Game) -> Game {
    game.submit(Faction::M26, 0, "0:terror")
        .expect("Terror is offered at the opening position");
    game
}
