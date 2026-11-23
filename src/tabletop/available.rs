//! The Available Forces ledger (#159; spec #157): a second ledger left of the
//! Resources ledger, one row per faction, showing the pieces each can still place
//! (R 1.4.1). Each row is the faction's emblem and its kinds in three fixed columns:
//! Troops or Guerrillas, Police (Government only), and Bases or Casinos, each the
//! kind's piece with its count. A kind with none left keeps its cell and draws its
//! piece dimmed beside "0".
//!
//! The ledger reads the committed position's `available` groups and computes nothing
//! else, so a hovered or pinned preview never changes it.
//!
//! Hovering or focusing a row opens that faction's tag (#160), the accepted top-display
//! tag (`top`): each kind's count of its inventory (`crate::INVENTORY`, the p. 20
//! chart) and its number on the map, then the rule (R 1.4.1). Its frame is `hover tag
//! · Government` in the same FigCraft file; it opens left of both ledgers, below the
//! standings. Placement is the FigCraft
//! frame `16:9 available forces` of `art/atlas/atlas-available-forces.figcraft`,
//! recorded in `atlas-available-forces.layout.json`; the pieces are the VectorCraft
//! exports of `atlas-available-forces.vectorcraft` (AVAILABLE.md).
use super::palette::{INK, QUIET};
use super::*;
use crate::{Kind, Observation, inventory};
use bevy::text::LetterSpacing;

/// Node rectangles of the FigCraft frame, in frame units: the owner's 3840×2160
/// display, where one reference pixel is 1.6875 units. The ledger is anchored to the
/// window's top right, as the main view's `available-forces` area is.
mod fig {
    pub const SCALE: f32 = 1.6875;
    pub const FRAME_WIDTH: f32 = 3840.;
    pub const PLATE: [f32; 4] = [2928.75, 33.75, 494., 320.63];
    pub const CAPTION: [f32; 4] = [2928.75, 66., 494., 29.];
    pub const CAPTION_SIZE: f32 = 24.;
    pub const CAPTION_SPACING: f32 = 3.;
    /// Row 0's emblem top; the rows follow the Resources ledger's rhythm, 31 reference
    /// pixels apart.
    pub const ROW_TOP: f32 = 128.25;
    pub const ROW_STEP: f32 = 31. * SCALE;
    pub const EMBLEM_X: f32 = 2952.75;
    pub const EMBLEM: f32 = 45.56;
    /// Each column's piece: its left edge and its top below the row's.
    pub const CUBE_X: f32 = 3027.25;
    pub const GUERRILLA_X: f32 = 3034.25;
    pub const POLICE_X: f32 = 3157.25;
    pub const BASE_X: f32 = 3280.75;
    pub const BASE_DROP: f32 = 6.;
    /// Each column's count: a fixed box beside its piece, 3.375 above the row.
    pub const COUNT_X: [f32; 3] = [3082.75, 3212.75, 3346.75];
    pub const COUNT_RISE: f32 = 3.375;
    pub const COUNT_W: f32 = 56.;
    pub const COUNT_SIZE: f32 = 43.875;
    /// The frame `zero state`: a kind with none left draws its piece at this opacity.
    pub const ZERO_OPACITY: f32 = 0.4;
    /// Each faction's row as its hover and focus target: the band across the plate
    /// between the rules around it (hidden rectangles `<faction>-row`).
    pub const ROWS: [[f32; 4]; 4] = [
        [2928.75, 125., 494., 52.25],
        [2928.75, 177.25, 494., 52.3],
        [2928.75, 229.55, 494., 52.3],
        [2928.75, 281.85, 494., 72.53],
    ];
    /// Where a faction's tag opens (hidden rectangle `government-tag`): 13.5 (8 at
    /// the reference) left of the plate, its top 13.5 below the standings area's foot.
    pub const TAG_OPEN: [f32; 2] = [2125.5, 209.25];
}

/// A piece's size in frame units: its VectorCraft export is drawn at 3× the reference.
const fn piece(png: [f32; 2]) -> [f32; 2] {
    [png[0] / 3. * fig::SCALE, png[1] / 3. * fig::SCALE]
}
const CUBE: [f32; 2] = piece([77., 81.]);
const GUERRILLA: [f32; 2] = piece([52., 82.]);
const BASE: [f32; 2] = piece([100., 61.]);

