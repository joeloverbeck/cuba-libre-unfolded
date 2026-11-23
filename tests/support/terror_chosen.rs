//! The Terror-chosen start, shared by the library's unit tests (`src/lib.rs`) and
//! the integration tests (`tests/integration/main.rs`), which each bring it to
//! their crate root. The includer brings `Faction` and `Game` into scope.
use super::{Faction, Game};

/// `game`, an opening position, after 26July chooses Terror (R 2.3.4) as a player
/// does: submitted at match revision 0, so the position is at revision 1.
pub(crate) fn terror_chosen(mut game: Game) -> Game {
    game.submit(Faction::M26, 0, "0:terror")
        .expect("Terror is offered at the opening position");
    game
}
