//! The three brass chart insets are world geometry linked to the island's value discs.
use super::*;
use std::f32::consts::FRAC_PI_2;

struct Site {
    id: &'static str,
    disc: [f32; 2],
    route: [[f32; 2]; 3],
    title: &'static str,
}

const SITES: [Site; 3] = [
    Site {
        id: "ec-pinar-la-habana",
        disc: [242., 403.],
        route: [[223., 343.], [156., 241.], [143., 133.]],
        title: "chart-cigars.png",
    },
    Site {
        id: "ec-las-villas-camaguey",
        disc: [750., 535.],
        route: [[704., 599.], [632., 657.], [622., 755.]],
        title: "chart-factory.png",
    },
    Site {
        id: "ec-oriente-sierra",
        disc: [1170., 700.],
        route: [[1202., 645.], [1217., 525.], [1203., 371.]],
        title: "chart-sugar-cane.png",
    },
];

pub(super) fn disc(space: &str) -> Option<[f32; 2]> {
    SITES
        .iter()
        .find(|site| site.id == space)
        .map(|site| site.disc)
}

pub(super) fn slot(space: &str, bay: usize) -> Option<[f32; 2]> {
    let site = SITES.iter().find(|site| site.id == space)?;
    let [x, z] = site.route[2];
    let row = bay / 3;
    let col = bay % 3;
    let center_x = -0.64 + col as f32 * 0.54 + (row % 2) as f32 * 0.20;
    let offset = Vec2::new(center_x - map::body_offset(), -0.225 + row as f32 * 0.425);
    let offset = offset * (100. / map::scale());
    Some([x + offset.x, z + offset.y])
}

