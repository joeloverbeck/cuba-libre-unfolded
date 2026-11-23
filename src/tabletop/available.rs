//! The Available Forces ledger (#159; spec #157): a second ledger left of the
//! Resources ledger, one row per faction, showing the pieces each can still place
//! (R 1.4.1). Each row is the faction's emblem and its kinds in three fixed columns:
//! Troops or Guerrillas, Police (Government only), and Bases or Casinos, each the
//! kind's piece with its count. A kind with none left keeps its cell and draws its
//! piece dimmed beside "0".
//!
//! The ledger reads the committed position's `available` groups and computes nothing
//! else, so a hovered or pinned preview never changes it. Placement is the FigCraft
//! frame `16:9 available forces` of `art/atlas/atlas-available-forces.figcraft`,
//! recorded in `atlas-available-forces.layout.json`; the pieces are the VectorCraft
//! exports of `atlas-available-forces.vectorcraft` (AVAILABLE.md).
use super::palette::{INK, QUIET};
use super::*;
use crate::{Kind, Observation};
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

/// Every drawn rectangle by its part name, in frame units: images, then text.
fn layout() -> (Vec<(String, [f32; 4])>, Vec<(String, [f32; 4])>) {
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
        assert_eq!(tint(0).to_srgba().red, 1.);
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
