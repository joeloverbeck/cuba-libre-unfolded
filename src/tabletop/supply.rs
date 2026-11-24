//! The marker supply (#172; spec #166): the Terror and Sabotage markers not on the
//! map, of the 20 the two kinds share (R 3.3.4), drawn in the top band as selected
//! graphic M1 · Ficha. It is the two-faced marker, a Terror face and a Sabotage face,
//! beside the number available and "of 20".
//!
//! It reads the committed position's `marker_supply` and computes nothing else, so a
//! hovered or pinned preview never changes it. At 0 the number takes `action/primary`
//! and the marker is its dimmed export, as the study draws it.
//!
//! Placement is the main view's layout, `art/atlas/atlas-main-view.figcraft`, as
//! selected layout A · Franja places it: the area `marker-supply`, centred between the
//! outside-influence area and the standings in every window (`areas::Anchor::TopBetween`),
//! with the marker and the two texts inside it. The marker's two exports are VectorCraft
//! graphics from `art/atlas/atlas-marker-supply.vectorcraft` (TOP.md).
use super::palette::{INK, PRIMARY, QUIET};
use super::*;
use crate::{MARKER_SUPPLY, Observation};

/// Node rectangles of the FigCraft frame `16:9 main view`, in frame units: the owner's
/// 3840×2160 display, where one reference pixel is 1.6875 units.
mod fig {
    /// The area `area/marker-supply`, which the parts below follow.
    pub const AREA: [f32; 4] = [964.69, 30.38, 291., 108.];
    /// The marker's export, at rest or dimmed: 54 reference pixels square.
    pub const MARKER: [f32; 4] = [983.6, 38.82, 91.13, 91.13];
    /// The number's left and top, and its type size (`Supply/count`: Lato Regular 38
    /// at the reference).
    pub const COUNT: [f32; 2] = [1093.29, 44.72];
    pub const COUNT_SIZE: f32 = 64.125;
    /// "of 20": its top, which puts it on the number's baseline, and its type size
    /// (`Supply/of`: Lato Regular 14 at the reference). Its left follows the number's
    /// width, `GAP` after it.
    pub const OF_TOP: f32 = 84.89;
    pub const OF_SIZE: f32 = 23.625;
    /// The space between the number and "of 20": 8 reference pixels in the study.
    pub const GAP: f32 = 13.5;
}

/// The marker's export at rest, and dimmed when the supply is empty.
pub(super) const MARKER: &str = "atlas-marker-supply.png";
pub(super) const MARKER_DIMMED: &str = "atlas-marker-supply-dimmed.png";

/// What the graphic shows: the number available, the words after it, and whether the
/// supply is empty, which dims the marker and reddens the number.
#[derive(Component, Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Shown {
    pub available: u8,
    pub of: String,
    pub empty: bool,
}

/// The graphic's display model, from the observation alone.
pub(super) fn shown(view: &Observation) -> Shown {
    let available = view.marker_supply.available;
    Shown {
        available,
        of: format!("of {MARKER_SUPPLY}"),
        empty: available == 0,
    }
}

impl Shown {
    /// The marker's export: dimmed when no marker is left.
    fn marker(&self) -> &'static str {
        if self.empty { MARKER_DIMMED } else { MARKER }
    }

    /// The number's colour: `action/primary` when no marker is left, else `ink`.
    fn ink(&self) -> Color {
        if self.empty { PRIMARY } else { INK }
    }
}

/// Marks the graphic's drawn parts, so the seat reports them as `table.marker_supply`.
/// Their names are `areas::Part`s, as the layout record names its nodes.
#[derive(Component)]
pub(super) struct Drawn;

/// The marker, drawn dimmed because no marker is left.
#[derive(Component)]
pub(super) struct Dimmed;

