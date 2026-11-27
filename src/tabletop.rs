//! Selected Atlas presentation. Input travels through Bevy picking into the checked game seam.
use crate::{
    Choice, Faction, Game, Kind, Observation, State, Terrain, Transfer, TransferChoice,
    assets::{self, Verified},
    resources,
    save::Session,
};
use bevy::{
    app::AnimationSystems,
    asset::{
        AssetPath, RenderAssetUsages,
        io::{
            AssetSourceBuilder,
            memory::{Dir, MemoryAssetReader},
        },
    },
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    gltf::GltfLoaderSettings,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    picking::{events::PointerClick, hover::HoverMap, pointer::PointerId},
    prelude::*,
    render::{
        Render, RenderApp,
        render_asset::RenderAssets,
        render_resource::PipelineCache,
        renderer::{RenderQueue, render_system},
        texture::GpuImage,
    },
    window::PrimaryWindow,
};
use emblems::Prose;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use unfolded_hover::Hover;
use unfolded_tabletop::{
    camera::{
        Framing, InScene, Orbit, PITCH, Pose, Scene, TableCamera, TableCameraPlugin,
        TableCameraSystems, TableFacts,
    },
    label_anchor::{LabelAnchor, LabelAnchorPlugin},
    pipeline, token,
};
mod areas;
mod available;
mod briefing;
mod broadsheet;
mod cards;
mod choice;
pub mod die;
#[cfg(test)]
mod discovery;
mod eligibility;
mod emblems;
mod execution;
mod faction_seats;
mod flight;
mod focus_mark;
mod force_travel;
mod highlights;
mod map;
mod marker_travel;
mod newspaper;
mod notice;
#[cfg(feature = "occupancy-review")]
mod occupancy_review;
#[cfg(feature = "occupancy-review")]
pub use occupancy_review::OccupancyReview;
mod palette;
mod piece_flips;
mod piece_visuals;
mod pile_tags;
mod piles;
mod popover;
mod proclamation_composer;
mod proclamation_reader;
mod roundel;
mod save_quit;
pub mod seat;
mod seat_cards;
mod settled;
mod status;
mod supply;
mod top;
mod transfer;
mod victory;
use piece_visuals::{add_group_face, group_color, group_half_height, group_mesh, spawn_group_body};

#[derive(Clone)]
pub struct Options {
    pub art: PathBuf,
    pub cards: PathBuf,
    pub save: PathBuf,
    pub recover: Option<PathBuf>,
    pub socket: Option<PathBuf>,
    /// Where agent sessions play handed-over factions, instead of the default.
    pub players: Option<PathBuf>,
    pub agent: bool,
    #[cfg(feature = "occupancy-review")]
    pub occupancy_review: Option<OccupancyReview>,
    pub seat_size: UVec2,
}
#[derive(Resource)]
struct Play {
    session: Option<Session>,
    options: Options,
    error: Option<String>,
    ready_frames: u8,
    #[cfg(feature = "occupancy-review")]
    review_ready: bool,
}
impl Play {
    fn admitted(&self) -> bool {
        #[cfg(feature = "occupancy-review")]
        if self.review_ready {
            return true;
        }
        self.session.is_some()
    }

    fn observe(&self) -> crate::Observation {
        #[cfg(feature = "occupancy-review")]
        if let Some(review) = &self.options.occupancy_review {
            return review.observe();
        }
        self.game().observe()
    }

