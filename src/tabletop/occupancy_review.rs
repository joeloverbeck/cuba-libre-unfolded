//! Fixed, read-only map positions for the Atlas owner comparison (#267).
//! Their only consumer is the native scene. They have no replay or player session.
use super::*;
use crate::{Group, Observation, inventory};
use serde_json::{Value, json};

#[derive(Clone)]
pub struct OccupancyReview {
    pub(super) name: String,
    pub(super) position: Game,
    target: String,
    cash: [u8; 4],
    sabotage: bool,
}

impl OccupancyReview {
    /// One target space at a time so saturating it never borrows pieces from the
    /// rest of the board. The envelopes and sources are in art/atlas/OCCUPANCY.md.
    pub fn named(name: &str) -> Result<Self, String> {
        use Faction::*;
        use Kind::*;
        use State::*;

        let (case, target) = name.split_once(':').ok_or(
            "Use dense:SPACE, stacks:SPACE, sparse:SPACE, empty:SPACE, sinatra:havana or sinatra-stacks:havana.",
        )?;
        let mut position = Game::opening();
        for space in &mut position.spaces {
            space.pieces.clear();
            space.support = 0;
        }
        let space = position
            .spaces
            .iter_mut()
            .find(|space| space.id == target)
            .ok_or_else(|| format!("Unknown occupancy review space: {target}"))?;
        let economic = space.terrain == Terrain::EconomicCenter;
        let sinatra = matches!(case, "sinatra" | "sinatra-stacks");
        if sinatra && target != "havana" {
            return Err("The shaded Sinatra case is specific to Havana.".into());
        }
        let mut cash = [0; 4];
        let mut sabotage = false;
        let mut give = |faction, kind, state, count| {
            space.pieces.push(Group {
                faction,
                kind,
                state,
                count,
            });
        };
        match case {
            "empty" => {}
            "sparse" => {
                give(M26, Guerrilla, Active, 1);
                if !economic {
                    space.support = -1;
                }
            }
            "dense" | "stacks" | "sinatra" | "sinatra-stacks" => {
                let stacks = matches!(case, "stacks" | "sinatra-stacks");
                give(Government, Troops, Ordinary, 15);
                give(Government, Police, Ordinary, 15);
                if stacks {
                    give(M26, Guerrilla, Underground, 15);
                    give(Directorio, Guerrilla, Active, 15);
                    give(Syndicate, Guerrilla, Underground, 6);
                    cash = [0, 4, 0, 0];
                } else {
                    give(M26, Guerrilla, Underground, 14);
                    give(M26, Guerrilla, Active, 1);
                    give(Directorio, Guerrilla, Underground, 1);
                    give(Directorio, Guerrilla, Active, 14);
                    give(Syndicate, Guerrilla, Underground, 5);
                    give(Syndicate, Guerrilla, Active, 1);
                    cash = [1, 1, 1, 1];
                }
                let casinos = if sinatra { 3 } else { 2 };
                if economic {
                    sabotage = true;
                } else {
                    if stacks {
                        give(Government, Base, Ordinary, 2);
                        give(Syndicate, Casino, Open, casinos);
                    } else {
                        give(Government, Base, Ordinary, 1);
                        give(M26, Base, Ordinary, 1);
                        give(Syndicate, Casino, Open, casinos - 1);
                        give(Syndicate, Casino, Closed, 1);
                    }
                    space.terror = 20;
                    space.support = -2;
                }
            }
            _ => return Err(format!("Unknown occupancy review case: {case}")),
        }
        for group in &mut position.available {
            let on_map = crate::game::total(
                position.spaces.iter().flat_map(|space| &space.pieces),
                group.faction,
                group.kind,
            );
            group.count = inventory(group.faction, group.kind) - on_map;
        }
        Ok(Self {
            name: name.into(),
            position,
            target: target.into(),
            cash,
            sabotage,
        })
    }

    pub(super) fn observe(&self) -> Observation {
        let mut view = self.position.observe();
        view.choices.clear();
        view.cash_available -= self.cash.iter().sum::<u8>();
        if self.sabotage {
            view.marker_supply = crate::MarkerSupply::left_by(0, 1);
        }
        view
    }