#[derive(Component)]
struct Station(&'static str);

fn material(
    materials: &mut Assets<StandardMaterial>,
    rgb: [u8; 3],
    metallic: f32,
) -> Handle<StandardMaterial> {
    let [r, g, b] = rgb;
    materials.add(StandardMaterial {
        base_color: Color::srgb_u8(r, g, b), // own colour: brass inset and chart materials
        metallic,
        perceptual_roughness: 0.77,
        ..default()
    })
}

fn chamfered(meshes: &mut Assets<Mesh>, w: f32, d: f32, cut: f32, height: f32) -> Handle<Mesh> {
    let (x, y) = (w / 2., d / 2.);
    let outline = ConvexPolygon::new([
        Vec2::new(-x + cut, -y),
        Vec2::new(x - cut, -y),
        Vec2::new(x, -y + cut),
        Vec2::new(x, y - cut),
        Vec2::new(x - cut, y),
        Vec2::new(-x + cut, y),
        Vec2::new(-x, y - cut),
        Vec2::new(-x, -y + cut),
    ])
    .expect("convex station outline");
    meshes.add(Extrusion::new(outline, height))
}

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    art: &Art,
) {
    let body = material(materials, [118, 92, 54], 0.10);
    let trim = material(materials, [149, 115, 61], 0.25);
    let field = material(materials, [229, 215, 176], 0.02);
    let link = material(materials, [149, 115, 63], 0.18);
    let hardware = material(materials, [102, 79, 48], 0.40);
    let (w, depth, cut, rim) = (2.04, 1.91, 0.115, 0.067);
    let (title_z, rule_z, title_w) = (-0.650, -0.467, 1.42);
    let layers = [
        (
            chamfered(meshes, w - 0.05, depth - 0.03, cut, 0.175),
            body,
            0.091,
        ),
        (chamfered(meshes, w, depth, cut, 0.073), trim.clone(), 0.206),
        (
            chamfered(meshes, w - 2.5 * rim, depth - 2.4 * rim, cut * 0.85, 0.055),
            field,
            0.2525,
        ),
    ];
    let mounts = meshes.add(Cylinder::new(0.039, 0.026));
    let name_plane = meshes.add(
        Plane3d::default()
            .mesh()
            .size(title_w, title_w * 113. / 490.),
    );
    let rule_mesh = meshes.add(Cuboid::new(w - 0.77_f32.min(w * 0.18), 0.014, 0.025));
    let long_rim = meshes.add(Cuboid::new(w - cut * 2.2, 0.062, rim));
    let short_rim = meshes.add(Cuboid::new(rim, 0.062, depth - cut * 2.2));
    let width = 0.077;
    let rail_mesh = meshes.add(Cuboid::new(1.0, 0.034, width));
    let seam_mesh = meshes.add(Cuboid::new(1.0, 0.011, width * 0.36));

    for site in &SITES {
        let route = site.route;
        let [x, z] = route[2];
        let center = map::at(x, z, 0.);
        let station = commands
            .spawn((
                Name::new(format!("Atlas offshore detail:{}", site.id)),
                Station(site.id),
                Transform::from_translation(center),
                Visibility::default(),
            ))
            .id();
        commands.entity(station).with_children(|children| {
            for (mesh, material, y) in &layers {
                children.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(0., *y, 0.)
                        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                    Pickable::IGNORE,
                ));
            }
            // The fill of the inset sits below the crown of its rim.
            for side in [-1., 1.] {
                children.spawn((
                    Mesh3d(long_rim.clone()),
                    MeshMaterial3d(trim.clone()),
                    Transform::from_xyz(0., 0.276, side * (depth / 2. - rim * 0.66)),
                    Pickable::IGNORE,
                ));
                children.spawn((
                    Mesh3d(short_rim.clone()),
                    MeshMaterial3d(trim.clone()),
                    Transform::from_xyz(side * (w / 2. - rim * 0.66), 0.276, 0.),
                    Pickable::IGNORE,
                ));
            }
            let rivet_x = w / 2. - cut * 1.20;
            let rivet_z = depth / 2. - cut * 1.16;
            for x in [-rivet_x, rivet_x] {
                for z in [-rivet_z, rivet_z] {
                    children.spawn((
                        Mesh3d(mounts.clone()),
                        MeshMaterial3d(hardware.clone()),
                        Transform::from_xyz(x, 0.284, z),
                        Pickable::IGNORE,
                    ));
                }
            }
            children.spawn((
                Mesh3d(name_plane.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(art.images[site.title].clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(0., 0.284, title_z),
                Pickable::IGNORE,
                bevy::light::NotShadowCaster,
            ));
            children.spawn((
                Mesh3d(rule_mesh.clone()),
                MeshMaterial3d(trim.clone()),
                Transform::from_xyz(0., 0.28, rule_z),
                Pickable::IGNORE,
            ));
        });

        // A continuous, slightly raised trace runs under the disc and deck.
        // Its top clears the printed ground while remaining below the tops of
        // city discs and stations, so those pieces still occlude it physically.
        let [start, first, second, end] =
            [site.disc, route[0], route[1], route[2]].map(|[x, z]| map::at(x, z, 0.165));
        let route: [Vec3; 25] = std::array::from_fn(|i| {
            let t = i as f32 / 24.;
            let u = 1. - t;
            start * (u * u * u)
                + first * (3. * u * u * t)
                + second * (3. * u * t * t)
                + end * (t * t * t)
        });
        for pair in route.windows(2) {
            let delta = pair[1] - pair[0];
            let center = (pair[1] + pair[0]) * 0.5;
            let rot = Quat::from_rotation_y((-delta.z).atan2(delta.x));
            commands.spawn((
                Name::new(format!("Atlas physical chart link:{}", site.id)),
                Mesh3d(rail_mesh.clone()),
                MeshMaterial3d(link.clone()),
                Transform::from_translation(center)
                    .with_rotation(rot)
                    .with_scale(Vec3::new(delta.length() + 0.025, 1., 1.)),
                Pickable::IGNORE,
                bevy::light::NotShadowCaster,
            ));
            commands.spawn((
                Mesh3d(seam_mesh.clone()),
                MeshMaterial3d(hardware.clone()),
                Transform::from_translation(center + Vec3::Y * 0.018)
                    .with_rotation(rot)
                    .with_scale(Vec3::new(delta.length() + 0.015, 1., 1.)),
                Pickable::IGNORE,
                bevy::light::NotShadowCaster,
            ));
        }
    }
}

pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let Some((camera, pose)) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .iter(world)
        .next()
        .map(|(camera, pose)| (camera.clone(), *pose))
    else {
        return serde_json::Value::Null;
    };
    let mut stations = world.query::<(&Station, &GlobalTransform)>();
    let seen: BTreeMap<_, _> = stations
        .iter(world)
        .map(|(station, at)| {
            let point = at.translation();
            (
                station.0,
                serde_json::json!({
                    "center_world": point.to_array(),
                    "center_px": camera.world_to_viewport(&pose, point).ok().map(|v| v.to_array()),
                }),
            )
        })
        .collect();
    serde_json::json!(seen)
}