    fn game(&self) -> Game {
        #[cfg(feature = "occupancy-review")]
        if let Some(review) = &self.options.occupancy_review {
            return review.position.clone();
        }
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
    /// Lato Black, the Broadsheet's issuer face.
    black: Handle<Font>,
    /// Noto Serif Display Bold, the newspaper's headlines and titles.
    headline: Handle<Font>,
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
    /// The current or the preview card, by its seat name.
    Card(cards::Slot),
    /// The played pile or the draw deck, by its seat name (#144).
    Pile(piles::Pile),
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
    /// The card tag's pin and Escape state.
    card: cards::State,
    /// The latest notice and its kind (#174): the seat reads every one, the slip
    /// draws a refusal or an error.
    notice: notice::Notice,
    dirty: bool,
    /// The action choice's pinned option, keyboard focus and briefing ribbon.
    choice: choice::State,
    /// Each faction's seat, indexed like `Faction::ALL`: the human's input submits nothing
    /// for one that is not the human's (`faction_seats`).
    seats: [crate::faction_seats::SeatState; 4],
    /// The host has both a human faction and a faction claimed by an agent.
    mixed_seats: bool,
    /// The seat controls pressed in this frame, until `faction_seats::serve` reads them.
    presses: Vec<seat_cards::Press>,
    /// The open sheet with a seat's line to copy.
    line: Option<seat_cards::Line>,
    /// The current card's editions (#208): while one is due or open nothing is decided.
    editions: crate::editions::Editions,
    composer: proclamation_composer::State,
    proclamation_reader: proclamation_reader::State,
    /// The transfer modal's counterpart, direction, typed amount and focus.
    transfer: transfer::State,
    /// The revision a Kidnap roll just created in this run, until the die's
    /// presentation starts it from the shared clock in the same frame (`die::present`).
    rolled: Option<u64>,
    /// The card a decision just played in this run, until the tray starts showing its
    /// advance from the shared clock in the same frame (`piles::follow`).
    played: Option<piles::Played>,
    /// Terror marker flights from committed count changes.
    marker_travel: marker_travel::Travel,
    /// State changes from a saved decision, until the map takes them up.
    piece_flips: Vec<piece_flips::Change>,
    /// A successfully saved change to the force inventory, before it is drawn.
    force_travel: Option<Vec<force_travel::Trip>>,
}
impl Interface {
    /// Whether the human plays `faction`.
    fn plays(&self, faction: Faction) -> bool {
        self.seats[faction.index()] == crate::faction_seats::SeatState::Human
    }
    /// Whether the human can confirm this offered faction choice.
    fn can_confirm(&self, choice: &Choice) -> bool {
        choice.supported && self.plays(choice.owner)
    }
    /// The factions the human plays, indexed like `Faction::ALL`.
    fn played(&self) -> [bool; 4] {
        Faction::ALL.map(|faction| self.plays(faction))
    }
}
#[derive(Component)]
struct UiRoot;
#[derive(Component)]
struct StandingText(Faction);
#[derive(Component)]
struct PieceVisual;
/// The displayed quantity and picking ownership of a group at its map slot.
#[derive(Component, PartialEq, Eq)]
struct RestingPiece {
    count: u8,
    owned: bool,
}
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
    /// The current or the preview card: hover opens its tag, a click pins it.
    Card(cards::Slot),
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
    /// A pile's count disc: the pile's target. Hover and focus open its tag
    /// (`pile_tags`); a click changes nothing.
    Pile(piles::Pile),
    /// An overlaid area's own rectangle (`areas::Area`): it takes the pointer and
    /// does nothing.
    Area,
    /// Lands the Kidnap die at once.
    SkipDie,
    /// Closes the landed Kidnap die and returns the panel to Terror.
    ContinueDie,
    /// Save · quit: closes the game. Every decision is already saved.
    Quit,
    /// A click on the notice slip dismisses it (#174).
    DismissNotice,
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
    /// A seat control (#206): hands a faction over, takes it back or copies its line.
    Seat(seat_cards::Press),
    /// The newspaper (#208): puts the open edition away, turns to another or takes the
    /// latest from the pile. Decides nothing.
    Edition(newspaper::Press),
    ComposeProclamation(proclamation_composer::Press),
    ReadProclamation(proclamation_reader::Press),
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
    // The asynchronous scene load consumes the same GLB bytes admitted above.
    let terrain = Dir::default();
    terrain.insert_asset(
        Path::new("atlas-terrain-map.glb"),
        verified.files["atlas-terrain-map.glb"].clone(),
    );
    app.register_asset_source(
        "atlas-map",
        AssetSourceBuilder::new(move || {
            Box::new(MemoryAssetReader {
                root: terrain.clone(),
            })
        }),
    );
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
            let mut decoded = if key == assets::CARD_BACK.name {
                piles::back(bytes)?
            } else {
                Image::from_buffer(
                    bytes,
                    ImageType::Extension("png"),
                    CompressedImageFormats::NONE,
                    true,
                    ImageSampler::linear(),
                    DRAWN_ONLY,
                )
                .map_err(|e| e.to_string())?
            };
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
    let black = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(verified.files["Lato-Black.ttf"].clone()));
    let headline = app
        .world_mut()
        .resource_mut::<Assets<Font>>()
        .add(Font::from_bytes(
            verified.files["NotoSerifDisplay-Bold.ttf"].clone(),
        ));
    let die_art = die::DieArt::load(&mut app, &verified.files["kidnap-die.json"])?;
    let main_view = areas::Layout::read(&verified.files["atlas-main-view.layout.json"])?;
    let server = app.world().resource::<AssetServer>();
    let scene = model_part(server, "atlas-map://atlas-terrain-map.glb#Scene0");
    let terror = model_part(server, "atlas-terror-token.glb#Scene0");
    let terror_clip = model_part(server, "atlas-terror-token.glb#Animation0");
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
            black,
            headline,
            scene,
            terror,
            terror_clip: terror_clip.clone(),
            labels: labels.labels,
        })
        .insert_resource(faction_seats::FactionSeats::new(
            options.players.clone(),
            windowless,
        ))
        .insert_resource(die_art)
        .insert_resource(main_view)
        .init_resource::<areas::Island>()
        .init_resource::<Counts>()
        .init_resource::<piece_flips::Flights>()
        .init_resource::<force_travel::Flights>()
        .init_resource::<die::Roll>()
        .init_resource::<piles::Advance>()
        .init_resource::<newspaper::Drawn>()
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
            #[cfg(feature = "occupancy-review")]
            review_ready: false,
        })
        .insert_resource(Interface {
            dirty: true,
            ..default()
        })
        // Where the camera waits while the map loads. `areas::home` then frames the
        // island in the layout's box for the window; art/stage/preview-stage.blend
        // mirrors that framing on the owner's display.
        .insert_resource(
            Orbit::new(Framing::new(Vec3::new(0.35, 0., 0.35), 9.6))
                .with_navigation(Pose::DEFAULT_PITCH, PITCH, 2.75..=90.)
                .expect("Atlas camera distance and pitch range"),
        )
        // The layout's `map` colour, which the authored sea also has: the window
        // shows it beyond the sea's edge.
        .insert_resource(ClearColor(palette::MAP))
        .add_plugins((
            MeshPickingPlugin,
            LabelAnchorPlugin,
            TableCameraPlugin::<()>::default(),
        ))
        .configure_sets(
            Update,
            TableCameraSystems
                .after(write_camera_facts)
                .before(status::hover),
        )
        .init_resource::<highlights::Targeting>()
        .init_resource::<popover::Popover>()
        .add_systems(Startup, (setup, highlights::setup, popover::setup))
        .add_systems(
            PreUpdate,
            proclamation_composer::prepare_input
                .after(bevy::input::InputSystems)
                .before(bevy::input_focus::InputFocusSystems::Dispatch),
        )
        .add_observer(map_click_closes_tag)
        .add_observer(piles::land_on_click)
        .add_systems(
            Update,
            (
                admit,
                highlights::refresh,
                highlights::show,
                (force_travel::stage, sync_pieces, piece_flips::animate).chain(),
                highlights::markers,
                popover::hover_piece,
                bind_spaces,
                hover_disclosures,
                // A playing card advance takes Space and Enter before anything else
                // reads them, but an open edition, which takes every key. The presented
                // die takes them too, so the action choice and the modal read the
                // keyboard first and stand aside while it is shown. A decision any of
                // them applied may have played a card, or have an edition to open.
                (
                    proclamation_reader::keys,
                    proclamation_composer::keys,
                    newspaper::keys,
                    piles::keys,
                    choice::keys,
                    transfer::keys,
                    faction_seats::serve,
                    proclamation_composer::publish,
                    die::present,
                    piles::follow,
                    newspaper::open,
                )
                    .chain(),
                draw_ui,
                (
                    areas::home,
                    write_camera_facts,
                    force_travel::animate.after(TableCameraSystems),
                )
                    .chain(),
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
                    cards::draw,
                    pile_tags::draw,
                    piles::present,
                    save_quit::hover,
                    newspaper::draw,
                    proclamation_composer::draw,
                    focus_mark::draw,
                )
                    .chain(),
                marker_travel::present,
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
        )
        .add_systems(Last, faction_seats::close);
    roundel::install(&mut app);
    Ok(app)
}
/// How the table keeps an image it only draws. The renderer takes the pixels, so the app
/// holds no second copy of them; the size stays, which the interface reads. An image any
/// system reads after it loads uses the default instead, and says why. Nothing can
/// upload these again from memory: a changed image must be loaded afresh.
const DRAWN_ONLY: RenderAssetUsages = RenderAssetUsages::RENDER_WORLD;