/// One filled cell: a kind and how many of it the faction has Available.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Cell {
    pub kind: Kind,
    pub count: u8,
}

/// One faction's row, its cells in column order.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Row {
    pub faction: Faction,
    pub cells: Vec<Cell>,
}

/// The kinds a faction holds in Available Forces (the p. 20 chart), in column order.
fn kinds(faction: Faction) -> &'static [Kind] {
    match faction {
        Faction::Government => &[Kind::Troops, Kind::Police, Kind::Base],
        Faction::M26 | Faction::Directorio => &[Kind::Guerrilla, Kind::Base],
        Faction::Syndicate => &[Kind::Guerrilla, Kind::Casino],
    }
}

/// The ledger's rows: each faction's Available pieces of each kind, whatever their
/// state, from the observation alone.
pub(super) fn rows(view: &Observation) -> Vec<Row> {
    Faction::ALL
        .into_iter()
        .map(|faction| Row {
            faction,
            cells: kinds(faction)
                .iter()
                .map(|&kind| Cell {
                    kind,
                    count: view
                        .available
                        .iter()
                        .filter(|group| group.faction == faction && group.kind == kind)
                        .map(|group| group.count)
                        .sum(),
                })
                .collect(),
        })
        .collect()
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Troops => "troops",
        Kind::Police => "police",
        Kind::Guerrilla => "guerrilla",
        Kind::Base => "base",
        Kind::Casino => "casino",
    }
}

/// A cell's node name in the FigCraft frame, which the seat reports as its part.
fn part(faction: Faction, kind: Kind) -> String {
    format!("{}-{}", slug(faction), kind_name(kind))
}

/// The piece's export: the Government's cubes have no faction in their name.
fn key(faction: Faction, kind: Kind) -> String {
    match kind {
        Kind::Troops | Kind::Police => format!("atlas-available-{}.png", kind_name(kind)),
        _ => format!("atlas-available-{}.png", part(faction, kind)),
    }
}

/// The piece's column, and its rectangle in a row whose emblem's top is `top`.
fn place(kind: Kind, top: f32) -> (usize, [f32; 4]) {
    let (column, x, [w, h], drop) = match kind {
        Kind::Troops => (0, fig::CUBE_X, CUBE, 0.),
        Kind::Guerrilla => (0, fig::GUERRILLA_X, GUERRILLA, 0.),
        Kind::Police => (1, fig::POLICE_X, CUBE, 0.),
        Kind::Base | Kind::Casino => (2, fig::BASE_X, BASE, fig::BASE_DROP),
    };
    (column, [x, top + drop, w, h])
}

/// Drawn rectangles by their part names, in frame units.
type Placed = Vec<(String, [f32; 4])>;

/// Every drawn rectangle: images, then text.
fn layout() -> (Placed, Placed) {
    let mut images = vec![("plate".to_string(), fig::PLATE)];
    let mut text = vec![("caption".to_string(), fig::CAPTION)];
    for faction in Faction::ALL {
        let top = fig::ROW_TOP + fig::ROW_STEP * faction.index() as f32;
        images.push((
            format!("{}-emblem", slug(faction)),
            [fig::EMBLEM_X, top, fig::EMBLEM, fig::EMBLEM],
        ));
        for &kind in kinds(faction) {
            let (column, rect) = place(kind, top);
            images.push((part(faction, kind), rect));
            text.push((
                format!("{}-count", part(faction, kind)),
                [
                    fig::COUNT_X[column],
                    top - fig::COUNT_RISE,
                    fig::COUNT_W,
                    fig::COUNT_SIZE,
                ],
            ));
        }
    }
    (images, text)
}

/// A frame rectangle in reference pixels, measured as a right-anchored area is: from
/// the right edge of a 1920 reference.
fn reference(rect: [f32; 4]) -> [f32; 4] {
    let [x, y, w, h] = rect.map(|v| v / fig::SCALE);
    [x - (fig::FRAME_WIDTH / fig::SCALE - WIDTH), y, w, h]
}

/// A piece's tint for its count. It only sets the opacity: the piece as exported, or,
/// with none of its kind left, at the frame `zero state`'s opacity.
fn tint(count: u8) -> Color {
    if count == 0 {
        Color::WHITE.with_alpha(fig::ZERO_OPACITY)
    } else {
        Color::WHITE
    }
}

