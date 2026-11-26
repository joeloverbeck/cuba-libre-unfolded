//! The Resource transfer modal in the selected TM-B design, "Two accounts" (#68, built
//! in #71): the executing faction's account and its counterpart's side by side, a "Give to | Ask from"
//! toggle driving a flow arrow from giver to recipient, the amount typed or set by the
//! quick values 1, 2 and Max or by Bevy's slider, the three limits, and in the giver's
//! decision the same modal in the giver's colours.
//!
//! Layout authority (foundation 4): `art/atlas/atlas-transfer.figcraft` owns the
//! structure, placement and sizes. Its reference-size masters are FigCraft auto layout
//! at the 1920×1280 reference; [`fig`] copies their padding, gaps and sizes, and the
//! runtime rebuilds the same auto layout with Bevy's flexbox, so hugging text keeps
//! the authored spacing. `atlas-transfer.layout.json` records the 16:9 frame's nodes
//! for the native comparison. Every plate is an admitted export (`transfer.sha256`).
//!
//! The game owns every fact: which transfers exist, their limits and why an amount is
//! refused ([`crate::TransferTerms`]). The counterpart, direction and typed amount are
//! presentation state, not saved; the modal submits only offered choices, through the
//! same checked seam as every other control. Opening the transfer (`Transfer::Selecting`),
//! a pending request and whether Wait set it aside are saved, so a restart shows the
//! modal, or the pending tab, as it was left (#75).
//!
//! Input beneath the modal: the scrim is a pickable node above the map and panels, so
//! no map region, piece or panel control is hit through it, and `camera_facts` blocks
//! the shared camera while it is shown. The standings and the Resources ledger stay
//! clear of the scrim.
use super::emblems;
use super::palette::{INK, PAPER, PRIMARY, QUIET};
use super::*;
use crate::{Bound, ConstructFunding, Observation, Transfer, TransferTerms};
use bevy::text::{Justify, LetterSpacing};
use bevy::ui::widget::NodeImageMode;
use bevy::ui_widgets::{
    Slider, SliderDragState, SliderPrecision, SliderRange, SliderStep, SliderThumb, SliderValue,
    TrackClick, ValueChange,
};

/// The TM-B reference-size masters in `atlas-transfer.figcraft`, in reference pixels.
pub(super) mod fig {
    /// The modal's width; its height hugs its content and it is centred.
    pub const WIDTH: f32 = 720.;
    /// The scrim leaves the top band clear, and the ledger's column to this depth.
    pub const SCRIM_TOP: f32 = 330. / 2.7;
    pub const SCRIM_NOTCH: [f32; 2] = [644. / 2.7, 600. / 2.7];
    /// Padding: top, right, bottom, left.
    pub const HEADER_PAD: [f32; 4] = [12., 20., 12., 20.];
    pub const BANNER_PAD: [f32; 4] = [8., 20., 8., 20.];
    pub const ACCOUNTS_PAD: [f32; 4] = [18., 24., 14., 24.];
    pub const ACCOUNTS_GAP: f32 = 14.;
    pub const CARD_W: f32 = 210.;
    pub const CARD_PAD: [f32; 4] = [14., 12., 14., 12.];
    pub const CARD_GAP: f32 = 4.;
    pub const EMBLEM: f32 = 52.;
    pub const BALANCE_GAP: f32 = 8.;
    pub const DIRECTION_GAP: f32 = 10.;
    pub const DIRECTION_BOTTOM: f32 = 22.;
    pub const TOGGLE_PAD: f32 = 3.;
    pub const SEGMENT_PAD: [f32; 2] = [7., 14.];
    pub const BAND: [f32; 2] = [176., 60.];
    pub const ARROW: [f32; 2] = [120., 40.];
    pub const COUNTERPART_GAP: f32 = 6.;
    pub const TABS_PAD: f32 = 8.;
    pub const TABS_GAP: f32 = 4.;
    pub const TAB: [f32; 2] = [56., 40.];
    pub const TAB_ON: [f32; 2] = [56., 44.];
    pub const TAB_EMBLEM: f32 = 28.;
    pub const TAB_EMBLEM_ON: f32 = 32.;
    pub const AMOUNT_PAD: [f32; 4] = [6., 24., 16., 24.];
    pub const AMOUNT_GAP: f32 = 8.;
    pub const ENTRY_GAP: f32 = 10.;
    pub const FIELD: [f32; 2] = [84., 44.];
    pub const FIELD_PAD: f32 = 12.;
    pub const CARET: [f32; 2] = [1.5, 24.];
    pub const QUICK_H: f32 = 36.;
    pub const QUICK_PAD: f32 = 14.;
    /// The slider row: track, thumb and the 0 and limit labels.
    pub const SLIDER: [f32; 2] = [672., 46.];
    pub const TRACK_TOP: f32 = 14.;
    pub const TRACK_H: f32 = 8.;
    pub const HATCH: [f32; 2] = [13., 10.];
    pub const THUMB: [f32; 2] = [20., 30.];
    pub const SLIDER_LABEL_TOP: f32 = 28.;
    pub const SLIDER_LABEL_W: f32 = 112.;
    pub const LIMITS_PAD: [f32; 4] = [8., 12., 8., 12.];
    pub const LIMITS_GAP: f32 = 18.;
    pub const LIMIT_GAP: f32 = 6.;
    pub const ICON: f32 = 16.;
    pub const FOOTER_PAD: [f32; 4] = [14., 24., 18., 24.];
    pub const FOOTER_GAP: f32 = 10.;
    pub const BUTTON_H: f32 = 44.;
    pub const BUTTON_PAD: f32 = 20.;
    pub const PRIMARY_PAD: f32 = 22.;
    pub const WAIT_GAP: f32 = 8.;
    /// Type sizes and letter spacing.
    pub const TITLE: f32 = 15.;
    pub const TITLE_SPACING: f32 = 2.4;
    pub const SMALL: f32 = 13.;
    pub const CAPS: f32 = 11.;
    pub const CAPS_SPACING: f32 = 1.6;
    pub const NAME: f32 = 16.;
    pub const ROLE: f32 = 12.;
    pub const NOW: f32 = 22.;
    pub const AFTER: f32 = 40.;
    pub const SEGMENT: f32 = 14.;
    pub const VALUE: f32 = 26.;
    pub const BUTTON: f32 = 15.;
    /// The pending tab in the top band, right of the standings (centred reference).
    pub const PENDING: [f32; 4] = [1200., 41., 361., 44.];
    /// Top and bottom, left, right.
    pub const PENDING_PAD: [f32; 3] = [9., 13., 9.];
    pub const PENDING_GAP: f32 = 10.;
    pub const PENDING_EMBLEM: f32 = 26.;
    pub const PENDING_BAND: [f32; 2] = [64., 24.];
    pub const PENDING_ARROW: [f32; 2] = [45., 15.];
    pub const OPEN: [f32; 2] = [57., 28.];
}

/// The executing faction's two transfer decisions (R 1.5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) enum Direction {
    /// The executor gives; the Resource transfer moves on confirmation.
    Give,
    /// The executor asks; the counterpart decides what moves.
    Ask,
}

/// A modal control that changes only the modal's presentation state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Input {
    Counterpart(Faction),
    Direction(Direction),
    /// A quick value; `None` is Max.
    Quick(Option<u8>),
    /// The amount field: it takes the keyboard's digits.
    Field,
}

/// The modal's presentation state, kept in `Interface`. Nothing here is saved.
#[derive(Default)]
pub(super) struct State {
    pub counterpart: Option<Faction>,
    pub direction: Option<Direction>,
    /// The typed amount's digits.
    pub entry: String,
    /// The revision whose transfer this state describes.
    opened: Option<u64>,
    /// The transfer it describes, as if never set aside by Wait.
    request: Option<Transfer>,
    /// The modal control with keyboard focus, by its seat name.
    pub focus: Option<String>,
}
impl State {
    /// The typed amount, 0 when empty.
    pub fn amount(&self) -> u8 {
        self.entry.parse().unwrap_or(0)
    }
    fn set(&mut self, amount: u8) {
        self.entry = amount.to_string();
    }
    /// Follows the match: a newly opened transfer starts empty, and a new request
    /// prefills the giver's amount with the amount asked. Wait and Open change only
    /// whether the same request's decision is shown, so the amount typed stays.
    fn sync(&mut self, revision: u64, transfer: Option<&Transfer>) {
        let Some(transfer) = transfer else {
            *self = Self::default();
            return;
        };
        if self.opened == Some(revision) {
            return;
        }
        let request = transfer.without_wait();
        if request != Transfer::Selecting && self.request.as_ref() == Some(&request) {
            self.opened = Some(revision);
            self.focus = None;
            return;
        }
        *self = Self {
            opened: Some(revision),
            request: Some(request),
            ..Self::default()
        };
        if let Transfer::Authorizing { amount, .. } = transfer {
            self.set(*amount);
        }
    }
}

