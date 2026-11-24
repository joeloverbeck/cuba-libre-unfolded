//! Briefings for 26July's options as 1st Eligible on card 17, as the selected AC-B
//! briefing tag shows them (`art/atlas/CHOICE.md`). Each says, in general terms, what
//! the option does in eligible spaces and why a faction takes it, its cost against
//! current Resources, what Directorio may do next, and the rules. They state facts
//! and never recommend a move (#58 story 7).
//!
//! The owner narrowed the briefings for #69 (#58, issuecomment-6940684089): they name
//! no province and compute no space's result, which the map popover previews (#72).
//! Pass keeps its exact numbers, from the rules authority's preview, since it involves
//! no space. Current Resources come from the observation; the wording only places them.
use super::emblems::{self, Naming, Segment};
use crate::game::CARD_17_UNSHADED;
use crate::{
    CARD_17_SHADED, Choice, Faction, Flip, KidnapStakes, Observation, Preview, TERROR_COST,
    resources,
};

/// Value lines are broken by hand at this many characters, which the 420 px column
/// holds at 15 px with margin (Lato averages about 6.5 px a character there), so each
/// authored line is one drawn line and the tag's height follows its content.
pub(super) const CHARS: usize = 58;
/// The same for the 12.5 px rules line.
pub(super) const RULES_CHARS: usize = 70;

/// One labelled entry: PURPOSE, COST, STEPS, RESULT, STATUS or NEXT, or a step's
/// preview entry (#84). `chips` are in-task remedies, drawn as HF-A remedy chips.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Entry {
    pub label: String,
    pub text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chips: Vec<String>,
}

/// A briefing's content. `option` is the choice id without its revision, such as
/// "pass" or "terror:kidnap".
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Briefing {
    pub option: String,
    pub title: String,
    pub rule: &'static str,
    pub entries: Vec<Entry>,
    pub rules: String,
}

pub(super) fn entry(label: impl Into<String>, text: impl Into<String>) -> Entry {
    Entry {
        label: label.into(),
        text: text.into(),
        chips: vec![],
    }
}

/// The choice listed now whose id ends with `:{action}`.
fn listed<'a>(view: &'a Observation, action: &str) -> Option<&'a Choice> {
    view.choices.iter().find(|c| c.action() == action)
}

