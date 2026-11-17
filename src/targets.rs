//! Map targeting (#72): while 26July executes Terror, every space is selectable,
//! blocked with a remedy, or not involved. The states are read from the rules
//! authority's own choices, so the map and the panel always agree:
//! - **Selectable:** a supported choice acts there now (R 3.3.4 Terror, R 4.3.3
//!   Kidnap).
//! - **Blocked with a remedy:** nothing there is supported now, but an in-task action
//!   the authority names (its `remedies`) would make one supported.
//! - **Not involved:** neither.
use crate::{Choice, Observation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpaceState {
    Selectable,
    Blocked,
    NotInvolved,
}

/// One space's state and the choices that act on it, in the observation's order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpaceTarget {
    pub space: String,
    pub state: SpaceState,
    pub choices: Vec<String>,
}

/// Every space's targeting state, in the observation's space order, or `None` when
/// no choice acts on a space: at the action choice, after either ending, and while
/// Kidnap is open, a Resource transfer is under way or removal mode is open. Region
/// clicks take effect only while this is `Some`.
pub fn space_targets(view: &Observation) -> Option<Vec<SpaceTarget>> {
    let targeted: Vec<&Choice> = view
        .choices
        .iter()
        .filter(|c| c.space().is_some())
        .collect();
    if targeted.is_empty() {
        return None;
    }
    Some(
        view.spaces
            .iter()
            .map(|space| {
                let here: Vec<&Choice> = targeted
                    .iter()
                    .copied()
                    .filter(|c| c.space() == Some(space.id.as_str()))
                    .collect();
                let state = if here.iter().any(|c| c.supported) {
                    SpaceState::Selectable
                } else if here.iter().any(|c| !c.remedies.is_empty()) {
                    SpaceState::Blocked
                } else {
                    SpaceState::NotInvolved
                };
                SpaceTarget {
                    space: space.id.clone(),
                    state,
                    choices: here.iter().map(|c| c.id.clone()).collect(),
                }
            })
            .collect(),
    )
}
