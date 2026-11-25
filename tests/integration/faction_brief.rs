//! What a faction's seat brief says (#204). Expectations are worked by hand from the March
//! 2019 Living Rules and the prepared position's inventory, never read from the brief's own
//! tables: each number's rule is beside it. That no brief holds anything hidden is
//! `faction_seats`' test, with the views.

use crate::{explore, faction_seats::later_positions};
use cuba_libre_unfolded::{
    Choice, Faction, Game, Kind, State,
    faction_seats::{Brief, Pieces},
};
use std::collections::{BTreeMap, BTreeSet};

use Faction::{Directorio, Government, M26, Syndicate};

/// Seed 5's Kidnap die is a 3.
fn played(actions: &[&str]) -> Game {
    let mut game = Game::opening_with_seed(5);
    for action in actions {
        explore::choose(&mut game, action);
    }
    game
}

fn brief(game: &Game, faction: Faction) -> Brief {
    Brief::of(&game.observe(), faction)
}

/// The prepared position, at 26July's decision on card 17 (tactical handoff, "Starting
/// position").
fn opening() -> Game {
    played(&[])
}

/// After the shaded text. Kidnap takes 3 Resources, which pay Terror in Santiago and Oriente
/// (R 4.3.3, 3.3.4). While the Event is executed the Syndicate gives Directorio 4 (R 1.5.1)
/// and Directorio removes its Guerrilla from Camagüey Province (R 1.4.1). Then every 26July
/// Guerrilla is Active (card 17, shaded).
fn after_shaded() -> Game {
    played(&[
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "terror:oriente",
        "end-terror",
        "event:shaded",
        "transfer",
        "ask:3:4",
        "authorize:4",
        "remove",
        "remove:camaguey-province:underground:1",
        "done-removal",
        "resolve-shaded",
    ])
}

/// After the unshaded text. Terror in Santiago alone; Directorio gives the Syndicate its 5
/// Resources (R 1.5.1) and grants 26July the free Rally, which places 4 Guerrillas in Sierra
/// Maestra: twice its 1 Base plus twice its 1 Population (R 3.3.1).
fn after_unshaded() -> Game {
    played(&[
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:unshaded",
        "transfer",
        "give:3:5",
        "rally:beneficiary:26july",
        "rally:sierra-maestra:place:4",
    ])
}

/// What a faction's standing must be: its total, its margin (R 7.3) and what contributes,
/// in the map's space order.
struct Expected {
    faction: Faction,
    score: i16,
    margin: i16,
    contributions: &'static [&'static str],
}

/// Total Support: 2 × the Population in Active Support + 1 × that in Passive Support
/// (R 1.6.3): 12 + 1 + 2 + 1 = 16, and 16 − 18 = −2 (R 7.3). Nothing played in these
/// positions shifts a space at Support, so it stays so after either text.
const SUPPORT: Expected = Expected {
    faction: Government,
    score: 16,
    margin: -2,
    contributions: &[
        "Havana: 6 × 2 = 12",
        "Camagüey City: 1 × 1 = 1",
        "Pinar del Río: 1 × 2 = 2",
        "La Habana: 1 × 1 = 1",
    ],
};

