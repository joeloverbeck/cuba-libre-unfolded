//! Selected open emblems and hover-only victory arithmetic.
//! All decoration is authored in VectorCraft; the game owns the facts.
use super::*;

#[derive(Component)]
pub(super) struct Tag;

#[derive(Component, Clone, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    pub faction: Faction,
    pub title: String,
    pub formula: String,
    pub rows: Vec<(String, String)>,
    pub conditions: Vec<String>,
    pub footer: String,
    pub bounds: [f32; 4],
}

fn center(faction: Faction) -> f32 {
    780. + 120. * faction.index() as f32
}
fn signed(value: i16) -> String {
    if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}
fn color(faction: Faction) -> Color {
    // The selected faction identity palette never signals winning/losing.
    match faction {
        Faction::Government => palette::GOVERNMENT,
        Faction::M26 => palette::M26,
        Faction::Directorio => palette::DIRECTORIO,
        Faction::Syndicate => palette::SYNDICATE,
    }
}

pub(super) fn header(
    commands: &mut Commands,
    art: &Art,
    view: &crate::Observation,
    scale: f32,
    offset: Vec2,
) {
    for standing in &view.standings {
        let faction = standing.faction;
        let x = center(faction);
        picture(
            commands,
            art,
            &format!("atlas-victory-icon-{}.png", slug(faction)),
            [x - 22., 18., 44., 44.],
            scale,
            offset,
            2,
        );
        let margin = text(
            commands,
            art,
            signed(standing.margin),
            x - 50.,
            64.,
            100.,
            40.,
            scale,
            offset,
            2,
        );
        commands
            .entity(margin)
            .insert((StandingText(faction), TextLayout::justify(Justify::Center)));
        // Contiguous hit rectangles expose icon and margin through real picking.
        for (name, top, height) in [("standing", 8., 56.), ("margin", 64., 52.)] {
            commands
                .spawn((
                    UiRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(offset.x + (x - 52.) * scale),
                        top: px(offset.y + top * scale),
                        width: px(104. * scale),
                        height: px(height * scale),
                        ..default()
                    },
                    GlobalZIndex(3),
                    Control {
                        name: format!("{name}:{}", faction.index()),
                        choice: false,
                        action: Action::Standing(faction),
                    },
                ))
                .observe(click);
        }
    }
}

