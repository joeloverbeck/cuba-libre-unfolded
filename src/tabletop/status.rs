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
    pub control: String,
    pub opinion: String,
    pub contribution: String,
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
    let game = play.game();
    let target = target.filter(|id| {
        game.spaces
            .iter()
            .any(|s| &s.id == id && s.terrain != Terrain::EconomicCenter)
    });
    let eligible = !ui.reader && !over_ui && play.session.is_some();
    ui.status
        .update(target, eligible, time.delta(), Duration::from_millis(250));
}

fn array(rect: Rect) -> [f32; 4] {
    [rect.min.x, rect.min.y, rect.max.x, rect.max.y]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
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
    controls: Query<&Control>,
    parents: Query<&ChildOf>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Shown>>,
) {
    let game = play.game();
    let (camera, camera_at) = *camera;
    let (scale, _) = layout(&window);
    let shown = ui.status.visible().and_then(|id| {
        let space = game.spaces.iter().find(|s| &s.id == id)?;
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
        for (node, at) in &cards {
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
        let size = Vec2::new(272., 128.) * scale;
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
        Some(Shown {
            space: id.clone(),
            control: space.control().map_or("Uncontrolled", Faction::name).into(),
            opinion: opinion.into(),
            contribution,
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
    for (key, top, height) in [
        (format!("atlas-control-{controller}.png"), 0., 36.),
        (format!("atlas-opinion-{political}.png"), 40., 88.),
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
        &shown.contribution,
        x + 12. * scale,
        y + 100. * scale,
        248. * scale,
        17. * scale,
        1.,
        Vec2::ZERO,
        6,
    );
    commands
        .entity(words)
        .remove::<UiRoot>()
        .insert((Tag, shown));
}
