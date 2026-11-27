//! The main view's layout, `art/atlas/atlas-main-view.figcraft` (MAIN-VIEW.md).
//!
//! The map fills the window and every interface area lies over it. The layout's
//! record gives each area's rectangle on the owner's 3840×2160 display, the edges of
//! the window it keeps its distance from, and the box the island fills at Home. This
//! module turns the record into window rectangles, draws the transparent rectangle
//! that makes an area take the pointer, and frames Home.
use super::*;
use bevy::mesh::VertexAttributeValues;
use unfolded_tabletop::camera::Pose;

/// Which edges of the window a rectangle keeps its distance from. Its size follows
/// the window's scale alone (ADR 0002), except for [`Anchor::Edges`], whose corners
/// each follow their own corner of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Anchor {
    TopLeft,
    /// The top edge and the window's horizontal centre.
    Top,
    TopRight,
    /// The top edge, and centred between the right edge of one area and the left edge
    /// of another, which `between` names (#172: the marker supply, between the
    /// outside-influence area and the standings).
    TopBetween,
    /// The bottom edge and the left edge of the second area `between` names, shrunk
    /// toward its own bottom right corner where it would come within
    /// [`BETWEEN_GAP`] of the first area's right edge (#206: the seat cards, between the
    /// Propaganda pips and Save · quit, in a window narrower than the display's shape).
    BottomBetween,
    /// The bottom and the left edge, beside the first area `between` names, where that
    /// keeps [`BETWEEN_GAP`] from the second; else as far above the first, on its left
    /// edge (#208: the pile of editions, beside the Propaganda pips where the seat cards
    /// leave room, and above the pips in a window narrower than the display's shape).
    BottomBeside,
    BottomLeft,
    BottomRight,
    Edges,
}

/// How near an [`Anchor::BottomBetween`] or [`Anchor::BottomBeside`] area comes to its
/// neighbour, in frame units.
const BETWEEN_GAP: f32 = 24.;

#[derive(Clone, Debug, serde::Deserialize)]
pub(super) struct Placed {
    /// x, y, width and height in the layout's frame.
    pub bounds: [f32; 4],
    pub anchor: Anchor,
    /// For [`Anchor::TopBetween`], [`Anchor::BottomBetween`] and [`Anchor::BottomBeside`],
    /// the areas to its left and its right.
    #[serde(default)]
    pub between: Option<[String; 2]>,
}

/// The layout record, `atlas-main-view.layout.json`, admitted at launch with the
/// other presentation art.
#[derive(Resource, Clone, Debug, serde::Deserialize)]
pub(super) struct Layout {
    frame: [f32; 2],
    island: Placed,
    areas: BTreeMap<String, Placed>,
    /// The notice slip (#174), drawn with two lines: its right edge, its foot and
    /// its width. It grows upward with its lines.
    #[serde(rename = "notice-slip")]
    slip: Placed,
    /// Where Save · quit's hover tag opens (#174), with its two note lines: its right
    /// edge and its foot. Its height follows its lines.
    #[serde(rename = "save-quit-tag")]
    save_quit_tag: Placed,
}

impl Layout {
    pub(super) fn read(bytes: &[u8]) -> Result<Self, String> {
        let layout: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Main view layout record: {e}"))?;
        // `clear` is measured between these three.
        for name in ["standings", "decision", "cards"] {
            if !layout.areas.contains_key(name) {
                return Err(format!("Main view layout record has no {name} area"));
            }
        }
        // An area placed between two others names two areas that are placed by their
        // own edges.
        for (name, placed) in &layout.areas {
            match (placed.anchor, &placed.between) {
                (Anchor::TopBetween | Anchor::BottomBetween, Some(pair)) => {
                    for other in pair {
                        let fixed = layout.areas.get(other).is_some_and(|o| {
                            !matches!(o.anchor, Anchor::TopBetween | Anchor::BottomBetween)
                        });
                        if !fixed {
                            return Err(format!(
                                "Main view layout record: {name} lies between {other}, which is not an area placed by its edges"
                            ));
                        }
                    }
                }
                // Beside an area placed by its own edges, clear of any other.
                (Anchor::BottomBeside, Some([beside, other])) => {
                    let placed = |name: &String| layout.areas.get(name).map(|o| o.anchor);
                    let moves = |anchor: Anchor| {
                        matches!(
                            anchor,
                            Anchor::TopBetween | Anchor::BottomBetween | Anchor::BottomBeside
                        )
                    };
                    if placed(beside).is_none_or(moves)
                        || placed(other).is_none_or(|o| o == Anchor::BottomBeside)
                    {
                        return Err(format!(
                            "Main view layout record: {name} cannot lie beside {beside} and clear of {other}"
                        ));
                    }
                }
                (Anchor::TopBetween | Anchor::BottomBetween | Anchor::BottomBeside, None) => {
                    return Err(format!(
                        "Main view layout record: {name} names no areas it lies between"
                    ));
                }
                (_, Some(_)) => {
                    return Err(format!(
                        "Main view layout record: {name} names areas but is not placed between them"
                    ));
                }
                (_, None) => {}
            }
        }
        Ok(layout)
    }

