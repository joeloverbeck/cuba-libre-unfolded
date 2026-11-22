//! The Kidnap die (#74; [ADR 0001]; spec #58 "The die presentation"). The roll is
//! committed and saved before anything moves; this module only shows it.
//!
//! - **Asset:** the selected box die and its flat tray (#68: box die, tray placement C),
//!   baked in Blender as four tumbles on the die's `Die tumble` node
//!   (`art/atlas/KIDNAP.md`). `kidnap-die.json` records each tumble's landing time,
//!   landed face and final rotation, and each face's normal and art direction.
//! - **The face:** a static rotation of `Die body`, below the animated node, puts the
//!   committed face up. Of the four cube rotations that show it, the one whose art reads
//!   upright to the tray camera is used (the free yaw, #67). Only the tray's place and a
//!   half turn sit above the animated node.
//! - **The tumble** and its half turn come from the revision the roll created
//!   ([`tumble`]), never from the shared random generator, OS entropy or the die.
//! - **Playback** seeks the clip at `min(RATE × (now − start), duration)` from the
//!   shared clock, with `start` set when this app applied the commit. A loaded or
//!   skipped roll has no start and shows the landed pose; nothing about the
//!   presentation is saved.
//! - **The tray** is its own UI area inside the operation panel, drawn by its own camera
//!   on its own render layer, so it never shows on the map or takes a region pick. The
//!   owner selected tray placement C (#68) and the AC-B panel (#69), which overlap: the
//!   AC-B panel's die-presented frame in `art/atlas/atlas-action-choice.figcraft`
//!   composes them (`art/atlas/atlas-kidnap-tray.layout.json`). While the die is
//!   presented, frame C's tray, result text and Continue Terror take AC-B's execution
//!   content area below the BRIEFING ribbon, and Continue takes AC-B's primary action.
//!
//! [ADR 0001]: ../../../docs/adr/0001-baked-die-tumbles-land-on-the-committed-face.md
use super::palette::{DESK, INK, QUIET};
use super::*;
use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    light::CascadeShadowConfigBuilder,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::f32::consts::PI;

/// Playback rate of the baked tumbles. The owner asked to "slow it down" (#68). Half
/// speed is exactly the same trajectory as a re-bake under a quarter of real gravity, or
/// the real fall of a die four times the size: friction and restitution have no time
/// unit, so the bounces, slides and settle keep their physical proportions, only longer.
/// The motion of 0.32–0.38 s becomes 0.63–0.77 s, the time a thrown die takes on a table;
/// below about half speed a bounce's hang time starts to read as slow motion. Every
/// clip-admission condition is a property of the keys, so a rate leaves them all intact.
/// The owner judges the result in #16.
pub const RATE: f64 = 0.5;
/// Screen up on the tray floor, toward which a landed face's art should point: the tray
/// camera looks from +Z toward −Z (Blender's preview camera on its −Y side, #67).
pub const SCREEN_UP: Vec3 = Vec3::NEG_Z;
/// The baked tumbles, `Tumble A` to `Tumble D`.
pub const CLIPS: usize = 4;
/// The tray's render layer: only its camera and light see it.
const LAYER: usize = 1;
/// Where the tray stands in the world: well below the map's sea and on its own layer.
const TRAY_AT: Vec3 = Vec3::new(0., -50., 0.);
/// The die-presented frame at the 1920×1280 reference (the 6144×3456 frame ÷ 2.7, with
/// the right-anchored panels' 960-unit letterbox removed): frame C's tray, moved down
/// 120 px so that it starts 8 px below the BRIEFING ribbon, at its full width, which the
/// 70 px floor of the landed face needs. `atlas-kidnap-tray.layout.json` records the
/// frame's own bounds; the native regression compares the two.
pub(super) const VIEWPORT: [f32; 4] = [1381.111, 400., 497.778, 379.259];
/// Panel title, result title and result detail: left, top and type size.
const TITLE: [f32; 3] = [1407.407, 307.407, 26.667];
/// The title's box ends where AC-B's header ends (its 118.8-unit right padding, 1812 px),
/// clear of the BRIEFING ribbon at 1820.
const TITLE_WIDTH: f32 = 1812. - 1407.407;
const RESULT: [f32; 3] = [1407.407, 800., 26.667];
const DETAIL: [f32; 3] = [1407.407, 835.185, 20.741];
/// Continue takes AC-B's primary action at the panel's foot (`choice::primary`).
const CONTINUE_LABEL: &str = "Continue Terror";
/// The Continue control's stable seat name.
const CONTINUE: &str = "kidnap-die:continue";
/// The tray's stable seat name: a click on it skips the tumble.
pub(super) const TRAY: &str = "kidnap-die";
/// The tray camera of #67's placement C: 80° above the floor, Blender's 50 mm lens on a
/// 36 mm sensor across the viewport's width, distance fitted so the inner floor fills
/// 2–98% of the frame.
const ELEVATION: f32 = 80. * PI / 180.;
const HALF_WIDTH_TAN: f32 = 18. / 50.;
/// Half the tray's 96 × 64 mm inner floor, in metres.
const INNER: [f32; 2] = [0.048, 0.032];
/// #67's preview sun (Blender Euler 22°, −14°, 25°), as the direction it shines.
const SUN: Vec3 = Vec3::new(0.045, -0.900, -0.434);