/// Draws the graphic from the committed position, inside its area wherever the window
/// puts it.
pub(super) fn draw(
    commands: &mut Commands,
    art: &Art,
    view: &Observation,
    layout: &areas::Layout,
    window: &Window,
) {
    let shown = shown(view);
    let area = layout.area("marker-supply", window);
    // One frame unit in this window, and where the frame's origin falls, so that each
    // part keeps its place inside the area.
    let unit = area.width() / fig::AREA[2];
    let origin = area.min - Vec2::new(fig::AREA[0], fig::AREA[1]) * unit;
    let marker = picture(commands, art, shown.marker(), fig::MARKER, unit, origin, 2);
    commands
        .entity(marker)
        .insert((Drawn, areas::Part("marker-supply".into())));
    if shown.empty {
        commands.entity(marker).insert(Dimmed);
    }
    // The number and "of 20" stand in a row, so "of 20" follows the number's width as
    // the study sets it, and keep their own tops, so they share a baseline.
    let [x, y] = fig::COUNT;
    let row = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(origin.x + x * unit),
                top: px(origin.y + y * unit),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                column_gap: px(fig::GAP * unit),
                ..default()
            },
            GlobalZIndex(2),
            Pickable::IGNORE,
        ))
        .id();
    let words = |text: String, size: f32, ink: Color, drop: f32, part: &str| {
        (
            Text::new(text.clone()),
            emblems::Plain(text),
            TextFont {
                font: art.font.clone().into(),
                font_size: (size * unit).into(),
                ..default()
            },
            TextColor(ink),
            Node {
                margin: UiRect::top(px(drop * unit)),
                ..default()
            },
            Pickable::IGNORE,
            Drawn,
            areas::Part(part.into()),
            ChildOf(row),
        )
    };
    commands.spawn(words(
        shown.available.to_string(),
        fig::COUNT_SIZE,
        shown.ink(),
        0.,
        "marker-supply-count",
    ));
    commands.spawn(words(
        shown.of.clone(),
        fig::OF_SIZE,
        QUIET,
        fig::OF_TOP - y,
        "marker-supply-of",
    ));
    commands.entity(row).insert(shown);
}

