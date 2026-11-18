//! Faction emblems inline in running text (#60). Player-facing sentences name a
//! faction by its accepted victory emblem, drawn at text height on the line's
//! baseline: "[26] pays 1 Resource". Headings and table labels that introduce a
//! faction keep its full name, and so does a briefing's first mention of each faction.
//! The words themselves keep every name: observations and seat reports read them
//! (`Plain`), so automated checks never depend on images.
use super::*;
use bevy::text::{ComputedTextBlock, TextLayoutInfo};
use bevy::ui::widget::InlineImage;

/// Which faction names in a text become emblems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Naming {
    /// Running text, buttons, previews and notices: every name becomes its emblem.
    Emblems,
    /// A briefing: each faction's first mention keeps its name, followed by its
    /// emblem so the reader learns the pairing; later mentions are emblems.
    FirstMention,
    /// A heading or label that introduces a faction keeps every name.
    Names,
}

/// A text to draw and how it names factions. Plain strings are running text.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Prose {
    pub text: String,
    pub naming: Naming,
    /// Drawn on a red surface, such as a primary button, where the red 26July star
    /// would vanish: 26July keeps its name there (local choice, put to the owner in #68).
    pub red: bool,
}
impl Prose {
    pub fn named(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            naming: Naming::Names,
            red: false,
        }
    }
    // The action-choice briefings name factions per text run (`segments_after`).
    #[allow(dead_code)]
    pub fn briefing(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            naming: Naming::FirstMention,
            red: false,
        }
    }
    pub fn on_red(self) -> Self {
        Self { red: true, ..self }
    }
}
impl From<String> for Prose {
    fn from(text: String) -> Self {
        Self {
            text,
            naming: Naming::Emblems,
            red: false,
        }
    }
}
impl From<&str> for Prose {
    fn from(text: &str) -> Self {
        text.to_owned().into()
    }
}
impl From<&String> for Prose {
    fn from(text: &String) -> Self {
        text.clone().into()
    }
}

/// The complete words of a drawn text, with every faction named. The seat reports
/// this instead of the rendered spans.
#[derive(Component, Clone, Debug)]
pub(super) struct Plain(pub String);

/// An emblem drawn inside a line of text.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct Emblem(pub Faction);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Segment {
    Words(String),
    Emblem(Faction),
}

/// Splits `text` into words and emblems. A name becomes an emblem only as a whole
/// word, so "Governments" or "126July" stay text. On a `red` surface 26July keeps its name.
pub(super) fn segments(text: &str, naming: Naming, red: bool) -> Vec<Segment> {
    segments_after(text, naming, red, &mut [false; 4])
}

/// [`segments`] for one of several texts that form a single briefing: `seen` carries
/// the factions already mentioned in its earlier texts, so only the briefing's first
/// mention of each keeps its name.
pub(super) fn segments_after(
    text: &str,
    naming: Naming,
    red: bool,
    seen: &mut [bool; 4],
) -> Vec<Segment> {
    if naming == Naming::Names {
        return vec![Segment::Words(text.into())];
    }
    let mut out = vec![];
    let mut words = String::new();
    let mut rest = text;
    let mut previous: Option<char> = None;
    while let Some(c) = rest.chars().next() {
        let found = Faction::ALL.into_iter().find(|faction| {
            rest.strip_prefix(faction.name()).is_some_and(|after| {
                !previous.is_some_and(char::is_alphanumeric)
                    && !after.chars().next().is_some_and(char::is_alphanumeric)
            })
        });
        if let Some(faction) = found.filter(|f| !(red && *f == Faction::M26)) {
            if naming == Naming::FirstMention && !seen[faction.index()] {
                words.push_str(faction.name());
                words.push(' ');
            }
            seen[faction.index()] = true;
            if !words.is_empty() {
                out.push(Segment::Words(std::mem::take(&mut words)));
            }
            out.push(Segment::Emblem(faction));
            rest = &rest[faction.name().len()..];
            previous = faction.name().chars().last();
        } else {
            words.push(c);
            rest = &rest[c.len_utf8()..];
            previous = Some(c);
        }
    }
    if !words.is_empty() || out.is_empty() {
        out.push(Segment::Words(words));
    }
    out
}

