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
//! - **The advance** (#145; R 2.3.8) is shown after it is saved, as the Kidnap die is
//!   (ADR 0001): the preview lifts and crosses to the played pile, then the deck's
//!   top card turns over where it lies, its back showing until it is past its edge.
//!   Both are the shared card motion (`unfolded_tabletop::card_motion`), read at the
//!   time since the commit on the shared clock. The numbers follow the cards: the
//!   deck's goes down as the preview leaves it, and the played pile's and the
//!   Propaganda pips change as it lands. A click, Space or Enter lands everything at
//!   once. [`Advance`] is never saved, so a restart, a resumed save and a seat that
//!   connects later all find the cards at rest.
use super::cards::Slot;
use super::palette::{INK, PAPER, QUIET};
use super::*;
use crate::{Observation, Piles};
use bevy::{
    camera::{
        SubCameraView, Viewport,
        visibility::{NoFrustumCulling, RenderLayers},
    },
    diagnostic::FrameCount,
    text::{Justify, LetterSpacing},
};
use unfolded_tabletop::{
    card,
    card_motion::{CardMove, Departure, Lift, Turn},
};

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
                top: Top::Face(view.current_card),
            },
            Self::Deck => Stack {
                pile: self,
                cards: view.piles.deck,
                top: Top::Face(view.preview_card),
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
/// The pile's paper, lit: the layout draws the cards' edges in this colour, which is
/// card stock and no variable of the interface palette.
const EDGE: Color = Color::srgb_u8(228, 221, 203); // own colour: card stock, the layout's pile edge #E4DDCB

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
    pub top: Top,
}
/// What lies uppermost on a pile. The seat reads a face as its card's number and a
/// face-down card as `"back"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Top {
    Face(u8),
    /// A card lying face down, as the deck's next card does while the advance is
    /// shown: the original shared back, whichever card it is.
    Back,
}
impl Top {
    /// The admitted image that shows it.
    fn image(self) -> &'static str {
        match self {
            Self::Face(card) => assets::card_face(card).name,
            Self::Back => assets::CARD_BACK.name,
        }
    }
}
impl serde::Serialize for Top {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Face(card) => serializer.serialize_u8(*card),
            Self::Back => serializer.serialize_str("back"),
        }
    }
}
/// The card lying face up on a pile.
#[derive(Component)]
pub(super) struct TopCard;

/// The cards at one moment: both piles as they are drawn, the played pile first, and
/// the numbers over them. At rest they are the model's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Lay {
    stacks: [Stack; 2],
    pub numbers: Piles,
}
impl Lay {
    pub(super) fn of(view: &Observation) -> Self {
        Self {
            stacks: Pile::ALL.map(|pile| pile.shown(view)),
            numbers: view.piles,
        }
    }
    fn stack(&self, pile: Pile) -> Stack {
        self.stacks[pile as usize]
    }
    /// The card face up on a pile, if one is.
    fn face(&self, pile: Pile) -> Option<u8> {
        match self.stack(pile) {
            Stack {
                cards: 1..,
                top: Top::Face(card),
                ..
            } => Some(card),
            _ => None,
        }
    }
}

/// A card the running app has just played (R 2.3.8): the revision that playing it
/// made and the cards as they lay before. `decide` records it once the advance is
/// saved, and [`follow`] starts the motion from the shared clock in the same frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Played {
    revision: u64,
    before: Lay,
}
impl Played {
    /// The card a decision played, if it played one: the preview before it is the
    /// current card after it, and the played pile holds one card more.
    pub(super) fn between(previous: &Observation, after: &Observation) -> Option<Self> {
        let (before, now) = (Lay::of(previous), Lay::of(after));
        (before.face(Pile::Deck).is_some()
            && now.face(Pile::Played) == before.face(Pile::Deck)
            && now.numbers.played == before.numbers.played + 1)
            .then_some(Self {
                revision: after.revision,
                before,
            })
    }
}

/// Seconds the played card takes to cross to the played pile, and the deck's top card
/// to turn over.
const CROSSING: f32 = 0.8;
const TURNING: f32 = 0.7;
/// The crossing card's rise at the middle of its way, in metres: the tray's view
/// shows 30 mm as 74 px toward the top of the screen.
const CROSSING_LIFT: f32 = 0.03;
/// The turning card's rise: half its width, so the long edge it turns on stays on the
/// pile and never dips into it.
const TURNING_LIFT: f32 = CARD_WIDTH / 2.;
/// How far the turning card is drawn toward the viewer at the middle of its turn, in
/// metres. Upright, its upper edge stands a card's width above the pile, which the
/// tray's view shows 160 px higher than the pile's top: 30 px above the tray. Drawn
/// 45 px nearer, as a hand turning a deck's top card draws it, it stays inside.
const TURNING_DRAW: f32 = 0.0095;