    pub(super) fn cash(&self, space: &str, group: usize) -> u8 {
        if self.target == space {
            self.cash.get(group).copied().unwrap_or_default()
        } else {
            0
        }
    }

    pub(super) fn sabotaged(&self, space: &str) -> bool {
        self.sabotage && self.target == space
    }
}

/// Simple solids expose the space reserved by several Cash markers under a carrier.
/// The comparison can change these dimensions before adopting their final art.
pub(super) fn cash_solids(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    space: &str,
    group_index: usize,
    quantity: u8,
    at: Vec3,
) -> f32 {
    const THICKNESS: f32 = 0.035;
    let scale = map::PIECE_SCALE;
    for layer in 0..quantity {
        let token = token::spawn_body(
            commands,
            meshes,
            materials,
            token::Body {
                name: format!("{space} Review Cash {group_index}.{layer}"),
                mesh: Mesh::from(Cuboid::new(0.26, THICKNESS, 0.24)),
                color: Color::srgb_u8(88, 141, 69), // own colour: review-only Cash reservation
                transform: Transform::from_translation(
                    at + Vec3::Y * (f32::from(layer) + 0.5) * THICKNESS * scale,
                )
                .with_scale(Vec3::splat(scale)),
            },
        );
        commands
            .entity(token)
            .insert((PieceVisual, Pickable::IGNORE));
    }
    f32::from(quantity) * THICKNESS * scale
}

pub(super) fn sabotage_solid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    art: &Art,
    space: &str,
    at: Vec3,
) {
    let token = token::spawn_body(
        commands,
        meshes,
        materials,
        token::Body {
            name: format!("{space} Review Sabotage"),
            mesh: Mesh::from(Cylinder::new(0.15, 0.075)),
            color: Color::srgb_u8(205, 151, 140), // own colour: review-only Sabotage reservation
            transform: Transform::from_translation(at + Vec3::Y * 0.0375 * map::PIECE_SCALE)
                .with_scale(Vec3::splat(map::PIECE_SCALE)),
        },
    );
    commands
        .entity(token)
        .insert((PieceVisual, Pickable::IGNORE));
    roundel::spawn(
        commands,
        art,
        space,
        "S".into(),
        token,
        Vec3::new(0.15, -0.0125, 0.),
    );
}

/// Screen and world bounds of actual piece, face and label mesh vertices.
pub(super) fn mesh_bounds(world: &mut World) -> Value {
    use bevy::mesh::VertexAttributeValues;

    let Ok((camera, camera_at)) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TableCamera>>()
        .single(world)
        .map(|(camera, at)| (camera.clone(), *at))
    else {
        return Value::Null;
    };
    let meshes = world
        .query::<(Entity, &Mesh3d, &GlobalTransform)>()
        .iter(world)
        .map(|(entity, mesh, at)| (entity, mesh.0.id(), *at))
        .collect::<Vec<_>>();
    let geometry = world.resource::<Assets<Mesh>>();
    let mut shapes = BTreeMap::<String, Footprint>::new();
    for (entity, mesh, at) in meshes {
        let mut current = Some(entity);
        let root = loop {
            let Some(e) = current else {
                break None;
            };
            let name = world.get::<Name>(e).map(Name::as_str);
            if world.get::<PieceVisual>(e).is_some()
                || name.is_some_and(|n| n.starts_with("Atlas label:"))
            {
                break name.map(|name| (e, name));
            }
            current = world.get::<ChildOf>(e).map(ChildOf::parent);
        };
        let (Some((root, name)), Some(VertexAttributeValues::Float32x3(vertices))) = (
            root,
            geometry
                .get(mesh)
                .and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION)),
        ) else {
            continue;
        };
        let name = if root != entity
            && (name.ends_with("Guerrilla Active") || name.ends_with("Casino Open"))
        {
            format!("{name} face")
        } else {
            name.to_owned()
        };
        let shape = shapes.entry(name).or_default();
        for &p in vertices {
            let point = at.transform_point(Vec3::from(p));
            shape.min = shape.min.min(point);
            shape.max = shape.max.max(point);
            if let Ok(pixel) = camera.world_to_viewport(&camera_at, point) {
                shape.screen.min = shape.screen.min.min(pixel);
                shape.screen.max = shape.screen.max.max(pixel);
            }
        }
    }
    let shapes: BTreeMap<_, _> = shapes
        .into_iter()
        .map(|(name, bounds)| {
            let rect = bounds.screen;
            let screen = (rect.min.is_finite() && rect.max.is_finite())
                .then_some([rect.min.x, rect.min.y, rect.max.x, rect.max.y]);
            (name, json!({"min": bounds.min.to_array(), "max": bounds.max.to_array(), "screen": screen}))
        })
        .collect();
    json!(shapes)
}