/// An emblem's height as a fraction of the type size. Lato's line reserves about
/// 0.94 em above the baseline (measured natively), so a 0.9 em emblem stays inside its line.
pub(super) const HEIGHT: f32 = 0.9;

/// The admitted export drawn for a faction's emblem.
pub(super) fn image(faction: Faction) -> String {
    format!("atlas-victory-icon-{}.png", slug(faction))
}

/// Adds the words and emblems after the first segment as children of the text
/// `root`. Each emblem is `height` logical pixels tall. Like their root text, the
/// spans and emblems take no pick: a click on them reaches the control beneath, and
/// is never taken for a click on the map (which closes the AC-B briefing tag).
pub(super) fn spawn_rest(
    commands: &mut Commands,
    art: &Art,
    root: Entity,
    rest: &[Segment],
    font: &TextFont,
    ink: TextColor,
    height: f32,
) {
    for segment in rest {
        match segment {
            Segment::Words(words) => {
                commands.spawn((
                    TextSpan::new(words.clone()),
                    font.clone(),
                    ink,
                    Pickable::IGNORE,
                    ChildOf(root),
                ));
            }
            Segment::Emblem(faction) => {
                commands.spawn((
                    InlineImage {
                        image: art.images[&image(*faction)].clone(),
                        height: Some(height),
                        ..default()
                    },
                    Emblem(*faction),
                    Pickable::IGNORE,
                    ChildOf(root),
                ));
            }
        }
    }
}

type Recolored<'w, 's> =
    Query<'w, 's, (&'static TextColor, &'static Children), (With<Text>, Changed<TextColor>)>;
/// Callers recolor a text after drawing it; its later words follow the root's ink.
pub(super) fn ink(
    roots: Recolored,
    mut spans: Query<&mut TextColor, (With<TextSpan>, Without<Text>)>,
) {
    for (ink, children) in &roots {
        for child in children {
            if let Ok(mut span) = spans.get_mut(*child) {
                *span = *ink;
            }
        }
    }
}

/// A text's type size in logical pixels, when it is set in pixels, as the development
/// seat reports it.
pub(super) fn type_size(font: &TextFont) -> Option<f32> {
    match font.font_size {
        FontSize::Px(px) => Some(px),
        _ => None,
    }
}

