//! Layout C · Separate actions, from atlas-casino-actions.figcraft (see CASINOS.md).

use super::*;

pub(super) const WIDTH: f32 = 354.;
const INSET: f32 = 14.;
const INNER: f32 = 326.;
const LINE: f32 = 17.;
const STEPPER_SPACE: f32 = 34.;
const CLOSE: &str = "popover:casino-close";
const RETURN: &str = "popover:casino-return";

pub(super) fn content(view: &Observation, target: &PieceTarget, quantity: u8) -> Option<Card> {
    let place = view.spaces.iter().find(|space| space.id == target.space)?;
    let open = target.state == State::Open;
    let quantity = quantity.clamp(1, target.count);
    let mut actions: Vec<&Choice> = target
        .choices
        .iter()
        .filter_map(|id| {
            let choice = view.choices.iter().find(|choice| choice.id == *id)?;
            let (_, _, count) = choice.removal().or_else(|| choice.casino_closure())?;
            (count == quantity).then_some(choice)
        })
        .collect();
    actions.sort_by_key(|choice| choice.casino_closure().is_none());

    let pivot = actions.iter().find_map(|choice| choice.preview.as_ref());
    let both = actions.len() == 2 && actions.iter().all(|choice| choice.supported);
    let control = pivot.map(|preview| match preview.control.first() {
        Some(change) => match (change.from, change.to) {
            (Some(from), None) if both => format!("Both choices end {} Control here.", from.name()),
            (Some(from), None) => format!("{} Control here ends.", from.name()),
            (_, Some(to)) => format!("{} takes Control here.", to.name()),
            (None, None) => "This space stays uncontrolled.".into(),
        },
        None => place
            .control()
            .map_or("This space stays uncontrolled.".into(), |faction| {
                format!("{} retains Control here.", faction.name())
            }),
    });
    let open_count = view.standings[Faction::Syndicate.index()].score;
    let count = pivot.map(|preview| {
        let after = preview
            .standings
            .iter()
            .find(|change| change.faction == Faction::Syndicate)
            .map_or(open_count, |change| change.score_to);
        if open_count == after {
            format!("Open Casinos stay at {open_count}")
        } else {
            format!("Open Casinos  {open_count} → {after}")
        }
    });
    let same_options = match actions.as_slice() {
        [close, returned] => {
            let close = close.preview.as_ref().and_then(|p| p.management.as_ref());
            let returned = returned
                .preview
                .as_ref()
                .and_then(|p| p.management.as_ref());
            close
                .zip(returned)
                .filter(|(close, returned)| close.construct_options == returned.construct_options)
                .map(|(close, _)| close.construct_options.as_slice())
        }
        _ => None,
    };
    let same_refusal = match actions.as_slice() {
        [close, returned]
            if !close.supported
                && !returned.supported
                && close.explanation == returned.explanation =>
        {
            Some(close.explanation.as_str())
        }
        _ => None,
    };

    let mut shared = vec![
        (Tone::Strong, control.unwrap_or_default()),
        (Tone::Plain, count.unwrap_or_default()),
    ];
    if let Some(reason) = same_refusal {
        shared = wrapped(Tone::Secondary, reason);
    }
    if let Some(options) = same_options {
        shared.extend(options.iter().flat_map(|note| wrapped(Tone::Plain, note)));
    }
    let mut rows = vec![Row {
        choice: None,
        owner: Faction::Syndicate,
        mark: Mark::Selectable,
        chosen: false,
        head: String::new(),
        kind: String::new(),
        lines: shared,
        remedies: vec![],
        no_remedy: None,
        button: None,
    }];
    for action in actions.iter().copied() {
        let close = action.casino_closure().is_some();
        let mut lines = vec![];
        if let Some(change) = action.preview.as_ref().and_then(|p| p.management.as_ref()) {
            let [before, after] = change.casinos_available;
            let pieces = if quantity == 1 {
                "Piece leaves"
            } else {
                "Pieces leave"
            };
            let closed = if quantity == 1 {
                "Closed piece stays"
            } else {
                "Closed pieces stay"
            };
            let detail = if close {
                format!("{closed} here. Available stays at {after}.")
            } else {
                format!("{pieces} the map. Available goes from {before} to {after}.")
            };
            lines.push((Tone::Plain, detail));
            if same_options.is_none() {
                lines.extend(
                    change
                        .construct_options
                        .iter()
                        .flat_map(|note| wrapped(Tone::Plain, note)),
                );
            }
        } else if same_refusal.is_none() {
            lines = wrapped(Tone::Secondary, &action.explanation);
        }
        rows.push(Row {
            choice: Some(action.id.clone()),
            owner: action.owner,
            mark: if action.supported {
                Mark::Selectable
            } else {
                Mark::Unavailable
            },
            chosen: action.supported,
            head: action.label.clone(),
            kind: if close { CLOSE } else { RETURN }.into(),
            lines,
            remedies: vec![],
            no_remedy: None,
            button: action.supported.then(|| action.label.clone()),
        });
    }
    Some(Card {
        kind: if open {
            CardKind::OpenCasino
        } else {
            CardKind::ClosedCasino
        },
        space: target.space.clone(),
        piece: Some(format!("{}:{}", target.space, target.group)),
        revision: view.revision,
        title: format!("Casino in {}", place.name),
        subtitle: format!(
            "Syndicate  ·  {} {} Casino{}",
            target.count,
            if open { "open" } else { "closed" },
            if target.count == 1 { "" } else { "s" }
        ),
        rows,
        stepper: (target.count > 1).then_some(Stepper {
            value: quantity,
            max: target.count,
            count: target.count,
        }),
        cancel: true,
        footer: if pivot.is_some() {
            "0 Resources · no Profit".into()
        } else {
            String::new()
        },
    })
}

