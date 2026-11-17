//! Foundation 2, "Engine authority": legal-action discovery stays complete for the
//! supported decision, even when presented hierarchically. During the Terror phase
//! every choice the authority offers to 26July, in every state `tests/honest_hints.rs`
//! explores, must have a surface that presents it: a panel step that pins and commits
//! it (or states its refusal), the commit or Kidnap's Cancel at the panel's foot, the
//! transfer modal, or a piece popover in removal mode.
#[path = "../../tests/support/explore.rs"]
mod explore;
use super::execution::{self, StepId};
use super::{Action, Interface, popover, transfer};
use crate::{ActionChoice, Choice, Faction, Game, Observation, targets};

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
        let amount = action
            .strip_prefix("give:")
            .map(|rest| (transfer::Direction::Give, rest))
            .or_else(|| {
                action
                    .strip_prefix("ask:")
                    .map(|rest| (transfer::Direction::Ask, rest))
            });
        if let Some((direction, rest)) = amount
            && let Some((other, n)) = rest.split_once(':')
            && typed(other.parse().unwrap_or(usize::MAX), direction, n).as_deref()
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
                            "{start}: {} ({}, supported {})",
                            choice.action(),
                            choice.label,
                            choice.supported
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
        "panel commit",
        "panel control",
        "transfer modal control",
        "transfer modal amount",
        "piece popover",
    ] {
        assert!(seen.contains(place), "{place} never presented a choice");
    }
}
