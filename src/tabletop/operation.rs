//! The Operation panel on the right: who decides now, what the current step means and
//! the choices the game offers. The game owns every fact; this module only words and
//! places them. The panel is rebuilt with the rest of the UI (`draw_ui`). At the
//! action choice the panel is the selected AC-B design (`choice.rs`, #69); during
//! Terror the BRIEFING ribbon reopens the Terror and Kidnap briefings.
//!
//! Temporary bootstrap (WORK-2; foundation 4 "Authoring boundary"). The panel frame
//! is the authored `atlas-panel.png`, but the execution button stack inside it (the
//! adaptive 44/52 px pitch) is an interim layout invented in code, not an authored
//! FigCraft design. #69 replaced the action choice's part of this bootstrap with the
//! selected AC-B layout. While the Kidnap die is presented (`die.rs`, #74) the stack
//! is not drawn: the AC-B panel's die-presented frame holds the tray, its result and
//! Continue Terror.
//! - Owner: #58.
//! - Purpose: let the development seat exercise Terror execution until the map
//!   popovers (#72's space popover now targets Terror and opening Kidnap; #73 adds
//!   removals) and the transfer modal (#71) replace the stack.
//! - Allowed consumers: agent verification of #58 only.
//! - Prohibited dependents: #16's owner review, any release, and any other UI
//!   positioning itself against this stack.
//! - Appetite: keep it working for #58's checks; no design or polish effort.
//! - Replacement: #72/#73 (map popovers for targets and removals) and #71 (transfer
//!   modal) replaced parts of it, and the rest is still this bootstrap. #73's removal
//!   mode replaced the list of removal groups: the stack now holds only Done there.
//!   #71 replaced the transfer part: the modal (`transfer.rs`) owns the give, ask,
//!   authorize, decline, Wait and Cancel choices, and this stack keeps only the button
//!   that opens it and 26July's withdrawals of a pending request. #72's space popover
//!   offers Terror and opening Kidnap on the map, and the stack still offers them too
//!   (#58 tests Terror from the panel). What remains in the stack during Terror: the
//!   Terror targets, Kidnap and its roll or cancel, Resource transfer, Remove own
//!   pieces, End Terror and Back to the action choice; no selected design replaces
//!   these yet.
//! - Expiry: before #75's end-to-end verification.
use super::*;

/// Draws the panel inside the authored operation frame. `offset` is the right-anchored
/// letterbox offset, so wide windows keep the panel at the right edge.
pub(super) fn panel(
    commands: &mut Commands,
    art: &Art,
    view: &crate::Observation,
    ui: &Interface,
    scale: f32,
    offset: Vec2,
) {
    let (s, o) = (scale, offset);
    // The selected blank component hides static example action affordances
    // after completion, so they cannot masquerade as current controls.
    picture(commands, art, "atlas-panel.png", PANEL, s, o, 2);
    if view.action == crate::ActionChoice::Open && !view.finished {
        return super::choice::panel(commands, art, view, ui, s, o);
    }
    if view.executing.is_some() {
        super::choice::ribbon(commands, art, s, o);
    }
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
    } else if view.kidnap_open {
        "26July · Kidnap opened".into()
    } else if let Some(crate::Transfer::Authorizing { giver, .. }) = &view.transfer {
        format!("Transfer request pending\n{} decides", giver.name())
    } else if view.transfer.is_some() {
        "26July · Resource transfer".into()
    } else if view.removal.is_some() {
        "26July · Voluntary removal".into()
    } else {
        format!("{} · {}", view.owner.name(), view.operation)
    }
}

/// The body below the title: the open transfer or removal step, otherwise the
/// Terror and Kidnap terms; a transfer under an owed Terror adds its reserve.
fn details(view: &crate::Observation) -> String {
    match view.action {
        crate::ActionChoice::Open => return String::new(),
        crate::ActionChoice::Pass => {
            let r = view.resources[Faction::M26.index()];
            return format!(
                "26July Resources: {} → {r} (R 2.3.3)\nNo Operation: no piece, Terror or support changed.",
                r - 1
            );
        }
        crate::ActionChoice::Terror => {}
    }
    if view.kidnap_open {
        return kidnap_details(view);
    }
    match &view.transfer {
        Some(crate::Transfer::Selecting) => "The transfer is open: give 26July's Resources\nto a faction, or ask one for Resources.\nA request is not a Resource transfer.".into(),
        Some(crate::Transfer::Authorizing { giver, amount, .. }) => format!(
            "26July asked {} for {amount}. Nothing moves until\n{} authorizes; Open in the top band shows\nits decision. 26July may withdraw the request.",
            giver.name(),
            giver.name()
        ),
        None if view.removal.is_some() => removal_details(view),
        None => terror_details(view),
    }
}

