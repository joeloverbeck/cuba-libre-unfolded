//! The selected refined B frame in art/atlas/paid-rally/rally.figcraft.
use super::*;
use crate::{Group, Kind, Space, State, support_level};

pub(super) const WIDTH: f32 = 430.;
pub(super) const HEIGHT: f32 = 539.;
const FACTION: Faction = Faction::Directorio;
const PROCEDURE: &str = "popover:row:recruit";

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Recruitment {
    quantities: Vec<(u8, String)>,
    selected: u8,
    arithmetic: String,
    exchange: &'static str,
    recovery: &'static str,
    preview: Option<Payment>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
struct Payment {
    here: [u8; 2],
    available: [u8; 2],
    resources: [u8; 2],
    control: String,
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
        .filter_map(|choice| Some((choice.rally_recruitment()?, *choice)))
        .collect();
    let initial = choices
        .iter()
        .find(|(quantity, _)| *quantity == 1)
        .or_else(|| choices.first())?;
    let (quantity, choice) = choices
        .iter()
        .find(|(_, choice)| Some(choice.action()) == selected)
        .unwrap_or(initial);
    let bases = count(&place.pieces, Kind::Base);
    let guerrillas = count(&place.pieces, Kind::Guerrilla);
    let available = count(&view.available, Kind::Guerrilla);
    let population = place.population;
    let arithmetic = if bases == 0 {
        "Place one Available Guerrilla Underground.".into()
    } else {
        let limit = choices.last().map_or(0, |(quantity, _)| *quantity);
        let ceiling = bases + population;
        if limit < ceiling {
            format!("{bases} Base + {population} population = {ceiling}. Available: {limit}.")
        } else {
            format!("{bases} Base + {population} population allows up to {limit}.")
        }
    };
    let exchange = if guerrillas < 2 {
        "Requires two Directorio Guerrillas here."
    } else if count(&view.available, Kind::Base) == 0 {
        "No Directorio Base is Available."
    } else if place
        .pieces
        .iter()
        .filter(|p| p.kind == Kind::Base || p.kind == Kind::Casino)
        .map(|p| p.count)
        .sum::<u8>()
        >= 2
    {
        "Both Base slots are occupied."
    } else {
        "Outside this continuation."
    };
    let recovery = if bases == 0 {
        "Requires a Directorio Base here."
    } else if guerrillas == 0 {
        "No Directorio Guerrillas here."
    } else if !place
        .pieces
        .iter()
        .any(|p| p.faction == FACTION && p.kind == Kind::Guerrilla && p.state == State::Active)
    {
        "Already Underground · outside this continuation."
    } else {
        "Outside this continuation."
    };
    let preview = choice.preview.as_ref().and_then(|preview| {
        let payment = preview
            .resources
            .iter()
            .find(|change| change.faction == FACTION)?;
        let (added, supply) = preview
            .placement_available_forces
            .first()
            .map_or((0, [available; 2]), |change| {
                (change.piece.count, [change.from, change.to])
            });
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
            here: [guerrillas, guerrillas + added],
            available: supply,
            resources: [payment.from, payment.to],
            control,
        })
    });
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
            head: "Recruit Guerrillas".into(),
            kind: String::new(),
            lines: Vec::new(),
            remedies: Vec::new(),
            no_remedy: None,
            button: choice.supported.then(|| {
                if *quantity == 0 {
                    "Pay 1  ·  Place 0 Guerrillas".into()
                } else {
                    format!("Pay 1  ·  {}", choice.label)
                }
            }),
        }],
        stepper: None,
        cancel: false,
        footer: String::new(),
        rally: Some(Recruitment {
            quantities: choices
                .iter()
                .map(|(n, choice)| (*n, choice.action().into()))
                .collect(),
            selected: *quantity,
            arithmetic,
            exchange,
            recovery,
            preview,
        }),
    })
}

fn control_name(control: Option<Faction>) -> &'static str {
    control.map_or("Uncontrolled", Faction::name)
}

fn quantity_name(quantity: u8) -> String {
    format!("popover:quantity:{quantity}")
}

pub(super) fn focusable(card: &Card, rally: &Recruitment) -> Vec<(String, Action)> {
    let mut stops = vec![(
        PROCEDURE.into(),
        Action::Row(rally.selected_action().into()),
    )];
    if rally.quantities.len() > 1 {
        stops.extend(
            rally
                .quantities
                .iter()
                .map(|(quantity, action)| (quantity_name(*quantity), Action::Row(action.clone()))),
        );
    }
    if card.rows[0].button.is_some() {
        stops.push(("popover:commit".into(), payment_action(card)));
    }
    stops
}

