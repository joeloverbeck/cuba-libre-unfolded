//! The FigCraft `Atlas` colour variables the new interface uses, in one place
//! (foundation 4, "Layout authority": the selected layouts' variables own the
//! palette). The values are the variables' own.
use bevy::prelude::Color;

/// `ink`: titles, values and body text.
pub(super) const INK: Color = Color::srgb_u8(32, 50, 56);
/// `ink-secondary`: secondary values, notes and quiet labels.
pub(super) const QUIET: Color = Color::srgb_u8(83, 106, 109);
/// `paper`: card fills, and text on `action/primary`.
pub(super) const PAPER: Color = Color::srgb_u8(248, 244, 232);
/// `action/primary`.
pub(super) const PRIMARY: Color = Color::srgb_u8(160, 53, 49);
/// `rule`.
pub(super) const RULE: Color = Color::srgb_u8(211, 219, 215);
