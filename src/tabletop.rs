//! Selected Atlas presentation. Input travels through Bevy picking into the checked game seam.
use crate::{
    Faction, Game, Kind, State, Terrain,
    assets::{self, Verified},
    save::Session,
};
use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, Viewport},
    core_pipeline::tonemapping::Tonemapping,
    image::{
        CompressedImageFormats, ImageAddressMode, ImageSampler, ImageSamplerDescriptor, ImageType,
    },
    input::mouse::AccumulatedMouseScroll,
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
    camera::{Capture, FrameInput, Orbit, wheel_steps},
    label_anchor::{LabelAnchor, LabelAnchorPlugin},
    token,
};
pub mod seat;
mod status;
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
            .map_or_else(Game::prepared, |s| s.game().clone())
    }
}
#[derive(Resource)]
struct Art {
    verified: Verified,
    images: BTreeMap<String, Handle<Image>>,
    font: Handle<Font>,
    scene: Handle<WorldAsset>,
    labels: Vec<LabelSpec>,
}
#[derive(serde::Deserialize, Clone)]
struct LabelSpec {
    space: String,
    bounds: [f32; 4],
    #[serde(default)]
    pieces: Option<[f32; 2]>,
}
#[derive(Resource, Default)]
struct Interface {
    hover: Hover<Faction>,
    status: Hover<String>,
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
#[derive(Component, Clone)]
struct Control {
    name: String,
    choice: bool,
    action: Action,
}
#[derive(Clone)]
enum Action {
    Standing(Faction),
    Close,
    Reader,
    Space(String),
    Decision(Faction, u64),
    Unsupported(String),
    Quit,
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
            plugins
                .set(unfolded_tabletop::capture::no_window())
                .disable::<bevy::winit::WinitPlugin>()
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
    let scene = app
        .world()
        .resource::<AssetServer>()
        .load("atlas-terrain-map.glb#Scene0");
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
            labels: labels.labels,
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
        .insert_resource(Orbit::new((Vec3::new(0.35, 0., 0.35), 9.6)))
        // Atlas SVG map rim fill; the viewport continues the authored sea.
        .insert_resource(ClearColor(Color::srgb_u8(216, 231, 227)))
        .add_plugins((MeshPickingPlugin, LabelAnchorPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                admit,
                sync_pieces,
                bind_spaces,
                hover_standings,
                draw_ui,
                camera_controls,
                status::hover,
                status::draw,
                victory::draw,
                animate_water,
            )
                .chain(),
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
fn setup(
    mut commands: Commands,
    art: Res<Art>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
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
            let token = token::spawn_body(
                &mut commands,
                &mut meshes,
                &mut materials,
                token::Body {
                    name: format!("{} Terror", space.id),
                    mesh: Cylinder::new(0.12, 0.045).into(),
                    color: Color::srgb(0.18, 0.18, 0.16),
                    transform: Transform::from_xyz(
                        piece_at.x + if space.id == "oriente" { 0.38 } else { 0. },
                        top + 0.0225,
                        piece_at.z + if space.id == "oriente" { 0. } else { 0.36 },
                    ),
                },
            );
            commands
                .entity(token)
                .insert((PieceVisual, status::SpaceTarget(space.id.clone())))
                .remove::<Pickable>();
            commands.spawn((
                count(
                    format!("T {}", space.terror),
                    token,
                    Vec3::new(0., 0.07, 0.),
                ),
                status::SpaceTarget(space.id.clone()),
                TextFont {
                    font: art.font.clone().into(),
                    font_size: 18.0.into(),
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

/// A count over its token. It stays pickable, so hovering it hovers its space.
fn count(text: String, token: Entity, offset: Vec3) -> impl Bundle {
    (
        Count,
        Text::new(text),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        LabelAnchor::entity(token, offset),
    )
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
    fonts: Res<Assets<Font>>,
    mut context: ResMut<bevy::text::FontCx>,
    ready: Res<RenderReady>,
    mut ui: ResMut<Interface>,
) {
    if play.error.is_some() || play.session.is_some() {
        return;
    }
    let report =
        unfolded_tabletop::loading::required_assets(&server, &[art.scene.id().untyped()], |_| {
            scenes.get(&art.scene).is_some()
        });
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
fn hover_standings(
    hover: Res<HoverMap>,
    controls: Query<&Control>,
    time: Res<Time>,
    mut ui: ResMut<Interface>,
) {
    let target = hover.get(&PointerId::Mouse).and_then(|hits| {
        hits.keys()
            .find_map(|e| match controls.get(*e).ok()?.action {
                Action::Standing(f) => Some(f),
                _ => None,
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
) {
    event.propagate(false);
    if event.button != bevy::picking::pointer::PointerButton::Primary || play.session.is_none() {
        return;
    }
    let Ok(control) = controls.get(event.entity) else {
        return;
    };
    if ui.reader && !matches!(control.action, Action::Close | Action::Quit) {
        return;
    }
    match &control.action {
        // Hover disclosures and development-seat map picks never change a decision.
        Action::Standing(_) | Action::Space(_) => return,
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
                        let view = play.game().observe();
                        let roll = view.kidnap.as_ref().unwrap();
                        format!(
                            "Kidnap rolled {} · transferred {}.\nGovernment {} · 26July {}. Saved.",
                            roll.die, roll.proceeds, view.resources[0], view.resources[1]
                        )
                    } else if control.name.ends_with(":confirm-removal") {
                        "Removed 1 own piece to Available. Control and standings updated; earlier Terror and Opposition remain.".into()
                    } else if control.name.ends_with(":cancel-removal") {
                        "Removal cancelled. No piece removed; earlier effects remain.".into()
                    } else if control.name.contains(":terror:") {
                        "Terror saved: paid 1, activated 1, placed marker and shifted Opposition."
                            .into()
                    } else if control.name.ends_with(":finish") {
                        format!(
                            "Saved {}. No Pass income.",
                            if previous.kidnap.is_some() {
                                "Operation + Kidnap"
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
#[allow(clippy::too_many_arguments)]
fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    window: Single<&Window, With<PrimaryWindow>>,
    play: Res<Play>,
    ui: Res<Interface>,
    time: Res<Time>,
    mut orbit: ResMut<Orbit>,
    mut cameras: Query<(&mut Transform, &mut Camera), With<Camera3d>>,
) {
    let (scale, offset) = layout(&window);
    let min = offset + Vec2::new(40., 176.) * scale;
    let size = Vec2::new(window.width() - 620. * scale, 684. * scale);
    let outside = window.cursor_position().is_none_or(|p| {
        p.x < min.x || p.y < min.y || p.x >= min.x + size.x || p.y >= min.y + size.y
    });
    let blocked = ui.reader || play.session.is_none();
    let step = orbit.step(FrameInput {
        keys: &keys,
        buttons: &buttons,
        window: &window,
        scroll_y: wheel_steps(&scroll),
        dt: time.delta_secs(),
        capture: Capture {
            pointer_blocked: blocked,
            over_ui: outside,
            keyboard: blocked,
            wheel: blocked || outside,
            pan: Some(blocked),
        },
    });
    for (mut transform, mut camera) in &mut cameras {
        *transform = step.transform;
        let physical_position = (min * window.scale_factor()).round().as_uvec2();
        let physical_end = ((min + size) * window.scale_factor()).round().as_uvec2();
        camera.viewport = Some(Viewport {
            physical_position,
            physical_size: physical_end - physical_position,
            ..default()
        });
    }
}
#[allow(clippy::too_many_arguments)] // Authored rectangle, type size and letterboxing coordinates.
fn text(
    commands: &mut Commands,
    art: &Art,
    content: impl Into<String>,
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    scale: f32,
    offset: Vec2,
    z: i32,
) -> Entity {
    commands
        .spawn((
            UiRoot,
            Text::new(content),
            TextFont {
                font: art.font.clone().into(),
                font_size: (size * scale).into(),
                ..default()
            },
            TextColor(Color::srgb(0.12, 0.14, 0.12)),
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
        .id()
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
    let words = text(
        commands,
        art,
        label,
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
        for (key, rect, offset) in [
            ("atlas-layout-title.png", [40., 0., 240., 64.], left),
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
                "Aid {} · US Alliance {} · Capabilities: {} · Momentum: {}\nEligible: {}\nPassed: {}",
                view.aid,
                view.alliance,
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
        let o = right;
        // The selected blank component hides static example action affordances
        // after completion, so they cannot masquerade as current controls.
        picture(
            &mut commands,
            &art,
            "atlas-panel.png",
            [1380., 287., 500., 653.],
            s,
            o,
            2,
        );
        text(
            &mut commands,
            &art,
            if view.finished {
                format!(
                    "26July completed {}\nNo Pass income",
                    if view.kidnap.is_some() {
                        "Operation + Kidnap"
                    } else {
                        "Operation Only"
                    }
                )
            } else if view.transfer.is_some() {
                format!("Resource transfer\n{} decides locally", view.owner.name())
            } else if view.removal.is_some() {
                "26July · voluntary own removal".into()
            } else {
                format!("{} · {}", view.owner.name(), view.operation)
            },
            1404.,
            330.,
            450.,
            25.,
            s,
            o,
            3,
        );
        text(
            &mut commands,
            &art,
            match &view.transfer {
                Some(crate::Transfer::Selecting) => "Choose the giver and recipient.\n26July must be one endpoint.\nThe giver chooses the quantity and authorizes.\nA request is not a donation.".into(),
                Some(crate::Transfer::Authorizing { giver, recipient, amount }) => format!(
                    "Giver: {} · balance {}\nRecipient: {} · balance {}\nAmount: {} Resources · whole numbers only\nAfter authorization: {} → {}; {} → {}\nZero is allowed. Decline moves nothing.",
                    giver.name(), view.resources[giver.index()], recipient.name(), view.resources[recipient.index()], amount,
                    giver.name(), view.resources[giver.index()] - amount, recipient.name(), view.resources[recipient.index()] + amount),
                None => match &view.removal {
                    Some(crate::Removal::Selecting) => format!("Choose one own piece to return to Available.\nCounts show how many remain in each group.\nConfirmation comes next; Cancel removes nothing.\n{}", if view.owed_terror.is_some() { "Owed Santiago: protect its last Underground." } else { "No obligation: removing all pieces is allowed." }),
                    Some(crate::Removal::Confirming { space, kind, state }) => {
                        let target = view.spaces.iter().find(|s| &s.id == space).unwrap();
                        let supply = view.available.iter().find(|p| p.faction == Faction::M26 && p.kind == *kind).unwrap().count;
                        format!("Remove 1 26July {state:?} {kind:?}\nfrom {}?\nAvailable {kind:?}: {supply} → {}\nControl and standings update immediately.\nEarlier Terror and Opposition stay.\nCancel leaves this piece on the map.", target.name, supply + 1)
                    }
                    None => operation_details(&view),
                },
            } + &if view.transfer.is_some() && view.owed_terror.is_some() {
                let owed = view.spaces.iter().find(|space| Some(&space.id) == view.owed_terror.as_ref()).unwrap();
                format!("\nCommitment: {} Terror owed; reserve 1.\n26July available to give: {} Resources.", owed.name, view.resources[Faction::M26.index()].saturating_sub(1))
            } else {
                String::new()
            },
            1404.,
            390.,
            450.,
            18.,
            s,
            o,
            3,
        );
        if !view.finished {
            for (i, choice) in view.choices.iter().enumerate() {
                control(
                    &mut commands,
                    &art,
                    &choice.id,
                    &if !choice.supported {
                        format!("{} — unavailable", choice.label)
                    } else {
                        choice.label.clone()
                    },
                    if choice.supported {
                        Action::Decision(view.owner, view.revision)
                    } else {
                        Action::Unsupported(choice.explanation.clone())
                    },
                    [
                        1404.,
                        (if view.transfer.is_some() {
                            570.
                        } else if view.removal.is_some() {
                            540.
                        } else {
                            590.
                        }) + i as f32 * 52.,
                        452.,
                        46.,
                    ],
                    s,
                    o,
                    choice.supported,
                    3,
                );
            }
        } else {
            text(
                &mut commands,
                &art,
                format!(
                    "Directorio next (display only):\n{}\n\n{}\nCard / eligibility have not advanced.",
                    view.next_options.join("\n"),
                    if view.kidnap.is_some() {
                        "Kidnap grants Event permission."
                    } else {
                        "No Event permission."
                    }
                ),
                1404.,
                590.,
                450.,
                25.,
                s,
                o,
                3,
            );
        }
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
        let status=play.error.clone().unwrap_or_else(||if play.session.is_none(){"Loading selected assets; no save created.".into()}else{format!("Saved revision {} · {} map / {} available\nCash 4 · Terror/Sabotage {} available",view.revision,view.map_piece_count,view.available_piece_count,view.terror_available)});
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

fn operation_details(view: &crate::Observation) -> String {
    let mut lines = vec![];
    for id in ["santiago", "oriente"] {
        let space = view.spaces.iter().find(|s| s.id == id).unwrap();
        let count = |faction, kind, state: Option<State>| -> u8 {
            space
                .pieces
                .iter()
                .filter(|p| {
                    p.faction == faction && p.kind == kind && state.is_none_or(|s| p.state == s)
                })
                .map(|p| p.count)
                .sum()
        };
        lines.push(format!(
            "{}: {} Underground · {} Active · {} Police{}",
            if id == "santiago" {
                "Santiago"
            } else {
                "Oriente"
            },
            count(Faction::M26, Kind::Guerrilla, Some(State::Underground)),
            count(Faction::M26, Kind::Guerrilla, Some(State::Active)),
            count(Faction::Government, Kind::Police, None),
            if space.terror > 0 { " · done" } else { "" }
        ));
    }
    if view.finished {
        let paid: u8 = view.spaces.iter().map(|s| s.terror).sum();
        lines.push(format!("Resolved {paid} Terror · paid {paid} Resources"));
        lines.push(format!("26July retains {} Resources", view.resources[1]));
    } else {
        lines.push(format!(
            "Terror costs 1 · 26July Resources {}",
            view.resources[1]
        ));
        lines.push("Activate 1 · place Terror · shift toward Active Opposition".into());
    }
    if let Some(roll) = &view.kidnap {
        lines.push(format!(
            "Kidnap rolled {} · moved {} (Government → 26July)",
            roll.die, roll.proceeds
        ));
        lines.push(format!(
            "At roll: Government {} → {} · 26July {} → {}",
            roll.government_before,
            roll.government_before - roll.proceeds,
            roll.m26_before,
            roll.m26_before + roll.proceeds
        ));
    } else if view.finished {
        lines.push("No Kidnap used.".into());
    } else {
        lines.push(format!(
            "Kidnap die 1–6 · possible proceeds {}–{}",
            view.resources[0].min(1),
            view.resources[0].min(6)
        ));
        lines.push("Santiago: total Guerrillas must outnumber Police".into());
        if view
            .spaces
            .iter()
            .any(|s| s.id == "santiago" && s.terror == 0)
        {
            lines.push("Early Kidnap commits Santiago Terror (pay 1).".into());
        }
    }
    if !view.finished {
        lines.push(
            if view.owed_terror.is_some() {
                "Owed: Santiago · keep 1 Resource + 1 Underground"
            } else {
                "No Terror owed · Finish permitted"
            }
            .into(),
        );
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    #[test]
    fn camera_input_respects_authored_viewport_and_reader_capture() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("task.json");
        let mut world = World::new();
        world.insert_resource(Play {
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
        });
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<ButtonInput<MouseButton>>();
        world.insert_resource(AccumulatedMouseScroll {
            delta: Vec2::new(0., 1.),
            ..default()
        });
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(1. / 60.));
        world.insert_resource(time);
        world.init_resource::<Interface>();
        world.insert_resource(Orbit::new((Vec3::ZERO, 10.)));
        let window = world
            .spawn((
                Window {
                    resolution: (1920, 1280).into(),
                    focused: true,
                    ..default()
                },
                PrimaryWindow,
            ))
            .id();
        world.spawn((Camera3d::default(), Camera::default(), Transform::default()));
        world
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(700., 500.)));
        world.run_system_once(camera_controls).unwrap();
        assert!(world.resource::<Orbit>().pose().distance < 10.);
        let inside = world.resource::<Orbit>().pose();
        world
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(1500., 500.)));
        world.run_system_once(camera_controls).unwrap();
        assert_eq!(world.resource::<Orbit>().pose(), inside);
        // Maximizing a 3:2 design onto wider screens must put the extra
        // horizontal room into the map, retaining the 40 px outer inset.
        world.resource_mut::<AccumulatedMouseScroll>().delta = Vec2::ZERO;
        for width in [2560, 3360] {
            world
                .get_mut::<Window>(window)
                .unwrap()
                .resolution
                .set(width as f32, 1440.);
            world.run_system_once(camera_controls).unwrap();
            let viewport = world
                .query_filtered::<&Camera, With<Camera3d>>()
                .single(&world)
                .unwrap()
                .viewport
                .as_ref()
                .unwrap();
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
        world.resource_mut::<Interface>().reader = true;
        world
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(700., 500.)));
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        world.run_system_once(camera_controls).unwrap();
        assert_eq!(world.resource::<Orbit>().pose(), inside);
    }
}
