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
use bevy::input_focus::{FocusCause, InputFocus};

/// A plate and its corner slice in reference pixels: the mark, its light form for the
/// modal's amount field, and its cut for the round segments of the modal's direction
/// toggle.
const FULL: (&str, f32) = ("atlas-focus-mark.png", 9.);
const LIGHT: (&str, f32) = ("atlas-focus-mark-field.png", 9.);
const PILL: (&str, f32) = ("atlas-focus-mark-pill.png", 19.);
/// How far the plates reach past the control, in reference pixels. Two option rows are
/// 4.1 apart.
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

/// The controls that are marked.
#[derive(Default, Debug, PartialEq)]
struct Marked {
    /// The control Enter would press.
    pressed: Option<Target>,
    /// Whether Tab has reached the amount field.
    field: bool,
}
/// A control to mark: one by its seat name, or the modal's primary button, whose name
/// follows the amount before it is drawn again.
#[derive(Debug, PartialEq)]
enum Target {
    Control(String),
    Primary,
}
impl Target {
    fn is(&self, control: &Control, part: Option<&transfer::Part>) -> bool {
        match self {
            Self::Control(name) => &control.name == name,
            Self::Primary => part.is_some_and(|part| part.0 == transfer::PRIMARY_PART),
        }
    }
}

/// Whether a control Enter would press carries the mark. The others' tags show focus.
fn carries(action: &Action) -> bool {
    !matches!(
        action,
        Action::Space(_) | Action::Card(_) | Action::Pile(_) | Action::Top(_)
    )
}

/// What is marked, from the same orders and defaults that the keyboard reads. A card in
/// the air (`flying`), an open edition and a presented die take Enter before any control
/// does.
fn marked(
    view: &Observation,
    ui: &Interface,
    targeting: &highlights::Targeting,
    popover: &popover::Popover,
    roll: &die::Roll,
    flying: bool,
) -> Marked {
    if flying || ui.editions.open().is_some() {
        return Marked::default();
    }
    if die::presented(view.revision, view.kidnap.as_ref(), roll).is_some() {
        return Marked {
            pressed: roll
                .landed()
                .then(|| Target::Control(die::CONTINUE.to_owned())),
            field: false,
        };
    }
    let dispatch = proclamation_composer::entry_available(view, ui, roll, flying);
    if transfer::shown(view.transfer.as_ref(), view.owner, ui) {
        return Marked {
            pressed: Some(match transfer::pressed(view, ui, dispatch) {
                transfer::Pressed::Primary(_) => Target::Primary,
                transfer::Pressed::Other(name) => Target::Control(name),
            }),
            field: ui.transfer.focus.as_deref() == Some(transfer::FIELD),
        };
    }
    Marked {
        pressed: choice::pressed(view, ui, targeting, popover, dispatch)
            .filter(|(_, action)| carries(action))
            .map(|(name, _)| Target::Control(name)),
        field: false,
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
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    controls: Query<(Entity, &Control, Option<&transfer::Part>), With<Node>>,
    marks: Query<(Entity, &FocusMark)>,
    mut input_focus: ResMut<InputFocus>,
    mut dispatch_focus: Local<Option<Entity>>,
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
    // The Dispatch's accessible button follows the same Tab stop as its drawn mark.
    let dispatch = wanted
        .pressed
        .as_ref()
        .filter(|target| matches!(target, Target::Control(name) if name == proclamation_composer::WRITE))
        .and_then(|target| {
            controls
                .iter()
                .find_map(|(entity, control, part)| target.is(control, part).then_some(entity))
        });
    let (primary, window) = window.into_inner();
    let focused = input_focus.get();
    if let Some(entity) = dispatch {
        if focused != Some(entity) {
            input_focus.set(entity, FocusCause::Navigated);
        }
    } else if focused.is_some_and(|entity| entity == primary || Some(entity) == *dispatch_focus) {
        // Bevy 0.20's Winit accessibility update rejects explicit focus on the
        // window entity. With focus empty, Bevy still dispatches keys to that window.
        input_focus.clear();
    }
    *dispatch_focus = dispatch;
    let s = reference_pixel(window);
    let field = wanted
        .field
        .then(|| Target::Control(transfer::FIELD.to_owned()));
    let mut missing = vec![];
    for (target, light) in [(wanted.pressed, false), (field, true)] {
        let Some(target) = target else { continue };
        if let Some((on, control, _)) = controls.iter().find(|(_, c, part)| target.is(c, *part)) {
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
        let round = matches!(
            control.action,
            Action::Transfer(transfer::Input::Direction(_))
        );
        let (plate, corner) = match (mark.light, round) {
            (true, _) => LIGHT,
            (false, true) => PILL,
            (false, false) => FULL,
        };
        let outset = match control.action {
            Action::Row(_) => -ROW_INSET,
            _ => OUTSET,
        };
        let plate = (plate, Some(corner), [outset; 4]);
        let drawn = popover::plate(&mut commands, &art, mark.on, plate, s);
        commands.entity(drawn).insert(mark);
        if round {
            // The toggle's segments touch: the mark lies over its neighbour's end.
            commands
                .entity(drawn)
                .insert(GlobalZIndex(transfer::LAYER + 1));
        }
    }
}

/// The drawn marks, for the seat: the control each is on, its form and its rectangle.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let marks: Vec<_> = world
        .query::<(
            &FocusMark,
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            &InheritedVisibility,
        )>()
        .iter(world)
        .filter(|(.., shown)| shown.get())
        .map(|(mark, node, at, _)| (*mark, seat::logical_bounds(node, at)))
        .collect();
    marks
        .into_iter()
        .map(|(mark, bounds)| {
            serde_json::json!({
                "control": world.get::<Control>(mark.on).map(|c| c.name.clone()),
                "form": if mark.light { "light" } else { "full" },
                "bounds": bounds,
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
            pressed: Some(Target::Control(name.into())),
            field: false,
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
                choice::pressed(&view, &ui, &aim(&view), &popover, false).is_some(),
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
        let primary = |field| Marked {
            pressed: Some(Target::Primary),
            field,
        };
        let view = after(&["terror", "transfer"]);
        let mut ui = Interface::default();
        assert_eq!(mark(&view, &ui), primary(false));
        ui.transfer.focus = Some(transfer::FIELD.into());
        assert_eq!(mark(&view, &ui), primary(true));
        ui.transfer.focus = Some("2:back".into());
        assert_eq!(mark(&view, &ui), pressed("2:back"));
        // Focus elsewhere on the table does not reach through the modal.
        ui.transfer.focus = None;
        ui.choice.focus = Some("quit".into());
        assert_eq!(mark(&view, &ui), primary(false));
    }

    /// A dragged slider changes the choice Enter would submit at once, and the drawn
    /// button's name only on release. The mark stays on the button.
    #[test]
    fn the_primary_stays_marked_while_the_amount_changes() {
        let view = after(&["terror", "transfer"]);
        let mut ui = Interface::default();
        ui.transfer.counterpart = Some(Faction::Syndicate);
        ui.transfer.direction = Some(transfer::Direction::Ask);
        for (amount, choice) in [("2", "2:ask:3:2"), ("5", "2:ask:3:5")] {
            ui.transfer.entry = amount.into();
            assert_eq!(
                transfer::pressed(&view, &ui, false),
                transfer::Pressed::Primary(choice.into())
            );
            assert_eq!(
                mark(&view, &ui),
                Marked {
                    pressed: Some(Target::Primary),
                    field: false,
                }
            );
        }
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