/// Whether the modal is shown: a transfer is open and not set aside by Wait, and the
/// faction that `decides` it is the human's (#206). Whether it was set aside is saved
/// (#75), so a restart shows the request as it was left.
pub(super) fn shown(transfer: Option<&Transfer>, decides: Faction, ui: &Interface) -> bool {
    ui.plays(decides)
        && match transfer {
            Some(Transfer::Selecting) => true,
            Some(Transfer::Authorizing { waited, .. }) => !waited,
            None => false,
        }
}

/// The giver's colour key, or "ink" while the executing faction composes.
fn tone(view: &Observation) -> &'static str {
    match &view.transfer {
        Some(Transfer::Authorizing { giver, .. }) => slug(*giver),
        _ => "ink",
    }
}

/// What the modal shows now, worked out from the observation and the modal's state.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Shown {
    pub giver: Option<Faction>,
    pub recipient: Option<Faction>,
    pub terms: Option<TransferTerms>,
    pub amount: u8,
    pub refusal: Option<String>,
    /// The choice the primary button submits, when it can.
    pub commit: Option<crate::Choice>,
    /// Why the primary button cannot submit.
    pub blocked: Option<String>,
    pub label: String,
}

impl Shown {
    fn balances(&self, terms: &TransferTerms, funding: Option<&ConstructFunding>) -> (i16, i16) {
        if let Some(funding) = funding
            && terms.recipient == Faction::Syndicate
        {
            if self.amount < funding.shortfall {
                return balances(terms, 0);
            }
            let previously_authorized = funding
                .authorizations
                .iter()
                .filter(|a| a.giver == terms.giver)
                .map(|a| i16::from(a.amount))
                .sum::<i16>();
            let (giver, recipient) = balances(terms, self.amount);
            return (
                giver - previously_authorized,
                recipient + i16::from(funding.authorized()),
            );
        }
        balances(terms, self.amount)
    }
}

pub(super) fn content(view: &Observation, state: &State) -> Shown {
    let executor = view.executing;
    let amount = state.amount();
    let offered = |action: String| {
        view.choices
            .iter()
            .find(|c| c.id == format!("{}:{action}", view.revision))
            .cloned()
    };
    match &view.transfer {
        Some(Transfer::Authorizing {
            giver, recipient, ..
        }) => {
            let terms = view.transfer_terms.first().cloned();
            let offered = offered(format!("authorize:{amount}"));
            let refusal = terms.as_ref().and_then(|t| t.refusal(amount)).or_else(|| {
                offered
                    .as_ref()
                    .filter(|choice| !choice.supported)
                    .map(|choice| choice.explanation.clone())
            });
            Shown {
                giver: Some(*giver),
                recipient: Some(*recipient),
                commit: offered.filter(|choice| choice.supported),
                blocked: refusal.clone(),
                label: format!("{} authorizes {amount}", giver.name()),
                terms,
                amount,
                refusal,
            }
        }
        _ => {
            let (giver, recipient) = match (state.counterpart, state.direction) {
                (Some(other), Some(Direction::Give)) => (executor, Some(other)),
                (Some(other), Some(Direction::Ask)) => (Some(other), executor),
                _ => (None, None),
            };
            let terms = view
                .transfer_terms
                .iter()
                .find(|t| Some(t.giver) == giver && Some(t.recipient) == recipient)
                .cloned();
            let refusal = terms.as_ref().and_then(|t| t.refusal(amount));
            let (label, action) = match (state.counterpart, state.direction) {
                (None, _) => ("Choose a counterpart".to_owned(), None),
                (Some(_), None) => ("Choose Give to or Ask from".to_owned(), None),
                (Some(other), Some(Direction::Give)) => (
                    format!("Give {amount} to {}", other.name()),
                    Some(format!("give:{}:{amount}", other.index())),
                ),
                (Some(other), Some(Direction::Ask)) => (
                    format!("Send request: {amount} from {}", other.name()),
                    Some(format!("ask:{}:{amount}", other.index())),
                ),
            };
            let commit = refusal
                .is_none()
                .then(|| action.and_then(offered))
                .flatten();
            let blocked = if commit.is_some() {
                None
            } else if let Some(refusal) = &refusal {
                Some(refusal.clone())
            } else if state.counterpart.is_none() {
                Some("Choose the other faction by its emblem first.".into())
            } else {
                Some("Choose Give to or Ask from first.".into())
            };
            Shown {
                giver,
                recipient,
                terms,
                amount,
                refusal,
                commit,
                blocked,
                label,
            }
        }
    }
}

/// The limits strip: the three bounds, the binding one in bold.
fn limits(terms: &TransferTerms, executor: Option<Faction>) -> [(&'static str, String, bool); 3] {
    let (g, r) = (terms.giver.name(), terms.recipient.name());
    let binding = |bound| terms.bound == bound;
    [
        (
            "balance",
            if binding(Bound::Balance) {
                format!("Limit {}: {g}'s balance", terms.limit)
            } else {
                format!("{g}'s balance: {}", terms.balance)
            },
            binding(Bound::Balance),
        ),
        (
            "cap",
            if binding(Bound::Cap) {
                format!("Limit {}: {r} holds at most 49", terms.limit)
            } else if terms.recipient == Faction::M26 {
                format!("{r} holds at most 49")
            } else {
                format!("{r} has room for {}", terms.room)
            },
            binding(Bound::Cap),
        ),
        (
            "reserve",
            if binding(Bound::ConstructPayment) {
                format!(
                    "Limit {}: pay for Limited Construct first to give more",
                    terms.limit
                )
            } else if binding(Bound::ConstructAuthorization) {
                format!(
                    "Limit {}: {} already authorized",
                    terms.limit, terms.reserved
                )
            } else if binding(Bound::Reserve) {
                format!(
                    "Limit {}: {} reserved for owed Terror",
                    terms.limit, terms.reserved
                )
            } else if terms.recipient == Faction::Syndicate && terms.reserved > 0 {
                format!("{} already authorized", terms.reserved)
            } else if executor == Some(Faction::Syndicate) {
                if terms.reserved > 0 {
                    format!("{} kept for Limited Construct", resources(terms.reserved))
                } else {
                    "No Resources held for Construct".into()
                }
            } else if terms.giver == Faction::M26 || terms.reserved == 0 {
                "No Terror owed: nothing reserved".into()
            } else {
                "Nothing reserved from this giver".into()
            },
            binding(Bound::Reserve)
                || binding(Bound::ConstructPayment)
                || binding(Bound::ConstructAuthorization),
        ),
    ]
}

/// Applies a modal control to the modal's state. Nothing here changes the match.
pub(super) fn input(ui: &mut Interface, view: &Observation, input: Input) {
    let state = &mut ui.transfer;
    match input {
        Input::Counterpart(faction) => {
            state.counterpart = Some(faction);
        }
        Input::Direction(direction) => state.direction = Some(direction),
        Input::Quick(Some(n)) => state.set(n),
        Input::Quick(None) => {
            if let Some(terms) = content(view, state).terms {
                state.set(terms.limit);
            }
        }
        Input::Field => state.focus = Some(FIELD.into()),
    }
    ui.dirty = true;
}

/// A seat-reachable name for a modal control.
fn tab_name(faction: Faction) -> String {
    format!("transfer:tab:{}", slug(faction))
}

fn counterparts(view: &Observation) -> impl Iterator<Item = Faction> + '_ {
    Faction::ALL
        .into_iter()
        .filter(|&faction| Some(faction) != view.executing)
}

/// The modal's and the pending tab's layer, over the scrim's.
pub(super) const LAYER: i32 = 31;
/// The primary button's part name.
pub(super) const PRIMARY_PART: &str = "button:primary";
/// The amount field's seat name.
pub(super) const FIELD: &str = "transfer:field";

/// The modal's controls in Tab order, by their seat names.
pub(super) fn focusable(view: &Observation, ui: &Interface) -> Vec<String> {
    let shown = content(view, &ui.transfer);
    let mut names = vec![];
    if matches!(view.transfer, Some(Transfer::Selecting)) {
        names.extend(counterparts(view).map(tab_name));
        names.extend([
            "transfer:direction:give".into(),
            "transfer:direction:ask".into(),
        ]);
    }
    names.push(FIELD.into());
    names.extend(["transfer:quick:1", "transfer:quick:2", "transfer:quick:max"].map(String::from));
    match &view.transfer {
        Some(Transfer::Selecting) => {
            names.push(format!("{}:back", view.revision));
        }
        _ => {
            names.push(format!("{}:wait", view.revision));
            names.push(format!("{}:decline", view.revision));
        }
    }
    names.push(
        shown
            .commit
            .map_or_else(|| "transfer:primary".to_owned(), |c| c.id),
    );
    names
}

/// What Enter presses in the modal: the focused control, or else the primary button,
/// which Enter on the amount field presses too. The focus mark is on it (#179).
#[derive(Debug, PartialEq)]
pub(super) enum Pressed {
    /// The primary button, by its seat name, which follows the amount. While the slider
    /// is dragged the drawn button keeps the name it was drawn with.
    Primary(String),
    /// Another control, by its seat name.
    Other(String),
}
impl Pressed {
    fn name(self) -> String {
        match self {
            Self::Primary(name) | Self::Other(name) => name,
        }
    }
}