/// What `kidnap-die.json` records of the shipped die.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Record {
    pub clips: Vec<ClipRecord>,
    pub faces: Vec<FaceRecord>,
}
/// One baked tumble: its length and landing in seconds of clip time, the face up when
/// `Die body` has no rotation, and the tumble node's last rotation (x, y, z, w).
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ClipRecord {
    pub name: String,
    pub duration: f32,
    pub landing: f32,
    pub landed_face: u8,
    pub end_rotation: [f32; 4],
}
/// A face's outward normal and the up direction of its art, in the die's glTF frame.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct FaceRecord {
    pub face: u8,
    pub normal: [f32; 3],
    pub up: [f32; 3],
}

/// Which baked tumble plays, and whether it is turned half about the tray's centre. The
/// tray is symmetric under a half turn, so each tumble's wall clearance holds either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tumble {
    pub clip: usize,
    pub half_turn: bool,
}

/// The tumble for a roll, from the revision it created alone: never from the die, the
/// shared random generator or OS entropy, so the same tumble does not keep pairing with
/// the same numbers, and a restart shows the same landed pose.
pub fn tumble(revision: u64) -> Tumble {
    let digest = Sha256::digest(format!("cuba-libre kidnap tumble {revision}"));
    Tumble {
        clip: usize::from(digest[0]) % CLIPS,
        half_turn: digest[1] & 1 == 1,
    }
}

/// The die's placement above the animated node: a half turn or none.
pub fn placement(tumble: Tumble) -> Quat {
    if tumble.half_turn {
        Quat::from_rotation_y(PI)
    } else {
        Quat::IDENTITY
    }
}

/// The cube's 24 rotations.
pub fn cube_rotations() -> Vec<Quat> {
    let axes = [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ];
    let mut rotations = vec![];
    for x in axes {
        for y in axes {
            if x.dot(y).abs() < 0.5 {
                rotations.push(Quat::from_mat3(&Mat3::from_cols(x, y, x.cross(y))));
            }
        }
    }
    rotations
}