fn positions() -> [(&'static str, Game, [Expected; 4]); 3] {
    const CASINOS: [&str; 3] = [
        "Havana: 1 Casino = 1",
        "Pinar del Río: 1 Casino = 1",
        "La Habana: 1 Casino = 1",
    ];
    [
        (
            "opening",
            opening(),
            [
                SUPPORT,
                // Total Opposition 1 + 1 + 2 + 2 = 6 (R 1.6.3) and 1 Base: 7 − 15 = −8.
                Expected {
                    faction: M26,
                    score: 7,
                    margin: -8,
                    contributions: &[
                        "Matanzas: 1 × 1 = 1",
                        "Camagüey Province: 1 × 1 = 1",
                        "Oriente: 2 × 1 = 2",
                        "Sierra Maestra: 1 × 2 = 2",
                        "Sierra Maestra: 1 Base = 1",
                    ],
                },
                // It alone is in Camagüey Province, Population 1; in Havana its 2 do not
                // exceed the Government's 10 and 1 open Casino (R 1.7). No Base: 1 − 9 = −8.
                Expected {
                    faction: Directorio,
                    score: 1,
                    margin: -8,
                    contributions: &["Camagüey Province: 1 × 1 = 1"],
                },
                // 3 open Casinos and 15 Resources: the lower of 3 − 7 and 15 − 30 (R 7.3).
                Expected {
                    faction: Syndicate,
                    score: 3,
                    margin: -15,
                    contributions: &[CASINOS[0], CASINOS[1], CASINOS[2], "Resources: 15"],
                },
            ],
        ),
        (
            "after the shaded text",
            after_shaded(),
            [
                SUPPORT,
                // Terror moved Santiago from Neutral to Passive and Oriente from Passive to
                // Active Opposition (R 3.3.4): 1 + 1 + 1 + 4 + 2 = 9 and 1 Base: 10 − 15.
                Expected {
                    faction: M26,
                    score: 10,
                    margin: -5,
                    contributions: &[
                        "Santiago de Cuba: 1 × 1 = 1",
                        "Matanzas: 1 × 1 = 1",
                        "Camagüey Province: 1 × 1 = 1",
                        "Oriente: 2 × 2 = 4",
                        "Sierra Maestra: 1 × 2 = 2",
                        "Sierra Maestra: 1 Base = 1",
                    ],
                },
                // Camagüey Province holds no piece, so nobody Controls it (R 1.7): 0 − 9.
                Expected {
                    faction: Directorio,
                    score: 0,
                    margin: -9,
                    contributions: &[],
                },
                // It gave 4 of its 15: the lower of 3 − 7 and 11 − 30.
                Expected {
                    faction: Syndicate,
                    score: 3,
                    margin: -19,
                    contributions: &[CASINOS[0], CASINOS[1], CASINOS[2], "Resources: 11"],
                },
            ],
        ),
        (
            "after the unshaded text",
            after_unshaded(),
            [
                SUPPORT,
                // Santiago at Passive Opposition: 1 + 1 + 1 + 2 + 2 = 7 and 1 Base: 8 − 15.
                // The placed Guerrillas add nothing (R 1.6.3).
                Expected {
                    faction: M26,
                    score: 8,
                    margin: -7,
                    contributions: &[
                        "Santiago de Cuba: 1 × 1 = 1",
                        "Matanzas: 1 × 1 = 1",
                        "Camagüey Province: 1 × 1 = 1",
                        "Oriente: 2 × 1 = 2",
                        "Sierra Maestra: 1 × 2 = 2",
                        "Sierra Maestra: 1 Base = 1",
                    ],
                },
                Expected {
                    faction: Directorio,
                    score: 1,
                    margin: -8,
                    contributions: &["Camagüey Province: 1 × 1 = 1"],
                },
                // It was given 5: the lower of 3 − 7 and 20 − 30.
                Expected {
                    faction: Syndicate,
                    score: 3,
                    margin: -10,
                    contributions: &[CASINOS[0], CASINOS[1], CASINOS[2], "Resources: 20"],
                },
            ],
        ),
    ]
}

/// All four briefs of each position give every faction's standing as worked from the rules:
/// the reader's own as its victory, the other three as its rivals'.
#[test]
fn every_standing_and_margin_is_worked_from_the_rules() {
    for (position, game, expected) in positions() {
        for faction in Faction::ALL {
            let brief = brief(&game, faction);
            assert_eq!(brief.faction, faction);
            assert_eq!(brief.revision, game.observe().revision, "{position}");
            let own = std::iter::once(&brief.victory);
            let told: Vec<_> = own.chain(&brief.rivals).collect();
            assert_eq!(told[0].standing.faction, faction);
            assert_eq!(told.len(), 4);
            for expected in &expected {
                let at = format!("{position}: {faction:?} is told of {:?}", expected.faction);
                let said = told
                    .iter()
                    .find(|said| said.standing.faction == expected.faction)
                    .expect(&at);
                assert_eq!(said.name, expected.faction.name(), "{at}");
                assert_eq!(said.standing.score, expected.score, "{at}");
                assert_eq!(said.standing.margin, expected.margin, "{at}");
                assert_eq!(said.standing.contributions, expected.contributions, "{at}");
            }
            // What the Government's margin leaves out of its condition: of the three Cities
            // only Havana is at Active Support (R 7.2).
            let government = told
                .iter()
                .find(|said| said.standing.faction == Government)
                .unwrap();
            assert_eq!(
                government.standing.extra,
                "Active Support cities 1/3 (not met)"
            );
        }
    }
}

/// A rival is told of in the words of its own brief's condition and standing, and nothing
/// else of that brief is in another's, so no brief holds text meant for another faction.
#[test]
fn a_rival_is_told_of_in_the_words_of_its_own_brief() {
    for (_, game, _) in positions() {
        let own = Faction::ALL.map(|faction| brief(&game, faction));
        for brief in &own {
            let rivals: BTreeSet<_> = brief
                .rivals
                .iter()
                .map(|rival| rival.standing.faction.index())
                .collect();
            assert_eq!(rivals.len(), 3);
            assert!(!rivals.contains(&brief.faction.index()));
            for rival in &brief.rivals {
                assert_eq!(*rival, own[rival.standing.faction.index()].victory);
            }
            assert!(brief.who.starts_with("You play "), "{}", brief.who);
            let whole = format!("{brief:?}");
            for other in own.iter().filter(|other| other.faction != brief.faction) {
                assert!(!whole.contains(other.who));
            }
        }
    }
}

type Piece<'a> = (Option<&'a str>, Kind, Option<State>, u8);

