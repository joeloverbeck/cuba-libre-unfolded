//! Selected Atlas presentation. Input travels through Bevy picking into the checked game seam.
use crate::{
    Choice, Faction, Game, Kind, Observation, State, Terrain, Transfer, TransferChoice,
    assets::{self, Verified},
    resources,
    save::Session,
};
use bevy::{
    app::AnimationSystems,
    asset::RenderAssetUsages,
    camera::Viewport,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
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
mod briefing;
mod choice;
pub mod die;
#[cfg(test)]
mod discovery;
mod eligibility;
mod emblems;
mod execution;
mod highlights;
mod palette;
mod popover;
pub mod seat;
mod status;
mod top;
mod transfer;
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
    /// Lato Bold, for the action choice's labels (FigCraft `Choice/*` styles) and the
    /// Kidnap result.
    bold: Handle<Font>,
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
    /// An action box of the eligibility indicator, by its seat name.
    Sequence(eligibility::Slot),
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
    /// The action choice's pinned option, keyboard focus and briefing ribbon.
    choice: choice::State,
    /// The transfer modal's counterpart, direction, typed amount and focus.
    transfer: transfer::State,
    /// The revision a Kidnap roll just created in this run, until the die's
    /// presentation starts it from the shared clock in the same frame (`die::present`).
    rolled: Option<u64>,
}
#[derive(Component)]
struct UiRoot;
#[derive(Component)]
struct StandingText(Faction);
#[derive(Component)]
struct PieceVisual;
/// An executing faction's token: removal mode marks it, and a click opens its piece
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
    /// A commit at the panel's foot: the action choice's for its pinned option, or EX-A's
    /// for its pinned or open step, and Kidnap's Cancel (#84). Its preview is the
    /// pinned tag, so the pointer on it opens nothing.
    Commit(Faction, u64),
    /// An option row at the action choice: pins (or unpins) its briefing.
    Pin(String),
    /// The BRIEFING ribbon during Terror: opens or closes the briefing tag.
    Ribbon,
    /// A briefing tab during Terror: "terror" or "terror:kidnap".
    Tab(String),
    /// An action box of the eligibility indicator: a hover disclosure only.
    Sequence(eligibility::Slot),
    Unsupported(String),
    /// Lands the Kidnap die at once.
    SkipDie,
    /// Closes the landed Kidnap die and returns the panel to Terror.
    ContinueDie,
    Quit,
    /// A space popover row: shows that action's preview, and decides nothing.
    Row(String),
    /// A control named apart from the choice it submits, as the panel's `Decision`
    /// does: a space popover's button, and EX-A's commit while it opens a pending
    /// request, whose choice TM-B's pending tab already carries (#84).
    Submit(Faction, u64, String),
    /// A 26July piece on the map: opens its popover in removal mode, else nothing.
    Piece(String),
    /// The piece popover's stepper: one fewer or one more; decides nothing.
    Quantity(i8),
    /// The piece popover's Cancel: closes it; decides nothing.
    Dismiss,
    /// A transfer modal control that decides nothing: a counterpart tab, the
    /// direction, a quick value, the amount field or the pending tab's Open.
    Transfer(transfer::Input),
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
/// The piece and Terror counts' type size in reference pixels. Like every interface
/// size it follows the window, not the desktop's scale factor (ADR 0002). When the game
/// fills the owner's 3840×2160 display it is about 26 pixels, the size the earlier fixed
/// 21 logical pixels had there through Xwayland's scale factor 2 (#50).
const COUNT_SIZE: f32 = 15.5;
/// The Operation panel's authored frame (`atlas-panel.png`) in reference pixels:
/// x, y, width, height.
const PANEL: [f32; 4] = [1380., 287., 500., 653.];
/// A faction's lower-case slug, as export file names and seat names spell it.
fn slug(faction: Faction) -> &'static str {
    match faction {
        Faction::Government => "government",
        Faction::M26 => "26july",
        Faction::Directorio => "directorio",
        Faction::Syndicate => "syndicate",
    }
}
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
        // The seat's surface is a logical primary window, whose size this would
        // log, so only the windowed table adds it.
        app.add_plugins(unfolded_tabletop::window_size::WindowSizeLogPlugin);
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
    let bold = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(verified.files["Lato-Bold.ttf"].clone()));
    let die_art = die::DieArt::load(&mut app, &verified.files["kidnap-die.json"])?;
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
            bold,
            scene,
            terror,
            terror_clip: terror_clip.clone(),
            labels: labels.labels,
        })
        .insert_resource(die_art)
        .init_resource::<die::Roll>()
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
        .insert_resource(ClearColor(palette::MAP))
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
        .add_observer(map_click_closes_tag)
        .add_systems(
            Update,
            (
                admit,
                highlights::refresh,
                highlights::show,
                (sync_pieces, scale_counts).chain(),
                highlights::markers,
                popover::hover_piece,
                bind_spaces,
                hover_disclosures,
                // The presented die takes Space and Enter, so the action choice and the
                // modal read the keyboard first and stand aside while it is shown.
                (choice::keys, transfer::keys, die::present).chain(),
                draw_ui,
                camera_facts_and_viewport,
                status::hover,
                status::draw,
                victory::draw,
                top::draw,
                popover::draw,
                popover::place,
                // Bevy chains at most 20 systems in one tuple.
                (
                    choice::draw,
                    eligibility::draw,
                    transfer::draw,
                    transfer::plates,
                )
                    .chain(),
                animate_water,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                (animate_terror, die::animate).before(AnimationSystems),
                emblems::ink,
            ),
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
        // Debanding needs tonemapping, so it is off too (#86).
        Tonemapping::None,
        DebandDither::Disabled,
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
    mut previous: Local<Option<(Faction, Vec<crate::Space>)>>,
) {
    let game = play.game();
    let executor = game.executing().unwrap_or(Faction::M26);
    if previous
        .as_ref()
        .is_some_and(|(faction, spaces)| *faction == executor && *spaces == game.spaces)
    {
        return;
    }
    *previous = Some((executor, game.spaces.clone()));
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
                        Color::srgb_u8(143, 185, 210) // own colour: piece palette (art/atlas/README.md)
                    } else {
                        Color::srgb_u8(49, 111, 156) // own colour: piece palette (art/atlas/README.md)
                    }
                }
                Faction::M26 => Color::srgb_u8(180, 60, 54), // own colour: piece palette (art/atlas/README.md)
                Faction::Directorio => Color::srgb_u8(228, 189, 70), // own colour: piece palette (art/atlas/README.md)
                Faction::Syndicate => Color::srgb_u8(65, 117, 87), // own colour: piece palette (art/atlas/README.md)
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
            if piece.faction == executor {
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
                count_font(&art),
                TextColor(Color::srgb(0.08, 0.08, 0.06)), // own colour: Atlas desk map counts (foundation 4)
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
                count_font(&art),
                TextColor(Color::srgb(0.08, 0.08, 0.06)), // own colour: Atlas desk map counts (foundation 4)
                GlobalZIndex(-1),
            ));
        }
    }
}
/// A count of pieces or Terror, kept over its token.
#[derive(Component)]
struct Count;

