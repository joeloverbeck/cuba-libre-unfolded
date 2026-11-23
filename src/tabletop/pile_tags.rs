//! The pile tags (#144; spec #137, "Played pile tag", "Draw deck tag"): hovering a
//! pile's count disc, or focusing it with Tab, opens its tag after the shared dwell.
//! The played pile's tag lists every card in the pile, most recent first; the draw
//! deck's gives its public facts and how soon the next Propaganda card can come.
//!
//! The geometry is the FigCraft frames of `art/atlas/atlas-main-view.figcraft`, built
//! on the accepted hover tag and recorded in `atlas-pile-tags.layout.json`. Every word
//! and number comes from the observation's `played_pile`, `deck` and `piles`; the tags
//! only place them. Opening one changes nothing in the match.
use super::palette::{INK, QUIET};
use super::piles::Pile;
use super::*;
use crate::Observation;
use bevy::text::{LetterSpacing, LineHeight};

/// Node coordinates in the FigCraft frames, relative to a tag's plate, in frame units:
/// the owner's 3840×2160 display, where one reference pixel is 1.6875 units.
mod fig {
    pub const SCALE: f32 = 1.6875;
    /// Where each tag opens on the main view: its left edge on its pile's, and its foot
    /// 16 above the piles' labels, as the card tag's is.
    pub const PLAYED_LEFT: f32 = 110.;
    pub const DECK_LEFT: f32 = 470.;
    pub const FOOT: f32 = 1546.;
    pub const TITLE: [f32; 2] = [60., 34.];
    pub const TITLE_SIZE: f32 = 54.;
    /// The rule's left and top; it ends 40 from the plate's right edge.
    pub const RULE: [f32; 2] = [60., 120.];
    pub const RULE_INSET: f32 = 100.;
    /// The ledger: labels at the left, values at a column of their own.
    pub const LABEL_X: f32 = 60.;
    pub const PLAYED_VALUE_X: f32 = 340.;
    pub const DECK_VALUE_X: f32 = 360.;
    pub const LEDGER_TOP: f32 = 146.;
    pub const LEDGER_LINE: f32 = 47.;
    pub const LEDGER_SIZE: f32 = 34.;
    /// MOST RECENT FIRST.
    pub const LIST_LABEL: [f32; 2] = [60., 270.];
    pub const CAPS_SIZE: f32 = 22.;
    pub const CAPS_SPACING: f32 = 2.;
    /// The played cards: a column holds at most `ROWS` of them, the columns are
    /// balanced, and each is `COLUMN` wide every `COLUMN_STEP`.
    pub const LIST_TOP: f32 = 308.;
    pub const ROW: f32 = 47.;
    pub const ROWS: usize = 18;
    pub const COLUMN: f32 = 600.;
    pub const COLUMN_STEP: f32 = 660.;
    /// The played pile's plate: as wide as its columns and their margins, and no
    /// narrower than sheet's 800.
    pub const PLAYED_SIDES: f32 = 100.;
    pub const PLAYED_MIN: f32 = 800.;
    pub const DECK_WIDTH: f32 = 820.;
    /// The note below: 36 under the list, or at 370 on the deck's tag; the plate ends
    /// 63 under it.
    pub const NOTE_GAP: f32 = 36.;
    pub const DECK_NOTE_TOP: f32 = 370.;
    pub const PLAYED_NOTE_W: f32 = 700.;
    pub const DECK_NOTE_W: f32 = 720.;
    pub const NOTE_SIZE: f32 = 30.;
    pub const NOTE_LINE: f32 = 39.;
    pub const FOOT_PAD: f32 = 63.;
}

fn r(v: f32) -> f32 {
    v / fig::SCALE
}

/// How the played cards are laid out: the number of columns and the rows in each but
/// the last, which holds the rest.
pub(super) fn columns(cards: usize) -> (usize, usize) {
    let columns = cards.div_ceil(fig::ROWS).max(1);
    (columns, cards.div_ceil(columns))
}

