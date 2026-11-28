//! Placement of the selected Atlas geometry, in the VectorCraft profile coordinates.
use super::*;
use crate::{Group, Space};
use std::sync::OnceLock;

pub(super) const PIECE_SCALE: f32 = 0.84;
const CITY_FACTOR: f32 = 0.72;

#[derive(serde::Deserialize)]
struct Placements {
    scale: f32,
    body_offset: f32,
    spaces: BTreeMap<String, [[f32; 2]; 13]>,
}

fn circle_placements() -> &'static Placements {
    static PLACEMENTS: OnceLock<Placements> = OnceLock::new();
    PLACEMENTS.get_or_init(|| {
        serde_json::from_str(include_str!("../../art/atlas/terrain-placements.json"))
            .expect("Atlas circular placements")
    })
}

pub(super) fn scale() -> f32 {
    circle_placements().scale
}

pub(super) fn body_offset() -> f32 {
    circle_placements().body_offset
}

pub(super) fn at(x: f32, z: f32, height: f32) -> Vec3 {
    Vec3::new(
        (x - 700.) / 100. * scale(),
        height,
        (z - 600.) / 100. * scale(),
    )
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
    let [x, z] = offshore::disc(&label.space).unwrap_or_else(|| placement(&label.space).label);
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

pub(super) fn slot_for(space: &Space, group: &Group) -> [f32; 2] {
    let index = group_index(group);
    if space.terrain == Terrain::EconomicCenter {
        // The rules allow only cubes and Guerrillas here. Keep their slots
        // separate from the ninth slot used for the center's marker.
        return offshore::slot(&space.id, index.min(8)).expect("Atlas Economic Center");
    }
    let circle = match index {
        8..=10 => {
            // The saved order keeps the first Base group in its slot when a
            // second faction arrives. A lone surviving group uses the first slot.
            let rank = space
                .pieces
                .iter()
                .filter(|other| other.kind == Kind::Base && other.count != 0)
                .take_while(|other| other.faction != group.faction)
                .count();
            8 + rank.min(1)
        }
        11..=12 => index - 1,
        _ => index,
    };
    land_slot(&space.id, circle)
}

fn group_index(group: &Group) -> usize {
    match (group.faction, group.kind, group.state) {
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
    }
}

pub(super) fn marker(space: &str) -> [f32; 2] {
    offshore::slot(space, 8).unwrap_or_else(|| land_slot(space, 12))
}

fn land_slot(space: &str, index: usize) -> [f32; 2] {
    let [x, z] = circle_placements().spaces[space][index];
    [x - body_offset() * 100. / scale(), z]
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

const PLACEMENTS: [(&str, Placement); 13] = [
    (
        "pinar-del-rio",
        Placement {
            label: [181.000, 447.000],
            city_pivot: None,
        },
    ),
    (
        "la-habana",
        Placement {
            label: [303.000, 391.000],
            city_pivot: None,
        },
    ),
    (
        "matanzas",
        Placement {
            label: [478.000, 403.000],
            city_pivot: None,
        },
    ),
    (
        "las-villas",
        Placement {
            label: [617.000, 438.000],
            city_pivot: None,
        },
    ),
    (
        "camaguey-province",
        Placement {
            label: [822.000, 522.000],
            city_pivot: None,
        },
    ),
    (
        "oriente",
        Placement {
            label: [1127.000, 645.000],
            city_pivot: None,
        },
    ),
    (
        "sierra-maestra",
        Placement {
            label: [1074.000, 731.000],
            city_pivot: None,
        },
    ),
    (
        "havana",
        Placement {
            label: [339.480, 251.480],
            city_pivot: Some([351., 356.]),
        },
    ),
    (
        "camaguey-city",
        Placement {
            label: [814.520, 653.400],
            city_pivot: Some([839., 630.]),
        },
    ),
    (
        "santiago",
        Placement {
            label: [1_174.4, 779.200],
            city_pivot: Some([1160., 775.]),
        },
    ),
    (
        "ec-pinar-la-habana",
        Placement {
            label: [239.000, 420.000],
            city_pivot: None,
        },
    ),
    (
        "ec-las-villas-camaguey",
        Placement {
            label: [747.000, 552.000],
            city_pivot: None,
        },
    ),
    (
        "ec-oriente-sierra",
        Placement {
            label: [1167.000, 717.000],
            city_pivot: None,
        },
    ),
];