impl Recruitment {
    fn selected_action(&self) -> &str {
        &self
            .quantities
            .iter()
            .find(|(n, _)| *n == self.selected)
            .expect("The shown quantity is offered")
            .1
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
    rally: &Recruitment,
    bold: &Handle<Font>,
    scale: f32,
) {
    use super::super::palette::DESK;
    let mut layout = Layout {
        commands,
        art,
        bold,
        scale,
    };
    // Absolute bounds are relative to their named FigCraft frame. The layout
    // remains fixed between quantities, including when the preview moves no pieces.
    layout.solid(body, [0., 0., WIDTH, 4.], "popover-rule", PRIMARY);
    layout.image(body, [20., 18., 30., 30.], &emblems::image(FACTION));
    layout.text(body, [61., 11., 348., 31.], &card.title, (26., true, INK));
    layout.text(
        body,
        [61., 48., 348., 19.],
        &card.subtitle,
        (16., false, QUIET),
    );

    let row = layout.solid(body, [12., 84., 406., 104.], "rally-recruit-row", DESK);
    layout.control(row, PROCEDURE, Action::Row(rally.selected_action().into()));
    layout.image(
        row,
        [8., 16., 12., 12.],
        "atlas-popover-mark-selectable.png",
    );
    layout.image(
        row,
        [28., 9., 19., 24.],
        "atlas-available-directorio-guerrilla.png",
    );
    layout.text(
        row,
        [53., 8., 340., 22.],
        "Recruit Guerrillas",
        (18., true, INK),
    );
    layout.text(
        row,
        [53., 34., 340., 25.],
        &rally.arithmetic,
        (16., false, QUIET),
    );
    if rally.quantities.len() > 1 {
        // The illustrated three cells use 47 px and an 8 px gap. The largest
        // ordinary Rally ceiling fits here by sharing the same row width.
        let count = rally.quantities.len() as f32;
        let width = ((345. - 8. * (count - 1.)) / count).min(47.);
        for (index, (quantity, action)) in rally.quantities.iter().enumerate() {
            let chosen = *quantity == rally.selected;
            let name = quantity_name(*quantity);
            let cell = layout.solid(
                row,
                [53. + index as f32 * (width + 8.), 61., width, 33.],
                &name,
                if chosen { INK } else { PAPER },
            );
            layout.control(cell, name, Action::Row(action.clone()));
            layout.centered(
                cell,
                [0., 4., width, 24.],
                quantity.to_string(),
                if chosen { PAPER } else { INK },
            );
        }
    }
    for y in [188., 238., 299.] {
        layout.solid(body, [20., y, 390., 1.], "", RULE);
    }
    layout.image(
        body,
        [40., 198., 19., 24.],
        "atlas-available-directorio-base.png",
    );
    layout.text(
        body,
        [65., 197., 343., 22.],
        "Exchange 2 Guerrillas for 1 Base",
        (18., true, QUIET),
    );
    layout.text(
        body,
        [65., 218., 343., 18.],
        rally.exchange,
        (15., false, QUIET),
    );
    layout.text(
        body,
        [65., 247., 343., 22.],
        "Turn all Guerrillas Underground",
        (18., true, QUIET),
    );
    layout.text(
        body,
        [65., 271., 343., 18.],
        rally.recovery,
        (15., false, QUIET),
    );

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

fn draw_payment(layout: &mut Layout, effect: Entity, preview: &Payment, quantity: u8) {
    let change = |[from, to]: [u8; 2]| format!("{from} → {to}");
    if quantity == 0 {
        layout.text(
            effect,
            [0., 35., 390., 23.],
            "No pieces move.",
            (19., true, INK),
        );
        layout.text(
            effect,
            [0., 66., 390., 19.],
            "This counts as Rallying in this space.",
            (16., false, QUIET),
        );
    } else {
        layout.text(effect, [189., 7., 92., 18.], "Here", (15., true, QUIET));
        layout.text(
            effect,
            [280., 7., 110., 18.],
            "Available",
            (15., true, QUIET),
        );
        layout.text(
            effect,
            [0., 37., 186., 20.],
            "Guerrillas",
            (17., false, INK),
        );
        layout.text(
            effect,
            [189., 36., 92., 22.],
            change(preview.here),
            (18., true, INK),
        );
        layout.text(
            effect,
            [280., 36., 110., 22.],
            change(preview.available),
            (18., true, INK),
        );
    }
    layout.text(
        effect,
        [0., 95., 174., 20.],
        "Directorio Resources",
        (17., false, INK),
    );
    layout.text(
        effect,
        [174., 93., 216., 22.],
        change(preview.resources),
        (18., true, INK),
    );
    layout.text(
        effect,
        [0., 124., 390., 22.],
        &preview.control,
        (15., false, QUIET),
    );
}