/// A count's face. Its size is set in reference pixels; `scale_counts` scales it to the
/// window before the interface is laid out.
fn count_font(art: &Art) -> TextFont {
    TextFont {
        font: art.font.clone().into(),
        font_size: COUNT_SIZE.into(),
        ..default()
    }
}

/// Keeps every count at `COUNT_SIZE` reference pixels at the window's layout scale:
/// counts spawned this frame, and all of them after a resize.
fn scale_counts(
    window: Single<&Window, With<PrimaryWindow>>,
    mut counts: Query<&mut TextFont, With<Count>>,
) {
    let size = FontSize::Px(COUNT_SIZE * layout(&window).0);
    for mut font in &mut counts {
        if font.font_size != size {
            font.font_size = size;
        }
    }
}

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
    die: Res<die::DieArt>,
) {
    if play.error.is_some() || play.session.is_some() {
        return;
    }
    let clip_ids: Vec<bevy::asset::UntypedAssetId> = std::iter::once(&art.terror_clip)
        .chain(&die.clips)
        .map(|clip| clip.id().untyped())
        .collect();
    let required: Vec<bevy::asset::UntypedAssetId> = [
        art.scene.id().untyped(),
        art.terror.id().untyped(),
        die.die.id().untyped(),
        die.tray.id().untyped(),
    ]
    .into_iter()
    .chain(clip_ids.iter().copied())
    .collect();
    let report = unfolded_tabletop::loading::required_assets(&server, &required, |id| {
        if clip_ids.contains(&id) {
            clips.contains(id.typed::<AnimationClip>())
        } else {
            scenes.contains(id.typed::<WorldAsset>())
        }
    });
    if !report.failures.is_empty() {
        play.error = Some(report.failures.join("; "));
        ui.dirty = true;
        return;
    }
    let font_ready = [&art.font, &art.bold].into_iter().all(|font| {
        fonts
            .get(font)
            .is_some_and(|font| unfolded_tabletop::loading::font_registered(font, &mut context))
    });
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
                Action::Sequence(slot) => Some(Target::Sequence(slot)),
                Action::Decision(..) | Action::Unsupported(_) | Action::Pin(_) if !rested => {
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

#[allow(clippy::too_many_arguments)] // A Bevy observer: its parameters are the resources it reads.
fn click(
    mut event: On<PointerClick>,
    controls: Query<&Control>,
    mut play: ResMut<Play>,
    mut ui: ResMut<Interface>,
    mut exit: MessageWriter<AppExit>,
    mut roll: ResMut<die::Roll>,
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
    // A popover's button, or a commit named apart from its choice, submits its choice
    // exactly as the panel's button does.
    let control = &match &control.action {
        Action::Submit(owner, revision, choice) => Control {
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
        Action::Standing(_) | Action::Top(_) | Action::Sequence(_) => return,
        // A region click opens or closes a space popover while Terror targets are
        // chosen, and does nothing otherwise; it never decides anything itself. An
        // opened popover wins over the AC-B briefing tag, which reopens from the
        // BRIEFING ribbon (owner's clash rule, #58).
        Action::Space(space) => {
            // The transfer modal's scrim takes the map's input beneath it.
            if transfer::shown(play.game().transfer.as_ref()) {
                return;
            }
            if popover.click_space(space, &targeting) {
                ui.choice.close();
            }
            return;
        }
        Action::Row(action) => {
            popover.choose_row(action);
            return;
        }
        Action::Piece(key) => {
            popover.click_piece(key, &targeting);
            // A piece popover wins over a step's tag, as a space popover does.
            if popover.open() {
                ui.choice.close();
                ui.dirty = true;
            }
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
        Action::Submit(..) => return,
        Action::Close => {
            ui.reader = false;
        }
        Action::Reader => {
            ui.reader = true;
        }
        Action::Decision(owner, rev) | Action::Commit(owner, rev) => {
            decide(&mut play, &mut ui, *owner, *rev, &control.name);
        }
        Action::Pin(_) | Action::Ribbon | Action::Tab(_) => {
            choice::activate(&mut ui, &control.action);
            // Whichever opened last wins: the tag opened from the ribbon, or a step's
            // tag pinned from its row during Terror (#84), closes an open popover
            // (orchestrator implementation judgement, #58 carry-forward).
            let step = matches!(control.action, Action::Pin(_))
                && play.game().action == crate::ActionChoice::Terror;
            if (ui.choice.ribbon && matches!(control.action, Action::Ribbon))
                || (step && ui.choice.pinned.is_some())
            {
                popover.close_all();
            }
        }
        Action::Unsupported(reason) => ui.notice = reason.clone(),
        Action::Transfer(input) => transfer::input(&mut ui, &play.game().observe(), *input),
        // Presentation only: the roll was saved before the die moved.
        Action::SkipDie => roll.skip(),
        Action::ContinueDie => roll.close(play.game().observe().revision),
        Action::Quit => {
            exit.write(AppExit::Success);
        }
    }
    ui.dirty = true;
}
/// A primary click on the map, outside any interface node, closes the open AC-B
/// briefing tag, or during Terror the pinned EX-A step tag (#84), so the Santiago and
/// Oriente targets it covers become clickable (orchestrator implementation judgement,
/// recorded in #58's carry-forward; the owner accepted the coverage). A click on the
/// tag itself is an interface node and keeps it.
fn map_click_closes_tag(
    event: On<PointerClick>,
    nodes: Query<(), With<Node>>,
    play: Res<Play>,
    mut ui: ResMut<Interface>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary
        || play.session.is_none()
        || nodes.contains(event.entity)
    {
        return;
    }
    let step = play.game().action == crate::ActionChoice::Terror && ui.choice.pinned.is_some();
    if !(ui.choice.ribbon || step) {
        return;
    }
    ui.choice.close();
    ui.dirty = true;
}
/// Submits a decision through the checked game seam and words its outcome. Pointer
/// clicks and the keyboard's Enter on a focused control both come here.
fn decide(play: &mut Play, ui: &mut Interface, owner: Faction, rev: u64, name: &str) {
    let previous = play.game().observe();
    match play.session.as_mut().unwrap().submit(owner, rev, name) {
        Ok(()) => {
            let after = play.game().observe();
            if name.ends_with(":kidnap:roll") {
                // The result is saved; the tray shows it from this frame's clock
                // (die::present), and the notice names it once the die lands.
                ui.rolled = Some(after.revision);
            }
            ui.notice = notice(&previous, &after, owner, name);
        }
        Err(e) => ui.notice = e.to_string(),
    };
    // A committed decision leaves its step; its pin and focus go with it, and a step
    // the decision opened holds the panel with its tag pinned (EX-A, #84).
    ui.choice.decided(&play.game().observe());
}
/// "a, b and c".
fn listing(parts: &[String]) -> String {
    match parts {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}
/// The session notice for the committed decision `name`. Every rules fact it states
/// is the authority's (foundation 2, "Engine authority"): the decision's own id, read
/// through `Choice`'s accessors; the preview the authority showed for it, from
/// `previous`; or the record in `after`, the observation once it was committed.
fn notice(previous: &Observation, after: &Observation, owner: Faction, name: &str) -> String {
    let choice = previous.choices.iter().find(|c| c.id == name);
    let action = name.split_once(':').map_or(name, |(_, a)| a);
    let place = |id: &str| {
        after
            .spaces
            .iter()
            .find(|s| s.id == id)
            .map_or(id.to_owned(), |s| s.name.clone())
    };
    if let Some(transfer) = choice.and_then(Choice::transfer) {
        return match transfer {
            TransferChoice::Give { recipient, amount } => {
                format!(
                    "{} gave {amount} to {}. Saved.",
                    owner.name(),
                    recipient.name()
                )
            }
            TransferChoice::Ask { .. } => match after.transfer {
                Some(Transfer::Authorizing {
                    giver,
                    recipient,
                    amount,
                    ..
                }) => format!(
                    "Transfer request saved: {} asks {} for {amount}. Nothing moves until {} decides.",
                    recipient.name(),
                    giver.name(),
                    giver.name()
                ),
                _ => "Transfer request saved.".into(),
            },
            TransferChoice::Authorize { amount } => match previous.transfer {
                Some(Transfer::Authorizing {
                    giver, recipient, ..
                }) => format!(
                    "{} gave {amount} to {}. Saved.",
                    giver.name(),
                    recipient.name()
                ),
                _ => "Decision saved.".into(),
            },
        };
    }
    if let Some((_, _, n)) = choice.and_then(Choice::removal) {
        return format!(
            "Removed {n} own piece{} to Available. Removal stays open until Done; earlier Terror, support shifts and Kidnap results remain. Saved.",
            if n == 1 { "" } else { "s" }
        );
    }
    if let Some(space) = action.strip_prefix("terror:") {
        // What the authority previewed for this Terror, which committing applied.
        let Some(preview) = choice.and_then(|c| c.preview.as_ref()) else {
            return format!("Terror saved in {}.", place(space));
        };
        let mut done = vec![];
        for r in preview.resources.iter().filter(|r| r.to < r.from) {
            done.push(format!("paid {}", resources(r.from - r.to)));
        }
        let state = |state: State| match state {
            State::Underground => "Underground",
            State::Active => "Active",
            _ => "",
        };
        for f in &preview.flips {
            done.push(format!(
                "flipped {} {} Guerrilla{} to {}",
                f.count,
                state(f.from),
                if f.count == 1 { "" } else { "s" },
                state(f.to)
            ));
        }
        for _ in &preview.terror {
            done.push("placed a Terror marker".into());
        }
        for shift in &preview.support {
            done.push(format!("shifted {}", shift.label));
        }
        return format!("Terror saved in {}: {}.", place(space), listing(&done));
    }
    match action {
        "decline" => format!("{} declined. No resources moved.", owner.name()),
        "wait" => format!(
            "Waiting for {}. The request stays pending. Saved.",
            owner.name()
        ),
        "reopen" => format!("{} decides the Transfer request. Saved.", owner.name()),
        "back" => "Transfer closed. Nothing moved. Saved.".into(),
        "transfer" => "Resource transfer open: nothing has moved. Saved.".into(),
        "kidnap" => "Kidnap opened: nothing rolled. Roll the die or cancel. Saved.".into(),
        "kidnap:cancel" => "Kidnap cancelled: nothing rolled and nothing changed. Saved.".into(),
        "kidnap:roll" => {
            "Kidnap roll saved. The die is landing: click it or press Space to skip.".into()
        }
        "remove" => format!(
            "Removal mode: click a ringed {} piece on the map. Saved.",
            owner.name()
        ),
        "done-removal" => "Removal mode closed; removed pieces stay in Available. Saved.".into(),
        "pass" => {
            // The record Pass committed: its Resources, 26July's Eligibility for the
            // next card and the next faction's role.
            let gained: Vec<String> = after
                .recorded_resources
                .iter()
                .map(|r| format!("+{}", resources(r.to.saturating_sub(r.from))))
                .collect();
            let eligibility = match after.eligibility.m26_next_card {
                Some(true) => " and stays Eligible",
                Some(false) => " and becomes Ineligible after this card",
                None => "",
            };
            let next = after.next.as_ref().map_or(String::new(), |n| {
                format!(" {} is {}.", n.faction.name(), n.role)
            });
            format!(
                "26July passed: {}{eligibility}.{next} Saved.",
                listing(&gained)
            )
        }
        "terror" => match after.executing {
            Some(executing) => {
                let open = after
                    .choices
                    .iter()
                    .any(|c| c.supported && c.action() == "transfer");
                format!(
                    "Terror chosen: {} is executing{}. Saved.",
                    executing.name(),
                    if open { ", and transfers are open" } else { "" }
                )
            }
            None => "Terror chosen. Saved.".into(),
        },
        "back-to-choice" => "Back at the action choice. Nothing had happened; any pending request is withdrawn. Saved.".into(),
        "end-terror" => match after.eligibility.m26_box {
            Some(record) => format!("Terror ended: {} recorded. Saved.", record.name()),
            None => "Terror ended. Saved.".into(),
        },
        "event:shaded" => format!(
            "{} chose the shaded Event. Resolve it to apply its effects. Saved.",
            owner.name()
        ),
        "resolve-shaded" => "Shaded Event resolved. Eligibility for card 18 recorded. Saved.".into(),
        "event:unshaded" => "Directorio chooses who will Rally. Saved.".into(),
        "advance-card" => "Pact of Caracas is current. Government is 1st Eligible. Saved.".into(),
        "government:assault" => "Assault chosen. Select a space to inspect its targets. Saved.".into(),
        "end-assault" => "Operation Only recorded. Syndicate is 2nd Eligible. Saved.".into(),
        _ if crate::Game::assault_payment_space(action).is_some() => {
            let result = after.assault_result.as_ref().expect("Assault payment has a result");
            format!("Assault resolved in {}. Saved.", result.space_name())
        }
        _ if action.starts_with("rally:") => match &after.event {
            Some(crate::EventProgress::Rally(rally)) => {
                format!("{} directs the free Rally. Saved.", rally.beneficiary.name())
            }
            _ => "Unshaded Event resolved. Eligibility for card 18 recorded. Saved.".into(),
        },
        _ => "Decision saved.".into(),
    }
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
/// What the table says to the shared camera: the reader, an unopened session, the
/// transfer modal and a presented Kidnap die block it, and only the map shows the
/// scene. The die's camera is a second `Camera3d`, which the shared count anchors do
/// not follow, so the map holds still under its counts while the die is shown.
fn camera_facts(window: &Window, play: &Play, ui: &Interface, roll: &die::Roll) -> TableFacts {
    let (modal, die) = play.session.as_ref().map_or((false, false), |session| {
        let game = session.game();
        (
            transfer::shown(game.transfer.as_ref()),
            die::presented(game.revision, game.kidnap.as_ref(), roll).is_some(),
        )
    });
    TableFacts {
        blocks_camera: ui.reader || play.session.is_none() || modal || die,
        scene: Scene::Within(map_rect(window)),
        ..default()
    }
}
fn camera_facts_and_viewport(
    window: Single<&Window, With<PrimaryWindow>>,
    play: Res<Play>,
    ui: Res<Interface>,
    roll: Res<die::Roll>,
    mut table: ResMut<TableFacts>,
    mut cameras: Query<&mut Camera, With<TableCamera>>,
) {
    *table = camera_facts(&window, &play, &ui, &roll);
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
    styled(
        commands,
        art,
        content,
        &art.font,
        Color::srgb(0.12, 0.14, 0.12), // own colour: Atlas desk text (foundation 4)
        [x, y, w, size],
        scale,
        offset,
        z,
    )
}
/// Text in a chosen face and colour: `place` is the authored left, top, width and type
/// size. Inline emblems keep the same height and baseline as in `text`.
#[allow(clippy::too_many_arguments)]
fn styled(
    commands: &mut Commands,
    art: &Art,
    content: impl Into<Prose>,
    face: &Handle<Font>,
    color: Color,
    place: [f32; 4],
    scale: f32,
    offset: Vec2,
    z: i32,
) -> Entity {
    let [x, y, w, size] = place;
    let content: Prose = content.into();
    let font = TextFont {
        font: face.clone().into(),
        font_size: (size * scale).into(),
        ..default()
    };
    let ink = TextColor(color);
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

fn card_picture(
    commands: &mut Commands,
    art: &Art,
    face: &assets::CardFace,
    [x, y, width]: [f32; 3],
    scale: f32,
    offset: Vec2,
    z: i32,
) {
    let height = width * face.size.1 as f32 / face.size.0 as f32;
    picture(
        commands,
        art,
        face.name,
        [x, y, width, height],
        scale,
        offset,
        z,
    );
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
        .insert(TextColor(Color::srgb(0.98, 0.97, 0.92))); // own colour: Atlas desk button label (foundation 4)
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
    roll: Res<die::Roll>,
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
        top::header(
            &mut commands,
            &art,
            &die::ledger(&view, &roll),
            s,
            left,
            right,
        );
    }
    let current = assets::card_face(view.current_card);
    let preview = assets::card_face(view.preview_card);
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
        card_picture(&mut commands, &art, current, [320., 380., 420.], s, o, 3);
        card_picture(&mut commands, &art, preview, [1000., 380., 420.], s, o, 3);
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
        let current_status = if view.current_card == 18 {
            "current · Event inactive"
        } else if view.eligibility.next_card.is_some() {
            "Event resolved"
        } else if view.event.is_some() {
            "Event in progress"
        } else if view.eligibility.directorio_handoff() {
            "current · Event open to Directorio"
        } else {
            "current · Event inactive"
        };
        text(
            &mut commands,
            &art,
            format!("{} {} — {current_status}", view.current_card, current.title),
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
            format!(
                "{} {} — preview · Event inactive",
                view.preview_card, preview.title
            ),
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
        if let Some(kidnap) = die::presented(view.revision, view.kidnap.as_ref(), &roll) {
            die::panel(&mut commands, &art, kidnap, roll.landed(), s, right);
        } else {
            execution::panel(&mut commands, &art, &view, &ui, s, right);
        }
        eligibility::indicator(&mut commands, &art, &view, s, left);
        let o = left;
        card_picture(&mut commands, &art, current, [100., 1040., 108.], s, o, 2);
        card_picture(&mut commands, &art, preview, [366., 1040., 108.], s, o, 2);
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
                "{} current · {} preview · {} cards in draw deck",
                view.current_card, view.preview_card, view.draw_count
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

    /// The notice for `action` submitted by its owner, with `tamper` applied to the
    /// observations before and after it: each fact the notice states must follow them.
    fn noticed(
        game: &mut Game,
        action: &str,
        tamper: impl FnOnce(&mut Observation, &mut Observation),
    ) -> String {
        let mut previous = game.observe();
        let id = format!("{}:{action}", previous.revision);
        let owner = previous.choices.iter().find(|c| c.id == id).unwrap().owner;
        game.submit(owner, previous.revision, &id).unwrap();
        let mut after = game.observe();
        tamper(&mut previous, &mut after);
        notice(&previous, &after, owner, &id)
    }

    /// Foundation 2, "Engine authority": the notices state the record the decision
    /// committed. Pass's gain, 26July's Eligibility and the next faction's role come
    /// from the observation after it; changed there to facts the old fixed sentence
    /// ("+1 Resource", "Directorio is 1st Eligible") would misstate, they follow.
    #[test]
    fn the_pass_notice_states_the_recorded_pass() {
        let pass = noticed(&mut Game::opening(), "pass", |_, _| {});
        assert_eq!(
            pass,
            "26July passed: +1 Resource and stays Eligible. Directorio is 1st Eligible. Saved."
        );
        let capped = noticed(&mut Game::opening(), "pass", |_, after| {
            after.recorded_resources[0].from = 49;
            after.recorded_resources[0].to = 49;
            after.eligibility.m26_next_card = Some(false);
            let next = after.next.as_mut().unwrap();
            next.faction = Faction::Government;
            next.role = "2nd Eligible".into();
        });
        assert_eq!(
            capped,
            "26July passed: +0 Resources and becomes Ineligible after this card. Government is 2nd Eligible. Saved."
        );
    }

    /// The End Terror notice names the box the authority recorded, not one guessed
    /// from whether Kidnap was rolled.
    #[test]
    fn the_end_terror_notice_states_the_recorded_box() {
        let mut game = Game::prepared();
        let only = noticed(&mut game.clone(), "end-terror", |_, _| {});
        assert_eq!(only, "Terror ended: Operation Only recorded. Saved.");
        let recorded = noticed(&mut game, "end-terror", |previous, after| {
            previous.kidnap = Some(crate::Kidnap {
                die: 3,
                proceeds: 3,
                government_before: 6,
                m26_before: 0,
                random_before: 0,
                revision: 0,
            });
            after.eligibility.m26_box = Some(crate::SequenceBox::OperationOnly);
        });
        assert_eq!(recorded, "Terror ended: Operation Only recorded. Saved.");
    }

    /// A Terror notice words the authority's preview of that Terror, which committing
    /// applied: Santiago is Neutral, so it shifts to Passive Opposition (R 3.3.4).
    /// A changed preview changes the notice, which the old fixed sentence ("paid 1
    /// Resource … shifted 1 level") did not follow.
    #[test]
    fn a_terror_notice_states_the_authority_s_preview() {
        let mut funded = Game::prepared_with_seed(5);
        funded.submit(Faction::M26, 0, "0:transfer").unwrap();
        funded
            .submit(
                Faction::M26,
                1,
                &format!("1:ask:{}:2", Faction::Syndicate.index()),
            )
            .unwrap();
        funded
            .submit(Faction::Syndicate, 2, "2:authorize:2")
            .unwrap();
        let santiago = noticed(&mut funded.clone(), "terror:santiago", |_, _| {});
        assert_eq!(
            santiago,
            "Terror saved in Santiago de Cuba: paid 1 Resource, flipped 1 Underground Guerrilla to Active, placed a Terror marker and shifted Neutral → Passive Opposition."
        );
        let changed = noticed(&mut funded, "terror:santiago", |previous, _| {
            let choice = previous
                .choices
                .iter_mut()
                .find(|c| c.action() == "terror:santiago")
                .unwrap();
            let preview = choice.preview.as_mut().unwrap();
            preview.resources[0].to = preview.resources[0].from - 2;
            preview.flips[0].count = 2;
            preview.support.clear();
        });
        assert_eq!(
            changed,
            "Terror saved in Santiago de Cuba: paid 2 Resources, flipped 2 Underground Guerrillas to Active and placed a Terror marker."
        );
    }

    /// Transfer notices read the decision through its typed accessor and the request
    /// the authority recorded.
    #[test]
    fn transfer_notices_name_the_decision_and_the_recorded_request() {
        let mut game = Game::prepared();
        game.submit(Faction::M26, 0, "0:transfer").unwrap();
        let asked = noticed(
            &mut game.clone(),
            &format!("ask:{}:2", Faction::Government.index()),
            |_, _| {},
        );
        assert_eq!(
            asked,
            "Transfer request saved: 26July asks Government for 2. Nothing moves until Government decides."
        );
        let mut gave = Game::prepared();
        gave.submit(Faction::M26, 0, "0:transfer").unwrap();
        gave.submit(
            Faction::M26,
            1,
            &format!("1:ask:{}:2", Faction::Syndicate.index()),
        )
        .unwrap();
        let authorized = noticed(&mut gave, "authorize:2", |_, _| {});
        assert_eq!(authorized, "Syndicate gave 2 to 26July. Saved.");
    }
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
            camera_facts(&window, &play, &ui, &die::Roll::default()),
        );
        assert!(orbit.pose().distance < 10.);
        let inside = orbit.pose();
        window.set_cursor_position(Some(Vec2::new(1500., 500.)));
        drive(
            &mut orbit,
            &window,
            &keys,
            1.,
            camera_facts(&window, &play, &ui, &die::Roll::default()),
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
            camera_facts(&window, &play, &ui, &die::Roll::default()),
        );
        assert_eq!(orbit.pose(), inside);
    }
}
