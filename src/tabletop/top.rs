//! The top display: outside influence on Cuba (Aid, US Alliance) at the top left,
//! the faction economy (Resources ledger, Cash supply) at the top right. Placement
//! follows `art/atlas/atlas-top-display.figcraft`; every drawn element is a
//! VectorCraft export from `atlas-top-display.vectorcraft`. The game owns the values.
use super::*;

/// One hoverable element of the top display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Item {
    Aid,
    Alliance,
    Resources,
    Cash,
}

impl Item {
    const ALL: [Self; 4] = [Self::Aid, Self::Alliance, Self::Resources, Self::Cash];
    fn name(self) -> &'static str {
        match self {
            Self::Aid => "aid",
            Self::Alliance => "alliance",
            Self::Resources => "resources",
            Self::Cash => "cash",
        }
    }
    /// The hover target in reference pixels, and whether it is anchored to the
    /// right edge (else the left), as the layout's anchors are.
    fn target(self) -> ([f32; 4], bool) {
        match self {
            Self::Aid => ([36., 24., 108., 50.], false),
            Self::Alliance => ([150., 18., 260., 64.], false),
            Self::Resources => ([1688., 20., 192., 190.], true),
            Self::Cash => ([1702., 222., 178., 44.], true),
        }
    }
}

/// The US Alliance track, best level first (R 6.3.1).
const LEVELS: [&str; 3] = ["Firm", "Reluctant", "Embargoed"];
/// COIN Operation cost per space at each level (R 6.3.1).
const COIN_COST: [u8; 3] = [2, 3, 4];
/// The game's Cash markers (R 4.5.2).
const CASH_MARKERS: u8 = 4;
const INK: Color = Color::srgb_u8(30, 50, 57);
const QUIET: Color = Color::srgb_u8(83, 106, 109);

fn level(alliance: &str) -> usize {
    LEVELS
        .iter()
        .position(|level| *level == alliance)
        .unwrap_or_else(|| panic!("Unsupported US Alliance level {alliance}"))
}

#[derive(Component)]
pub(super) struct Tag;

#[derive(Component, Clone, Debug, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    pub item: Item,
    pub title: String,
    pub rows: Vec<(String, String)>,
    pub notes: Vec<String>,
    pub bounds: [f32; 4],
}

/// Names a drawn part by its node name in `atlas-top-display.figcraft`, so the
/// development seat can compare native placement with the layout.
#[derive(Component)]
pub(super) struct Part(pub String);

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
    let track = LEVELS[level(view.alliance)].to_lowercase();
    place(
        "alliance".into(),
        &format!("atlas-top-alliance-{track}.png"),
        [150., 18., 260., 64.],
        left,
        2,
    );
    place(
        "ledger".into(),
        "atlas-top-ledger.png",
        [1688., 20., 192., 190.],
        right,
        2,
    );
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
            &format!("atlas-victory-icon-{}.png", victory::key(faction)),
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
        let ([x, y, w, h], anchored_right) = item.target();
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

/// What a tag explains. Values come from the observation; the wording restates
/// the rules cited beside each line.
fn content(view: &crate::Observation, item: Item) -> Shown {
    let (title, rows, notes) = match item {
        Item::Aid => (
            "Aid",
            vec![("Level".into(), format!("{} of 49", view.aid))],
            vec![
                // R 1.8, R 3.1
                "Foreign assistance, added to Government Resources in each Propaganda Round's earnings and when an Event says so. Aid itself cannot pay a cost.".into(),
                // R 6.3.1
                "Falls by 10, to no less than 0, when a Propaganda Round finds Total Support of 18 or less.".into(),
            ],
        ),
        Item::Alliance => {
            let current = level(view.alliance);
            let rows = LEVELS
                .iter()
                .zip(COIN_COST)
                .enumerate()
                .map(|(index, (name, cost))| {
                    let label = if index == current {
                        format!("{name} · now")
                    } else {
                        (*name).into()
                    };
                    let effect = if index == 2 {
                        format!("{cost} per space · no Air Strike")
                    } else {
                        format!("{cost} per space")
                    };
                    (label, effect)
                })
                .collect();
            (
                "US Alliance",
                rows,
                vec![
                    // R 6.3.1
                    "The cost of each Government COIN Operation; Garrison pays it once in total.".into(),
                    "Drops one level, if possible, when a Propaganda Round finds Total Support of 18 or less. Events can also shift it.".into(),
                ],
            )
        }
        Item::Resources => {
            let rows = Faction::ALL
                .into_iter()
                .map(|f| (f.name().to_string(), view.resources[f.index()].to_string()))
                .collect();
            let mut notes: Vec<String> = vec![
                // R 1.8
                "Each faction's own amount, from 0 to 49, pays for its Operations.".into(),
                "Transfers go only to or from the faction executing an Operation, Special Activity or Event.".into(),
            ];
            notes.extend(
                Faction::ALL
                    .into_iter()
                    .filter(|f| view.resources[f.index()] >= 45)
                    .map(|f| format!("{} is near the 49 maximum.", f.name())),
            );
            ("Resources", rows, notes)
        }
        Item::Cash => (
            "Cash",
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
                // R 4.5.2
                "Each marker sits under a Guerrilla or cube and belongs to that piece's faction.".into(),
                "Cash is not Resources: it pays no cost directly.".into(),
            ],
        ),
    };
    Shown {
        item,
        title: title.into(),
        rows,
        notes,
        bounds: [0.; 4],
    }
}

