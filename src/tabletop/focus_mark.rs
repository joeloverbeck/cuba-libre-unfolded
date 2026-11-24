//! The focus mark (#179): selected A · Doble filete, an ink line outside a control
//! across a paper gap. It is on the control Enter would press and on no other, whatever
//! the pointer rests on. In the transfer modal Enter presses the primary button while
//! nothing has focus and while the amount field has it; the field then has the light
//! form. Map spaces, cards, piles, the marker supply and the Available Forces rows have
//! no mark: Enter commits nothing there, and their tags show focus.
//!
//! The plates are the VectorCraft exports of `art/atlas/atlas-focus-mark.vectorcraft`
//! (`focus-mark.sha256`), nine-sliced around the control they mark (FOCUS-MARK.md).
use super::*;
use crate::Observation;
use bevy::ui::widget::NodeImageMode;

/// The mark, and the light form for the modal's amount field: 24×24 reference pixels,
/// exported at 3×.
const FULL: &str = "atlas-focus-mark.png";
const LIGHT: &str = "atlas-focus-mark-field.png";
/// The plates' corner slices and how far they reach past the control, in reference
/// pixels. Two option rows are 4.1 apart.
const CORNER: f32 = 9.;
const OUTSET: f32 = 4.;
/// A popover's row is as wide as its card, so its mark lies inside it, this far in.
const ROW_INSET: f32 = 1.;

/// A drawn mark, a child of the control it marks.
#[derive(Component, Clone, Copy, PartialEq)]
pub(super) struct FocusMark {
    /// The control's node.
    on: Entity,
    light: bool,
    /// The reference pixel it was drawn at.
    s: f32,
}

/// The controls that are marked, by their seat names.
#[derive(Default, Debug, PartialEq)]
struct Marked {
    /// The control Enter would press.
    pressed: Option<String>,
    /// The amount field, while Tab has reached it.
    field: Option<String>,
}

/// Whether a control Enter would press carries the mark. The others' tags show focus.
fn carries(action: &Action) -> bool {
    !matches!(
        action,
        Action::Space(_) | Action::Card(_) | Action::Pile(_) | Action::Top(_)
    )
}

/// What is marked, from the same orders and defaults that the keyboard reads. A card in
/// the air (`flying`) and a presented die take Enter before any control does.
fn marked(
    view: &Observation,
    ui: &Interface,
    targeting: &highlights::Targeting,
    popover: &popover::Popover,
    roll: &die::Roll,
    flying: bool,
) -> Marked {
    if flying {
        return Marked::default();
    }
    if die::presented(view.revision, view.kidnap.as_ref(), roll).is_some() {
        return Marked {
            pressed: roll.landed().then(|| die::CONTINUE.to_owned()),
            field: None,
        };
    }
    if transfer::shown(view.transfer.as_ref()) {
        return Marked {
            pressed: Some(transfer::pressed(view, ui)),
            field: ui
                .transfer
                .focus
                .clone()
                .filter(|focus| focus == transfer::FIELD),
        };
    }
    Marked {
        pressed: choice::pressed(view, ui, targeting, popover)
            .filter(|(_, action)| carries(action))
            .map(|(name, _)| name),
        field: None,
    }
}

/// Keeps the mark on the control Enter would press, as focus moves and as the control
/// is drawn again.
#[allow(clippy::too_many_arguments)] // What the keyboard reads, and the drawn controls.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    targeting: Res<highlights::Targeting>,
    popover: Res<popover::Popover>,
    roll: Res<die::Roll>,
    advance: Res<piles::Advance>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    controls: Query<(Entity, &Control), With<Node>>,
    marks: Query<(Entity, &FocusMark)>,
) {
    // A decision made in this frame is targeted in the next.
    let view = targeting.view.as_ref().filter(|view| {
        play.session
            .as_ref()
            .is_some_and(|s| s.game().revision == view.revision)
    });
    let wanted = view.map_or_else(Marked::default, |view| {
        let flying = advance.in_the_air(view, time.elapsed_secs_f64());
        marked(view, &ui, &targeting, &popover, &roll, flying)
    });
    let s = reference_pixel(&window);
    let mut missing = vec![];
    for (name, light) in [(&wanted.pressed, false), (&wanted.field, true)] {
        let Some(name) = name else { continue };
        if let Some((on, control)) = controls.iter().find(|(_, c)| &c.name == name) {
            missing.push((FocusMark { on, light, s }, control));
        }
    }
    for (entity, mark) in &marks {
        if let Some(at) = missing.iter().position(|(wanted, _)| wanted == mark) {
            missing.swap_remove(at);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (mark, control) in missing {
        let edge = match control.action {
            Action::Row(_) => px(ROW_INSET * s),
            _ => px(-OUTSET * s),
        };
        commands.spawn((
            ImageNode::new(art.images[if mark.light { LIGHT } else { FULL }].clone()).with_mode(
                NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(CORNER * 3.),
                    max_corner_scale: s / 3.,
                    ..default()
                }),
            ),
            Node {
                position_type: PositionType::Absolute,
                left: edge,
                top: edge,
                right: edge,
                bottom: edge,
                ..default()
            },
            Pickable::IGNORE,
            mark,
            ChildOf(mark.on),
        ));
    }
}