/// Removal mode (#73): the map marks the pieces and each piece's popover decides, so
/// the panel says how and keeps only Done.
fn removal_details(view: &crate::Observation) -> String {
    let mut sentences = vec![
        "Voluntary removal (R 1.4.1). Each 26July piece on the map carries a mark: a red ring where it may be removed.".to_owned(),
        "Click a ringed piece to see what removing it changes, choose how many from a stack, then Remove or Cancel.".to_owned(),
        "Removal stays open for more pieces until Done. Earlier Terror markers, support shifts and Kidnap results stay.".to_owned(),
    ];
    sentences.push(if view.owed_terror.is_some() {
        "Santiago Terror is owed: its last Underground Guerrilla carries the ink lock and stays (accepted commitment policy, local).".into()
    } else {
        "No Terror is owed: any own piece may go.".into()
    });
    sentences
        .iter()
        .flat_map(|s| super::preview::wrap(s, 48))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A choice the transfer modal shows and submits, not this stack.
fn modal_owned(choice: &crate::Choice) -> bool {
    let action = choice.action();
    ["give:", "ask:", "authorize:"]
        .iter()
        .any(|p| action.starts_with(p))
        || matches!(action, "back" | "decline" | "wait")
}

/// One button per offered choice, stacked below the body. A choice the game marks
/// unsupported still appears, and activating it shows its explanation. In removal mode
/// the pieces' popovers offer the removals, so only Done is a button.
fn choices(commands: &mut Commands, art: &Art, view: &crate::Observation, s: f32, o: Vec2) {
    let shown: Vec<&crate::Choice> = view
        .choices
        .iter()
        .filter(|c| c.removal().is_none() && !modal_owned(c))
        .collect();
    // The stack ends at the panel's foot. It tightens once it holds more than five
    // buttons, such as the two disabled Terror targets. It is the execution part of
    // this module's WORK-2 bootstrap (record above), which expires before #75; #69's
    // AC-B design replaced only the action choice's part.
    let count = shown.len() as f32;
    let pitch = if count > 5. { 44. } else { 52. };
    let top = 926. - count * pitch;
    for (i, choice) in shown.into_iter().enumerate() {
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

/// An opened Kidnap's stakes, from the roll's preview: the target, the possible
/// proceeds and their guaranteed minimum, the owed Terror and card 17's Event. The
/// pointer on Roll shows the full preview.
fn kidnap_details(view: &crate::Observation) -> String {
    let Some(stakes) = view
        .choices
        .iter()
        .find(|c| c.id.ends_with(":kidnap:roll"))
        .and_then(|c| c.preview.as_ref())
        .and_then(|p| p.kidnap.as_ref())
    else {
        return "Kidnap is open; its roll is unavailable now.".into();
    };
    let mut sentences = vec![
        "Kidnap (Special Activity, R 4.1, 4.3.3). Nothing is rolled yet.".to_owned(),
        format!(
            "Target: {} in {}, which has {}.",
            stakes.target.name(),
            view.spaces
                .iter()
                .find(|s| s.id == stakes.space)
                .map_or(stakes.space.as_str(), |s| s.name.as_str()),
            units(stakes.government)
        ),
        format!(
            "A die 1–6 takes {} · guaranteed minimum {}.",
            super::preview::proceeds(stakes),
            units(stakes.minimum)
        ),
    ];
    if stakes.commits_santiago {
        sentences.push("Rolling owes Santiago Terror (accepted commitment policy): keep 1 Resource and its last Underground Guerrilla; End Terror waits for it.".into());
    }
    sentences.push(format!(
        "After End Terror, Directorio may execute card 17's Event (R 2.3.4, 5.2), shaded: \"{}\"",
        crate::CARD_17_SHADED
    ));
    sentences.push(
        "The roll is saved before it is shown. Cancel rolls nothing and changes nothing.".into(),
    );
    sentences
        .iter()
        .flat_map(|s| super::preview::wrap(s, 48))
        .collect::<Vec<_>>()
        .join("\n")
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
            "Kidnap (Special Activity, R 4.1, 4.3.3): with\nTerror in Santiago, a die 1–6 takes {}–{}\nResources from Government. Open it to see\nthe stakes; nothing rolls until Roll.",
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

    /// GLOSSARY.md: "Voluntary removal" and "Resource transfer", never their
    /// _Avoid_ terms "donation" or "voluntary own removal".
    #[test]
    fn transfer_and_removal_steps_use_the_glossary_terms() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:transfer").unwrap();
        let transfer = details(&game.observe());
        assert!(
            transfer.contains("A request is not a Resource transfer."),
            "{transfer}"
        );
        assert!(!transfer.to_lowercase().contains("donat"), "{transfer}");
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:remove").unwrap();
        assert_eq!(heading(&game.observe()), "26July · Voluntary removal");
    }

    #[test]
    fn an_opened_kidnap_states_its_stakes_from_the_rules_authority() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        let view = game.observe();
        assert_eq!(heading(&view), "26July · Kidnap opened");
        let details = details(&view);
        let words = details.replace('\n', " ");
        for fact in [
            "Target: Government in Santiago de Cuba,",
            "which has 6 Resources",
            "takes 1–6 Resources",
            "guaranteed minimum 1 Resource",
            "accepted commitment policy",
            "R 2.3.4, 5.2",
            "Cancel rolls nothing",
            "Nothing is rolled yet.",
        ] {
            assert!(words.contains(fact), "{fact}: {details}");
        }
        assert!(
            details.lines().all(|l| l.chars().count() <= 48),
            "{details}"
        );
    }
}