/// Lines a note occupies at 17 px in the 420 px column (Lato averages under 8.4 px
/// per character there, so 48 characters always fit one line).
fn note_lines(note: &str) -> usize {
    note.chars().count().div_ceil(48).max(1)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Shown>>,
) {
    let (scale, left) = layout(&window);
    let right = left + Vec2::new(window.width() - WIDTH * scale, 0.);
    let shown = ui
        .hover
        .visible()
        .filter(|_| !ui.reader)
        .and_then(|target| match target {
            Target::Top(item) => Some(*item),
            Target::Standing(_) => None,
        })
        .map(|item| {
            let mut shown = content(&play.game().observe(), item);
            let lines: usize = shown.notes.iter().map(|note| note_lines(note)).sum();
            let height = 88. + shown.rows.len() as f32 * 28. + lines as f32 * 23. + 18.;
            let ([x, y, _, h], anchored_right) = item.target();
            // Tags open just below their element, except the Resources tag, which opens
            // beside the ledger so the Cash supply beneath stays visible.
            let (origin, x, top) = match (item, anchored_right) {
                (Item::Resources, _) => (right, x - 8. - 468., y),
                (_, true) => (right, 1880. - 468., y + h + 8.),
                (_, false) => (left, x.min(1880. - 468.), y + h + 8.),
            };
            shown.bounds = [
                origin.x + x * scale,
                origin.y + top * scale,
                origin.x + (x + 468.) * scale,
                origin.y + (top + height) * scale,
            ];
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
    let mut words = |content: &str, left: f32, top: f32, width: f32, size: f32, ink: Color| {
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
    words(&shown.title, 24., 18., 420., 27., INK);
    for (index, (label, value)) in shown.rows.iter().enumerate() {
        let top = 76. + index as f32 * 28.;
        words(label, 24., top, 200., 18., INK);
        words(value, 230., top, 214., 18., QUIET);
    }
    let mut top = 88. + shown.rows.len() as f32 * 28.;
    for note in &shown.notes {
        words(note, 24., top, 420., 17., QUIET);
        top += note_lines(note) as f32 * 23.;
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
        crate::Game::prepared().observe()
    }

    #[test]
    fn tags_state_the_prepared_values() {
        let view = view();
        let aid = content(&view, Item::Aid);
        assert_eq!(aid.rows, [("Level".into(), "15 of 49".into())]);
        let alliance = content(&view, Item::Alliance);
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
        assert!(!resources.notes.iter().any(|n| n.contains("near the 49")));
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
    fn resources_warn_only_near_the_maximum() {
        let mut view = view();
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
    fn every_level_has_its_own_track_and_unknown_levels_refuse() {
        for (index, name) in LEVELS.iter().enumerate() {
            assert_eq!(level(name), index);
        }
        assert!(std::panic::catch_unwind(|| level("Allied")).is_err());
    }

    #[test]
    fn notes_fit_their_estimated_lines() {
        for item in Item::ALL {
            for note in content(&view(), item).notes {
                assert!(note_lines(&note) <= 3, "{note}");
            }
        }
    }
}