/// `Die body`'s static rotation for `face` at the end of `tumble`: the cube rotation
/// that puts the face up, and of the four that do, the one whose art reads most nearly
/// upright to the tray camera.
pub fn body_rotation(record: &Record, tumble: Tumble, face: u8) -> Quat {
    let landed = placement(tumble) * Quat::from_array(record.clips[tumble.clip].end_rotation);
    let face = record
        .faces
        .iter()
        .find(|f| f.face == face)
        .expect("a die face from 1 to 6");
    let (normal, up) = (Vec3::from_array(face.normal), Vec3::from_array(face.up));
    let upright = |r: &Quat| (landed * *r * up).dot(SCREEN_UP);
    cube_rotations()
        .into_iter()
        .filter(|r| (landed * *r * normal).y > 0.5)
        .max_by(|a, b| upright(a).total_cmp(&upright(b)))
        .expect("a cube rotation shows every face")
}

/// Clip time for a roll whose commit this app applied at `start`:
/// `min(RATE × (now − start), duration)`. Without a start (loaded or skipped), the end.
pub fn seek(start: Option<f64>, now: f64, duration: f32) -> f32 {
    start.map_or(duration, |start| {
        (((now - start) * RATE) as f32).clamp(0., duration)
    })
}

/// The loaded die and tray, their four clips and the record.
#[derive(Resource)]
pub(super) struct DieArt {
    pub die: Handle<WorldAsset>,
    pub tray: Handle<WorldAsset>,
    pub clips: Vec<Handle<AnimationClip>>,
    graph: Handle<AnimationGraph>,
    nodes: Vec<AnimationNodeIndex>,
    record: Record,
}
impl DieArt {
    pub fn load(app: &mut App, record: &[u8]) -> Result<Self, String> {
        let record: Record = serde_json::from_slice(record).map_err(|e| e.to_string())?;
        if record.clips.len() != CLIPS || record.faces.len() != 6 {
            return Err("The Kidnap die record must list 4 tumbles and 6 faces".into());
        }
        let server = app.world().resource::<AssetServer>();
        let die = server.load("atlas-kidnap-die.glb#Scene0");
        let tray = server.load("atlas-kidnap-tray.glb#Scene0");
        let clips: Vec<Handle<AnimationClip>> = (0..CLIPS)
            .map(|i| server.load(format!("atlas-kidnap-die.glb#Animation{i}")))
            .collect();
        let (graph, nodes) = AnimationGraph::from_clips(clips.iter().cloned());
        let graph = app
            .world_mut()
            .resource_mut::<Assets<AnimationGraph>>()
            .add(graph);
        Ok(Self {
            die,
            tray,
            clips,
            graph,
            nodes,
            record,
        })
    }
}

/// The presentation's own state, which is never saved.
#[derive(Resource, Default)]
pub(super) struct Roll {
    /// The revision a roll created in this run, and the clock time when it was applied.
    start: Option<(u64, f64)>,
    /// The revision whose landed die the player closed with Continue.
    closed: Option<u64>,
    /// Whether the shown die has landed, as of this frame.
    landed: bool,
}
impl Roll {
    /// The running app applied a roll that created `revision`, at clock time `now`.
    pub fn commit(&mut self, revision: u64, now: f64) {
        *self = Self {
            start: Some((revision, now)),
            closed: None,
            landed: false,
        };
    }
    pub fn skip(&mut self) {
        self.start = None;
    }
    pub fn close(&mut self, revision: u64) {
        self.closed = Some(revision);
    }
    pub fn landed(&self) -> bool {
        self.landed
    }
    fn start(&self, revision: u64) -> Option<f64> {
        self.start
            .filter(|(at, _)| *at == revision)
            .map(|(_, start)| start)
    }
}

/// The roll the tray presents: one made by the latest decision, until Continue closes
/// it. A #8 roll records revision 0 and is never presented.
pub(super) fn presented<'a>(
    revision: u64,
    kidnap: Option<&'a crate::Kidnap>,
    roll: &Roll,
) -> Option<&'a crate::Kidnap> {
    kidnap.filter(|k| k.revision == revision && revision > 0 && roll.closed != Some(revision))
}

