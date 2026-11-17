//! The integration tests, as one test binary. Each module used to be its own binary;
//! together they link the Bevy-built library once instead of sixteen times, and
//! their tests share one parallel libtest pool. The native regressions stay in
//! `tests/native.rs`, a separate binary that `scripts/native` runs with `--ignored`.
//!
//! Run one module with a name filter, e.g.
//! `cargo test --locked --test integration seat_brief::`.
mod action_choice;
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
mod terror;
mod terror_token;
mod transfers;