/// "A, B or C".
fn either(options: &[String]) -> String {
    match options {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

const NOT_IN_TASK: &str =
    "Not in this task: listed so the option set is complete. It cannot be chosen here.";

fn card_17_second(view: &Observation) -> Faction {
    view.eligibility
        .second
        .expect("card 17 has a 2nd Eligible faction")
}

/// What the 2nd Eligible faction may do once 26July's action is recorded (R 2.3.4).
fn second_after_operation(view: &Observation) -> String {
    format!(
        "{}, 2nd Eligible: Pass or a Limited Operation; with a Special Activity, also card 17's Event (R 2.3.4).",
        card_17_second(view).name()
    )
}

fn second_after_event(view: &Observation) -> String {
    format!(
        "{}, 2nd Eligible: Pass, or an Operation with a Special Activity if desired (R 2.3.4).",
        card_17_second(view).name()
    )
}

fn pass(view: &Observation, m26: u8) -> Briefing {
    let preview: Option<&Preview> = listed(view, "pass").and_then(|c| c.preview.as_ref());
    let mut result = vec![];
    let next = preview.and_then(|p| p.next.as_ref());
    if let Some(preview) = preview {
        for change in &preview.resources {
            let gained = change.to.saturating_sub(change.from);
            result.push(format!(
                "+{} ({} → {}).",
                resources(gained),
                change.from,
                change.to
            ));
        }
        match preview.m26_next_card {
            Some(true) => result.push(format!("{} stays Eligible.", Faction::M26.name())),
            Some(false) => result.push(format!(
                "{} becomes Ineligible after this card.",
                Faction::M26.name()
            )),
            None => {}
        }
        if preview.standings.is_empty() && preview.control.is_empty() {
            result.push("Standings and Control unchanged.".into());
        }
    }
    let mut entries = vec![
        entry(
            "PURPOSE",
            "Takes no action on card 17 and gains a Resource.",
        ),
        entry("COST", format!("None · 26July has {m26}.")),
    ];
    if !result.is_empty() {
        entries.push(entry("RESULT", result.join(" ")));
    }
    let mut rules = "Rules: R 2.3.3, R 2.3.4.".to_owned();
    if let Some(next) = next {
        let faction = next.faction.name();
        entries.push(entry(
            "NEXT",
            format!(
                "{faction} becomes {}: {}. The task ends there.",
                next.role,
                either(&next.options)
            ),
        ));
        rules.push_str(&format!(
            " The task shows {faction}'s options without executing them."
        ));
    }
    Briefing {
        option: "pass".into(),
        title: "Pass".into(),
        rule: "R 2.3.3",
        entries,
        rules,
    }
}

fn terror(view: &Observation, m26: u8) -> Briefing {
    Briefing {
        option: "terror".into(),
        title: "Terror · Operation".into(),
        rule: "R 3.3.4",
        entries: vec![
            entry(
                "PURPOSE",
                "Shifts each selected space 1 level toward Active Opposition, which counts toward 26July's victory (R 7.2).",
            ),
            entry(
                "COST",
                format!(
                    "{} per City or Province · 26July has {m26}. While 26July executes Terror, Kidnap proceeds or a Resource transfer can add Resources.",
                    resources(TERROR_COST)
                ),
            ),
            entry(
                "STEPS",
                "Select spaces with an Underground 26July Guerrilla, each once. In each: pay, flip 1 Underground Guerrilla there to Active, place a Terror marker, shift 1 level toward Active Opposition. Each space's result is previewed before it commits.",
            ),
            entry(
                "NEXT",
                format!(
                    "{}, 2nd Eligible: Pass or a Limited Operation; after Kidnap, also card 17's Event (R 2.3.4).",
                    card_17_second(view).name()
                ),
            ),
        ],
        rules: "Rules: R 3.3.4, R 2.3.4. Going back to the action choice before the first effect is local policy.".into(),
    }
}

/// Kidnap's possible proceeds, from the rules authority's stakes when it offers them
/// (opening Kidnap and its roll); otherwise the authority's stakes in the observation
/// (R 4.3.3: a die 1–6, at most what Government has).
fn kidnap_result(view: &Observation) -> String {
    if let Some(roll) = &view.kidnap {
        return format!(
            "Rolled {}: Government gave {}.",
            roll.die,
            resources(roll.proceeds)
        );
    }
    let stakes = listed(view, "kidnap:roll")
        .or_else(|| listed(view, "kidnap"))
        .and_then(|c| c.preview.as_ref())
        .and_then(|p| p.kidnap.as_ref());
    if let Some(stakes) = stakes {
        return format!(
            "Proceeds {} while Government has {}; at least {}.",
            proceeds(stakes),
            stakes.government,
            stakes.minimum
        );
    }
    if let Some(choice) = listed(view, "kidnap").filter(|c| !c.supported) {
        // The rules authority's reason, without the space it names: its first sentence
        // compares the Guerrillas and Police there; its last says whether the Terror
        // Kidnap accompanies is still guaranteed.
        let first = choice
            .explanation
            .split_inclusive(". ")
            .next()
            .unwrap_or_default()
            .trim();
        let mut result = format!("Unavailable now. {first}");
        if !choice.explanation.contains("guaranteed feasible") {
            result.push_str(
                " The Terror it accompanies needs an Underground Guerrilla and guaranteed payment.",
            );
        }
        return result;
    }
    match &view.kidnap_stakes {
        Some(stakes) => format!(
            "Proceeds {} while Government has {}; at least {}.",
            proceeds(stakes),
            stakes.government,
            stakes.minimum
        ),
        None => "Not offered now.".into(),
    }
}

fn kidnap(view: &Observation, m26: u8) -> Briefing {
    let mut entries = vec![
        entry(
            "PURPOSE",
            "Takes Resources from Government (City or economic center) or the Syndicate (open Casino), in a space selected for this Terror.",
        ),
        entry(
            "COST",
            format!(
                "No Resource · 26July has {m26}. It must accompany Terror in its space; rolling before that Terror makes it owed (accepted commitment policy)."
            ),
        ),
        entry(
            "STEPS",
            "Needs more 26July Guerrillas than Police there. Open it to preview, then roll one die; the target gives up to the result.",
        ),
        entry("RESULT", kidnap_result(view)),
        entry(
            "NEXT",
            format!(
                "{} may then also take card 17's Event (R 2.3.4, 5.2). Shaded: {CARD_17_SHADED}",
                card_17_second(view).name()
            ),
        ),
    ];
    if view.action == crate::ActionChoice::Open {
        entries.insert(3, entry("STATUS", "Offered once Terror is chosen."));
    }
    Briefing {
        option: "terror:kidnap".into(),
        title: "Kidnap · Special Activity".into(),
        rule: "R 4.3.3",
        entries,
        rules: "Rules: R 4.1, R 4.3.3, R 2.3.4, R 5.2.".into(),
    }
}

/// An option listed but not in this task: what it is for, its cost, its status and
/// what Directorio could do after it.
fn absent(
    option: &str,
    title: &str,
    rule: &'static str,
    purpose: &str,
    cost: String,
    next: String,
) -> Briefing {
    Briefing {
        option: option.into(),
        title: format!("{title} · not in this task"),
        rule,
        entries: vec![
            entry("PURPOSE", purpose),
            entry("COST", cost),
            entry("STATUS", NOT_IN_TASK),
            entry("NEXT", next),
        ],
        rules: format!("Rules: {rule}, R 2.3.4."),
    }
}

/// Directorio's legal options and the recorded reason the Event is available.
fn directorio(view: &Observation, option: &str) -> Option<Briefing> {
    let choice = listed(view, option)?;
    let (title, rule, printed) = match option {
        "directorio:pass" => ("Pass", "R 2.3.3", None),
        "directorio:limited-operation" => ("Limited Operation", "R 2.3.5", None),
        "event:unshaded" => (
            "Alberto Bayo · unshaded",
            "R 5.2",
            Some(("VET TRAINS GUERRILLAS", CARD_17_UNSHADED)),
        ),
        "event:shaded" => (
            "Alberto Bayo · shaded",
            "R 5.2",
            Some(("MEXICO BLOCKS TRAINING BY CUBAN EXPAT", CARD_17_SHADED)),
        ),
        _ => return None,
    };
    let mut entries = Vec::new();
    if let Some((heading, text)) = printed {
        entries.push(entry(heading, text));
        entries.push(entry(
            "WHY DIRECTORIO MAY CHOOSE",
            "Kidnap made Terror an Operation with Special Activity, opening the Event to the 2nd Eligible faction (R 2.3.4, 5.2).",
        ));
        if let Some(roll) = &view.kidnap {
            let targets: Vec<_> = view
                .resolved
                .iter()
                .filter_map(|r| view.spaces.iter().find(|s| s.id == r.space))
                .map(|s| s.name.as_str())
                .collect();
            let paid = view.resolved.iter().map(|r| r.paid).sum();
            entries.push(entry(
                "26JULY'S RESULT",
                format!(
                    "Terror in {} · paid {}. Kidnap rolled {} and took {} from Government.",
                    targets.join(", "),
                    resources(paid),
                    roll.die,
                    resources(roll.proceeds)
                ),
            ));
        }
        if option == "event:shaded"
            && let Some(preview) = &choice.preview
        {
            entries.push(entry("ACTIVATE", activation_lines(view, &preview.flips)));
        }
    }
    if choice.supported {
        entries.push(entry("COST", &choice.explanation));
    } else {
        entries.push(entry("IN THIS BUILD", &choice.explanation));
    }
    Some(Briefing {
        option: option.into(),
        title: title.into(),
        rule,
        entries,
        rules: String::new(),
    })
}

pub(super) fn activation_lines(view: &Observation, flips: &[Flip]) -> String {
    if flips.is_empty() {
        return "No Underground 26July Guerrillas remain.".into();
    }
    flips
        .iter()
        .map(|flip| {
            let space = view
                .spaces
                .iter()
                .find(|s| s.id == flip.space)
                .map_or(flip.space.as_str(), |s| s.name.as_str());
            format!(
                "{space} · {} Underground Guerrilla{} → Active",
                flip.count,
                if flip.count == 1 { "" } else { "s" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The briefing for one option at the action choice, or for the Terror and Kidnap
/// tabs during execution. `option` is the choice id without its revision.
pub(super) fn content(view: &Observation, option: &str) -> Option<Briefing> {
    if view.current_card == 18 {
        let choice = listed(view, option)?;
        if view.passed[Faction::Government.index()] {
            return Some(Briefing {
                option: option.into(),
                title: choice.label.clone(),
                rule: "R 2.3.3–4",
                entries: vec![
                    entry("OPTION", &choice.explanation),
                    entry(
                        "WHY SYNDICATE IS FIRST",
                        "Government Passed. Syndicate is the next Eligible faction in card order and inherits the 1st Eligible options.",
                    ),
                ],
                rules: String::new(),
            });
        }
        if view.eligibility.government_box == Some(crate::SequenceBox::OperationOnly) {
            return Some(Briefing {
                option: option.into(),
                title: choice.label.clone(),
                rule: "R 2.3.3–5",
                entries: vec![
                    entry("OPTION", &choice.explanation),
                    entry(
                        "2ND ELIGIBLE",
                        "Government recorded Operation Only. Syndicate may Pass or choose a Limited Operation.",
                    ),
                ],
                rules: String::new(),
            });
        }
        if view.executing == Some(Faction::Government) {
            let (rule, label) = match option {
                "end-assault" => ("R 2.3.4", "RESULT"),
                "transfer" => ("R 1.5.1, 1.8", "GIVER'S DECISION"),
                "remove" | "done-removal" => ("R 1.4.1", "VOLUNTARY REMOVAL"),
                _ if view.transfer.is_some() => ("R 1.5.1, 1.8", "GIVER'S DECISION"),
                _ if option.starts_with("remove:") => ("R 1.4.1", "VOLUNTARY REMOVAL"),
                _ => ("R 3.1, 3.2.4, 6.3.1", "ASSAULT"),
            };
            // An Assault plan is no panel option: its space's popover previews it (#169).
            let entries = vec![entry(label, &choice.explanation)];
            return Some(Briefing {
                option: option.into(),
                title: choice.label.clone(),
                rule,
                entries,
                rules: String::new(),
            });
        }
        let role = if view.eligibility.event_restriction.is_some() {
            "26July is Ineligible through card 18 under the shaded Event. Directorio executed the Event."
        } else {
            "26July executed Terror and Directorio executed the Event on card 17."
        };
        return Some(Briefing {
            option: option.into(),
            title: if option == "government:pass" {
                "Pass".into()
            } else {
                choice.label.clone()
            },
            rule: "R 2.3.1–4",
            entries: vec![
                entry("OPTION", &choice.explanation),
                entry(
                    "WHY GOVERNMENT IS FIRST",
                    format!(
                        "{role} Government and Syndicate took no action on card 17. Government's symbol comes before Syndicate's."
                    ),
                ),
            ],
            rules: String::new(),
        });
    }
    if view.eligibility.directorio_handoff() {
        return directorio(view, option);
    }
    if view
        .event
        .as_ref()
        .is_some_and(crate::EventProgress::pending)
    {
        let choice = listed(view, option)?;
        let (rule, label) = match option {
            "resolve-shaded" => ("R 5.1, 5.2", "EVENT EFFECT"),
            "transfer" => ("R 1.5.1", "GIVER'S DECISION"),
            "remove" | "done-removal" => ("R 1.4.1", "VOLUNTARY REMOVAL"),
            _ if matches!(
                view.event,
                Some(crate::EventProgress::ChooseRallyBeneficiary)
            ) =>
            {
                ("R 5.1", "QUALIFYING BASE SPACES")
            }
            _ => ("R 3.3.1, 5.5", "PLACEMENT AND COST"),
        };
        let mut entries = vec![entry(label, &choice.explanation)];
        let mut rules = String::new();
        if let Some(preview) = &choice.preview {
            let (effects, notes) = super::execution::previewed(view, preview);
            entries.extend(effects);
            rules = notes.join(" ");
        }
        return Some(Briefing {
            option: option.into(),
            title: choice.label.clone(),
            rule,
            entries,
            rules,
        });
    }
    let m26 = view.resources[Faction::M26.index()];
    let per_space = format!("1 Resource per space · 26July has {m26}.");
    let free = format!("No Resource beyond its Operation · 26July has {m26}.");
    Some(match option {
        "pass" => pass(view, m26),
        "terror" => terror(view, m26),
        "terror:kidnap" => kidnap(view, m26),
        "event:unshaded" => absent(
            option,
            "Unshaded Event",
            "R 5.2",
            &format!("Card 17, Vet trains guerrillas: {CARD_17_UNSHADED}"),
            format!("None · 26July has {m26}."),
            second_after_event(view),
        ),
        "event:shaded" => absent(
            option,
            "Shaded Event",
            "R 5.2",
            &format!(
                "Card 17, Mexico blocks training by Cuban expat: {CARD_17_SHADED} Any faction may choose either text."
            ),
            format!("None · 26July has {m26}."),
            second_after_event(view),
        ),
        "rally" => absent(
            option,
            "Rally",
            "R 3.3.1",
            "Places 26July Guerrillas or a Base, or flips Guerrillas Underground, in Neutral or Opposition spaces.",
            per_space,
            second_after_operation(view),
        ),
        "rally:infiltrate" => absent(
            option,
            "Infiltrate",
            "R 4.3.1",
            "Replaces 1 Police or Troops cube with a 26July Guerrilla in a space without Support.",
            free,
            second_after_operation(view),
        ),
        "march" => absent(
            option,
            "March",
            "R 3.3.2",
            "Moves 26July Guerrillas into adjacent spaces.",
            format!(
                "1 Resource per City or Province destination (0 for an economic center) · 26July has {m26}."
            ),
            second_after_operation(view),
        ),
        "march:infiltrate" => absent(
            option,
            "Infiltrate",
            "R 4.3.1",
            "Replaces 1 Police or Troops cube with a 26July Guerrilla in a space without Support.",
            free,
            second_after_operation(view),
        ),
        "attack" => absent(
            option,
            "Attack",
            "R 3.3.3",
            "Activates 26July Guerrillas in a space and rolls to remove up to 2 enemy pieces there.",
            per_space,
            second_after_operation(view),
        ),
        "attack:ambush" => absent(
            option,
            "Ambush",
            "R 4.3.2",
            "Makes one Attack space succeed without a roll, Activating only 1 Underground Guerrilla.",
            free,
            second_after_operation(view),
        ),
        _ => return None,
    })
}

/// Breaks a text into lines of at most `width` characters at spaces. A newline in
/// the text always starts a new line.
pub(super) fn lines(text: &str, width: usize) -> Vec<String> {
    text.split('\n')
        .flat_map(|part| wrap(part, width))
        .collect()
}

/// An emblem's width in characters of the text it stands in: 0.9 em high and about as
/// wide (`emblems::HEIGHT`), beside Lato's 0.43 em average character.
const EMBLEM_CHARS: usize = 2;

/// [`lines`] broken by what is drawn, for text whose first mention of a faction draws
/// its name and emblem and whose later ones draw the emblem alone (#60). `seen` holds
/// the factions already mentioned in the briefing's earlier texts.
pub(super) fn drawn_lines(text: &str, width: usize, seen: &[bool; 4]) -> Vec<String> {
    let mut seen = *seen;
    text.split('\n')
        .flat_map(|part| {
            wrap_by(part, width, |word| {
                emblems::segments_after(word, Naming::FirstMention, false, &mut seen)
                    .iter()
                    .map(|segment| match segment {
                        Segment::Words(words) => words.chars().count(),
                        Segment::Emblem(_) => EMBLEM_CHARS,
                    })
                    .sum()
            })
        })
        .collect()
}

/// Breaks a sentence into lines of at most `width` characters at spaces.
pub(super) fn wrap(sentence: &str, width: usize) -> Vec<String> {
    wrap_by(sentence, width, |word| word.chars().count())
}

/// Breaks a sentence at spaces into lines whose words, as `measure` counts each in
/// turn, and the spaces between them come to at most `width`.
fn wrap_by(sentence: &str, width: usize, mut measure: impl FnMut(&str) -> usize) -> Vec<String> {
    let mut lines = vec![];
    let mut line = String::new();
    let mut used = 0;
    for word in sentence.split_whitespace() {
        let size = measure(word);
        if !line.is_empty() && used + 1 + size > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        used += size + usize::from(!line.is_empty());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    fn text(briefing: &Briefing) -> String {
        briefing
            .entries
            .iter()
            .map(|e| format!("{}: {}", e.label, e.text))
            .chain([briefing.title.clone(), briefing.rules.clone()])
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn get(view: &Observation, option: &str, label: &str) -> String {
        content(view, option)
            .unwrap()
            .entries
            .into_iter()
            .find(|e| e.label == label)
            .unwrap_or_else(|| panic!("{option} has no {label}"))
            .text
    }

    const OPTIONS: [&str; 11] = [
        "pass",
        "event:unshaded",
        "event:shaded",
        "rally",
        "rally:infiltrate",
        "march",
        "march:infiltrate",
        "attack",
        "attack:ambush",
        "terror",
        "terror:kidnap",
    ];

    /// Every listed option has a briefing with a purpose, a cost against current
    /// Resources, Directorio's next options and a rule reference.
    #[test]
    fn every_listed_option_is_briefed() {
        let view = Game::opening().observe();
        let listed: Vec<_> = view
            .choices
            .iter()
            .map(|c| c.id.split_once(':').unwrap().1.to_owned())
            .collect();
        assert_eq!(listed, OPTIONS);
        for option in OPTIONS {
            let b = content(&view, option).unwrap();
            let labels: Vec<_> = b.entries.iter().map(|e| e.label.as_str()).collect();
            for label in ["PURPOSE", "COST", "NEXT"] {
                assert!(labels.contains(&label), "{option}: {labels:?}");
            }
            assert!(b.rule.starts_with("R "), "{option}");
            assert!(b.rules.contains(b.rule), "{option}");
            assert!(
                text(&b).contains("Directorio"),
                "{option} names Directorio's options"
            );
        }
    }

    /// R 2.3.3: +1 Resource (0 → 1), 26July stays Eligible, Directorio becomes 1st
    /// Eligible with every 1st Eligible option (R 2.3.4).
    #[test]
    fn pass_keeps_its_exact_numbers() {
        let view = Game::opening().observe();
        assert_eq!(get(&view, "pass", "COST"), "None · 26July has 0.");
        assert_eq!(
            get(&view, "pass", "RESULT"),
            "+1 Resource (0 → 1). 26July stays Eligible. Standings and Control unchanged."
        );
        assert_eq!(
            get(&view, "pass", "NEXT"),
            "Directorio becomes 1st Eligible: Pass (+1 if executed later), Operation (with or without a Special Activity) or Event. The task ends there."
        );
        // 26July's Eligibility and the next faction are the preview's, not fixed
        // sentences: changed there, the briefing follows; without a preview it
        // names no next faction.
        let mut changed = view.clone();
        let pass = changed
            .choices
            .iter_mut()
            .find(|c| c.action() == "pass")
            .unwrap();
        let preview = pass.preview.as_mut().unwrap();
        preview.m26_next_card = Some(false);
        preview.next.as_mut().unwrap().faction = Faction::Syndicate;
        assert!(
            get(&changed, "pass", "RESULT").contains("26July becomes Ineligible after this card."),
        );
        assert!(get(&changed, "pass", "NEXT").starts_with("Syndicate becomes 1st Eligible"));
        assert!(
            content(&changed, "pass")
                .unwrap()
                .rules
                .contains("Syndicate's options")
        );
        changed
            .choices
            .iter_mut()
            .find(|c| c.action() == "pass")
            .unwrap()
            .preview = None;
        let without_preview = content(&changed, "pass").unwrap();
        assert!(without_preview.entries.iter().all(|e| e.label != "NEXT"));
        assert!(!without_preview.rules.contains("Directorio"));
    }

    /// The owner's refinement: Operation briefings are general. They name no
    /// province and give no space's computed result.
    #[test]
    fn operation_briefings_name_no_place_and_no_space_result() {
        let view = Game::opening().observe();
        for option in OPTIONS.iter().filter(|o| **o != "pass") {
            let words = text(&content(&view, option).unwrap());
            for place in [
                "Santiago",
                "Oriente",
                "Sierra",
                "Havana",
                "Camagüey",
                "Las Villas",
                "Matanzas",
                "Pinar",
                "La Habana",
            ] {
                assert!(!words.contains(place), "{option} names {place}: {words}");
            }
            assert!(!words.contains("margin"), "{option}: {words}");
        }
    }

    #[test]
    fn briefings_recommend_no_move() {
        let mut game = Game::opening();
        let mut views = vec![game.observe()];
        game.submit(Faction::M26, 0, "0:terror").unwrap();
        views.push(game.observe());
        for view in &views {
            for option in OPTIONS {
                let words = text(&content(view, option).unwrap()).to_lowercase();
                for advice in [
                    "should",
                    "recommend",
                    "best",
                    "better",
                    "consider",
                    "advis",
                    "suggest",
                    "worth",
                    "you ",
                ] {
                    assert!(!words.contains(advice), "{option}: {advice} in {words}");
                }
            }
        }
    }

    fn step(game: &mut Game, action: &str) {
        let view = game.observe();
        let id = format!("{}:{action}", view.revision);
        game.submit(view.owner, view.revision, &id).unwrap();
    }

    #[test]
    fn assault_side_choices_show_the_rules_that_govern_them() {
        let mut game = crate::terror_chosen(Game::opening_with_seed(2));
        for action in [
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
        ] {
            step(&mut game, action);
        }
        let view = game.observe();
        for (action, rule, label) in [
            ("transfer", "R 1.5.1, 1.8", "GIVER'S DECISION"),
            ("remove", "R 1.4.1", "VOLUNTARY REMOVAL"),
        ] {
            let briefing = content(&view, action).unwrap();
            assert_eq!(briefing.rule, rule);
            assert_eq!(briefing.entries[0].label, label);
        }
        let finish = content(&view, "end-assault").unwrap();
        assert_eq!(finish.rule, "R 2.3.4");
        assert_eq!(finish.entries[0].label, "RESULT");
    }

    /// Cost and Kidnap's range follow current Resources, before and after a transfer,
    /// a Terror and a removal. Worked from the rules baseline: 26July has 0 and
    /// Government 6 at the start; Government gives 2 (6 → 4, 0 → 2); a Terror pays 1
    /// (2 → 1); a die 1–6 takes at most what Government has (R 4.3.3). Removing a
    /// Santiago Guerrilla leaves 1 Guerrilla against 1 Police, so Kidnap is no longer
    /// possible there (R 4.3.3: more 26July Guerrillas than Police).
    #[test]
    fn values_follow_current_resources() {
        let mut game = Game::opening();
        let view = game.observe();
        assert!(get(&view, "terror", "COST").contains("26July has 0."));
        assert_eq!(
            get(&view, "terror:kidnap", "RESULT"),
            "Proceeds 1–6 Resources while Government has 6; at least 1."
        );
        step(&mut game, "terror");
        for action in ["transfer", "ask:0:2", "authorize:2"] {
            step(&mut game, action);
        }
        let view = game.observe();
        assert_eq!(view.resources, [4, 2, 5, 15]);
        assert!(get(&view, "terror", "COST").contains("26July has 2."));
        assert_eq!(
            get(&view, "terror:kidnap", "RESULT"),
            "Proceeds 1–4 Resources while Government has 4; at least 1."
        );
        step(&mut game, "terror:oriente");
        let view = game.observe();
        assert!(get(&view, "terror", "COST").contains("26July has 1."));
        for action in ["remove", "remove:santiago:underground:1", "done-removal"] {
            step(&mut game, action);
        }
        let view = game.observe();
        let result = get(&view, "terror:kidnap", "RESULT");
        assert_eq!(
            result,
            "Unavailable now. 1 Underground Guerrilla vs 1 Police; must outnumber."
        );
        let kidnap = view
            .choices
            .iter()
            .find(|c| c.id.ends_with(":kidnap"))
            .unwrap();
        assert!(!kidnap.supported);
        assert!(
            kidnap
                .explanation
                .starts_with("1 Underground Guerrilla vs 1 Police;")
        );
    }

    #[test]
    fn value_lines_fit_the_column() {
        let mut game = Game::opening();
        let mut views = vec![game.observe()];
        game.submit(Faction::M26, 0, "0:terror").unwrap();
        views.push(game.observe());
        for view in &views {
            for option in OPTIONS {
                let b = content(view, option).unwrap();
                for e in &b.entries {
                    for line in lines(&e.text, CHARS) {
                        assert!(line.chars().count() <= CHARS, "{line}");
                    }
                }
                assert!(b.title.chars().count() <= 33, "{}", b.title);
            }
        }
    }

    /// A faction's first mention draws its name and its emblem, a later one its emblem
    /// alone (#60). Lines are broken by what is drawn, so none wraps again in its box
    /// and none stops short (#166's playtest).
    #[test]
    fn lines_are_broken_by_what_each_mention_draws() {
        let text =
            "Government has 1 Resource and cannot pay for a space. Look at the map or finish.";
        // Drawn: "Government" + space + emblem (2) + 49 characters reach 58 only without
        // "Look"; by its letters alone the line holds "Look" too.
        assert_eq!(
            drawn_lines(text, CHARS, &[false; 4]),
            [
                "Government has 1 Resource and cannot pay for a space.",
                "Look at the map or finish."
            ]
        );
        // Already mentioned, Government is an emblem: the same words fit with "Look".
        let mut seen = [false; 4];
        seen[Faction::Government.index()] = true;
        assert_eq!(
            drawn_lines(text, CHARS, &seen),
            [
                "Government has 1 Resource and cannot pay for a space. Look at the",
                "map or finish."
            ]
        );
        // A newline in the text still starts a new line.
        assert_eq!(drawn_lines("One\nTwo", CHARS, &[false; 4]), ["One", "Two"]);
    }
}