    /// A placed rectangle in logical window pixels. One frame unit is the same length
    /// in every window of a given scale: the frame is as tall as the reference.
    pub(super) fn place(&self, placed: &Placed, window: &Window) -> Rect {
        let unit = reference_pixel(window) * HEIGHT / self.frame[1];
        let size = Vec2::new(window.width(), window.height());
        let frame = Vec2::from(self.frame);
        let [x, y, w, h] = placed.bounds;
        let (min, max) = (Vec2::new(x, y), Vec2::new(x + w, y + h));
        // A point's place measured from the window's left or top, centre, or right
        // or bottom.
        let from_start = |p: Vec2| p * unit;
        let from_end = |p: Vec2| size - (frame - p) * unit;
        let from_centre = |p: Vec2| size / 2. + (p - frame / 2.) * unit;
        let (min, max) = match placed.anchor {
            Anchor::TopLeft => (from_start(min), from_start(max)),
            Anchor::Top => (
                Vec2::new(from_centre(min).x, from_start(min).y),
                Vec2::new(from_centre(max).x, from_start(max).y),
            ),
            Anchor::TopRight => (
                Vec2::new(from_end(min).x, from_start(min).y),
                Vec2::new(from_end(max).x, from_start(max).y),
            ),
            Anchor::BottomLeft => (
                Vec2::new(from_start(min).x, from_end(min).y),
                Vec2::new(from_start(max).x, from_end(max).y),
            ),
            Anchor::BottomRight => (from_end(min), from_end(max)),
            Anchor::Edges => (from_start(min), from_end(max)),
            Anchor::TopBetween => {
                let [left, right] = self.neighbours(placed);
                let middle =
                    (self.place(left, window).max.x + self.place(right, window).min.x) / 2.;
                let half = w * unit / 2.;
                (
                    Vec2::new(middle - half, from_start(min).y),
                    Vec2::new(middle + half, from_start(max).y),
                )
            }
            Anchor::BottomBetween => {
                let [left, right] = self.neighbours(placed);
                let clear = right.bounds[0] - (x + w);
                let corner = Vec2::new(
                    self.place(right, window).min.x - clear * unit,
                    from_end(max).y,
                );
                let room = corner.x - self.place(left, window).max.x - BETWEEN_GAP * unit;
                let fits = (room / (w * unit)).clamp(0., 1.);
                (corner - Vec2::new(w, h) * unit * fits, corner)
            }
            Anchor::BottomBeside => {
                let [beside, other] = self.neighbours(placed);
                let size = Vec2::new(w, h) * unit;
                let gap = BETWEEN_GAP * unit;
                let here = Vec2::new(from_start(min).x, from_end(min).y);
                let corner = if here.x + size.x + gap <= self.place(other, window).min.x {
                    here
                } else {
                    let beside = self.place(beside, window).min;
                    Vec2::new(beside.x, beside.y - gap - size.y)
                };
                (corner, corner + size)
            }
        };
        Rect { min, max }
    }

    /// The areas a between area lies between. `read` checked that both exist and are
    /// placed by edges.
    fn neighbours(&self, placed: &Placed) -> [&Placed; 2] {
        placed
            .between
            .as_ref()
            .map(|[l, r]| [&self.areas[l], &self.areas[r]])
            .expect("read checks that a between area names its neighbours")
    }