/// A faction's row, its hover and focus target, in reference pixels, anchored right.
pub(super) fn row(faction: Faction) -> [f32; 4] {
    reference(fig::ROWS[faction.index()])
}

/// Where a faction's tag opens, its top left in reference pixels, anchored right.
pub(super) fn opens() -> [f32; 2] {
    let [x, y, _, _] = reference([fig::TAG_OPEN[0], fig::TAG_OPEN[1], 0., 0.]);
    [x, y]
}

/// A kind as the tag's row names it.
fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Troops => "Troops",
        Kind::Police => "Police",
        Kind::Guerrilla => "Guerrillas",
        Kind::Base => "Bases",
        Kind::Casino => "Casinos",
    }
}

/// A faction's tag: its title, a row per kind reading "N of T · M on the map" (its
/// Available count, its inventory and its pieces on the map in any state, a closed
/// Casino included), and the rule, broken into drawn lines.
pub(super) fn tag(
    view: &Observation,
    faction: Faction,
) -> (String, Vec<(String, String)>, Vec<&'static str>) {
    let row = rows(view)
        .into_iter()
        .find(|row| row.faction == faction)
        .expect("every faction has a row");
    let rows = row
        .cells
        .iter()
        .map(|cell| {
            let on_map: u32 = view
                .spaces
                .iter()
                .flat_map(|space| &space.pieces)
                .filter(|group| group.faction == faction && group.kind == cell.kind)
                .map(|group| u32::from(group.count))
                .sum();
            (
                kind_label(cell.kind).to_owned(),
                format!(
                    "{} of {} · {on_map} on the map",
                    cell.count,
                    inventory(faction, cell.kind)
                ),
            )
        })
        .collect();
    (
        format!("Available {} Forces", faction.name()),
        rows,
        vec![
            // R 1.4.1, first bullet.
            "Pieces are placed only from Available Forces.\nWith none of a kind left, a placement is skipped\nand a replacement only removes (R 1.4.1).",
            // R 1.4.1, second bullet.
            "While executing, a faction may return its own\npieces from the map to Available.",
        ],
    )
}

/// The row that has keyboard focus, if one has: its tag opens as hover opens it.
pub(super) fn focused(ui: &Interface) -> Option<top::Item> {
    let name = ui.choice.focus.as_deref()?;
    Faction::ALL
        .into_iter()
        .map(top::Item::Available)
        .find(|item| item.name() == name)
}

/// The rows Tab moves through, after the panel's controls, by their seat names.
pub(super) fn focusable() -> impl Iterator<Item = (String, Action)> {
    Faction::ALL.into_iter().map(|faction| {
        let item = top::Item::Available(faction);
        (item.name().to_owned(), Action::Top(item))
    })
}

/// Names a drawn part of the ledger by its node name in the FigCraft frame.
#[derive(Component)]
pub(super) struct Part(pub String);

/// The rows the ledger shows, kept on its plate for the seat.
#[derive(Component)]
pub(super) struct Shown(pub Vec<Row>);

/// A piece drawn dimmed, because none of its kind is left.
#[derive(Component)]
pub(super) struct Dimmed;