/// The modal's stop that Enter presses now.
pub(super) fn pressed(view: &Observation, ui: &Interface) -> Pressed {
    let mut order = focusable(view, ui);
    // The primary is the last stop.
    let primary = order.pop().unwrap();
    ui.transfer
        .focus
        .as_ref()
        .filter(|focus| *focus != FIELD && order.contains(focus))
        .map_or(Pressed::Primary(primary), |focus| {
            Pressed::Other(focus.clone())
        })
}

/// Activates the modal control `name` as a click would.
fn activate(play: &mut Play, ui: &mut Interface, view: &Observation, name: &str) {
    let rest = name.strip_prefix("transfer:");
    let input = match rest {
        Some("direction:give") => Some(Input::Direction(Direction::Give)),
        Some("direction:ask") => Some(Input::Direction(Direction::Ask)),
        Some("quick:1") => Some(Input::Quick(Some(1))),
        Some("quick:2") => Some(Input::Quick(Some(2))),
        Some("quick:max") => Some(Input::Quick(None)),
        Some("field") => Some(Input::Field),
        Some(tab) if tab.starts_with("tab:") => Faction::ALL
            .into_iter()
            .find(|f| tab_name(*f) == name)
            .map(Input::Counterpart),
        _ => None,
    };
    if let Some(input) = input {
        return self::input(ui, view, input);
    }
    if name == "transfer:primary" {
        super::notice::refuse(ui, content(view, &ui.transfer).blocked.unwrap_or_default());
        return;
    }
    if let Some(choice) = view.choices.iter().find(|c| c.id == name) {
        decide(play, ui, choice.owner, view.revision, name);
        ui.dirty = true;
    }
}

/// Keyboard reach while the modal is shown: digits and Backspace edit the amount, Tab
/// and Shift+Tab move focus through the modal's controls, Enter activates the focused
/// control or else the primary button, and Escape cancels the open transfer or, in
/// the giver's decision, waits.
pub(super) fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
) {
    if play.session.is_none() || keys.get_just_pressed().next().is_none() {
        return;
    }
    let view = play.game().observe();
    ui.transfer.sync(view.revision, view.transfer.as_ref());
    if !shown(view.transfer.as_ref(), view.owner, &ui) {
        return;
    }
    const DIGITS: [(KeyCode, KeyCode, char); 10] = [
        (KeyCode::Digit0, KeyCode::Numpad0, '0'),
        (KeyCode::Digit1, KeyCode::Numpad1, '1'),
        (KeyCode::Digit2, KeyCode::Numpad2, '2'),
        (KeyCode::Digit3, KeyCode::Numpad3, '3'),
        (KeyCode::Digit4, KeyCode::Numpad4, '4'),
        (KeyCode::Digit5, KeyCode::Numpad5, '5'),
        (KeyCode::Digit6, KeyCode::Numpad6, '6'),
        (KeyCode::Digit7, KeyCode::Numpad7, '7'),
        (KeyCode::Digit8, KeyCode::Numpad8, '8'),
        (KeyCode::Digit9, KeyCode::Numpad9, '9'),
    ];
    for (code, pad, digit) in DIGITS {
        if keys.any_just_pressed([code, pad]) {
            let entry = &mut ui.transfer.entry;
            // Two digits cover every amount up to the 49 cap; a leading zero goes.
            if entry == "0" {
                entry.clear();
            }
            if entry.len() < 2 {
                entry.push(digit);
            }
            ui.dirty = true;
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        ui.transfer.entry.pop();
        ui.dirty = true;
    }
    if keys.just_pressed(KeyCode::Escape) {
        let name = match view.transfer {
            Some(Transfer::Selecting) => format!("{}:back", view.revision),
            _ => format!("{}:wait", view.revision),
        };
        activate(&mut play, &mut ui, &view, &name);
        return;
    }
    let order = focusable(&view, &ui);
    let at = ui
        .transfer
        .focus
        .as_ref()
        .and_then(|f| order.iter().position(|n| n == f));
    if keys.just_pressed(KeyCode::Tab) {
        let back = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        let n = order.len();
        let next = match (at, back) {
            (None, false) => 0,
            (None, true) => n - 1,
            (Some(i), false) => (i + 1) % n,
            (Some(i), true) => (i + n - 1) % n,
        };
        ui.transfer.focus = Some(order[next].clone());
        ui.dirty = true;
        return;
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        let name = pressed(&view, &ui).name();
        activate(&mut play, &mut ui, &view, &name);
    }
}

/// The slider's changes set the amount, within the giver's limit; the hatched
/// reserved unit cannot be dragged into.
fn slide(change: On<ValueChange<f32>>, mut ui: ResMut<Interface>, play: Res<Play>) {
    let view = play.game().observe();
    let limit = content(&view, &ui.transfer).terms.map_or(0, |t| t.limit);
    let amount = (change.value.round().max(0.) as u8).min(limit);
    if ui.transfer.amount() != amount || ui.transfer.entry.is_empty() {
        ui.transfer.set(amount);
        ui.dirty = true;
    }
}

/// The modal's and the pending tab's entities, redrawn by [`draw`].
#[derive(Component)]
pub(super) struct Modal;

/// Names a drawn part by its node name in `atlas-transfer.figcraft`, so the seat can
/// compare native placement with the layout record.
#[derive(Component)]
pub(super) struct Part(pub String);

/// Parts that follow the amount while the slider is dragged, without a redraw.
#[derive(Component, Clone, Copy)]
pub(super) enum Live {
    Thumb,
    Fill,
    Value,
    GiverAfter,
    RecipientAfter,
}

type Drawn = (u64, Option<Transfer>, String, [u32; 2]);

/// Draws the modal, or the pending tab after Wait, when what they show changes. A
/// slider press or drag in progress only moves its live parts; the modal redraws on
/// release.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    mut ui: ResMut<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Modal>>,
    held: Query<(&SliderDragState, Has<bevy::ui::Pressed>), With<Slider>>,
    mut live: Query<(&Live, &mut Node, Option<&mut Text>)>,
    mut previous: Local<Option<Drawn>>,
) {
    let Some(game) = play.session.as_ref().map(Session::game) else {
        return;
    };
    ui.transfer.sync(game.revision, game.transfer.as_ref());
    let s = reference_pixel(&window);
    // A press or drag on the slider keeps its entity until release.
    if held.iter().any(|(d, pressed)| d.dragging || pressed) {
        let view = game.observe();
        let shown = content(&view, &ui.transfer);
        if let Some(terms) = &shown.terms {
            follow(&mut live, terms, &shown, view.construct_funding.as_ref(), s);
        }
        return;
    }
    let key = (
        game.revision,
        game.transfer.clone(),
        // A seat handed over or taken back shows or hides the modal and Open.
        format!(
            "{:?}{:?}{}{:?}{}",
            ui.transfer.counterpart,
            ui.transfer.direction,
            ui.transfer.entry,
            ui.transfer.focus,
            ui.plays(game.owner()),
        ),
        [window.width().to_bits(), window.height().to_bits()],
    );
    if previous.as_ref() == Some(&key) {
        return;
    }
    *previous = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    if game.transfer.is_none() {
        return;
    }
    let view = game.observe();
    let extra = window.width() - WIDTH * s;
    let centered = Vec2::new(extra / 2., 0.);
    if !shown(view.transfer.as_ref(), view.owner, &ui) {
        pending(&mut commands, &art, &view, &ui, s, centered);
        return;
    }
    scrim(&mut commands, &art, &window, s, Vec2::ZERO);
    // The modal sits in the middle of the window; its note above stays at the top.
    let middle = centered + bottom(&window) / 2.;
    modal(&mut commands, &art, &view, &ui, s, middle);
}

/// Moves the thumb and fill and rewrites the amount and balances during a drag.
fn follow(
    live: &mut Query<(&Live, &mut Node, Option<&mut Text>)>,
    terms: &TransferTerms,
    shown: &Shown,
    funding: Option<&ConstructFunding>,
    s: f32,
) {
    let amount = shown.amount;
    let (giver, recipient) = shown.balances(terms, funding);
    let x = position(terms, amount) * s;
    for (part, mut node, text) in live.iter_mut() {
        match part {
            Live::Thumb => node.left = px(x - fig::THUMB[0] * s / 2.),
            Live::Fill => node.width = px(x),
            Live::Value => {
                if let Some(mut text) = text {
                    text.0 = amount.to_string();
                }
            }
            Live::GiverAfter => {
                if let Some(mut text) = text {
                    text.0 = giver.to_string();
                }
            }
            Live::RecipientAfter => {
                if let Some(mut text) = text {
                    text.0 = recipient.to_string();
                }
            }
        }
    }
}

/// The giver's and the recipient's balances after `amount` moves: the typed amount's
/// effect, which a refused amount takes below 0 or past the cap, as the design shows
/// it in red.
fn balances(terms: &TransferTerms, amount: u8) -> (i16, i16) {
    (
        i16::from(terms.balance) - i16::from(amount),
        i16::from(terms.held) + i16::from(amount),
    )
}

