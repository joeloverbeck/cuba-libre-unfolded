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
            "26July completed {}\nNo Pass income",
            if view.kidnap.is_some() {
                "Operation + Kidnap"
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
            Some(crate::Removal::Selecting) => format!("Choose one own piece to return to Available.\nCounts show how many remain in each group.\nConfirmation comes next; Cancel removes nothing.\n{}", if view.owed_terror.is_some() { "Owed Santiago: protect its last Underground." } else { "No obligation: removing all pieces is allowed." }),
            Some(crate::Removal::Confirming { space, kind, state }) => {
                let target = view.spaces.iter().find(|s| &s.id == space).unwrap();
                let supply = view.available.iter().find(|p| p.faction == Faction::M26 && p.kind == *kind).unwrap().count;
                format!("Remove 1 26July {state:?} {kind:?}\nfrom {}?\nAvailable {kind:?}: {supply} → {}\nControl and standings update immediately.\nEarlier Terror and Opposition stay.\nCancel leaves this piece on the map.", target.name, supply + 1)
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
    let top = if view.transfer.is_some() {
        570.
    } else if view.removal.is_some() {
        540.
    } else {
        590.
    };
    for (i, choice) in view.choices.iter().enumerate() {
        control(
            commands,
            art,
            &choice.id,
            &if !choice.supported {
                format!("{} — unavailable", choice.label)
            } else {
                choice.label.clone()
            },
            if choice.supported {
                Action::Decision(choice.owner, view.revision)
            } else {
                Action::Unsupported(choice.explanation.clone())
            },
            [1404., top + i as f32 * 52., 452., 46.],
            s,
            o,
            choice.supported,
            3,
        );
    }
}

/// What the finished Operation leaves Directorio, shown in place of the choices.
fn next(view: &crate::Observation) -> String {
    if view.action == crate::ActionChoice::Pass {
        return format!(
            "Directorio is now 1st Eligible (display only):\n{}\n\nCard / eligibility have not advanced.",
            view.next_options.join("\n")
        );
    }
    format!(
        "Directorio next (display only):\n{}\n\n{}\nCard / eligibility have not advanced.",
        view.next_options.join("\n"),
        if view.kidnap.is_some() {
            "Kidnap grants Event permission."
        } else {
            "No Event permission."
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
                Action::Unsupported(choice.explanation.clone())
            },
            [1404. + indent, 480. + i as f32 * 40., 452. - indent, 36.],
            s,
            o,
            choice.supported,
            3,
        );
    }
}

/// The Terror and Kidnap terms for Santiago and Oriente, or the resolved record once
/// the Operation is finished.
fn terror_details(view: &crate::Observation) -> String {
    let mut lines = vec![];
    for id in ["santiago", "oriente"] {
        let space = view.spaces.iter().find(|s| s.id == id).unwrap();
        let count = |faction, kind, state: Option<State>| -> u8 {
            space
                .pieces
                .iter()
                .filter(|p| {
                    p.faction == faction && p.kind == kind && state.is_none_or(|s| p.state == s)
                })
                .map(|p| p.count)
                .sum()
        };
        lines.push(format!(
            "{}: {} Underground · {} Active · {} Police{}",
            if id == "santiago" {
                "Santiago"
            } else {
                "Oriente"
            },
            count(Faction::M26, Kind::Guerrilla, Some(State::Underground)),
            count(Faction::M26, Kind::Guerrilla, Some(State::Active)),
            count(Faction::Government, Kind::Police, None),
            if space.terror > 0 { " · done" } else { "" }
        ));
    }
    if view.finished {
        let paid: u8 = view.spaces.iter().map(|s| s.terror).sum();
        // Balances are on the top display's Resources ledger (#46); costs stay here.
        lines.push(format!("Resolved {paid} Terror · paid {paid} Resources"));
    } else {
        lines.push("Terror costs 1".into());
        lines.push("Activate 1 · place Terror · shift toward Active Opposition".into());
    }
    if let Some(roll) = &view.kidnap {
        lines.push(format!(
            "Kidnap rolled {} · moved {} (Government → 26July)",
            roll.die, roll.proceeds
        ));
        lines.push(format!(
            "At roll: Government {} → {} · 26July {} → {}",
            roll.government_before,
            roll.government_before - roll.proceeds,
            roll.m26_before,
            roll.m26_before + roll.proceeds
        ));
    } else if view.finished {
        lines.push("No Kidnap used.".into());
    } else {
        lines.push(format!(
            "Kidnap die 1–6 · possible proceeds {}–{}",
            view.resources[0].min(1),
            view.resources[0].min(6)
        ));
        lines.push("Santiago: total Guerrillas must outnumber Police".into());
        if view
            .spaces
            .iter()
            .any(|s| s.id == "santiago" && s.terror == 0)
        {
            lines.push("Early Kidnap commits Santiago Terror (pay 1).".into());
        }
    }
    if !view.finished {
        lines.push(
            if view.owed_terror.is_some() {
                "Owed: Santiago · keep 1 Resource + 1 Underground"
            } else {
                "No Terror owed · Finish permitted"
            }
            .into(),
        );
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_text_keeps_costs_and_leaves_balances_to_the_ledger() {
        let details = terror_details(&Game::prepared().observe());
        assert!(details.contains("Terror costs 1\n"), "{details}");
        assert!(!details.contains("26July Resources"), "{details}");
        assert!(!details.contains("retains"), "{details}");
    }
}