/// Loads one part of an authored model with `DRAWN_ONLY` textures. Its meshes keep their
/// copies: picking, the island survey, the popovers and the space tags read them.
fn model_part<A: Asset>(server: &AssetServer, part: impl Into<AssetPath<'static>>) -> Handle<A> {
    server
        .load_builder()
        .with_settings(|settings: &mut GltfLoaderSettings| settings.load_materials = DRAWN_ONLY)
        .load(part.into())
}
fn render_ready(
    images: Res<RenderAssets<GpuImage>>,
    queue: Res<RenderQueue>,
    ready: Res<RenderReady>,
    pipelines: Res<PipelineCache>,
) {
    ready
        .pipelines
        .store(pipeline::is_ready(&pipelines), Ordering::Release);
    if ready.images.iter().all(|id| images.get(*id).is_some()) {
        let flag = ready.ready.clone();
        queue.on_submitted_work_done(move || flag.store(true, Ordering::Release));
    }
}
/// art/stage/preview-stage.blend mirrors this light, sea and region
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
        // Preserve the authored unlit sea colour, which is the layout's `map`.
        // Debanding needs tonemapping, so it is off too (#86). The camera has no
        // viewport: the map fills the window (#142).
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::from_xyz(-0.2, 8.84, 5.17).looking_at(Vec3::new(-0.2, 0., -0.5), Vec3::Y),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            // Over the map (0) and the card tray (1); the Kidnap tray (3) draws over it.
            order: 2,
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
    commands.spawn((
        WorldAssetRoot(art.scene.clone()),
        Transform::from_scale(Vec3::new(map::SCALE, 1., map::SCALE)),
    ));
    let game = Game::opening();
    for label in &art.labels {
        let space = game.spaces.iter().find(|s| s.id == label.space).unwrap();
        let [x, y, w, h] = map::label_bounds(label);
        let at = map::at(x + w / 2., y + h / 2., map::top(space.terrain) + 0.004);
        commands.spawn((
            Name::new(format!("Atlas label:{}", space.id)),
            Mesh3d(
                meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(w * map::SCALE / 100., h * map::SCALE / 100.),
                ),
            ),
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
#[allow(clippy::too_many_arguments)] // A Bevy system: its parameters are the resources it reads.
#[allow(clippy::type_complexity)]
fn sync_pieces(
    mut commands: Commands,
    play: Res<Play>,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    roots: Query<
        (Entity, &Name, &Transform, Option<&RestingPiece>),
        (
            Or<(With<PieceVisual>, With<RestingPiece>)>,
            Without<TerrorToken>,
        ),
    >,
    mut counts: ResMut<Counts>,
    mut previous: Local<Option<(Faction, Vec<crate::Space>)>>,
    mut ui: ResMut<Interface>,
    time: Res<Time>,
    mut flights: ResMut<piece_flips::Flights>,
    force_flights: Res<force_travel::Flights>,
    mut force_generation: Local<u64>,
    targeting: Res<highlights::Targeting>,
) {
    if counts.settling > 0 {
        counts.settling -= 1;
    }
    let game = play.game();
    let executor = game.executing().unwrap_or(Faction::M26);
    let new_motion = flights.refresh(&mut ui.piece_flips, time.elapsed_secs_f64());
    let interrupted = flights.settle_missing_destinations(&game.spaces);
    let motion_changed = new_motion || interrupted;
    let board_changed = motion_changed
        || *force_generation != force_flights.generation()
        || previous
            .as_ref()
            .is_none_or(|(faction, spaces)| *faction != executor || *spaces != game.spaces);
    if !board_changed && !targeting.is_changed() {
        return;
    }
    if board_changed {
        *previous = Some((executor, game.spaces.clone()));
        *force_generation = force_flights.generation();
        counts.unsettle();
    }
    #[cfg(feature = "occupancy-review")]
    let review = play.options.occupancy_review.as_ref();
    #[cfg(feature = "occupancy-review")]
    let recalculate_fixture = review.is_some();
    #[cfg(not(feature = "occupancy-review"))]
    let recalculate_fixture = false;
    let mut resting = BTreeMap::new();
    for (entity, name, at, piece) in &roots {
        if let Some(piece) = piece.filter(|_| !recalculate_fixture) {
            resting.insert(name.as_str(), (entity, at, piece));
        } else if recalculate_fixture || motion_changed {
            commands.entity(entity).despawn();
        }
    }
    for label in &art.labels {
        let space = game.spaces.iter().find(|s| s.id == label.space).unwrap();
        let top = map::top(space.terrain);
        let scale = map::PIECE_SCALE;
        let moving = flights
            .in_space(&space.id)
            .filter(|_| motion_changed)
            .collect::<Vec<_>>();
        let pieces = space
            .pieces
            .iter()
            .map(|p| (p, None))
            .chain(moving.iter().map(|(piece, change)| (piece, Some(*change))));
        for (piece, flight) in pieces {
            let count = force_flights.count(&space.id, piece, flights.count(&space.id, piece));
            let own_key = (flight.is_none() && piece.faction == executor).then(|| {
                format!(
                    "{}:{}",
                    space.id,
                    crate::removal_group(piece.kind, piece.state)
                )
            });
            let awaiting_arrival = flight.is_none() && count == 0;
            if awaiting_arrival
                && own_key
                    .as_deref()
                    .and_then(|key| targeting.piece(key))
                    .is_none()
            {
                continue;
            }
            let height = group_half_height(piece.kind);
            let [x, y] = map::slot(&space.id, piece);
            let base = map::at(x, y, top);
            #[cfg(feature = "occupancy-review")]
            let base = if let Some(review) = review {
                let i = space
                    .pieces
                    .iter()
                    .position(|group| std::ptr::eq(group, piece))
                    .expect("review group in the observed space");
                base + Vec3::Y
                    * occupancy_review::cash_solids(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &space.id,
                        i,
                        review.cash(&space.id, i),
                        base,
                    )
            } else {
                base
            };
            let rest = Transform::from_translation(base + Vec3::Y * height * scale)
                .with_scale(Vec3::splat(scale));
            let name = format!(
                "{} {:?} {:?} {:?}",
                space.id, piece.faction, piece.kind, piece.state
            );
            let stationary = RestingPiece {
                count,
                owned: own_key.is_some(),
            };
            if flight.is_none()
                && let Some((entity, at, old)) = resting.remove(name.as_str())
            {
                // Bevy pairs press and release on the same entity. A landing in
                // another slot must leave this group's body and roundel in place.
                if *at == rest && *old == stationary {
                    continue;
                }
                commands.entity(entity).despawn();
            }
            let motion = flight.map(|change| {
                let from = crate::Group {
                    state: change.flip.from,
                    ..piece.clone()
                };
                let [x, y] = map::slot(&space.id, &from);
                let start = map::at(x, y, top) + Vec3::Y * height * scale;
                let motion = piece_flips::Motion::new(
                    start,
                    rest,
                    height * scale,
                    group_color(piece),
                    change,
                );
                (from, motion)
            });
            let token = if awaiting_arrival {
                // The existing removal cue shows the committed slot while the
                // visible body is in flight. A mesh without a material is an
                // anchor and pick surface here, without a second solid or count.
                commands
                    .spawn((
                        Name::new(name),
                        Mesh3d(meshes.add(group_mesh(piece.kind))),
                        rest,
                    ))
                    .id()
            } else {
                let entity = spawn_group_body(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    piece,
                    name,
                    motion
                        .as_ref()
                        .map_or(rest, |(_, motion)| motion.source_pose()),
                );
                commands.entity(entity).insert(PieceVisual);
                entity
            };
            if let Some((from, motion)) = motion {
                add_group_face(
                    &mut commands,
                    &art,
                    &mut meshes,
                    &mut materials,
                    token,
                    &from,
                    true,
                );
                commands.entity(token).insert(motion);
            } else {
                commands
                    .entity(token)
                    .insert((stationary, status::SpaceTarget(space.id.clone())))
                    .remove::<Pickable>();
            }
            if let Some(key) = own_key {
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
            if awaiting_arrival {
                continue;
            }
            add_group_face(
                &mut commands,
                &art,
                &mut meshes,
                &mut materials,
                token,
                piece,
                false,
            );
            if flight.is_some() {
                continue;
            }
            let reach = match piece.kind {
                Kind::Troops | Kind::Police | Kind::Guerrilla => 0.13,
                Kind::Base | Kind::Casino => 0.15,
            };
            roundel::spawn(
                &mut commands,
                &art,
                &space.id,
                count.to_string(),
                token,
                Vec3::new(reach, -height - (base.y - top) / scale + 0.025, 0.),
            );
        }
        #[cfg(feature = "occupancy-review")]
        if play
            .options
            .occupancy_review
            .as_ref()
            .is_some_and(|review| review.sabotaged(&space.id))
        {
            let [x, y] = map::marker(&space.id);
            let marker = map::at(x, y, top);
            occupancy_review::sabotage_solid(
                &mut commands,
                &mut meshes,
                &mut materials,
                &art,
                &space.id,
                marker,
            );
        }
    }
    for (entity, _, _) in resting.into_values() {
        commands.entity(entity).despawn();
    }
}
/// A quantity beside a force group or Terror marker.
#[derive(Component)]
struct Count;

/// The development seat waits while roundels acquire their screen bounds and
/// become pickable following a spawn or camera move.
#[derive(Resource, Default)]
struct Counts {
    settling: u8,
}
impl Counts {
    /// The frames a count takes to be placed, measured and picked where it stands.
    const SETTLING: u8 = 3;
    /// Holds the seat while the counts follow a change of the pieces or the camera.
    fn unsettle(&mut self) {
        self.settling = Self::SETTLING;
    }
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
    if play.error.is_some() || play.admitted() {
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
    let font_ready = [&art.font, &art.bold, &art.black, &art.headline]
        .into_iter()
        .all(|font| {
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
    #[cfg(feature = "occupancy-review")]
    if play.options.occupancy_review.is_some() {
        play.review_ready = true;
        ui.dirty = true;
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
        Ok(session) => {
            ui.editions = crate::editions::Editions::from(&session.observe());
            play.session = Some(session);
        }
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
/// One dwell for every hover disclosure: victory standings, the top display and the
/// card tag.
fn hover_disclosures(
    hover: Res<HoverMap>,
    controls: Query<&Control>,
    top_tags: Query<&top::TagSurface>,
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
            if let Ok(tag) = top_tags.get(*e) {
                return Some(Target::Top(tag.0));
            }
            let control = controls.get(*e).ok()?;
            match control.action {
                Action::Standing(f) => Some(Target::Standing(f)),
                Action::Top(item) => Some(Target::Top(item)),
                Action::Sequence(slot) => Some(Target::Sequence(slot)),
                Action::Card(slot) => Some(Target::Card(slot)),
                Action::Pile(pile) => Some(Target::Pile(pile)),
                Action::Quit => Some(Target::Top(top::Item::SaveQuit)),
                Action::Decision(..) | Action::Unsupported(_) | Action::Pin(_) if !rested => {
                    Some(Target::Choice(control.name.clone()))
                }
                _ => None,
            }
        })
    });
    // Keyboard focus on a card, a pile, the marker supply, an Available Forces row or
    // Save · quit opens its tag as the pointer does, with the same dwell.
    let target = target
        .or_else(|| pile_tags::focused(&ui).map(Target::Pile))
        .or_else(|| supply::focused(&ui).map(Target::Top))
        .or_else(|| available::focused(&ui).map(Target::Top))
        .or_else(|| {
            (ui.choice.focus.as_deref() == Some(save_quit::NAME))
                .then_some(Target::Top(top::Item::SaveQuit))
        });
    let target = cards::target(&mut ui, target);
    ui.hover
        .update(target, true, time.delta(), Duration::from_millis(400));
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
    time: Res<Time>,
    frame: Res<bevy::diagnostic::FrameCount>,
    mut advance: ResMut<piles::Advance>,
) {
    event.propagate(false);
    if event.button != bevy::picking::pointer::PointerButton::Primary || play.session.is_none() {
        return;
    }
    if ui.proclamation_reader.is_open() {
        if let Ok(control) = controls.get(event.entity) {
            match control.action {
                Action::ReadProclamation(press) => {
                    let session = play.session.as_mut().expect("An admitted match");
                    proclamation_reader::press(&mut ui, press, session);
                }
                Action::Seat(press @ seat_cards::Press::TakeBack(_)) => {
                    ui.clicked = true;
                    ui.presses.push(press);
                }
                _ => {}
            }
        }
        return;
    }
    // A seat is handed over and taken back at any time, with a card in the air too, which
    // the click also lands.
    if let Ok(Control {
        action: Action::Seat(press),
        ..
    }) = controls.get(event.entity)
    {
        ui.clicked = true;
        ui.presses.push(*press);
        return;
    }
    // A click while a played card is in the air lands the cards and does nothing
    // else (presentation only: the advance was saved before anything moved).
    let view = play.game().observe();
    if advance.lands(&view, time.elapsed_secs_f64(), frame.0) {
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
    match &control.action {
        // Hover disclosures never change a decision.
        Action::Standing(_)
        | Action::Top(_)
        | Action::Sequence(_)
        | Action::Pile(_)
        | Action::Area => return,
        // A region click opens or closes a space popover while Terror targets are
        // chosen, and does nothing otherwise; it never decides anything itself. An
        // opened popover wins over the AC-B briefing tag, which reopens from the
        // BRIEFING ribbon (owner's clash rule, #58).
        Action::Space(space) => {
            // The transfer modal's scrim takes the map's input beneath it.
            let game = play.game();
            if transfer::shown(game.transfer.as_ref(), game.owner(), &ui) {
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
        Action::Submit(..) | Action::Seat(_) => return,
        // Reading an edition changes nothing in the match.
        Action::Edition(press) => newspaper::press(&mut ui, *press),
        Action::ComposeProclamation(press) => {
            if *press == proclamation_composer::Press::Open {
                if proclamation_composer::entry_available(&view, &ui, &roll, false) {
                    proclamation_composer::open(&play, &mut ui);
                }
            } else {
                proclamation_composer::press(&mut ui, *press);
            }
            return;
        }
        Action::ReadProclamation(press) => {
            let session = play.session.as_mut().expect("An admitted match");
            proclamation_reader::press(&mut ui, *press, session);
        }
        // Reading a card changes nothing in the match.
        Action::Card(slot) => cards::activate(&mut ui, *slot),
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
        Action::Unsupported(reason) => notice::refuse(&mut ui, reason.clone()),
        Action::DismissNotice => ui.notice.dismiss(),
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

fn consume_proclamation_keys(keys: &mut ButtonInput<KeyCode>) {
    let pressed = keys.get_just_pressed().copied().collect::<Vec<_>>();
    for key in pressed {
        if !faction_seats::KEYS.contains(&key) {
            keys.clear_just_pressed(key);
        }
    }
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
    time: Res<Time>,
    frame: Res<bevy::diagnostic::FrameCount>,
    mut advance: ResMut<piles::Advance>,
) {
    if event.button != bevy::picking::pointer::PointerButton::Primary
        || play.session.is_none()
        || nodes.contains(event.entity)
    {
        return;
    }
    // A click while a played card is in the air only lands it.
    if advance.lands(&play.game().observe(), time.elapsed_secs_f64(), frame.0) {
        return;
    }
    // A click on the map puts the sheet with a seat's line away.
    if ui.line.take().is_some() {
        ui.dirty = true;
    }
    let step = play.game().action == crate::ActionChoice::Terror && ui.choice.pinned.is_some();
    if !(ui.choice.ribbon || step) {
        return;
    }
    ui.choice.close();
    ui.dirty = true;
}
/// Submits a decision through the checked game seam and words its outcome. Pointer
/// clicks and the keyboard's Enter on a focused control both come here. It submits nothing
/// for a faction handed over to an agent; turning the card is the human's, whichever
/// faction the rules authority names for it (#197). While an edition is due or open it
/// refuses: the human decides knowing what it says (#208).
fn decide(play: &mut Play, ui: &mut Interface, owner: Faction, rev: u64, name: &str) {
    if ui.editions.holds() {
        return notice::refuse(ui, notice::EDITION_FIRST);
    }
    if !ui.plays(owner) && Choice::action_of(name) != crate::game::ADVANCE_CARD {
        return notice::refuse(
            ui,
            format!(
                "{0} is an agent's to play. Take {0} back on its card to decide.",
                owner.name()
            ),
        );
    }
    if let Err(failure) = submit(play, ui, owner, rev, name) {
        ui.notice = notice::Notice::failed(&failure);
    }
    // A committed decision leaves its step; its pin and focus go with it, and a step
    // the decision opened holds the panel with its tag pinned (EX-A, #84).
    ui.choice.decided(&play.game().observe());
}
/// Submits a decision, the human's or an agent's, through the checked submission, which
/// saves it, and has the table show it: the position it made.
fn submit(
    play: &mut Play,
    ui: &mut Interface,
    owner: Faction,
    rev: u64,
    name: &str,
) -> Result<Observation, crate::save::Submit> {
    let previous = play.game().observe();
    let session = play.session.as_mut().unwrap();
    let reply = session.submit(owner, rev, name)?;
    let after = session.observe();
    ui.marker_travel.committed(&previous, &after);
    if let Some(trips) = force_travel::committed(&previous, &after) {
        ui.force_travel = Some(trips);
    }
    let flips = piece_flips::committed(&previous, &after, name);
    if !flips.is_empty() {
        ui.piece_flips = flips;
    }
    let played = ui.played();
    ui.editions
        .decided(&after, owner, |told| session.reports_since(told), played);
    if name.ends_with(":kidnap:roll") {
        // The result is saved; the tray shows it from this frame's clock
        // (die::present), and the notice names it once the die lands.
        ui.rolled = Some(after.revision);
    }
    // A played card is saved before it moves; the tray shows it cross from
    // this frame's clock (piles::follow).
    ui.played = piles::Played::between(&previous, &after);
    ui.notice = notice::Notice::success(notice(&previous, &after, owner, name, reply.as_ref()));
    Ok(after)
}
/// A plate drawn in nine slices, its corners `slice` frame units at `unit` window pixels
/// a unit: the seat cards' and the newspaper's.
fn sliced(slice: f32, unit: f32) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(slice * 2.),
        max_corner_scale: unit / 2.,
        ..default()
    })
}
/// The notice for the committed decision `name`, a success: the seat reads it and no
/// slip draws it (#174). Every rules fact it states is the authority's (foundation 2,
/// "Engine authority"): the decision's own id, read through `Choice`'s accessors; the
/// preview the authority showed for it, from `previous`; the record in `after`; or the
/// committed funding reply.
fn notice(
    previous: &Observation,
    after: &Observation,
    owner: Faction,
    name: &str,
    reply: Option<&crate::ConstructFundingReply>,
) -> String {
    use crate::ConstructFundingReply::{Held, Settled};
    match reply {
        Some(Held {
            authorization,
            shortfall,
        }) => {
            return format!(
                "{} conditionally authorized {} for Syndicate. Syndicate still needs {}. Saved.",
                authorization.giver.name(),
                resources(authorization.amount),
                resources(*shortfall),
            );
        }
        Some(Settled(settlement)) if !settlement.earlier.is_empty() => {
            let final_reply = settlement.final_reply.authorization;
            return format!(
                "{} authorized {}. Syndicate received {} from the combined authorizations. Saved.",
                final_reply.giver.name(),
                resources(final_reply.amount),
                resources(settlement.total()),
            );
        }
        Some(Settled(settlement)) => {
            let final_reply = settlement.final_reply.authorization;
            return format!(
                "{} gave {} to Syndicate. Saved.",
                final_reply.giver.name(),
                final_reply.amount,
            );
        }
        None => {}
    }
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
                }) if after.construct_funding.is_some() => format!(
                    "Transfer request saved. {} asks {} for {amount}. {}",
                    recipient.name(),
                    giver.name(),
                    crate::ConstructFunding::CONDITION
                ),
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
    if let Some((id, _, n)) = choice.and_then(Choice::casino_closure) {
        return format!(
            "Closed {n} {} in {}. Choose Done to return to Construct. Saved.",
            if n == 1 { "Casino" } else { "Casinos" },
            place(id)
        );
    }
    if let Some((_, group, n)) = choice.and_then(Choice::removal) {
        if after.executing == Some(Faction::Syndicate) {
            let piece = match (crate::removal_piece(group), n) {
                (Some((Kind::Casino, _)), 1) => "Casino",
                (Some((Kind::Casino, _)), _) => "Casinos",
                (_, 1) => "Guerrilla",
                _ => "Guerrillas",
            };
            return format!(
                "Returned {n} {piece} to Available Forces. Choose Done to return to Construct. Saved."
            );
        }
        if after.current_card == 18 {
            return format!(
                "Returned {n} {} to Available Forces. Choose Done to review Assault. Saved.",
                if n == 1 { "piece" } else { "pieces" }
            );
        }
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
        return format!(
            "Terror saved in {}: {}.",
            place(space),
            crate::reports::listing(&done)
        );
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
        "remove" if owner == Faction::Syndicate => {
            "Piece management is open. Select a Syndicate piece on the map. Saved.".into()
        }
        "remove" => format!(
            "Removal mode: click a ringed {} piece on the map. Saved.",
            owner.name()
        ),
        "done-removal" if owner == Faction::Syndicate => "Returned to Construct. Piece changes are saved.".into(),
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
                crate::reports::listing(&gained)
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
        "back-to-choice"
            if previous.construct_funding.as_ref().is_some_and(|funding| funding.authorized() > 0) =>
        {
            "Back at the action choice. Pending requests and conditional authorizations are cleared. Saved.".into()
        }
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
        crate::game::MANIFESTO_EVENT => format!(
            "{} executes Sierra Maestra Manifesto. {} decides whether to place. Saved.",
            owner.name(),
            after.owner.name()
        ),
        action if crate::game::manifesto::is_placement(action) => {
            let placement = after.manifesto.as_ref().and_then(crate::Manifesto::placement_in_last_outcome)
                .expect("A committed placement has a record");
            let space = after.spaces.iter().find(|s| s.id == placement.space)
                .expect("A committed placement has a space");
            format!("{} placed {} in {}. Saved.", owner.name(), placement.piece_description(), space.name)
        }
        crate::game::MANIFESTO_DECLINE => {
            let next = match &after.next {
                Some(next) => format!("{} is {}.", next.faction.name(), next.role),
                None => format!("{} decides whether to place.", after.owner.name()),
            };
            format!("{} declined placement. {next} Saved.", owner.name())
        }
        "advance-card" => format!(
            "{} is current. {} is 1st Eligible. Saved.",
            after.cards.current.title,
            after.eligibility.first.name(),
        ),
        "government:assault" => "Assault chosen. Choose a space on the map. Saved.".into(),
        "end-assault" => "Operation Only recorded. Syndicate is 2nd Eligible. Saved.".into(),
        _ if crate::Game::assault_payment_space(action).is_some() => {
            let result = after.assault_paid.last().expect("Assault payment has a record");
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
/// One reference pixel in logical window pixels. The reference's top-left corner is
/// the window's, so an area anchored there needs no offset. An area keeps its
/// distance from the edge or centre its layout anchors it to (`areas::Anchor`): the
/// callers add the window's spare width for the right edge, half of it for the
/// centre, and [`bottom`] for the bottom edge. Nothing is letterboxed.
fn reference_pixel(window: &Window) -> f32 {
    (window.width() / WIDTH).min(window.height() / HEIGHT)
}
/// The smallest rectangle holding `bounds`, if any, and the point `p`.
fn grow(bounds: Option<Rect>, p: Vec2) -> Option<Rect> {
    Some(bounds.map_or(Rect::from_corners(p, p), |b| b.union_point(p)))
}
/// How far a bottom-anchored area lies below its place in the reference: the
/// window's spare height.
fn bottom(window: &Window) -> Vec2 {
    Vec2::new(0., window.height() - HEIGHT * reference_pixel(window))
}
/// What the table says to the shared camera: an unopened session, the transfer modal, an
/// open edition and a presented Kidnap die block it. The whole window shows the scene; the
/// camera itself leaves the pointer to any interface node it is over, and each
/// overlaid area has one (`areas::Area`). The die no longer needs the block for its
/// counts (#177); lifting it is the owner's design decision.
fn camera_facts(play: &Play, ui: &Interface, roll: &die::Roll) -> TableFacts {
    let (modal, die) = play.session.as_ref().map_or((false, false), |session| {
        let game = session.game();
        (
            transfer::shown(game.transfer.as_ref(), game.owner(), ui),
            die::presented(game.revision, game.kidnap.as_ref(), roll).is_some(),
        )
    });
    TableFacts {
        blocks_camera: !play.admitted()
            || modal
            || die
            || ui.editions.open().is_some()
            || ui.composer.is_open()
            || ui.proclamation_reader.is_open(),
        scene: Scene::WholeWindow,
        ..default()
    }
}
fn write_camera_facts(
    play: Res<Play>,
    ui: Res<Interface>,
    roll: Res<die::Roll>,
    mut table: ResMut<TableFacts>,
) {
    *table = camera_facts(&play, &ui, &roll);
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
    main_view: Res<areas::Layout>,
    advance: Res<piles::Advance>,
    time: Res<Time>,
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
    let s = reference_pixel(&window);
    let extra = window.width() - WIDTH * s;
    let right = Vec2::new(extra, 0.);
    let centered = Vec2::new(extra / 2., 0.);
    let view = play.observe();
    // The map fills the window (#142). Each area keeps its distance from the edge
    // or centre the main view's layout anchors it to, and takes the pointer over
    // its own rectangle.
    areas::draw(&mut commands, &main_view, &window, ui.editions.pile() > 0);
    let frame = picture(
        &mut commands,
        &art,
        "atlas-layout-operation.png",
        [1380., 280., 500., 660.],
        s,
        right,
        1,
    );
    commands
        .entity(frame)
        .insert(areas::Part("decision-frame".into()));
    victory::header(&mut commands, &art, &view, s, centered);
    // The owner retired the "CUBA LIBRE" title in #46: its corner, at the window's
    // top left, holds Aid and the US Alliance.
    top::header(
        &mut commands,
        &art,
        &die::ledger(&view, &roll),
        s,
        Vec2::ZERO,
        right,
    );
    // #159: Available Forces stand left of the Resources ledger, from the committed
    // position, so a preview never changes them.
    available::draw(&mut commands, &art, &view, s, right);
    // #172: the marker supply stands between the US Alliance track and the standings,
    // from the committed position.
    supply::draw(&mut commands, &art, &view, &main_view, &window);
    #[cfg(not(feature = "occupancy-review"))]
    let reviewing = false;
    #[cfg(feature = "occupancy-review")]
    let reviewing = if let Some(review) = &play.options.occupancy_review {
        text(
            &mut commands,
            &art,
            "Review only · Atlas occupancy",
            1408.,
            307.,
            444.,
            25.,
            s,
            right,
            2,
        );
        text(
            &mut commands,
            &art,
            review.name.clone(),
            1408.,
            356.,
            444.,
            20.,
            s,
            right,
            2,
        );
        text(
            &mut commands,
            &art,
            "Cash and Sabotage use temporary solids.",
            1408.,
            407.,
            444.,
            18.,
            s,
            right,
            2,
        );
        true
    } else {
        false
    };
    if !reviewing {
        if let Some(kidnap) = die::presented(view.revision, view.kidnap.as_ref(), &roll) {
            die::panel(&mut commands, &art, kidnap, roll.landed(), s, right);
        } else {
            execution::panel(&mut commands, &art, &view, &ui, s, right);
        }
    }
    eligibility::indicator(&mut commands, &art, &view, s, Vec2::ZERO);
    // The pips follow the cards while an advance is shown, and the count discs wait.
    let scene = advance.scene(&view, time.elapsed_secs_f64());
    piles::overlay(&mut commands, &art, &scene, &window);
    // #208: the pile of this card's editions beside the pips.
    newspaper::pile(&mut commands, &art, &view, &ui, &main_view, &window);
    let dispatch = proclamation_composer::entry_available(&view, &ui, &roll, scene.in_the_air());
    proclamation_composer::entry(&mut commands, &art, &view, &ui, dispatch, s);
    proclamation_reader::draw(&mut commands, &art, play.session.as_ref(), &ui, &window);
    // #206: the four seats between the pips and Save · quit.
    seat_cards::draw(&mut commands, &art, &view, &ui, &main_view, &window);
    // #174: Save · quit in the bottom-right corner, and above it and the seats the notice
    // slip when a refusal, an error or the loading state is shown.
    if play.session.is_some() || !play.admitted() {
        save_quit::draw(&mut commands, &art, &main_view, &window);
    }
    notice::draw(
        &mut commands,
        &art,
        notice::slip(&play, &ui),
        &main_view,
        &window,
    );
}

/// A table whose match is open on a fresh save at `save`, for unit tests of what a
/// decision writes.
#[cfg(test)]
fn test_play(save: PathBuf) -> Play {
    Play {
        session: Some(Session::open(&save, None).unwrap()),
        options: Options {
            art: PathBuf::new(),
            cards: PathBuf::new(),
            save,
            recover: None,
            socket: None,
            players: None,
            agent: false,
            #[cfg(feature = "occupancy-review")]
            occupancy_review: None,
            seat_size: UVec2::new(1920, 1280),
        },
        error: None,
        ready_frames: 5,
        #[cfg(feature = "occupancy-review")]
        review_ready: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unfolded_tabletop::camera::Input;

    /// #161: with the counts on the ledger, the removal texts still say where the
    /// pieces go (R 1.4.1): the preview's map line, and the notices of a removal, of
    /// Done, and of Government's removal before its card 18 Assault.
    #[test]
    fn removal_texts_say_the_pieces_return_to_available() {
        let mut terror = crate::terror_chosen(Game::opening());
        terror.apply("remove").unwrap();
        let view = terror.observe();
        let base = view
            .choices
            .iter()
            .find(|c| c.action() == "remove:sierra-maestra:base:1")
            .unwrap();
        let (entries, _) = execution::previewed(&view, base.preview.as_ref().unwrap());
        assert!(
            entries.iter().any(|e| e
                .text
                .contains("Sierra Maestra: 1 26July Base returns to Available")),
            "{entries:?}"
        );
        assert_eq!(
            noticed(&mut terror, "remove:santiago:underground:2", |_, _| {}),
            "Removed 2 own pieces to Available. Removal stays open until Done; earlier \
             Terror, support shifts and Kidnap results remain. Saved."
        );
        assert_eq!(
            noticed(&mut terror, "done-removal", |_, _| {}),
            "Removal mode closed; removed pieces stay in Available. Saved."
        );

        let mut funded = crate::terror_chosen(Game::opening_with_seed(2));
        for action in [
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "transfer",
            "give:0:1",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
            "remove",
        ] {
            funded.apply(action).unwrap();
        }
        assert_eq!(
            noticed(&mut funded, "remove:santiago:troops:1", |_, _| {}),
            "Returned 1 piece to Available Forces. Choose Done to review Assault. Saved."
        );
    }

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
        let reply = game.submit(owner, previous.revision, &id).unwrap();
        let mut after = game.observe();
        tamper(&mut previous, &mut after);
        notice(&previous, &after, owner, &id, reply.as_ref())
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
        let mut game = crate::terror_chosen(Game::opening());
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
        let mut funded = crate::terror_chosen(Game::opening_with_seed(5));
        funded.submit(Faction::M26, 1, "1:transfer").unwrap();
        funded
            .submit(
                Faction::M26,
                2,
                &format!("2:ask:{}:2", Faction::Syndicate.index()),
            )
            .unwrap();
        funded
            .submit(Faction::Syndicate, 3, "3:authorize:2")
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
        let mut game = crate::terror_chosen(Game::opening());
        game.submit(Faction::M26, 1, "1:transfer").unwrap();
        let asked = noticed(
            &mut game.clone(),
            &format!("ask:{}:2", Faction::Government.index()),
            |_, _| {},
        );
        assert_eq!(
            asked,
            "Transfer request saved: 26July asks Government for 2. Nothing moves until Government decides."
        );
        let mut gave = crate::terror_chosen(Game::opening());
        gave.submit(Faction::M26, 1, "1:transfer").unwrap();
        gave.submit(
            Faction::M26,
            2,
            &format!("2:ask:{}:2", Faction::Syndicate.index()),
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
        // #160: an Available Forces row, by the name the seat points at.
        assert_eq!(
            serde_json::to_value(Target::Top(top::Item::Available(Faction::M26))).unwrap(),
            "available:26july"
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
    /// The map is the whole window: the wheel zooms wherever the pointer is, unless
    /// the table blocks the camera. An area's own rectangle stops it natively (the
    /// main view regression), through the interface node the shared camera finds.
    #[test]
    fn camera_takes_input_over_the_whole_window() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("task.json");
        let mut play = test_play(save);
        let mut window = Window {
            resolution: (1920, 1280).into(),
            focused: true,
            ..default()
        };
        let roll = die::Roll::default();
        let ui = Interface::default();
        assert_eq!(camera_facts(&play, &ui, &roll).scene, Scene::WholeWindow);
        let mut orbit = Orbit::new(Framing::new(Vec3::ZERO, 10.));
        let keys = ButtonInput::default();
        let mut distance = 10.;
        // Over the old map opening, the old right margin, the old desk under the
        // map and the window's corner.
        for at in [
            Vec2::new(700., 500.),
            Vec2::new(1360., 500.),
            Vec2::new(1000., 1250.),
            Vec2::new(2., 2.),
        ] {
            window.set_cursor_position(Some(at));
            drive(
                &mut orbit,
                &window,
                &keys,
                1.,
                camera_facts(&play, &ui, &roll),
            );
            assert!(orbit.pose().distance < distance, "no zoom at {at}");
            distance = orbit.pose().distance;
        }
        play.session = None;
        drive(
            &mut orbit,
            &window,
            &keys,
            1.,
            camera_facts(&play, &ui, &roll),
        );
        assert_eq!(
            orbit.pose().distance,
            distance,
            "an unopened session blocks"
        );
    }

    /// Areas scale with the smaller of the window's width over 1920 and height over
    /// 1280, and the bottom row lies against the bottom edge in a taller window.
    #[test]
    fn nothing_is_letterboxed() {
        for (width, height, scale, low) in [
            (1920, 1280, 1., 0.),
            (3840, 2160, 1.6875, 0.),
            (2520, 1080, 0.84375, 0.),
            (1600, 1280, 1600. / 1920., 1280. - 1280. * 1600. / 1920.),
        ] {
            let window = Window {
                resolution: (width, height).into(),
                ..default()
            };
            let s = reference_pixel(&window);
            assert!((s - scale).abs() < 1e-6, "{width}x{height}");
            assert!((bottom(&window).y - low).abs() < 1e-3, "{width}x{height}");
        }
    }
}
