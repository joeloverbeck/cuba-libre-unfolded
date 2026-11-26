//! The accepted prepared Cuba Libre tactical task; not a full game.
pub mod game;
pub use game::*;
pub mod assets;
pub mod editions;
pub mod faction_seats;
pub mod player_folders;
pub mod reports;
pub mod save;
pub mod seat_brief;
pub mod seat_wait;
#[cfg(feature = "tabletop")]
pub mod tabletop;
pub mod targets;

#[cfg(test)]
#[path = "../tests/support/terror_chosen.rs"]
mod terror_chosen;
#[cfg(test)]
pub(crate) use terror_chosen::terror_chosen;
// The longest actions, whose editions the newspaper sets.
#[cfg(all(test, feature = "tabletop"))]
#[path = "../tests/support/longest.rs"]
mod longest;
/// The longest actions' seed: its Kidnap die is a 3.
#[cfg(all(test, feature = "tabletop"))]
const SEED: u64 = 5;