fn wrapped(tone: Tone, words: &str) -> Vec<(Tone, String)> {
    super::super::briefing::wrap(words, 49)
        .into_iter()
        .map(|line| (tone, line))
        .collect()
}

pub(super) fn controls(card: &Card) -> impl Iterator<Item = (String, Action)> + '_ {
    card.rows.iter().filter_map(|row| {
        let choice = row.choice.as_ref()?;
        row.button.as_ref()?;
        Some((
            row.kind.clone(),
            Action::Submit(row.owner, card.revision, choice.clone()),
        ))
    })
}

struct Layout<'a> {
    shared: &'a [(Tone, String)],
    close: Option<&'a Row>,
    returned: &'a Row,
    stepper: f32,
}

impl<'a> Layout<'a> {
    fn new(card: &'a Card) -> Self {
        let mut actions = card.rows.iter().skip(1);
        let first = actions.next().expect("Casino card has a return action");
        let (close, returned) = actions.next().map_or((None, first), |r| (Some(first), r));
        Self {
            shared: &card.rows[0].lines,
            close,
            returned,
            stepper: if card.stepper.is_some() {
                STEPPER_SPACE
            } else {
                0.
            },
        }
    }

    fn shared_extra(&self) -> f32 {
        self.shared.len().saturating_sub(2) as f32 * LINE
    }

    fn row_extra(row: &Row) -> f32 {
        row.lines.len().saturating_sub(1) as f32 * LINE
    }

    fn offset(&self) -> f32 {
        self.stepper + self.shared_extra()
    }

    fn end(&self) -> f32 {
        self.offset() + Self::row_extra(self.returned) + self.close.map_or(-77., Self::row_extra)
    }
}

pub(super) fn height(card: &Card) -> f32 {
    300. + Layout::new(card).end()
}

fn slot(commands: &mut Commands, parent: Entity, s: f32, rect: [f32; 4]) -> Entity {
    let [x, y, w, h] = rect.map(|n| n * s);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                top: px(y),
                width: px(w),
                height: if h > 0. { px(h) } else { Val::Auto },
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(parent),
        ))
        .id()
}

#[allow(clippy::too_many_arguments)]
fn caption(
    commands: &mut Commands,
    ink: &Ink,
    parent: Entity,
    s: f32,
    text: &str,
    at: [f32; 3],
    font: &Handle<Font>,
    size: f32,
    color: Color,
) {
    let [x, y, w] = at;
    let node = slot(commands, parent, s, [x, y, w, 0.]);
    words(
        commands,
        ink,
        node,
        text,
        Naming::Names,
        false,
        font,
        size * s,
        color,
        true,
    );
}

fn divider(commands: &mut Commands, ink: &Ink, body: Entity, s: f32, y: f32) {
    let node = slot(commands, body, s, [INSET, y, INNER, 1.]);
    rule(
        commands,
        ink.art,
        node,
        s,
        plate::DIVIDER,
        "popover-casino-divider".into(),
    );
}