    /// Every area that lies over the map, by its name in the layout.
    pub(super) fn areas<'a>(
        &'a self,
        window: &'a Window,
    ) -> impl Iterator<Item = (&'a str, Rect)> + 'a {
        self.areas
            .iter()
            .map(|(name, placed)| (name.as_str(), self.place(placed, window)))
    }

    pub(super) fn area(&self, name: &str, window: &Window) -> Rect {
        self.place(&self.areas[name], window)
    }

    /// The notice slip's place in this window. Its right edge, its foot and its width
    /// are the slip's; it grows upward from its foot.
    pub(super) fn slip(&self, window: &Window) -> Rect {
        self.place(&self.slip, window)
    }

    /// Where Save · quit's hover tag opens in this window. Its right edge and foot are
    /// the tag's; its height follows its lines.
    pub(super) fn save_quit_tag(&self, window: &Window) -> Rect {
        self.place(&self.save_quit_tag, window)
    }

    /// The box the thirteen spaces fill at Home.
    pub(super) fn island(&self, window: &Window) -> Rect {
        self.place(&self.island, window)
    }

    /// The part of the window between the areas at its edges, where space and piece
    /// popovers open: below the standings, left of the decision panel, above the
    /// cards, and inside the left margin the areas keep.
    pub(super) fn clear(&self, window: &Window) -> Rect {
        let cards = self.area("cards", window);
        Rect {
            min: Vec2::new(cards.min.x, self.area("standings", window).max.y),
            max: Vec2::new(self.area("decision", window).min.x, cards.min.y),
        }
    }
}

/// An area's transparent rectangle. It is an ordinary pickable node, so a press or
/// a scroll on the area stops there: the shared camera counts the pointer as over
/// the interface, and no space or piece beneath is picked. The area's own controls
/// lie above it. The seat names it `area:<name>`.
#[derive(Component)]
pub(super) struct Area(pub String);

/// Where the seat may rest the pointer on an area whose middle its own controls
/// cover: the middle, then just inside each edge and corner.
pub(super) fn bare_points(centre: Vec2, size: Vec2) -> Vec<Vec2> {
    let reach = (size / 2. - Vec2::splat(4.)).max(Vec2::ZERO);
    [
        (0., 0.),
        (-1., 0.),
        (1., 0.),
        (0., -1.),
        (0., 1.),
        (-1., -1.),
        (1., -1.),
        (-1., 1.),
        (1., 1.),
    ]
    .into_iter()
    .map(|(x, y)| centre + reach * Vec2::new(x, y))
    .collect()
}

/// Names a drawn part by its node name in `atlas-main-view.figcraft`, so the
/// development seat can compare native placement with the layout.
#[derive(Component)]
pub(super) struct Part(pub String);

/// The thirteen spaces' mesh vertices in the world, read once the map is loaded.
#[derive(Resource, Default)]
pub(super) struct Island {
    spaces: BTreeMap<String, Vec<Vec3>>,
}

impl Island {
    pub(super) fn surveyed(&self) -> bool {
        !self.spaces.is_empty()
    }
}

/// Draws each area's rectangle. The editions' is there with the pile (`editions`).
pub(super) fn draw(commands: &mut Commands, layout: &Layout, window: &Window, editions: bool) {
    for (name, rect) in layout.areas(window) {
        if name == newspaper::AREA && !editions {
            continue;
        }
        commands.spawn((
            UiRoot,
            Area(name.into()),
            Control {
                name: format!("area:{name}"),
                choice: false,
                action: Action::Area,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(rect.min.x),
                top: px(rect.min.y),
                width: px(rect.width()),
                height: px(rect.height()),
                ..default()
            },
            // Above the piece counts, below everything the areas draw.
            GlobalZIndex(0),
        ));
    }
}

/// Bevy's default perspective camera, which the table camera is: 45° top to bottom.
const FIELD_OF_VIEW: f32 = std::f32::consts::FRAC_PI_4;

/// Where a world point falls in a window seen from a framing's Home pose, in logical
/// pixels. `None` behind the camera.
fn project(point: Vec3, framing: Framing, window: Vec2) -> Option<Vec2> {
    let camera = Pose::at(framing).transform();
    let local = camera.rotation.inverse() * (point - camera.translation);
    let depth = -local.z;
    if depth <= 0. {
        return None;
    }
    let half = (FIELD_OF_VIEW / 2.).tan() * depth;
    let ndc = Vec2::new(local.x / (half * window.x / window.y), local.y / half);
    Some(Vec2::new(
        (ndc.x + 1.) / 2. * window.x,
        (1. - ndc.y) / 2. * window.y,
    ))
}

