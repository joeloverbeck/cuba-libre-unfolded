//! The top display: outside influence on Cuba (Aid, US Alliance) at the top left,
//! the faction economy (Resources ledger, Cash supply) at the top right. Placement,
//! including the hover tags, follows `art/atlas/atlas-top-display.figcraft`; every
//! drawn element is a VectorCraft export from `atlas-top-display.vectorcraft`. The
//! game owns the values and the rules they follow. The Available Forces ledger's rows
//! open this tag too (#160); their targets, words and place are `available`'s. So
//! does the marker supply (#173), whose target is `supply`'s and whose tag opens below
//! it, as the main view's layout places it.
use super::palette::{INK, QUIET};
use super::*;
use crate::{CASH_MARKERS, MARKER_SUPPLY, UsAlliance};

/// One hoverable element of the top display, named for the development seat. It is
/// reported (`table.hover`, `table.top.item`) by its seat name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Item {
    Aid,
    UsAlliance,
    Resources,
    Cash,
    /// A faction's row of the Available Forces ledger (#160).
    Available(Faction),
    /// Save · quit in the bottom-right corner (#174), which opens the same tag. It is
    /// not a top-band target: the plate is its own control (`save_quit`).
    SaveQuit,
    /// The marker supply between the US Alliance track and the standings (#173).
    MarkerSupply,
}

/// The Available Forces rows' seat names, `available:<faction>`, in `Faction::ALL`
/// order.
static AVAILABLE_NAMES: std::sync::LazyLock<[String; 4]> =
    std::sync::LazyLock::new(|| Faction::ALL.map(|f| format!("available:{}", slug(f))));

impl Item {
    const ALL: [Self; 9] = [
        Self::Aid,
        Self::UsAlliance,
        Self::Resources,
        Self::Cash,
        Self::Available(Faction::Government),
        Self::Available(Faction::M26),
        Self::Available(Faction::Directorio),
        Self::Available(Faction::Syndicate),
        Self::MarkerSupply,
    ];
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Aid => "aid",
            Self::UsAlliance => "us-alliance",
            Self::Resources => "resources",
            Self::Cash => "cash",
            Self::Available(faction) => &AVAILABLE_NAMES[faction.index()],
            Self::SaveQuit => "save-quit",
            Self::MarkerSupply => "marker-supply",
        }
    }
    /// The hover target in reference pixels, and whether it is anchored to the
    /// right edge (else the left), as the layout's anchors are. The marker supply has
    /// none: the main view's layout places it, and `supply` draws its target.
    fn target(self) -> Option<([f32; 4], bool)> {
        Some(match self {
            Self::Aid => ([36., 24., 108., 50.], false),
            Self::UsAlliance => (TRACK, false),
            Self::Resources => (LEDGER, true),
            Self::Cash => ([1702., 222., 178., 44.], true),
            Self::Available(faction) => (available::row(faction), true),
            // The main view's `save-quit` area at the reference, from its bottom-right.
            Self::SaveQuit => (SAVE_QUIT, true),
            Self::MarkerSupply => return None,
        })
    }
}

impl serde::Serialize for Item {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// Layout rectangles in reference pixels, from the FigCraft frame.
const TRACK: [f32; 4] = [150., 18., 260., 64.];
const LEDGER: [f32; 4] = [1688., 20., 192., 190.];
/// Save · quit's plate in a 1920×1280 reference, 40 from the right and bottom edges.
const SAVE_QUIT: [f32; 4] = [1728., 1198., 152., 42.];

/// Hover-tag geometry from the FigCraft `hover tag` frame, in reference pixels.
pub(super) mod tag {
    pub const WIDTH: f32 = 468.;
    pub const GAP: f32 = 8.;
    pub const ROWS_TOP: f32 = 76.;
    pub const ROW_HEIGHT: f32 = 28.;
    pub const NOTES_GAP: f32 = 12.;
    pub const NOTE_LINE: f32 = 23.;
    pub const BOTTOM: f32 = 14.;
    /// Notes are broken by hand at this many characters, which a 420 px column
    /// at 17 px holds with margin at Lato's widths (native captures show no wrap),
    /// so each authored line is one drawn line.
    #[cfg(test)]
    pub const NOTE_CHARS: usize = 50;
}

#[derive(Component)]
pub(super) struct Tag;

/// Names a drawn part by its node name in `atlas-top-display.figcraft`, so the
/// development seat can compare native placement with the layout.
#[derive(Component)]
pub(super) struct Part(pub String);

#[derive(Component, Clone, Debug, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    pub item: Item,
    pub title: String,
    pub rows: Vec<(String, String)>,
    pub notes: Vec<String>,
    pub bounds: [f32; 4],
}

