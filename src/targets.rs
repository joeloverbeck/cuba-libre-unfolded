//! Map targeting (#72, #169): while a decision offers choices that act on a space,
//! every space is selectable, blocked with a remedy, of no effect, or not involved.
//! The states are read from the rules authority's own choices, each of which states
//! the space it acts on and whether it changes anything there ([`crate::Site`]), so
//! the map and the panel always agree, whatever the Operation:
//! - **Selectable:** a supported choice there changes something now (R 3.3.4
//!   Terror, R 4.3.3 Kidnap, an R 3.2.4 Assault plan that removes or closes a piece).
//! - **Blocked with a remedy:** no such choice is supported now, but an in-task action
//!   the authority names (its `remedies`) would make one supported.
//! - **No effect:** the space's only choices change nothing on the map, as the
//!   Assault plan that removes nothing. The space stays choosable (R 3.2.4, 4.1),
//!   whether or not its choices are supported.
//! - **Unpayable:** a choice there would change something, and only its cost stops it:
//!   nothing in this task can pay and no remedy is named (#180). The space stays
//!   choosable, so its popover can say why.
//! - **Not involved:** no choice acts there, or none could for another reason than its
//!   cost: a resolved Terror, a Terror with no Underground Guerrilla, or a space already
//!   paid in this Assault (R 3.1).
//!
//! In removal mode (#73) piece states replace them: each of the executor's groups is
//! removable, or protected by the owed Terror (accepted commitment policy, local),
//! and spaces holding none of the executor's pieces are dimmed.
use crate::{Choice, Kind, Observation, State, removal_group};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpaceState {
    Selectable,
    Blocked,
    /// Choosable, and every choice there changes nothing on the map (#169).
    NoEffect,
    /// A choice there would act, and only its cost stops it: nothing in this task can
    /// pay for it and no remedy is named. Choosable, to read why (#180).
    Unpayable,
    NotInvolved,
}

impl SpaceState {
    /// The states whose spaces the keyboard reaches, in the order Tab reaches them
    /// (#168): where a supported choice acts, then where an in-task remedy would open
    /// one, then where the only choices change nothing (#169), then where nothing can
    /// pay (#180). This list is the one rule for that order: a state added later takes
    /// its place here, and a state not listed takes no focus.
    pub const FOCUS_ORDER: &'static [SpaceState] = &[
        SpaceState::Selectable,
        SpaceState::Blocked,
        SpaceState::NoEffect,
        SpaceState::Unpayable,
    ];

    /// Whether choosing a space in this state opens what the authority offers there.
    pub fn choosable(self) -> bool {
        Self::FOCUS_ORDER.contains(&self)
    }

    /// The state whose graphic the map draws for this one: a no-effect or unpayable
    /// space is as quiet as a space no choice acts on, so it draws that state's veil.
    pub fn drawn(self) -> SpaceState {
        match self {
            SpaceState::NoEffect | SpaceState::Unpayable => SpaceState::NotInvolved,
            state => state,
        }
    }
}

/// The spaces Tab reaches, by [`SpaceState::FOCUS_ORDER`], each state's spaces in the
/// observation's space order.
pub fn focus_order(targets: &[SpaceTarget]) -> Vec<&str> {
    SpaceState::FOCUS_ORDER
        .iter()
        .flat_map(|state| {
            targets
                .iter()
                .filter(move |t| t.state == *state)
                .map(|t| t.space.as_str())
        })
        .collect()
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
                let effect = |c: &Choice| c.site.as_ref().is_some_and(|site| site.effect);
                let state = if here.iter().any(|c| effect(c) && c.supported) {
                    SpaceState::Selectable
                } else if here.iter().any(|c| effect(c) && !c.remedies.is_empty()) {
                    SpaceState::Blocked
                } else if here.iter().all(|c| !effect(c)) && !here.is_empty() {
                    SpaceState::NoEffect
                } else if here
                    .iter()
                    .any(|c| c.site.as_ref().is_some_and(|s| s.effect && s.unpayable))
                {
                    SpaceState::Unpayable
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

/// An executing faction's group in piece management.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PieceState {
    /// At least one piece can be returned to Available Forces or closed in place.
    Removable,
    /// Every action on this group is refused. Protected Casinos can still preview why.
    Protected,
}

/// A group on the map in removal mode, and its available piece actions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PieceTarget {
    pub space: String,
    /// The state and kind as the piece action names them.
    pub group: String,
    pub kind: Kind,
    pub state: State,
    pub count: u8,
    /// The largest quantity a supported action can return or close now.
    pub removable: u8,
    pub mark: PieceState,
    /// The choice IDs for each action and quantity in the rules menu.
    pub choices: Vec<String>,
}

/// Removal mode's map: the executor's groups and the spaces dimmed because they hold none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RemovalTargets {
    pub pieces: Vec<PieceTarget>,
    pub dimmed: Vec<String>,
}

/// The removal targets, read from the authority's removal choices, or `None` outside
/// removal mode.
pub fn removal_targets(view: &Observation) -> Option<RemovalTargets> {
    view.removal.as_ref()?;
    let executor = view.executing?;
    let mut pieces = vec![];
    let mut dimmed = vec![];
    for space in &view.spaces {
        let mut holds = false;
        for p in space.pieces.iter().filter(|p| p.faction == executor) {
            let group = removal_group(p.kind, p.state);
            let here: Vec<&Choice> = view
                .choices
                .iter()
                .filter(|c| {
                    c.removal()
                        .or_else(|| c.casino_closure())
                        .is_some_and(|(s, g, _)| s == space.id && g == group)
                })
                .collect();
            if here.is_empty() {
                continue;
            }
            holds = true;
            let removable = here
                .iter()
                .filter(|c| c.supported)
                .filter_map(|c| {
                    c.removal()
                        .or_else(|| c.casino_closure())
                        .map(|(_, _, n)| n)
                })
                .max()
                .unwrap_or(0);
            pieces.push(PieceTarget {
                space: space.id.clone(),
                group: group.into(),
                kind: p.kind,
                state: p.state,
                count: p.count,
                removable,
                mark: if removable > 0 {
                    PieceState::Removable
                } else {
                    PieceState::Protected
                },
                choices: here.iter().map(|c| c.id.clone()).collect(),
            });
        }
        if !holds {
            dimmed.push(space.id.clone());
        }
    }
    Some(RemovalTargets { pieces, dimmed })
}