/// Where the middle of a card in the air rests over a pile: clear of the pile's top
/// face, whichever of its own sides is down.
fn aloft(pile: Pile) -> Vec3 {
    rest(pile) + Vec3::Y * (2. * FACE_LIFT + CARD_THICKNESS / 2.)
}

/// Step 2 of the advance: the played card lifts off the deck and crosses to the
/// played pile.
fn crossing() -> CardMove {
    CardMove::new(
        aloft(Pile::Deck),
        Transform::from_translation(aloft(Pile::Played)),
        CROSSING,
    )
    .lifted(Lift::arc(CROSSING_LIFT))
}

/// Step 3: the deck's top card turns over about its length where it lies, back up
/// until it stands on its edge and face up after.
fn turning() -> CardMove {
    let at = aloft(Pile::Deck);
    CardMove::new(at, Transform::from_translation(at), TURNING)
        .lifted(Lift::arc(TURNING_LIFT))
        .departing(Departure {
            offset: Vec3::Z * TURNING_DRAW,
            share: 0.5,
        })
        .turned(Turn::over(Vec3::Z))
}

/// Which step of the advance a card in the air is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Step {
    Crossing,
    Turning,
}

/// A card in the air: neither pile's solid holds it.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Flying {
    card: u8,
    step: Step,
}

/// What the tray shows at one moment: the cards lying on it, and the card in the air
/// with its pose in the tray's coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Scene {
    pub lay: Lay,
    flying: Option<(Flying, Transform)>,
}

/// The advance `elapsed` seconds after its commit, from the cards before it and
/// after it. The deck's number goes down as the played card leaves it; the played
/// pile's number and the Propaganda pips change as it lands. The deck shows the back
/// of its next card until that card has turned over, and the turning card is still
/// one of the deck's.
fn shown(before: &Lay, after: &Lay, elapsed: f32) -> Scene {
    let face_down = |cards: u8| Stack {
        pile: Pile::Deck,
        cards,
        top: Top::Back,
    };
    let turns = after.face(Pile::Deck);
    match (before.face(Pile::Deck), turns) {
        (Some(card), _) if elapsed < CROSSING => Scene {
            lay: Lay {
                stacks: [before.stack(Pile::Played), face_down(after.numbers.deck)],
                numbers: Piles {
                    deck: after.numbers.deck,
                    ..before.numbers
                },
            },
            flying: Some((
                Flying {
                    card,
                    step: Step::Crossing,
                },
                crossing().pose(elapsed),
            )),
        },
        (_, Some(card)) if elapsed < CROSSING + TURNING => Scene {
            lay: Lay {
                stacks: [
                    after.stack(Pile::Played),
                    face_down(after.numbers.deck - 1),
                ],
                numbers: after.numbers,
            },
            flying: Some((
                Flying {
                    card,
                    step: Step::Turning,
                },
                turning().pose(elapsed - CROSSING),
            )),
        },
        _ => Scene {
            lay: *after,
            flying: None,
        },
    }
}

/// The advance's presentation, which is never saved (ADR 0001): the card the running
/// app just played and the clock time when its commit was applied. A loaded save has
/// none and a skipped motion drops it, so the cards lie at rest.
#[derive(Resource, Default)]
pub(super) struct Advance {
    start: Option<(Played, f64)>,
    /// The frame in which an input landed the cards. That input does nothing else.
    landed: Option<u32>,
}
impl Advance {
    fn commit(&mut self, played: Played, now: f64) {
        self.start = Some((played, now));
    }
    /// The card whose playing made the match's present revision, and the seconds
    /// since its commit at clock time `now`. Any later decision ends its showing.
    fn elapsed(&self, view: &Observation, now: f64) -> Option<(Played, f32)> {
        self.start
            .filter(|(played, _)| played.revision == view.revision)
            .map(|(played, start)| (played, (now - start).max(0.) as f32))
    }
    /// What the tray shows at clock time `now` for the match as `view` observes it.
    pub(super) fn scene(&self, view: &Observation, now: f64) -> Scene {
        let after = Lay::of(view);
        match self.elapsed(view, now) {
            Some((played, elapsed)) => shown(&played.before, &after, elapsed),
            None => Scene {
                lay: after,
                flying: None,
            },
        }
    }
    /// Whether an input in frame `frame` lands the cards and so does nothing else: it
    /// does while a card is in the air, and for the rest of the frame that ended it.
    pub(super) fn lands(&mut self, view: &Observation, now: f64, frame: u32) -> bool {
        if self.scene(view, now).flying.is_some() {
            self.start = None;
            self.landed = Some(frame);
        }
        self.landed == Some(frame)
    }
}

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
                // This camera is always there, and the shared label anchors follow
                // one 3D camera only: marked as another view, it leaves the piece
                // counts and the popovers to the map's camera.
                unfolded_tabletop::label_anchor::OtherView,
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
                MeshMaterial3d(materials.add(paper())),
                Transform::from_xyz(0., -tall / 2., 0.),
                drawn.clone(),
            ));
            let mut top = pile.spawn((
                Name::new("Top card"),
                Mesh3d(meshes.add(card::face_with_corners(CARD_WIDTH, CARD_LENGTH, CORNER))),
                printed(materials, art, stack.top),
                Transform::from_xyz(0., FACE_LIFT, 0.),
                drawn,
            ));
            if matches!(stack.top, Top::Face(_)) {
                top.insert(TopCard);
            }
        });
}

