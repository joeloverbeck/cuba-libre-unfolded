//! Placement of the selected Atlas geometry, in the VectorCraft profile coordinates.
use super::*;
use crate::Group;

pub(super) const SCALE: f32 = 3.;
pub(super) const PIECE_SCALE: f32 = 0.84;
const CITY_FACTOR: f32 = 0.72;

pub(super) fn at(x: f32, z: f32, height: f32) -> Vec3 {
    Vec3::new((x - 700.) / 100. * SCALE, height, (z - 600.) / 100. * SCALE)
}

pub(super) fn top(terrain: Terrain) -> f32 {
    match terrain {
        Terrain::City => 0.24,
        Terrain::EconomicCenter => 0.28,
        _ => 0.14,
    }
}

struct Placement {
    label: [f32; 2],
    origin: [f32; 2],
    city_pivot: Option<[f32; 2]>,
}

fn placement(space: &str) -> &'static Placement {
    &PLACEMENTS
        .iter()
        .find(|(id, _)| *id == space)
        .expect("Atlas space placement")
        .1
}

pub(super) fn label_bounds(label: &LabelSpec) -> [f32; 4] {
    let [x, z] = placement(&label.space).label;
    let [_, _, width, height] = label.bounds;
    let factor = match label.space.as_str() {
        space if space.starts_with("ec-") => 0.66,
        "la-habana" => 0.85 * 0.81,
        "pinar-del-rio" | "sierra-maestra" => 0.85 * 0.85,
        _ => 0.85,
    };
    let (w, h) = (width * factor, height * factor);
    [x - w / 2., z - h / 2., w, h]
}

pub(super) fn slot(space: &str, group: &Group) -> [f32; 2] {
    let key = match (group.faction, group.kind, group.state) {
        (Faction::Government, Kind::Troops, State::Ordinary) => 0,
        (Faction::Government, Kind::Police, State::Ordinary) => 1,
        (Faction::M26, Kind::Guerrilla, State::Underground) => 2,
        (Faction::M26, Kind::Guerrilla, State::Active) => 3,
        (Faction::Directorio, Kind::Guerrilla, State::Underground) => 4,
        (Faction::Directorio, Kind::Guerrilla, State::Active) => 5,
        (Faction::Syndicate, Kind::Guerrilla, State::Underground) => 6,
        (Faction::Syndicate, Kind::Guerrilla, State::Active) => 7,
        (Faction::Government, Kind::Base, State::Ordinary) => 8,
        (Faction::M26, Kind::Base, State::Ordinary) => 9,
        (Faction::Directorio, Kind::Base, State::Ordinary) => 10,
        (Faction::Syndicate, Kind::Casino, State::Open) => 11,
        (Faction::Syndicate, Kind::Casino, State::Closed) => 12,
        _ => unreachable!("unsupported Atlas force group"),
    };
    formation_at(space, key)
}

pub(super) fn marker(space: &str) -> [f32; 2] {
    formation_at(space, 13)
}

fn formation_at(space: &str, index: usize) -> [f32; 2] {
    let formation = match space {
        "la-habana" => &NARROW,
        id if id.starts_with("ec-") => &EC,
        _ => &LAND,
    };
    let [x, z] = placement(space).origin;
    let [dx, dz] = formation[index];
    [x + dx, z + dz]
}

pub(super) fn highlight_bounds(space: &str, rect: [f32; 4]) -> [f32; 4] {
    let Some([px, pz]) = placement(space).city_pivot else {
        return rect;
    };
    let [x, z, w, h] = rect;
    [
        px + CITY_FACTOR * (x - px),
        pz + CITY_FACTOR * (z - pz),
        CITY_FACTOR * w,
        CITY_FACTOR * h,
    ]
}

/// Use the original city extents to keep the selected Home framing.
pub(super) fn framing_point(space: &str, point: Vec3) -> Vec3 {
    let Some([x, z]) = placement(space).city_pivot else {
        return point;
    };
    let pivot = at(x, z, 0.);
    pivot + (point - pivot) / Vec3::new(CITY_FACTOR, 1., CITY_FACTOR)
}