#[allow(clippy::too_many_arguments)]
fn action(
    commands: &mut Commands,
    ink: &Ink,
    body: Entity,
    card: &Card,
    row: &Row,
    bold: &Handle<Font>,
    s: f32,
    [rule_y, copy_y, button_y]: [f32; 3],
) {
    divider(commands, ink, body, s, rule_y);
    let regular = &ink.art.font;
    for (at, (tone, line)) in row.lines.iter().enumerate() {
        caption(
            commands,
            ink,
            body,
            s,
            line,
            [INSET, copy_y + at as f32 * LINE, INNER],
            regular,
            13.,
            if *tone == Tone::Secondary { QUIET } else { INK },
        );
    }
    let button = slot(
        commands,
        body,
        s,
        [INSET, button_y + Layout::row_extra(row), INNER, 32.],
    );
    let mut entity = commands.entity(button);
    entity.entry::<Node>().and_modify(|mut node| {
        node.justify_content = JustifyContent::Center;
        node.align_items = AlignItems::Center;
    });
    entity.insert(Part(row.kind.clone()));
    let primary = row.kind == CLOSE && row.button.is_some();
    if let (Some(choice), Some(_)) = (row.choice.as_ref(), row.button.as_ref()) {
        entity
            .remove::<Pickable>()
            .insert(Control {
                name: row.kind.clone(),
                choice: false,
                action: Action::Submit(row.owner, card.revision, choice.clone()),
            })
            .observe(click);
    }
    plate(
        commands,
        ink.art,
        button,
        if primary {
            plate::PRIMARY
        } else {
            plate::CANCEL
        },
        s,
    );
    let color = if primary {
        PAPER
    } else if row.button.is_some() {
        INK
    } else {
        QUIET
    };
    words(
        commands,
        ink,
        button,
        &row.head,
        Naming::Names,
        primary,
        bold,
        14. * s,
        color,
        false,
    );
}

pub(super) fn draw(
    commands: &mut Commands,
    ink: &Ink,
    body: Entity,
    card: &Card,
    bold: &Handle<Font>,
    s: f32,
) {
    let regular = &ink.art.font;
    let layout = Layout::new(card);
    let bar = slot(commands, body, s, [0., 0., WIDTH, 3.]);
    rule(
        commands,
        ink.art,
        bar,
        3. * s,
        plate::RULE_PRIMARY,
        "popover-rule".into(),
    );
    let graphic = slot(commands, body, s, [14., 16., 36., 22.]);
    let file = if card.kind == CardKind::OpenCasino {
        "atlas-casino-open.png"
    } else {
        "atlas-casino-plain.png"
    };
    commands
        .entity(graphic)
        .insert(ImageNode::new(ink.art.images[file].clone()));
    caption(
        commands,
        ink,
        body,
        s,
        &card.title,
        [60., 10., 280.],
        bold,
        16.,
        INK,
    );
    caption(
        commands,
        ink,
        body,
        s,
        &card.subtitle,
        [60., 32., 280.],
        regular,
        12.,
        QUIET,
    );
    if let Some(stepper) = card.stepper {
        let node = slot(commands, body, s, [INSET, 52., INNER, 26.]);
        quantity_stepper(commands, ink, node, stepper, "Quantity", bold, s);
    }
    divider(commands, ink, body, s, 57. + layout.stepper);
    for (at, (tone, line)) in layout.shared.iter().enumerate() {
        let y = if at == 0 {
            66.
        } else {
            88. + (at - 1) as f32 * LINE
        };
        let (font, color) = match tone {
            Tone::Strong => (bold, INK),
            Tone::Secondary => (regular, QUIET),
            _ => (regular, INK),
        };
        caption(
            commands,
            ink,
            body,
            s,
            line,
            [INSET, y + layout.stepper, INNER],
            font,
            13.,
            color,
        );
    }
    let offset = layout.offset();
    if let Some(close) = layout.close {
        action(
            commands,
            ink,
            body,
            card,
            close,
            bold,
            s,
            [114., 124., 148.].map(|y| y + offset),
        );
    }
    let return_offset = offset + layout.close.map_or(-77., Layout::row_extra);
    action(
        commands,
        ink,
        body,
        card,
        layout.returned,
        bold,
        s,
        [191., 202., 225.].map(|y| y + return_offset),
    );

    let y = 269. + layout.end();
    let cancel = slot(commands, body, s, [14., y, 43., 20.]);
    commands
        .entity(cancel)
        .remove::<Pickable>()
        .insert(Control {
            name: "popover:cancel".into(),
            choice: false,
            action: Action::Dismiss,
        })
        .observe(click);
    words(
        commands,
        ink,
        cancel,
        "Cancel",
        Naming::Names,
        false,
        bold,
        13. * s,
        QUIET,
        false,
    );
    caption(
        commands,
        ink,
        body,
        s,
        &card.footer,
        [170., y, 170.],
        regular,
        11.,
        QUIET,
    );
}
