//! A choice's preview, shown while the pointer rests on its button: the consequences
//! the rules authority computed for an irreversible step, or why a disabled choice
//! cannot be taken now and what in this task would allow it. This module only words
//! and places the game's facts. It is a temporary card beside the Operation panel.
//!
//! Temporary bootstrap (WORK-2; foundation 4 "Authoring boundary"). The card's
//! geometry (`card::X` 880, `Y` 287, `WIDTH` 488 and its offsets) is an interim
//! layout invented in code, not an authored FigCraft design.
//! - Owner: #58.
//! - Purpose: let the development seat exercise previews and action choices before
//!   #68 selects designs.
//! - Allowed consumers: agent verification of #58 only.
//! - Prohibited dependents: #16's owner review, any release, and any other UI
//!   positioning itself against this card.
//! - Appetite: keep it working for #58's checks; no design or polish effort.
//! - Replacement: the selected designs in #69 (action choice and briefings) and
//!   #72/#73 (map popovers for targets and removals). #72's space popover now
//!   previews Terror and opening Kidnap on the map, and #73's piece popover previews
//!   each removal, whose buttons left the panel; this card still serves the panel's
//!   remaining buttons until #69 replaces it.
//! - Expiry: before #75's end-to-end verification.
use super::*;
use crate::{Choice, KidnapStakes, Observation, Preview, SequenceBox};

/// Card geometry in reference pixels, right-anchored beside the Operation panel.
mod card {
    pub const X: f32 = 880.;
    pub const Y: f32 = 287.;
    pub const WIDTH: f32 = 488.;
    pub const TITLE: f32 = 27.;
    pub const BODY_TOP: f32 = 70.;
    pub const LINE: f32 = 20.5;
    pub const BOTTOM: f32 = 18.;
    /// Lines are broken by hand at this many characters, which a 440 px column at
    /// 17 px holds with margin (native captures measure about 7.4 px a character),
    /// so each authored line is one drawn line.
    pub const CHARS: usize = 56;
}
use super::top::{INK, QUIET};

#[derive(Component)]
pub(super) struct Tag;

/// What the card shows, as the development seat reads it.
#[derive(Component, Clone, Debug, serde::Serialize, PartialEq)]
pub(super) struct Shown {
    pub choice: String,
    pub title: String,
    pub lines: Vec<String>,
    pub bounds: [f32; 4],
}

fn space_name<'a>(view: &'a Observation, id: &'a str) -> &'a str {
    view.spaces
        .iter()
        .find(|s| s.id == id)
        .map_or(id, |s| s.name.as_str())
}

fn piece(kind: Kind, state: State) -> String {
    match (kind, state) {
        (Kind::Guerrilla, State::Underground) => "Underground Guerrilla".into(),
        (Kind::Guerrilla, State::Active) => "Active Guerrilla".into(),
        (kind, _) => format!("{kind:?}"),
    }
}

fn controller(faction: Option<Faction>) -> String {
    faction.map_or("uncontrolled".into(), |f| format!("{} Control", f.name()))
}

fn record(record: SequenceBox) -> &'static str {
    match record {
        SequenceBox::Pass => "Pass (R 2.3.3)",
        SequenceBox::OperationOnly => "Operation Only (R 2.3.4)",
        SequenceBox::OperationWithSpecialActivity => "Operation with Special Activity (R 2.3.4)",
    }
}

/// A Kidnap roll's possible proceeds as a range of Resources, such as "1–6 Resources"
/// or "0 Resources" once Government has none.
pub(super) fn proceeds(stakes: &KidnapStakes) -> String {
    match (stakes.proceeds.first(), stakes.proceeds.last()) {
        (Some(low), Some(high)) if low != high => format!("{low}–{high} Resources"),
        (Some(1), _) => "1 Resource".into(),
        (Some(n), _) => format!("{n} Resources"),
        _ => "nothing".into(),
    }
}