const LAND: [[f32; 2]; 14] = [
    [-34.818, -12.460],
    [-18.998, -12.460],
    [-3.178, -12.460],
    [-26.908, 0.000],
    [12.642, -12.460],
    [-11.088, 0.000],
    [28.462, -12.460],
    [4.732, 0.000],
    [-34.818, 12.460],
    [-18.998, 12.460],
    [-3.178, 12.460],
    [12.642, 12.460],
    [28.462, 12.460],
    [20.552, 0.000],
];
const EC: [[f32; 2]; 14] = [
    [-10.613, -13.832],
    [4.257, -13.832],
    [-18.049, -4.984],
    [-3.178, -4.984],
    [-18.049, 4.984],
    [-3.178, 4.984],
    [11.693, 4.984],
    [6.062, 14.952],
    [0.000, 0.000],
    [0.000, 0.000],
    [0.000, 0.000],
    [0.000, 0.000],
    [0.000, 0.000],
    [11.693, -4.984],
];
const NARROW: [[f32; 2]; 14] = [
    [-26.194, -13.720],
    [-10.850, -13.720],
    [4.494, -13.720],
    [-26.194, 0.000],
    [19.838, -13.720],
    [-10.850, 0.000],
    [4.494, 0.000],
    [19.838, 0.000],
    [-41.538, 13.720],
    [-26.194, 13.720],
    [-10.850, 13.720],
    [4.494, 13.720],
    [19.838, 13.720],
    [35.182, 13.720],
];
const PLACEMENTS: [(&str, Placement); 13] = [
    (
        "pinar-del-rio",
        Placement {
            label: [181.000, 447.000],
            origin: [115.000, 498.000],
            city_pivot: None,
        },
    ),
    (
        "la-habana",
        Placement {
            label: [303.000, 391.000],
            origin: [366.000, 406.000],
            city_pivot: None,
        },
    ),
    (
        "matanzas",
        Placement {
            label: [478.000, 403.000],
            origin: [472.000, 460.000],
            city_pivot: None,
        },
    ),
    (
        "las-villas",
        Placement {
            label: [617.000, 438.000],
            origin: [656.000, 495.000],
            city_pivot: None,
        },
    ),
    (
        "camaguey-province",
        Placement {
            label: [822.000, 522.000],
            origin: [891.000, 579.000],
            city_pivot: None,
        },
    ),
    (
        "oriente",
        Placement {
            label: [1127.000, 645.000],
            origin: [1049.000, 657.000],
            city_pivot: None,
        },
    ),
    (
        "sierra-maestra",
        Placement {
            label: [1074.000, 731.000],
            origin: [1035.000, 776.000],
            city_pivot: None,
        },
    ),
    (
        "havana",
        Placement {
            label: [339.480, 251.480],
            origin: [339.480, 302.480],
            city_pivot: Some([351., 356.]),
        },
    ),
    (
        "camaguey-city",
        Placement {
            label: [814.520, 653.400],
            origin: [814.520, 698.400],
            city_pivot: Some([839., 630.]),
        },
    ),
    (
        "santiago",
        Placement {
            label: [1_174.4, 779.200],
            origin: [1_174.4, 830.200],
            city_pivot: Some([1160., 775.]),
        },
    ),
    (
        "ec-pinar-la-habana",
        Placement {
            label: [239.000, 420.000],
            origin: [242.000, 402.000],
            city_pivot: None,
        },
    ),
    (
        "ec-las-villas-camaguey",
        Placement {
            label: [747.000, 552.000],
            origin: [750.000, 534.000],
            city_pivot: None,
        },
    ),
    (
        "ec-oriente-sierra",
        Placement {
            label: [1167.000, 717.000],
            origin: [1170.000, 699.000],
            city_pivot: None,
        },
    ),
];
