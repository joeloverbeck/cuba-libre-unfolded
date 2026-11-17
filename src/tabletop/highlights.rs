//! The selected map highlight family, HF-A Survey ink (#68), applied to the map while
//! 26July executes Terror (#72). Every graphic is an unchanged VectorCraft export of
//! `art/proposals/issue-66/map-highlights.vectorcraft`; `art/atlas/highlights.json`
//! places each on the region tops it covers and names the state it shows. Runtime only
//! decides which admitted graphics are visible, from the rules authority's choices
//! (`crate::targets`): it draws nothing of its own.
use super::*;
use crate::{
    Observation,
    targets::{SpaceState, SpaceTarget, space_targets},
};

/// The observation's targeting states, refreshed when the match revision changes, so
/// the highlights, the popover and the seat read one computation.
#[derive(Resource, Default)]
pub(super) struct Targeting {
    key: Option<(bool, u64)>,
    pub view: Option<Observation>,
    pub targets: Option<Vec<SpaceTarget>>,
}

impl Targeting {
    pub fn state(&self, space: &str) -> Option<SpaceState> {
        self.targets
            .as_ref()?
            .iter()
            .find(|t| t.space == space)
            .map(|t| t.state)
    }
}

pub(super) fn refresh(play: Res<Play>, mut targeting: ResMut<Targeting>) {
    let game = play.game();
    let key = (play.session.is_some(), game.revision);
    if targeting.key == Some(key) {
        return;
    }
    let view = game.observe();
    targeting.targets = play
        .session
        .is_some()
        .then(|| space_targets(&view))
        .flatten();
    targeting.view = Some(view);
    targeting.key = Some(key);
}

#[derive(serde::Deserialize)]
struct Manifest {
    graphics: Vec<Graphic>,
}

#[derive(serde::Deserialize)]
struct Graphic {
    file: String,
    state: SpaceState,
    spaces: Vec<String>,
    /// Atlas units: left, top, width, height.
    rect: [f32; 4],
    height: f32,
}

/// A placed highlight: shown while every space it covers is in its state.
#[derive(Component)]
pub(super) struct Decal {
    pub file: String,
    state: SpaceState,
    spaces: Vec<String>,
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
                spaces: graphic.spaces,
            },
        ));
    }
}

pub(super) fn show(
    targeting: Res<Targeting>,
    ui: Res<Interface>,
    mut decals: Query<(&Decal, &mut Visibility)>,
) {
    for (decal, mut visibility) in &mut decals {
        let shown = !ui.reader
            && decal
                .spaces
                .iter()
                .all(|space| targeting.state(space) == Some(decal.state));
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