/// The played pile's plate width for a number of columns, in frame units.
fn played_width(columns: usize) -> f32 {
    (fig::PLAYED_SIDES + fig::COLUMN_STEP * columns as f32).max(fig::PLAYED_MIN)
}

/// The pile whose tag is open: the hovered or focused one.
pub(super) fn shown_pile(ui: &Interface) -> Option<Pile> {
    match ui.hover.visible() {
        Some(Target::Pile(pile)) => Some(*pile),
        _ => None,
    }
}

/// The pile that has keyboard focus, if one has: its tag opens as hover opens it.
pub(super) fn focused(ui: &Interface) -> Option<Pile> {
    ui.choice
        .focus
        .as_deref()
        .and_then(|name| Pile::ALL.into_iter().find(|pile| pile.name() == name))
}

#[derive(Component)]
pub(super) struct Tag;

/// Names a drawn part of a tag by its node name in the FigCraft frames.
#[derive(Component)]
pub(super) struct Part(pub String);

/// What the open pile tag shows, for the development seat.
#[derive(Component, Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Shown {
    pub pile: Pile,
    pub title: &'static str,
    /// Each ledger line: its label and its value.
    pub ledger: Vec<[String; 2]>,
    /// The played cards, by column, most recent first; empty on the deck's tag.
    pub columns: Vec<Vec<String>>,
    /// The cards set in bold as Propaganda cards.
    pub propaganda: Vec<u8>,
    pub note: String,
    /// The plate in logical window pixels: left, top, right, bottom.
    pub bounds: [f32; 4],
}

fn content(view: &Observation, pile: Pile) -> Shown {
    match pile {
        Pile::Played => {
            let played = &view.played_pile;
            let (_, rows) = columns(played.cards.len());
            Shown {
                pile,
                title: "Played pile",
                ledger: vec![
                    ["Cards".into(), played.words.cards.clone()],
                    ["Propaganda".into(), played.words.propaganda.clone()],
                ],
                columns: played.words.rows.chunks(rows).map(<[_]>::to_vec).collect(),
                propaganda: played
                    .cards
                    .iter()
                    .filter(|c| c.propaganda)
                    .map(|c| c.number)
                    .collect(),
                note: played.words.note.into(),
                bounds: [0.; 4],
            }
        }
        Pile::Deck => {
            let deck = &view.deck.words;
            Shown {
                pile,
                title: "Draw deck",
                ledger: vec![
                    ["Cards".into(), deck.cards.clone()],
                    ["Face down".into(), deck.face_down.clone()],
                    ["Propaganda left".into(), deck.propaganda_left.clone()],
                    ["Next Propaganda".into(), deck.next_propaganda.clone()],
                ],
                columns: vec![],
                propaganda: vec![],
                note: deck.note.clone(),
                bounds: [0.; 4],
            }
        }
    }
}

/// One line of the tag: its face, colour, size and line height in frame units.
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    font: &Handle<Font>,
    content: &str,
    size: f32,
    line: f32,
    ink: Color,
    s: f32,
    node: Node,
    part: String,
) -> Entity {
    commands
        .spawn((
            Tag,
            Part(part),
            Text::new(content),
            TextFont {
                font: font.clone().into(),
                font_size: (r(size) * s).into(),
                ..default()
            },
            LineHeight::Px(r(line) * s),
            TextColor(ink),
            node,
            Pickable::IGNORE,
        ))
        .id()
}

