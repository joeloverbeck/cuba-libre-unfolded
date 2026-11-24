//! Foundation 2, "Engine authority": legal-action discovery stays complete for the
//! supported decision, even when presented hierarchically. During the Terror phase
//! every choice the authority offers to 26July, in every state `tests/integration/honest_hints.rs`
//! explores, must have a surface that presents it: a panel step that pins and commits
//! it (or states its refusal), a space's popover for a choice that names a space (a
//! Terror target is chosen on the map only, #171), the panel's record line for a
//! resolved Terror, the commit or Kidnap's Cancel at the panel's foot, the transfer
//! modal, or a piece popover in removal mode.
#[path = "../../tests/support/explore.rs"]
mod explore;
use super::execution::{self, StepId};
use super::{Action, Interface, popover, transfer};
use crate::{ActionChoice, Choice, Faction, Game, Observation, TransferChoice, targets};

/// Where `choice` is presented in `view`, or `None` when no surface offers it.
fn surface(view: &Observation, choice: &Choice) -> Option<&'static str> {
    let action = choice.action();
    let submits = |commit: &execution::Commit| match &commit.action {
        Action::Commit(..) => commit.name == choice.id,
        Action::Submit(_, _, id) => *id == choice.id,
        _ => false,
    };
    // A step row that pins it; its commit submits it, or states the refusal.
    if let Some(id) = StepId::of(action)
        && execution::steps(view)
            .iter()
            .any(|s| s.id == id && s.interactive)
    {
        let commit = execution::commit(view, Some(action));
        let presented = if choice.supported {
            commit.enabled && submits(&commit)
        } else {
            !commit.enabled && matches!(commit.action, Action::Unsupported(_))
        };
        if presented {
            return Some("panel step");
        }
    }
    // A choice that names a space is presented by that space's popover, which a click
    // or Enter opens where the map marks the space selectable or blocked (#168): the
    // chosen row with the one button, or a blocked row with its reason (#171).
    if let Some(space) = choice.space()
        && targets::space_targets(view).is_some_and(|all| {
            all.iter().any(|t| {
                t.space == space
                    && matches!(
                        t.state,
                        targets::SpaceState::Selectable | targets::SpaceState::Blocked
                    )
            })
        })
        && let Some(card) = popover::content(view, space, Some(action))
        && let Some(row) = card
            .rows
            .iter()
            .find(|r| r.choice.as_ref() == Some(&choice.id))
    {
        if choice.supported && row.chosen && row.button.is_some() {
            return Some("space popover");
        }
        if !choice.supported && row.button.is_none() && !row.lines.is_empty() {
            return Some("space popover (refused)");
        }
    }
    // A refused choice that names a space, with nothing in this task to change it,
    // leaves its space not involved: the map veils it, and a click opens nothing. The
    // authority's reason stays in the observation for the seat; the map does not word
    // it (#171 moved Terror's targets off the panel, whose row gave it).
    if let Some(space) = choice.space()
        && !choice.supported
        && choice.remedies.is_empty()
        && targets::space_targets(view).is_some_and(|all| {
            all.iter()
                .any(|t| t.space == space && t.state == targets::SpaceState::NotInvolved)
        })
        && !(action.starts_with("terror:") && view.resolved.iter().any(|r| r.space == space))
    {
        return Some("map veil");
    }
    // A resolved Terror whose space offers nothing else: the panel's record line.
    if let Some(space) = choice.space()
        && action.starts_with("terror:")
        && !choice.supported
    {
        let record = format!("terror-record:{space}");
        if execution::lines(view)
            .iter()
            .any(|(part, _)| *part == record)
        {
            return Some("panel record");
        }
    }
    if submits(&execution::commit(view, None)) {
        return Some("panel commit");
    }
    if execution::focusable(view, None)
        .iter()
        .any(|(name, _)| *name == choice.id)
    {
        return Some("panel control");
    }
    if transfer::shown(view.transfer.as_ref()) {
        let ui = Interface::default();
        if transfer::focusable(view, &ui).contains(&choice.id) {
            return Some("transfer modal control");
        }
        // An amount: its counterpart, direction and typed amount make the modal's
        // primary submit it.
        let typed = |counterpart: usize, direction, n: &str| {
            let mut state = transfer::State::default();
            state.counterpart = Faction::ALL.get(counterpart).copied();
            state.direction = Some(direction);
            state.entry = n.to_owned();
            transfer::content(view, &state).commit.map(|c| c.id)
        };
        let amount = match choice.transfer() {
            Some(TransferChoice::Give { recipient, amount }) => {
                Some((recipient, transfer::Direction::Give, amount))
            }
            Some(TransferChoice::Ask { giver, amount }) => {
                Some((giver, transfer::Direction::Ask, amount))
            }
            _ => None,
        };
        if let Some((other, direction, n)) = amount
            && typed(other.index(), direction, &n.to_string()).as_deref()
                == Some(choice.id.as_str())
        {
            return Some("transfer modal amount");
        }
    }
    if let Some((space, group, n)) = choice.removal()
        && let Some(target) = targets::removal_targets(view).and_then(|r| {
            r.pieces
                .into_iter()
                .find(|p| p.space == space && p.group == group)
        })
    {
        let card = |quantity| popover::piece_content(view, &target, quantity);
        let row_offers = |card: Option<popover::Card>| {
            card.is_some_and(|c| c.rows.iter().any(|r| r.choice.as_ref() == Some(&choice.id)))
        };
        if choice.supported && row_offers(card(n)) {
            return Some("piece popover");
        }
        // Refused: the protected tag offers it refused, or the stepper's limit line
        // gives the next quantity's reason.
        if !choice.supported
            && (row_offers(card(1))
                || (n == target.removable + 1
                    && card(target.removable).is_some_and(|c| {
                        c.rows
                            .iter()
                            .any(|r| r.lines.iter().any(|(_, l)| *l == choice.explanation))
                    })))
        {
            return Some("piece popover (refused)");
        }
    }
    None
}

#[test]
fn every_choice_offered_to_26july_during_terror_has_a_surface() {
    let mut missing = std::collections::BTreeSet::new();
    let mut seen = std::collections::BTreeSet::new();
    for (start, game) in explore::starts() {
        for node in explore::explore(game) {
            let view = node.game.observe();
            if view.action != ActionChoice::Terror || view.finished {
                continue;
            }
            for choice in view.choices.iter().filter(|c| c.owner == Faction::M26) {
                match surface(&view, choice) {
                    Some(place) => {
                        seen.insert(place);
                    }
                    None => {
                        missing.insert(format!(
                            "{start}: {} ({}, supported {}, remedies {:?}, mode {:?}): {}",
                            choice.action(),
                            choice.label,
                            choice.supported,
                            choice.remedies,
                            execution::mode(&view),
                            choice.explanation
                        ));
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "no surface presents:\n{}",
        missing.into_iter().collect::<Vec<_>>().join("\n")
    );
    // Each kind of surface was exercised, so none passes vacuously.
    for place in [
        "panel step",
        "space popover",
        "space popover (refused)",
        "panel record",
        "map veil",
        "panel commit",
        "panel control",
        "transfer modal control",
        "transfer modal amount",
        "piece popover",
        "piece popover (refused)",
    ] {
        assert!(seen.contains(place), "{place} never presented a choice");
    }
}