/// Draws the ledger at rest from the committed position.
pub(super) fn draw(commands: &mut Commands, art: &Art, view: &Observation, s: f32, right: Vec2) {
    let rows = rows(view);
    let counts: BTreeMap<String, u8> = rows
        .iter()
        .flat_map(|row| {
            row.cells
                .iter()
                .map(|cell| (part(row.faction, cell.kind), cell.count))
        })
        .collect();
    let (images, text_parts) = layout();
    for (name, rect) in images {
        let key = if name == "plate" {
            "atlas-available-forces-plate.png".to_string()
        } else if let Some(faction) = name.strip_suffix("-emblem") {
            format!("atlas-victory-icon-{faction}.png")
        } else {
            let faction = Faction::ALL
                .into_iter()
                .find(|f| name.starts_with(&format!("{}-", slug(*f))))
                .expect("a cell names its faction");
            let kind = kinds(faction)
                .iter()
                .copied()
                .find(|k| part(faction, *k) == name)
                .expect("a cell names its kind");
            key(faction, kind)
        };
        let z = if name == "plate" { 2 } else { 3 };
        let entity = picture(commands, art, &key, reference(rect), s, right, z);
        commands.entity(entity).insert(Part(name.clone()));
        if let Some(&count) = counts.get(&name) {
            let mut image = ImageNode::new(art.images[&key].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch);
            image.color = tint(count);
            commands.entity(entity).insert(image);
            if count == 0 {
                commands.entity(entity).insert(Dimmed);
            }
        }
        if name == "plate" {
            commands.entity(entity).insert(Shown(rows.clone()));
        }
    }
    for (name, rect) in text_parts {
        let [x, y, w, _] = reference(rect);
        let entity = if name == "caption" {
            let caption = styled(
                commands,
                art,
                "AVAILABLE FORCES",
                &art.bold,
                QUIET,
                [x, y, w, fig::CAPTION_SIZE / fig::SCALE],
                s,
                right,
                3,
            );
            commands.entity(caption).insert((
                TextLayout::justify(Justify::Center),
                LetterSpacing::Px(fig::CAPTION_SPACING / fig::SCALE * s),
            ));
            caption
        } else {
            let cell = name.strip_suffix("-count").expect("a count names its cell");
            let count = text(
                commands,
                art,
                counts[cell].to_string(),
                x,
                y,
                w,
                fig::COUNT_SIZE / fig::SCALE,
                s,
                right,
                3,
            );
            commands.entity(count).insert(TextColor(INK));
            count
        };
        commands.entity(entity).insert(Part(name));
    }
}

