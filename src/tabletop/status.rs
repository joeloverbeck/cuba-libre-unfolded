//! Cuba's selected hover-only coastal tag; opening timing is shared.
use super::*;
use bevy::camera::primitives::Aabb;
use bevy::mesh::VertexAttributeValues;

#[derive(Component)]
pub(super) struct SpaceTarget(pub String);
#[derive(Component)]
pub(super) struct Tag;
#[derive(Component, Clone, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    pub space: String,
    /// The space type, and a Province's terrain, as its terrain row names it.
    pub terrain: &'static str,
    /// Absent for an Economic Center, which has neither Control nor
    /// Support/Opposition (R 1.6, 1.7).
    pub control: Option<String>,
    pub opinion: Option<String>,
    pub contribution: Option<String>,
    /// While a decision offers choices on the map, the rules authority's reason for
    /// the space's state, drawn as the tag's last line (#170); otherwise absent.
    pub reason: Option<String>,
    /// Where that line's row is drawn, in logical window pixels.
    pub reason_bounds: Option<[f32; 4]>,
    pub bounds: [f32; 4],
    pub region_bounds: [f32; 4],
    pub occupied_bounds: Vec<[f32; 4]>,
    pub viewport: [f32; 4],
    pub leader: [[f32; 2]; 2],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn hover(
    hits: Res<HoverMap>,
    targets: Query<&SpaceTarget>,
    controls: Query<&Control>,
    parents: Query<&ChildOf>,
    nodes: Query<(), With<Node>>,
    play: Res<Play>,
    time: Res<Time>,
    mut ui: ResMut<Interface>,
) {
    let hits = hits.get(&PointerId::Mouse);
    let over_ui = hits.is_some_and(|hits| {
        hits.keys()
            .any(|e| nodes.contains(*e) && !targets.contains(*e))
    });
    let target = hits.and_then(|hits| {
        hits.iter()
            .filter_map(|(entity, hit)| {
                let mut current = Some(*entity);
                while let Some(e) = current {
                    if let Ok(target) = targets.get(e) {
                        return Some((target.0.clone(), hit.depth));
                    }
                    if let Ok(Control {
                        action: Action::Space(id),
                        ..
                    }) = controls.get(e)
                    {
                        return Some((id.clone(), hit.depth));
                    }
                    current = parents.get(e).ok().map(ChildOf::parent);
                }
                None
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    });
    // A space with keyboard focus opens its tag as hover does, after the same dwell
    // (#168). The pointer over the map wins; over an interface node it gives way to
    // focus.
    let (target, eligible) = match (target, focused(&ui)) {
        (Some(pointed), _) if !over_ui => (Some(pointed), true),
        (_, Some(focused)) => (Some(focused), true),
        (pointed, None) => (pointed, !over_ui),
    };
    let eligible = eligible && play.session.is_some();
    ui.status
        .update(target, eligible, time.delta(), Duration::from_millis(250));
}

/// The space that has keyboard focus, by its id (#168).
pub(super) fn focused(ui: &Interface) -> Option<String> {
    ui.choice
        .focus
        .as_deref()?
        .strip_prefix("space:")
        .map(str::to_owned)
}

/// What kind of space this is, in the words of the terrain row and the popover.
pub(super) fn kind(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::City => "City",
        Terrain::Forest => "Province · Forest",
        Terrain::Grassland => "Province · Grassland",
        Terrain::Mountain => "Province · Mountain",
        Terrain::EconomicCenter => "Economic center",
    }
}

/// The authored terrain row for a space type (STATUS.md).
fn kind_row(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::City => "atlas-space-city.png",
        Terrain::Forest => "atlas-space-forest.png",
        Terrain::Grassland => "atlas-space-grassland.png",
        Terrain::Mountain => "atlas-space-mountain.png",
        Terrain::EconomicCenter => "atlas-space-economic-center.png",
    }
}

