//! The selected Atlas solids and face marks, shared by resting and moving pieces.
use super::*;
use crate::Group;

pub(super) fn group_half_height(kind: Kind) -> f32 {
    match kind {
        Kind::Guerrilla => 0.12,
        Kind::Troops | Kind::Police => 0.09,
        Kind::Base | Kind::Casino => 0.0375,
    }
}

pub(super) fn group_color(group: &Group) -> Color {
    match group.faction {
        Faction::Government if group.kind == Kind::Police => {
            Color::srgb_u8(143, 185, 210) // own colour: piece palette (art/atlas/README.md)
        }
        Faction::Government => Color::srgb_u8(49, 111, 156), // own colour: piece palette (art/atlas/README.md)
        Faction::M26 => Color::srgb_u8(180, 60, 54), // own colour: piece palette (art/atlas/README.md)
        Faction::Directorio => Color::srgb_u8(228, 189, 70), // own colour: piece palette (art/atlas/README.md)
        Faction::Syndicate => Color::srgb_u8(65, 117, 87), // own colour: piece palette (art/atlas/README.md)
    }
}

pub(super) fn spawn_group_body(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    group: &Group,
    name: String,
    transform: Transform,
) -> Entity {
    let mesh = match group.kind {
        Kind::Troops | Kind::Police => Mesh::from(Cuboid::new(0.18, 0.18, 0.18)),
        Kind::Guerrilla => Cylinder::new(0.10, 0.24)
            .mesh()
            .resolution(8)
            .build()
            .with_duplicated_vertices()
            .with_computed_flat_normals(),
        Kind::Base | Kind::Casino => Mesh::from(Cylinder::new(0.15, 0.075)),
    };
    token::spawn_body(
        commands,
        meshes,
        materials,
        token::Body {
            name,
            mesh,
            color: group_color(group),
            transform,
        },
    )
}

pub(super) fn add_group_face(
    commands: &mut Commands,
    art: &Art,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    parent: Entity,
    group: &Group,
    bottom: bool,
) {
    let (mark, width) = match (group.kind, group.state) {
        (_, State::Active) => ("atlas-active-star.png", 0.17),
        (Kind::Casino, State::Open) => ("atlas-casino-gold.png", 0.21),
        _ => return,
    };
    let y = group_half_height(group.kind) + 0.002;
    let transform = if bottom {
        Transform::from_xyz(0., -y, 0.).with_rotation(Quat::from_rotation_z(std::f32::consts::PI))
    } else {
        Transform::from_xyz(0., y, 0.)
    };
    token::add_face(
        commands,
        meshes,
        materials,
        parent,
        token::Face {
            mesh: Plane3d::default().mesh().size(width, width).build(),
            image: art.images[mark].clone(),
            alpha: AlphaMode::Blend,
            transform,
        },
    );
}