fn content(view: &crate::Observation, faction: Faction) -> Shown {
    let standing = &view.standings[faction.index()];
    let mut rows = vec![];
    let mut bases = 0;
    // Render the model's contributing public facts directly. This is not a
    // parser of Standing's prose, nor an alternate source of totals/margins.
    for space in &view.spaces {
        let multiplier = match faction {
            Faction::Government => space.support.max(0),
            Faction::M26 => (-space.support).max(0),
            Faction::Directorio => i8::from(space.control() == Some(faction)),
            Faction::Syndicate => 0,
        };
        if multiplier > 0 && space.population > 0 {
            rows.push((
                space.name.clone(),
                format!(
                    "{} × {multiplier} = {}",
                    space.population,
                    space.population as i16 * multiplier as i16
                ),
            ));
        }
        for piece in &space.pieces {
            if piece.faction != faction {
                continue;
            }
            if matches!(faction, Faction::M26 | Faction::Directorio) && piece.kind == Kind::Base {
                bases += piece.count as i16;
                rows.push((format!("{} Base", space.name), piece.count.to_string()));
            }
            if faction == Faction::Syndicate
                && piece.kind == Kind::Casino
                && piece.state == State::Open
            {
                rows.push((
                    space.name.clone(),
                    format!(
                        "{} {}",
                        piece.count,
                        if piece.count == 1 {
                            "Casino"
                        } else {
                            "Casinos"
                        }
                    ),
                ));
            }
        }
    }
    let score = standing.score;
    let margin = signed(standing.margin);
    let formula = match faction {
        Faction::Government => format!("{score} Support − 18 = {margin}"),
        Faction::M26 => format!(
            "{} Opposition + {bases} {} − 15 = {margin}",
            score - bases,
            if bases == 1 { "Base" } else { "Bases" }
        ),
        Faction::Directorio => format!(
            "{} Controlled pop. + {bases} Bases − 9 = {margin}",
            score - bases
        ),
        Faction::Syndicate => format!(
            "Lower of {} and {} = {margin}",
            signed(score - 7),
            signed(view.resources[3] as i16 - 30)
        ),
    };
    let met = |yes| if yes { "met" } else { "not met" };
    let conditions = match faction {
        Faction::Government => {
            let cities = view
                .spaces
                .iter()
                .filter(|s| s.terrain == Terrain::City)
                .collect::<Vec<_>>();
            let active = cities.iter().filter(|s| s.support == 2).count();
            let mut lines = vec![
                format!("Support >18 · {}", met(score > 18)),
                format!(
                    "All Cities Active Support · {active}/{} · {}",
                    cities.len(),
                    met(active == cities.len())
                ),
            ];
            lines.extend(
                cities
                    .iter()
                    .filter(|s| s.support != 2)
                    .map(|s| format!("Still needed: {}", s.name)),
            );
            lines
        }
        Faction::M26 => vec![format!("Opposition + Bases >15 · {}", met(score > 15))],
        Faction::Directorio => vec![format!("Controlled pop. + Bases >9 · {}", met(score > 9))],
        Faction::Syndicate => vec![
            format!(
                "Open Casinos >7 · {score} − 7 = {} · {}",
                signed(score - 7),
                met(score > 7)
            ),
            format!(
                "Resources >30 · {} − 30 = {} · {}",
                view.resources[3],
                signed(view.resources[3] as i16 - 30),
                met(view.resources[3] > 30)
            ),
            "Both conditions are required.".into(),
        ],
    };
    let supplies = view
        .available
        .iter()
        .filter(|p| p.faction == faction)
        .map(|p| {
            let kind = match p.kind {
                Kind::Troops => "Troops",
                Kind::Police => "Police",
                Kind::Base if p.count == 1 => "Base",
                Kind::Base => "Bases",
                Kind::Guerrilla if p.count == 1 => "Guerrilla",
                Kind::Guerrilla => "Guerrillas",
                Kind::Casino if p.count == 1 => "Casino",
                Kind::Casino => "Casinos",
            };
            format!("{} {kind}", p.count)
        })
        .collect::<Vec<_>>()
        .join(" · ");
    // Resources are on the top display's ledger (#46); Syndicate's stay in its
    // calculation because they are a victory condition.
    let footer = format!("Available: {supplies}");
    Shown {
        faction,
        title: faction.name().into(),
        formula,
        rows,
        conditions,
        footer,
        bounds: [0.; 4],
    }
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
    let (scale, mut offset) = layout(&window);
    offset.x += (window.width() - WIDTH * scale) / 2.;
    let standing = ui.hover.visible().and_then(|target| match target {
        Target::Standing(faction) => Some(faction),
        Target::Top(_) | Target::Choice(_) | Target::Sequence(_) => None,
    });
    let shown = standing.filter(|_| !ui.reader).map(|faction| {
        let mut shown = content(&play.game().observe(), *faction);
        let height = 246.
            + shown.rows.len() as f32 * 28.
            + shown.conditions.len() as f32 * 27.
            + shown.footer.lines().count() as f32 * 23.;
        let x = (center(*faction) - 234.).clamp(40., WIDTH - 508.);
        shown.bounds = [
            offset.x + x * scale,
            offset.y + 120. * scale,
            offset.x + (x + 468.) * scale,
            offset.y + (120. + height) * scale,
        ];
        shown
    });
    if *previous == shown {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    *previous = shown.clone();
    let Some(shown) = shown else {
        return;
    };
    let [x, y, right, bottom] = shown.bounds;
    let cue = picture(
        &mut commands,
        &art,
        "atlas-victory-hover.png",
        [center(shown.faction) - 52., 4., 104., 112.],
        scale,
        offset,
        1,
    );
    commands.entity(cue).remove::<UiRoot>().insert(Tag);
    let plate = picture(
        &mut commands,
        &art,
        &format!("atlas-victory-tag-{}.png", slug(shown.faction)),
        [x, y, right - x, bottom - y],
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
    // The VectorCraft-authored tag's own text colours, which foundation 4 keeps until a
    // layout decision reopens it (#53). They are not declared as `Atlas` variables: the
    // ink differs from `ink`, and `quiet` only happens to equal `ink-secondary`.
    let ink = Color::srgb_u8(30, 50, 57); // own colour: VectorCraft victory tag (foundation 4)
    let quiet = Color::srgb_u8(83, 106, 109); // own colour: VectorCraft victory tag (foundation 4)
    // The title introduces the faction, so it keeps its name.
    words(Prose::named(&shown.title), 24., 18., 420., 27., ink);
    words(
        (&shown.formula).into(),
        24.,
        59.,
        420.,
        20.,
        color(shown.faction),
    );
    words(
        if shown.faction == Faction::Syndicate {
            "OPEN CASINOS"
        } else {
            "SCORE CONTRIBUTIONS"
        }
        .into(),
        24.,
        112.,
        420.,
        13.,
        quiet,
    );
    for (index, (label, value)) in shown.rows.iter().enumerate() {
        let top = 140. + index as f32 * 28.;
        words(label.into(), 24., top, 270., 18., ink);
        words(value.into(), 300., top, 144., 18., quiet);
    }
    let condition_top = 160. + shown.rows.len() as f32 * 28.;
    words(
        "AT THE NEXT PROPAGANDA CHECK".into(),
        24.,
        condition_top,
        420.,
        13.,
        quiet,
    );
    for (index, line) in shown.conditions.iter().enumerate() {
        words(
            line.into(),
            24.,
            condition_top + 28. + index as f32 * 27.,
            420.,
            18.,
            ink,
        );
    }
    let footer_top = condition_top + 54. + shown.conditions.len() as f32 * 27.;
    words((&shown.footer).into(), 24., footer_top, 420., 17., quiet);
    for top in [98., condition_top - 14.] {
        let rule = picture(
            &mut commands,
            &art,
            "atlas-victory-rule.png",
            [x + 24. * scale, y + top * scale, 420. * scale, scale],
            1.,
            Vec2::ZERO,
            8,
        );
        commands.entity(rule).remove::<UiRoot>().insert(Tag);
    }
    commands.entity(plate).insert(shown);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footers_leave_resources_to_the_ledger_but_syndicate_keeps_its_condition() {
        let view = crate::Game::prepared().observe();
        for faction in Faction::ALL {
            let shown = content(&view, faction);
            assert!(shown.footer.starts_with("Available: "), "{}", shown.footer);
            assert!(!shown.footer.contains("Resources"), "{}", shown.footer);
        }
        assert!(
            content(&view, Faction::Syndicate)
                .conditions
                .iter()
                .any(|line| line.starts_with("Resources >30 · 15 − 30"))
        );
    }
}