fn array(rect: Rect) -> [f32; 4] {
    [rect.min.x, rect.min.y, rect.max.x, rect.max.y]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    // The match, and the targeting whose reasons the tag draws (#170).
    (play, targeting): (Res<Play>, Res<super::highlights::Targeting>),
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<TableCamera>>,
    meshes: Query<(Entity, &Mesh3d, &Aabb, &GlobalTransform)>,
    mesh_assets: Res<Assets<Mesh>>,
    targets: Query<&SpaceTarget>,
    labels: Query<(&SpaceTarget, &ComputedNode, &UiGlobalTransform)>,
    // An open space popover (#72): the tag stays visible beside it.
    cards: Query<(&ComputedNode, &UiGlobalTransform), With<super::popover::CardNode>>,
    // Every area lies over the map (#142); a tag must not hide one (#138: the
    // taller tag moved Havana's over the Sequence of Play track).
    areas: Query<(&ComputedNode, &UiGlobalTransform), With<super::areas::Area>>,
    controls: Query<&Control>,
    parents: Query<&ChildOf>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Shown>>,
) {
    let game = play.game();
    let (camera, camera_at) = *camera;
    let scale = reference_pixel(&window);
    let shown = ui.status.visible().and_then(|id| {
        let space = game.spaces.iter().find(|s| &s.id == id)?;
        let reason = targeting.reason(id).map(str::to_owned);
        // The map camera draws the whole window (#142); the areas over it are
        // `occupied` below.
        let viewport = camera.logical_viewport_rect()?;
        // Use the authored solid region's full projected mesh bounds, not its
        // caption or centroid: Havana's city and pieces must stay unobscured.
        let mut regions = BTreeMap::<String, Rect>::new();
        let mut coast = Vec::new();
        let mut coast_height = f32::NEG_INFINITY;
        for (entity, mesh, _, at) in &meshes {
            let mut current = Some(entity);
            let mut belongs = None;
            while let Some(e) = current {
                if let Ok(Control {
                    action: Action::Space(found),
                    ..
                }) = controls.get(e)
                {
                    belongs = Some(found.clone());
                    break;
                }
                current = parents.get(e).ok().map(ChildOf::parent);
            }
            let Some(belongs) = belongs else {
                continue;
            };
            if let Some(VertexAttributeValues::Float32x3(vertices)) = mesh_assets
                .get(&mesh.0)
                .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
            {
                for vertex in vertices {
                    let world = at.transform_point(Vec3::from_array(*vertex));
                    if let Ok(p) = camera.world_to_viewport(camera_at, world) {
                        if belongs == *id {
                            if world.y > coast_height + 0.001 {
                                coast.clear();
                                coast_height = world.y;
                            }
                            if (world.y - coast_height).abs() < 0.001 {
                                coast.push(p);
                            }
                        }
                        regions
                            .entry(belongs.clone())
                            .and_modify(|region| {
                                region.min = region.min.min(p);
                                region.max = region.max.max(p);
                            })
                            .or_insert(Rect::from_corners(p, p));
                    }
                }
            }
        }
        let region = *regions.get(id)?;
        if !viewport.contains(region.center()) {
            return None;
        }
        let mut occupied = regions.values().copied().collect::<Vec<_>>();
        for (entity, _, bounds, at) in &meshes {
            if targets.get(entity).is_err() {
                continue;
            }
            let points = [-1., 1.]
                .into_iter()
                .flat_map(|x| {
                    [-1., 1.].into_iter().flat_map(move |y| {
                        [-1., 1.].into_iter().map(move |z| {
                            Vec3::from(bounds.center)
                                + Vec3::from(bounds.half_extents) * Vec3::new(x, y, z)
                        })
                    })
                })
                .filter_map(|corner| {
                    camera
                        .world_to_viewport(camera_at, at.transform_point(corner))
                        .ok()
                })
                .collect::<Vec<_>>();
            if let Some(first) = points.first() {
                let mut r = Rect::from_corners(*first, *first);
                for p in points {
                    r.min = r.min.min(p);
                    r.max = r.max.max(p);
                }
                occupied.push(r);
            }
        }
        for (node, at) in cards.iter().chain(&areas) {
            occupied.push(Rect::from_center_size(
                at.translation * node.inverse_scale_factor,
                node.size() * node.inverse_scale_factor,
            ));
        }
        for (_, node, at) in &labels {
            let center = at.translation * node.inverse_scale_factor;
            occupied.push(Rect::from_center_size(
                center,
                node.size() * node.inverse_scale_factor,
            ));
        }
        let size = Vec2::new(272., tag_height(space.terrain, reason.is_some())) * scale;
        let gap = 22. * scale;
        let center = region.center();
        let mut candidates = vec![
            Vec2::new(region.max.x + gap, center.y - size.y / 2.),
            Vec2::new(center.x - size.x / 2., region.min.y - size.y - gap),
            Vec2::new(center.x - size.x / 2., region.max.y + gap),
            Vec2::new(region.min.x - size.x - gap, center.y - size.y / 2.),
        ];
        let margin = 2. * scale;
        let lo = viewport.min + Vec2::splat(margin);
        let hi = viewport.max - Vec2::splat(margin) - size;
        if hi.cmplt(lo).any() {
            return None;
        }
        candidates.extend([
            Vec2::new(region.min.x - size.x - gap, lo.y),
            Vec2::new(region.max.x + gap, lo.y),
            Vec2::new(region.min.x - size.x - gap, hi.y),
            Vec2::new(region.max.x + gap, hi.y),
        ]);
        // Try the coast across the available viewport as well as the nearest
        // four sides. Prefer empty water over covering another region.
        for ix in 0..=12 {
            for iy in 0..=5 {
                candidates.push(lo + (hi - lo) * Vec2::new(ix as f32 / 12., iy as f32 / 5.));
            }
        }
        let tag = candidates
            .into_iter()
            .map(|p| Rect::from_corners(p.clamp(lo, hi), p.clamp(lo, hi) + size))
            .filter(|r| {
                r.max.x + gap <= region.min.x
                    || r.min.x >= region.max.x + gap
                    || r.max.y + gap <= region.min.y
                    || r.min.y >= region.max.y + gap
            })
            .filter(|r| {
                occupied.iter().all(|other| {
                    r.max.x + 2. * scale <= other.min.x
                        || r.min.x >= other.max.x + 2. * scale
                        || r.max.y + 2. * scale <= other.min.y
                        || r.min.y >= other.max.y + 2. * scale
                })
            })
            .min_by(|a, b| {
                let leader_length = |r: &Rect| {
                    coast
                        .iter()
                        .map(|p| p.distance_squared(p.clamp(r.min, r.max)))
                        .min_by(f32::total_cmp)
                        .unwrap_or(f32::INFINITY)
                };
                leader_length(a).total_cmp(&leader_length(b))
            })?;
        // A vertex on the actual authored coast unambiguously belongs to the
        // region, unlike an AABB corner. Its nearest exposed coast keeps the
        // leader away from names and pieces in the interior.
        let start = *coast.iter().min_by(|a, b| {
            a.distance_squared(a.clamp(tag.min, tag.max))
                .total_cmp(&b.distance_squared(b.clamp(tag.min, tag.max)))
        })?;
        let end = start.clamp(tag.min, tag.max);
        let (control, opinion, contribution) = if space.terrain == Terrain::EconomicCenter {
            (None, None, None)
        } else {
            let opinion = match space.support {
                -2 => "Active Opposition",
                -1 => "Passive Opposition",
                0 => "Neutral",
                1 => "Passive Support",
                2 => "Active Support",
                _ => return None,
            };
            let contribution = if space.support == 0 {
                "0 Support/Opposition".into()
            } else {
                format!(
                    "×{} · {} +{}",
                    space.support.abs(),
                    if space.support > 0 {
                        "Government"
                    } else {
                        "26July"
                    },
                    space.population as i16 * space.support.abs() as i16
                )
            };
            (
                Some(space.control().map_or("Uncontrolled", Faction::name).into()),
                Some(opinion.into()),
                Some(contribution),
            )
        };
        let reason_bounds = reason.as_ref().map(|_| {
            let top = tag.min.y + reason_top(space.terrain) * scale;
            [tag.min.x, top, tag.max.x, top + REASON_ROW * scale]
        });
        Some(Shown {
            space: id.clone(),
            terrain: kind(space.terrain),
            control,
            opinion,
            contribution,
            reason,
            reason_bounds,
            bounds: array(tag),
            region_bounds: array(region),
            occupied_bounds: occupied.into_iter().map(array).collect(),
            viewport: array(viewport),
            leader: [start.to_array(), end.to_array()],
        })
    });
    if *previous == shown {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    *previous = shown.clone();
    let Some(shown) = shown else {
        return;
    };
    let [x, y, _, _] = shown.bounds;
    let space = game.spaces.iter().find(|s| s.id == shown.space).unwrap();
    let controller = space.control().map_or("uncontrolled", slug);
    let political = match space.support {
        -2 => "active-opposition",
        -1 => "passive-opposition",
        0 => "neutral",
        1 => "passive-support",
        _ => "active-support",
    };
    let a = Vec2::from_array(shown.leader[0]);
    let b = Vec2::from_array(shown.leader[1]);
    let mid = (a + b) / 2.;
    let length = a.distance(b);
    let line = picture(
        &mut commands,
        &art,
        "atlas-status-leader.png",
        [mid.x - length / 2., mid.y - scale, length, 2. * scale],
        1.,
        Vec2::ZERO,
        4,
    );
    commands.entity(line).remove::<UiRoot>().insert((
        Tag,
        UiTransform {
            rotation: Rot2::radians((b - a).to_angle()),
            ..default()
        },
    ));
    let anchor = picture(
        &mut commands,
        &art,
        "atlas-status-anchor.png",
        [a.x - 4. * scale, a.y - 4. * scale, 8. * scale, 8. * scale],
        1.,
        Vec2::ZERO,
        5,
    );
    commands.entity(anchor).remove::<UiRoot>().insert(Tag);
    let terrain = picture(
        &mut commands,
        &art,
        kind_row(space.terrain),
        [x, y, 272. * scale, 36. * scale],
        1.,
        Vec2::ZERO,
        5,
    );
    commands.entity(terrain).remove::<UiRoot>().insert(Tag);
    if let Some(contribution) = &shown.contribution {
        for (key, top, height) in [
            (format!("atlas-control-{controller}.png"), 40., 36.),
            (format!("atlas-opinion-{political}.png"), 80., 88.),
        ] {
            let row = picture(
                &mut commands,
                &art,
                &key,
                [x, y + top * scale, 272. * scale, height * scale],
                1.,
                Vec2::ZERO,
                5,
            );
            commands.entity(row).remove::<UiRoot>().insert(Tag);
        }
        let words = text(
            &mut commands,
            &art,
            contribution,
            x + TEXT_LEFT * scale,
            y + 140. * scale,
            TEXT_WIDTH * scale,
            TEXT_SIZE * scale,
            1.,
            Vec2::ZERO,
            6,
        );
        commands.entity(words).remove::<UiRoot>().insert(Tag);
    }
    // The open decision's line (#170): the authority's words in the contribution's
    // text style, on the authored row plate below the tag's other rows.
    if let Some(reason) = &shown.reason {
        let top = reason_top(space.terrain);
        let row = picture(
            &mut commands,
            &art,
            REASON_PLATE,
            [x, y + top * scale, 272. * scale, REASON_ROW * scale],
            1.,
            Vec2::ZERO,
            5,
        );
        commands.entity(row).remove::<UiRoot>().insert(Tag);
        let words = text(
            &mut commands,
            &art,
            reason,
            x + TEXT_LEFT * scale,
            y + (top + REASON_TEXT_TOP) * scale,
            TEXT_WIDTH * scale,
            TEXT_SIZE * scale,
            1.,
            Vec2::ZERO,
            6,
        );
        commands
            .entity(words)
            .remove::<UiRoot>()
            .insert((Tag, Reason));
    }
    commands.entity(terrain).insert(shown);
}

/// The tag's drawn line of the open decision's reason (#170), for the seat.
#[derive(Component)]
pub(super) struct Reason;

/// The tag's dynamic text style (STATUS.md): Lato at 17 reference pixels, 12 in from
/// the row's left edge and at most 248 wide.
const TEXT_SIZE: f32 = 17.;
const TEXT_LEFT: f32 = 12.;
const TEXT_WIDTH: f32 = 248.;
/// The reason's row plate, a Control row frame with no mark or label (STATUS.md).
const REASON_PLATE: &str = "atlas-status-reason.png";
const REASON_ROW: f32 = 36.;
/// The reason's text top within its row, which centres one 17-pixel line in it.
const REASON_TEXT_TOP: f32 = 8.;

/// The rows above the reason: the terrain row alone for an Economic Center;
/// otherwise the terrain row, 4 px, Control, 4 px and the political row (STATUS.md).
fn rows_height(terrain: Terrain) -> f32 {
    if terrain == Terrain::EconomicCenter {
        36.
    } else {
        168.
    }
}

/// Where the reason's row starts in the tag: 4 px below the rows above it.
fn reason_top(terrain: Terrain) -> f32 {
    rows_height(terrain) + 4.
}

/// The tag's authored height, with the reason's row when the open decision gives one.
fn tag_height(terrain: Terrain, reason: bool) -> f32 {
    if reason {
        reason_top(terrain) + REASON_ROW
    } else {
        rows_height(terrain)
    }
}

#[cfg(test)]
mod tests {
    use super::super::highlights::Targeting;
    use super::*;
    use crate::targets::space_targets;

    fn choose(game: &mut Game, action: &str) {
        let view = game.observe();
        let id = format!("{}:{action}", view.revision);
        let owner = view
            .choices
            .iter()
            .find(|c| c.id == id)
            .map_or(view.owner, |c| c.owner);
        game.submit(owner, view.revision, &id)
            .unwrap_or_else(|e| panic!("{action}: {e}"));
    }

    /// The targeting the map and the tag read, as `highlights::refresh` computes it.
    fn targeting(game: &Game) -> Targeting {
        let view = game.observe();
        let mut targeting = Targeting::default();
        targeting.targets = space_targets(&view);
        targeting.view = Some(view);
        targeting
    }

    /// Card 18 with Government's Assault open after the shaded Event; 26July gave
    /// Government `give` of the 5 Resources seed 2's Kidnap roll brought it.
    fn assault(give: u8) -> Game {
        let mut game = crate::terror_chosen(Game::opening_with_seed(2));
        for action in ["kidnap", "kidnap:roll", "terror:santiago"] {
            choose(&mut game, action);
        }
        if give > 0 {
            choose(&mut game, "transfer");
            choose(&mut game, &format!("give:0:{give}"));
        }
        for action in [
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
        ] {
            choose(&mut game, action);
        }
        game
    }

    /// #170: the tag's line for each kind of reason is the rules authority's, drawn on
    /// one row below the tag's other rows; a space that needs no reason, and every
    /// space outside a decision that offers spaces, keeps the tag it had.
    #[test]
    fn the_tag_s_line_is_the_authority_s_reason_for_each_kind() {
        let terror = crate::terror_chosen(Game::opening_with_seed(2));
        let mut paid = assault(4);
        choose(&mut paid, "assault:santiago:26july");
        let mut resolved = crate::terror_chosen(Game::opening_with_seed(2));
        for action in ["kidnap", "kidnap:roll", "terror:santiago", "remove"] {
            choose(&mut resolved, action);
        }
        choose(&mut resolved, "remove:oriente:underground:1");
        choose(&mut resolved, "done-removal");
        let mut owed = crate::terror_chosen(Game::opening_with_seed(2));
        choose(&mut owed, "kidnap");
        choose(&mut owed, "kidnap:roll");
        for (game, space, terrain, line) in [
            // Assault: why it would remove nothing (R 3.2.4).
            (
                &assault(4),
                "matanzas",
                Terrain::Grassland,
                Some("No enemy pieces (R 3.2.4)"),
            ),
            (
                &assault(4),
                "ec-oriente-sierra",
                Terrain::EconomicCenter,
                Some("No enemy pieces (R 3.2.4)"),
            ),
            (
                &assault(4),
                "sierra-maestra",
                Terrain::Mountain,
                Some("No Troops (R 3.2.4)"),
            ),
            // Assault: a transfer would open a space Government cannot pay for.
            (
                &assault(0),
                "havana",
                Terrain::City,
                Some("A Resource transfer opens it"),
            ),
            // Assault: assaulted in this Operation, and what it paid (R 3.1).
            (
                &paid,
                "santiago",
                Terrain::City,
                Some("Assaulted · paid 2 Resources"),
            ),
            // Assault: a selectable space needs no reason.
            (&paid, "havana", Terrain::City, None),
            // Terror: what would open a blocked space.
            (
                &terror,
                "oriente",
                Terrain::Forest,
                Some("A Resource transfer opens it"),
            ),
            // Terror: owed after Kidnap (R 4.3.3).
            (
                &owed,
                "santiago",
                Terrain::City,
                Some("Terror is owed here (R 4.3.3)"),
            ),
            // Terror: what a resolved Terror paid (R 3.3.4).
            (
                &resolved,
                "santiago",
                Terrain::City,
                Some("Terror here paid 1 Resource"),
            ),
            // Terror: no choice can act there.
            (
                &resolved,
                "oriente",
                Terrain::Forest,
                Some("No Underground 26July Guerrilla"),
            ),
            (
                &terror,
                "havana",
                Terrain::City,
                Some("No Underground 26July Guerrilla"),
            ),
            // At the action choice no choice acts on a space: no line.
            (&Game::opening(), "matanzas", Terrain::Grassland, None),
        ] {
            let targeting = targeting(game);
            assert_eq!(targeting.reason(space), line, "{space}");
            let view = targeting.view.as_ref().unwrap();
            assert_eq!(
                view.spaces.iter().find(|s| s.id == space).unwrap().terrain,
                terrain
            );
            let rows = if terrain == Terrain::EconomicCenter {
                36.
            } else {
                168.
            };
            assert_eq!(
                tag_height(terrain, line.is_some()),
                if line.is_some() {
                    rows + 4. + 36.
                } else {
                    rows
                },
                "{space}"
            );
        }
        // A suspended targeting (a presented die, removal mode) shows no line, though
        // the observation still has the reasons.
        let mut suspended = targeting(&terror);
        suspended.targets = None;
        assert_eq!(suspended.reason("havana"), None);
        // The rows stay where the tag's specification puts them.
        assert_eq!(reason_top(Terrain::City), 172.);
        assert_eq!(reason_top(Terrain::EconomicCenter), 40.);
        assert!(REASON_TEXT_TOP * 2. + TEXT_SIZE * 1.2 <= REASON_ROW + 1.);
    }
}
