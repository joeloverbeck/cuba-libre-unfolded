//! The integration tests, as one test binary. Each module used to be its own binary;
//! together they link the Bevy-built library once instead of sixteen times, and
//! their tests share one parallel libtest pool. The native regressions stay in
//! `tests/native.rs`, a separate binary that `scripts/native` runs with `--ignored`.
//!
//! Run one module with a name filter, e.g.
//! `cargo test --locked --test integration seat_brief::`.

/// The task's reachable states, which `honest_hints` and `seat_reference` share. It
/// is loaded once here: Clippy refuses one file loaded as two modules of a binary.
#[path = "../support/explore.rs"]
mod explore;
// Not unused: `explore` takes these from its includer, through `super::`.
use cuba_libre_unfolded::{Choice, Faction, Game, TransferChoice};

mod action_choice;
mod assault;
mod assault_side;
mod card_facts;
mod eligibility;
mod highlights;
mod honest_hints;
mod kidnap;
mod kidnap_die;
mod map_targets;
mod persistence;
mod popover;
mod position;
mod previews;
mod removal;
mod seat_brief;
mod seat_reference;
mod terror;
mod terror_token;
mod transfers;