/// Every drawn text containing an emblem, for the development seat. Each entry has
/// the text's words with every faction named, its font size and bounds, and for each
/// emblem its faction, bounds, the baseline of its line (the commonest bottom of
/// the letters beside it) and whether text glued to it ("[S]." or "[26]'s") wrapped
/// away from it. Rectangles are logical window pixels, [left, top, right, bottom].
pub(super) fn observe(world: &mut World) -> Vec<serde_json::Value> {
    let mut texts = vec![];
    let mut query = world.query::<(
        &Plain,
        &TextFont,
        &ComputedNode,
        &UiGlobalTransform,
        &TextLayoutInfo,
        &ComputedTextBlock,
    )>();
    for (plain, font, node, at, layout, block) in query.iter(world) {
        if layout.inline_boxes.is_empty() {
            continue;
        }
        let scale = node.inverse_scale_factor;
        let origin = at.translation * scale - node.size() * scale / 2.;
        let logical = |r: Rect| {
            [
                origin.x + r.min.x * scale,
                origin.y + r.min.y * scale,
                origin.x + r.max.x * scale,
                origin.y + r.max.y * scale,
            ]
        };
        let sections: Vec<Entity> = block.entities().iter().map(|e| e.entity).collect();
        let emblems: Vec<_> = layout
            .inline_boxes
            .iter()
            .map(|(entity, _, r)| {
                // Most letters and digits rest on the baseline; symbols such as "×",
                // "·" and "+" float above it and descenders drop below. The commonest
                // whole-pixel bottom is the baseline.
                let mut counts = BTreeMap::<i32, usize>::new();
                for g in layout
                    .glyphs
                    .iter()
                    .filter(|g| g.position.y > r.min.y && g.position.y < r.max.y)
                {
                    *counts
                        .entry((g.position.y + g.atlas_info.rect.size().y / 2.).round() as i32)
                        .or_default() += 1;
                }
                let baseline = counts
                    .into_iter()
                    .max_by_key(|(bottom, count)| (*count, *bottom))
                    .map(|(bottom, _)| origin.y + bottom as f32 * scale);
                let follower = sections
                    .iter()
                    .position(|e| e == entity)
                    .map(|index| index + 1);
                let wrapped = follower.and_then(|f| {
                    layout
                        .glyphs
                        .iter()
                        .find(|g| g.section_index as usize == f)
                        // On the next line even a full stop starts below this baseline.
                        .map(|g| g.position.y - g.atlas_info.rect.size().y / 2. > r.max.y)
                });
                (
                    *entity,
                    logical(*r),
                    baseline,
                    follower.and_then(|f| sections.get(f).copied()),
                    wrapped,
                )
            })
            .collect();
        texts.push((
            plain.0.clone(),
            type_size(font),
            [
                origin.x,
                origin.y,
                origin.x + node.size().x * scale,
                origin.y + node.size().y * scale,
            ],
            emblems,
        ));
    }
    texts
        .into_iter()
        .map(|(text, size, bounds, emblems)| {
            let emblems = emblems
                .into_iter()
                .map(|(entity, rect, baseline, follower, wrapped)| {
                    let glued = follower
                        .and_then(|f| world.get::<TextSpan>(f))
                        .is_some_and(|s| s.0.chars().next().is_some_and(|c| !c.is_whitespace()));
                    serde_json::json!({
                        "faction": world.get::<Emblem>(entity).map(|e| e.0.name()),
                        "bounds": rect,
                        "baseline": baseline,
                        "glued_text_wrapped": glued && wrapped == Some(true),
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({"text": text, "font_size": size, "bounds": bounds, "emblems": emblems})
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Segment::{Emblem as E, Words as W};
    fn w(text: &str) -> Segment {
        W(text.into())
    }

    #[test]
    fn running_text_names_factions_by_emblem() {
        assert_eq!(
            segments("Syndicate gave 1 to 26July. Saved.", Naming::Emblems, false),
            [
                E(Faction::Syndicate),
                w(" gave 1 to "),
                E(Faction::M26),
                w(". Saved.")
            ]
        );
        assert_eq!(
            segments("Kidnap · Santiago · Government", Naming::Emblems, false),
            [w("Kidnap · Santiago · "), E(Faction::Government)]
        );
    }

    #[test]
    fn only_whole_names_become_emblems() {
        for text in ["Governments", "126July", "26Julys", "No faction here", ""] {
            assert_eq!(segments(text, Naming::Emblems, false), [w(text)]);
        }
        assert_eq!(
            segments("(Government → 26July)", Naming::Emblems, false),
            [
                w("("),
                E(Faction::Government),
                w(" → "),
                E(Faction::M26),
                w(")")
            ]
        );
    }

    #[test]
    fn headings_keep_names_and_briefings_name_each_faction_once() {
        let text = "26July · Terror";
        assert_eq!(segments(text, Naming::Names, false), [w(text)]);
        assert_eq!(
            segments(
                "Directorio may act. 26July pays; Directorio then Passes.",
                Naming::FirstMention,
                false
            ),
            [
                w("Directorio "),
                E(Faction::Directorio),
                w(" may act. 26July "),
                E(Faction::M26),
                w(" pays; "),
                E(Faction::Directorio),
                w(" then Passes.")
            ]
        );
    }

    #[test]
    fn red_surfaces_keep_26july_named_and_draw_the_others() {
        assert_eq!(
            segments("Syndicate → 26July", Naming::Emblems, true),
            [E(Faction::Syndicate), w(" → 26July")]
        );
        assert_eq!(
            segments("26July → Government", Naming::Emblems, true),
            [w("26July → "), E(Faction::Government)]
        );
    }

    #[test]
    fn every_emblem_is_an_admitted_victory_export() {
        let manifest = crate::assets::VICTORY_MANIFEST;
        for faction in Faction::ALL {
            assert!(
                manifest.contains(&format!("  art/atlas/{}", image(faction))),
                "{faction:?}"
            );
        }
    }
}