/// The amount's x along the track, in reference pixels: the track spans the giver's
/// balance.
fn position(terms: &TransferTerms, amount: u8) -> f32 {
    if terms.balance == 0 {
        0.
    } else {
        fig::SLIDER[0] * f32::from(amount.min(terms.balance)) / f32::from(terms.balance)
    }
}

/// An admitted export (`transfer.sha256`).
fn image(art: &Art, key: &str) -> Handle<Image> {
    art.images[key].clone()
}

/// A FigCraft plate behind a node that may hug its content. Bevy draws a node's own
/// image inside its padding, so [`plates`] draws the plate as the node's first child,
/// filling its whole box.
#[derive(Component)]
pub(super) struct Plate(ImageNode);

/// A plate, nine-sliced so its corners and strokes keep the authored size while its
/// sides and middle stretch. `border` is the corner in reference pixels; the exports
/// are at 3×.
fn plate(art: &Art, key: &str) -> Plate {
    let border = if key.contains("-toggle") {
        19.
    } else if key.contains("-segment-on") {
        15.
    } else if key.contains("-pending-band-") {
        12.
    } else if key.contains("-band-") {
        30.
    } else {
        6.
    };
    Plate(
        ImageNode::new(image(art, key)).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(border * 3.),
            ..default()
        })),
    )
}

/// Draws each new plate as the first child of its node, over the node's whole box and
/// beneath its content.
pub(super) fn plates(mut commands: Commands, added: Query<(Entity, &Plate), Added<Plate>>) {
    for (entity, Plate(image)) in &added {
        let child = commands
            .spawn((
                image.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0.),
                    top: px(0.),
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(entity).insert_children(0, &[child]);
    }
}

fn pad(p: [f32; 4], s: f32) -> UiRect {
    UiRect {
        top: px(p[0] * s),
        right: px(p[1] * s),
        bottom: px(p[2] * s),
        left: px(p[3] * s),
    }
}

/// A child node of `parent`.
fn child(commands: &mut Commands, parent: Entity, node: Node) -> Entity {
    commands
        .spawn((node, Pickable::IGNORE, ChildOf(parent)))
        .id()
}

/// A child text of `parent`, in the modal. Running text and buttons name factions by #60's inline
/// emblems (story 60, "Ask [D] for Resources"); headings and labels pass
/// [`Prose::named`] and keep the names, and on a red surface ([`Prose::on_red`])
/// 26July keeps its name. The seat reads every name (`emblems::Plain`).
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    art: &Art,
    parent: Entity,
    text: impl Into<Prose>,
    size: f32,
    bold: bool,
    color: Color,
    spacing: f32,
    s: f32,
) -> Entity {
    let prose: Prose = text.into();
    let font = TextFont {
        font: if bold {
            art.bold.clone()
        } else {
            art.font.clone()
        }
        .into(),
        font_size: (size * s).into(),
        ..default()
    };
    let segments = emblems::segments(&prose.text, prose.naming, prose.red);
    let (first, rest) = match segments.split_first() {
        Some((emblems::Segment::Words(words), rest)) => (words.clone(), rest),
        _ => (String::new(), &segments[..]),
    };
    let mut entity = commands.spawn((
        Text::new(first),
        emblems::Plain(prose.text),
        font.clone(),
        TextColor(color),
        Pickable::IGNORE,
        ChildOf(parent),
    ));
    if spacing > 0. {
        entity.insert(LetterSpacing::Px(spacing * s));
    }
    let root = entity.id();
    emblems::spawn_rest(
        commands,
        art,
        root,
        rest,
        &font,
        TextColor(color),
        size * s * emblems::HEIGHT,
    );
    root
}

fn part(commands: &mut Commands, entity: Entity, name: &str) {
    commands.entity(entity).insert(Part(name.into()));
}

/// The scrim over the map, the panels, Save · quit and the notice slip. The top band stays clear, and so
/// does the Resources ledger's column to its foot.
fn scrim(commands: &mut Commands, art: &Art, window: &Window, s: f32, left: Vec2) {
    let top = left.y + fig::SCRIM_TOP * s;
    let notch_x = window.width() - fig::SCRIM_NOTCH[0] * s;
    let notch_y = left.y + fig::SCRIM_NOTCH[1] * s;
    for (i, [x, y, w, h]) in [
        [0., top, notch_x, window.height() - top],
        [
            notch_x,
            notch_y,
            window.width() - notch_x,
            window.height() - notch_y,
        ],
    ]
    .into_iter()
    .enumerate()
    {
        let entity = commands
            .spawn((
                Modal,
                ImageNode::new(image(art, "atlas-transfer-scrim.png"))
                    .with_mode(NodeImageMode::Stretch),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(x),
                    top: px(y),
                    width: px(w),
                    height: px(h),
                    ..default()
                },
                GlobalZIndex(30),
            ))
            .id();
        if i == 0 {
            part(commands, entity, "scrim");
        }
    }
}

/// A control node: a plate that hugs its content, with the seat's name and action.
#[allow(clippy::too_many_arguments)]
fn button(
    commands: &mut Commands,
    art: &Art,
    parent: Entity,
    key: &str,
    node: Node,
    name: &str,
    choice: bool,
    action: Action,
) -> Entity {
    let mut entity = commands.spawn((
        node,
        Control {
            name: name.into(),
            choice,
            action,
        },
        ChildOf(parent),
    ));
    // An unselected toggle segment has no plate of its own.
    if !key.is_empty() {
        entity.insert(plate(art, key));
    }
    entity.observe(super::click).id()
}