/// What the seat reports of the graphic (`table.marker_supply`): what it shows, and
/// each drawn part's bounds in logical window pixels, with a text's words and whether
/// the marker is dimmed.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let shown = world.query::<&Shown>().iter(world).next().cloned();
    let mut parts = world
        .query_filtered::<(
            &areas::Part,
            &ComputedNode,
            &UiGlobalTransform,
            Option<&emblems::Plain>,
            Has<Dimmed>,
        ), With<Drawn>>()
        .iter(world)
        .map(|(part, node, at, text, dimmed)| {
            serde_json::json!({
                "part": part.0,
                "text": text.map(|t| &t.0),
                "dimmed": dimmed,
                "bounds": seat::logical_bounds(node, at),
            })
        })
        .collect::<Vec<_>>();
    parts.sort_by_key(|p| p["part"].as_str().unwrap_or_default().to_owned());
    serde_json::json!({
        "shown": shown,
        "parts": parts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, MarkerSupply};

    /// The prepared scenario has no Terror or Sabotage marker on the map (the accepted
    /// tactical handoff), so the graphic shows 20 of 20, at rest.
    #[test]
    fn the_prepared_scenario_shows_20_of_20_at_rest() {
        let shown = shown(&Game::opening().observe());
        assert_eq!(
            shown,
            Shown {
                available: 20,
                of: "of 20".into(),
                empty: false
            }
        );
        assert_eq!(shown.marker(), MARKER);
        assert_eq!(shown.ink(), INK);
    }

    /// Each Terror places a marker from the supply (R 3.3.4): one fewer after each.
    #[test]
    fn each_terror_shows_one_fewer() {
        let mut game = crate::terror_chosen(Game::opening());
        for action in ["transfer", "ask:2:2", "authorize:2"] {
            let view = game.observe();
            game.submit(
                view.owner,
                view.revision,
                &format!("{}:{action}", view.revision),
            )
            .unwrap();
        }
        let mut expected = 20;
        for target in ["santiago", "oriente"] {
            assert_eq!(shown(&game.observe()).available, expected);
            let view = game.observe();
            game.submit(
                view.owner,
                view.revision,
                &format!("{}:terror:{target}", view.revision),
            )
            .unwrap();
            expected -= 1;
        }
        assert_eq!(shown(&game.observe()).available, 18);
    }

    fn record() -> serde_json::Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
    }

    fn rect(value: &serde_json::Value) -> Vec<f32> {
        value
            .as_array()
            .unwrap_or_else(|| panic!("{value} is not recorded"))
            .iter()
            .map(|n| n.as_f64().unwrap() as f32)
            .collect()
    }

    /// Every part is where the main view's layout record places it: the area, the
    /// marker, the number, and "of 20" with the gap after the number that FigCraft
    /// measured for "20".
    #[test]
    fn every_part_matches_the_layout_record() {
        let record = record();
        let near = |a: &[f32], b: &[f32], what: &str| {
            for (x, y) in a.iter().zip(b) {
                assert!((x - y).abs() <= 0.01, "{what}: {a:?} against {b:?}");
            }
        };
        near(
            &rect(&record["areas"]["marker-supply"]["bounds"]),
            &fig::AREA,
            "area",
        );
        near(
            &rect(&record["parts"]["marker-supply"]),
            &fig::MARKER,
            "marker",
        );
        let count = rect(&record["text"]["marker-supply-count"]);
        near(&count[..2], &fig::COUNT, "count");
        let of = rect(&record["text"]["marker-supply-of"]);
        near(
            &[of[0] - (count[0] + count[2]), of[1]],
            &[fig::GAP, fig::OF_TOP],
            "of",
        );
        // The study's sizes, 38 and 14 reference pixels, on the owner's display.
        assert_eq!(
            [fig::COUNT_SIZE, fig::OF_SIZE],
            [38. * 1.6875, 14. * 1.6875]
        );
        // The marker is 54 reference pixels square, its export 3× that.
        assert!((fig::MARKER[2] - 54. * 1.6875).abs() < 0.01);
        // The graphic lies inside its area and, for "20", is centred in it.
        let right = of[0] + of[2];
        let [x, y, w, h] = fig::AREA;
        assert!(fig::MARKER[0] >= x && right <= x + w);
        assert!(fig::MARKER[1] >= y && fig::MARKER[1] + fig::MARKER[3] <= y + h);
        assert!(((fig::MARKER[0] + right) / 2. - (x + w / 2.)).abs() < 0.5);
    }

    /// Both exports are admitted at launch.
    #[test]
    fn both_marker_exports_are_admitted() {
        for file in [MARKER, MARKER_DIMMED] {
            assert!(
                crate::assets::MARKER_SUPPLY_MANIFEST
                    .lines()
                    .any(|line| line.ends_with(&format!("art/atlas/{file}"))),
                "{file} is not admitted"
            );
        }
    }

    /// With all 20 markers on the map no marker is available (R 3.3.4): the number is
    /// 0 in `action/primary` and the marker is its dimmed export, whatever the split
    /// between the two kinds. No position this build reaches places 20 markers, so the
    /// observation is constructed.
    #[test]
    fn with_all_20_on_the_map_the_number_reddens_and_the_marker_dims() {
        let mut view = Game::opening().observe();
        for (terror, sabotage) in [(20, 0), (17, 3)] {
            view.marker_supply = MarkerSupply::left_by(terror, sabotage);
            let shown = shown(&view);
            assert_eq!(
                (shown.available, shown.of.as_str(), shown.empty),
                (0, "of 20", true)
            );
            assert_eq!(shown.marker(), MARKER_DIMMED);
            assert_eq!(shown.ink(), PRIMARY);
        }
        // One marker left is not empty.
        view.marker_supply = MarkerSupply::left_by(16, 3);
        let shown = shown(&view);
        assert_eq!((shown.available, shown.empty), (1, false));
        assert_eq!((shown.marker(), shown.ink()), (MARKER, INK));
    }
}
