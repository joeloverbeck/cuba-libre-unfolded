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
#[path = "../support/terror_chosen.rs"]
mod terror_chosen;
// Not unused: `explore` and `terror_chosen` take these from their includer,
// through `super::`.
use cuba_libre_unfolded::{Choice, Faction, Game, TransferChoice};
use terror_chosen::terror_chosen;

mod action_choice;
mod admission;
mod assault;
mod assault_side;
mod available_forces;
mod card_facts;
mod change_texts;
mod eligibility;
mod focus_mark;
mod highlights;
mod honest_hints;
mod kidnap;
mod kidnap_die;
mod main_view;
mod map_targets;
mod marker_supply;
mod persistence;
mod pile_tags;
mod piles;
mod popover;
mod position;
mod previews;
mod removal;
mod save_quit;
mod seat_brief;
mod seat_reference;
mod terror;
mod terror_token;
mod transfers;