fn projected(points: &[Vec3], framing: Framing, window: Vec2) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for point in points {
        let p = project(*point, framing, window)?;
        bounds = super::grow(bounds, p);
    }
    bounds
}

/// The Home framing that shows `points` as large as `into` allows, centred in it.
fn fit(points: &[Vec3], window: Vec2, into: Rect) -> Option<Framing> {
    if points.is_empty() || into.width() <= 0. || into.height() <= 0. {
        return None;
    }
    let centre = points.iter().sum::<Vec3>() / points.len() as f32;
    let mut framing = Framing::new(Vec3::new(centre.x, 0., centre.z), 10.);
    let tan = (FIELD_OF_VIEW / 2.).tan();
    // Perspective makes neither step exact, so repeat them: each pass leaves a
    // fraction of the last one's error.
    for _ in 0..32 {
        let seen = projected(points, framing, window)?;
        framing.distance *= (seen.width() / into.width()).max(seen.height() / into.height());
        let seen = projected(points, framing, window)?;
        // Moving the target left moves the picture right; moving it away from the
        // camera moves the picture down, foreshortened by the pitch.
        let shift = (into.center() - seen.center()) * 2. * framing.distance * tan / window.y;
        framing.target.x -= shift.x;
        framing.target.z -= shift.y / Pose::DEFAULT_PITCH.sin();
    }
    Some(framing)
}

/// Home shows all thirteen spaces inside the layout's island box, whatever the
/// window's shape. A camera resting at Home follows a change of window; one the
/// player has moved stays where it is, and Home returns it.
#[allow(clippy::too_many_arguments)]
pub(super) fn home(
    window: Single<&Window, With<PrimaryWindow>>,
    play: Res<Play>,
    art: Res<Art>,
    layout: Res<Layout>,
    mut island: ResMut<Island>,
    mut orbit: ResMut<Orbit>,
    meshes: Query<(Entity, &Mesh3d, &GlobalTransform)>,
    mesh_assets: Res<Assets<Mesh>>,
    controls: Query<&Control>,
    parents: Query<&ChildOf>,
    mut counts: ResMut<Counts>,
    mut framed: Local<Option<Vec2>>,
) {
    // The session opens only after the map has loaded and been drawn, so every
    // region's place in the world is settled by then.
    if !play.admitted() {
        return;
    }
    if !island.surveyed() {
        let mut spaces = BTreeMap::<String, Vec<Vec3>>::new();
        for (entity, mesh, at) in &meshes {
            let mut current = Some(entity);
            let space = loop {
                let Some(e) = current else { break None };
                if let Ok(Control {
                    action: Action::Space(space),
                    ..
                }) = controls.get(e)
                {
                    break Some(space.clone());
                }
                current = parents.get(e).ok().map(ChildOf::parent);
            };
            let (Some(space), Some(VertexAttributeValues::Float32x3(vertices))) = (
                space,
                mesh_assets
                    .get(&mesh.0)
                    .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)),
            ) else {
                continue;
            };
            spaces.entry(space).or_default().extend(
                vertices
                    .iter()
                    .map(|v| at.transform_point(Vec3::from_array(*v))),
            );
        }
        if spaces.len() < art.labels.len() {
            return;
        }
        island.spaces = spaces;
    }
    let size = Vec2::new(window.width(), window.height());
    if *framed == Some(size) {
        return;
    }
    let points: Vec<Vec3> = island.spaces.values().flatten().copied().collect();
    let Some(framing) = fit(&points, size, layout.island(&window)) else {
        return;
    };
    let resting = orbit.pose() == Pose::at(orbit.home());
    orbit.set_home(framing);
    if resting {
        orbit.focus(framing);
        // The counts follow the camera's jump over the next frames.
        counts.unsettle();
    }
    *framed = Some(size);
}