/// Draws the open pile tag, if any, above the piles.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, (With<Tag>, Without<ChildOf>)>,
    mut previous: Local<Option<Shown>>,
) {
    let (s, left) = layout(&window);
    // The tags open above the piles, which lie against the window's bottom.
    let left = left + bottom(&window);
    let view = play.game().observe();
    let shown = play.session.as_ref().and(shown_pile(&ui)).map(|pile| {
        let mut shown = content(&view, pile);
        let x = match pile {
            Pile::Played => fig::PLAYED_LEFT,
            Pile::Deck => fig::DECK_LEFT,
        };
        // The plate's top follows its content; the seat reads it from the drawn plate.
        let foot = left.y + (HEIGHT - r(2160. - fig::FOOT)) * s;
        shown.bounds = [left.x + r(x) * s, 0., 0., foot];
        shown
    });
    if *previous == shown {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    previous.clone_from(&shown);
    let Some(shown) = shown else {
        return;
    };
    let [x, _, _, foot] = shown.bounds;
    let at = |fx: f32, fy: f32, fw: Option<f32>| Node {
        position_type: PositionType::Absolute,
        left: px(r(fx) * s),
        top: px(r(fy) * s),
        width: fw.map_or(Val::Auto, |fw| px(r(fw) * s)),
        ..default()
    };
    let (width, value_x, note_top, note_w) = match shown.pile {
        Pile::Played => {
            let rows = shown.columns.first().map_or(0, Vec::len);
            (
                played_width(shown.columns.len()),
                fig::PLAYED_VALUE_X,
                fig::LIST_TOP + fig::ROW * rows as f32 + fig::NOTE_GAP,
                fig::PLAYED_NOTE_W,
            )
        }
        Pile::Deck => (
            fig::DECK_WIDTH,
            fig::DECK_VALUE_X,
            fig::DECK_NOTE_TOP,
            fig::DECK_NOTE_W,
        ),
    };
    // The plate stands on the tag's foot and grows upward with its note, the only part
    // whose height follows its words: it is the plate's one child in the flow.
    let plate = commands
        .spawn((
            Tag,
            Part("plate".into()),
            shown.clone(),
            ImageNode::new(art.images["atlas-top-tag.png"].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                bottom: px(window.height() - foot),
                width: px(r(width) * s),
                flex_direction: FlexDirection::Column,
                padding: UiRect::bottom(px(r(fig::FOOT_PAD) * s)),
                ..default()
            },
            GlobalZIndex(24),
            Pickable::IGNORE,
        ))
        .id();
    let mut children = vec![];
    let [rx, ry] = fig::RULE;
    children.push(
        commands
            .spawn((
                Tag,
                Part("rule".into()),
                ImageNode::new(art.images["atlas-victory-rule.png"].clone())
                    .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
                Node {
                    height: px(r(2.) * s),
                    ..at(rx, ry, Some(width - fig::RULE_INSET))
                },
                Pickable::IGNORE,
            ))
            .id(),
    );
    let [tx, ty] = fig::TITLE;
    children.push(words(
        &mut commands,
        &art.font,
        shown.title,
        fig::TITLE_SIZE,
        fig::TITLE_SIZE * 1.2,
        INK,
        s,
        at(tx, ty, None),
        "title".into(),
    ));
    for (i, [label, value]) in shown.ledger.iter().enumerate() {
        let y = fig::LEDGER_TOP + fig::LEDGER_LINE * i as f32;
        let part = ledger_part(shown.pile, i);
        children.push(words(
            &mut commands,
            &art.font,
            label,
            fig::LEDGER_SIZE,
            fig::LEDGER_SIZE * 1.2,
            INK,
            s,
            at(fig::LABEL_X, y, None),
            format!("label-{part}"),
        ));
        children.push(words(
            &mut commands,
            &art.font,
            value,
            fig::LEDGER_SIZE,
            fig::LEDGER_SIZE * 1.2,
            QUIET,
            s,
            at(value_x, y, None),
            part.into(),
        ));
    }
    if shown.pile == Pile::Played {
        let [lx, ly] = fig::LIST_LABEL;
        let caps = words(
            &mut commands,
            &art.bold,
            "MOST RECENT FIRST",
            fig::CAPS_SIZE,
            fig::CAPS_SIZE * 1.2,
            QUIET,
            s,
            at(lx, ly, None),
            "label-list".into(),
        );
        commands
            .entity(caps)
            .insert(LetterSpacing::Px(r(fig::CAPS_SPACING) * s));
        children.push(caps);
        let mut row_index = 0;
        for (column, rows) in shown.columns.iter().enumerate() {
            for (row, line) in rows.iter().enumerate() {
                let card = &view.played_pile.cards[row_index];
                row_index += 1;
                children.push(words(
                    &mut commands,
                    if card.propaganda {
                        &art.bold
                    } else {
                        &art.font
                    },
                    line,
                    fig::LEDGER_SIZE,
                    fig::ROW,
                    INK,
                    s,
                    at(
                        fig::LABEL_X + fig::COLUMN_STEP * column as f32,
                        fig::LIST_TOP + fig::ROW * row as f32,
                        Some(fig::COLUMN),
                    ),
                    format!("column-{column}-row-{row}"),
                ));
            }
        }
    }
    // The note flows: it sets the plate's height.
    children.push(words(
        &mut commands,
        &art.font,
        &shown.note,
        fig::NOTE_SIZE,
        fig::NOTE_LINE,
        QUIET,
        s,
        Node {
            margin: UiRect {
                left: px(r(fig::LABEL_X) * s),
                top: px(r(note_top) * s),
                ..default()
            },
            width: px(r(note_w) * s),
            ..default()
        },
        "note".into(),
    ));
    commands.entity(plate).add_children(&children);
}

/// The part name of a ledger value, as the FigCraft frames name it.
fn ledger_part(pile: Pile, line: usize) -> &'static str {
    match (pile, line) {
        (Pile::Played, 0) => "cards",
        (Pile::Played, _) => "propaganda",
        (Pile::Deck, 0) => "cards",
        (Pile::Deck, 1) => "face-down",
        (Pile::Deck, 2) => "propaganda-left",
        (Pile::Deck, _) => "next-propaganda",
    }
}

