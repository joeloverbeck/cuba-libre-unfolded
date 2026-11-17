//! The accepted prepared Cuba Libre tactical task; not a full game.
pub mod game;
pub use game::*;
pub mod assets;
pub mod save;
pub mod seat_brief;
#[cfg(feature = "tabletop")]
pub mod tabletop;
pub mod targets;