/// What the seat reports of the ledger (`table.available_forces`): its rows by
/// faction, kind and count, and each drawn part's bounds in logical window pixels,
/// with a count's text and whether a piece is dimmed.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let rows = world
        .query::<&Shown>()
        .iter(world)
        .next()
        .map(|shown| shown.0.clone());
    let mut parts = world
        .query::<(
            &Part,
            &ComputedNode,
            &UiGlobalTransform,
            Option<&emblems::Plain>,
            Has<Dimmed>,
        )>()
        .iter(world)
        .map(|(part, node, at, text, dimmed)| {
            (
                part.0.clone(),
                serde_json::json!({
                    "part": part.0,
                    "text": text.map(|t| &t.0),
                    "dimmed": dimmed,
                    "bounds": seat::logical_bounds(node, at),
                }),
            )
        })
        .collect::<Vec<_>>();
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    serde_json::json!({
        "rows": rows,
        "parts": parts.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, Group, State};

    fn record() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../art/atlas/atlas-available-forces.layout.json"
        ))
        .unwrap()
    }

    fn counts(rows: &[Row]) -> Vec<Vec<(Kind, u8)>> {
        rows.iter()
            .map(|row| row.cells.iter().map(|c| (c.kind, c.count)).collect())
            .collect()
    }

    /// The prepared scenario's Available Forces, from the accepted tactical handoff
    /// (the p. 20 chart less the pieces it sets up on the map).
    #[test]
    fn the_prepared_scenario_shows_every_faction_s_counts() {
        let rows = rows(&Game::opening().observe());
        assert_eq!(
            rows.iter().map(|r| r.faction).collect::<Vec<_>>(),
            Faction::ALL
        );
        assert_eq!(
            counts(&rows),
            [
                vec![(Kind::Troops, 3), (Kind::Police, 8), (Kind::Base, 2)],
                vec![(Kind::Guerrilla, 12), (Kind::Base, 3)],
                vec![(Kind::Guerrilla, 12), (Kind::Base, 4)],
                vec![(Kind::Guerrilla, 6), (Kind::Casino, 7)],
            ]
        );
    }

    /// A kind with none left keeps its cell with "0", and Guerrillas returned Active
    /// count with those still Underground.
    #[test]
    fn a_kind_with_none_left_keeps_its_cell_and_states_add_up() {
        let mut view = Game::opening().observe();
        view.available.retain(|g| {
            !(g.faction == Faction::Government && g.kind == Kind::Base)
                && !(g.faction == Faction::Syndicate && g.kind == Kind::Guerrilla)
        });
        view.available.push(Group {
            faction: Faction::Syndicate,
            kind: Kind::Guerrilla,
            state: State::Underground,
            count: 0,
        });
        view.available.push(Group {
            faction: Faction::M26,
            kind: Kind::Guerrilla,
            state: State::Active,
            count: 2,
        });
        let rows = rows(&view);
        assert_eq!(
            counts(&rows),
            [
                vec![(Kind::Troops, 3), (Kind::Police, 8), (Kind::Base, 0)],
                vec![(Kind::Guerrilla, 14), (Kind::Base, 3)],
                vec![(Kind::Guerrilla, 12), (Kind::Base, 4)],
                vec![(Kind::Guerrilla, 0), (Kind::Casino, 7)],
            ]
        );
        // Each zero is drawn: its piece at the zero state's opacity, the rest as
        // exported.
        for row in &rows {
            for cell in &row.cells {
                let alpha = tint(cell.count).alpha();
                if cell.count == 0 {
                    assert_eq!(alpha, 0.4, "{:?} {:?}", row.faction, cell.kind);
                } else {
                    assert_eq!(alpha, 1., "{:?} {:?}", row.faction, cell.kind);
                }
            }
        }
        // Only the opacity changes: the piece keeps its exported colours.
        assert_eq!(tint(0).with_alpha(1.), Color::WHITE);
    }

    /// Every drawn part is where the FigCraft frame records it, within its rounding to
    /// two decimals, and the record holds no part the ledger does not draw.
    #[test]
    fn every_part_matches_the_layout_record() {
        let record = record();
        assert_eq!(record["frame"], serde_json::json!([3840, 2160]));
        assert_eq!(record["anchor"], "top-right");
        let (images, text) = layout();
        for (group, drawn) in [("parts", images), ("text", text)] {
            let recorded = record[group].as_object().unwrap();
            assert_eq!(recorded.len(), drawn.len(), "{group}");
            for (name, rect) in drawn {
                let at: Vec<f32> = recorded[&name]
                    .as_array()
                    .unwrap_or_else(|| panic!("{name} is not recorded"))
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                let compared = if group == "text" { 3 } else { 4 };
                for i in 0..compared {
                    assert!(
                        (at[i] - rect[i]).abs() <= 0.01,
                        "{name}: {rect:?} against the record's {at:?}"
                    );
                }
            }
        }
        assert_eq!(
            record["zero"]["opacity"].as_f64().unwrap() as f32,
            fig::ZERO_OPACITY
        );
        // The ledger is the main view's ninth area.
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let area = &main["areas"]["available-forces"];
        assert_eq!(area["anchor"], "top-right");
        let bounds: Vec<f32> = area["bounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        assert_eq!(bounds, fig::PLATE);
    }

    /// Each piece's place has its export's proportions, and every export is admitted.
    #[test]
    fn every_piece_is_an_admitted_export_drawn_at_its_own_proportions() {
        let record = record();
        let admitted = |file: &str| {
            crate::assets::AVAILABLE_FORCES_MANIFEST
                .lines()
                .any(|line| line.ends_with(&format!("art/atlas/{file}")))
        };
        assert!(admitted("atlas-available-forces-plate.png"));
        for faction in Faction::ALL {
            for &kind in kinds(faction) {
                let file = key(faction, kind);
                assert!(admitted(&file), "{file} is not admitted");
                let size = &record["exports"][&file]["size"];
                let png = [
                    size[0].as_f64().unwrap() as f32,
                    size[1].as_f64().unwrap() as f32,
                ];
                let (_, [_, _, w, h]) = place(kind, 0.);
                assert_eq!([w, h], piece(png), "{file}");
            }
        }
    }

    fn tag_rows(view: &Observation, faction: Faction) -> Vec<(String, String)> {
        tag(view, faction).1
    }

    fn pairs(rows: &[(&str, &str)]) -> Vec<(String, String)> {
        rows.iter()
            .map(|(l, v)| ((*l).to_owned(), (*v).to_owned()))
            .collect()
    }

    /// #160: the prepared scenario's tags, from the p. 20 chart and the accepted
    /// tactical handoff (the chart less the pieces it sets up), not from the ledger.
    #[test]
    fn every_faction_s_tag_states_its_counts_and_the_rule() {
        let view = Game::opening().observe();
        assert_eq!(
            tag_rows(&view, Faction::Government),
            pairs(&[
                ("Troops", "3 of 15 · 12 on the map"),
                ("Police", "8 of 15 · 7 on the map"),
                ("Bases", "2 of 2 · 0 on the map"),
            ])
        );
        assert_eq!(
            tag_rows(&view, Faction::M26),
            pairs(&[
                ("Guerrillas", "12 of 15 · 3 on the map"),
                ("Bases", "3 of 4 · 1 on the map"),
            ])
        );
        assert_eq!(
            tag_rows(&view, Faction::Directorio),
            pairs(&[
                ("Guerrillas", "12 of 15 · 3 on the map"),
                ("Bases", "4 of 4 · 0 on the map"),
            ])
        );
        assert_eq!(
            tag_rows(&view, Faction::Syndicate),
            pairs(&[
                ("Guerrillas", "6 of 6 · 0 on the map"),
                ("Casinos", "7 of 10 · 3 on the map"),
            ])
        );
        let titles: Vec<String> = Faction::ALL.map(|f| tag(&view, f).0).into();
        assert_eq!(
            titles,
            [
                "Available Government Forces",
                "Available 26July Forces",
                "Available Directorio Forces",
                "Available Syndicate Forces"
            ]
        );
        assert_eq!(
            tag(&view, Faction::M26).2.join("\n"),
            "Pieces are placed only from Available Forces.\n\
             With none of a kind left, a placement is skipped\n\
             and a replacement only removes (R 1.4.1).\n\
             While executing, a faction may return its own\n\
             pieces from the map to Available."
        );
    }

    /// A closed Casino is still on the map (R 1.4.5, R 3.2.4): closing one changes no
    /// count in the Syndicate's tag.
    #[test]
    fn a_closed_casino_counts_on_the_map() {
        let mut view = Game::opening().observe();
        let casino = view
            .spaces
            .iter_mut()
            .flat_map(|space| &mut space.pieces)
            .find(|group| group.kind == Kind::Casino)
            .unwrap();
        casino.state = State::Closed;
        assert_eq!(
            tag_rows(&view, Faction::Syndicate)[1],
            ("Casinos".into(), "7 of 10 · 3 on the map".into())
        );
    }

    fn rect(value: &serde_json::Value) -> [f32; 4] {
        let v: Vec<f32> = value
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_f64().unwrap() as f32)
            .collect();
        [
            v[0],
            v[1],
            v.get(2).copied().unwrap_or(0.),
            v.get(3).copied().unwrap_or(0.),
        ]
    }

    fn near(a: [f32; 4], b: [f32; 4], what: &str) {
        for i in 0..4 {
            assert!((a[i] - b[i]).abs() <= 0.01, "{what}: {a:?} against {b:?}");
        }
    }

    /// The FigCraft frame `hover tag · Government` is the accepted top-display tag at
    /// 1.6875 units per reference pixel: the runtime draws the ledger's tags with
    /// `top`'s geometry, so the frame must hold that geometry.
    #[test]
    fn the_tag_frame_is_the_top_display_tag() {
        use super::super::top::tag as geometry;
        let record = record();
        let frame = &record["tag"];
        let k = fig::SCALE;
        // Three rows and five note lines: the Government's tag.
        let height = geometry::ROWS_TOP
            + 3. * geometry::ROW_HEIGHT
            + geometry::NOTES_GAP
            + 5. * geometry::NOTE_LINE
            + geometry::BOTTOM;
        let [w, h, _, _] = rect(&frame["frame"]);
        near(
            [w, h, 0., 0.],
            [geometry::WIDTH * k, height * k, 0., 0.],
            "frame",
        );
        near(
            rect(&frame["parts"]["plate"]),
            [0., 0., geometry::WIDTH * k, height * k],
            "plate",
        );
        // The title at 24, 18, 420 wide; the rule at 24, 62, 420 by 1 (top.rs, TOP.md).
        let title = rect(&frame["text"]["title"]);
        near(
            [title[0], title[1], title[2], 0.],
            [24. * k, 18. * k, 420. * k, 0.],
            "title",
        );
        near(
            rect(&frame["parts"]["rule"]),
            [24. * k, 62. * k, 420. * k, k],
            "rule",
        );
        for row in 0..3 {
            let top = (geometry::ROWS_TOP + row as f32 * geometry::ROW_HEIGHT) * k;
            let label = rect(&frame["text"][format!("row-{row}-label")]);
            let value = rect(&frame["text"][format!("row-{row}-value")]);
            near(
                [label[0], label[1], label[2], 0.],
                [24. * k, top, 200. * k, 0.],
                "label",
            );
            near(
                [value[0], value[1], value[2], 0.],
                [230. * k, top, 214. * k, 0.],
                "value",
            );
        }
        let notes = rect(&frame["text"]["notes"]);
        let top = (geometry::ROWS_TOP + 3. * geometry::ROW_HEIGHT + geometry::NOTES_GAP) * k;
        near(
            [notes[0], notes[1], notes[2], 0.],
            [24. * k, top, 420. * k, 0.],
            "notes",
        );
        // Every note line fits one drawn line.
        let view = Game::opening().observe();
        for faction in Faction::ALL {
            for line in tag(&view, faction).2.iter().flat_map(|n| n.lines()) {
                assert!(line.chars().count() <= 50, "{line}");
            }
        }
    }

    /// Each row's target and the tag's place are the record's.
    #[test]
    fn rows_and_the_tag_s_place_match_the_record() {
        let record = record();
        for faction in Faction::ALL {
            near(
                rect(&record["tag"]["rows"][slug(faction)]),
                fig::ROWS[faction.index()],
                slug(faction),
            );
        }
        let open = rect(&record["tag"]["open"]);
        near(
            [open[0], open[1], 0., 0.],
            [fig::TAG_OPEN[0], fig::TAG_OPEN[1], 0., 0.],
            "open",
        );
        // The rows tile the plate below its header rule, top to bottom.
        let mut top = fig::ROWS[0][1];
        for [x, y, w, h] in fig::ROWS {
            assert_eq!([x, w], [fig::PLATE[0], fig::PLATE[2]]);
            assert!((y - top).abs() < 0.01);
            top = y + h;
        }
        assert!((top - (fig::PLATE[1] + fig::PLATE[3])).abs() < 0.01);
        // 8 reference pixels left of the plate, and 8 below the standings area's foot.
        let main: serde_json::Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let [_, sy, _, sh] = rect(&main["areas"]["standings"]["bounds"]);
        let gap = super::super::top::tag::GAP * fig::SCALE;
        let width = super::super::top::tag::WIDTH * fig::SCALE;
        assert!((fig::TAG_OPEN[0] - (fig::PLATE[0] - gap - width)).abs() < 0.01);
        assert!((fig::TAG_OPEN[1] - (sy + sh + gap)).abs() < 0.01);
    }

    /// #160: every faction's tag, where `top` places it, covers no area of the main
    /// view (the two ledgers, the Cash supply, the standings, the decision panel and
    /// the rest) and stays inside the window, at 3:2, 16:9 and 21:9.
    #[test]
    fn the_tag_covers_no_area_and_stays_inside_the_window() {
        let layout = super::super::areas::Layout::read(include_bytes!(
            "../../art/atlas/atlas-main-view.layout.json"
        ))
        .unwrap();
        let view = Game::opening().observe();
        for (width, height) in [
            (1920., 1280.),
            (2160., 1440.),
            (1920., 1080.),
            (3840., 2160.),
            (2520., 1080.),
            (3440., 1440.),
        ] {
            let window = Window {
                resolution: (width as u32, height as u32).into(),
                ..default()
            };
            let s = super::super::reference_pixel(&window);
            let right = Vec2::new(width - WIDTH * s, 0.);
            for faction in Faction::ALL {
                let shown = super::super::top::content(&view, top::Item::Available(faction));
                let [l, t, r, b] = super::super::top::bounds(&shown, s, right);
                let at = format!("{faction:?} at {width}x{height}");
                assert!(l >= 0. && t >= 0. && r <= width && b <= height, "{at}");
                let tag = Rect::new(l, t, r, b);
                for (name, area) in layout.areas(&window) {
                    assert!(
                        tag.intersect(area).is_empty(),
                        "{at}: covers {name} {area:?} with {tag:?}"
                    );
                }
            }
        }
    }

    /// The ledger lies left of the Resources ledger at its top and height, and right of
    /// the decision panel's left edge, which bounds the popovers' clear region.
    #[test]
    fn the_ledger_stands_beside_resources_at_its_top_and_height() {
        let [x, y, w, h] = reference(fig::PLATE);
        // The Resources ledger is 1688, 20, 192×190 (top.rs).
        assert!((y - 20.).abs() < 0.01 && (h - 190.).abs() < 0.01);
        assert!(x + w < 1688.);
        assert!((x - 1380.).abs() < 0.01, "{x}");
    }
}