/// A card's printed side, which keeps its colours whatever the light.
fn printed(
    materials: &mut Assets<StandardMaterial>,
    art: &Art,
    side: Top,
) -> MeshMaterial3d<StandardMaterial> {
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(art.images[side.image()].clone()),
        unlit: true,
        ..default()
    }))
}

/// The card in the air: one card of the shared solid with its face on one side and
/// the back on the other, about the middle of its thickness. The back is turned about
/// the card's length, as the card itself turns, so it reads upright when it is up.
fn spawn_flying(
    commands: &mut Commands,
    tray: Entity,
    flying: Flying,
    pose: Transform,
    art: &Art,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let drawn = (
        RenderLayers::layer(LAYER),
        Pickable::IGNORE,
        NoFrustumCulling,
    );
    let side = meshes.add(card::face_with_corners(CARD_WIDTH, CARD_LENGTH, CORNER));
    let above = CARD_THICKNESS / 2. + FACE_LIFT;
    commands
        .spawn((
            Name::new("Card in the air"),
            flying,
            ChildOf(tray),
            pose,
            Visibility::default(),
        ))
        .with_children(|card| {
            card.spawn((
                Name::new("Card"),
                Mesh3d(meshes.add(card::pile_with_corners(
                    CARD_WIDTH,
                    CARD_LENGTH,
                    CORNER,
                    CARD_THICKNESS,
                    1,
                ))),
                MeshMaterial3d(materials.add(paper())),
                drawn.clone(),
            ));
            card.spawn((
                Name::new("Face"),
                Mesh3d(side.clone()),
                printed(materials, art, Top::Face(flying.card)),
                Transform::from_xyz(0., above, 0.),
                drawn.clone(),
            ));
            card.spawn((
                Name::new("Back"),
                Mesh3d(side),
                printed(materials, art, Top::Back),
                Transform::from_xyz(0., -above, 0.)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                drawn.clone(),
            ));
        });
    let reach = 2. * fig::SHADOW_REACH / PER_METRE;
    let (lies, strength) = shade(&pose);
    commands.spawn((
        Name::new("Shadow of the card in the air"),
        Shade,
        ChildOf(tray),
        Mesh3d(meshes.add(Rectangle::new(CARD_WIDTH + reach, CARD_LENGTH + reach))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(strength),
            base_color_texture: Some(art.images[SHADOW].clone()),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        lies,
        drawn,
    ));
}

/// The shadow of the card in the air: the pile's own blurred shadow (`pile-shadow`),
/// cast straight down onto the cards beneath.
#[derive(Component)]
pub(super) struct Shade;

/// Where the shadow of a card in the air lies and how dark it is, from the card's
/// pose. It lies under the card, as wide as the card is seen from above, so a card on
/// its edge casts none; and it darkens as the card rises, from nothing on the pile to
/// the pile's own shadow at the crossing's full rise.
fn shade(pose: &Transform) -> (Transform, f32) {
    let rise = pose.translation.y - aloft(Pile::Deck).y;
    (
        Transform::from_xyz(
            pose.translation.x,
            1.5 * FACE_LIFT,
            pose.translation.z + fig::SHADOW_DROP / PER_METRE,
        )
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(pose.up().y.abs().max(0.001), 1., 1.)),
        (rise / CROSSING_LIFT).clamp(0., 1.),
    )
}

/// The cards' paper, as the light falls on a pile's side or a card's edge.
fn paper() -> StandardMaterial {
    StandardMaterial {
        base_color: EDGE,
        perceptual_roughness: 1.,
        reflectance: 0.,
        ..default()
    }
}

