//! Selected Atlas presentation. Input travels through Bevy picking into the checked game seam.
use crate::{
    Faction, Game, Kind, State, Terrain,
    assets::{self, Verified},
    save::Session,
};
use bevy::{
    app::AnimationSystems,
    asset::RenderAssetUsages,
    camera::Viewport,
    core_pipeline::tonemapping::Tonemapping,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    picking::{events::PointerClick, hover::HoverMap, pointer::PointerId},
    prelude::*,
    render::{
        Render, RenderApp,
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache},
        renderer::{RenderQueue, render_system},
        texture::GpuImage,
    },
    window::PrimaryWindow,
};
use emblems::Prose;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use unfolded_hover::Hover;
use unfolded_tabletop::{
    camera::{
        Framing, InScene, Orbit, Scene, TableCamera, TableCameraPlugin, TableCameraSystems,
        TableFacts,
    },
    label_anchor::{LabelAnchor, LabelAnchorPlugin},
    token,
};
mod emblems;
mod highlights;
mod operation;
mod popover;
mod preview;
pub mod seat;
mod status;
mod top;
mod victory;

#[derive(Clone)]
pub struct Options {
    pub art: PathBuf,
    pub cards: PathBuf,
    pub save: PathBuf,
    pub recover: Option<PathBuf>,
    pub socket: Option<PathBuf>,
    pub agent: bool,
    pub seat_size: UVec2,
}
#[derive(Resource)]
struct Play {
    session: Option<Session>,
    options: Options,
    error: Option<String>,
    ready_frames: u8,
}
impl Play {
    fn game(&self) -> Game {
        self.session
            .as_ref()
            .map_or_else(Game::opening, |s| s.game().clone())
    }
}
#[derive(Resource)]
struct Art {
    verified: Verified,
    images: BTreeMap<String, Handle<Image>>,
    font: Handle<Font>,
    scene: Handle<WorldAsset>,
    terror: Handle<WorldAsset>,
    terror_clip: Handle<AnimationClip>,
    labels: Vec<LabelSpec>,
}
/// The authored Terror bell loop, played as one graph node by every token.
#[derive(Resource)]
struct TerrorLoop {
    clip: Handle<AnimationClip>,
    graph: Handle<AnimationGraph>,
    node: AnimationNodeIndex,
}
/// A space's Terror token. `phase` is its fixed place in the loop, as a fraction.
#[derive(Component)]
struct TerrorToken {
    phase: f64,
}
/// Marks the token's glTF animation player once it plays the loop for `space`.
#[derive(Component)]
struct TerrorPlayer(String);
#[derive(serde::Deserialize, Clone)]
struct LabelSpec {
    space: String,
    bounds: [f32; 4],
    #[serde(default)]
    pieces: Option<[f32; 2]>,
}
/// What one shared hover lifecycle can disclose: a victory standing, an element of
/// the top display, or a choice's preview. Serialized for the seat as the faction,
/// the element's name or the choice id.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(untagged)]
enum Target {
    Standing(Faction),
    Top(top::Item),
    Choice(String),
}
#[derive(Resource, Default)]
struct Interface {
    hover: Hover<Target>,
    status: Hover<String>,
    /// Set by a click; the next frame records the pointer's position in `rested`.
    clicked: bool,
    /// Where the pointer rested at its last click. A choice's preview stays closed
    /// until the pointer moves, so the button that replaces the clicked one does not
    /// open a card under a pointer that has not left it.
    rested: Option<Vec2>,
    reader: bool,
    notice: String,
    dirty: bool,
}
#[derive(Component)]
struct UiRoot;
#[derive(Component)]
struct StandingText(Faction);
#[derive(Component)]
struct PieceVisual;
/// A 26July group's token: removal mode marks it, and a click on it opens its piece
/// popover (#73). `key` is `space:group` as removal ids name it; `base` is the region
/// top it stands on.
#[derive(Component)]
struct OwnPiece {
    key: String,
    base: f32,
}
#[derive(Component, Clone)]
struct Control {
    name: String,
    choice: bool,
    action: Action,
}
#[derive(Clone)]
enum Action {
    Standing(Faction),
    Top(top::Item),
    Close,
    Reader,
    Space(String),
    Decision(Faction, u64),
    Unsupported(String),
    Quit,
    /// A space popover row: shows that action's preview, and decides nothing.
    Row(String),
    /// A space popover's button: submits its choice like the panel's `Decision`.
    Commit(Faction, u64, String),
    /// A 26July piece on the map: opens its popover in removal mode, else nothing.
    Piece(String),
    /// The piece popover's stepper: one fewer or one more; decides nothing.
    Quantity(i8),
    /// The piece popover's Cancel: closes it; decides nothing.
    Dismiss,
}
#[derive(Resource, Clone)]
struct RenderReady {
    images: Vec<AssetId<Image>>,
    ready: Arc<AtomicBool>,
    /// Whether every render pipeline compiled, which admission waits for. The agent
    /// seat finds this for a capture itself.
    pipelines: Arc<AtomicBool>,
}
#[derive(Resource)]
struct Water {
    frames: Vec<Handle<Image>>,
    material: Handle<StandardMaterial>,
    frame: usize,
}
const WIDTH: f32 = 1920.;
const HEIGHT: f32 = 1280.;
// The authored region mesh, labels and group centers share the same map coordinates.
fn map_at(x: f32, y: f32, height: f32) -> Vec3 {
    Vec3::new((x - 700.) / 100., height, (y - 600.) / 100.)
}

