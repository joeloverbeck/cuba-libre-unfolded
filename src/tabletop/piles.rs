//! The two piles of cards on the table (#143; spec #137, "Cards on the table"; R 2.2,
//! 2.3.8): the played pile with the current card face up on top, and the draw deck
//! with the preview face up on top. The main view's layout places them
//! (`art/atlas/MAIN-VIEW.md`, `atlas-main-view.layout.json`).
//!
//! - **The tray** is a screen-fixed part of the window with its own camera on its own
//!   render layer, as the Kidnap die's is, so the piles never move with the map
//!   camera or take a region pick. It draws after the map and before the interface,
//!   so the count discs and every tag lie over the cards.
//! - **The view** is the layout's: each pile's top card lies flat on its rectangle,
//!   and its side shows below it. The camera looks straight down from well in front
//!   of the piles and draws only the part of its picture that holds them (a shifted
//!   lens), so a face parallel to the table keeps its shape and half of a pile's
//!   height shows as its near side.
//! - **The piles** are the shared round-cornered card solid (`unfolded_tabletop::card`).
//!   A pile is as tall as its cards at a real card's proportions, and its side shows a
//!   seam every six to seven cards (the owner's selection in #140): the solid is drawn
//!   in [`layers`] layers.
//! - **The numbers** are the rules model's `piles`: the played pile counts the current
//!   card and the draw deck the preview. Nothing here knows the order of the cards
//!   under the preview.
//!
//! The cards at rest are all this module shows. The motion of a card being played is
//! #145's: [`rest`] is where a card lies on top of a pile.
use super::cards::Slot;
use super::palette::{INK, PAPER, QUIET};
use super::*;
use crate::Observation;
use bevy::{
    camera::{
        SubCameraView, Viewport,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    text::{Justify, LetterSpacing},
};
use unfolded_tabletop::card;

/// One of the two piles, named for the development seat as `pile:played` and
/// `pile:deck`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Pile {
    Played,
    Deck,
}
impl Pile {
    pub(super) const ALL: [Self; 2] = [Self::Played, Self::Deck];
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Played => "pile:played",
            Self::Deck => "pile:deck",
        }
    }
    /// The card lying face up on the pile: the current card or the preview.
    pub(super) fn slot(self) -> Slot {
        match self {
            Self::Played => Slot::Current,
            Self::Deck => Slot::Preview,
        }
    }
    /// How many cards the pile holds and which card lies on top, from the model.
    fn shown(self, view: &Observation) -> Stack {
        match self {
            Self::Played => Stack {
                pile: self,
                cards: view.piles.played,
                top: view.current_card,
            },
            Self::Deck => Stack {
                pile: self,
                cards: view.piles.deck,
                top: view.preview_card,
            },
        }
    }
    /// The top card's rectangle in the layout's frame.
    fn face(self) -> [f32; 4] {
        match self {
            Self::Played => fig::PLAYED,
            Self::Deck => fig::DECK,
        }
    }
    /// The parts the layout draws for this pile, by their names in it.
    fn parts(self) -> PileParts {
        match self {
            Self::Played => PileParts {
                label: ("played-label", "CURRENT CARD", fig::PLAYED_LABEL),
                disc: ("played-count-disc", fig::PLAYED_DISC),
                count: ("played-count", fig::PLAYED_COUNT),
                caption: ("played-caption", "played pile", fig::PLAYED_CAPTION),
            },
            Self::Deck => PileParts {
                label: ("deck-label", "PREVIEW", fig::DECK_LABEL),
                disc: ("deck-count-disc", fig::DECK_DISC),
                count: ("deck-count", fig::DECK_COUNT),
                caption: ("deck-caption", "draw deck", fig::DECK_CAPTION),
            },
        }
    }
}
impl serde::Serialize for Pile {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

struct PileParts {
    /// Name, words, and left, top and width.
    label: (&'static str, &'static str, [f32; 3]),
    disc: (&'static str, [f32; 4]),
    count: (&'static str, [f32; 3]),
    caption: (&'static str, &'static str, [f32; 3]),
}

/// Node rectangles in `atlas-main-view.figcraft`'s one frame, which is the owner's
/// 3840×2160 display. A unit test holds each to the layout record.
mod fig {
    /// Frame units per reference pixel.
    pub const SCALE: f32 = 1.6875;
    pub const FRAME: [f32; 2] = [3840., 2160.];
    /// `card-tray`: the part of the window the tray's camera draws. It leaves room
    /// above the piles for a card that lifts or turns (#145).
    pub const TRAY: [f32; 4] = [60., 1480., 760., 660.];
    pub const PLAYED: [f32; 4] = [110., 1610., 300., 430.];
    pub const DECK: [f32; 4] = [470., 1610., 300., 430.];
    /// Text: left, top and width. Captions and marks are Lato; sizes below.
    pub const PLAYED_LABEL: [f32; 3] = [110., 1562., 231.];
    pub const DECK_LABEL: [f32; 3] = [470., 1562., 134.];
    pub const LABEL_SIZE: f32 = 26.;
    pub const TRACKING: f32 = 3.;
    pub const PLAYED_DISC: [f32; 4] = [356., 1994., 80., 80.];
    pub const DECK_DISC: [f32; 4] = [716., 2008., 80., 80.];
    pub const PLAYED_COUNT: [f32; 3] = [356., 2011., 80.];
    pub const DECK_COUNT: [f32; 3] = [716., 2025., 80.];
    pub const COUNT_SIZE: f32 = 38.;
    pub const PLAYED_CAPTION: [f32; 3] = [110., 2082., 132.];
    pub const DECK_CAPTION: [f32; 3] = [470., 2096., 127.];
    pub const CAPTION_SIZE: f32 = 28.;
    pub const PROPAGANDA_LABEL: [f32; 3] = [850., 1936., 193.];
    pub const PROPAGANDA_LABEL_SIZE: f32 = 24.;
    /// The first pip, and the step to the next.
    pub const PIP: [f32; 4] = [850., 1982., 34., 34.];
    pub const PIP_STEP: f32 = 48.;
    pub const PROPAGANDA_CAPTION: [f32; 3] = [850., 2030., 160.];
    /// `pile-shadow`, the blurred rectangle under a pile: how far its export reaches
    /// beyond the card on every side, and how far below the card it lies.
    pub const SHADOW_REACH: f32 = 39.;
    pub const SHADOW_DROP: f32 = 12.;
}

/// The tray's render layer: only its camera and light see it. The Kidnap tray has 1.
const LAYER: usize = 2;
/// Where the tray stands in the world: well below the map's sea and the Kidnap tray.
const TRAY_AT: Vec3 = Vec3::new(0., -80., 0.);
/// A card in metres: 63 mm wide, as long as the layout draws it, 0.3 mm thick. At the
/// layout's 300 px a card is then 1.43 px thick seen edge-on.
const CARD_WIDTH: f32 = 0.063;
const CARD_LENGTH: f32 = CARD_WIDTH * fig::PLAYED[3] / fig::PLAYED[2];
pub(super) const CARD_THICKNESS: f32 = 0.0003;
/// The corner radius as a share of the card's width: the admitted faces print black
/// outside their rounded corners, as far in as 5.34% on card 18 (CARD-TAG.md), so the
/// shared shape's 4.5% would show it.
const CORNER: f32 = 0.056;
/// Frame units per metre on the table.
const PER_METRE: f32 = fig::PLAYED[2] / CARD_WIDTH;
/// How much of a pile's height shows as its near side: the selected pile frames of
/// #140 looked from 60° above the table, where a height shows at half its length.
pub(super) const SIDE_SHOWN: f32 = 0.5;
/// The camera's height above the cards' top faces, in metres. From this far a pile's
/// height changes its side's width across the tray by under a pixel.
const CAMERA_HEIGHT: f32 = 4.;
/// Half the side of the camera's whole picture, in frame units. The tray is the part
/// of it that lies [`SIDE_SHOWN`] × [`CAMERA_HEIGHT`] behind the point below the lens.
const PICTURE_HALF: f32 = 10240.;
/// A face lies this far above its pile, so the two never share a depth.
const FACE_LIFT: f32 = 0.00002;
/// The pile's paper, lit: the layout draws the edges `#E4DDCB`.
const EDGE: Color = Color::srgb_u8(228, 221, 203);

/// The layers a pile of `cards` cards is drawn in: one seam for every six to seven
/// cards, and at least one seam wherever two cards meet (the owner's selection in
/// #140: the count divided by 7, rounded, and at least 2). A single card has no seam,
/// because no two cards meet in it; the selection's rule would cut one halfway up it.
pub(super) fn layers(cards: u8) -> usize {
    match cards {
        0 | 1 => usize::from(cards),
        _ => ((2 * usize::from(cards) + 7) / 14).max(2),
    }
}

/// A pile's height in metres.
pub(super) fn height(cards: u8) -> f32 {
    f32::from(cards) * CARD_THICKNESS
}

/// A point of the layout's frame on the table, in the tray's own coordinates: the
/// cards' top faces lie at height 0, +X is right on screen and +Z is down.
fn on_table(frame: Vec2) -> Vec3 {
    let centre = Vec2::new(
        fig::TRAY[0] + fig::TRAY[2] / 2.,
        fig::TRAY[1] + fig::TRAY[3] / 2.,
    );
    let at = (frame - centre) / PER_METRE;
    Vec3::new(at.x, 0., at.y)
}

/// Where a card lies at rest on top of a pile, in the tray's coordinates: the centre
/// of its face. #145 moves cards between these two places.
pub(super) fn rest(pile: Pile) -> Vec3 {
    let [x, y, w, h] = pile.face();
    on_table(Vec2::new(x + w / 2., y + h / 2.))
}

/// The point of the frame straight below the lens. A pile's near side shows
/// [`SIDE_SHOWN`] of its height at the cards' near edge.
fn below_lens() -> Vec2 {
    Vec2::new(
        fig::TRAY[0] + fig::TRAY[2] / 2.,
        fig::PLAYED[1] + fig::PLAYED[3] + SIDE_SHOWN * CAMERA_HEIGHT * PER_METRE,
    )
}

/// Where a point of the tray falls in the layout's frame.
#[cfg(test)]
fn seen(point: Vec3) -> Vec2 {
    let axis = on_table(below_lens());
    let from_axis =
        Vec2::new(point.x - axis.x, point.z - axis.z) * CAMERA_HEIGHT / (CAMERA_HEIGHT - point.y);
    below_lens() + from_axis * PER_METRE
}

/// The tray camera's projection and the part of its picture that the tray is.
fn lens() -> (PerspectiveProjection, SubCameraView) {
    let axis = below_lens();
    (
        PerspectiveProjection {
            fov: 2. * (PICTURE_HALF / (CAMERA_HEIGHT * PER_METRE)).atan(),
            aspect_ratio: 1.,
            near: CAMERA_HEIGHT - 0.5,
            far: CAMERA_HEIGHT + 0.5,
            ..default()
        },
        SubCameraView {
            full_size: UVec2::splat((2. * PICTURE_HALF) as u32),
            offset: Vec2::new(fig::TRAY[0], fig::TRAY[1]) - (axis - Vec2::splat(PICTURE_HALF)),
            size: UVec2::new(fig::TRAY[2] as u32, fig::TRAY[3] as u32),
        },
    )
}

/// One frame unit in logical window pixels: like every interface size it follows
/// the window alone (ADR 0002).
fn unit(window: &Window) -> f32 {
    layout(window).0 / fig::SCALE
}

/// A frame rectangle in logical window pixels. The card area keeps its distance from
/// the window's left and bottom edges.
fn place(window: &Window, [x, y, w, h]: [f32; 4]) -> Rect {
    let unit = unit(window);
    let min = Vec2::new(x * unit, window.height() - (fig::FRAME[1] - y) * unit);
    Rect {
        min,
        max: min + Vec2::new(w, h) * unit,
    }
}

/// The tray's rectangle in physical window pixels, inside the window.
fn viewport(window: &Window) -> Viewport {
    let rect = place(window, fig::TRAY);
    let factor = window.scale_factor();
    let limit = UVec2::new(window.physical_width(), window.physical_height());
    let min = (rect.min.max(Vec2::ZERO) * factor)
        .round()
        .as_uvec2()
        .min(limit.saturating_sub(UVec2::ONE));
    let max = (rect.max * factor)
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

/// The tray: the parent of its camera, its light and both piles.
#[derive(Component)]
pub(super) struct Tray;
#[derive(Component)]
pub(super) struct TrayCamera;
/// A pile as it is drawn: how many cards it holds and which card lies on top. It is
/// the parent of the pile's solid, its shadow and its top card, and stands where that
/// card rests.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Stack {
    pub pile: Pile,
    pub cards: u8,
    pub top: u8,
}
/// The card lying face up on a pile. #145 lifts and turns these.
#[derive(Component)]
pub(super) struct TopCard;

fn spawn_tray(commands: &mut Commands, window: &Window) -> Entity {
    let layer = RenderLayers::layer(LAYER);
    let (projection, sub_view) = lens();
    let eye = on_table(below_lens()) + Vec3::Y * CAMERA_HEIGHT;
    commands
        .spawn((
            Name::new("Card tray"),
            Tray,
            Transform::from_translation(TRAY_AT),
            Visibility::default(),
        ))
        .with_children(|tray| {
            // Lights the piles' near sides from above and in front, so the upper
            // facet of each seam's groove falls darker than the paper around it.
            tray.spawn((
                DirectionalLight {
                    illuminance: 2600.,
                    shadow_maps_enabled: false,
                    ..default()
                },
                layer.clone(),
                Transform::default().looking_to(Vec3::new(0.25, -0.85, -0.45), Vec3::Y),
            ));
            tray.spawn((
                Name::new("Card tray camera"),
                Camera3d::default(),
                TrayCamera,
                // Renders the seat's image over the map and under the interface.
                unfolded_tabletop::capture::OnSurface,
                Tonemapping::None,
                DebandDither::Disabled,
                Camera {
                    order: 1,
                    clear_color: ClearColorConfig::None,
                    viewport: Some(viewport(window)),
                    sub_camera_view: Some(sub_view),
                    ..default()
                },
                Projection::Perspective(projection),
                AmbientLight {
                    color: Color::WHITE,
                    brightness: 420.,
                    ..default()
                },
                layer,
                // Straight down, with the top of the screen away from the viewer.
                Transform::from_translation(eye).looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
        })
        .id()
}

fn spawn_stack(
    commands: &mut Commands,
    tray: Entity,
    stack: Stack,
    art: &Art,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    if stack.cards == 0 {
        // An empty pile shows nothing: no card lies there.
        commands.spawn((stack, ChildOf(tray), Transform::default()));
        return;
    }
    let layer = RenderLayers::layer(LAYER);
    let tall = height(stack.cards);
    let seams = layers(stack.cards);
    let reach = 2. * fig::SHADOW_REACH / PER_METRE;
    let drawn = (layer.clone(), Pickable::IGNORE, NoFrustumCulling);
    commands
        .spawn((
            Name::new(stack.pile.name()),
            stack,
            ChildOf(tray),
            Transform::from_translation(rest(stack.pile)),
            Visibility::default(),
        ))
        .with_children(|pile| {
            pile.spawn((
                Name::new("Pile shadow"),
                Mesh3d(meshes.add(Rectangle::new(CARD_WIDTH + reach, CARD_LENGTH + reach))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(art.images[SHADOW].clone()),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                // Flat on the table under the pile, a little toward the viewer.
                Transform::from_xyz(0., -tall - FACE_LIFT, fig::SHADOW_DROP / PER_METRE)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                drawn.clone(),
            ));
            pile.spawn((
                Name::new("Pile"),
                Mesh3d(meshes.add(card::pile_with_corners(
                    CARD_WIDTH,
                    CARD_LENGTH,
                    CORNER,
                    tall / seams as f32,
                    seams,
                ))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: EDGE,
                    perceptual_roughness: 1.,
                    reflectance: 0.,
                    ..default()
                })),
                Transform::from_xyz(0., -tall / 2., 0.),
                drawn.clone(),
            ));
            pile.spawn((
                Name::new("Top card"),
                TopCard,
                Mesh3d(meshes.add(card::face_with_corners(CARD_WIDTH, CARD_LENGTH, CORNER))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(art.images[assets::card_face(stack.top).name].clone()),
                    // The face keeps its printed colours whatever the light.
                    unlit: true,
                    ..default()
                })),
                Transform::from_xyz(0., FACE_LIFT, 0.),
                drawn,
            ));
        });
}

/// The pile shadow's export (`pile-shadow` in `atlas-main-view.figcraft`).
const SHADOW: &str = "atlas-main-view-pile-shadow.png";
const DISC: &str = "atlas-main-view-count-disc.png";
const PIP_PLAYED: &str = "atlas-main-view-pip-played.png";
const PIP: &str = "atlas-main-view-pip.png";

/// Keeps the tray in step with the match: each pile holds the cards the model counts
/// and shows the model's card on top. A change redraws that pile at rest.
#[allow(clippy::too_many_arguments)] // A Bevy system: its parameters are the resources it reads.
pub(super) fn present(
    mut commands: Commands,
    play: Res<Play>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    trays: Query<Entity, With<Tray>>,
    stacks: Query<(Entity, &Stack)>,
    mut cameras: Query<&mut Camera, With<TrayCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tray = trays
        .iter()
        .next()
        .unwrap_or_else(|| spawn_tray(&mut commands, &window));
    let view = play.game().observe();
    for pile in Pile::ALL {
        let wanted = pile.shown(&view);
        let mut drawn = false;
        for (entity, stack) in &stacks {
            if stack.pile != pile {
                continue;
            }
            if *stack == wanted && !drawn {
                drawn = true;
            } else {
                commands.entity(entity).despawn();
            }
        }
        if !drawn {
            spawn_stack(
                &mut commands,
                tray,
                wanted,
                &art,
                &mut meshes,
                &mut materials,
            );
        }
    }
    let wanted = viewport(&window);
    for mut camera in &mut cameras {
        let unchanged = camera.viewport.as_ref().is_some_and(|v| {
            v.physical_position == wanted.physical_position
                && v.physical_size == wanted.physical_size
        });
        if !unchanged {
            camera.viewport = Some(wanted.clone());
        }
    }
}

/// A pile's count disc, which is the pile's target for the pointer and the seat.
#[derive(Component)]
pub(super) struct Disc(Pile);
/// A Propaganda pip, and whether it is filled.
#[derive(Component)]
pub(super) struct Pip {
    index: u8,
    played: bool,
}

/// Text in the card area: its place and size in frame units.
#[allow(clippy::too_many_arguments)]
fn words(
    commands: &mut Commands,
    window: &Window,
    font: &Handle<Font>,
    content: impl Into<String>,
    [x, y, w]: [f32; 3],
    size: f32,
    ink: Color,
    part: &str,
) -> Entity {
    let content: String = content.into();
    let at = place(window, [x, y, w, 0.]);
    commands
        .spawn((
            UiRoot,
            areas::Part(part.into()),
            emblems::Plain(content.clone()),
            Text::new(content),
            TextFont {
                font: font.clone().into(),
                font_size: (size * unit(window)).into(),
                ..default()
            },
            TextColor(ink),
            // One line each: the layout's box is the words' own width.
            TextLayout::no_wrap(),
            Node {
                position_type: PositionType::Absolute,
                left: px(at.min.x),
                top: px(at.min.y),
                width: px(at.width()),
                ..default()
            },
            GlobalZIndex(4),
            Pickable::IGNORE,
        ))
        .id()
}

fn node(window: &Window, rect: [f32; 4]) -> Node {
    let at = place(window, rect);
    Node {
        position_type: PositionType::Absolute,
        left: px(at.min.x),
        top: px(at.min.y),
        width: px(at.width()),
        height: px(at.height()),
        ..default()
    }
}

/// What lies over the tray: each pile's label, count disc and caption, the Propaganda
/// pips, and a target on each top card and each pile. Every number is the model's.
pub(super) fn overlay(commands: &mut Commands, art: &Art, view: &Observation, window: &Window) {
    let unit = unit(window);
    let tracking = LetterSpacing::Px(fig::TRACKING * unit);
    for pile in Pile::ALL {
        let parts = pile.parts();
        let stack = pile.shown(view);
        let slot = pile.slot();
        // The top card: hovering or focusing it opens its tag, a click pins it.
        let mut face = node(window, pile.face());
        face.border_radius = BorderRadius::all(px(CORNER * fig::PLAYED[2] * unit));
        commands
            .spawn((
                UiRoot,
                face,
                GlobalZIndex(2),
                Control {
                    name: slot.name().into(),
                    choice: false,
                    action: Action::Card(slot),
                },
            ))
            .observe(click);
        let (name, text, at) = parts.label;
        let label = words(
            commands,
            window,
            &art.bold,
            text,
            at,
            fig::LABEL_SIZE,
            QUIET,
            name,
        );
        commands.entity(label).insert(tracking);
        let (name, rect) = parts.disc;
        let mut disc = node(window, rect);
        disc.border_radius = BorderRadius::MAX;
        commands
            .spawn((
                UiRoot,
                areas::Part(name.into()),
                Disc(pile),
                ImageNode::new(art.images[DISC].clone())
                    .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
                disc,
                GlobalZIndex(3),
                Control {
                    name: pile.name().into(),
                    choice: false,
                    action: Action::Pile,
                },
            ))
            .observe(click);
        let (name, at) = parts.count;
        let count = words(
            commands,
            window,
            &art.bold,
            stack.cards.to_string(),
            at,
            fig::COUNT_SIZE,
            PAPER,
            name,
        );
        commands
            .entity(count)
            .insert(TextLayout::justify(Justify::Center));
        let (name, text, at) = parts.caption;
        words(
            commands,
            window,
            &art.font,
            text,
            at,
            fig::CAPTION_SIZE,
            QUIET,
            name,
        );
    }
    let label = words(
        commands,
        window,
        &art.bold,
        "PROPAGANDA",
        fig::PROPAGANDA_LABEL,
        fig::PROPAGANDA_LABEL_SIZE,
        QUIET,
        "propaganda-label",
    );
    commands.entity(label).insert(tracking);
    const PIPS: [&str; 4] = [
        "propaganda-pip-0",
        "propaganda-pip-1",
        "propaganda-pip-2",
        "propaganda-pip-3",
    ];
    for (index, name) in PIPS
        .into_iter()
        .enumerate()
        .take(usize::from(view.piles.propaganda_cards))
    {
        let played = index < usize::from(view.piles.propaganda_played);
        let [x, y, w, h] = fig::PIP;
        commands.spawn((
            UiRoot,
            areas::Part(name.into()),
            Pip {
                index: index as u8,
                played,
            },
            ImageNode::new(art.images[if played { PIP_PLAYED } else { PIP }].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            node(window, [x + fig::PIP_STEP * index as f32, y, w, h]),
            GlobalZIndex(3),
            Pickable::IGNORE,
        ));
    }
    words(
        commands,
        window,
        &art.font,
        format!(
            "{} of {} played",
            view.piles.propaganda_played, view.piles.propaganda_cards
        ),
        fig::PROPAGANDA_CAPTION,
        fig::CAPTION_SIZE,
        INK,
        "propaganda-caption",
    );
}

/// What holds the seat until the tray shows the match: a pile that is not drawn yet
/// or still shows another count or card.
pub(super) fn unsettled(world: &mut World) -> Vec<String> {
    let view = world.resource::<Play>().game().observe();
    let drawn: Vec<Stack> = world.query::<&Stack>().iter(world).copied().collect();
    let cameras = world
        .query_filtered::<(), With<TrayCamera>>()
        .iter(world)
        .count();
    let current = Pile::ALL.iter().all(|pile| {
        drawn
            .iter()
            .filter(|s| s.pile == *pile)
            .eq([&pile.shown(&view)])
    });
    if cameras == 1 && current {
        vec![]
    } else {
        vec!["card piles".into()]
    }
}

/// What the tray shows, for the development seat (`table.piles`): the tray's
/// rectangle, and for each pile its cards, its seams' layers, the card on top, and
/// where the tray's own camera puts the top card's face and the pile's foot. Bounds
/// are [left, top, right, bottom] in logical window pixels.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let camera = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TrayCamera>>()
        .iter(world)
        .next()
        .map(|(camera, at)| (camera.clone(), *at));
    let tray = camera
        .as_ref()
        .and_then(|(camera, _)| camera.logical_viewport_rect())
        .map(|r| [r.min.x, r.min.y, r.max.x, r.max.y]);
    let mut piles = world
        .query::<(&Stack, &GlobalTransform)>()
        .iter(world)
        .map(|(stack, at)| {
            // The four corners of the card's outline at a height in the pile.
            let outline = |below: f32| -> Option<[f32; 4]> {
                let (camera, view) = camera.as_ref()?;
                let mut bounds: Option<Rect> = None;
                for (x, z) in [(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)] {
                    let corner = at.transform_point(Vec3::new(
                        x * CARD_WIDTH / 2.,
                        -below,
                        z * CARD_LENGTH / 2.,
                    ));
                    let p = camera.world_to_viewport(view, corner).ok()?;
                    bounds = Some(bounds.map_or(Rect::from_corners(p, p), |b| Rect {
                        min: b.min.min(p),
                        max: b.max.max(p),
                    }));
                }
                bounds.map(|b| [b.min.x, b.min.y, b.max.x, b.max.y])
            };
            (
                stack.pile,
                serde_json::json!({
                    "pile": stack.pile,
                    "card": stack.pile.slot(),
                    "cards": stack.cards,
                    "layers": layers(stack.cards),
                    "top": stack.top,
                    "face": outline(0.),
                    "foot": outline(height(stack.cards)),
                }),
            )
        })
        .collect::<Vec<_>>();
    piles.sort_by_key(|(pile, _)| *pile);
    let faces = world
        .query_filtered::<(), With<TopCard>>()
        .iter(world)
        .count();
    let mut discs = world
        .query::<(&Disc, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .map(|(disc, node, at)| {
            (
                disc.0,
                serde_json::json!({"pile": disc.0, "bounds": seat::logical_bounds(node, at)}),
            )
        })
        .collect::<Vec<_>>();
    discs.sort_by_key(|(pile, _)| *pile);
    let mut pips = world
        .query::<&Pip>()
        .iter(world)
        .map(|pip| (pip.index, pip.played))
        .collect::<Vec<_>>();
    pips.sort_unstable();
    serde_json::json!({
        "tray": tray,
        "piles": piles.into_iter().map(|(_, pile)| pile).collect::<Vec<_>>(),
        "faces": faces,
        "discs": discs.into_iter().map(|(_, disc)| disc).collect::<Vec<_>>(),
        "pips": pips.into_iter().map(|(_, played)| played).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn record() -> Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
    }
    fn rect(value: &Value) -> [f32; 4] {
        let v: Vec<f32> = value
            .as_array()
            .unwrap_or_else(|| panic!("{value} is no rectangle"))
            .iter()
            .map(|n| n.as_f64().unwrap() as f32)
            .collect();
        [v[0], v[1], v[2], v[3]]
    }

    /// The owner's selection in #140, for the piles it was drawn from and the rest of
    /// a game: the count divided by 7, rounded, and at least 2. Worked by hand:
    /// 18 / 7 = 2.57, 34 / 7 = 4.86, 48 / 7 = 6.86, 4 / 7 = 0.57, 52 / 7 = 7.43,
    /// 19 / 7 = 2.71, 33 / 7 = 4.71, 10 / 7 = 1.43, 11 / 7 = 1.57, 25 / 7 = 3.57.
    #[test]
    fn a_pile_shows_a_seam_every_six_to_seven_cards() {
        for (cards, expected) in [
            (18, 3),
            (34, 5),
            (48, 7),
            (4, 2),
            (52, 7),
            (19, 3),
            (33, 5),
            (2, 2),
            (10, 2),
            (11, 2),
            (24, 3),
            (25, 4),
        ] {
            assert_eq!(layers(cards), expected, "{cards} cards");
        }
        // One card is one layer: a seam marks where two cards meet.
        assert_eq!(layers(1), 1);
        assert_eq!(layers(0), 0);
    }

    /// A pile keeps a real card's proportions: 0.3 mm of a 63 mm card, which is
    /// 1.43 px for each card at the layout's 300 px, seen edge-on.
    #[test]
    fn a_pile_is_as_tall_as_its_cards() {
        let edge_on = |cards: u8| height(cards) * PER_METRE;
        assert!((edge_on(1) - 300. * 0.3 / 63.).abs() < 1e-3);
        assert!((edge_on(18) - 25.71).abs() < 0.01);
        assert!((edge_on(34) - 48.57).abs() < 0.01);
        assert!((edge_on(19) - edge_on(18) - 1.4286).abs() < 1e-3);
    }

    /// The code's rectangles are the layout record's.
    #[test]
    fn the_card_area_is_where_the_layout_records_it() {
        let layout = record();
        assert_eq!(rect(&layout["tray"]["bounds"]), fig::TRAY);
        assert_eq!(layout["tray"]["anchor"], "bottom-left");
        assert_eq!(rect(&layout["tray"]["played-pile"]), fig::PLAYED);
        assert_eq!(rect(&layout["tray"]["draw-deck"]), fig::DECK);
        assert_eq!(
            [
                layout["frame"][0].as_f64().unwrap() as f32,
                layout["frame"][1].as_f64().unwrap() as f32
            ],
            fig::FRAME
        );
        for (name, expected) in [
            ("played-count-disc", fig::PLAYED_DISC),
            ("deck-count-disc", fig::DECK_DISC),
        ] {
            assert_eq!(rect(&layout["parts"][name]), expected, "{name}");
        }
        for index in 0..4 {
            let [x, y, w, h] = fig::PIP;
            assert_eq!(
                rect(&layout["parts"][format!("propaganda-pip-{index}")]),
                [x + fig::PIP_STEP * index as f32, y, w, h]
            );
        }
        for (name, expected) in [
            ("played-label", fig::PLAYED_LABEL),
            ("deck-label", fig::DECK_LABEL),
            ("played-count", fig::PLAYED_COUNT),
            ("deck-count", fig::DECK_COUNT),
            ("played-caption", fig::PLAYED_CAPTION),
            ("deck-caption", fig::DECK_CAPTION),
            ("propaganda-label", fig::PROPAGANDA_LABEL),
            ("propaganda-caption", fig::PROPAGANDA_CAPTION),
        ] {
            let [x, y, w, _] = rect(&layout["text"][name]);
            assert_eq!([x, y, w], expected, "{name}");
        }
        // The shadow's export reaches as far beyond the card as the record says.
        let shadow = &layout["exports"]["pile-shadow"];
        let [.., w, h] = rect(&shadow["node"]);
        assert_eq!([w, h], [fig::PLAYED[2], fig::PLAYED[3]]);
        assert_eq!(shadow["size"][0], f64::from(w + 2. * fig::SHADOW_REACH));
        assert_eq!(shadow["size"][1], f64::from(h + 2. * fig::SHADOW_REACH));
    }

    /// The tray's camera draws each top card flat on its rectangle of the layout, and
    /// the pile's foot below it by half its height: 12.86 px for 18 cards and
    /// 24.29 px for 34 (1.4286 px a card, halved).
    #[test]
    fn the_camera_shows_the_piles_as_the_layout_draws_them() {
        let layout = record();
        for (pile, cards, base) in [
            (Pile::Played, 18, "played-pile-base"),
            (Pile::Deck, 34, "deck-pile-base"),
        ] {
            let [x, y, w, h] = pile.face();
            let centre = rest(pile);
            let corner = |sx: f32, sz: f32, below: f32| {
                seen(centre + Vec3::new(sx * CARD_WIDTH / 2., -below, sz * CARD_LENGTH / 2.))
            };
            // The face: exactly its rectangle.
            assert!(corner(-1., -1., 0.).abs_diff_eq(Vec2::new(x, y), 0.01));
            assert!(corner(1., 1., 0.).abs_diff_eq(Vec2::new(x + w, y + h), 0.01));
            // The foot: the card's near edge, lower by half the pile's height, and no
            // more than a pixel to either side.
            let tall = height(cards);
            let [bx, by, bw, bh] = rect(&layout["tray"][base]);
            for (sx, edge) in [(-1., bx), (1., bx + bw)] {
                let foot = corner(sx, 1., tall);
                assert!((foot.x - edge).abs() < 1., "{pile:?}: {foot}");
                assert!((foot.y - (by + bh)).abs() < 0.5, "{pile:?}: {foot}");
            }
            let shown = corner(0., 1., tall).y - (y + h);
            assert!(
                (shown - SIDE_SHOWN * tall * PER_METRE).abs() < 0.2,
                "{pile:?} shows {shown} px of its side"
            );
        }
        // The part of the camera's picture that is the tray lies inside that picture.
        let (_, sub) = lens();
        assert!(sub.offset.cmpge(Vec2::ZERO).all());
        assert!(
            (sub.offset + sub.size.as_vec2())
                .cmple(sub.full_size.as_vec2())
                .all()
        );
    }

    /// The piles are square and apart, inside the tray, and the tray inside the frame.
    #[test]
    fn the_piles_lie_apart_inside_the_tray() {
        let [tx, ty, tw, th] = fig::TRAY;
        assert!(tx >= 0. && ty >= 0. && tx + tw <= fig::FRAME[0] && ty + th <= fig::FRAME[1]);
        let (played, deck) = (fig::PLAYED, fig::DECK);
        assert_eq!(
            (played[1], played[2], played[3]),
            (deck[1], deck[2], deck[3])
        );
        assert!(played[0] + played[2] < deck[0]);
        // A full deck's foot and shadow still fall inside the tray.
        let deepest = SIDE_SHOWN * height(crate::DECK_CARDS) * PER_METRE
            + fig::SHADOW_DROP
            + fig::SHADOW_REACH;
        for [x, y, w, h] in [played, deck] {
            assert!(x - fig::SHADOW_REACH >= tx && x + w + fig::SHADOW_REACH <= tx + tw);
            assert!(y >= ty && y + h + deepest <= ty + th);
        }
    }

    /// In any window the tray keeps its distance from the left and bottom edges and
    /// its size follows the window's scale alone.
    #[test]
    fn the_tray_follows_the_window_s_left_and_bottom_edges() {
        for (width, height) in [
            (3840., 2160.),
            (1920., 1280.),
            (1920., 1080.),
            (2520., 1080.),
        ] {
            let window = Window {
                resolution: (width as u32, height as u32).into(),
                ..default()
            };
            let unit = (width / WIDTH).min(height / HEIGHT) / fig::SCALE;
            let tray = place(&window, fig::TRAY);
            assert!((tray.min.x - fig::TRAY[0] * unit).abs() < 0.01);
            assert!((height - tray.max.y - (2160. - 2140.) * unit).abs() < 0.01);
            assert!((tray.width() - fig::TRAY[2] * unit).abs() < 0.01);
            assert!(tray.min.y >= 0. && tray.max.x <= width);
        }
    }

    #[test]
    fn the_piles_read_the_model_s_numbers() {
        let view = crate::Game::opening().observe();
        assert_eq!(
            Pile::Played.shown(&view),
            Stack {
                pile: Pile::Played,
                cards: 18,
                top: 17
            }
        );
        assert_eq!(
            Pile::Deck.shown(&view),
            Stack {
                pile: Pile::Deck,
                cards: 34,
                top: 18
            }
        );
    }

    #[test]
    fn every_export_the_card_area_draws_is_admitted() {
        for key in [SHADOW, DISC, PIP_PLAYED, PIP] {
            assert!(
                crate::assets::MAIN_VIEW_MANIFEST
                    .lines()
                    .any(|line| line.ends_with(&format!("art/atlas/{key}"))),
                "{key} is not admitted"
            );
        }
    }
}