pub(super) fn header(
    commands: &mut Commands,
    art: &Art,
    view: &crate::Observation,
    scale: f32,
    left: Vec2,
    right: Vec2,
) {
    let mut place = |part: String, key: &str, rect: [f32; 4], offset: Vec2, z: i32| {
        let entity = picture(commands, art, key, rect, scale, offset, z);
        commands.entity(entity).insert(Part(part));
    };
    place(
        "aid".into(),
        "atlas-top-aid.png",
        [40., 30., 44., 38.],
        left,
        2,
    );
    place(
        "us-alliance".into(),
        &format!(
            "atlas-top-us-alliance-{}.png",
            view.alliance.name().to_lowercase()
        ),
        TRACK,
        left,
        2,
    );
    place("ledger".into(), "atlas-top-ledger.png", LEDGER, right, 2);
    place(
        "resources-icon".into(),
        "atlas-top-resources.png",
        [1765.5, 28., 37., 37.],
        right,
        3,
    );
    for faction in Faction::ALL {
        let row = faction.index();
        place(
            format!("ledger-row-{row}-emblem"),
            &format!("atlas-victory-icon-{}.png", slug(faction)),
            [1704., 76. + 31. * row as f32, 27., 27.],
            right,
            3,
        );
    }
    for slot in 0..CASH_MARKERS {
        // Filled while that marker is Available; an empty ring while it is on the map.
        let key = if slot < view.cash_available {
            "atlas-top-cash.png"
        } else {
            "atlas-top-cash-empty.png"
        };
        place(
            format!("cash-slot-{slot}"),
            key,
            [1706. + 46. * f32::from(slot), 226., 36., 36.],
            right,
            2,
        );
    }
    let aid = text(
        commands,
        art,
        view.aid.to_string(),
        92.,
        30.,
        50.,
        30.,
        scale,
        left,
        2,
    );
    commands
        .entity(aid)
        .insert((TextColor(INK), Part("aid-value".into())));
    for faction in Faction::ALL {
        let row = faction.index();
        let value = text(
            commands,
            art,
            view.resources[row].to_string(),
            1790.,
            74. + 31. * row as f32,
            74.,
            26.,
            scale,
            right,
            3,
        );
        commands.entity(value).insert((
            TextLayout::justify(Justify::Right),
            TextColor(INK),
            Part(format!("ledger-row-{row}-value")),
        ));
    }
    for item in Item::ALL {
        let Some(([x, y, w, h], anchored_right)) = item.target() else {
            continue;
        };
        let offset = if anchored_right { right } else { left };
        commands
            .spawn((
                UiRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(offset.x + x * scale),
                    top: px(offset.y + y * scale),
                    width: px(w * scale),
                    height: px(h * scale),
                    ..default()
                },
                GlobalZIndex(3),
                Control {
                    name: item.name().into(),
                    choice: false,
                    action: Action::Top(item),
                },
            ))
            .observe(click);
    }
}

