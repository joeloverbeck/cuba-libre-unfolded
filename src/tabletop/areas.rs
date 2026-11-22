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
    BottomLeft,
    BottomRight,
    Edges,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
pub(super) struct Placed {
    /// x, y, width and height in the layout's frame.
    pub bounds: [f32; 4],
    pub anchor: Anchor,
}

/// The layout record, `atlas-main-view.layout.json`, admitted at launch with the
/// other presentation art.
#[derive(Resource, Clone, Debug, serde::Deserialize)]
pub(super) struct Layout {
    frame: [f32; 2],
    island: Placed,
    areas: BTreeMap<String, Placed>,
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
        Ok(layout)
    }

    /// A placed rectangle in logical window pixels. One frame unit is the same length
    /// in every window of a given scale: the frame is as tall as the reference.
    pub(super) fn place(&self, placed: Placed, window: &Window) -> Rect {
        let unit = layout(window).0 * HEIGHT / self.frame[1];
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
        };
        Rect { min, max }
    }

    /// Every area that lies over the map, by its name in the layout.
    pub(super) fn areas<'a>(
        &'a self,
        window: &'a Window,
    ) -> impl Iterator<Item = (&'a str, Rect)> + 'a {
        self.areas
            .iter()
            .map(|(name, placed)| (name.as_str(), self.place(*placed, window)))
    }

    fn area(&self, name: &str, window: &Window) -> Rect {
        self.place(self.areas[name], window)
    }

    /// The box the thirteen spaces fill at Home.
    pub(super) fn island(&self, window: &Window) -> Rect {
        self.place(self.island, window)
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
pub(super) struct Island(BTreeMap<String, Vec<Vec3>>);

impl Island {
    pub(super) fn surveyed(&self) -> bool {
        !self.0.is_empty()
    }
}

pub(super) fn draw(commands: &mut Commands, layout: &Layout, window: &Window) {
    for (name, rect) in layout.areas(window) {
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
        bounds = Some(bounds.map_or(Rect::from_corners(p, p), |b| Rect {
            min: b.min.min(p),
            max: b.max.max(p),
        }));
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
    mut framed: Local<Option<Vec2>>,
) {
    // The session opens only after the map has loaded and been drawn, so every
    // region's place in the world is settled by then.
    if play.session.is_none() {
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
        island.0 = spaces;
    }
    let size = Vec2::new(window.width(), window.height());
    if *framed == Some(size) {
        return;
    }
    let points: Vec<Vec3> = island.0.values().flatten().copied().collect();
    let Some(framing) = fit(&points, size, layout.island(&window)) else {
        return;
    };
    let resting = orbit.pose() == Pose::at(orbit.home());
    orbit.set_home(framing);
    if resting {
        orbit.focus(framing);
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
                .0
                .iter()
                .filter_map(|(space, points)| {
                    let mut bounds: Option<Rect> = None;
                    for point in points {
                        let p = camera.world_to_viewport(&at, *point).ok()?;
                        bounds = Some(bounds.map_or(Rect::from_corners(p, p), |b| Rect {
                            min: b.min.min(p),
                            max: b.max.max(p),
                        }));
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
            assert!(close(layout.place(*placed, &window), placed.bounds));
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
            let s = super::super::layout(&window).0;
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
                ("decision", [1380., 280., 500., 660.], right, 0.),
                ("session", [1380., 964., 500., 264.], right, bottom),
                ("cards", [40., 964., 600., 264.], 0., bottom),
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
            }
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
