//! The Operation panel on the right: who decides now, what the current step means and
//! the choices the game offers. The game owns every fact; this module only words and
//! places them. The panel is rebuilt with the rest of the UI (`draw_ui`).
use super::*;

/// Draws the panel inside the authored operation frame. `offset` is the right-anchored
/// letterbox offset, so wide windows keep the panel at the right edge.
pub(super) fn panel(
    commands: &mut Commands,
    art: &Art,
    view: &crate::Observation,
    scale: f32,
    offset: Vec2,
) {
    let (s, o) = (scale, offset);
    // The selected blank component hides static example action affordances
    // after completion, so they cannot masquerade as current controls.
    picture(
        commands,
        art,
        "atlas-panel.png",
        [1380., 287., 500., 653.],
        s,
        o,
        2,
    );
    text(
        commands,
        art,
        Prose::named(heading(view)),
        1404.,
        330.,
        450.,
        25.,
        s,
        o,
        3,
    );
    text(
        commands,
        art,
        details(view),
        1404.,
        390.,
        450.,
        18.,
        s,
        o,
        3,
    );
    if !view.finished {
        choices(commands, art, view, s, o);
    } else {
        text(commands, art, next(view), 1404., 590., 450., 25., s, o, 3);
    }
}

/// The panel's title: the deciding faction and its step, or the finished record.
fn heading(view: &crate::Observation) -> String {
    if view.action == crate::ActionChoice::Pass {
        "26July passed\n+1 Resource · stays Eligible".into()
    } else if view.finished {
        format!(
            "26July ended Terror\n{} recorded",
            if view.kidnap.is_some() {
                "Operation with Special Activity"
            } else {
                "Operation Only"
            }
        )
    } else if view.transfer.is_some() {
        format!("Resource transfer\n{} decides locally", view.owner.name())
    } else if view.removal.is_some() {
        "26July · voluntary own removal".into()
    } else {
        format!("{} · {}", view.owner.name(), view.operation)
    }
}

/// The body below the title: the open transfer or removal step, otherwise the
/// Terror and Kidnap terms; a transfer under an owed Terror adds its reserve.
fn details(view: &crate::Observation) -> String {
    match view.action {
        crate::ActionChoice::Open => return opening_details(view),
        crate::ActionChoice::Pass => {
            let r = view.resources[Faction::M26.index()];
            return format!(
                "26July Resources: {} → {r} (R 2.3.3)\nNo Operation: no piece, Terror or support changed.",
                r - 1
            );
        }
        crate::ActionChoice::Terror => {}
    }
    let body = match &view.transfer {
        Some(crate::Transfer::Selecting) => "Choose the giver and recipient.\n26July must be one endpoint.\nThe giver chooses the quantity and authorizes.\nA request is not a donation.".into(),
        Some(crate::Transfer::Authorizing { giver, recipient, amount }) => format!(
            "Giver: {} · balance {}\nRecipient: {} · balance {}\nAmount: {} Resources · whole numbers only\nAfter authorization: {} → {}; {} → {}\nZero is allowed. Decline moves nothing.",
            giver.name(), view.resources[giver.index()], recipient.name(), view.resources[recipient.index()], amount,
            giver.name(), view.resources[giver.index()] - amount, recipient.name(), view.resources[recipient.index()] + amount),
        None => match &view.removal {
            Some(crate::Removal::Selecting) => format!("Voluntary removal (R 1.4.1): choose one own piece\nto return to Available. Point at a piece's button\nfor its preview; confirmation comes next.\n{}", if view.owed_terror.is_some() { "Santiago Terror is owed: its last Underground\nGuerrilla stays (accepted commitment policy)." } else { "No Terror is owed: any own piece may go." }),
            Some(crate::Removal::Confirming { space, kind, state }) => {
                let target = view.spaces.iter().find(|s| &s.id == space).unwrap();
                let supply = view.available.iter().find(|p| p.faction == Faction::M26 && p.kind == *kind).unwrap().count;
                format!("Remove 1 26July {state:?} {kind:?}\nfrom {}?\nAvailable {kind:?}: {supply} → {}\nPoint at Confirm for its preview.\nEarlier Terror and Opposition stay.\nCancel leaves this piece on the map.", target.name, supply + 1)
            }
            None => terror_details(view),
        },
    };
    body + &if view.transfer.is_some() && view.owed_terror.is_some() {
        let owed = view
            .spaces
            .iter()
            .find(|space| Some(&space.id) == view.owed_terror.as_ref())
            .unwrap();
        format!(
            "\nCommitment: {} Terror owed; reserve 1.\n26July available to give: {} Resources.",
            owed.name,
            view.resources[Faction::M26.index()].saturating_sub(1)
        )
    } else {
        String::new()
    }
}

/// One button per offered choice, stacked below the body. A choice the game marks
/// unsupported still appears, and activating it shows its explanation.
fn choices(commands: &mut Commands, art: &Art, view: &crate::Observation, s: f32, o: Vec2) {
    if view.action == crate::ActionChoice::Open {
        return opening_choices(commands, art, view, s, o);
    }
    // The stack ends at the panel's foot. It tightens once it holds more than five
    // buttons, such as the two disabled Terror targets; the selected design (#69)
    // replaces this temporary stack.
    let count = view.choices.len() as f32;
    let pitch = if count > 5. { 44. } else { 52. };
    let top = 926. - count * pitch;
    for (i, choice) in view.choices.iter().enumerate() {
        control(
            commands,
            art,
            &choice.id,
            &if !choice.supported {
                format!("{} · unavailable", choice.label)
            } else {
                choice.label.clone()
            },
            if choice.supported {
                Action::Decision(choice.owner, view.revision)
            } else {
                Action::Unsupported(refusal(choice))
            },
            [1404., top + i as f32 * pitch, 452., pitch - 6.],
            s,
            o,
            choice.supported,
            3,
        );
    }
}