/// The preview's facts as sentences, in the order a player checks them.
fn consequences(view: &Observation, preview: &Preview) -> Vec<String> {
    let mut lines = vec![];
    if let Some(stakes) = &preview.kidnap {
        lines.push(format!(
            "Target: {} in {} (R 4.3.3), which has {}",
            stakes.target.name(),
            space_name(view, &stakes.space),
            stakes.government
        ));
        lines.push(format!(
            "Possible proceeds: {} · guaranteed minimum {}",
            proceeds(stakes),
            stakes.minimum
        ));
    }
    for r in &preview.resources {
        lines.push(format!(
            "{} Resources: {} → {}",
            r.faction.name(),
            r.from,
            r.to
        ));
    }
    for f in &preview.flips {
        lines.push(format!(
            "{}: flip {} {} to {}",
            space_name(view, &f.space),
            f.count,
            piece(f.kind, f.from),
            piece(f.kind, f.to).replace(" Guerrilla", "")
        ));
    }
    for r in &preview.removed {
        lines.push(format!(
            "{}: {} {} {} returns to Available",
            space_name(view, &r.space),
            r.count,
            r.faction.name(),
            piece(r.kind, r.state)
        ));
    }
    for space in &preview.terror {
        lines.push(format!("{}: Terror marker placed", space_name(view, space)));
    }
    for s in &preview.support {
        lines.push(format!("{}: {}", space_name(view, &s.space), s.label));
    }
    for c in &preview.control {
        lines.push(format!(
            "{}: {} → {}",
            space_name(view, &c.space),
            controller(c.from),
            controller(c.to)
        ));
    }
    for s in &preview.standings {
        lines.push(format!(
            "{} victory standing {} → {} (margin {} → {})",
            s.faction.name(),
            s.score_from,
            s.score_to,
            s.margin_from,
            s.margin_to
        ));
    }
    if lines.is_empty() {
        lines.push("No Resources, pieces, support, Control or standings change.".into());
    }
    for u in &preview.unavailable {
        lines.push(format!("Becomes unavailable: {}", u.label));
    }
    if let Some(records) = preview.records {
        lines.push(format!("Records: {}", record(records)));
    }
    if let Some(next) = &preview.next {
        lines.push(format!(
            "{}, {}, may: {}",
            next.faction.name(),
            next.role,
            next.options.join(" · ")
        ));
    }
    lines.extend(preview.notes.iter().cloned());
    lines
}

/// The card for one listed choice.
fn content(view: &Observation, choice: &Choice) -> Shown {
    let mut sentences = vec![];
    if !choice.supported {
        sentences.push(format!("Unavailable now. {}", choice.explanation));
        sentences.push(if choice.remedies.is_empty() {
            "Nothing in this task can make it available.".into()
        } else {
            format!("In this task: {}.", choice.remedies.join(" · "))
        });
    } else if let Some(preview) = &choice.preview {
        sentences.extend(consequences(view, preview));
    } else {
        sentences.push(choice.explanation.clone());
    }
    Shown {
        choice: choice.id.clone(),
        title: choice.label.clone(),
        lines: sentences
            .iter()
            .flat_map(|s| wrap(s, card::CHARS))
            .collect(),
        bounds: [0.; 4],
    }
}

