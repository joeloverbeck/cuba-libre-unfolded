//! The accepted prepared Cuba Libre tactical task; not a full game.
pub mod game;
pub use game::*;
pub mod assets;
pub mod save;
pub mod seat_brief;
#[cfg(feature = "tabletop")]
pub mod tabletop;
pub mod targets;

#[cfg(test)]
#[path = "../tests/support/terror_chosen.rs"]
mod terror_chosen;
#[cfg(test)]
pub(crate) use terror_chosen::terror_chosen;