/// The back as the tray draws it: the admitted scan made smaller once, to twice the
/// card's size in the layout, so that drawing it does not skip four of every five of
/// its printed dots. The same picture, not another one: only its size changes.
pub(super) fn back(scan: &[u8]) -> Result<Image, String> {
    let scan = image::load_from_memory_with_format(scan, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let size = (2. * Vec2::new(fig::PLAYED[2], fig::PLAYED[3])).as_uvec2();
    let small = scan.resize_exact(size.x, size.y, image::imageops::FilterType::Lanczos3);
    Ok(Image::from_dynamic(
        small,
        true,
        bevy::asset::RenderAssetUsages::default(),
    ))
}

/// The pile shadow's export (`pile-shadow` in `atlas-main-view.figcraft`).
const SHADOW: &str = "atlas-main-view-pile-shadow.png";
const DISC: &str = "atlas-main-view-count-disc.png";
const PIP_PLAYED: &str = "atlas-main-view-pip-played.png";
const PIP: &str = "atlas-main-view-pip.png";

/// The keys that land a playing advance.
const LANDING_KEYS: [KeyCode; 3] = [KeyCode::Space, KeyCode::Enter, KeyCode::NumpadEnter];

/// Space or Enter lands a playing advance at once and does nothing else: the systems
/// after this one never see that press, so it commits no focused choice.
pub(super) fn keys(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    play: Res<Play>,
    time: Res<Time>,
    frame: Res<FrameCount>,
    mut advance: ResMut<Advance>,
) {
    if keys.any_just_pressed(LANDING_KEYS)
        && advance.lands(&play.game().observe(), time.elapsed_secs_f64(), frame.0)
    {
        for key in LANDING_KEYS {
            keys.clear_just_pressed(key);
        }
    }
}

/// A primary click anywhere lands a playing advance at once. A click on a control
/// asks [`Advance::lands`] too, and then does nothing else.
pub(super) fn land_on_click(
    event: On<PointerClick>,
    play: Res<Play>,
    time: Res<Time>,
    frame: Res<FrameCount>,
    mut advance: ResMut<Advance>,
) {
    if event.button == bevy::picking::pointer::PointerButton::Primary && play.session.is_some() {
        advance.lands(&play.game().observe(), time.elapsed_secs_f64(), frame.0);
    }
}

/// Starts a just-saved advance from the shared clock, in the frame its decision was
/// applied, and has the numbers over the tray redrawn when the cards they count
/// leave or land.
pub(super) fn follow(
    play: Res<Play>,
    time: Res<Time>,
    mut ui: ResMut<Interface>,
    mut advance: ResMut<Advance>,
    mut numbers: Local<Option<Piles>>,
) {
    let now = time.elapsed_secs_f64();
    if let Some(played) = ui.played.take() {
        advance.commit(played, now);
    }
    let shown = advance.scene(&play.game().observe(), now).lay.numbers;
    if numbers.replace(shown) != Some(shown) {
        ui.dirty = true;
    }
}

/// Keeps the tray in step with the match: each pile holds the cards the model counts
/// and shows the model's card on top, and while an advance is shown the cards lie and
/// fly as [`shown`] has them at this frame's time. A change redraws that pile.
#[allow(clippy::too_many_arguments)] // A Bevy system: its parameters are the resources it reads.
pub(super) fn present(
    mut commands: Commands,
    play: Res<Play>,
    art: Res<Art>,
    time: Res<Time>,
    advance: Res<Advance>,
    window: Single<&Window, With<PrimaryWindow>>,
    trays: Query<Entity, With<Tray>>,
    stacks: Query<(Entity, &Stack)>,
    mut cards: Query<(Entity, &Flying, &mut Transform), Without<Shade>>,
    mut shades: Query<
        (Entity, &mut Transform, &MeshMaterial3d<StandardMaterial>),
        (With<Shade>, Without<Flying>),
    >,
    mut cameras: Query<&mut Camera, With<TrayCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tray = trays
        .iter()
        .next()
        .unwrap_or_else(|| spawn_tray(&mut commands, &window));
    let scene = advance.scene(&play.game().observe(), time.elapsed_secs_f64());
    let mut aloft = false;
    for (entity, flying, mut at) in &mut cards {
        match scene.flying {
            Some((wanted, pose)) if wanted == *flying && !aloft => {
                *at = pose;
                aloft = true;
            }
            _ => commands.entity(entity).despawn(),
        }
    }
    for (entity, mut lies, material) in &mut shades {
        match scene.flying {
            Some((_, pose)) if aloft => {
                let (under, strength) = shade(&pose);
                *lies = under;
                if let Some(mut material) = materials.get_mut(&material.0) {
                    material.base_color.set_alpha(strength);
                }
            }
            _ => commands.entity(entity).despawn(),
        }
    }
    if let (Some((flying, pose)), false) = (scene.flying, aloft) {
        spawn_flying(
            &mut commands,
            tray,
            flying,
            pose,
            &art,
            &mut meshes,
            &mut materials,
        );
    }
    for pile in Pile::ALL {
        let wanted = scene.lay.stack(pile);
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
pub(super) fn overlay(commands: &mut Commands, art: &Art, lay: &Lay, window: &Window) {
    let unit = unit(window);
    let tracking = LetterSpacing::Px(fig::TRACKING * unit);
    for pile in Pile::ALL {
        let parts = pile.parts();
        let cards = [lay.numbers.played, lay.numbers.deck][pile as usize];
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
            cards.to_string(),
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
        .take(usize::from(lay.numbers.propaganda_cards))
    {
        let played = index < usize::from(lay.numbers.propaganda_played);
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
            lay.numbers.propaganda_played, lay.numbers.propaganda_cards
        ),
        fig::PROPAGANDA_CAPTION,
        fig::CAPTION_SIZE,
        INK,
        "propaganda-caption",
    );
}

/// The seat's name for what holds the keyboard while an advance is shown: Enter or
/// Space on it lands the cards.
pub(super) const ADVANCE: &str = "card-advance";
/// The components whose activation lands a playing advance for the seat: the card
/// area, and the cards and piles in it. The owner's click lands it anywhere.
pub(super) const LANDING: [&str; 5] = [
    "area:cards",
    "card:current",
    "card:preview",
    "pile:played",
    "pile:deck",
];
/// How far a card's own up may lean from level before the tray's camera, which looks
/// straight down, sees only its edge: about 3°.
const EDGE_ON: f32 = 0.05;

/// Whether a card is in the air at the clock's present time.
pub(super) fn playing(world: &World) -> bool {
    let view = world.resource::<Play>().game().observe();
    let now = world.resource::<Time>().elapsed_secs_f64();
    world
        .resource::<Advance>()
        .scene(&view, now)
        .flying
        .is_some()
}

/// What holds the seat: an advance until its cards have landed, and then the tray
/// until it shows the match, with no pile undrawn or showing another count or card.
pub(super) fn unsettled(world: &mut World) -> Vec<String> {
    if playing(world) {
        return vec!["card advance".into()];
    }
    let view = world.resource::<Play>().game().observe();
    let drawn: Vec<Stack> = world.query::<&Stack>().iter(world).copied().collect();
    let cameras = world
        .query_filtered::<(), With<TrayCamera>>()
        .iter(world)
        .count();
    let aloft = world.query::<&Flying>().iter(world).count();
    let current = Pile::ALL.iter().all(|pile| {
        drawn
            .iter()
            .filter(|s| s.pile == *pile)
            .eq([&pile.shown(&view)])
    });
    if cameras == 1 && current && aloft == 0 {
        vec![]
    } else {
        vec!["card piles".into()]
    }
}

/// What the tray shows, for the development seat (`table.piles`): the tray's
/// rectangle, and for each pile its cards, its seams' layers, the card on top, and
/// where the tray's own camera puts the top card's face and the pile's foot. Bounds
/// are [left, top, right, bottom] in logical window pixels. While an advance is shown,
/// `advance` is the card in the air: its number, its step, the seconds since the
/// commit, which side the camera sees and the card's bounds; at rest it is null.
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let camera = world
        .query_filtered::<(&Camera, &GlobalTransform), With<TrayCamera>>()
        .iter(world)
        .next()
        .map(|(camera, at)| (camera.clone(), *at));
    let view = world.resource::<Play>().game().observe();
    let now = world.resource::<Time>().elapsed_secs_f64();
    let elapsed = world
        .resource::<Advance>()
        .elapsed(&view, now)
        .map(|(_, elapsed)| elapsed);
    let advance = world
        .query::<(&Flying, &GlobalTransform)>()
        .iter(world)
        .next()
        .map(|(flying, at)| {
            let mut bounds: Option<Rect> = None;
            for corner in [
                (-1., -1., -1.),
                (1., -1., -1.),
                (1., -1., 1.),
                (-1., -1., 1.),
                (-1., 1., -1.),
                (1., 1., -1.),
                (1., 1., 1.),
                (-1., 1., 1.),
            ] {
                let half = Vec3::new(CARD_WIDTH, CARD_THICKNESS, CARD_LENGTH) / 2.;
                let corner = at.transform_point(Vec3::from(corner) * half);
                let seen = camera
                    .as_ref()
                    .and_then(|(camera, view)| camera.world_to_viewport(view, corner).ok());
                if let Some(p) = seen {
                    bounds = Some(bounds.map_or(Rect::from_corners(p, p), |b| Rect {
                        min: b.min.min(p),
                        max: b.max.max(p),
                    }));
                }
            }
            // The tray's camera looks straight down, so the side that is up shows.
            let up = at.up().y;
            let showing = if up > EDGE_ON {
                "face"
            } else if up < -EDGE_ON {
                "back"
            } else {
                "edge"
            };
            serde_json::json!({
                "card": flying.card,
                "step": flying.step,
                "elapsed": elapsed,
                "showing": showing,
                "bounds": bounds.map(|b| [b.min.x, b.min.y, b.max.x, b.max.y]),
            })
        });
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
        "advance": advance,
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
                top: Top::Face(17),
            }
        );
        assert_eq!(
            Pile::Deck.shown(&view),
            Stack {
                pile: Pile::Deck,
                cards: 34,
                top: Top::Face(18),
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

    /// The cards on card 17 and after card 18 is played, as #143 worked them from
    /// the prepared deck: 18 and 34 with one Propaganda card played, then 19 and 33.
    fn card_17() -> Lay {
        Lay::of(&crate::Game::opening().observe())
    }
    fn card_18(propaganda_played: u8) -> Lay {
        Lay {
            stacks: [
                Stack {
                    pile: Pile::Played,
                    cards: 19,
                    top: Top::Face(18),
                },
                Stack {
                    pile: Pile::Deck,
                    cards: 33,
                    top: Top::Face(19),
                },
            ],
            numbers: Piles {
                played: 19,
                deck: 33,
                propaganda_played,
                propaganda_cards: 4,
            },
        }
    }
    fn face_down(cards: u8) -> Stack {
        Stack {
            pile: Pile::Deck,
            cards,
            top: Top::Back,
        }
    }
    /// The card in the air at a moment of the advance from card 17 to card 18.
    fn in_the_air(elapsed: f32) -> (Flying, Transform) {
        shown(&card_17(), &card_18(1), elapsed)
            .flying
            .unwrap_or_else(|| panic!("no card is in the air at {elapsed} s"))
    }

    /// The four steps of the advance sheet, with the shared motion's own curves
    /// worked by hand: the path eases with smoothstep, 3v² − 2v³, and the lift is the
    /// half sine of that. A quarter of the way through a move the path is at
    /// 3/16 − 2/64 = 0.15625 and the lift at sin(0.15625 π) = 0.4714; halfway they
    /// are at 0.5 and 1. The piles' middles are 620 px and 260 px from the frame's
    /// left, 180 px either side of the tray's, which is 0.0378 m at 300 px to 63 mm.
    #[test]
    fn the_played_card_crosses_and_then_the_deck_s_top_card_turns_over() {
        let (before, after) = (card_17(), card_18(1));
        let apart = 180. / PER_METRE;
        let low = 2. * FACE_LIFT + CARD_THICKNESS / 2.;
        let near = |a: Vec3, b: Vec3| a.abs_diff_eq(b, 1e-5);

        // Step 1 is the cards at rest before the commit. As the commit is applied
        // the played card is still on the deck, face up, and already counted off it.
        let start = shown(&before, &after, 0.);
        assert_eq!(
            start.lay.stacks,
            [before.stack(Pile::Played), face_down(33)]
        );
        assert_eq!(
            (start.lay.numbers.played, start.lay.numbers.deck),
            (18, 33)
        );
        let (card, pose) = start.flying.unwrap();
        assert_eq!(
            card,
            Flying {
                card: 18,
                step: Step::Crossing
            }
        );
        assert!(near(pose.translation, rest(Pile::Deck) + Vec3::Y * low));
        assert!(pose.up().y > 0.999);

        // Step 2: a quarter of the way it has left the deck by 0.15625 of the way
        // and risen 0.4714 of 30 mm; halfway it is between the piles, 30 mm up.
        let (_, quarter) = in_the_air(CROSSING / 4.);
        assert!(near(
            quarter.translation,
            Vec3::new(
                apart * (1. - 2. * 0.15625),
                low + 0.03 * 0.471_397,
                rest(Pile::Deck).z
            )
        ));
        let (card, half) = in_the_air(CROSSING / 2.);
        assert_eq!(card.card, 18);
        assert!(near(
            half.translation,
            Vec3::new(0., low + 0.03, rest(Pile::Deck).z)
        ));
        // It stays level and face up, and the deck shows a back all the way.
        assert!(half.up().y > 0.999);
        let crossing = shown(&before, &after, CROSSING - 0.001);
        assert_eq!(crossing.lay, start.lay);

        // It lands: the played pile holds it and counts it, and step 3 begins with
        // the deck's top card, 19, lying back up over the 32 cards under it.
        let landed = shown(&before, &after, CROSSING);
        assert_eq!(
            landed.lay.stacks,
            [after.stack(Pile::Played), face_down(32)]
        );
        assert_eq!(landed.lay.numbers, after.numbers);
        let (card, pose) = landed.flying.unwrap();
        assert_eq!(
            card,
            Flying {
                card: 19,
                step: Step::Turning
            }
        );
        assert!(near(pose.translation, rest(Pile::Deck) + Vec3::Y * low));
        assert!(pose.up().y < -0.999, "the back is up");

        // A quarter of the way through the turn it has turned 0.15625 of a half
        // turn, 28.1°, so its back still shows: its own up points down at
        // −cos 28.1° = −0.882. It has risen 0.4714 of half its width.
        let (_, quarter) = in_the_air(CROSSING + TURNING / 4.);
        assert!((quarter.up().y + 0.881_92).abs() < 1e-4);
        assert!(
            (quarter.translation.y - (low + CARD_WIDTH / 2. * 0.471_397)).abs() < 1e-5,
            "{quarter:?}"
        );
        // Halfway it stands on its edge, half its width up, drawn toward the viewer.
        let (_, half) = in_the_air(CROSSING + TURNING / 2.);
        assert!(half.up().y.abs() < 1e-4);
        assert!(near(
            half.translation,
            rest(Pile::Deck) + Vec3::new(0., low + CARD_WIDTH / 2., TURNING_DRAW)
        ));
        // Past its edge the face shows, and the back never does again.
        let (_, late) = in_the_air(CROSSING + 0.75 * TURNING);
        assert!((late.up().y - 0.881_92).abs() < 1e-4);

        // Step 4: the cards at rest, as the model has them.
        for elapsed in [CROSSING + TURNING, 2., 3600.] {
            assert_eq!(
                shown(&before, &after, elapsed),
                Scene {
                    lay: after,
                    flying: None
                }
            );
        }
    }

    /// The Propaganda pips change as the played card lands, not before: worked for a
    /// played card that is a Propaganda card, the second of the four.
    #[test]
    fn the_propaganda_pips_change_as_the_card_lands() {
        let (before, after) = (card_17(), card_18(2));
        for (elapsed, played) in [
            (0., 1),
            (CROSSING / 2., 1),
            (CROSSING - 0.001, 1),
            (CROSSING, 2),
            (CROSSING + TURNING / 2., 2),
            (CROSSING + TURNING, 2),
        ] {
            assert_eq!(
                shown(&before, &after, elapsed)
                    .lay
                    .numbers
                    .propaganda_played,
                played,
                "{elapsed} s"
            );
        }
    }

    /// At every 240th of a second of the advance the card in the air is clear of
    /// both piles' top faces and wholly inside the tray's part of the window, and the
    /// turning card shows its back until it stands on its edge and never moves
    /// further from where it lies than it is drawn toward the viewer.
    #[test]
    fn a_card_in_the_air_stays_over_the_piles_and_inside_the_tray() {
        let [left, top, width, height] = fig::TRAY;
        let half = Vec3::new(CARD_WIDTH, CARD_THICKNESS, CARD_LENGTH) / 2.;
        let mut highest = f32::MAX;
        for step in 0..(240. * (CROSSING + TURNING)) as u32 {
            let elapsed = step as f32 / 240.;
            let (card, pose) = in_the_air(elapsed);
            for corner in 0..8 {
                let sign = Vec3::new(
                    if corner & 1 == 0 { -1. } else { 1. },
                    if corner & 2 == 0 { -1. } else { 1. },
                    if corner & 4 == 0 { -1. } else { 1. },
                );
                let point = pose.transform_point(sign * half);
                assert!(
                    point.y > 1.5 * FACE_LIFT,
                    "{card:?} dips to {} at {elapsed} s",
                    point.y
                );
                let at = seen(point);
                highest = highest.min(at.y);
                assert!(
                    at.x > left + 8.
                        && at.x < left + width - 8.
                        && at.y > top + 8.
                        && at.y < top + height - 8.,
                    "{card:?} is drawn at {at} at {elapsed} s"
                );
            }
            if card.step == Step::Turning {
                let turned = elapsed - CROSSING;
                assert_eq!(pose.up().y < 0., turned < TURNING / 2., "{elapsed} s");
                let from = pose.translation - aloft(Pile::Deck);
                assert!(from.x.abs() < 1e-6 && (0.0..=TURNING_DRAW + 1e-6).contains(&from.z));
            }
        }
        // The turning card's upper edge comes within 16 px of the tray's top.
        assert!((highest - top - 15.8).abs() < 1., "{highest}");
    }

    /// Only the decision that plays a card starts an advance: through card 17's
    /// Terror, Kidnap and Event nothing does, and the advance to card 18 does, with
    /// the cards as they lay on card 17.
    #[test]
    fn only_a_played_card_starts_an_advance() {
        let mut game = crate::Game::opening();
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
        ] {
            let previous = game.observe();
            game.submit(
                previous.owner,
                previous.revision,
                &format!("{}:{action}", previous.revision),
            )
            .unwrap();
            assert_eq!(Played::between(&previous, &game.observe()), None, "{action}");
        }
        let previous = game.observe();
        game.submit(
            previous.owner,
            previous.revision,
            &format!("{}:advance-card", previous.revision),
        )
        .unwrap();
        let after = game.observe();
        assert_eq!(
            Played::between(&previous, &after),
            Some(Played {
                revision: after.revision,
                before: card_17()
            })
        );
        assert_eq!(Lay::of(&after), card_18(1));
    }

    /// The advance's showing is the running app's alone. Nothing loaded has one; a
    /// skip lands it and takes that frame's input; it ends by itself after its two
    /// moves; and any later decision ends it.
    #[test]
    fn a_loaded_skipped_finished_or_superseded_advance_lies_at_rest() {
        let mut game = crate::Game::opening();
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
        ] {
            let view = game.observe();
            game.submit(
                view.owner,
                view.revision,
                &format!("{}:{action}", view.revision),
            )
            .unwrap();
        }
        let previous = game.observe();
        game.submit(
            previous.owner,
            previous.revision,
            &format!("{}:advance-card", previous.revision),
        )
        .unwrap();
        let view = game.observe();
        let rest = Scene {
            lay: Lay::of(&view),
            flying: None,
        };
        let played = Played::between(&previous, &view).unwrap();

        // A save opened on card 18 was advanced in another run.
        let mut advance = Advance::default();
        assert_eq!(advance.scene(&view, 0.), rest);
        assert!(!advance.lands(&view, 0., 1));

        // Applied at 10 s on the shared clock, it plays for 1.5 s.
        advance.commit(played, 10.);
        assert!(advance.scene(&view, 10.).flying.is_some());
        assert!(advance.scene(&view, 11.49).flying.is_some());
        assert_eq!(advance.scene(&view, 11.5), rest);
        assert!(!advance.lands(&view, 11.5, 2), "nothing is left to land");

        // A skip in frame 7 lands it, takes every input of that frame and no later.
        advance.commit(played, 20.);
        assert!(advance.lands(&view, 20.4, 7));
        assert_eq!(advance.scene(&view, 20.4), rest);
        assert!(advance.lands(&view, 20.4, 7));
        assert!(!advance.lands(&view, 20.42, 8));

        // The next decision shows its own result at rest.
        advance.commit(played, 30.);
        let mut later = view.clone();
        later.revision += 1;
        assert_eq!(advance.scene(&later, 30.1).flying, None);
    }

    /// A card in the air casts the pile's shadow straight down, 12 px toward the
    /// viewer as a pile's lies. Halfway across it is 30 mm up: the whole shadow, as
    /// wide as the card. On a pile it casts none. A quarter through its turn the
    /// card is 0.4714 of half its width up, 14.85 mm of the 30, and seen from above
    /// it is cos 28.1° = 0.882 of its width; on its edge it has no width.
    #[test]
    fn a_card_in_the_air_casts_its_shadow_straight_down() {
        let (_, pose) = in_the_air(CROSSING / 2.);
        let (lies, strength) = shade(&pose);
        assert_eq!(strength, 1.);
        assert!((lies.translation.x - pose.translation.x).abs() < 1e-6);
        assert!((lies.translation.z - pose.translation.z - 12. / PER_METRE).abs() < 1e-6);
        assert!(lies.translation.y > FACE_LIFT && lies.translation.y < 2. * FACE_LIFT);
        assert!((lies.scale.x - 1.).abs() < 1e-4);
        for elapsed in [0., CROSSING] {
            assert!(shade(&in_the_air(elapsed).1).1 < 1e-4, "{elapsed} s");
        }
        let (lies, strength) = shade(&in_the_air(CROSSING + TURNING / 4.).1);
        assert!((strength - 0.4950).abs() < 1e-3, "{strength}");
        assert!((lies.scale.x - 0.881_92).abs() < 1e-4);
        let (lies, _) = shade(&in_the_air(CROSSING + TURNING / 2.).1);
        assert!(lies.scale.x < 0.01);
    }
}