/// What a tag explains. Values come from the observation and the game model; the
/// notes restate the rules cited beside them, broken into drawn lines.
pub(super) fn content(view: &crate::Observation, item: Item) -> Shown {
    // The marker supply's notes state its cap, which the rules model declares once.
    let marker_notes: [String; 2];
    let (title, rows, notes): (String, Vec<(String, String)>, Vec<&str>) = match item {
        Item::Aid => (
            "Aid".into(),
            vec![("Level".into(), view.aid.to_string())],
            vec![
                // R 1.8, R 3.1
                "Foreign assistance: added to Government Resources\nin each Propaganda Round's earnings and when an\nEvent says so. Aid itself cannot pay a cost.",
                // R 6.3.1 (and its note: Events adjust Aid as their text says)
                "Events can raise or lower it. It falls by 10, to\nno less than 0, when a Propaganda Round finds\nTotal Support of 18 or less.",
            ],
        ),
        Item::UsAlliance => (
            "US Alliance".into(),
            UsAlliance::ALL
                .into_iter()
                .map(|level| {
                    let label = if level == view.alliance {
                        format!("{} · now", level.name())
                    } else {
                        level.name().into()
                    };
                    let mut effect = format!("{} per space", level.coin_cost());
                    if !level.allows_air_strike() {
                        effect.push_str(" · no Air Strike");
                    }
                    (label, effect)
                })
                .collect(),
            vec![
                // R 6.3.1
                "Each Government COIN Operation costs this much\nper space; Garrison pays it once in total.",
                "Drops one level, if possible, when a Propaganda\nRound finds Total Support of 18 or less. Events\ncan also shift it.",
            ],
        ),
        Item::Resources => (
            "Resources".into(),
            Faction::ALL
                .into_iter()
                .map(|f| (f.name().to_string(), view.resources[f.index()].to_string()))
                .collect(),
            vec![
                // R 1.8
                "Each faction's own amount pays for its Operations.",
                "Transfers go only to or from the faction executing\nan Operation, Special Activity or Event.",
            ],
        ),
        Item::Cash => (
            "Cash".into(),
            vec![
                (
                    "Available".into(),
                    format!("{} of {CASH_MARKERS}", view.cash_available),
                ),
                (
                    "On the map".into(),
                    (CASH_MARKERS - view.cash_available).to_string(),
                ),
            ],
            vec![
                // R 4.5.2, R 6.2.4
                "Each marker sits under a Guerrilla or cube and\nbelongs to that piece's faction.",
                "After Operations without a Special Activity, its\nowner may remove it for a free Limited Operation.\nAt a Propaganda Round each becomes +6 Resources\nor an available Base there (Syndicate: Casino).",
            ],
        ),
        Item::Available(faction) => {
            let tag = available::tag(view, faction);
            (tag.title, tag.rows, tag.notes)
        }
        Item::SaveQuit => (
            "Save · quit".into(),
            Vec::new(),
            vec!["Every decision is saved as it is made, and this\nbutton closes the game."],
        ),
        Item::MarkerSupply => {
            let supply = view.marker_supply;
            (
                "Marker supply".into(),
                vec![
                    ("Terror on the map".into(), supply.terror_on_map.to_string()),
                    (
                        "Sabotage on the map".into(),
                        supply.sabotage_on_map.to_string(),
                    ),
                    (
                        "Available".into(),
                        format!("{} of {MARKER_SUPPLY}", supply.available),
                    ),
                ],
                // R 3.3.4's cap, and what returns markers (R 6.3.2, 6.3.3, 6.5). Only
                // what the rules state: nothing on Terror's other effects at 0, nor on
                // Reprisal or Government Earnings, which the rules leave open (#166).
                {
                    marker_notes = [
                        format!(
                            "Terror and Sabotage share {MARKER_SUPPLY} markers. Terror\nplaces no marker when all {MARKER_SUPPLY} are on the map\n(R 3.3.4)."
                        ),
                        "Civic Action and Agitation return Terror markers\nto the marker supply (R 6.3.2, 6.3.3); the Reset\nPhase returns the Terror and Sabotage markers\n(R 6.5).".into(),
                    ];
                    marker_notes.iter().map(String::as_str).collect()
                },
            )
        }
    };
    let mut notes: Vec<String> = notes.into_iter().map(String::from).collect();
    if item == Item::Resources {
        // The 49 cap (R 1.8) is shown only where it is about to matter.
        notes.extend(
            Faction::ALL
                .into_iter()
                .filter(|f| view.resources[f.index()] >= 45)
                .map(|f| format!("{} is near the 49 maximum.", f.name())),
        );
    }
    Shown {
        item,
        title,
        rows,
        notes,
        bounds: [0.; 4],
    }
}

fn note_lines(note: &str) -> usize {
    note.lines().count()
}