/// An account card: emblem, name, role and "now → after" balance.
#[allow(clippy::too_many_arguments)]
fn account(
    commands: &mut Commands,
    art: &Art,
    parent: Entity,
    faction: Faction,
    role: &str,
    now: u8,
    after: Option<(i16, Live, bool)>,
    key: &str,
    s: f32,
) -> Entity {
    let card = commands
        .spawn((
            plate(art, key),
            Node {
                width: px(fig::CARD_W * s),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(fig::CARD_GAP * s),
                padding: pad(fig::CARD_PAD, s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(parent),
        ))
        .id();
    commands.spawn((
        ImageNode::new(image(art, &emblems::image(faction))),
        Node {
            width: px(fig::EMBLEM * s),
            height: px(fig::EMBLEM * s),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(card),
    ));
    words(
        commands,
        art,
        card,
        Prose::named(faction.name()),
        fig::NAME,
        true,
        INK,
        0.,
        s,
    );
    words(commands, art, card, role, fig::ROLE, false, QUIET, 0., s);
    let balance = child(
        commands,
        card,
        Node {
            column_gap: px(fig::BALANCE_GAP * s),
            align_items: AlignItems::Baseline,
            ..default()
        },
    );
    match after {
        Some((after, live, refused)) => {
            words(
                commands,
                art,
                balance,
                now.to_string(),
                fig::NOW,
                false,
                QUIET,
                0.,
                s,
            );
            words(commands, art, balance, "→", fig::NOW, false, QUIET, 0., s);
            let value = words(
                commands,
                art,
                balance,
                after.to_string(),
                fig::AFTER,
                true,
                if refused { PRIMARY } else { INK },
                0.,
                s,
            );
            commands.entity(value).insert(live);
            words(
                commands,
                art,
                card,
                "Resources: now → after",
                fig::ROLE,
                false,
                QUIET,
                0.,
                s,
            );
        }
        None => {
            words(
                commands,
                art,
                balance,
                now.to_string(),
                fig::AFTER,
                true,
                INK,
                0.,
                s,
            );
            words(
                commands,
                art,
                card,
                "Resources now",
                fig::ROLE,
                false,
                QUIET,
                0.,
                s,
            );
        }
    }
    card
}

/// The modal itself: header, accounts, amount and footer, centred in the window.
fn modal(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    s: f32,
    centered: Vec2,
) {
    let state = &ui.transfer;
    let shown = content(view, state);
    let tone = tone(view);
    let decision = matches!(view.transfer, Some(Transfer::Authorizing { .. }));
    let refused = shown.refusal.is_some();
    // A full reference-area box centres the modal; it takes no input itself.
    let frame = commands
        .spawn((
            Modal,
            Node {
                position_type: PositionType::Absolute,
                left: px(centered.x),
                top: px(centered.y),
                width: px(WIDTH * s),
                height: px(HEIGHT * s),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(LAYER),
            Pickable::IGNORE,
        ))
        .id();
    let root = commands
        .spawn((
            plate(art, "atlas-transfer-modal.png"),
            Node {
                width: px(fig::WIDTH * s),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(s)),
                ..default()
            },
            ChildOf(frame),
        ))
        .id();
    part(commands, root, "transfer-modal");
    // Header band, in the deciding faction's colour.
    let executor = view.executing.unwrap_or(view.owner);
    let activity = match (view.current_card, &view.event) {
        (18, _) if executor == Faction::Syndicate => "Construct",
        (18, _) => "Assault",
        (_, Some(crate::EventProgress::Rally(_))) => "the free Rally",
        (_, Some(_)) => "Alberto Bayo",
        (_, None) => "Terror",
    };
    let header = commands
        .spawn((
            plate(art, &format!("atlas-transfer-header-{tone}.png")),
            Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: pad(fig::HEADER_PAD, s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(root),
        ))
        .id();
    part(commands, header, "header");
    let (title, note) = match view.transfer {
        Some(Transfer::Authorizing {
            giver, recipient, ..
        }) => (
            format!("{} DECIDES", giver.name().to_uppercase()),
            format!("Transfer request from {}  ·  R 1.5.1", recipient.name()),
        ),
        _ => (
            "RESOURCE TRANSFER".into(),
            format!(
                "Open while {} executes {}  ·  R 1.5.1",
                executor.name(),
                activity
            ),
        ),
    };
    let t = words(
        commands,
        art,
        header,
        Prose::named(title),
        fig::TITLE,
        true,
        PAPER,
        fig::TITLE_SPACING,
        s,
    );
    part(commands, t, "title");
    let note = Prose::from(note);
    // On 26July's red header band, 26July keeps its name (#60).
    let note = if tone == slug(Faction::M26) {
        note.on_red()
    } else {
        note
    };
    words(commands, art, header, note, fig::SMALL, false, PAPER, 0., s);
    // The reserved state: a request another faction started, which 26July answers.
    if let Some(Transfer::Authorizing {
        giver: Faction::M26,
        ..
    }) = view.transfer
        && view.event.is_none()
    {
        let banner = commands
            .spawn((
                plate(art, "atlas-transfer-banner.png"),
                Node {
                    column_gap: px(8. * s),
                    align_items: AlignItems::Center,
                    padding: pad(fig::BANNER_PAD, s),
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(root),
            ))
            .id();
        part(commands, banner, "reserved-banner");
        words(
            commands,
            art,
            banner,
            "RESERVED STATE",
            fig::CAPS,
            true,
            INK,
            fig::CAPS_SPACING,
            s,
        );
        words(
            commands,
            art,
            banner,
            "for a request another faction's seat starts. Nothing in this task creates one.",
            fig::ROLE,
            false,
            QUIET,
            0.,
            s,
        );
    }
    // Accounts: the executing faction, the direction and the counterpart.
    let accounts = child(
        commands,
        root,
        Node {
            column_gap: px(fig::ACCOUNTS_GAP * s),
            align_items: AlignItems::FlexEnd,
            padding: pad(fig::ACCOUNTS_PAD, s),
            ..default()
        },
    );
    let terms = shown.terms.as_ref();
    let after = |faction: Faction| -> Option<(i16, Live, bool)> {
        let terms = terms?;
        let (giver, recipient) = shown.balances(terms, view.construct_funding.as_ref());
        if faction == terms.giver {
            Some((giver, Live::GiverAfter, refused))
        } else {
            Some((recipient, Live::RecipientAfter, refused))
        }
    };
    let role = match (shown.giver, shown.recipient) {
        (Some(giver), _) if giver == executor => " · giver",
        (_, Some(recipient)) if recipient == executor => " · recipient",
        _ => "",
    };
    let executor_role = format!("executing {activity}{role}");
    let executor_key = if shown.giver == Some(executor) {
        format!("atlas-transfer-account-{tone}.png")
    } else {
        "atlas-transfer-account.png".into()
    };
    let card = account(
        commands,
        art,
        accounts,
        executor,
        &executor_role,
        view.resources[executor.index()],
        after(executor),
        &executor_key,
        s,
    );
    part(commands, card, &format!("account:{}", slug(executor)));
    // Direction: the toggle while composing, the request's words in a decision.
    let direction = child(
        commands,
        accounts,
        Node {
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(fig::DIRECTION_GAP * s),
            padding: UiRect::bottom(px(fig::DIRECTION_BOTTOM * s)),
            ..default()
        },
    );
    let label = match view.transfer {
        Some(Transfer::Authorizing {
            recipient, amount, ..
        }) => format!("{} ASKS FOR {amount}", recipient.name().to_uppercase()),
        _ => "DIRECTION".into(),
    };
    words(
        commands,
        art,
        direction,
        Prose::named(label),
        fig::CAPS,
        true,
        QUIET,
        fig::CAPS_SPACING,
        s,
    );
    if !decision {
        let toggle = commands
            .spawn((
                plate(art, "atlas-transfer-toggle.png"),
                Node {
                    padding: UiRect::all(px(fig::TOGGLE_PAD * s)),
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(direction),
            ))
            .id();
        part(commands, toggle, "direction-toggle");
        for (d, text) in [(Direction::Give, "Give to"), (Direction::Ask, "Ask from")] {
            let on = state.direction == Some(d);
            let name = format!(
                "transfer:direction:{}",
                if d == Direction::Give { "give" } else { "ask" }
            );
            let segment = button(
                commands,
                art,
                toggle,
                if on {
                    "atlas-transfer-segment-on.png"
                } else {
                    ""
                },
                Node {
                    padding: UiRect::axes(px(fig::SEGMENT_PAD[1] * s), px(fig::SEGMENT_PAD[0] * s)),
                    ..default()
                },
                &name,
                false,
                Action::Transfer(Input::Direction(d)),
            );
            words(
                commands,
                art,
                segment,
                text,
                fig::SEGMENT,
                on,
                if on { PAPER } else { QUIET },
                0.,
                s,
            );
        }
    }
    let flowing = shown.giver.is_some();
    let band = commands
        .spawn((
            plate(
                art,
                &if flowing {
                    format!("atlas-transfer-band-{tone}.png")
                } else {
                    "atlas-transfer-band-empty.png".into()
                },
            ),
            Node {
                width: px(fig::BAND[0] * s),
                height: px(fig::BAND[1] * s),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(direction),
        ))
        .id();
    part(commands, band, "flow-band");
    if flowing {
        // The arrow points from the giver's account to the recipient's.
        let mut arrow = ImageNode::new(image(art, "atlas-transfer-arrow.png"));
        arrow.flip_x = shown.giver != Some(executor);
        commands.spawn((
            arrow,
            Node {
                width: px(fig::ARROW[0] * s),
                height: px(fig::ARROW[1] * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(band),
            Part("flow-arrow".into()),
        ));
    }
    let caption = match (&view.transfer, shown.giver) {
        (Some(Transfer::Authorizing { giver, .. }), _) if executor == Faction::Syndicate => {
            format!("{} chooses how much\nto authorize", giver.name())
        }
        (Some(Transfer::Authorizing { giver, .. }), _) => format!(
            "{} sets what moves:\nanything from 0 to {}",
            giver.name(),
            terms.map_or(0, |t| t.limit)
        ),
        (_, Some(giver)) if giver == executor => {
            format!("{} would give;\nit moves on confirmation", executor.name())
        }
        (_, Some(giver)) => format!(
            "{} would give;\n{} decides the amount",
            giver.name(),
            giver.name()
        ),
        _ => "The arrow shows who gives\nonce both are chosen".into(),
    };
    let c = words(
        commands,
        art,
        direction,
        caption,
        fig::ROLE,
        false,
        QUIET,
        0.,
        s,
    );
    commands
        .entity(c)
        .insert(TextLayout::justify(Justify::Center));
    // The counterpart: emblem tabs over its card while composing.
    let counterpart = child(
        commands,
        accounts,
        Node {
            width: px(fig::CARD_W * s),
            flex_direction: FlexDirection::Column,
            row_gap: px(fig::COUNTERPART_GAP * s),
            ..default()
        },
    );
    let other = match view.transfer {
        Some(Transfer::Authorizing {
            giver, recipient, ..
        }) => Some(if giver == executor { recipient } else { giver }),
        _ => state.counterpart,
    };
    let heading = match view.transfer {
        Some(Transfer::Authorizing { giver, .. }) if giver == executor => "REQUESTER",
        Some(Transfer::Authorizing { .. }) => "GIVER",
        _ => "COUNTERPART",
    };
    words(
        commands,
        art,
        counterpart,
        Prose::named(heading),
        fig::CAPS,
        true,
        QUIET,
        fig::CAPS_SPACING,
        s,
    );
    if !decision {
        let tabs = child(
            commands,
            counterpart,
            Node {
                column_gap: px(fig::TABS_GAP * s),
                align_items: AlignItems::FlexEnd,
                padding: UiRect::horizontal(px(fig::TABS_PAD * s)),
                ..default()
            },
        );
        for faction in counterparts(view) {
            let on = state.counterpart == Some(faction);
            let ([w, h], e) = if on {
                (fig::TAB_ON, fig::TAB_EMBLEM_ON)
            } else {
                (fig::TAB, fig::TAB_EMBLEM)
            };
            let name = tab_name(faction);
            let tab = button(
                commands,
                art,
                tabs,
                if on {
                    "atlas-transfer-tab-on.png"
                } else {
                    "atlas-transfer-tab.png"
                },
                Node {
                    width: px(w * s),
                    height: px(h * s),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                &name,
                false,
                Action::Transfer(Input::Counterpart(faction)),
            );
            part(commands, tab, &name);
            commands.spawn((
                ImageNode::new(image(art, &emblems::image(faction))),
                Node {
                    width: px(e * s),
                    height: px(e * s),
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(tab),
            ));
        }
    }
    match other {
        Some(faction) => {
            let role = match view.transfer {
                Some(Transfer::Authorizing { giver, .. }) if giver == executor => {
                    "requester · recipient"
                }
                _ if shown.giver == Some(faction) => "giver",
                _ if shown.recipient == Some(faction) => "recipient",
                _ => "choose Give to or Ask from",
            };
            let key = if shown.giver == Some(faction) {
                format!("atlas-transfer-account-{tone}.png")
            } else {
                "atlas-transfer-account.png".into()
            };
            let card = account(
                commands,
                art,
                counterpart,
                faction,
                role,
                view.resources[faction.index()],
                after(faction),
                &key,
                s,
            );
            commands.entity(card).insert(Node {
                width: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(fig::CARD_GAP * s),
                padding: pad(fig::CARD_PAD, s),
                ..default()
            });
            part(commands, card, "account:counterpart");
        }
        None => {
            let empty = commands
                .spawn((
                    plate(art, "atlas-transfer-account-empty.png"),
                    Node {
                        height: px(190. * s),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: px(fig::CARD_GAP * s),
                        ..default()
                    },
                    Pickable::IGNORE,
                    ChildOf(counterpart),
                ))
                .id();
            part(commands, empty, "account:counterpart");
            words(
                commands,
                art,
                empty,
                "No counterpart yet",
                fig::NAME,
                true,
                QUIET,
                0.,
                s,
            );
            words(
                commands,
                art,
                empty,
                "pick an emblem tab",
                fig::ROLE,
                false,
                QUIET,
                0.,
                s,
            );
        }
    }
    amount(commands, art, view, state, &shown, tone, root, s);
    commands.spawn((
        ImageNode::new(image(art, "atlas-transfer-rule.png")).with_mode(NodeImageMode::Stretch),
        Node {
            height: px(s),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(root),
    ));
    footer(commands, art, view, &shown, root, s);
}

/// The amount: label, field and quick values, the refusal, the slider and the limits.
#[allow(clippy::too_many_arguments)]
fn amount(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    state: &State,
    shown: &Shown,
    tone: &str,
    root: Entity,
    s: f32,
) {
    let terms = shown.terms.as_ref();
    let section = child(
        commands,
        root,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(fig::AMOUNT_GAP * s),
            padding: pad(fig::AMOUNT_PAD, s),
            ..default()
        },
    );
    let entry = child(
        commands,
        section,
        Node {
            column_gap: px(fig::ENTRY_GAP * s),
            align_items: AlignItems::Center,
            ..default()
        },
    );
    let label = match (&view.transfer, terms) {
        (Some(Transfer::Authorizing { giver, .. }), _) => format!(
            "AMOUNT {} GIVES  ·  prefilled with the request",
            giver.name().to_uppercase()
        ),
        (_, Some(t)) if Some(t.giver) == view.executing => {
            format!("AMOUNT TO GIVE  ·  0 to {}", t.limit)
        }
        (_, Some(t)) => format!("AMOUNT REQUESTED  ·  0 to {}", t.limit),
        _ => "AMOUNT  ·  opens once both are chosen".into(),
    };
    let l = words(
        commands,
        art,
        entry,
        Prose::named(label),
        fig::CAPS,
        true,
        QUIET,
        fig::CAPS_SPACING,
        s,
    );
    commands.entity(l).insert(Node {
        flex_grow: 1.,
        ..default()
    });
    let open = terms.is_some();
    let field_key = if shown.refusal.is_some() {
        "atlas-transfer-field-refused.png".to_owned()
    } else {
        format!("atlas-transfer-field-{tone}.png")
    };
    let field = button(
        commands,
        art,
        entry,
        &field_key,
        Node {
            width: px(fig::FIELD[0] * s),
            height: px(fig::FIELD[1] * s),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            column_gap: px(2. * s),
            padding: UiRect::horizontal(px(fig::FIELD_PAD * s)),
            ..default()
        },
        FIELD,
        false,
        Action::Transfer(Input::Field),
    );
    part(commands, field, "number-field");
    let value = words(
        commands,
        art,
        field,
        if state.entry.is_empty() {
            "0".to_owned()
        } else {
            state.entry.clone()
        },
        fig::VALUE,
        true,
        if shown.refusal.is_some() {
            PRIMARY
        } else if open && !state.entry.is_empty() {
            INK
        } else {
            QUIET
        },
        0.,
        s,
    );
    commands.entity(value).insert(Live::Value);
    if open {
        commands.spawn((
            ImageNode::new(image(art, "atlas-transfer-caret.png")),
            Node {
                width: px(fig::CARET[0] * s),
                height: px(fig::CARET[1] * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(field),
        ));
    }
    let max = terms.map(|t| t.limit);
    for (name, text, value) in [
        ("transfer:quick:1", "1".to_owned(), Some(1)),
        ("transfer:quick:2", "2".to_owned(), Some(2)),
        (
            "transfer:quick:max",
            max.map_or("Max".to_owned(), |m| format!("Max {m}")),
            None,
        ),
    ] {
        let current = open
            && !state.entry.is_empty()
            && value.or(max) == Some(shown.amount)
            && !(value.is_none() && max.is_some_and(|m| m <= 2));
        let quick = button(
            commands,
            art,
            entry,
            &if current {
                format!("atlas-transfer-quick-{tone}.png")
            } else {
                "atlas-transfer-quick.png".into()
            },
            Node {
                height: px(fig::QUICK_H * s),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(fig::QUICK_PAD * s)),
                ..default()
            },
            name,
            false,
            Action::Transfer(Input::Quick(value)),
        );
        part(commands, quick, name);
        words(
            commands,
            art,
            quick,
            text,
            fig::SEGMENT,
            true,
            if current { PAPER } else { INK },
            0.,
            s,
        );
    }
    if let Some(refusal) = &shown.refusal {
        let r = words(
            commands,
            art,
            section,
            refusal,
            fig::SMALL,
            true,
            PRIMARY,
            0.,
            s,
        );
        part(commands, r, "refusal");
    }
    // The slider spans the giver's balance; a unit held for owed Terror is hatched.
    let row = child(
        commands,
        section,
        Node {
            width: px(fig::SLIDER[0] * s),
            height: px(fig::SLIDER[1] * s),
            ..default()
        },
    );
    let abs = |x: f32, y: f32, w: f32, h: f32| Node {
        position_type: PositionType::Absolute,
        left: px(x * s),
        top: px(y * s),
        width: px(w * s),
        height: px(h * s),
        ..default()
    };
    commands.spawn((
        plate(art, "atlas-transfer-track.png"),
        abs(0., fig::TRACK_TOP, fig::SLIDER[0], fig::TRACK_H),
        Pickable::IGNORE,
        ChildOf(row),
        Part("track".into()),
    ));
    let at = terms.map_or(0., |t| position(t, shown.amount));
    if terms.is_some() {
        commands.spawn((
            plate(art, &format!("atlas-transfer-fill-{tone}.png")),
            abs(0., fig::TRACK_TOP, at, fig::TRACK_H),
            Pickable::IGNORE,
            ChildOf(row),
            Live::Fill,
        ));
    }
    if let Some(t) = terms.filter(|t| t.reserved > 0 && t.balance > 0) {
        let from = position(t, t.balance - t.reserved);
        commands.spawn((
            ImageNode::new(image(art, "atlas-transfer-hatch.png")).with_mode(
                NodeImageMode::Tiled {
                    tile_x: true,
                    tile_y: false,
                    stretch_value: s / 3.,
                },
            ),
            abs(from, fig::HATCH[0], fig::SLIDER[0] - from, fig::HATCH[1]),
            Pickable::IGNORE,
            ChildOf(row),
            Part("reserve-hatch".into()),
        ));
    }
    let balance = terms.map_or(0, |t| t.balance);
    let slider = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-fig::THUMB[0] / 2. * s),
                top: px(0.),
                width: px((fig::SLIDER[0] + fig::THUMB[0]) * s),
                height: px(fig::THUMB[1] * s),
                ..default()
            },
            Slider {
                track_click: TrackClick::Snap,
                ..default()
            },
            SliderValue(f32::from(shown.amount.min(balance))),
            SliderRange::new(0., f32::from(balance.max(1))),
            SliderStep(1.),
            SliderPrecision(0),
            Control {
                name: "transfer:slider".into(),
                choice: false,
                action: Action::Transfer(Input::Field),
            },
            ChildOf(row),
            Part("slider".into()),
        ))
        .observe(slide)
        .id();
    if terms.is_some() {
        commands.spawn((
            ImageNode::new(image(art, "atlas-transfer-thumb.png")),
            Node {
                position_type: PositionType::Absolute,
                left: px(at * s),
                top: px(0.),
                width: px(fig::THUMB[0] * s),
                height: px(fig::THUMB[1] * s),
                ..default()
            },
            SliderThumb,
            Live::Thumb,
            ChildOf(slider),
            Part("thumb".into()),
        ));
    }
    let tick = |commands: &mut Commands, text: String, x: f32, bold: bool, right: bool| {
        let node = commands
            .spawn((
                abs(x, fig::SLIDER_LABEL_TOP, fig::SLIDER_LABEL_W, 16.),
                Pickable::IGNORE,
                ChildOf(row),
            ))
            .id();
        let t = words(
            commands,
            art,
            node,
            text,
            fig::ROLE,
            bold,
            if bold { INK } else { QUIET },
            0.,
            s,
        );
        if right {
            commands.entity(t).insert((
                TextLayout::justify(Justify::Right),
                Node {
                    width: Val::Percent(100.),
                    ..default()
                },
            ));
        }
    };
    tick(commands, "0".into(), 0., false, false);
    match terms {
        Some(t) if t.limit < t.balance => {
            tick(
                commands,
                format!("{} · balance", t.balance),
                fig::SLIDER[0] - fig::SLIDER_LABEL_W,
                false,
                true,
            );
            tick(
                commands,
                format!("{} · limit", t.limit),
                position(t, t.limit) - fig::SLIDER_LABEL_W / 2.,
                true,
                false,
            );
        }
        Some(t) => tick(
            commands,
            format!("{} · limit", t.limit),
            fig::SLIDER[0] - fig::SLIDER_LABEL_W,
            true,
            true,
        ),
        None => tick(
            commands,
            "limit".into(),
            fig::SLIDER[0] - fig::SLIDER_LABEL_W,
            true,
            true,
        ),
    }
    // The limits strip.
    let strip = commands
        .spawn((
            plate(art, "atlas-transfer-strip.png"),
            Node {
                column_gap: px(fig::LIMITS_GAP * s),
                align_items: AlignItems::Center,
                padding: pad(fig::LIMITS_PAD, s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(section),
        ))
        .id();
    part(commands, strip, "limits");
    let entries: Vec<(&str, String, bool)> = match terms {
        Some(t) => limits(t, view.executing)
            .into_iter()
            .filter(|(icon, ..)| {
                view.current_card != 18
                    || *icon != "reserve"
                    || (view.executing == Some(Faction::Syndicate) && t.reserved > 0)
            })
            .collect(),
        None => vec![(
            "balance",
            if view.current_card == 18 {
                "Limited by the giver’s balance and the recipient’s 49-Resource cap".into()
            } else {
                "Limits: the giver's balance, the recipient's 49 cap, the owed-Terror reserve"
                    .into()
            },
            false,
        )],
    };
    for (icon, text, binding) in entries {
        let item = child(
            commands,
            strip,
            Node {
                column_gap: px(fig::LIMIT_GAP * s),
                align_items: AlignItems::Center,
                ..default()
            },
        );
        commands.spawn((
            ImageNode::new(image(art, &format!("atlas-transfer-icon-{icon}.png"))),
            Node {
                width: px(fig::ICON * s),
                height: px(fig::ICON * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(item),
        ));
        let t = words(
            commands,
            art,
            item,
            text,
            fig::SMALL,
            binding,
            if binding { INK } else { QUIET },
            0.,
            s,
        );
        part(commands, t, &format!("limit:{icon}"));
    }
}

/// The footer: a note and the buttons, right-aligned.
fn footer(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    shown: &Shown,
    root: Entity,
    s: f32,
) {
    let footer = child(
        commands,
        root,
        Node {
            column_gap: px(fig::FOOTER_GAP * s),
            align_items: AlignItems::Center,
            padding: pad(fig::FOOTER_PAD, s),
            ..default()
        },
    );
    let decision = match view.transfer {
        Some(Transfer::Authorizing { giver, .. }) => Some(giver),
        _ => None,
    };
    let note = match (decision, shown.giver) {
        (Some(_), _) => "Wait keeps it pending, across save and quit.".to_owned(),
        (None, Some(giver)) if Some(giver) == view.executing => {
            format!("{}'s own Resource transfer moves at once.", giver.name())
        }
        (None, Some(giver)) => format!("Nothing moves until {} authorizes.", giver.name()),
        (None, None) => "Zero and repeated transfers are allowed.".into(),
    };
    let funding = view
        .construct_funding
        .as_ref()
        .filter(|_| shown.giver != view.executing);
    let note = if let Some(funding) = funding {
        format!("{}\n{}", funding.summary(), ConstructFunding::CONDITION)
    } else {
        note
    };
    let n = words(commands, art, footer, note, fig::ROLE, false, QUIET, 0., s);
    if funding.is_some() {
        part(commands, n, "construct-funding");
    }
    commands.entity(n).insert(Node {
        flex_grow: 1.,
        ..default()
    });
    let secondary = |commands: &mut Commands, action: &str, text: &str, icon: bool| {
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        let b = button(
            commands,
            art,
            footer,
            "atlas-transfer-button.png",
            Node {
                height: px(fig::BUTTON_H * s),
                align_items: AlignItems::Center,
                column_gap: px(fig::WAIT_GAP * s),
                padding: UiRect::horizontal(px(fig::BUTTON_PAD * s)),
                ..default()
            },
            &id,
            true,
            Action::Decision(owner, view.revision),
        );
        part(commands, b, &format!("button:{action}"));
        if icon {
            commands.spawn((
                ImageNode::new(image(art, "atlas-transfer-icon-wait.png")),
                Node {
                    width: px(fig::ICON * s),
                    height: px(fig::ICON * s),
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(b),
            ));
        }
        words(commands, art, b, text, fig::BUTTON, false, INK, 0., s);
    };
    match decision {
        Some(_) => {
            secondary(commands, "wait", "Wait", true);
            secondary(commands, "decline", "Decline", false);
        }
        None => secondary(commands, "back", "Cancel", false),
    }
    let (key, name, action) = match &shown.commit {
        Some(choice) => (
            match decision {
                Some(giver) => format!("atlas-transfer-primary-{}.png", slug(giver)),
                None if view.executing == Some(Faction::Directorio) => {
                    "atlas-transfer-primary-directorio.png".into()
                }
                None => "atlas-transfer-primary.png".into(),
            },
            choice.id.clone(),
            Action::Decision(choice.owner, view.revision),
        ),
        None => (
            "atlas-transfer-primary-disabled.png".into(),
            "transfer:primary".into(),
            Action::Unsupported(shown.blocked.clone().unwrap_or_default()),
        ),
    };
    let primary = button(
        commands,
        art,
        footer,
        &key,
        Node {
            height: px(fig::BUTTON_H * s),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(fig::PRIMARY_PAD * s)),
            ..default()
        },
        &name,
        shown.commit.is_some(),
        action,
    );
    part(commands, primary, PRIMARY_PART);
    // The primary button is red while 26July composes, and in the giver's colour in a
    // decision. The faction whose colour it is keeps its name there, since its emblem
    // would vanish on it: #60's rule for 26July on red (accepted in #68), applied to
    // each giver's own colour. Its label names no other faction.
    let label = match (shown.commit.is_some(), decision) {
        (true, None | Some(Faction::M26)) => Prose::from(&shown.label).on_red(),
        (true, Some(_)) => Prose::named(&shown.label),
        (false, _) => Prose::from(&shown.label),
    };
    words(
        commands,
        art,
        primary,
        label,
        fig::BUTTON,
        true,
        PAPER,
        0.,
        s,
    );
}

/// After Wait: a tab in the top band, right of the standings, names the pending
/// request; Open, the authority's `reopen` choice, shows the giver's decision again. An
/// agent's giver has the tab from the request on, without Open (#206).
fn pending(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    ui: &Interface,
    s: f32,
    centered: Vec2,
) {
    let Some(Transfer::Authorizing {
        giver,
        recipient,
        amount,
        ..
    }) = view.transfer
    else {
        return;
    };
    let [x, y, w, h] = fig::PENDING;
    let tab = commands
        .spawn((
            Modal,
            plate(art, "atlas-transfer-pending.png"),
            Node {
                position_type: PositionType::Absolute,
                left: px(centered.x + x * s),
                top: px(centered.y + y * s),
                width: px(w * s),
                height: px(h * s),
                align_items: AlignItems::Center,
                column_gap: px(fig::PENDING_GAP * s),
                padding: UiRect {
                    left: px(fig::PENDING_PAD[1] * s),
                    right: px(fig::PENDING_PAD[2] * s),
                    top: px(fig::PENDING_PAD[0] * s),
                    bottom: px(fig::PENDING_PAD[0] * s),
                },
                ..default()
            },
            GlobalZIndex(LAYER),
            Pickable::IGNORE,
        ))
        .id();
    part(commands, tab, "pending-tab");
    let emblem = |commands: &mut Commands, faction: Faction| {
        commands.spawn((
            ImageNode::new(image(art, &emblems::image(faction))),
            Node {
                width: px(fig::PENDING_EMBLEM * s),
                height: px(fig::PENDING_EMBLEM * s),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(tab),
        ));
    };
    let tone = slug(giver);
    emblem(commands, recipient);
    let band = commands
        .spawn((
            plate(art, &format!("atlas-transfer-pending-band-{tone}.png")),
            Node {
                width: px(fig::PENDING_BAND[0] * s),
                height: px(fig::PENDING_BAND[1] * s),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(tab),
        ))
        .id();
    let mut arrow = ImageNode::new(image(art, "atlas-transfer-arrow.png"));
    arrow.flip_x = true;
    commands.spawn((
        arrow,
        Node {
            width: px(fig::PENDING_ARROW[0] * s),
            height: px(fig::PENDING_ARROW[1] * s),
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(band),
    ));
    emblem(commands, giver);
    let text = child(
        commands,
        tab,
        Node {
            flex_grow: 1.,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    );
    let status = words(
        commands,
        art,
        text,
        view.construct_funding.as_ref().map_or_else(
            || format!("{amount} requested · pending"),
            |funding| {
                format!(
                    "{} authorized · {} needed",
                    funding.authorized(),
                    funding.shortfall
                )
            },
        ),
        fig::SMALL,
        false,
        INK,
        0.,
        s,
    );
    let owner = words(
        commands,
        art,
        text,
        if view.construct_funding.is_some() {
            format!("{} decides · {amount} requested", giver.name())
        } else {
            format!("{} decides", giver.name())
        },
        fig::ROLE,
        false,
        QUIET,
        0.,
        s,
    );
    if view.construct_funding.is_some() {
        part(commands, status, "pending-construct-funding");
        part(commands, owner, "pending-construct-giver");
    }
    if !ui.plays(giver) {
        return;
    }
    let open = button(
        commands,
        art,
        tab,
        &format!("atlas-transfer-open-{tone}.png"),
        Node {
            width: px(fig::OPEN[0] * s),
            height: px(fig::OPEN[1] * s),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        &format!("{}:reopen", view.revision),
        true,
        Action::Decision(giver, view.revision),
    );
    part(commands, open, "pending-open");
    words(commands, art, open, "Open", fig::SMALL, false, PAPER, 0., s);
}

/// The modal's presentation state as the seat reads it.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let view = world.resource::<Play>().game().observe();
    let ui = world.resource::<Interface>();
    let shown_now = shown(view.transfer.as_ref(), view.owner, ui);
    let content = content(&view, &ui.transfer);
    let state = serde_json::json!({
        "shown": shown_now,
        "pending": matches!(view.transfer, Some(Transfer::Authorizing { .. })) && !shown_now,
        "counterpart": ui.transfer.counterpart,
        "direction": ui.transfer.direction,
        "entry": ui.transfer.entry,
        "amount": content.amount,
        "limit": content.terms.as_ref().map(|t| t.limit),
        "refusal": content.refusal,
        "commit": content.commit.map(|c| c.id),
        "label": content.label,
        "focus": ui.transfer.focus,
    });
    let mut parts = world
        .query::<(&Part, &ComputedNode, &UiGlobalTransform, Option<&emblems::Plain>)>()
        .iter(world)
        .map(|(part, node, at, text)| {
            (
                part.0.clone(),
                serde_json::json!({"part":part.0,"text":text.map(|t| &t.0),"bounds":seat::logical_bounds(node, at)}),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    serde_json::json!({
        "state": state,
        "parts": parts.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened() -> Game {
        let mut game = crate::terror_chosen(Game::opening());
        game.submit(Faction::M26, 1, "1:transfer").unwrap();
        game
    }

    /// #174: activating the primary button while it is blocked writes its reason as a
    /// refusal, which the notice slip draws; nothing is decided.
    #[test]
    fn the_blocked_primary_button_writes_a_refusal() {
        let dir = tempfile::tempdir().unwrap();
        let mut play = super::super::test_play(dir.path().join("task.json"));
        let mut ui = Interface::default();
        let view = play.game().observe();
        activate(&mut play, &mut ui, &view, "transfer:primary");
        assert_eq!(ui.notice.kind, super::super::notice::Kind::Refusal);
        assert_eq!(
            ui.notice.text,
            "Choose the other faction by its emblem first."
        );
        assert_eq!(play.game().observe().revision, view.revision);
    }

    /// The primary button submits only an offered choice, and words a refusal
    /// otherwise: from the rules authority's terms, never its own arithmetic.
    #[test]
    fn the_primary_button_follows_the_selection_and_the_terms() {
        let view = opened().observe();
        let mut state = State::default();
        let shown = content(&view, &state);
        assert_eq!(shown.label, "Choose a counterpart");
        assert!(shown.commit.is_none());
        state.counterpart = Some(Faction::Syndicate);
        assert_eq!(content(&view, &state).label, "Choose Give to or Ask from");
        state.direction = Some(Direction::Ask);
        state.set(2);
        let shown = content(&view, &state);
        assert_eq!(shown.label, "Send request: 2 from Syndicate");
        assert_eq!(shown.commit.unwrap().id, "2:ask:3:2");
        state.direction = Some(Direction::Give);
        let shown = content(&view, &state);
        assert_eq!(shown.label, "Give 2 to Syndicate");
        assert!(shown.commit.is_none());
        assert_eq!(
            shown.refusal.as_deref(),
            Some("2 is more than 0: 26July has 0 Resources.")
        );
    }

    /// The giver's decision is prefilled with the amount asked, and a new request
    /// prefills again.
    #[test]
    fn the_giver_s_amount_starts_from_the_request() {
        let mut game = opened();
        game.submit(Faction::M26, 2, "2:ask:3:2").unwrap();
        let view = game.observe();
        let mut state = State::default();
        state.sync(view.revision, view.transfer.as_ref());
        assert_eq!(state.amount(), 2);
        state.set(5);
        state.sync(view.revision, view.transfer.as_ref());
        assert_eq!(state.amount(), 5, "a redraw keeps the giver's typing");
        let shown = content(&view, &state);
        assert_eq!(shown.label, "Syndicate authorizes 5");
        assert_eq!(shown.commit.unwrap().owner, Faction::Syndicate);
        state.sync(view.revision, None);
        assert_eq!(state.entry, "");
    }

    /// The accounts show the typed amount's effect, refused or not: a giver asked for
    /// more than it holds goes below 0 rather than stopping there (#58 review).
    #[test]
    fn the_accounts_show_a_refused_amount_s_effect() {
        let view = opened().observe();
        let terms = |giver: Faction, recipient: Faction| {
            view.transfer_terms
                .iter()
                .find(|t| t.giver == giver && t.recipient == recipient)
                .unwrap()
                .clone()
        };
        // Prepared position: 26July holds 0, Syndicate 15 (#8's scenario).
        let give = terms(Faction::M26, Faction::Syndicate);
        assert!(give.refusal(2).is_some());
        assert_eq!(balances(&give, 2), (-2, 17));
        let ask = terms(Faction::Syndicate, Faction::M26);
        assert_eq!(balances(&ask, 5), (10, 5));
        assert!(ask.refusal(20).is_some());
        assert_eq!(balances(&ask, 20), (-5, 20));
    }

    /// TM-B's limits strip, worked from the prepared position and the Kidnap position.
    #[test]
    fn the_limits_strip_names_the_binding_limit() {
        let view = opened().observe();
        let ask = view
            .transfer_terms
            .iter()
            .find(|t| t.giver == Faction::Syndicate)
            .unwrap();
        assert_eq!(
            limits(ask, view.executing).map(|(_, t, b)| (t, b)),
            [
                ("Limit 15: Syndicate's balance".into(), true),
                ("26July holds at most 49".into(), false),
                ("No Terror owed: nothing reserved".into(), false),
            ]
        );
        let mut game = crate::terror_chosen(Game::opening_with_seed(5)); // shared roll 3
        game.submit(Faction::M26, 1, "1:kidnap").unwrap();
        game.submit(Faction::M26, 2, "2:kidnap:roll").unwrap();
        game.submit(Faction::M26, 3, "3:transfer").unwrap();
        let view = game.observe();
        let give = view
            .transfer_terms
            .iter()
            .find(|t| t.recipient == Faction::Directorio)
            .unwrap();
        assert_eq!(
            limits(give, view.executing).map(|(_, t, b)| (t, b)),
            [
                ("26July's balance: 3".into(), false),
                ("Directorio has room for 44".into(), false),
                ("Limit 2: 1 reserved for owed Terror".into(), true),
            ]
        );
    }

    #[test]
    fn the_slider_track_spans_the_giver_s_balance() {
        let view = opened().observe();
        let ask = view
            .transfer_terms
            .iter()
            .find(|t| t.giver == Faction::Syndicate)
            .unwrap();
        // TM-B: 2 of 15 sits 89.6 reference px along the 672 px track.
        assert!((position(ask, 2) - 89.6).abs() < 0.01);
        assert_eq!(position(ask, 15), fig::SLIDER[0]);
    }
}