struct Footprint {
    min: Vec3,
    max: Vec3,
    screen: Rect,
}

impl Default for Footprint {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
            screen: Rect {
                min: Vec2::splat(f32::INFINITY),
                max: Vec2::splat(f32::NEG_INFINITY),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_spaces_have_the_printed_occupancy_envelopes() {
        // P page 36 for the location values. R §§1.4.1–2 and p.20 for 66
        // cubes/Guerrillas, plus at most two Bases and two Casinos outside ECs.
        let printed = [
            ("havana", 6, 0),
            ("camaguey-city", 1, 0),
            ("santiago", 1, 0),
            ("pinar-del-rio", 1, 0),
            ("la-habana", 1, 0),
            ("matanzas", 1, 0),
            ("las-villas", 2, 0),
            ("camaguey-province", 1, 0),
            ("oriente", 2, 0),
            ("sierra-maestra", 1, 0),
            ("ec-pinar-la-habana", 0, 3),
            ("ec-las-villas-camaguey", 0, 3),
            ("ec-oriente-sierra", 0, 2),
        ];
        for (id, population, economy) in printed {
            for (case, populated) in [
                ("dense", true),
                ("stacks", true),
                ("sparse", false),
                ("empty", false),
            ] {
                let review = OccupancyReview::named(&format!("{case}:{id}")).unwrap();
                let view = review.observe();
                assert_eq!(view.spaces.len(), 13);
                let target = view.spaces.iter().find(|space| space.id == id).unwrap();
                assert_eq!(
                    (target.population, target.economy),
                    (population, economy),
                    "{id}"
                );
                let ec = economy != 0;
                let expected = match (case, ec) {
                    ("dense", true) => (8, 66),
                    ("dense", false) => (12, 70),
                    ("stacks", true) => (5, 66),
                    ("stacks", false) => (7, 70),
                    ("sparse", _) => (1, 1),
                    _ => (0, 0),
                };
                assert_eq!(
                    (target.pieces.len(), view.map_piece_count),
                    expected,
                    "{case}:{id}"
                );
                assert_eq!(
                    view.marker_supply.terror_on_map,
                    if populated && !ec { 20 } else { 0 }
                );
                assert_eq!(
                    view.marker_supply.sabotage_on_map,
                    u8::from(populated && ec)
                );
                assert_eq!(view.cash_available, if populated { 0 } else { 4 });
                assert!(view.choices.is_empty());
            }
        }
    }

    #[test]
    fn sinatra_adds_one_casino_only_in_havana() {
        // P p. 35 and R §§1.4.2, 5.1.1 allow one additional Open Casino.
        assert!(OccupancyReview::named("sinatra:la-habana").is_err());
        for (case, faces) in [
            ("sinatra:havana", vec![(State::Open, 2), (State::Closed, 1)]),
            ("sinatra-stacks:havana", vec![(State::Open, 3)]),
        ] {
            let view = OccupancyReview::named(case).unwrap().observe();
            assert_eq!(view.map_piece_count, 71);
            let havana = view
                .spaces
                .iter()
                .find(|space| space.id == "havana")
                .unwrap();
            assert_eq!(
                havana
                    .pieces
                    .iter()
                    .filter(|group| group.kind == Kind::Casino)
                    .map(|group| (group.state, group.count))
                    .collect::<Vec<_>>(),
                faces
            );
            assert_eq!(
                havana
                    .pieces
                    .iter()
                    .filter(|group| group.kind == Kind::Base)
                    .map(|group| group.count)
                    .sum::<u8>(),
                2
            );
        }
    }
}
