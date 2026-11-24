//! The selected map highlight family, HF-A Survey ink (#68), applied to the map while
//! a decision offers choices on its spaces, 26July's Terror (#72) and Government's
//! Assault (#169), and in removal mode (#73). Every graphic is an
//! unchanged VectorCraft export of `art/proposals/issue-66/map-highlights.vectorcraft`;
//! `art/atlas/highlights.json` places each on the top of the one space it covers
//! (#167: every space has its own rule, dashed rule and veil) and names the state it
//! shows, and places the piece markers under the pieces. Runtime only decides
//! which admitted graphics are visible, from the rules authority's choices
//! (`crate::targets`): it draws nothing of its own.
use super::*;
use crate::{
    Observation,
    targets::{
        PieceState, RemovalTargets, SpaceState, SpaceTarget, removal_targets, space_targets,
    },
};

/// The observation's targeting states, refreshed when the match revision changes, so
/// the highlights, the popover and the seat read one computation.
#[derive(Resource, Default)]
pub(super) struct Targeting {
    key: Option<(bool, u64, bool)>,
    pub view: Option<Observation>,
    pub targets: Option<Vec<SpaceTarget>>,
    /// Removal mode's piece states and dimmed spaces (#73).
    pub removal: Option<RemovalTargets>,
}

impl Targeting {
    pub fn state(&self, space: &str) -> Option<SpaceState> {
        self.targets
            .as_ref()?
            .iter()
            .find(|t| t.space == space)
            .map(|t| t.state)
    }
    /// The space state whose graphic the map shows: a targeting state, or in removal
    /// mode "not involved" for a space holding no piece belonging to the executing faction.
    pub fn shown(&self, space: &str) -> Option<SpaceState> {
        self.state(space).or_else(|| {
            self.removal
                .as_ref()?
                .dimmed
                .iter()
                .any(|s| s == space)
                .then_some(SpaceState::NotInvolved)
        })
    }
    /// The rules authority's reason for `space`'s state, while the map shows the open
    /// decision's states (#170): the space tag's last line. `None` where the authority
    /// gives none, and whenever the targeting is suspended.
    pub fn reason(&self, space: &str) -> Option<&str> {
        self.targets.as_ref()?;
        self.view
            .as_ref()?
            .space_reasons
            .iter()
            .find(|r| r.space == space)
            .map(|r| r.reason.as_str())
    }
    /// An executing faction's group in removal mode, by its `space:group` key.
    pub fn piece(&self, key: &str) -> Option<&crate::targets::PieceTarget> {
        self.removal
            .as_ref()?
            .pieces
            .iter()
            .find(|p| format!("{}:{}", p.space, p.group) == key)
    }
}

/// While the panel presents a Kidnap roll (#74) nothing is targeted and removal mode is
/// suspended: the choices wait for Continue, so the map neither highlights, marks
/// pieces nor opens a popover.
pub(super) fn refresh(play: Res<Play>, roll: Res<die::Roll>, mut targeting: ResMut<Targeting>) {
    let game = play.game();
    let die = die::presented(game.revision, game.kidnap.as_ref(), &roll).is_some();
    let key = (play.session.is_some(), game.revision, die);
    if targeting.key == Some(key) {
        return;
    }
    let view = game.observe();
    targeting.targets = (play.session.is_some() && !die)
        .then(|| space_targets(&view))
        .flatten();
    targeting.removal = (play.session.is_some() && !die)
        .then(|| removal_targets(&view))
        .flatten();
    targeting.view = Some(view);
    targeting.key = Some(key);
}

#[derive(serde::Deserialize)]
struct Manifest {
    graphics: Vec<Graphic>,
    pieces: Vec<PieceMarker>,
}

/// A removal-mode marker, drawn under an executing faction's group in its state.
#[derive(serde::Deserialize)]
struct PieceMarker {
    file: String,
    mark: PieceState,
    /// Atlas units across.
    size: f32,
    /// World units above the region top.
    lift: f32,
}

/// The admitted marker materials and their placement.
#[derive(Resource)]
pub(super) struct Markers {
    mesh: Handle<Mesh>,
    lift: f32,
    materials: Vec<(PieceState, String, Handle<StandardMaterial>)>,
}

/// A marker placed under an executing faction's group in removal mode.
#[derive(Component, Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct Marker {
    pub piece: String,
    pub file: String,
}