/// What the seat reports of the main view (`table.main_view`): each area's rectangle,
/// each named part's bounds, and each space's bounds as the map camera shows it. All
/// are [left, top, right, bottom] in logical window pixels.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let mut parts = world
        .query::<(
            &Part,
            &ComputedNode,
            &UiGlobalTransform,
            Option<&emblems::Plain>,
        )>()
        .iter(world)
        .map(|(part, node, at, text)| {
            (
                part.0.clone(),
                serde_json::json!({
                    "part": part.0,
                    "text": text.map(|t| &t.0),
                    "bounds": seat::logical_bounds(node, at),
                }),
            )
        })
        .collect::<Vec<_>>();
    // The Sequence of Play track's plate, which its own module draws and marks.
    for (node, at) in world
        .query_filtered::<(&ComputedNode, &UiGlobalTransform), With<eligibility::Drawn>>()
        .iter(world)
    {
        parts.push((
            "sequence-plate".into(),
            serde_json::json!({
                "part": "sequence-plate",
                "text": null,
                "bounds": seat::logical_bounds(node, at),
            }),
        ));
    }
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    let areas: BTreeMap<String, [f32; 4]> = world
        .query::<(&Area, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .map(|(area, node, at)| (area.0.clone(), seat::logical_bounds(node, at)))
        .collect();
    let camera = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .single(world)
        .ok()
        .map(|(camera, at)| (camera.clone(), *at));
    let spaces: BTreeMap<String, [f32; 4]> = camera
        .map(|(camera, at)| {
            world
                .resource::<Island>()
                .spaces
                .iter()
                .filter_map(|(space, points)| {
                    let mut bounds: Option<Rect> = None;
                    for point in points {
                        let p = camera.world_to_viewport(&at, *point).ok()?;
                        bounds = super::grow(bounds, p);
                    }
                    let b = bounds?;
                    Some((space.clone(), [b.min.x, b.min.y, b.max.x, b.max.y]))
                })
                .collect()
        })
        .unwrap_or_default();
    let (island, clear) = {
        let layout = world.resource::<Layout>().clone();
        let mut windows = world.query_filtered::<&Window, With<PrimaryWindow>>();
        windows.single(world).ok().map_or((None, None), |window| {
            let array = |r: Rect| [r.min.x, r.min.y, r.max.x, r.max.y];
            (
                Some(array(layout.island(window))),
                Some(array(layout.clear(window))),
            )
        })
    };
    serde_json::json!({
        "areas": areas,
        "island": island,
        "clear": clear,
        "spaces": spaces,
        "parts": parts.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> Layout {
        Layout::read(include_bytes!(
            "../../art/atlas/atlas-main-view.layout.json"
        ))
        .unwrap()
    }
    fn window(width: f32, height: f32) -> Window {
        Window {
            resolution: (width as u32, height as u32).into(),
            ..default()
        }
    }
    fn close(a: Rect, b: [f32; 4]) -> bool {
        let b = Rect::new(b[0], b[1], b[0] + b[2], b[1] + b[3]);
        a.min.abs_diff_eq(b.min, 0.01) && a.max.abs_diff_eq(b.max, 0.01)
    }

    /// On the owner's display an area is where the layout's frame draws it.
    #[test]
    fn the_owners_display_shows_the_frame_as_drawn() {
        let layout = record();
        let window = window(3840., 2160.);
        for placed in layout.areas.values().chain([&layout.island]) {
            assert!(close(layout.place(placed, &window), placed.bounds));
        }
    }

    /// The accepted areas are where the reference rectangles their modules draw at
    /// put them, in every window: left, centre and right as before, and the bottom
    /// row against the bottom edge instead of a letterbox.
    #[test]
    fn areas_follow_their_edges_at_the_reference_scale() {
        let layout = record();
        for (width, height) in [
            (1920., 1280.),
            (1920., 1080.),
            (2520., 1080.),
            (3840., 2160.),
            (1600., 1280.),
        ] {
            let window = window(width, height);
            let s = super::super::reference_pixel(&window);
            let right = width - WIDTH * s;
            let centre = right / 2.;
            let bottom = height - HEIGHT * s;
            for (name, [x, y, w, h], dx, dy) in [
                ("outside-influence", [40., 18., 370., 64.], 0., 0.),
                (
                    "sequence-of-play",
                    [60., 480. / 2.7, 232., 1036.7 / 2.7],
                    0.,
                    0.,
                ),
                ("standings", [728., 8., 464., 108.], centre, 0.),
                ("resources", [1688., 20., 192., 246.], right, 0.),
                // #159: Available Forces, directly left of the Resources ledger.
                (
                    "available-forces",
                    [1380., 20., 494. / 1.6875, 190.],
                    right,
                    0.,
                ),
                ("decision", [1380., 280., 500., 660.], right, 0.),
                // #174: Save · quit, 40 from the right and the bottom edges.
                ("save-quit", [1728., 1198., 152., 42.], right, bottom),
                // #143: the two piles and the Propaganda pips took the card
                // panel's place. These are the layout's own rectangles, in
                // reference pixels from the bottom of a 1280-high reference.
                (
                    "cards",
                    [
                        67.5 / 1.6875,
                        HEIGHT - (2160. - 1594.) / 1.6875,
                        734.5 / 1.6875,
                        536. / 1.6875,
                    ],
                    0.,
                    bottom,
                ),
                (
                    "propaganda",
                    [
                        834. / 1.6875,
                        HEIGHT - (2160. - 1920.) / 1.6875,
                        226. / 1.6875,
                        160. / 1.6875,
                    ],
                    0.,
                    bottom,
                ),
            ] {
                let area = layout.area(name, &window);
                assert!(
                    close(area, [dx + x * s, dy + y * s, w * s, h * s]),
                    "{name} at {width}x{height}: {area:?}"
                );
                assert!(
                    area.min.cmpge(Vec2::ZERO).all()
                        && area.max.cmple(Vec2::new(width, height)).all(),
                    "{name} leaves a {width}x{height} window: {area:?}"
                );
            }
            let island = layout.island(&window);
            assert!(island.width() > 0. && island.height() > 0.);
            for (name, area) in layout.areas(&window) {
                assert!(
                    island.intersect(area).is_empty(),
                    "{name} covers the island box at {width}x{height}"
                );
                // #159: no area overlaps another, whatever the window's shape.
                for (other, rect) in layout.areas(&window) {
                    assert!(
                        name == other || area.intersect(rect).is_empty(),
                        "{name} overlaps {other} at {width}x{height}"
                    );
                }
            }
        }
    }

    /// #172: the marker supply keeps the size the window's scale gives it and its
    /// distance from the top, and is centred between the right edge of the
    /// outside-influence area and the left edge of the standings, which move apart at
    /// different rates as the window widens. On the owner's display that is the frame's
    /// own place.
    #[test]
    fn the_marker_supply_is_centred_between_its_neighbours_in_every_window() {
        let layout = record();
        let placed = &layout.areas["marker-supply"];
        assert_eq!(placed.anchor, Anchor::TopBetween);
        assert_eq!(
            placed.between,
            Some(["outside-influence".to_string(), "standings".to_string()])
        );
        let [x, y, w, h] = placed.bounds;
        let frame_middle = (67.5 + 624.38 + 1528.5) / 2.;
        assert!((x + w / 2. - frame_middle).abs() < 0.01, "{x} {w}");
        for (width, height) in [
            (1920., 1280.),
            (2160., 1440.),
            (1600., 1280.),
            (1920., 1080.),
            (3840., 2160.),
            (2520., 1080.),
            (3440., 1440.),
        ] {
            let window = window(width, height);
            let s = super::super::reference_pixel(&window);
            // In reference pixels: outside influence ends at 410 from the left; the
            // standings begin at 728 of a 1920 reference centred in the window.
            let left = 410. * s;
            let right = (width - WIDTH * s) / 2. + 728. * s;
            let k = s / 1.6875;
            let area = layout.area("marker-supply", &window);
            let at = format!("{width}x{height}: {area:?}");
            assert!((area.center().x - (left + right) / 2.).abs() < 0.01, "{at}");
            assert!((area.min.y - y * k).abs() < 0.01, "{at}");
            assert!((area.width() - w * k).abs() < 0.01, "{at}");
            assert!((area.height() - h * k).abs() < 0.01, "{at}");
            assert!(left < area.min.x && area.max.x < right, "{at}");
        }
    }

    /// #174, layout A · Franja: the notice slip stands above Save · quit, their right edges
    /// aligned, at the space popover's width (340), and Save · quit's hover tag opens above
    /// the plate, its right edge on the window's 40 margin, at the top display tag's width
    /// (468). They stood 14 and 8 reference pixels above the plate; since the owner's ruling
    /// of 2026-11-25 (#206) both are 126 frame units higher. In every window neither covers
    /// the plate, any other area, the seat cards whichever stands forward, or the decision
    /// panel, which the slip clears with nine lines (25 of card and 15 a line).
    #[test]
    fn the_slip_and_the_tag_stand_above_the_seat_cards_and_save_quit() {
        let layout = record();
        for (width, height) in [
            (3840., 2160.),
            (4096., 2160.),
            (3440., 1440.),
            (2560., 1080.),
            (2520., 1080.),
            (2160., 1440.),
            (1920., 1280.),
            (1920., 1080.),
            (1600., 1280.),
            (1440., 960.),
        ] {
            let window = window(width, height);
            let s = super::super::reference_pixel(&window);
            let raised = 126. / 1.6875 * s;
            let plate = layout.area("save-quit", &window);
            let slip = layout.slip(&window);
            let tag = layout.save_quit_tag(&window);
            let at = format!("{width}x{height}");
            assert!((slip.max.x - plate.max.x).abs() < 0.02, "{at}");
            assert!(
                (plate.min.y - slip.max.y - 14. * s - raised).abs() < 0.02,
                "{at}"
            );
            assert!((slip.width() - 340. * s).abs() < 0.02, "{at}");
            assert!((tag.max.x - (width - 40. * s)).abs() < 0.02, "{at}");
            assert!(
                (plate.min.y - tag.max.y - 8. * s - raised).abs() < 0.02,
                "{at}"
            );
            assert!((tag.width() - 468. * s).abs() < 0.02, "{at}");
            let decision = layout.area("decision", &window);
            let nine = (25. + 9. * 15.) * s;
            assert!(slip.max.y - nine > decision.max.y, "{at}: room for lines");
            for (name, area) in layout.areas(&window) {
                let slip = Rect {
                    min: Vec2::new(slip.min.x, slip.max.y - nine),
                    ..slip
                };
                for (what, open) in [("slip", slip), ("tag", tag)] {
                    let shared = open.intersect(area);
                    assert!(shared.is_empty(), "{at}: the {what} covers {name}");
                }
            }
        }
    }

    /// #208: the pile of editions keeps its place beside the Propaganda pips wherever the
    /// seat cards leave it 24 units, and stands 24 above the pips, on their left edge,
    /// where they do not.
    #[test]
    fn the_pile_of_editions_stands_beside_the_pips_or_above_them() {
        let layout = record();
        let placed = &layout.areas["editions"];
        assert_eq!(placed.anchor, Anchor::BottomBeside);
        assert_eq!(
            placed.between,
            Some(["propaganda".to_string(), "seats".to_string()])
        );
        let [x, y, w, h] = placed.bounds;
        for (width, height, beside) in [
            (3840., 2160., true),
            (1920., 1080., true),
            (2520., 1080., true),
            (1920., 1280., false),
            (1440., 960., false),
            (1600., 1280., false),
        ] {
            let window = window(width, height);
            let k = super::super::reference_pixel(&window) / 1.6875;
            let pile = layout.area("editions", &window);
            let pips = layout.area("propaganda", &window);
            let seats = layout.area("seats", &window);
            let at = format!("{width}x{height}: {pile:?}");
            assert!((pile.width() - w * k).abs() < 0.01 && (pile.height() - h * k).abs() < 0.01);
            if beside {
                assert!((pile.min.x - x * k).abs() < 0.01, "{at}");
                assert!(
                    (height - pile.max.y - (2160. - y - h) * k).abs() < 0.01,
                    "{at}"
                );
                assert!(seats.min.x - pile.max.x >= 24. * k - 0.01, "{at}");
            } else {
                assert!((pile.min.x - pips.min.x).abs() < 0.01, "{at}");
                assert!((pips.min.y - pile.max.y - 24. * k).abs() < 0.01, "{at}");
            }
        }
    }

    /// #206: the seat cards keep their distance from Save · quit and from the bottom edge
    /// in every window, at the size the scale gives them, and shrink toward that corner
    /// only where they would come within 24 units of the Propaganda pips. On the owner's
    /// display that is the frame's own place.
    #[test]
    fn the_seat_cards_stand_between_the_pips_and_save_quit() {
        let layout = record();
        let placed = &layout.areas["seats"];
        assert_eq!(placed.anchor, Anchor::BottomBetween);
        assert_eq!(
            placed.between,
            Some(["propaganda".to_string(), "save-quit".to_string()])
        );
        let [x, y, w, h] = placed.bounds;
        for (width, height, shrinks) in [
            (3840., 2160., false),
            (1920., 1080., false),
            (2520., 1080., false),
            (3440., 1440., false),
            (1920., 1280., true),
            (1600., 1280., true),
        ] {
            let window = window(width, height);
            let k = super::super::reference_pixel(&window) / 1.6875;
            let pips = layout.area("propaganda", &window);
            let quit = layout.area("save-quit", &window);
            let area = layout.area("seats", &window);
            let at = format!("{width}x{height}: {area:?}");
            assert!(
                (quit.min.x - area.max.x - (3516. - x - w) * k).abs() < 0.02,
                "{at}"
            );
            assert!(
                (height - area.max.y - (2160. - y - h) * k).abs() < 0.02,
                "{at}"
            );
            assert!((area.width() / area.height() - w / h).abs() < 0.001, "{at}");
            assert!(area.min.x - pips.max.x >= 24. * k - 0.02, "{at}");
            assert_eq!(area.width() < w * k - 0.02, shrinks, "{at}");
            if shrinks {
                assert!((area.min.x - pips.max.x - 24. * k).abs() < 0.02, "{at}");
            }
        }
    }

    /// A between area must name two areas placed by their own edges.
    #[test]
    fn a_between_area_names_two_placed_areas() {
        let record = |marker: &str| {
            format!(
                r#"{{"frame":[3840,2160],"island":{{"bounds":[0,0,1,1],"anchor":"edges"}},"areas":{{
                "standings":{{"bounds":[0,0,1,1],"anchor":"top"}},
                "decision":{{"bounds":[0,0,1,1],"anchor":"top-right"}},
                "cards":{{"bounds":[0,0,1,1],"anchor":"bottom-left"}},
                "marker-supply":{marker}}},
                "notice-slip":{{"bounds":[0,0,1,1],"anchor":"bottom-right"}},
                "save-quit-tag":{{"bounds":[0,0,1,1],"anchor":"bottom-right"}}}}"#
            )
        };
        let read = |marker: &str| Layout::read(record(marker).as_bytes()).map(|_| ());
        assert_eq!(
            read(r#"{"bounds":[0,0,1,1],"anchor":"top-between","between":["cards","standings"]}"#),
            Ok(())
        );
        for refused in [
            r#"{"bounds":[0,0,1,1],"anchor":"top-between"}"#,
            r#"{"bounds":[0,0,1,1],"anchor":"top-between","between":["nowhere","standings"]}"#,
            r#"{"bounds":[0,0,1,1],"anchor":"top-between","between":["marker-supply","standings"]}"#,
            r#"{"bounds":[0,0,1,1],"anchor":"top","between":["cards","standings"]}"#,
        ] {
            assert!(read(refused).is_err(), "{refused}");
        }
    }

    /// Home shows every point inside the box, as large as the box allows and centred
    /// in it, for an island lying across the view as Cuba does.
    #[test]
    fn home_fits_the_points_into_the_box() {
        let points = [
            Vec3::new(-6.3, 0.14, -0.4),
            Vec3::new(-3.9, 0.24, -2.6),
            Vec3::new(0.4, 0., 1.9),
            Vec3::new(4.2, 0.14, 0.3),
            Vec3::new(5.6, 0.24, 3.1),
        ];
        for (window, into) in [
            (Vec2::new(3840., 2160.), Rect::new(620., 470., 2850., 1574.)),
            (Vec2::new(1920., 1280.), Rect::new(367., 278., 1333., 930.)),
            (Vec2::new(2520., 1080.), Rect::new(310., 235., 2025., 787.)),
        ] {
            let framing = fit(&points, window, into).unwrap();
            let seen = projected(&points, framing, window).unwrap();
            assert!(
                seen.min.cmpge(into.min - 0.5).all() && seen.max.cmple(into.max + 0.5).all(),
                "{seen:?} leaves {into:?}"
            );
            assert!(seen.center().abs_diff_eq(into.center(), 0.5), "{seen:?}");
            assert!(
                (seen.width() - into.width()).abs() < 1.
                    || (seen.height() - into.height()).abs() < 1.,
                "{seen:?} could be larger in {into:?}"
            );
        }
    }
}