fn told(pieces: &[Pieces]) -> Vec<Piece<'_>> {
    pieces
        .iter()
        .map(|p| (p.space.as_deref(), p.kind, p.state, p.count))
        .collect()
}

/// A faction's own pieces by space, and its Available Forces: the chart of p. 20 less what
/// is on the map (R 1.4.1).
#[test]
fn its_pieces_and_available_forces_are_those_of_the_position() {
    use Kind::{Base, Casino, Guerrilla, Police, Troops};
    use State::{Active, Open, Ordinary, Underground};
    let on = |space, kind, state, count| (Some(space), kind, Some(state), count);
    let left = |kind, count| (None, kind, None, count);

    let opening = opening();
    let government = brief(&opening, Government);
    assert_eq!(
        told(&government.pieces),
        [
            on("Havana", Troops, Ordinary, 6),
            on("Havana", Police, Ordinary, 4),
            on("Camagüey City", Troops, Ordinary, 1),
            on("Camagüey City", Police, Ordinary, 2),
            on("Santiago de Cuba", Troops, Ordinary, 2),
            on("Santiago de Cuba", Police, Ordinary, 1),
            on("Las Villas", Troops, Ordinary, 3),
        ]
    );
    // 15 − 12 Troops, 15 − 7 Police, 2 Bases.
    assert_eq!(
        told(&government.available),
        [left(Troops, 3), left(Police, 8), left(Base, 2)]
    );
    let syndicate = brief(&opening, Syndicate);
    assert_eq!(
        told(&syndicate.pieces),
        [
            on("Havana", Casino, Open, 1),
            on("Pinar del Río", Casino, Open, 1),
            on("La Habana", Casino, Open, 1),
        ]
    );
    // 6 Guerrillas; 10 − 3 Casinos.
    assert_eq!(
        told(&syndicate.available),
        [left(Guerrilla, 6), left(Casino, 7)]
    );

    // All 26July Guerrillas are Active; 15 − 3 Guerrillas, 4 − 1 Bases.
    let shaded = after_shaded();
    let m26 = brief(&shaded, M26);
    assert_eq!(
        told(&m26.pieces),
        [
            on("Santiago de Cuba", Guerrilla, Active, 2),
            on("Oriente", Guerrilla, Active, 1),
            on("Sierra Maestra", Base, Ordinary, 1),
        ]
    );
    assert_eq!(told(&m26.available), [left(Guerrilla, 12), left(Base, 3)]);
    // Its removed Guerrilla went to Available: 15 − 2.
    let directorio = brief(&shaded, Directorio);
    assert_eq!(
        told(&directorio.pieces),
        [on("Havana", Guerrilla, Underground, 2)]
    );
    assert_eq!(
        told(&directorio.available),
        [left(Guerrilla, 13), left(Base, 4)]
    );

    // Santiago's Terror Activated one of two; the Rally placed four: 15 − 7.
    let m26 = brief(&after_unshaded(), M26);
    let mut pieces = told(&m26.pieces);
    pieces.sort_by_key(|piece| format!("{piece:?}"));
    assert_eq!(
        pieces,
        [
            on("Oriente", Guerrilla, Underground, 1),
            on("Santiago de Cuba", Guerrilla, Active, 1),
            on("Santiago de Cuba", Guerrilla, Underground, 1),
            on("Sierra Maestra", Base, Ordinary, 1),
            on("Sierra Maestra", Guerrilla, Underground, 4),
        ]
    );
    assert_eq!(told(&m26.available), [left(Guerrilla, 8), left(Base, 3)]);
}

const NOT: &str = "not in this build";
const FREE: &str =
    "on card 17, as the free Rally its unshaded Event grants where the faction has a Base";