pub fn run(options: Options) -> Result<(), String> {
    if options.agent {
        return seat::run(options);
    }
    let mut app = build(options, false)?;
    let result = app.run();
    if result != AppExit::Success {
        return Err("Native application exited unsuccessfully".into());
    }
    Ok(())
}
fn build(options: Options, windowless: bool) -> Result<App, String> {
    let verified = assets::inspect(&options.art, &options.cards).map_err(|e| format!("{e:#}"))?;
    let mut app = App::new();
    let plugins = DefaultPlugins.set(AssetPlugin {
        file_path: verified.root.to_string_lossy().into_owned(),
        ..default()
    });
    if windowless {
        app.add_plugins(
            unfolded_tabletop::capture::windowless(plugins)
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
        );
    } else {
        app.add_plugins(plugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cuba Libre Unfolded — tactical candidate".into(),
                resolution: (1440, 960).into(),
                ..default()
            }),
            ..default()
        }));
    }
    let mut images = BTreeMap::new();
    for (key, bytes) in &verified.files {
        if key.ends_with(".png") {
            let mut decoded = Image::from_buffer(
                bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::linear(),
                RenderAssetUsages::default(),
            )
            .map_err(|e| e.to_string())?;
            if key.starts_with("water/") {
                decoded.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..ImageSamplerDescriptor::linear()
                });
            }
            images.insert(
                key.clone(),
                app.world_mut().resource_mut::<Assets<Image>>().add(decoded),
            );
        }
    }
    let font = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(verified.files["Lato-Regular.ttf"].clone()));
    let server = app.world().resource::<AssetServer>();
    let scene = server.load("atlas-terrain-map.glb#Scene0");
    let terror = server.load("atlas-terror-token.glb#Scene0");
    let terror_clip = server.load("atlas-terror-token.glb#Animation0");
    let (graph, node) = AnimationGraph::from_clip(terror_clip.clone());
    let graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(graph);
    #[derive(serde::Deserialize)]
    struct Labels {
        labels: Vec<LabelSpec>,
    }
    let labels: Labels = serde_json::from_slice(&verified.files["terrain-labels.json"])
        .map_err(|e| e.to_string())?;
    if (0..24).any(|frame| !images.contains_key(&format!("water/water-c-{frame:05}.png"))) {
        return Err("Selected Trade wind strokes animation requires all 24 authored frames".into());
    }
    let ready = RenderReady {
        images: images.values().map(Handle::id).collect(),
        ready: Arc::new(AtomicBool::new(false)),
        pipelines: Arc::new(AtomicBool::new(false)),
    };
    app.sub_app_mut(RenderApp)
        .insert_resource(ready.clone())
        .add_systems(Render, render_ready.after(render_system));
    app.insert_resource(ready)
        .insert_resource(Art {
            verified,
            images,
            font,
            scene,
            terror,
            terror_clip: terror_clip.clone(),
            labels: labels.labels,
        })
        .insert_resource(TerrorLoop {
            clip: terror_clip,
            graph,
            node,
        })
        .insert_resource(Play {
            session: None,
            options,
            error: None,
            ready_frames: 0,
        })
        .insert_resource(Interface {
            dirty: true,
            ..default()
        })
        // art/stage/preview-stage.blend mirrors this home framing; update it too.
        .insert_resource(Orbit::new(Framing::new(Vec3::new(0.35, 0., 0.35), 9.6)))
        // Atlas SVG map rim fill; the viewport continues the authored sea.
        .insert_resource(ClearColor(Color::srgb_u8(216, 231, 227)))
        .add_plugins((
            MeshPickingPlugin,
            LabelAnchorPlugin,
            TableCameraPlugin::<()>::default(),
        ))
        .configure_sets(
            Update,
            TableCameraSystems
                .after(camera_facts_and_viewport)
                .before(status::hover),
        )
        .init_resource::<highlights::Targeting>()
        .init_resource::<popover::Popover>()
        .add_systems(Startup, (setup, highlights::setup, popover::setup))
        .add_systems(
            Update,
            (
                admit,
                highlights::refresh,
                highlights::show,
                sync_pieces,
                highlights::markers,
                popover::hover_piece,
                bind_spaces,
                hover_disclosures,
                draw_ui,
                camera_facts_and_viewport,
                status::hover,
                status::draw,
                victory::draw,
                top::draw,
                preview::draw,
                popover::draw,
                popover::place,
                animate_water,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (animate_terror.before(AnimationSystems), emblems::ink),
        );
    Ok(app)
}
fn render_ready(
    images: Res<RenderAssets<GpuImage>>,
    queue: Res<RenderQueue>,
    ready: Res<RenderReady>,
    pipelines: Res<PipelineCache>,
) {
    ready.pipelines.store(
        pipelines.waiting_pipelines().next().is_none()
            && pipelines
                .pipelines()
                .all(|p| matches!(p.state, CachedPipelineState::Ok(_))),
        Ordering::Release,
    );
    if ready.images.iter().all(|id| images.get(*id).is_some()) {
        let flag = ready.ready.clone();
        queue.on_submitted_work_done(move || flag.store(true, Ordering::Release));
    }
}
/// art/stage/preview-stage.blend mirrors this viewport, light, sea and region
/// tops, and `sync_pieces`' placement; update it when they change.
fn setup(
    mut commands: Commands,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        TableCamera,
        // Renders the seat's image under the UI camera; nothing without one.
        unfolded_tabletop::capture::OnSurface,
        // Preserve the authored unlit sea color against the UI's matching rim.
        Tonemapping::None,
        Camera {
            viewport: Some(Viewport {
                physical_position: UVec2::new(40, 176),
                physical_size: UVec2::new(1300, 684),
                ..default()
            }),
            ..default()
        },
        Transform::from_xyz(-0.2, 8.84, 5.17).looking_at(Vec3::new(-0.2, 0., -0.5), Vec3::Y),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.1, -0.5, 0.)),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 150.,
        ..default()
    });
    // Only admitted runtime frames participate; proposal assets stay outside the
    // manifest. Playback is ordinary texture selection, with no custom shader.
    let frames: Vec<_> = (0..24)
        .map(|frame| art.images[&format!("water/water-c-{frame:05}.png")].clone())
        .collect();
    if let Some(first) = frames.first() {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(first.clone()),
            unlit: true,
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(200. / 6.)),
            ..default()
        });
        commands.spawn((
            Name::new("Authored sea"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(200., 200.))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0., -0.06, 0.),
            Pickable::IGNORE,
        ));
        commands.insert_resource(Water {
            frames,
            material,
            frame: 0,
        });
    }
    commands.spawn((WorldAssetRoot(art.scene.clone()), Transform::default()));
    let game = Game::prepared();
    for label in &art.labels {
        let space = game.spaces.iter().find(|s| s.id == label.space).unwrap();
        let [x, y, w, h] = label.bounds;
        let top = match space.terrain {
            Terrain::City => 0.24,
            Terrain::EconomicCenter => 0.28,
            _ => 0.14,
        };
        let at = map_at(x + w / 2., y + h / 2., top + 0.004);
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(w / 100., h / 100.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(
                    art.images[&format!("atlas-terrain-label-{}.png", space.id)].clone(),
                ),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(at),
            Pickable::IGNORE,
        ));
    }
}
fn sync_pieces(
    mut commands: Commands,
    play: Res<Play>,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    roots: Query<Entity, With<PieceVisual>>,
    mut previous: Local<Vec<crate::Space>>,
) {
    let game = play.game();
    if *previous == game.spaces {
        return;
    }
    *previous = game.spaces.clone();
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    for label in &art.labels {
        let space = game.spaces.iter().find(|s| s.id == label.space).unwrap();
        let [x, y, w, h] = label.bounds;
        let top = match space.terrain {
            Terrain::City => 0.24,
            Terrain::EconomicCenter => 0.28,
            _ => 0.14,
        };
        let at = map_at(x + w / 2., y + h / 2., top + 0.004);
        let piece_at = label
            .pieces
            .map_or(at + Vec3::new(0., 0., 0.55), |[x, y]| map_at(x, y, top));
        for (i, piece) in space.pieces.iter().enumerate() {
            let mesh = match piece.kind {
                Kind::Troops | Kind::Police => Mesh::from(Cuboid::new(0.18, 0.18, 0.18)),
                Kind::Guerrilla => Cylinder::new(0.10, 0.24)
                    .mesh()
                    .resolution(8)
                    .build()
                    .with_duplicated_vertices()
                    .with_computed_flat_normals(),
                _ => Mesh::from(Cylinder::new(0.15, 0.075)),
            };
            let color = match piece.faction {
                Faction::Government => {
                    if piece.kind == Kind::Police {
                        Color::srgb_u8(143, 185, 210)
                    } else {
                        Color::srgb_u8(49, 111, 156)
                    }
                }
                Faction::M26 => Color::srgb_u8(180, 60, 54),
                Faction::Directorio => Color::srgb_u8(228, 189, 70),
                Faction::Syndicate => Color::srgb_u8(65, 117, 87),
            };
            let height = match piece.kind {
                Kind::Guerrilla => 0.12,
                Kind::Troops | Kind::Police => 0.09,
                _ => 0.0375,
            };
            let token = token::spawn_body(
                &mut commands,
                &mut meshes,
                &mut materials,
                token::Body {
                    name: format!(
                        "{} {:?} {:?} {:?}",
                        space.id, piece.faction, piece.kind, piece.state
                    ),
                    mesh,
                    color,
                    transform: Transform::from_xyz(
                        piece_at.x + (i as f32 - (space.pieces.len() - 1) as f32 / 2.) * 0.40,
                        top + height,
                        piece_at.z,
                    ),
                },
            );
            commands
                .entity(token)
                .insert((PieceVisual, status::SpaceTarget(space.id.clone())))
                .remove::<Pickable>();
            if piece.faction == Faction::M26 {
                let key = format!(
                    "{}:{}",
                    space.id,
                    crate::removal_group(piece.kind, piece.state)
                );
                commands
                    .entity(token)
                    .insert((
                        OwnPiece {
                            key: key.clone(),
                            base: top,
                        },
                        Control {
                            name: format!("own-piece:{key}"),
                            choice: false,
                            action: Action::Piece(key),
                        },
                    ))
                    .observe(click);
            }
            let mark = match piece.state {
                State::Active => Some("atlas-active-star.png"),
                State::Closed => Some("atlas-closed-slash.png"),
                _ => None,
            };
            if let Some(mark) = mark {
                token::add_face(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    token,
                    token::Face {
                        mesh: Plane3d::default().mesh().size(0.17, 0.17).build(),
                        image: art.images[mark].clone(),
                        alpha: AlphaMode::Blend,
                        transform: Transform::from_xyz(0., height + 0.002, 0.),
                    },
                );
            }
            commands.spawn((
                count(
                    piece.count.to_string(),
                    token,
                    Vec3::new(0., height + 0.06, 0.),
                ),
                status::SpaceTarget(space.id.clone()),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: 21.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.08, 0.08, 0.06)),
                GlobalZIndex(-1),
            ));
        }
        if space.terror > 0 {
            // One authored bell per space; the count carries the quantity. Its glTF
            // meshes stay pickable, and hover walks up to this SpaceTarget.
            let index = game.spaces.iter().position(|s| s.id == space.id).unwrap();
            let token = commands
                .spawn((
                    Name::new(format!("{} Terror", space.id)),
                    WorldAssetRoot(art.terror.clone()),
                    Transform::from_xyz(
                        piece_at.x + if space.id == "oriente" { 0.38 } else { 0. },
                        top,
                        piece_at.z + if space.id == "oriente" { 0. } else { 0.36 },
                    ),
                    TerrorToken {
                        // Golden-ratio steps keep neighbouring bells out of unison.
                        phase: (index as f64 * 0.618_033_988_75).fract(),
                    },
                    PieceVisual,
                    status::SpaceTarget(space.id.clone()),
                ))
                .id();
            commands.spawn((
                count(space.terror.to_string(), token, Vec3::new(0., 0.33, 0.)),
                status::SpaceTarget(space.id.clone()),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: 21.0.into(),
                    ..default()
                },
                TextColor(Color::srgb(0.08, 0.08, 0.06)),
                GlobalZIndex(-1),
            ));
        }
    }
}
/// A count of pieces or Terror, kept over its token.
#[derive(Component)]
struct Count;