/// The drawn marks, for the seat: the control each is on, its form and its rectangle.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let marks: Vec<_> = world
        .query::<(
            &FocusMark,
            &ChildOf,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            &InheritedVisibility,
        )>()
        .iter(world)
        .filter(|(.., shown)| shown.get())
        .map(|(mark, parent, node, at, _)| {
            let scale = node.inverse_scale_factor();
            let half = node.size() * scale / 2.;
            let middle = at.translation * scale;
            let rect = Rect::from_center_half_size(middle, half);
            (mark.light, parent.parent(), rect)
        })
        .collect();
    marks
        .into_iter()
        .map(|(light, on, r)| {
            let control = world.get::<Control>(on).map(|c| c.name.clone());
            serde_json::json!({
                "control": control,
                "form": if light { "light" } else { "full" },
                "bounds": [r.min.x, r.min.y, r.max.x, r.max.y],
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    /// A match played to the revision after `steps`, each decided by 26July.
    fn after(steps: &[&str]) -> Observation {
        let mut game = Game::opening();
        for step in steps {
            let at = game.revision;
            game.submit(Faction::M26, at, &format!("{at}:{step}"))
                .unwrap();
        }
        game.observe()
    }
    fn aim(view: &Observation) -> highlights::Targeting {
        let mut targeting = highlights::Targeting::default();
        targeting.targets = crate::targets::space_targets(view);
        targeting.view = Some(view.clone());
        targeting
    }
    fn mark(view: &Observation, ui: &Interface) -> Marked {
        let popover = popover::Popover::default();
        marked(view, ui, &aim(view), &popover, &die::Roll::default(), false)
    }
    fn pressed(name: &str) -> Marked {
        Marked {
            pressed: Some(name.into()),
            field: None,
        }
    }

    /// Outside the modal Enter presses the focused control alone, so nothing is marked
    /// without focus, or while focus names a control that is not there.
    #[test]
    fn the_focused_control_is_marked_while_it_is_a_stop() {
        let view = after(&[]);
        let mut ui = Interface::default();
        assert_eq!(mark(&view, &ui), Marked::default());
        for name in ["option:pass", "option:rally", "quit"] {
            ui.choice.focus = Some(name.into());
            assert_eq!(mark(&view, &ui), pressed(name));
        }
        ui.choice.focus = Some("popover:commit".into());
        assert_eq!(mark(&view, &ui), Marked::default());
    }

    /// The controls whose tags show focus, and where Enter commits nothing, have no
    /// mark, though they are stops.
    #[test]
    fn spaces_cards_piles_and_ledgers_are_not_marked() {
        let view = after(&["terror"]);
        let mut ui = Interface::default();
        for name in [
            "space:santiago",
            "marker-supply",
            "available:government",
            "card:current",
            "pile:deck",
        ] {
            ui.choice.focus = Some(name.into());
            let popover = popover::Popover::default();
            assert!(
                choice::pressed(&view, &ui, &aim(&view), &popover).is_some(),
                "{name} is a stop"
            );
            assert_eq!(mark(&view, &ui), Marked::default(), "{name}");
        }
    }

    /// An open popover's row and button are marked; once it closes they are not. Its
    /// button keeps its name, so a popover opened later has focus on its own button:
    /// Enter presses that one, and it is marked.
    #[test]
    fn the_popover_s_button_is_marked_while_a_popover_shows_it() {
        let view = after(&["terror"]);
        let targeting = aim(&view);
        let mut popover = popover::Popover::default();
        let roll = die::Roll::default();
        let mut ui = Interface::default();
        popover.click_space("santiago", &targeting);
        for name in ["popover:row:kidnap", "popover:commit"] {
            ui.choice.focus = Some(name.into());
            let at = marked(&view, &ui, &targeting, &popover, &roll, false);
            assert_eq!(at, pressed(name));
        }
        popover.close_all();
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, false),
            Marked::default()
        );
        popover.click_space("santiago", &targeting);
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, false),
            pressed("popover:commit")
        );
    }

    /// In the modal Enter presses the primary button unless another control than the
    /// amount field has focus. The field then has the light form beside it.
    #[test]
    fn the_modal_s_primary_is_marked_until_another_control_has_focus() {
        let view = after(&["terror", "transfer"]);
        let mut ui = Interface::default();
        assert_eq!(mark(&view, &ui), pressed("transfer:primary"));
        ui.transfer.focus = Some(transfer::FIELD.into());
        assert_eq!(
            mark(&view, &ui),
            Marked {
                pressed: Some("transfer:primary".into()),
                field: Some(transfer::FIELD.into()),
            }
        );
        ui.transfer.focus = Some("2:back".into());
        assert_eq!(mark(&view, &ui), pressed("2:back"));
        // Focus elsewhere on the table does not reach through the modal.
        ui.transfer.focus = None;
        ui.choice.focus = Some("quit".into());
        assert_eq!(mark(&view, &ui), pressed("transfer:primary"));
    }

    /// A card in the air takes Enter, and so does the die: in the air it only lands,
    /// and once landed Enter presses Continue, whatever had focus.
    #[test]
    fn a_card_or_die_in_the_air_leaves_nothing_to_press() {
        let view = after(&["terror", "kidnap", "kidnap:roll"]);
        let targeting = aim(&view);
        let popover = popover::Popover::default();
        let mut ui = Interface::default();
        ui.choice.focus = Some("quit".into());
        let mut roll = die::Roll::default();
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, false),
            Marked::default()
        );
        roll.land();
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, false),
            pressed(die::CONTINUE)
        );
        roll.close(view.revision);
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, false),
            pressed("quit")
        );
        assert_eq!(
            marked(&view, &ui, &targeting, &popover, &roll, true),
            Marked::default()
        );
    }
}