/// Breaks a sentence into lines of at most `width` characters at spaces.
pub(super) fn wrap(sentence: &str, width: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut line = String::new();
    for word in sentence.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[allow(clippy::too_many_arguments)] // The shared hover, cached card and layout inputs.
pub(super) fn draw(
    mut commands: Commands,
    play: Res<Play>,
    ui: Res<Interface>,
    art: Res<Art>,
    window: Single<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Tag>>,
    mut previous: Local<Option<Shown>>,
    mut cached: Local<Option<Shown>>,
) {
    let (scale, left) = layout(&window);
    let right = left + Vec2::new(window.width() - WIDTH * scale, 0.);
    let target = ui
        .hover
        .visible()
        .filter(|_| !ui.reader)
        .and_then(|target| match target {
            Target::Choice(id) => Some(id.clone()),
            Target::Standing(_) | Target::Top(_) => None,
        });
    // A choice id carries its match revision, so its card is computed once rather
    // than observing every preview on every frame.
    if let Some(id) = &target
        && cached.as_ref().is_none_or(|c| &c.choice != id)
    {
        let view = play.game().observe();
        *cached = view
            .choices
            .iter()
            .find(|c| &c.id == id)
            .map(|choice| content(&view, choice));
    }
    let shown = target
        .and_then(|id| cached.clone().filter(|c| c.choice == id))
        .map(|mut shown| {
            let height = card::BODY_TOP + shown.lines.len() as f32 * card::LINE + card::BOTTOM;
            shown.bounds = [
                right.x + card::X * scale,
                right.y + card::Y * scale,
                right.x + (card::X + card::WIDTH) * scale,
                right.y + (card::Y + height) * scale,
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
    let mut words = |content: String, top: f32, size: f32, ink: Color| {
        let entity = text(
            &mut commands,
            &art,
            content,
            x + 24. * scale,
            y + top * scale,
            440. * scale,
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
    words(shown.title.clone(), 18., card::TITLE * 0.8, INK);
    words(shown.lines.join("\n"), card::BODY_TOP, 17., QUIET);
    let rule = picture(
        &mut commands,
        &art,
        "atlas-victory-rule.png",
        [x + 24. * scale, y + 58. * scale, 440. * scale, scale],
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

    fn card(game: &Game, action: &str) -> Shown {
        let view = game.observe();
        let id = format!("{}:{action}", view.revision);
        let choice = view.choices.iter().find(|c| c.id == id).unwrap();
        content(&view, choice)
    }

    #[test]
    fn a_disabled_target_states_its_reason_and_remedies() {
        let lines = card(&Game::prepared(), "terror:santiago").lines.join(" ");
        assert!(
            lines.contains("Cost: 1 Resource per space (26July has 0)"),
            "{lines}"
        );
        assert!(
            lines.contains("In this task: Kidnap here first · Request a Resource transfer."),
            "{lines}"
        );
    }

    #[test]
    fn the_zero_space_end_terror_card_warns_and_names_directorio() {
        let lines = card(&Game::prepared(), "end-terror").lines.join(" ");
        for fact in [
            "Records: Operation Only (R 2.3.4)",
            "Directorio, 2nd Eligible, may: Pass (+1 if executed later) · Limited Operation",
            "no Pass income",
        ] {
            assert!(lines.contains(fact), "{fact}: {lines}");
        }
    }

    #[test]
    fn the_kidnap_card_shows_the_stakes_and_no_result() {
        let mut game = Game::prepared();
        let open = card(&game, "kidnap").lines.join(" ");
        game.submit(Faction::M26, 0, "0:kidnap").unwrap();
        let roll = card(&game, "kidnap:roll").lines.join(" ");
        for lines in [&open, &roll] {
            for fact in [
                "Target: Government in Santiago de Cuba (R 4.3.3), which has 6",
                "Possible proceeds: 1–6 Resources · guaranteed minimum 1",
                "accepted commitment policy",
                "Becomes unavailable: End Terror",
            ] {
                assert!(lines.contains(fact), "{fact}: {lines}");
            }
            assert!(!lines.contains("No Resources"), "{lines}");
        }
        assert!(open.contains("Opening rolls nothing"), "{open}");
    }

    #[test]
    fn lines_fit_the_column() {
        for action in ["end-terror", "terror:santiago", "terror:oriente", "kidnap"] {
            for line in card(&Game::prepared(), action).lines {
                assert!(line.chars().count() <= card::CHARS, "{line}");
            }
        }
    }
}