/// A Resource count with its unit.
fn units(n: u8) -> String {
    format!("{n} Resource{}", if n == 1 { "" } else { "s" })
}

/// What activating a disabled choice reports: its reason and in-task remedies.
fn refusal(choice: &crate::Choice) -> String {
    if choice.remedies.is_empty() {
        choice.explanation.clone()
    } else {
        format!(
            "{} In this task: {}.",
            choice.explanation,
            choice.remedies.join(" · ")
        )
    }
}

/// What the finished Operation leaves Directorio, shown in place of the choices.
fn next(view: &crate::Observation) -> String {
    if view.action == crate::ActionChoice::Pass {
        return format!(
            "Directorio is now 1st Eligible (shown, not executed):\n{}\n\nThe card and Eligibility have not advanced.",
            view.next_options.join("\n")
        );
    }
    format!(
        "Directorio, 2nd Eligible, may (shown, not executed):\n{}\n\n{}\nThe card and Eligibility have not advanced.",
        view.next_options.join("\n"),
        if view.kidnap.is_some() {
            "After a Special Activity, card 17's Event\nis open to Directorio (R 2.3.4, 5.2)."
        } else {
            "Operation Only: no Event for Directorio\n(R 2.3.4)."
        }
    )
}

/// The action choice's body: card 17's order, 26July's Resources and what this task
/// supports.
fn opening_details(view: &crate::Observation) -> String {
    let order = view.eligibility.order.map(Faction::name).join(", ");
    format!(
        "Order: {order}\n26July has {} Resources · all four Eligible\nPlayable here: Pass and Terror",
        view.resources[Faction::M26.index()]
    )
}

/// Every listed option in one compact stack; a Special Activity sits indented under
/// the Operation it accompanies.
fn opening_choices(commands: &mut Commands, art: &Art, view: &crate::Observation, s: f32, o: Vec2) {
    for (i, choice) in view.choices.iter().enumerate() {
        let indent = if choice.accompanies.is_some() {
            36.
        } else {
            0.
        };
        control(
            commands,
            art,
            &choice.id,
            &choice.label,
            if choice.supported {
                Action::Decision(choice.owner, view.revision)
            } else {
                Action::Unsupported(refusal(choice))
            },
            [1404. + indent, 480. + i as f32 * 40., 452. - indent, 36.],
            s,
            o,
            choice.supported,
            3,
        );
    }
}

/// The Terror and Kidnap terms with units and rule references, or the resolved record
/// once Terror has ended. The map shows the pieces; each button's preview shows what
/// it changes.
fn terror_details(view: &crate::Observation) -> String {
    let m26 = view.resources[Faction::M26.index()];
    let resolved: Vec<&str> = view
        .spaces
        .iter()
        .filter(|s| s.terror > 0)
        .map(|s| s.name.as_str())
        .collect();
    let mut lines = vec![];
    if view.finished {
        lines.push(if resolved.is_empty() {
            "No Terror resolved: no effect, no Pass income.".into()
        } else {
            format!(
                "Terror resolved: {} · paid {}",
                resolved.join(", "),
                units(resolved.len() as u8)
            )
        });
    } else {
        lines.push(format!("Cost: 1 Resource per space (26July has {m26})"));
        lines.push("Terror (R 3.3.4), in each selected space:".into());
        lines.push("flip 1 Underground Guerrilla there to Active,".into());
        lines.push("place a Terror marker and shift 1 level toward".into());
        lines.push("Active Opposition.".into());
        if !resolved.is_empty() {
            lines.push(format!("Resolved: {}", resolved.join(", ")));
        }
    }
    if let Some(roll) = &view.kidnap {
        lines.push(format!(
            "Kidnap rolled {} · {} (Government → 26July)",
            roll.die,
            units(roll.proceeds)
        ));
    } else if !view.finished {
        lines.push(format!(
            "Kidnap (Special Activity, R 4.1, 4.3.3): with\nTerror in Santiago, a die 1–6 takes {}–{}\nResources from Government.",
            view.resources[0].min(1),
            view.resources[0].min(6)
        ));
    }
    if view.owed_terror.is_some() {
        lines.push("Santiago Terror owed (accepted commitment policy):\nkeep 1 Resource and 1 Underground Guerrilla there.".into());
    }
    if view.executing.is_some() {
        lines.push("Resource transfers open: 26July is executing Terror.".into());
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_text_spells_out_units_and_activation_without_piece_counts() {
        let details = terror_details(&Game::prepared().observe());
        assert!(
            details.contains("Cost: 1 Resource per space (26July has 0)"),
            "{details}"
        );
        assert!(
            details.contains("flip 1 Underground Guerrilla there"),
            "{details}"
        );
        assert!(details.contains("R 3.3.4"), "{details}");
        assert!(!details.contains("Police"), "{details}");
        assert!(!details.contains("Activate 1"), "{details}");
        assert!(!details.contains("Full Terror"), "{details}");
    }
}