/// A count over its token. It stays pickable, so hovering it hovers its space, and it
/// is part of the scene, so the wheel and a drag over it still move the camera.
fn count(text: String, token: Entity, offset: Vec3) -> impl Bundle {
    (
        Count,
        InScene,
        Text::new(text),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        LabelAnchor::entity(token, offset),
    )
}
/// Bind each spawned bell's glTF player to the loop and pose it from the shared
/// clock. A token respawned by a state change keeps its place in the loop, and the
/// development seat's manual clock drives it exactly like desktop time.
fn animate_terror(
    mut commands: Commands,
    time: Res<Time>,
    terror: Res<TerrorLoop>,
    clips: Res<Assets<AnimationClip>>,
    tokens: Query<(&TerrorToken, &status::SpaceTarget)>,
    parents: Query<&ChildOf>,
    mut players: Query<(Entity, &mut AnimationPlayer, Has<TerrorPlayer>)>,
) {
    let Some(length) = clips.get(&terror.clip).map(AnimationClip::duration) else {
        return;
    };
    for (entity, mut player, bound) in &mut players {
        let Some((token, space)) = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| tokens.get(ancestor).ok())
        else {
            continue;
        };
        if !bound {
            commands.entity(entity).insert((
                AnimationGraphHandle(terror.graph.clone()),
                TerrorPlayer(space.0.clone()),
            ));
            player.play(terror.node).pause();
        }
        let seek = (time.elapsed_secs_f64() + token.phase * f64::from(length))
            .rem_euclid(f64::from(length));
        if let Some(active) = player.animation_mut(terror.node) {
            active.set_seek_time(seek as f32);
        }
    }
}
fn animate_water(
    time: Res<Time>,
    water: Option<ResMut<Water>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(mut water) = water else {
        return;
    };
    // The development seat advances this same clock explicitly. Decoration is
    // never an unsettled game action and captures do not advance it.
    let frame = (time.elapsed_secs_f64() * 6.).floor() as usize % water.frames.len();
    if frame != water.frame {
        if let Some(mut material) = materials.get_mut(&water.material) {
            material.base_color_texture = Some(water.frames[frame].clone());
        }
        water.frame = frame;
    }
}
#[allow(clippy::too_many_arguments)]
fn admit(
    mut play: ResMut<Play>,
    art: Res<Art>,
    server: Res<AssetServer>,
    scenes: Res<Assets<WorldAsset>>,
    clips: Res<Assets<AnimationClip>>,
    fonts: Res<Assets<Font>>,
    mut context: ResMut<bevy::text::FontCx>,
    ready: Res<RenderReady>,
    mut ui: ResMut<Interface>,
) {
    if play.error.is_some() || play.session.is_some() {
        return;
    }
    let report = unfolded_tabletop::loading::required_assets(
        &server,
        &[
            art.scene.id().untyped(),
            art.terror.id().untyped(),
            art.terror_clip.id().untyped(),
        ],
        |id| {
            if id == art.terror_clip.id().untyped() {
                clips.get(&art.terror_clip).is_some()
            } else {
                scenes.contains(id.typed::<WorldAsset>())
            }
        },
    );
    if !report.failures.is_empty() {
        play.error = Some(report.failures.join("; "));
        ui.dirty = true;
        return;
    }
    let font_ready = fonts
        .get(&art.font)
        .is_some_and(|font| unfolded_tabletop::loading::font_registered(font, &mut context));
    if !report.loaded
        || !font_ready
        || !ready.ready.load(Ordering::Acquire)
        || !ready.pipelines.load(Ordering::Acquire)
    {
        play.ready_frames = 0;
        return;
    }
    play.ready_frames += 1;
    if play.ready_frames < 5 {
        return;
    }
    // Recheck file-backed scene identity after load, before creating/replacing a save.
    let result = assets::inspect(&art.verified.root, &play.options.cards)
        .map_err(|e| format!("{e:#}"))
        .and_then(|_| {
            Session::open(&play.options.save, play.options.recover.as_deref())
                .map_err(|e| e.to_string())
        });
    match result {
        Ok(session) => play.session = Some(session),
        Err(error) => play.error = Some(error),
    };
    ui.dirty = true;
}
fn bind_spaces(
    mut commands: Commands,
    art: Res<Art>,
    nodes: Query<(Entity, &Name), Without<Control>>,
) {
    for (entity, name) in &nodes {
        if art.labels.iter().any(|l| l.space == name.as_str()) {
            commands
                .entity(entity)
                .insert(Control {
                    name: format!("space:{}", name.as_str()),
                    choice: false,
                    action: Action::Space(name.as_str().into()),
                })
                .observe(click);
        }
    }
}
/// One dwell for every hover disclosure: victory standings and the top display.
fn hover_disclosures(
    hover: Res<HoverMap>,
    controls: Query<&Control>,
    pointers: Query<(&PointerId, &bevy::picking::pointer::PointerLocation)>,
    time: Res<Time>,
    mut ui: ResMut<Interface>,
) {
    let at = pointers
        .iter()
        .find(|(id, _)| **id == PointerId::Mouse)
        .and_then(|(_, l)| l.location.as_ref().map(|l| l.position));
    if std::mem::take(&mut ui.clicked) {
        ui.rested = at;
    } else if ui.rested.is_some() && ui.rested != at {
        ui.rested = None;
    }
    let rested = ui.rested.is_some();
    let target = hover.get(&PointerId::Mouse).and_then(|hits| {
        hits.keys().find_map(|e| {
            let control = controls.get(*e).ok()?;
            match control.action {
                Action::Standing(f) => Some(Target::Standing(f)),
                Action::Top(item) => Some(Target::Top(item)),
                Action::Decision(..) | Action::Unsupported(_) if !rested => {
                    Some(Target::Choice(control.name.clone()))
                }
                _ => None,
            }
        })
    });
    let eligible = !ui.reader;
    ui.hover
        .update(target, eligible, time.delta(), Duration::from_millis(400));
}