/// The open pile tag and each of its drawn parts, in logical window pixels, for the
/// seat (`table.pile_tag`).
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let plate = world
        .query::<(&Shown, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .next()
        .map(|(shown, node, at)| {
            let mut shown = shown.clone();
            shown.bounds = seat::logical_bounds(node, at);
            shown
        });
    let mut parts = world
        .query_filtered::<(&Part, &ComputedNode, &UiGlobalTransform, Option<&Text>), With<Tag>>()
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
        "shown": plate,
        "parts": parts.into_iter().map(|(_, part)| part).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> serde_json::Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-pile-tags.layout.json")).unwrap()
    }
    fn rect(value: &serde_json::Value) -> [f32; 4] {
        std::array::from_fn(|i| value[i].as_f64().unwrap() as f32)
    }

    #[test]
    fn the_columns_hold_every_card_and_at_most_18_each() {
        // 18 on card 17 fit one column; 19 after the advance balance into 10 and 9; 51
        // late in a game into three of 17; 52 into 18, 18 and 16.
        for (cards, expected) in [
            (1, (1, 1)),
            (18, (1, 18)),
            (19, (2, 10)),
            (36, (2, 18)),
            (37, (3, 13)),
            (51, (3, 17)),
            (52, (3, 18)),
        ] {
            let (columns, rows) = columns(cards);
            assert_eq!((columns, rows), expected, "{cards}");
            assert!(columns * rows >= cards && rows <= fig::ROWS);
        }
    }

    #[test]
    fn every_frame_matches_the_layout_s_rule() {
        let record = record();
        for (name, cards) in [
            ("played-card-17", 18),
            ("played-card-18", 19),
            ("played-late", 51),
        ] {
            let frame = &record["frames"][name];
            let (columns, rows) = columns(cards);
            let [w, h] = [
                frame["size"][0].as_f64().unwrap() as f32,
                frame["size"][1].as_f64().unwrap() as f32,
            ];
            assert_eq!(w, played_width(columns), "{name}");
            let parts = &frame["parts"];
            let note = rect(&parts["note"]);
            assert_eq!(
                note[1],
                fig::LIST_TOP + fig::ROW * rows as f32 + fig::NOTE_GAP,
                "{name}"
            );
            assert_eq!(h, note[1] + note[3] + fig::FOOT_PAD, "{name}");
            assert_eq!(
                rect(&parts["rule"]),
                [fig::RULE[0], fig::RULE[1], w - fig::RULE_INSET, 2.]
            );
            for column in 0..columns {
                let at = rect(&parts[format!("column-{column}").as_str()]);
                assert_eq!(
                    at[0],
                    fig::LABEL_X + fig::COLUMN_STEP * column as f32,
                    "{name}"
                );
                assert_eq!([at[1], at[2]], [fig::LIST_TOP, fig::COLUMN], "{name}");
            }
            assert_eq!(rect(&parts["cards"])[0], fig::PLAYED_VALUE_X);
        }
        let deck = &record["frames"]["deck-card-17"];
        assert_eq!(deck["size"][0].as_f64().unwrap() as f32, fig::DECK_WIDTH);
        let note = rect(&deck["parts"]["note"]);
        assert_eq!(
            [note[0], note[1], note[2]],
            [fig::LABEL_X, fig::DECK_NOTE_TOP, fig::DECK_NOTE_W]
        );
        assert_eq!(
            deck["size"][1].as_f64().unwrap() as f32,
            note[1] + note[3] + fig::FOOT_PAD
        );
        for (i, part) in ["cards", "face-down", "propaganda-left", "next-propaganda"]
            .into_iter()
            .enumerate()
        {
            assert_eq!(ledger_part(Pile::Deck, i), part);
            let at = rect(&deck["parts"][part]);
            assert_eq!(
                [at[0], at[1]],
                [
                    fig::DECK_VALUE_X,
                    fig::LEDGER_TOP + fig::LEDGER_LINE * i as f32
                ]
            );
        }
    }

    #[test]
    fn the_tags_open_above_their_piles_where_the_layout_records_it() {
        let record = record();
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        for (name, left, frame) in [
            ("played-pile-tag", fig::PLAYED_LEFT, "played-card-17"),
            ("deck-pile-tag", fig::DECK_LEFT, "deck-card-17"),
        ] {
            let [x, y, w, h] = rect(&record["open"][name]);
            assert_eq!(x, left, "{name}");
            assert_eq!(y + h, fig::FOOT, "{name}");
            let size = &record["frames"][frame]["size"];
            assert_eq!(
                [w, h],
                [
                    size[0].as_f64().unwrap() as f32,
                    size[1].as_f64().unwrap() as f32
                ]
            );
            // Above both piles and their labels, which stay visible beneath it.
            for pile in ["played-pile", "draw-deck"] {
                assert!(y + h <= rect(&main["tray"][pile])[1], "{name}");
            }
            for label in ["played-label", "deck-label"] {
                assert!(y + h <= rect(&main["text"][label])[1], "{name}");
            }
        }
        // The largest tag, 18 rows in three columns, fits inside the reference window,
        // so every tag stays inside every window.
        let top = fig::FOOT
            - (fig::LIST_TOP + fig::ROW * fig::ROWS as f32 + fig::NOTE_GAP + 117. + fig::FOOT_PAD);
        assert!(top >= 2160. - HEIGHT * fig::SCALE, "{top}");
        assert!(fig::PLAYED_LEFT + played_width(3) <= WIDTH * fig::SCALE);
        assert!(fig::DECK_LEFT + fig::DECK_WIDTH <= WIDTH * fig::SCALE);
    }

    #[test]
    fn the_tags_state_the_model_s_values() {
        let view = crate::Game::opening().observe();
        let played = content(&view, Pile::Played);
        assert_eq!(played.ledger[0][1], "18 · current card on top");
        assert_eq!(played.ledger[1][1], "1 of 4 played");
        assert_eq!(played.columns.len(), 1);
        assert_eq!(played.columns[0], view.played_pile.words.rows);
        assert_eq!(played.propaganda, [49]);
        let deck = content(&view, Pile::Deck);
        assert_eq!(
            deck.ledger[3],
            [
                "Next Propaganda".to_string(),
                "within the next 8 cards".into()
            ]
        );
        assert_eq!(deck.note, view.deck.words.note);
    }

    #[test]
    fn focus_on_a_pile_names_it() {
        let mut ui = Interface::default();
        ui.choice.focus = Some("pile:deck".into());
        assert_eq!(focused(&ui), Some(Pile::Deck));
        ui.choice.focus = Some("card:preview".into());
        assert_eq!(focused(&ui), None);
    }
}