#[derive(serde::Deserialize)]
struct Graphic {
    file: String,
    state: SpaceState,
    space: String,
    /// Atlas units: left, top, width, height.
    rect: [f32; 4],
    height: f32,
}

/// A placed highlight: shown while its space is in its state.
#[derive(Component)]
pub(super) struct Decal {
    pub file: String,
    state: SpaceState,
    space: String,
}

pub(super) fn setup(
    mut commands: Commands,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Admitted by hash with its graphics (assets::HIGHLIGHT_MANIFEST).
    let manifest: Manifest = serde_json::from_slice(&art.verified.files["highlights.json"])
        .expect("admitted highlight manifest");
    for graphic in manifest.graphics {
        let [x, y, w, h] = graphic.rect;
        commands.spawn((
            Name::new(format!("highlight {}", graphic.file)),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(w / 100., h / 100.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(art.images[&graphic.file].clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                // Sorted before the region labels just above it, so names and population
                // badges stay over every highlight, however far each plane's centre is
                // from the camera. As a depth offset, 10 units is far below the 0.002
                // world-unit gap to the region top, so the depth test is unchanged.
                depth_bias: -10.,
                ..default()
            })),
            Transform::from_translation(map_at(x + w / 2., y + h / 2., graphic.height)),
            Visibility::Hidden,
            Pickable::IGNORE,
            bevy::light::NotShadowCaster,
            Decal {
                file: graphic.file,
                state: graphic.state,
                space: graphic.space,
            },
        ));
    }
    let size = manifest.pieces.first().map_or(64., |p| p.size);
    commands.insert_resource(Markers {
        mesh: meshes.add(Plane3d::default().mesh().size(size / 100., size / 100.)),
        lift: manifest.pieces.first().map_or(0.003, |p| p.lift),
        materials: manifest
            .pieces
            .iter()
            .map(|marker| {
                (
                    marker.mark,
                    marker.file.clone(),
                    materials.add(StandardMaterial {
                        base_color_texture: Some(art.images[&marker.file].clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        depth_bias: -10.,
                        ..default()
                    }),
                )
            })
            .collect(),
    });
}

/// Places a marker under each eligible group in removal mode: the red survey station
/// where a piece may be removed, the ink ring and lock where the owed Terror protects
/// it. Markers follow the pieces and sit under them, so pieces and counts stay on top.
pub(super) fn markers(
    mut commands: Commands,
    targeting: Res<Targeting>,
    art: Option<Res<Markers>>,
    pieces: Query<(&OwnPiece, &Transform)>,
    placed: Query<Entity, With<Marker>>,
    mut previous: Local<Vec<(Marker, [i32; 3])>>,
) {
    let Some(art) = art else {
        return;
    };
    let mut wanted = vec![];
    if let Some(removal) = targeting.removal.as_ref() {
        for target in &removal.pieces {
            let key = format!("{}:{}", target.space, target.group);
            let Some((piece, at)) = pieces.iter().find(|(p, _)| p.key == key) else {
                continue;
            };
            let Some((_, file, _)) = art.materials.iter().find(|(m, ..)| *m == target.mark) else {
                continue;
            };
            let at = Vec3::new(at.translation.x, piece.base + art.lift, at.translation.z);
            wanted.push((
                Marker {
                    piece: key,
                    file: file.clone(),
                },
                (at * 1000.).round().as_ivec3().to_array(),
            ));
        }
    }
    if *previous == wanted {
        return;
    }
    for entity in &placed {
        commands.entity(entity).despawn();
    }
    for (marker, at) in &wanted {
        let material = art
            .materials
            .iter()
            .find(|(_, file, _)| *file == marker.file)
            .map(|(.., m)| m.clone())
            .unwrap();
        commands.spawn((
            Name::new(format!("marker {} {}", marker.piece, marker.file)),
            Mesh3d(art.mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(IVec3::from_array(*at).as_vec3() / 1000.),
            Pickable::IGNORE,
            bevy::light::NotShadowCaster,
            marker.clone(),
        ));
    }
    *previous = wanted;
}

pub(super) fn show(targeting: Res<Targeting>, mut decals: Query<(&Decal, &mut Visibility)>) {
    for (decal, mut visibility) in &mut decals {
        // A no-effect space draws the veil of a space no choice acts on (#169).
        let shown = targeting.shown(&decal.space).map(SpaceState::drawn) == Some(decal.state);
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
