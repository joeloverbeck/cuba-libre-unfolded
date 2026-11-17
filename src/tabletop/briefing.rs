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
use crate::{CARD_17_SHADED, Choice, Faction, Observation, Preview};

/// Value lines are broken by hand at this many characters, which the 420 px column
/// holds at 15 px with margin (Lato averages about 6.5 px a character there), so each
/// authored line is one drawn line and the tag's height follows its content.
pub(super) const CHARS: usize = 58;
/// The same for the 12.5 px rules line.
pub(super) const RULES_CHARS: usize = 70;

/// One labelled entry: PURPOSE, COST, STEPS, RESULT, STATUS or NEXT.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(super) struct Entry {
    pub label: &'static str,
    pub text: String,
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

fn entry(label: &'static str, text: impl Into<String>) -> Entry {
    Entry {
        label,
        text: text.into(),
    }
}

fn resources(n: u8) -> String {
    format!("{n} Resource{}", if n == 1 { "" } else { "s" })
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

/// What the 2nd Eligible faction may do once 26July's action is recorded (R 2.3.4).
fn second_after_operation(view: &Observation) -> String {
    format!(
        "{}, 2nd Eligible: Pass or a Limited Operation; with a Special Activity, also card 17's Event (R 2.3.4).",
        view.eligibility.second.name()
    )
}

fn second_after_event(view: &Observation) -> String {
    format!(
        "{}, 2nd Eligible: Pass, or an Operation with a Special Activity if desired (R 2.3.4).",
        view.eligibility.second.name()
    )
}

fn pass(view: &Observation, m26: u8) -> Briefing {
    let preview: Option<&Preview> = listed(view, "pass").and_then(|c| c.preview.as_ref());
    let mut result = vec![];
    let mut next = "Directorio becomes 1st Eligible. The task ends there (R 2.3.3).".to_owned();
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
        result.push(format!("{} stays Eligible.", Faction::M26.name()));
        if preview.standings.is_empty() && preview.control.is_empty() {
            result.push("Standings and Control unchanged.".into());
        }
        if let Some(n) = &preview.next {
            next = format!(
                "{} becomes {}: {}. The task ends there.",
                n.faction.name(),
                n.role,
                either(&n.options)
            );
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
    entries.push(entry("NEXT", next));
    Briefing {
        option: "pass".into(),
        title: "Pass".into(),
        rule: "R 2.3.3",
        entries,
        rules:
            "Rules: R 2.3.3, R 2.3.4. The task shows Directorio's options without executing them."
                .into(),
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
                    "1 Resource per City or Province · 26July has {m26}. While 26July executes Terror, Kidnap proceeds or a Resource transfer can add Resources."
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
                    view.eligibility.second.name()
                ),
            ),
        ],
        rules: "Rules: R 3.3.4, R 2.3.4. Going back to the action choice before the first effect is local policy.".into(),
    }
}

/// Kidnap's possible proceeds, from the rules authority's stakes when it offers them
/// (opening Kidnap and its roll); otherwise the same range read from Government's
/// current Resources (R 4.3.3: a die 1–6, at most what Government has).
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
            super::preview::proceeds(stakes),
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
    let government = view.resources[Faction::Government.index()];
    let (low, high) = (government.min(1), government.min(6));
    let range = if low == high {
        resources(low)
    } else {
        format!("{low}–{}", resources(high))
    };
    format!("Proceeds {range} while Government has {government}; at least {low}.")
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
                view.eligibility.second.name()
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

/// The briefing for one option at the action choice, or for the Terror and Kidnap
/// tabs during execution. `option` is the choice id without its revision.
pub(super) fn content(view: &Observation, option: &str) -> Option<Briefing> {
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
            "Card 17, Vet trains guerrillas: 26July or DR free Rallies in each space it has a Base (as if spaces Neutral).",
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

/// Breaks a sentence into lines of at most `width` characters at spaces.
pub(super) fn lines(text: &str, width: usize) -> Vec<String> {
    super::preview::wrap(text, width)
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
            let labels: Vec<_> = b.entries.iter().map(|e| e.label).collect();
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
            "Unavailable now. 1 Guerrillas (Active + Underground) vs 1 Police; must outnumber."
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
                .starts_with("1 Guerrillas (Active + Underground) vs 1 Police;")
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
}
