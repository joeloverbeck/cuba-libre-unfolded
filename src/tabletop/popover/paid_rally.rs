//! The selected refined B frame in art/atlas/paid-rally/rally.figcraft.
use super::*;
use crate::{Group, Kind, RallyAction, Space, State, support_level};

pub(super) const WIDTH: f32 = 430.;
pub(super) const HEIGHT: f32 = 539.;
const FACTION: Faction = Faction::Directorio;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Procedure {
    Recruit,
    Exchange,
    Recover,
}

impl Procedure {
    const ALL: [Self; 3] = [Self::Recruit, Self::Exchange, Self::Recover];

    fn of(action: RallyAction) -> Self {
        match action {
            RallyAction::Place(_) => Self::Recruit,
            RallyAction::ReplaceWithBase { .. } => Self::Exchange,
            RallyAction::FlipUnderground(_) => Self::Recover,
        }
    }

    fn control(self) -> &'static str {
        match self {
            Self::Recruit => "popover:row:recruit",
            Self::Exchange => "popover:row:exchange",
            Self::Recover => "popover:row:recover",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Recruit => "Recruit Guerrillas",
            Self::Exchange => "Exchange 2 Guerrillas for 1 Base",
            Self::Recover => "Turn all Guerrillas Underground",
        }
    }

    fn image(self) -> Option<&'static str> {
        match self {
            Self::Recruit => Some("atlas-available-directorio-guerrilla.png"),
            Self::Exchange => Some("atlas-available-directorio-base.png"),
            Self::Recover => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
struct OptionRow {
    effect: RallyAction,
    action: String,
}

impl OptionRow {
    fn variant(&self) -> (String, String) {
        match self.effect {
            RallyAction::Place(n) => (format!("popover:quantity:{n}"), n.to_string()),
            RallyAction::ReplaceWithBase { underground, .. } => (
                format!("popover:exchange:{underground}"),
                match underground {
                    0 => "2 Active",
                    1 => "1 of each",
                    _ => "2 Underground",
                }
                .into(),
            ),
            RallyAction::FlipUnderground(_) => unreachable!("Recovery has no variants"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Procedures {
    options: Vec<OptionRow>,
    selected: RallyAction,
    arithmetic: String,
    recovery: String,
    preview: Option<Payment>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
struct Payment {
    here: [u8; 2],
    available: [u8; 2],
    bases_here: [u8; 2],
    bases_available: [u8; 2],
    active: [u8; 2],
    underground: [u8; 2],
    resources: [u8; 2],
    control: String,
    standing: Option<[i16; 2]>,
}

fn count(pieces: &[Group], kind: Kind) -> u8 {
    crate::game::total(pieces, FACTION, kind)
}

pub(super) fn content(
    view: &Observation,
    place: &Space,
    here: &[&Choice],
    selected: Option<&str>,
) -> Option<Card> {
    let choices: Vec<_> = here
        .iter()
        .filter_map(|choice| Some((choice.paid_rally_action()?, *choice)))
        .collect();
    let recruit = choices
        .iter()
        .find(|(effect, _)| matches!(effect, RallyAction::Place(_)));
    let initial = choices
        .iter()
        .find(|(effect, _)| *effect == RallyAction::Place(1))
        .or_else(|| choices.first())?;
    let &(effect, choice) = choices
        .iter()
        .find(|(_, choice)| Some(choice.action()) == selected)
        .unwrap_or(initial);

    let arithmetic = recruit
        .and_then(|(_, choice)| choice.explanation.lines().next())
        .unwrap_or("No Available Guerrillas.")
        .into();
    let recovery = match choices.iter().find_map(|(effect, _)| match effect {
        RallyAction::FlipUnderground(n) => Some(*n),
        _ => None,
    }) {
        None => "Requires a Directorio Base.",
        Some(0) => match count(&place.pieces, Kind::Guerrilla) {
            0 => "No Guerrillas here.",
            1 => "Already Underground.",
            2 => "Both are already Underground.",
            _ => "All are already Underground.",
        },
        Some(1) => "Turn 1 Active Guerrilla Underground.",
        Some(_) => "Turn all Active Guerrillas Underground.",
    };

    Some(Card {
        kind: CardKind::PaidRally,
        space: place.id.clone(),
        piece: None,
        revision: view.revision,
        title: place.name.clone(),
        subtitle: format!("Directorio Rally  ·  {}", support_level(place.support)),
        rows: vec![Row {
            choice: Some(choice.id.clone()),
            owner: choice.owner,
            mark: if choice.supported {
                Mark::Selectable
            } else {
                Mark::Unavailable
            },
            chosen: true,
            head: Procedure::of(effect).title().into(),
            kind: String::new(),
            lines: Vec::new(),
            remedies: Vec::new(),
            no_remedy: None,
            button: choice.supported.then(|| match effect {
                RallyAction::Place(0) => "Pay 1  ·  Place 0 Guerrillas".into(),
                RallyAction::Place(_) => format!("Pay 1  ·  {}", choice.label),
                RallyAction::ReplaceWithBase { .. } => "Pay 1  ·  Exchange for 1 Base".into(),
                RallyAction::FlipUnderground(_) => "Pay 1  ·  Turn Guerrillas Underground".into(),
            }),
        }],
        stepper: None,
        cancel: false,
        footer: String::new(),
        rally: Some(Procedures {
            options: choices
                .iter()
                .map(|(effect, choice)| OptionRow {
                    effect: *effect,
                    action: choice.action().into(),
                })
                .collect(),
            selected: effect,
            arithmetic,
            recovery: recovery.into(),
            preview: payment_preview(view, place, choice),
        }),
    })
}

fn payment_preview(view: &Observation, place: &Space, choice: &Choice) -> Option<Payment> {
    let preview = choice.preview.as_ref()?;
    let payment = preview
        .resources
        .iter()
        .find(|change| change.faction == FACTION)?;
    let change = |kind| {
        let here = count(&place.pieces, kind);
        let available = count(&view.available, kind);
        let placed = preview
            .placement_available_forces
            .iter()
            .find(|p| p.piece.kind == kind && p.piece.faction == FACTION);
        let added = placed.map_or(0, |p| p.piece.count);
        let returned: u8 = preview
            .removed
            .iter()
            .filter(|p| p.kind == kind && p.faction == FACTION)
            .map(|p| p.count)
            .sum();
        let available_after = placed.map_or(available + returned, |p| p.to);
        (
            [here, here + added - returned],
            [available, available_after],
        )
    };
    let (here, available) = change(Kind::Guerrilla);
    let (bases_here, bases_available) = change(Kind::Base);

    let face_count = |state| {
        place
            .pieces
            .iter()
            .find(|p| p.faction == FACTION && p.kind == Kind::Guerrilla && p.state == state)
            .map_or(0, |p| p.count)
    };
    let flips = preview
        .flips
        .iter()
        .find(|p| p.faction == FACTION && p.to == State::Underground)
        .map_or(0, |p| p.count);
    let active = face_count(State::Active);
    let underground = face_count(State::Underground);
    let standing = preview
        .standings
        .iter()
        .find(|change| change.faction == FACTION)
        .map(|s| [s.score_from, s.score_to]);
    let control = match (place.control(), preview.control.first()) {
        (_, Some(change)) => format!(
            "Control: {} → {}.",
            control_name(change.from),
            control_name(change.to)
        ),
        (Some(owner), None) => format!("{} retains Control.", owner.name()),
        (None, None) => "The space stays Uncontrolled.".into(),
    };
    Some(Payment {
        here,
        available,
        bases_here,
        bases_available,
        active: [active, active - flips],
        underground: [underground, underground + flips],
        resources: [payment.from, payment.to],
        control,
        standing,
    })
}

fn control_name(control: Option<Faction>) -> &'static str {
    control.map_or("Uncontrolled", Faction::name)
}

pub(super) fn focusable(card: &Card, rally: &Procedures) -> Vec<(String, Action)> {
    let mut stops = Vec::new();
    for procedure in Procedure::ALL {
        if let Some(option) = rally.row_choice(procedure) {
            stops.push((
                procedure.control().into(),
                Action::Row(option.action.clone()),
            ));
        }
        if let Some(variants) = rally.selected_variants(procedure) {
            stops.extend(
                variants
                    .iter()
                    .map(|option| (option.variant().0, Action::Row(option.action.clone()))),
            );
        }
    }
    if card.rows[0].button.is_some() {
        stops.push(("popover:commit".into(), payment_action(card)));
    }
    stops
}

impl Procedures {
    fn options_for(&self, procedure: Procedure) -> impl Iterator<Item = &OptionRow> {
        self.options
            .iter()
            .filter(move |option| Procedure::of(option.effect) == procedure)
    }

    fn row_choice(&self, procedure: Procedure) -> Option<&OptionRow> {
        self.options_for(procedure)
            .find(|option| option.effect == self.selected)
            .or_else(|| {
                self.options_for(procedure)
                    .find(|option| option.effect == RallyAction::Place(1))
            })
            .or_else(|| self.options_for(procedure).next())
    }

    fn selected_variants(&self, procedure: Procedure) -> Option<Vec<&OptionRow>> {
        if Procedure::of(self.selected) != procedure {
            return None;
        }
        let variants: Vec<_> = self.options_for(procedure).collect();
        (variants.len() > 1).then_some(variants)
    }
}

fn payment_action(card: &Card) -> Action {
    let row = &card.rows[0];
    Action::Submit(
        row.owner,
        card.revision,
        row.choice.clone().expect("A payment is a choice"),
    )
}

type Bounds = [f32; 4];

struct Layout<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    art: &'a Art,
    bold: &'a Handle<Font>,
    scale: f32,
}

impl Layout<'_, '_, '_> {
    fn node(&mut self, parent: Entity, bounds: Bounds, part: &str) -> Entity {
        let [x, y, w, h] = bounds.map(|n| px(n * self.scale));
        let mut entity = self.commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: x,
                top: y,
                width: w,
                height: h,
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(parent),
        ));
        if !part.is_empty() {
            entity.insert(Part(part.into()));
        }
        entity.id()
    }

    fn solid(&mut self, parent: Entity, bounds: Bounds, part: &str, color: Color) -> Entity {
        let node = self.node(parent, bounds, part);
        self.commands.entity(node).insert(BackgroundColor(color));
        node
    }

    fn image(&mut self, parent: Entity, bounds: Bounds, file: &str) {
        let node = self.node(parent, bounds, "");
        self.commands
            .entity(node)
            .insert(ImageNode::new(self.art.images[file].clone()));
    }

    fn control(&mut self, entity: Entity, name: impl Into<String>, action: Action) {
        self.commands
            .entity(entity)
            .remove::<Pickable>()
            .insert(Control {
                name: name.into(),
                choice: false,
                action,
            })
            .observe(click);
    }

    fn text(
        &mut self,
        parent: Entity,
        bounds: Bounds,
        words: impl Into<String>,
        style: (f32, bool, Color),
    ) -> Entity {
        let node = self.node(parent, bounds, "");
        let (size, bold, color) = style;
        let words = words.into();
        let font = if bold { self.bold } else { &self.art.font };
        self.commands.entity(node).insert((
            Text::new(words.clone()),
            emblems::Plain(words),
            TextFont {
                font: font.clone().into(),
                font_size: (size * self.scale).into(),
                ..default()
            },
            TextColor(color),
        ));
        node
    }

    fn centered(&mut self, parent: Entity, bounds: Bounds, words: impl Into<String>, color: Color) {
        let node = self.text(parent, bounds, words, (18., true, color));
        self.commands
            .entity(node)
            .insert(TextLayout::justify(Justify::Center));
    }
}

pub(super) fn draw(
    commands: &mut Commands,
    art: &Art,
    body: Entity,
    card: &Card,
    rally: &Procedures,
    bold: &Handle<Font>,
    scale: f32,
) {
    let mut layout = Layout {
        commands,
        art,
        bold,
        scale,
    };
    layout.solid(body, [0., 0., WIDTH, 4.], "popover-rule", PRIMARY);
    layout.image(body, [20., 18., 30., 30.], &emblems::image(FACTION));
    layout.text(body, [61., 11., 348., 31.], &card.title, (26., true, INK));
    layout.text(
        body,
        [61., 48., 348., 19.],
        &card.subtitle,
        (16., false, QUIET),
    );

    // The FigCraft variants give the selected procedure the extra row height.
    // Its heading and controls move together, while the preview and Pay stay fixed.
    let selected = Procedure::of(rally.selected);
    let mut top = 84.;
    for (procedure, collapsed) in Procedure::ALL.into_iter().zip([50., 50., 61.]) {
        let height = if procedure == selected {
            collapsed + 54.
        } else {
            collapsed
        };
        draw_procedure(
            &mut layout,
            body,
            [12., top, 406., height],
            card,
            rally,
            procedure,
        );
        top += height;
        layout.solid(body, [20., top, 390., 1.], "", RULE);
    }

    let effect = layout.node(body, [20., 310., 390., 155.], "rally-payment-preview");
    layout.text(
        effect,
        [0., 1., 176., 17.],
        "IF YOU PAY",
        (14., true, QUIET),
    );
    if let Some(preview) = &rally.preview {
        draw_payment(&mut layout, effect, preview, rally.selected);
    } else {
        layout.text(
            effect,
            [0., 35., 390., 24.],
            "Needs 1 Directorio Resource.",
            (19., true, INK),
        );
    }

    let button = [20., 479., 390., 44.];
    if let Some(label) = &card.rows[0].button {
        let pay = layout.solid(body, button, "popover-commit", PRIMARY);
        layout.control(pay, "popover:commit", payment_action(card));
        layout.centered(pay, [0., 9., 390., 25.], label, PAPER);
    } else if rally.preview.is_some() {
        layout.centered(body, button, "Directorio’s seat pays", QUIET);
    }
}

fn draw_procedure(
    layout: &mut Layout,
    body: Entity,
    bounds: Bounds,
    card: &Card,
    rally: &Procedures,
    procedure: Procedure,
) {
    use super::super::palette::DESK;
    let selected = Procedure::of(rally.selected) == procedure;
    let choice = rally.row_choice(procedure);
    let row = if selected {
        let row = layout.solid(body, bounds, "rally-selected-row", DESK);
        layout.image(row, [8., 16., 12., 12.], mark_image(card.rows[0].mark));
        row
    } else {
        layout.node(body, bounds, "")
    };
    if let Some(choice) = choice {
        layout.control(row, procedure.control(), Action::Row(choice.action.clone()));
    }
    if let Some(image) = procedure.image() {
        layout.image(row, [28., 9., 19., 24.], image);
    }
    let color = if choice.is_some() { INK } else { QUIET };
    layout.text(
        row,
        [53., 8., 343., 22.],
        procedure.title(),
        (18., true, color),
    );

    match procedure {
        Procedure::Recruit if selected => {
            layout.text(
                row,
                [53., 34., 340., 25.],
                &rally.arithmetic,
                (16., false, QUIET),
            );
        }
        Procedure::Exchange => {
            let summary = match rally.selected {
                RallyAction::ReplaceWithBase { underground, .. } => match underground {
                    0 => "Return both Active Guerrillas.",
                    1 => "Return 1 Active and 1 Underground Guerrilla.",
                    _ => "Return both Underground Guerrillas.",
                },
                _ if choice.is_none() => "No legal exchange here.",
                _ => "",
            };
            if !summary.is_empty() {
                layout.text(
                    row,
                    [53., 43. - if selected { 0. } else { 16. }, 343., 22.],
                    summary,
                    (if selected { 16. } else { 15. }, false, QUIET),
                );
            }
        }
        Procedure::Recover => {
            layout.text(
                row,
                [53., 32., 343., 22.],
                &rally.recovery,
                (15., false, QUIET),
            );
        }
        Procedure::Recruit => {}
    }
    if let Some(variants) = rally.selected_variants(procedure) {
        draw_variants(layout, row, rally.selected, &variants);
    }
}

fn draw_variants(layout: &mut Layout, row: Entity, selected: RallyAction, variants: &[&OptionRow]) {
    let recruit = Procedure::of(selected) == Procedure::Recruit;
    let count = variants.len() as f32;
    let width = ((345. - 8. * (count - 1.)) / count).min(if recruit { 47. } else { 109. });
    let (top, height) = if recruit { (61., 33.) } else { (66., 28.) };

    for (index, option) in variants.iter().enumerate() {
        let chosen = option.effect == selected;
        let (name, label) = option.variant();
        let cell = layout.solid(
            row,
            [53. + index as f32 * (width + 8.), top, width, height],
            &name,
            if chosen { INK } else { PAPER },
        );
        layout.control(cell, name, Action::Row(option.action.clone()));
        let color = if chosen { PAPER } else { INK };
        let size = if recruit { 18. } else { 15. };
        let text = layout.text(cell, [0., 4., width, 24.], label, (size, true, color));
        layout
            .commands
            .entity(text)
            .insert(TextLayout::justify(Justify::Center));
    }
}

fn draw_payment(layout: &mut Layout, effect: Entity, preview: &Payment, action: RallyAction) {
    let control = match preview.standing {
        Some([from, to]) => {
            let control = if preview.control == "Directorio retains Control." {
                "Control retained"
            } else {
                preview.control.trim_end_matches('.')
            };
            format!("{control}  ·  Directorio standing {from} → {to}")
        }
        None => preview.control.clone(),
    };
    match action {
        RallyAction::Place(0) | RallyAction::FlipUnderground(0) => {
            let words = if action == RallyAction::Place(0) {
                "No pieces move."
            } else {
                "No Guerrillas change face."
            };
            layout.text(effect, [0., 35., 390., 23.], words, (19., true, INK));
            layout.text(
                effect,
                [0., 66., 390., 19.],
                "This counts as Rallying in this space.",
                (16., false, QUIET),
            );
        }
        RallyAction::Place(_) | RallyAction::ReplaceWithBase { .. } => {
            layout.text(effect, [189., 7., 92., 18.], "Here", (15., true, QUIET));
            layout.text(
                effect,
                [280., 7., 110., 18.],
                "Available",
                (15., true, QUIET),
            );
            payment_row(
                layout,
                effect,
                36.,
                "Guerrillas",
                preview.here,
                Some(preview.available),
            );
            if matches!(action, RallyAction::ReplaceWithBase { .. }) {
                payment_row(
                    layout,
                    effect,
                    63.,
                    "Bases",
                    preview.bases_here,
                    Some(preview.bases_available),
                );
            }
        }
        RallyAction::FlipUnderground(_) => {
            layout.text(effect, [189., 7., 92., 18.], "Here", (15., true, QUIET));
            payment_row(
                layout,
                effect,
                36.,
                "Active Guerrillas",
                preview.active,
                None,
            );
            payment_row(
                layout,
                effect,
                63.,
                "Underground Guerrillas",
                preview.underground,
                None,
            );
        }
    }
    let [from, to] = preview.resources;
    layout.text(
        effect,
        [0., 95., 174., 20.],
        "Directorio Resources",
        (17., false, INK),
    );
    layout.text(
        effect,
        [174., 93., 216., 22.],
        format!("{from} → {to}"),
        (18., true, INK),
    );
    layout.text(effect, [0., 124., 390., 22.], control, (15., false, QUIET));
}

fn payment_row(
    layout: &mut Layout,
    effect: Entity,
    top: f32,
    label: &str,
    [from, to]: [u8; 2],
    available: Option<[u8; 2]>,
) {
    layout.text(effect, [0., top + 1., 186., 20.], label, (17., false, INK));
    layout.text(
        effect,
        [189., top, 92., 22.],
        format!("{from} → {to}"),
        (18., true, INK),
    );
    if let Some([from, to]) = available {
        layout.text(
            effect,
            [280., top, 110., 22.],
            format!("{from} → {to}"),
            (18., true, INK),
        );
    }
}