/// Where a tag opens in this window, in logical window pixels as [left, top, right,
/// bottom]. Its height follows its rows and note lines.
pub(super) fn bounds(shown: &Shown, layout: &areas::Layout, window: &Window) -> [f32; 4] {
    let scale = reference_pixel(window);
    let right = Vec2::new(window.width() - WIDTH * scale, 0.);
    let item = shown.item;
    let lines: usize = shown.notes.iter().map(|note| note_lines(note)).sum();
    let height = tag::ROWS_TOP
        + shown.rows.len() as f32 * tag::ROW_HEIGHT
        + tag::NOTES_GAP
        + lines as f32 * tag::NOTE_LINE
        + tag::BOTTOM;
    let Some(([x, y, _, h], anchored_right)) = item.target() else {
        // The marker supply's tag opens just below its area, centred under the graphic,
        // but kept left of the standings by the tags' gap: below the US Alliance track
        // and clear of the standings in every window (#173).
        let area = layout.area("marker-supply", window);
        let standings = layout.area("standings", window);
        let width = tag::WIDTH * scale;
        let left = (area.center().x - width / 2.).min(standings.min.x - tag::GAP * scale - width);
        let top = area.max.y + tag::GAP * scale;
        return [left, top, left + width, top + height * scale];
    };
    // Tags open just below their element, except the Resources tag, which opens
    // beside the ledger so the Cash supply beneath stays visible, and a faction's
    // Available Forces tag, which opens left of both ledgers below the standings
    // (#160).
    let (origin, x, top) = match (item, anchored_right) {
        (Item::Resources, _) => (right, x - tag::GAP - tag::WIDTH, y),
        (Item::Available(_), _) => {
            let [x, y] = available::opens();
            (right, x, y)
        }
        // Save · quit's tag opens above the plate, right edges on the window's margin,
        // as the top band's right-hand tags open below theirs; `draw` puts its right
        // edge and foot on the main view's `save-quit-tag` (#174).
        (Item::SaveQuit, _) => (right, 1880. - tag::WIDTH, y - tag::GAP - height),
        (_, true) => (right, 1880. - tag::WIDTH, y + h + tag::GAP),
        (_, false) => (Vec2::ZERO, x.min(1880. - tag::WIDTH), y + h + tag::GAP),
    };
    [
        origin.x + x * scale,
        origin.y + top * scale,
        origin.x + (x + tag::WIDTH) * scale,
        origin.y + (top + height) * scale,
    ]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    layout: Res<areas::Layout>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Shown>>,
) {
    let scale = reference_pixel(&window);
    let shown = ui
        .hover
        .visible()
        .and_then(|target| match target {
            Target::Top(item) => Some(*item),
            Target::Standing(_)
            | Target::Choice(_)
            | Target::Sequence(_)
            | Target::Card(_)
            | Target::Pile(_) => None,
        })
        .map(|item| {
            let mut shown = content(&play.game().observe(), item);
            shown.bounds = bounds(&shown, &layout, &window);
            if item == Item::SaveQuit {
                // Its right edge and foot are the main view's `save-quit-tag` (#174).
                let place = layout.save_quit_tag(&window);
                let [left, top, right, foot] = shown.bounds;
                let shift = place.max - Vec2::new(right, foot);
                shown.bounds = [left + shift.x, top + shift.y, place.max.x, place.max.y];
            }
            shown
        });
    if *previous == shown {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    previous.clone_from(&shown);
    let Some(shown) = shown else {
        return;
    };
    let [x, y, right_edge, bottom] = shown.bounds;
    let plate = picture(
        &mut commands,
        &art,
        "atlas-top-tag.png",
        [x, y, right_edge - x, bottom - y],
        1.,
        Vec2::ZERO,
        7,
    );
    commands.entity(plate).remove::<UiRoot>().insert(Tag);
    let mut words = |content: Prose, left: f32, top: f32, width: f32, size: f32, ink: Color| {
        let entity = text(
            &mut commands,
            &art,
            content,
            x + left * scale,
            y + top * scale,
            width * scale,
            size * scale,
            1.,
            Vec2::ZERO,
            9,
        );
        commands
            .entity(entity)
            .remove::<UiRoot>()
            .insert((Tag, TextColor(ink)));
    };
    // A title that names a faction introduces it and keeps its name.
    words(Prose::named(&shown.title), 24., 18., 420., 27., INK);
    for (index, (label, value)) in shown.rows.iter().enumerate() {
        let top = tag::ROWS_TOP + index as f32 * tag::ROW_HEIGHT;
        // Row labels introduce what their value belongs to; faction rows keep names.
        words(Prose::named(label), 24., top, 200., 18., INK);
        words(value.into(), 230., top, 214., 18., QUIET);
    }
    let mut top = tag::ROWS_TOP + shown.rows.len() as f32 * tag::ROW_HEIGHT + tag::NOTES_GAP;
    for note in &shown.notes {
        words(note.into(), 24., top, 420., 17., QUIET);
        top += note_lines(note) as f32 * tag::NOTE_LINE;
    }
    let rule = picture(
        &mut commands,
        &art,
        "atlas-victory-rule.png",
        [x + 24. * scale, y + 62. * scale, 420. * scale, scale],
        1.,
        Vec2::ZERO,
        8,
    );
    commands.entity(rule).remove::<UiRoot>().insert(Tag);
    commands.entity(plate).insert(shown);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> crate::Observation {
        crate::terror_chosen(crate::Game::opening()).observe()
    }

    #[test]
    fn tags_state_the_prepared_values() {
        let view = view();
        let aid = content(&view, Item::Aid);
        assert_eq!(aid.rows, [("Level".into(), "15".into())]);
        let alliance = content(&view, Item::UsAlliance);
        assert_eq!(
            alliance.rows,
            [
                ("Firm · now".into(), "2 per space".into()),
                ("Reluctant".into(), "3 per space".into()),
                ("Embargoed".into(), "4 per space · no Air Strike".into()),
            ]
        );
        let resources = content(&view, Item::Resources);
        assert_eq!(
            resources
                .rows
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            ["6", "0", "5", "15"]
        );
        let cash = content(&view, Item::Cash);
        assert_eq!(
            cash.rows,
            [
                ("Available".into(), "4 of 4".into()),
                ("On the map".into(), "0".into()),
            ]
        );
    }

    #[test]
    fn the_49_cap_shows_only_near_it() {
        let mut view = view();
        for item in Item::ALL {
            let shown = content(&view, item);
            let all = format!("{:?}{:?}", shown.rows, shown.notes);
            assert!(!all.contains("49"), "{item:?}: {all}");
        }
        view.resources[3] = 45;
        let notes = content(&view, Item::Resources).notes;
        assert_eq!(
            notes
                .iter()
                .filter(|n| n.contains("near the 49"))
                .collect::<Vec<_>>(),
            ["Syndicate is near the 49 maximum."]
        );
    }

    #[test]
    fn the_current_level_is_marked_at_every_level() {
        let mut view = view();
        for level in UsAlliance::ALL {
            view.alliance = level;
            let rows = content(&view, Item::UsAlliance).rows;
            let marked: Vec<_> = rows
                .iter()
                .filter(|(label, _)| label.ends_with("now"))
                .collect();
            assert_eq!(marked.len(), 1);
            assert!(marked[0].0.starts_with(level.name()));
        }
    }

    #[test]
    fn every_note_line_fits_one_drawn_line() {
        let mut view = view();
        view.resources = [45; 4];
        for item in Item::ALL {
            for note in content(&view, item).notes {
                for line in note.lines() {
                    assert!(line.chars().count() <= tag::NOTE_CHARS, "{line}");
                }
            }
        }
    }

    /// #174: Save · quit's tag is the top display's tag with one sentence, which says
    /// that every decision is already saved and that the button closes the game. On
    /// the owner's display it opens where the main view's `save-quit-tag` is.
    #[test]
    fn save_quit_s_tag_says_decisions_are_saved_and_opens_above_the_plate() {
        let shown = content(&view(), Item::SaveQuit);
        assert_eq!(shown.title, "Save · quit");
        assert!(shown.rows.is_empty());
        assert_eq!(
            shown.notes.join(" ").replace('\n', " "),
            "Every decision is saved as it is made, and this button closes the game."
        );
        for line in shown.notes.iter().flat_map(|note| note.lines()) {
            assert!(line.chars().count() <= tag::NOTE_CHARS, "{line}");
        }
        let record: serde_json::Value =
            serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json"))
                .unwrap();
        let [x, y, w, h]: [f32; 4] =
            serde_json::from_value(record["save-quit-tag"]["bounds"].clone()).unwrap();
        // 3840×2160: one reference pixel is 1.6875, and the right edge is 600 further.
        let layout = areas::Layout::read(include_bytes!(
            "../../art/atlas/atlas-main-view.layout.json"
        ))
        .unwrap();
        let window = Window {
            resolution: (3840, 2160).into(),
            ..default()
        };
        let at = bounds(&shown, &layout, &window);
        for (drawn, recorded) in at.into_iter().zip([x, y, x + w, y + h]) {
            assert!(
                (drawn - recorded).abs() <= 0.02,
                "{at:?} against {x} {y} {w} {h}"
            );
        }
        let [px, py, pw, ph]: [f32; 4] =
            serde_json::from_value(record["areas"]["save-quit"]["bounds"].clone()).unwrap();
        let ([rx, ry, rw, rh], _) = Item::SaveQuit.target().unwrap();
        for (drawn, recorded) in [rx * 1.6875 + 600., ry * 1.6875, rw * 1.6875, rh * 1.6875]
            .into_iter()
            .zip([px, py, pw, ph])
        {
            assert!((drawn - recorded).abs() <= 0.02, "{drawn} {recorded}");
        }
    }

    #[test]
    fn every_level_has_an_admitted_track_and_keeps_its_observed_name() {
        for level in UsAlliance::ALL {
            let key = format!("atlas-top-us-alliance-{}.png", level.name().to_lowercase());
            assert!(
                crate::assets::TOP_MANIFEST
                    .lines()
                    .any(|line| line.ends_with(&format!("art/atlas/{key}"))),
                "{key} is not admitted"
            );
            assert_eq!(serde_json::to_value(level).unwrap(), level.name());
        }
    }
}