/// The tray's world, spawned for one presented roll.
#[derive(Component)]
pub(super) struct Stage {
    revision: u64,
    tumble: Tumble,
    face: u8,
}
#[derive(Component)]
pub(super) struct TrayCamera;
/// Marks the die's glTF player once it plays the stage's tumble.
#[derive(Component)]
pub(super) struct DiePlayer;
/// A text of the presentation, by its FigCraft layer name, for the seat.
#[derive(Component)]
pub(super) struct Part(&'static str);

/// The ledger while the die is in the air: the balances before the roll. On landing or
/// skip it shows the transfer, Government → 26July.
pub(super) fn ledger(view: &crate::Observation, roll: &Roll) -> crate::Observation {
    let mut shown = view.clone();
    if let Some(kidnap) = presented(view.revision, view.kidnap.as_ref(), roll)
        && !roll.landed
    {
        shown.resources[Faction::Government.index()] = kidnap.government_before;
        shown.resources[Faction::M26.index()] = kidnap.m26_before;
    }
    shown
}

/// The tray's rectangle in physical window pixels, inside the window.
fn viewport(window: &Window) -> Viewport {
    let (scale, left) = layout(window);
    let right = left + Vec2::new(window.width() - WIDTH * scale, 0.);
    let [x, y, w, h] = VIEWPORT;
    let factor = window.scale_factor();
    let limit = UVec2::new(window.physical_width(), window.physical_height());
    let min = ((right + Vec2::new(x, y) * scale) * factor)
        .round()
        .as_uvec2()
        .min(limit.saturating_sub(UVec2::ONE));
    let max = ((right + Vec2::new(x + w, y + h) * scale) * factor)
        .round()
        .as_uvec2()
        .min(limit)
        .max(min + UVec2::ONE);
    Viewport {
        physical_position: min,
        physical_size: max - min,
        ..default()
    }
}

/// The camera's distance from the tray's centre that fits the inner floor's corners into
/// 2–98% of the frame, as #67's Blender framing did.
fn fitted_distance(fov: f32, aspect: f32) -> f32 {
    let (half_height, half_width) = ((fov / 2.).tan(), (fov / 2.).tan() * aspect);
    let fits = |distance: f32| {
        let eye = Vec3::new(0., ELEVATION.sin(), ELEVATION.cos()) * distance;
        let camera = Transform::from_translation(eye).looking_at(Vec3::ZERO, Vec3::Y);
        [-1., 1.].into_iter().all(|sx: f32| {
            [-1., 1.].into_iter().all(|sz: f32| {
                // In the camera's frame it looks along −Z.
                let at =
                    camera.rotation.inverse() * (Vec3::new(sx * INNER[0], 0., sz * INNER[1]) - eye);
                let depth = -at.z;
                (at.x / (depth * half_width)).abs() <= 0.96
                    && (at.y / (depth * half_height)).abs() <= 0.96
            })
        })
    };
    let (mut near, mut far) = (0.05, 2.0);
    for _ in 0..40 {
        let middle = (near + far) / 2.;
        if fits(middle) {
            far = middle;
        } else {
            near = middle;
        }
    }
    far
}

/// The tray camera's framing, in the tray's frame (floor top at 0, +Y up): its eye,
/// which looks at the floor's centre with +Y up, its vertical field of view in radians
/// and its aspect ratio. The size of the viewport in reference pixels is
/// [`viewport_size`]; tests measure the landed face through this same framing.
pub fn tray_camera() -> (Vec3, f32, f32) {
    let [.., w, h] = VIEWPORT;
    let aspect = w / h;
    let fov = 2. * (HALF_WIDTH_TAN / aspect).atan();
    let distance = fitted_distance(fov, aspect);
    (
        Vec3::new(0., ELEVATION.sin(), ELEVATION.cos()) * distance,
        fov,
        aspect,
    )
}

/// The tray viewport's width and height at the 1920×1280 reference.
pub fn viewport_size() -> [f32; 2] {
    [VIEWPORT[2], VIEWPORT[3]]
}

/// #67's legibility floor for the landed top face at 1920×1080, in pixels: the square
/// root of the projected area of its 16 mm square. The study states it as "about
/// 70 px"; 69.5 px is that approximation made explicit (orchestrator judgement at the
/// #58 review), so Tumble A with the half turn, at 69.8 px, meets it.
pub const FACE_FLOOR: f32 = 69.5;

fn spawn_stage(commands: &mut Commands, art: &DieArt, revision: u64, face: u8, window: &Window) {
    let tumble = tumble(revision);
    let (eye, fov, aspect) = tray_camera();
    let layer = RenderLayers::layer(LAYER);
    commands
        .spawn((
            Name::new("Kidnap tray stage"),
            Stage {
                revision,
                tumble,
                face,
            },
            Transform::from_translation(TRAY_AT),
            Visibility::default(),
        ))
        .with_children(|stage| {
            stage.spawn((
                Name::new("Kidnap tray"),
                WorldAssetRoot(art.tray.clone()),
                Transform::default(),
            ));
            stage.spawn((
                Name::new("Kidnap die"),
                WorldAssetRoot(art.die.clone()),
                Transform::from_rotation(placement(tumble)),
            ));
            stage.spawn((
                DirectionalLight {
                    illuminance: 3000.,
                    shadow_maps_enabled: true,
                    // The defaults are sized for a map in metres-to-decimetres; at a
                    // 16 mm die they would bias the whole shadow away.
                    shadow_depth_bias: 0.0006,
                    shadow_normal_bias: 0.6,
                    ..default()
                },
                CascadeShadowConfigBuilder {
                    num_cascades: 1,
                    minimum_distance: 0.001,
                    maximum_distance: 1.,
                    first_cascade_far_bound: 1.,
                    ..default()
                }
                .build(),
                layer.clone(),
                Transform::default().looking_to(SUN, Vec3::Y),
            ));
            stage.spawn((
                Name::new("Kidnap tray camera"),
                Camera3d::default(),
                TrayCamera,
                // Renders the seat's image after the UI camera, over the panel.
                unfolded_tabletop::capture::OnSurface,
                Tonemapping::None,
                DebandDither::Disabled,
                Camera {
                    order: 2,
                    clear_color: ClearColorConfig::None,
                    viewport: Some(viewport(window)),
                    ..default()
                },
                Projection::Perspective(PerspectiveProjection {
                    fov,
                    aspect_ratio: aspect,
                    near: 0.005,
                    far: 5.,
                    ..default()
                }),
                // #67's preview world: a warm desk fill.
                AmbientLight {
                    color: DESK,
                    brightness: 400.,
                    ..default()
                },
                layer,
                Transform::from_translation(eye).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}

/// The words that report a landed roll, in the session notice. Kidnap proceeds are a
/// movement the rules require, not a Resource transfer (glossary), so the notice says
/// Government pays them; running text draws both names as emblems.
fn landed_notice(view: &crate::Observation, kidnap: &crate::Kidnap) -> String {
    format!(
        "Kidnap rolled {} · Government pays {} to 26July.\nGovernment {} · 26July {}. Saved.",
        kidnap.die,
        crate::resources(kidnap.proceeds),
        view.resources[Faction::Government.index()],
        view.resources[Faction::M26.index()]
    )
}

/// Keeps the tray in step with the saved roll: spawns it for a presented roll and
/// removes it otherwise, takes Space or Enter (skip while the die is in the air,
/// Continue once it has landed), and redraws the panel and notice when the die lands.
#[allow(clippy::too_many_arguments)]
pub(super) fn present(
    mut commands: Commands,
    play: Res<Play>,
    art: Res<DieArt>,
    mut roll: ResMut<Roll>,
    mut ui: ResMut<Interface>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    stages: Query<(Entity, &Stage)>,
    mut cameras: Query<&mut Camera, With<TrayCamera>>,
) {
    if let Some(revision) = ui.rolled {
        ui.rolled = None;
        roll.commit(revision, time.elapsed_secs_f64());
    }
    let game = play.session.as_ref().map(Session::game);
    let shown = game.and_then(|g| presented(g.revision, g.kidnap.as_ref(), &roll));
    let revision = game.map_or(0, |g| g.revision);
    let mut spawned = false;
    for (entity, stage) in &stages {
        if shown.is_some_and(|k| stage.revision == revision && stage.face == k.die) {
            spawned = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(kidnap) = shown else {
        roll.landed = false;
        return;
    };
    if !spawned {
        spawn_stage(&mut commands, &art, revision, kidnap.die, &window);
    }
    if keys.any_just_pressed([KeyCode::Space, KeyCode::Enter]) {
        if roll.landed {
            roll.close(revision);
            ui.dirty = true;
            return;
        }
        roll.skip();
    }
    let clip = &art.record.clips[tumble(revision).clip];
    let landed = seek(roll.start(revision), time.elapsed_secs_f64(), clip.duration) >= clip.landing;
    if landed && !roll.landed {
        ui.notice = landed_notice(&play.game().observe(), kidnap);
        ui.dirty = true;
    }
    roll.landed = landed;
    for mut camera in &mut cameras {
        camera.viewport = Some(viewport(&window));
    }
}

/// Poses the die: binds its glTF player to the stage's tumble and seeks it from the
/// shared clock, sets `Die body`'s face rotation, and keeps every tray mesh on the
/// tray's layer and out of picking.
#[allow(clippy::too_many_arguments)]
pub(super) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<DieArt>,
    roll: Res<Roll>,
    stages: Query<&Stage>,
    parents: Query<&ChildOf>,
    names: Query<(Entity, &Name)>,
    mut transforms: Query<&mut Transform>,
    mut players: Query<(Entity, &mut AnimationPlayer, Has<DiePlayer>)>,
    meshes: Query<Entity, (With<Mesh3d>, Without<RenderLayers>)>,
) {
    let stage_of = |entity: Entity| {
        parents
            .iter_ancestors(entity)
            .find_map(|ancestor| stages.get(ancestor).ok())
    };
    for entity in &meshes {
        if stage_of(entity).is_some() {
            commands
                .entity(entity)
                .insert((RenderLayers::layer(LAYER), Pickable::IGNORE));
        }
    }
    for (entity, name) in &names {
        if name.as_str() == "Die body"
            && let Some(stage) = stage_of(entity)
            && let Ok(mut transform) = transforms.get_mut(entity)
        {
            transform.rotation = body_rotation(&art.record, stage.tumble, stage.face);
        }
    }
    for (entity, mut player, bound) in &mut players {
        let Some(stage) = stage_of(entity) else {
            continue;
        };
        let node = art.nodes[stage.tumble.clip];
        if !bound {
            commands
                .entity(entity)
                .insert((AnimationGraphHandle(art.graph.clone()), DiePlayer));
            player.play(node).pause();
        }
        let duration = art.record.clips[stage.tumble.clip].duration;
        let at = seek(
            roll.start(stage.revision),
            time.elapsed_secs_f64(),
            duration,
        );
        if let Some(active) = player.animation_mut(node) {
            active.set_seek_time(at);
        }
    }
}

/// The AC-B panel while it presents a roll (its die-presented frame): the title, the
/// BRIEFING ribbon, the tray (a control that skips while the die is in the air), and
/// once the die lands the result with the faction's emblem and Continue.
pub(super) fn panel(
    commands: &mut Commands,
    art: &Art,
    kidnap: &crate::Kidnap,
    landed: bool,
    scale: f32,
    offset: Vec2,
) {
    let (s, o) = (scale, offset);
    picture(commands, art, "atlas-panel.png", PANEL, s, o, 2);
    let [x, y, size] = TITLE;
    let title = styled(
        commands,
        art,
        Prose::named("26July · Kidnap · Santiago"),
        &art.font,
        INK,
        [x, y, TITLE_WIDTH, size],
        s,
        o,
        3,
    );
    commands.entity(title).insert(Part("panel title"));
    // The die is presented during Terror, so AC-B's ribbon still reopens the briefings.
    super::choice::ribbon(commands, art, s, o);
    if !landed {
        let [x, y, w, h] = VIEWPORT;
        commands
            .spawn((
                UiRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(o.x + x * s),
                    top: px(o.y + y * s),
                    width: px(w * s),
                    height: px(h * s),
                    ..default()
                },
                GlobalZIndex(4),
                Control {
                    name: TRAY.into(),
                    choice: false,
                    action: Action::SkipDie,
                },
            ))
            .observe(click);
        return;
    }
    let [x, y, size] = RESULT;
    let result = styled(
        commands,
        art,
        format!("26July rolls {}", kidnap.die),
        &art.bold,
        INK,
        [x, y, 450., size],
        s,
        o,
        3,
    );
    commands.entity(result).insert(Part("kidnap-result title"));
    let [x, y, size] = DETAIL;
    let detail = styled(
        commands,
        art,
        format!(
            "Kidnap: Government → 26July, {}",
            crate::resources(kidnap.proceeds)
        ),
        &art.font,
        QUIET,
        [x, y, 450., size],
        s,
        o,
        3,
    );
    commands.entity(detail).insert(Part("kidnap-result detail"));
    super::choice::primary(
        commands,
        art,
        Control {
            name: CONTINUE.into(),
            choice: false,
            action: Action::ContinueDie,
        },
        CONTINUE_LABEL,
        true,
        CONTINUE,
        s,
        o,
    );
}

/// Whether the panel presents a roll now: then the die holds the keyboard.
pub(super) fn shown(world: &World) -> bool {
    world.resource::<Play>().session.as_ref().is_some_and(|s| {
        let game = s.game();
        presented(
            game.revision,
            game.kidnap.as_ref(),
            world.resource::<Roll>(),
        )
        .is_some()
    })
}

/// What the development seat holds unsettled: the die until it lands, and its scene
/// until it spawned and plays its tumble.
pub(super) fn unsettled(world: &mut World) -> Vec<String> {
    let mut pending = vec![];
    if !shown(world) {
        return pending;
    }
    let stages = world.query::<&Stage>().iter(world).count();
    let playing = world.query::<&DiePlayer>().iter(world).count();
    let layered = world
        .query_filtered::<(), (With<Mesh3d>, With<RenderLayers>)>()
        .iter(world)
        .count();
    if stages == 0 || playing == 0 || layered == 0 {
        pending.push("Kidnap die spawn".into());
    }
    if !world.resource::<Roll>().landed {
        pending.push("Kidnap die landing".into());
    }
    pending
}

/// The presented roll as the seat reports it: its tumble, face, clip time and landing,
/// and the tray's rectangle in logical window pixels.
pub(super) fn observe(world: &mut World) -> Value {
    let Some((revision, tumble, face)) = world
        .query::<&Stage>()
        .iter(world)
        .next()
        .map(|s| (s.revision, s.tumble, s.face))
    else {
        return Value::Null;
    };
    let mut texts: Vec<Value> = world
        .query::<(&Part, &emblems::Plain, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .map(|(part, plain, node, at)| {
            let scale = node.inverse_scale_factor;
            let min = at.translation * scale - node.size() * scale / 2.;
            let max = min + node.size() * scale;
            json!({"part": part.0, "text": plain.0, "bounds": [min.x, min.y, max.x, max.y]})
        })
        .collect();
    texts.sort_by_key(|t| t["part"].to_string());
    let viewport = world
        .query_filtered::<&Camera, With<TrayCamera>>()
        .iter(world)
        .next()
        .and_then(Camera::logical_viewport_rect)
        .map(|r| [r.min.x, r.min.y, r.max.x, r.max.y]);
    let top_face = top_face(world);
    let clip = &world.resource::<DieArt>().record.clips[tumble.clip];
    let roll = world.resource::<Roll>();
    let now = world.resource::<Time>().elapsed_secs_f64();
    let at = seek(roll.start(revision), now, clip.duration);
    let landed = roll.landed;
    json!({
        "revision": revision,
        "clip": clip.name,
        "half_turn": tumble.half_turn,
        "face": face,
        "seek": at,
        "landing": clip.landing,
        "duration": clip.duration,
        "rate": RATE,
        "landed": landed,
        "viewport": viewport,
        "top_face": top_face,
        "text": texts,
    })
}

/// Half the die's 16 mm edge, in metres.
const HALF_EDGE: f32 = 0.008;

/// The die's uppermost face as the tray camera shows it, in logical window pixels: the
/// square root of its projected area, the measure of #67's legibility floor (about
/// 70 px at 1920×1080). The face is the whole 16 mm square of the cube about `Die
/// body`'s origin, the four corners highest in the world.
fn top_face(world: &mut World) -> Option<f32> {
    let (camera, eye) = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TrayCamera>>()
        .iter(world)
        .next()
        .map(|(c, at)| (c.clone(), *at))?;
    let body = world
        .query::<(&Name, &GlobalTransform)>()
        .iter(world)
        .find(|(name, _)| name.as_str() == "Die body")
        .map(|(_, at)| *at)?;
    let mut corners: Vec<Vec3> = (0..8)
        .map(|i| {
            let sign = |bit: u32| if i & (1 << bit) == 0 { -1. } else { 1. };
            body.transform_point(Vec3::new(sign(0), sign(1), sign(2)) * HALF_EDGE)
        })
        .collect();
    corners.sort_by(|a, b| b.y.total_cmp(&a.y));
    let points: Vec<Vec2> = corners[..4]
        .iter()
        .map(|c| camera.world_to_viewport(&eye, *c).ok())
        .collect::<Option<_>>()?;
    // The shoelace area of the four corners taken around their centre.
    let centre = points.iter().copied().sum::<Vec2>() / 4.;
    let mut around = points;
    around.sort_by(|a, b| {
        let angle = |p: &Vec2| (p.y - centre.y).atan2(p.x - centre.x);
        angle(a).total_cmp(&angle(b))
    });
    let area = (0..4)
        .map(|i| around[i].perp_dot(around[(i + 1) % 4]))
        .sum::<f32>()
        .abs()
        / 2.;
    Some(area.sqrt())
}

/// What Enter reaches at the seat, by its control id: the tray's skip while the die is
/// in the air, and Continue once it has landed.
pub(super) fn focus(world: &mut World) -> Option<&'static str> {
    let landed = world.resource::<Roll>().landed;
    let shown = world.query::<&Stage>().iter(world).next().is_some();
    shown.then_some(if landed { CONTINUE } else { TRAY })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    /// #75: #8's seed 3 rolls 4, so Government pays 4 of its 6 (R 4.3.3): Government
    /// 6 → 2, 26July 0 → 4. The words never call the proceeds a transfer.
    #[test]
    fn the_landed_notice_says_government_pays_the_proceeds() {
        let mut game = Game::prepared_with_seed(3);
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        game.submit(Faction::M26, 1, "1:kidnap:roll").unwrap();
        let view = game.observe();
        let notice = landed_notice(&view, view.kidnap.as_ref().unwrap());
        assert_eq!(
            notice,
            "Kidnap rolled 4 · Government pays 4 Resources to 26July.\nGovernment 2 · 26July 4. Saved."
        );
        assert!(!notice.to_lowercase().contains("transfer"), "{notice}");
    }
}