fn click(
    mut event: On<PointerClick>,
    controls: Query<&Control>,
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
    mut exit: MessageWriter<AppExit>,
    targeting: Res<highlights::Targeting>,
    mut popover: ResMut<popover::Popover>,
) {
    event.propagate(false);
    if event.button != bevy::picking::pointer::PointerButton::Primary || play.session.is_none() {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    // A popover's button submits its choice exactly as the panel's button does.
    let control = &match &control.action {
        Action::Commit(owner, revision, choice) => Control {
            name: choice.clone(),
            choice: true,
            action: Action::Decision(*owner, *revision),
        },
        _ => control.clone(),
    };
    ui.clicked = true;
    if ui.reader && !matches!(control.action, Action::Close | Action::Quit) {
        return;
    }
    match &control.action {
        // Hover disclosures never change a decision.
        Action::Standing(_) | Action::Top(_) => return,
        // A region click opens or closes a space popover while Terror targets are
        // chosen, and does nothing otherwise; it never decides anything itself.
        Action::Space(space) => {
            popover.click_space(space, &targeting);
            return;
        }
        Action::Row(action) => {
            popover.choose_row(action);
            return;
        }
        Action::Piece(key) => {
            popover.click_piece(key, &targeting);
            return;
        }
        Action::Quantity(step) => {
            popover.step(*step, &targeting);
            return;
        }
        Action::Dismiss => {
            popover.close();
            return;
        }
        Action::Commit(..) => return,
        Action::Close => {
            ui.reader = false;
        }
        Action::Reader => {
            ui.reader = true;
        }
        Action::Decision(owner, rev) => {
            let previous = play.game().observe();
            match play
                .session
                .as_mut()
                .unwrap()
                .submit(*owner, *rev, &control.name)
            {
                Ok(()) => {
                    ui.notice = if control.name.ends_with(":authorize") {
                        if let Some(crate::Transfer::Authorizing {
                            giver,
                            recipient,
                            amount,
                        }) = previous.transfer
                        {
                            format!(
                                "{} gave {amount} to {}. Saved.",
                                giver.name(),
                                recipient.name()
                            )
                        } else {
                            "Decision saved.".into()
                        }
                    } else if control.name.ends_with(":decline") {
                        format!("{} declined. No resources moved.", owner.name())
                    } else if control.name.ends_with(":wait") {
                        format!(
                            "Waiting for {}. Save / quit keeps this decision.",
                            owner.name()
                        )
                    } else if control.name.ends_with(":kidnap") {
                        "Kidnap opened: nothing rolled. Roll the die or cancel. Saved.".into()
                    } else if control.name.ends_with(":kidnap:cancel") {
                        "Kidnap cancelled: nothing rolled and nothing changed. Saved.".into()
                    } else if control.name.ends_with(":kidnap:roll") {
                        let view = play.game().observe();
                        let roll = view.kidnap.as_ref().unwrap();
                        format!(
                            "Kidnap rolled {} · transferred {}.\nGovernment {} · 26July {}. Saved.",
                            roll.die, roll.proceeds, view.resources[0], view.resources[1]
                        )
                    } else if control.name.contains(":remove:") {
                        let n = control.name.rsplit(':').next().unwrap_or("1");
                        format!(
                            "Removed {n} own piece{} to Available. Removal stays open until Done; earlier Terror, support shifts and Kidnap results remain. Saved.",
                            if n == "1" { "" } else { "s" }
                        )
                    } else if control.name.ends_with(":remove") {
                        "Removal mode: click a ringed 26July piece on the map. Saved.".into()
                    } else if control.name.ends_with(":done-removal") {
                        "Removal mode closed; removed pieces stay in Available. Saved.".into()
                    } else if control.name.contains(":terror:") {
                        "Terror saved: paid 1 Resource, flipped 1 Underground Guerrilla to Active, placed a Terror marker and shifted 1 level toward Active Opposition."
                            .into()
                    } else if control.name.ends_with(":pass") {
                        "26July passed: +1 Resource and stays Eligible. Directorio is 1st Eligible. Saved.".into()
                    } else if control.name.ends_with(":terror") {
                        "Terror chosen: 26July is executing, and transfers are open. Saved.".into()
                    } else if control.name.ends_with(":back-to-choice") {
                        "Back at the action choice. Nothing had happened; any pending request is withdrawn. Saved.".into()
                    } else if control.name.ends_with(":end-terror") {
                        format!(
                            "Terror ended: {} recorded. Saved.",
                            if previous.kidnap.is_some() {
                                "Operation with Special Activity"
                            } else {
                                "Operation Only"
                            }
                        )
                    } else {
                        "Decision saved; resources unchanged.".into()
                    };
                }
                Err(e) => ui.notice = e.to_string(),
            };
        }
        Action::Unsupported(reason) => ui.notice = reason.clone(),
        Action::Quit => {
            exit.write(AppExit::Success);
        }
    }
    ui.dirty = true;
}
fn layout(window: &Window) -> (f32, Vec2) {
    let scale = (window.width() / WIDTH).min(window.height() / HEIGHT);
    (
        scale,
        Vec2::new(0., (window.height() - HEIGHT * scale) / 2.),
    )
}
/// The authored map rectangle, in logical window pixels: the only part of the window
/// that shows the scene. Extra width on wide windows goes to the map.
fn map_rect(window: &Window) -> Rect {
    let (scale, offset) = layout(window);
    let min = offset + Vec2::new(40., 176.) * scale;
    let size = Vec2::new(window.width() - 620. * scale, 684. * scale);
    Rect {
        min,
        max: min + size,
    }
}
/// The camera's viewport: the map rectangle, in physical pixels.
fn map_viewport(window: &Window) -> Viewport {
    let map = map_rect(window);
    let physical_position = (map.min * window.scale_factor()).round().as_uvec2();
    let physical_end = (map.max * window.scale_factor()).round().as_uvec2();
    Viewport {
        physical_position,
        physical_size: physical_end - physical_position,
        ..default()
    }
}
/// What the table says to the shared camera: the reader and an unopened session
/// block it, and only the map shows the scene.
fn camera_facts(window: &Window, play: &Play, ui: &Interface) -> TableFacts {
    TableFacts {
        blocks_camera: ui.reader || play.session.is_none(),
        scene: Scene::Within(map_rect(window)),
        ..default()
    }
}
fn camera_facts_and_viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    play: Res<Play>,
    ui: Res<Interface>,
    mut table: ResMut<TableFacts>,
    mut cameras: Query<&mut Camera, With<TableCamera>>,
) {
    *table = camera_facts(&window, &play, &ui);
    for mut camera in &mut cameras {
        camera.viewport = Some(map_viewport(&window));
    }
}
#[allow(clippy::too_many_arguments)] // Authored rectangle, type size and letterboxing coordinates.
fn text(
    commands: &mut Commands,
    art: &Art,
    content: impl Into<Prose>,
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    scale: f32,
    offset: Vec2,
    z: i32,
) -> Entity {
    let content: Prose = content.into();
    let font = TextFont {
        font: art.font.clone().into(),
        font_size: (size * scale).into(),
        ..default()
    };
    let ink = TextColor(Color::srgb(0.12, 0.14, 0.12));
    let segments = emblems::segments(&content.text, content.naming, content.red);
    let (first, rest) = match segments.split_first() {
        Some((emblems::Segment::Words(words), rest)) => (words.clone(), rest),
        _ => (String::new(), &segments[..]),
    };
    let root = commands
        .spawn((
            UiRoot,
            Text::new(first),
            emblems::Plain(content.text),
            font.clone(),
            ink,
            Node {
                position_type: PositionType::Absolute,
                left: px(offset.x + x * scale),
                top: px(offset.y + y * scale),
                width: px(w * scale),
                ..default()
            },
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id();
    emblems::spawn_rest(
        commands,
        art,
        root,
        rest,
        &font,
        ink,
        size * scale * emblems::HEIGHT,
    );
    root
}
fn picture(
    commands: &mut Commands,
    art: &Art,
    key: &str,
    rect: [f32; 4],
    scale: f32,
    offset: Vec2,
    z: i32,
) -> Entity {
    let [x, y, w, h] = rect;
    commands
        .spawn((
            UiRoot,
            ImageNode::new(art.images[key].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            Node {
                position_type: PositionType::Absolute,
                left: px(offset.x + x * scale),
                top: px(offset.y + y * scale),
                width: px(w * scale),
                height: px(h * scale),
                ..default()
            },
            GlobalZIndex(z),
            Pickable::IGNORE,
        ))
        .id()
}
#[allow(clippy::too_many_arguments)]
fn control(
    commands: &mut Commands,
    art: &Art,
    name: &str,
    label: &str,
    action: Action,
    rect: [f32; 4],
    scale: f32,
    offset: Vec2,
    choice: bool,
    z: i32,
) {
    let node = picture(
        commands,
        art,
        if choice {
            "atlas-button-primary.png"
        } else {
            "atlas-button.png"
        },
        rect,
        scale,
        offset,
        z,
    );
    commands
        .entity(node)
        .remove::<Pickable>()
        .insert(Control {
            name: name.into(),
            choice,
            action,
        })
        .observe(click);
    // The primary button is red: 26July keeps its name there (emblems::Prose::red).
    let label = Prose::from(label);
    let words = text(
        commands,
        art,
        if choice { label.on_red() } else { label },
        rect[0] + 12.,
        rect[1] + 10.,
        rect[2] - 24.,
        19.,
        scale,
        offset,
        z + 1,
    );
    commands
        .entity(words)
        .insert(TextColor(Color::srgb(0.98, 0.97, 0.92)));
}
#[allow(clippy::too_many_arguments)]
fn draw_ui(
    mut commands: Commands,
    art: Res<Art>,
    play: Res<Play>,
    mut ui: ResMut<Interface>,
    roots: Query<Entity, With<UiRoot>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut previous_size: Local<Vec2>,
) {
    let size = Vec2::new(window.width(), window.height());
    if !ui.dirty && *previous_size == size {
        return;
    }
    *previous_size = size;
    ui.dirty = false;
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    let (s, left) = layout(&window);
    let extra = window.width() - WIDTH * s;
    let right = left + Vec2::new(extra, 0.);
    let centered = left + Vec2::new(extra / 2., 0.);
    let o = if ui.reader { centered } else { left };
    let view = play.game().observe();
    // The desk surrounds the native viewport. It must not paint over the
    // camera's pixels; all four strips consume the authored flat desk asset.
    let map_top = left.y + 176. * s;
    let map_bottom = left.y + 860. * s;
    for rect in if ui.reader {
        vec![[0., 0., window.width(), window.height()]]
    } else {
        vec![
            [0., 0., window.width(), map_top],
            [0., map_bottom, window.width(), window.height() - map_bottom],
            [0., map_top, 40. * s, map_bottom - map_top],
            [
                window.width() - 580. * s,
                map_top,
                580. * s,
                map_bottom - map_top,
            ],
        ]
    } {
        picture(
            &mut commands,
            &art,
            "atlas-layout-background.png",
            rect,
            1.,
            Vec2::ZERO,
            0,
        );
    }
    if !ui.reader {
        // The owner retired the "CUBA LIBRE" title in #46: its corner holds Aid and
        // the US Alliance (top::header).
        for (key, rect, offset) in [
            (
                "atlas-layout-map-fill.png",
                [40., 136., 1300. + extra / s, 40.],
                left,
            ),
            (
                "atlas-layout-map-fill.png",
                [40., 860., 1300. + extra / s, 80.],
                left,
            ),
            ("atlas-layout-map-header.png", [40., 136., 1300., 40.], left),
            ("atlas-layout-map-footer.png", [40., 860., 1300., 80.], left),
            (
                "atlas-layout-operation.png",
                [1380., 280., 500., 660.],
                right,
            ),
            ("atlas-layout-card.png", [40., 964., 600., 264.], left),
            ("atlas-layout-session.png", [1380., 964., 500., 264.], right),
        ] {
            let layer = if matches!(
                key,
                "atlas-layout-map-header.png" | "atlas-layout-map-footer.png"
            ) {
                2
            } else {
                1
            };
            picture(&mut commands, &art, key, rect, s, offset, layer);
        }
        victory::header(&mut commands, &art, &view, s, centered);
        top::header(&mut commands, &art, &view, s, left, right);
    }
    if ui.reader {
        picture(
            &mut commands,
            &art,
            "atlas-victory-reader.png",
            [0., 0., WIDTH, HEIGHT],
            s,
            o,
            1,
        );
        let factions = |flags: [bool; 4]| {
            let names = Faction::ALL
                .into_iter()
                .filter(|faction| flags[faction.index()])
                .map(Faction::name)
                .collect::<Vec<_>>();
            if names.is_empty() {
                "none".into()
            } else {
                names.join(", ")
            }
        };
        let effects = |values: &[String]| {
            if values.is_empty() {
                "none".into()
            } else {
                values.join(", ")
            }
        };
        text(
            &mut commands,
            &art,
            format!(
                "Capabilities: {} · Momentum: {}\nEligible: {}\nPassed: {}",
                effects(&view.capabilities),
                effects(&view.momentum),
                factions(view.eligible),
                factions(view.passed)
            ),
            320.,
            290.,
            1200.,
            18.,
            s,
            o,
            4,
        );
        picture(
            &mut commands,
            &art,
            "card-17.png",
            [320., 380., 420., 602.1],
            s,
            o,
            3,
        );
        picture(
            &mut commands,
            &art,
            "card-18.png",
            [1000., 380., 420., 601.05],
            s,
            o,
            3,
        );
        control(
            &mut commands,
            &art,
            "close",
            "Close reader",
            Action::Close,
            [1400., 1110., 210., 48.],
            s,
            o,
            false,
            4,
        );
        text(
            &mut commands,
            &art,
            "17 Alberto Bayo — current; Event inactive",
            320.,
            1030.,
            600.,
            23.,
            s,
            o,
            4,
        );
        text(
            &mut commands,
            &art,
            "18 Pact of Caracas — preview; inactive",
            1000.,
            1030.,
            600.,
            23.,
            s,
            o,
            4,
        );
        text(
            &mut commands,
            &art,
            format!(
                "Played: {} · {} remain, including the preview",
                view.played_cards
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                view.draw_count
            ),
            320.,
            1070.,
            1250.,
            18.,
            s,
            o,
            4,
        );
    } else {
        operation::panel(&mut commands, &art, &view, s, right);
        let o = left;
        picture(
            &mut commands,
            &art,
            "card-17.png",
            [100., 1040., 108., 154.8],
            s,
            o,
            2,
        );
        picture(
            &mut commands,
            &art,
            "card-18.png",
            [366., 1040., 108., 154.55],
            s,
            o,
            2,
        );
        control(
            &mut commands,
            &art,
            "reader",
            "Read both cards",
            Action::Reader,
            [64., 1200., 500., 42.],
            s,
            o,
            false,
            2,
        );
        text(
            &mut commands,
            &art,
            format!(
                "17 current · 18 preview · {} cards in draw deck",
                view.draw_count
            ),
            64.,
            1003.,
            560.,
            16.,
            s,
            o,
            2,
        );
        let o = right;
        let status = play.error.clone().unwrap_or_else(|| {
            if play.session.is_none() {
                "Loading selected assets; no save created.".into()
            } else {
                format!(
                    "Saved revision {} · {} map / {} available\nTerror/Sabotage {} available",
                    view.revision,
                    view.map_piece_count,
                    view.available_piece_count,
                    view.terror_available
                )
            }
        });
        text(
            &mut commands,
            &art,
            format!("{status}\n{}", ui.notice),
            1404.,
            1020.,
            450.,
            18.,
            s,
            o,
            2,
        );
        control(
            &mut commands,
            &art,
            "quit",
            "Save / quit",
            Action::Quit,
            [1690., 1140., 166., 44.],
            s,
            o,
            false,
            2,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unfolded_tabletop::camera::Input;
    #[test]
    fn top_display_targets_serialize_as_the_seat_reads_them() {
        assert_eq!(
            serde_json::to_value(Target::Standing(Faction::M26)).unwrap(),
            "M26"
        );
        assert_eq!(
            serde_json::to_value(Target::Top(top::Item::UsAlliance)).unwrap(),
            "us-alliance"
        );
    }
    /// One frame of the shared camera at this table, from plain input values.
    fn drive(
        orbit: &mut Orbit,
        window: &Window,
        keys: &ButtonInput<KeyCode>,
        wheel_lines: f32,
        table: TableFacts,
    ) {
        orbit.drive(
            Input {
                keys,
                buttons: &ButtonInput::default(),
                window,
                wheel_lines,
                dt: 1. / 60.,
                pointer_over_ui: false,
                focus_on_button: false,
            },
            &table,
        );
    }
    #[test]
    fn camera_input_respects_authored_viewport_and_reader_capture() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("task.json");
        let play = Play {
            session: Some(Session::open(&save, None).unwrap()),
            options: Options {
                art: PathBuf::new(),
                cards: PathBuf::new(),
                save,
                recover: None,
                socket: None,
                agent: false,
                seat_size: UVec2::new(1920, 1280),
            },
            error: None,
            ready_frames: 5,
        };
        let mut ui = Interface::default();
        let mut window = Window {
            resolution: (1920, 1280).into(),
            focused: true,
            ..default()
        };
        let mut orbit = Orbit::new(Framing::new(Vec3::ZERO, 10.));
        let keys = ButtonInput::default();
        window.set_cursor_position(Some(Vec2::new(700., 500.)));
        drive(
            &mut orbit,
            &window,
            &keys,
            1.,
            camera_facts(&window, &play, &ui),
        );
        assert!(orbit.pose().distance < 10.);
        let inside = orbit.pose();
        window.set_cursor_position(Some(Vec2::new(1500., 500.)));
        drive(
            &mut orbit,
            &window,
            &keys,
            1.,
            camera_facts(&window, &play, &ui),
        );
        assert_eq!(orbit.pose(), inside);
        // Maximizing a 3:2 design onto wider screens must put the extra
        // horizontal room into the map, retaining the 40 px outer inset.
        for width in [2560, 3360] {
            window.resolution.set(width as f32, 1440.);
            let viewport = map_viewport(&window);
            assert_eq!(
                viewport.physical_position.x, 45,
                "maximized map has a wide left gap"
            );
            assert!(
                ((viewport.physical_position.x + viewport.physical_size.x) as f32
                    - (width as f32 - 652.5))
                    .abs()
                    <= 1.,
                "extra window width must extend the map toward its right-anchored panels"
            );
        }
        ui.reader = true;
        window.set_cursor_position(Some(Vec2::new(700., 500.)));
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::KeyW);
        drive(
            &mut orbit,
            &window,
            &keys,
            0.,
            camera_facts(&window, &play, &ui),
        );
        assert_eq!(orbit.pose(), inside);
    }
}