/// A name and its section, with where this build plays it.
type Operation = (&'static str, &'static str, &'static str);
/// A name, its section and what it may accompany, with where this build plays it.
type Activity = (&'static str, &'static str, &'static str, &'static str);

/// The Operations of R 3.2 and 3.3 and the Special Activities of R 4.2 to 4.5, faction by
/// faction.
const RULES: [(Faction, [Operation; 4], [Activity; 3]); 4] = [
    (
        Government,
        [
            ("Train", "R 3.2.1", NOT),
            ("Garrison", "R 3.2.2", NOT),
            ("Sweep", "R 3.2.3", NOT),
            ("Assault", "R 3.2.4", "on card 18"),
        ],
        [
            ("Transport", "R 4.2.1", "any Operation (R 4.1.1)", NOT),
            ("Air Strike", "R 4.2.2", "Garrison, Sweep or Assault", NOT),
            ("Reprisal", "R 4.2.3", "Garrison, Sweep or Assault", NOT),
        ],
    ),
    (
        M26,
        [
            ("Rally", "R 3.3.1", FREE),
            ("March", "R 3.3.2", NOT),
            ("Attack", "R 3.3.3", NOT),
            ("Terror", "R 3.3.4", "on card 17"),
        ],
        [
            ("Infiltrate", "R 4.3.1", "Rally or March", NOT),
            ("Ambush", "R 4.3.2", "Attack", NOT),
            ("Kidnap", "R 4.3.3", "Terror", "on card 17"),
        ],
    ),
    (
        Directorio,
        [
            ("Rally", "R 3.3.1", FREE),
            ("March", "R 3.3.2", NOT),
            ("Attack", "R 3.3.3", NOT),
            ("Terror", "R 3.3.4", NOT),
        ],
        [
            ("Subvert", "R 4.4.1", "Rally or March", NOT),
            ("Ambush", "R 4.4.2", "Attack", NOT),
            ("Assassinate", "R 4.4.3", "Terror", NOT),
        ],
    ),
    (
        Syndicate,
        [
            ("Rally", "R 3.3.1", NOT),
            ("March", "R 3.3.2", NOT),
            ("Terror", "R 3.3.4", NOT),
            ("Construct", "R 3.3.5", NOT),
        ],
        [
            ("Profit", "R 4.5.1", "Rally, March or Construct", NOT),
            ("Muscle", "R 4.5.3", "Rally, March or Construct", NOT),
            ("Bribe", "R 4.5.4", "any Operation (R 4.1.1)", NOT),
        ],
    ),
];

#[test]
fn each_faction_has_the_operations_and_special_activities_of_the_rules() {
    for (faction, operations, activities) in RULES {
        let brief = brief(&opening(), faction);
        let told: Vec<_> = brief
            .operations
            .iter()
            .map(|o| (o.name, o.rule, o.played.as_str()))
            .collect();
        assert_eq!(told, operations, "{faction:?}");
        assert!(brief.operations.iter().all(|o| o.accompanies.is_none()));
        let told: Vec<_> = brief
            .special_activities
            .iter()
            .map(|a| (a.name, a.rule, a.accompanies.unwrap(), a.played.as_str()))
            .collect();
        assert_eq!(told, activities, "{faction:?}");
        for action in brief.operations.iter().chain(&brief.special_activities) {
            assert!(action.purpose.ends_with('.'), "{}", action.purpose);
        }
        assert!(brief.rules.edition.contains("March 2019"));
        assert!(
            brief
                .rules
                .read_at
                .ends_with("CL-RULES-2019-LivingRules.pdf")
        );
    }
}

/// The thresholds of R 7.2 and 7.3, and where the rules of one Operation differ by faction
/// (R 3.3.1, 3.3.4). The accepted reading of 26July's Terror is the tactical handoff's
/// "Direction reconciliation": toward Active Opposition only. The rest of the wording is
/// checked against the rules by its reviewer.
#[test]
fn what_differs_by_faction_in_the_rules_differs_in_the_briefs() {
    let cases = [
        (
            Government,
            "exceeds 18 (R 7.2)",
            "Total Support − 18 (R 7.3)",
            &[][..],
        ),
        (
            M26,
            "exceeds 15 (R 7.2)",
            "Bases − 15 (R 7.3)",
            &[
                ("Rally", "at Neutral or Opposition"),
                ("Rally", "replaces 2 with a Base"),
                ("Terror", "1 level toward Active Opposition"),
            ],
        ),
        (
            Directorio,
            "exceeds 9 (R 7.2)",
            "Bases − 9 (R 7.3)",
            &[
                ("Rally", "at Neutral, Passive Support or Passive Opposition"),
                ("Rally", "replaces 2 with a Base"),
                ("Terror", "1 level toward Neutral"),
            ],
        ),
        (
            Syndicate,
            "more than 7 open Casinos and Resources above 30 (R 7.2)",
            "open Casinos − 7 and Syndicate Resources − 30 (R 7.3)",
            &[
                ("Rally", "where it has an open Casino"),
                ("Terror", "1 level toward Neutral"),
            ],
        ),
    ];
    for (faction, condition, margin, wording) in cases {
        let brief = brief(&opening(), faction);
        let victory = &brief.victory;
        assert!(victory.condition.contains(condition), "{victory:?}");
        assert!(victory.margin_rule.contains(margin), "{victory:?}");
        for (name, said) in wording {
            let operation = brief.operations.iter().find(|o| o.name == *name).unwrap();
            assert!(operation.purpose.contains(said), "{operation:?}");
        }
    }
}

/// The Operation or Special Activity `choice` takes, if it takes one. Every action the
/// positions offer is named, so a new one fails here until it is.
fn taken(choice: &Choice) -> Option<(Faction, &'static str)> {
    match choice.action() {
        "rally:beneficiary:26july" => Some((M26, "Rally")),
        "rally:beneficiary:directorio" => Some((Directorio, "Rally")),
        "government:assault" => Some((choice.owner, "Assault")),
        "government:pass" | "kidnap:cancel" => None,
        action => match action.split(':').next().unwrap() {
            "terror" => Some((choice.owner, "Terror")),
            "kidnap" => Some((choice.owner, "Kidnap")),
            "rally" => Some((choice.owner, "Rally")),
            "assault" => Some((choice.owner, "Assault")),
            "pass" | "event" | "resolve-shaded" | "transfer" | "give" | "ask" | "authorize"
            | "decline" | "wait" | "reopen" | "back" | "back-to-choice" | "remove"
            | "done-removal" | "end-terror" | "end-assault" | "advance-card" => None,
            _ => panic!("{action}"),
        },
    }
}

/// A brief's marks are what the rules authority offers in play: an Operation or Special
/// Activity is played on a card exactly when some reachable position of it offers a choice
/// that takes it and can be chosen.
#[test]
fn what_a_brief_says_this_build_plays_is_what_the_authority_offers() {
    let explored = explore::all_explored().flat_map(|explored| &explored.nodes);
    let later = later_positions();
    let mut found: BTreeMap<(usize, &str), BTreeSet<u8>> = BTreeMap::new();
    for game in explored.map(|node| &node.game).chain(&later) {
        let seen = game.observe();
        for choice in seen.choices.iter().filter(|choice| choice.supported) {
            if let Some((faction, name)) = taken(choice) {
                let cards = found.entry((faction.index(), name)).or_default();
                cards.insert(seen.current_card);
            }
        }
    }
    let mut marked = BTreeMap::new();
    for faction in Faction::ALL {
        let brief = brief(&opening(), faction);
        for action in brief.operations.iter().chain(&brief.special_activities) {
            let cards: BTreeSet<u8> = [17, 18]
                .into_iter()
                .filter(|card| action.played.starts_with(&format!("on card {card}")))
                .collect();
            if !cards.is_empty() {
                marked.insert((faction.index(), action.name), cards);
            }
        }
    }
    assert_eq!(marked, found);
}

/// Each Operation and Special Activity the authority's own menus list is in the brief, by
/// the section the menu cites, and the brief lists no other. Directorio's menu names none.
#[test]
fn the_brief_and_the_authoritys_menus_name_the_same_actions() {
    const CARD_18: [&str; 9] = [
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
        "event:shaded",
        "resolve-shaded",
        "advance-card",
        "government:pass",
    ];
    for (faction, game) in [
        (M26, opening()),
        (Government, played(&CARD_18[..8])),
        (Syndicate, played(&CARD_18)),
    ] {
        let seen = game.observe();
        assert_eq!(seen.owner, faction);
        let mut listed: BTreeMap<String, String> = BTreeMap::new();
        for choice in &seen.choices {
            let slug = choice.action().rsplit(':').next().unwrap();
            if !matches!(slug, "pass" | "event" | "unshaded" | "shaded") {
                let cites = listed.entry(slug.to_owned()).or_default();
                cites.push_str(&choice.explanation);
            }
        }
        let brief = brief(&game, faction);
        for action in brief.operations.iter().chain(&brief.special_activities) {
            let slug = action.name.to_lowercase().replace(' ', "-");
            let cites = listed.remove(&slug).unwrap_or_else(|| panic!("{slug}"));
            let section = action.rule.trim_start_matches("R ");
            // Assault's opening cites the rules of its cost, not its own section.
            assert!(
                cites.contains(section) || action.name == "Assault",
                "{slug}: {cites}"
            );
        }
        assert!(listed.is_empty(), "{faction:?}: {listed:?}");
    }
}
