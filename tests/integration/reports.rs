//! Seam 1 of #197: the action reports (#207). Each expected report is worked by hand from
//! the March 2019 Living Rules and the tactical handoff's prepared position, with the rule
//! beside each number, and never from the app's own output.
//!
//! The prepared position: Resources Government 6, 26July 0, Directorio 5, Syndicate 15.
//! R 1.6.3 and 7.3 count Total Support 16 (Havana 6 × 2, Camagüey City 1, Pinar del Río
//! 1 × 2, La Habana 1), margin 16 − 18 = −2; Opposition + Bases 7 (Matanzas 1, Camagüey
//! Province 1, Oriente 2 × 1, Sierra Maestra 1 × 2, and 1 Base), margin 7 − 15 = −8;
//! Controlled Population + Bases 1 (Camagüey Province, where Directorio's Guerrilla is
//! alone, R 1.7), margin 1 − 9 = −8; 3 open Casinos, the Syndicate's margin being the lower
//! of 3 − 7 = −4 and 15 − 30 = −15. Available (R 1.4.1 and the chart, p. 20): Troops 15 − 12
//! = 3, Police 15 − 7 = 8, 26July Guerrillas 15 − 3 = 12, Directorio Guerrillas 15 − 3 = 12.
//! Seed 5's Kidnap die is a 3.
//!
//! The `reader` at the end reads a report's every sentence back, replays it from the
//! position before and compares the result with the position after, so it finds a number
//! or place that is not the committed result's.

use crate::{
    SEED,
    explore::{choose, drained, parallel},
    longest::{self, CARD_18, HANDOFF},
};
use cuba_libre_unfolded::{
    ActionChoice, Choice, EventOutcome, Faction, Game, Kind, Observation, Space, State, Terrain,
    Transfer, TransferChoice,
    reports::{Action, Article, FromTo, Report, Reports},
    save::Session,
};
use std::collections::BTreeMap;

/// A match played decision by decision. Its revisions are its decisions' count.
#[derive(Clone)]
struct Match {
    now: Game,
    /// What was seen before each decision, and now.
    seen: Vec<Observation>,
    /// Each decision: the faction that made it and its action.
    decisions: Vec<(Faction, String)>,
    reports: Reports,
}

impl Match {
    fn opening(seed: u64) -> Self {
        Self::at(Game::opening_with_seed(seed))
    }
    fn at(game: Game) -> Self {
        Self {
            reports: Reports::new(&game.observe()),
            seen: vec![game.observe()],
            now: game,
            decisions: vec![],
        }
    }
    fn played(seed: u64, actions: &[&str]) -> Self {
        let mut played = Self::opening(seed);
        played.play(actions);
        played
    }
    fn now(&self) -> &Observation {
        self.seen.last().unwrap()
    }
    fn revision(&self) -> u64 {
        self.now().revision
    }
    fn play(&mut self, actions: &[&str]) {
        for action in actions {
            let owner = self
                .now()
                .choices
                .iter()
                .find(|c| c.action() == *action)
                .unwrap_or_else(|| panic!("{action} is offered"))
                .owner;
            choose(&mut self.now, action);
            let seen = self.now.observe();
            self.reports.record(&seen);
            self.seen.push(seen);
            self.decisions.push((owner, (*action).to_owned()));
        }
    }
    /// The last report of what followed `revision`.
    fn last_since(&self, revision: u64) -> Report {
        self.reports.since(revision).pop().expect("A report")
    }
    /// The report of the last action, whole.
    fn last(&self) -> Report {
        self.reports.all().pop().expect("A report")
    }
}

fn at_card_18() -> Match {
    let mut played = Match::played(5, &HANDOFF);
    played.play(&CARD_18);
    played
}

/// Everything a report says, as text.
fn text(report: &Report) -> String {
    let mut lines = vec![
        format!(
            "[{:?} {:?}, card {} {}, {}, recorded {}, {}{}]",
            report.faction,
            report.action,
            report.card,
            report.card_title,
            report.role,
            report.recorded.unwrap_or("nothing"),
            if report.continues { "continues, " } else { "" },
            if report.complete {
                "complete"
            } else {
                "partway"
            },
        ),
        format!("[{}]", report.spaces.join("; ")),
        report.headline.clone(),
    ];
    lines.extend(report.summary.clone());
    let articles = |of: &[Article]| -> Vec<String> {
        of.iter()
            .map(|article| {
                let die = article.die.map_or(String::new(), |die| format!(" [{die}]"));
                format!("{}{die}: {}", article.title, article.lines.join(" "))
            })
            .collect()
    };
    lines.extend(articles(&report.articles));
    let change = |of: FromTo| {
        if of.from == of.to {
            of.to.to_string()
        } else {
            format!("{} → {}", of.from, of.to)
        }
    };
    lines.extend(report.standings.iter().map(|row| {
        format!(
            "| {:?} | {} | {} | {} |",
            row.faction,
            change(row.resources),
            change(row.score),
            change(row.margin)
        )
    }));
    lines.extend(articles(&report.notes));
    lines.join("\n")
}

/// `report` says exactly `expected`, worked by hand, and the reader agrees.
fn assert_report(played: &Match, report: &Report, expected: &str) {
    let expected: Vec<&str> = expected.trim().lines().map(str::trim).collect();
    assert_eq!(text(report), expected.join("\n"));
    reader::check(played, report).unwrap();
}

// --- A fixture of each kind --------------------------------------------------------

/// R 2.3.3: +1 Resource, 26July remains Eligible for the next card, and the next leftmost
/// Eligible faction, Directorio on card 17, is the new 1st Eligible.
#[test]
fn a_pass_of_26july() {
    let played = Match::played(5, &["pass"]);
    assert_report(
        &played,
        &played.last(),
        "[M26 Pass, card 17 Alberto Bayo, 1st Eligible, recorded Pass, complete]
        []
        26July Passes
        26July receives 1 Resource, 0 → 1.
        | Government | 6 | 16 | -2 |
        | M26 | 0 → 1 | 7 | -8 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Eligibility: 26July is in the Pass box. 26July is Eligible for the next card. Next: Directorio, 1st Eligible.",
    );
}

/// Government's Pass takes 3 (R 2.3.3). After the handoff Government has 6 − 3 = 3 (R 4.3.3)
/// and 26July 3 − 1 = 2 (R 3.3.4); Santiago's shift to Passive Opposition adds Population 1
/// to 26July's 7. Card 18 prints the same order, and 26July and Directorio are Ineligible
/// (R 2.3.7), so Government is 1st Eligible and Syndicate follows it.
#[test]
fn a_pass_of_government() {
    let mut played = at_card_18();
    played.play(&["government:pass"]);
    assert_report(
        &played,
        &played.last(),
        "[Government Pass, card 18 Pact of Caracas, 1st Eligible, recorded Pass, complete]
        []
        Government Passes
        Government receives 3 Resources, 3 → 6.
        | Government | 3 → 6 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Eligibility: Government is in the Pass box. Next: Syndicate, 1st Eligible.",
    );
}

/// Terror without Kidnap, paid by a Resource transfer. 26July asks for 3 and Syndicate, the
/// giver, moves 2 (R 1.5.1): 15 − 2 = 13, so its margin is 13 − 30 = −17. Each space pays 1,
/// flips 1 Underground Guerrilla and takes 1 marker of 20 (R 3.3.4). Oriente shifts −1 → −2
/// (Population 2: +2) and Santiago 0 → −1 (Population 1: +1), so 7 + 3 = 10 and the margin
/// is −5. Active Guerrillas still count for Control (R 1.7). Without a Special Activity it
/// is Operation Only (R 2.3.4), and 26July is Ineligible after the card (R 2.3.7).
#[test]
fn terror_without_kidnap() {
    let played = Match::played(
        5,
        &[
            "terror",
            "transfer",
            "ask:3:3",
            "authorize:2",
            "terror:oriente",
            "terror:santiago",
            "end-terror",
        ],
    );
    assert_report(
        &played,
        &played.last(),
        "[M26 Terror, card 17 Alberto Bayo, 1st Eligible, recorded Operation Only, complete]
        [Oriente; Santiago de Cuba]
        26July executes Terror in Oriente and Santiago de Cuba
        Syndicate gives 26July 2 Resources: 26July asked for 3 Resources. Syndicate 15 → 13, 26July 0 → 2.
        Terror in Oriente: 26July pays 1 Resource, 2 → 1. 1 Guerrilla turns Underground → Active. 1 Terror marker is placed. Passive Opposition → Active Opposition.
        Terror in Santiago de Cuba: 26July pays 1 Resource, 1 → 0. 1 Guerrilla turns Underground → Active. 1 Terror marker is placed. Neutral → Passive Opposition.
        | Government | 6 | 16 | -2 |
        | M26 | 0 | 7 → 10 | -8 → -5 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 → 13 | 3 | -15 → -17 |
        Markers: Terror and Sabotage markers available: 20 → 18 of 20.
        Victory totals: 26July Total Opposition + Bases: 7 → 10.
        Eligibility: 26July is in the Operation Only box. 26July is Ineligible for the next card. Next: Directorio, 2nd Eligible.",
    );
}

/// Terror with Kidnap, a Resource transfer and a voluntary removal. The die is 3, and 3 of
/// Government's 6 move (R 4.3.3); Terror pays 3 → 2 → 1; 26July gives its last Resource
/// (R 1.5.1). The Active Guerrilla of Oriente returns, 12 + 1 = 13 Available (R 1.4.1), and
/// Oriente, which it alone controlled, is uncontrolled (R 1.7). With a Special Activity the
/// 2nd Eligible may execute the Event (R 2.3.4).
#[test]
fn terror_with_kidnap_a_transfer_and_a_removal() {
    let played = Match::played(
        5,
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:oriente",
            "terror:santiago",
            "transfer",
            "give:2:1",
            "remove",
            "remove:oriente:active:1",
            "done-removal",
            "end-terror",
        ],
    );
    let report = played.last();
    assert_report(
        &played,
        &report,
        "[M26 Terror, card 17 Alberto Bayo, 1st Eligible, recorded Operation with Special Activity, complete]
        [Santiago de Cuba; Oriente]
        26July executes Terror in Oriente and Santiago de Cuba
        Kidnap in Santiago de Cuba: the die shows 3, and 26July takes 3 Resources from Government.
        Kidnap in Santiago de Cuba [3]: The die shows 3. 26July takes 3 Resources from Government: Government 6 → 3, 26July 0 → 3.
        Terror in Oriente: 26July pays 1 Resource, 3 → 2. 1 Guerrilla turns Underground → Active. 1 Terror marker is placed. Passive Opposition → Active Opposition.
        Terror in Santiago de Cuba: 26July pays 1 Resource, 2 → 1. 1 Guerrilla turns Underground → Active. 1 Terror marker is placed. Neutral → Passive Opposition.
        26July gives Directorio 1 Resource: 26July 1 → 0, Directorio 5 → 6.
        26July removes its own pieces: Oriente: 1 Active Guerrilla.
        | Government | 6 → 3 | 16 | -2 |
        | M26 | 0 | 7 → 10 | -8 → -5 |
        | Directorio | 5 → 6 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Control: Oriente: 26July → none.
        Available Forces: 26July Guerrillas: 12 → 13.
        Markers: Terror and Sabotage markers available: 20 → 18 of 20.
        Victory totals: 26July Total Opposition + Bases: 7 → 10.
        Eligibility: 26July is in the Operation with Special Activity box. 26July is Ineligible for the next card. Next: Directorio, 2nd Eligible.",
    );
}

/// A Kidnap that finds Government with nothing takes nothing (R 4.3.3). Government gave
/// 26July its 6 and 26July gave Directorio 6, then Syndicate gives 1 so that Santiago's
/// Terror can be paid: 1 − 1 = 0.
#[test]
fn a_kidnap_that_takes_nothing() {
    let mut played = Match::at(drained());
    played.play(&[
        "transfer",
        "ask:3:1",
        "authorize:1",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "end-terror",
    ]);
    let report = played.reports.all().remove(0);
    let die = played.now().kidnap.as_ref().unwrap().die;
    assert_eq!(
        report.summary.as_deref(),
        Some(
            format!(
                "Kidnap in Santiago de Cuba: the die shows {die}, and Government has no Resources to take."
            )
            .as_str()
        )
    );
    let kidnap = &report.articles[1];
    assert_eq!(kidnap.title, "Kidnap in Santiago de Cuba");
    assert_eq!(kidnap.die, Some(die));
    assert_eq!(
        kidnap.lines,
        [
            format!("The die shows {die}."),
            "Government has no Resources to take.".to_owned()
        ]
    );
    let resources: Vec<_> = report.standings.iter().map(|r| r.resources).collect();
    assert_eq!(
        resources.iter().map(|r| (r.from, r.to)).collect::<Vec<_>>(),
        [(0, 0), (0, 0), (11, 11), (15, 14)]
    );
}

/// Once a targeted faction runs out, no more is transferred (R 4.3.3): Government, which
/// gave 26July 5 of its 6, has 1 for a die of 3, so 1 moves and not 3.
#[test]
fn a_kidnap_that_takes_less_than_the_die() {
    let played = Match::played(
        5,
        &[
            "terror",
            "transfer",
            "ask:0:5",
            "authorize:5",
            "kidnap",
            "kidnap:roll",
        ],
    );
    assert_report(
        &played,
        &played.last(),
        "[M26 Terror, card 17 Alberto Bayo, 1st Eligible, recorded nothing, partway]
        [Santiago de Cuba]
        26July executes Terror
        Kidnap in Santiago de Cuba: the die shows 3, and 26July takes 1 Resource from Government.
        Government gives 26July 5 Resources: 26July asked for 5 Resources. Government 6 → 1, 26July 0 → 5.
        Kidnap in Santiago de Cuba [3]: The die shows 3. 26July takes 1 Resource from Government: Government 1 → 0, 26July 5 → 6.
        | Government | 6 → 0 | 16 | -2 |
        | M26 | 0 → 6 | 7 | -8 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |",
    );
}

/// Zero-space Operation Only (accepted local policy; R 3.1 sets no minimum): nothing is
/// paid and no income is taken.
#[test]
fn terror_in_no_space() {
    let played = Match::played(5, &["terror", "end-terror"]);
    assert_report(
        &played,
        &played.last(),
        "[M26 Terror, card 17 Alberto Bayo, 1st Eligible, recorded Operation Only, complete]
        []
        26July executes Terror in no space
        | Government | 6 | 16 | -2 |
        | M26 | 0 | 7 | -8 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Eligibility: 26July is in the Operation Only box. 26July is Ineligible for the next card. Next: Directorio, 2nd Eligible.",
    );
}

/// The shaded text: all 26July Guerrillas Active. After the handoff one of Santiago's two
/// and Oriente's one are Underground. Directorio first removes its Guerrilla of Camagüey
/// Province, 12 + 1 = 13 Available (R 1.4.1), so it controls nothing: 1 → 0, margin −9
/// (R 1.7, 7.3). 26July executed an Operation and Directorio the Event, so both are
/// Ineligible on card 18 and the others Eligible (R 2.3.7); the text keeps 26July so.
#[test]
fn the_shaded_event_with_a_removal() {
    let mut played = Match::played(5, &HANDOFF);
    played.play(&[
        "event:shaded",
        "remove",
        "remove:camaguey-province:underground:1",
        "done-removal",
        "resolve-shaded",
    ]);
    let expected = "[Directorio EventShaded, card 17 Alberto Bayo, 2nd Eligible, recorded nothing, complete]
        [Camagüey Province; Santiago de Cuba; Oriente]
        Directorio executes the Event, shaded text
        Directorio removes its own pieces: Camagüey Province: 1 Underground Guerrilla.
        26July's Guerrillas: 1 Guerrilla turns Underground → Active in Santiago de Cuba. 1 Guerrilla turns Underground → Active in Oriente.
        | Government | 3 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 → 0 | -8 → -9 |
        | Syndicate | 15 | 3 | -15 |
        Control: Camagüey Province: Directorio → none.
        Available Forces: Directorio Guerrillas: 12 → 13.
        Victory totals: Directorio Controlled Population + Bases: 1 → 0.
        Eligibility: Card 18: Government and Syndicate Eligible, 26July and Directorio Ineligible. The Event keeps 26July Ineligible through card 18.";
    assert_report(&played, &played.last(), expected);
    // Turning the card is in no action.
    played.play(&["advance-card"]);
    assert_report(&played, &played.last(), expected);
    assert_eq!(played.reports.since(played.revision()), []);
}

/// The unshaded text: 26July Rallies in each space where it has a Base, Sierra Maestra.
/// It places up to 2 × 1 Base + 2 × Population 1 = 4 (R 3.3.1), at no cost (R 5.5), and
/// 26July's own removal inside the Event is the Event's: 12 + 1 − 4 = 9 Available. Santiago
/// stays Government's, 3 cubes to 1 (R 1.7). Opposition is as it was.
#[test]
fn the_unshaded_event_with_26julys_rally() {
    let mut played = Match::played(5, &HANDOFF);
    played.play(&[
        "event:unshaded",
        "rally:beneficiary:26july",
        "remove",
        "remove:santiago:active:1",
        "done-removal",
        "rally:sierra-maestra:place:4",
    ]);
    assert_report(
        &played,
        &played.last(),
        "[Directorio EventUnshaded, card 17 Alberto Bayo, 2nd Eligible, recorded nothing, complete]
        [Santiago de Cuba; Sierra Maestra]
        Directorio executes the Event, unshaded text
        Directorio grants the free Rally to 26July.
        26July removes its own pieces: Santiago de Cuba: 1 Active Guerrilla.
        26July Rallies in Sierra Maestra: 4 Underground Guerrillas are placed from Available Forces.
        | Government | 3 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Available Forces: 26July Guerrillas: 12 → 9.
        Eligibility: Card 18: Government and Syndicate Eligible, 26July and Directorio Ineligible.",
    );
}

/// The unshaded text for Directorio, which has no Base: the part that cannot be carried out
/// is not (R 5.1.3). And a Rally that places 0 of up to 4.
#[test]
fn the_unshaded_event_with_no_rally_and_with_none_placed() {
    let standings = "| Government | 3 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Eligibility: Card 18: Government and Syndicate Eligible, 26July and Directorio Ineligible.";
    let mut played = Match::played(5, &HANDOFF);
    played.play(&["event:unshaded", "rally:beneficiary:directorio"]);
    assert_report(
        &played,
        &played.last(),
        &format!(
            "[Directorio EventUnshaded, card 17 Alberto Bayo, 2nd Eligible, recorded nothing, complete]
            []
            Directorio executes the Event, unshaded text
            Directorio grants the free Rally to Directorio.
            The free Rally: Directorio Rallies in no space.
            {standings}"
        ),
    );
    let mut played = Match::played(5, &HANDOFF);
    played.play(&[
        "event:unshaded",
        "rally:beneficiary:26july",
        "rally:sierra-maestra:place:0",
    ]);
    assert_report(
        &played,
        &played.last(),
        &format!(
            "[Directorio EventUnshaded, card 17 Alberto Bayo, 2nd Eligible, recorded nothing, complete]
            [Sierra Maestra]
            Directorio executes the Event, unshaded text
            Directorio grants the free Rally to 26July.
            26July Rallies in Sierra Maestra: No piece moves.
            {standings}"
        ),
    );
}

/// Assault, Operation Only, in several spaces, with a transfer and a removal. Firm costs 2
/// per space (R 6.3.1): Government 3 + 3 = 6 pays three. Havana is a City, where Troops and
/// Police count (R 3.2.4): Syndicate has no Guerrilla there, so its Casino closes instead of
/// being removed (R 1.4.5), and 3 → 2 are open. The Syndicate's margin is the lower of −5
/// and 12 − 30 = −18. Santiago's 2 Troops + 1 Police remove both Active 26July Guerrillas,
/// 12 + 2 = 14 Available, and Government still controls it. Las Villas has no enemy piece.
/// Camagüey City's Troop and 2 Police return: 3 + 1 and 8 + 2 Available, and nobody controls
/// it (R 1.7).
#[test]
fn an_assault_in_several_spaces_with_a_transfer_and_a_removal() {
    let mut played = at_card_18();
    played.play(&[
        "government:assault",
        "transfer",
        "ask:3:4",
        "authorize:3",
        "assault:havana:syndicate",
        "assault:las-villas:none",
        "assault:santiago:26july",
        "remove",
        "remove:camaguey-city:police:2",
        "remove:camaguey-city:troops:1",
        "done-removal",
        "end-assault",
    ]);
    assert_report(
        &played,
        &played.last(),
        "[Government Assault, card 18 Pact of Caracas, 1st Eligible, recorded Operation Only, complete]
        [Havana; Las Villas; Santiago de Cuba; Camagüey City]
        Government executes Assault in Havana, Las Villas and Santiago de Cuba
        Syndicate gives Government 3 Resources: Government asked for 4 Resources. Syndicate 15 → 12, Government 3 → 6.
        Assault in Havana: Government pays 2 Resources, 6 → 4. Syndicate: 1 Casino closes in place.
        Assault in Santiago de Cuba: Government pays 2 Resources, 2 → 0. 26July: 2 Active Guerrillas return to Available Forces.
        Government removes its own pieces: Camagüey City: 2 Police cubes and 1 Troop.
        Assault in Las Villas: Government pays 2 Resources, 4 → 2. No piece is removed or closed.
        | Government | 3 → 0 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 → 12 | 3 → 2 | -15 → -18 |
        Control: Camagüey City: Government → none.
        Available Forces: Government Troops: 3 → 4. Government Police cubes: 8 → 10. 26July Guerrillas: 12 → 14.
        Victory totals: Syndicate open Casinos: 3 → 2.
        Eligibility: Government is in the Operation Only box. Next: Syndicate, 2nd Eligible.",
    );
}

/// The zero-space Assault (the owner's ruling on #127): nothing is paid and no Pass income
/// is taken. Spaces where no piece changes are told together (one cost each, R 6.3.1).
#[test]
fn an_assault_in_no_space_and_in_spaces_without_enemies() {
    let mut played = at_card_18();
    let mut none = played.clone();
    none.play(&["government:assault", "end-assault"]);
    assert_report(
        &none,
        &none.last(),
        "[Government Assault, card 18 Pact of Caracas, 1st Eligible, recorded Operation Only, complete]
        []
        Government executes Assault in no space
        | Government | 3 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |
        Eligibility: Government is in the Operation Only box. Next: Syndicate, 2nd Eligible.",
    );
    // 3 + 5 = 8 pays four spaces.
    played.play(&[
        "government:assault",
        "transfer",
        "ask:3:5",
        "authorize:5",
        "assault:matanzas:none",
        "assault:las-villas:none",
        "assault:ec-oriente-sierra:none",
        "assault:camaguey-city:none",
        "end-assault",
    ]);
    assert_report(
        &played,
        &played.last(),
        "[Government Assault, card 18 Pact of Caracas, 1st Eligible, recorded Operation Only, complete]
        [Matanzas; Las Villas; Oriente–Sierra Maestra EC; Camagüey City]
        Government executes Assault in 4 spaces
        Syndicate gives Government 5 Resources: Government asked for 5 Resources. Syndicate 15 → 10, Government 3 → 8.
        Assault in 4 spaces: No piece is removed or closed. Government pays 2 Resources in each of Matanzas, Las Villas, Oriente–Sierra Maestra EC and Camagüey City.
        | Government | 3 → 0 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 → 10 | 3 | -15 → -20 |
        Eligibility: Government is in the Operation Only box. Next: Syndicate, 2nd Eligible.",
    );
}

// --- Partway ------------------------------------------------------------------------

/// Government Assaults Havana (3 − 2 = 1) and asks 26July for 2. The first report ends there
/// and the second starts there: it repeats neither Havana nor what was asked. 26July, which
/// has 2, gives 1 (R 1.5.1), so Government can pay for Santiago: 1 + 1 − 2 = 0.
#[test]
fn a_report_starts_where_the_previous_one_ended() {
    let mut played = at_card_18();
    let began = played.revision();
    played.play(&["government:assault", "assault:havana:syndicate"]);
    played.play(&["transfer", "ask:1:2"]);
    let partway = played.last_since(began);
    assert_report(
        &played,
        &partway,
        "[Government Assault, card 18 Pact of Caracas, 1st Eligible, recorded nothing, partway]
        [Havana]
        Government executes Assault: Havana so far
        Government asks 26July for 2 Resources. 26July decides.
        Assault in Havana: Government pays 2 Resources, 3 → 1. Syndicate: 1 Casino closes in place.
        | Government | 3 → 1 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 → 2 | -15 |
        Victory totals: Syndicate open Casinos: 3 → 2.",
    );
    assert_eq!(partway.from_revision, began);
    assert_eq!(partway.to_revision, played.revision());
    // Setting the decision aside and opening it again does nothing to report.
    played.play(&["wait", "reopen"]);
    assert_eq!(played.reports.since(partway.to_revision), []);
    played.play(&["authorize:1", "assault:santiago:26july", "end-assault"]);
    let rest = played.last_since(partway.to_revision);
    assert_eq!(rest.from_revision, partway.to_revision);
    assert_report(
        &played,
        &rest,
        "[Government Assault, card 18 Pact of Caracas, 1st Eligible, recorded Operation Only, continues, complete]
        [Santiago de Cuba]
        Government ends Assault: Santiago de Cuba
        26July gives Government 1 Resource: 26July 2 → 1, Government 1 → 2.
        Assault in Santiago de Cuba: Government pays 2 Resources, 2 → 0. 26July: 2 Active Guerrillas return to Available Forces.
        | Government | 1 → 0 | 16 | -2 |
        | M26 | 2 → 1 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 2 | -15 |
        Available Forces: 26July Guerrillas: 12 → 14.
        Eligibility: Government is in the Operation Only box. Next: Syndicate, 2nd Eligible.",
    );
    // Nothing is left to report after it.
    assert_eq!(played.reports.since(rest.to_revision), []);
}

/// 26July's Terror, reported when it asks Government and again when it ends: Government
/// answers with nothing, and Syndicate's own report of nothing is none.
#[test]
fn a_partway_terror_continues_and_ends() {
    let mut played = Match::played(5, &["terror", "transfer", "ask:0:2"]);
    let asked = played.last_since(0);
    assert_eq!(asked.headline, "26July executes Terror");
    assert_eq!(
        asked.summary.as_deref(),
        Some("26July asks Government for 2 Resources. Government decides.")
    );
    played.play(&[
        "decline",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "ask:3:1",
    ]);
    let second = played.last_since(asked.to_revision);
    assert_eq!(second.headline, "26July continues Terror: Santiago de Cuba");
    assert_eq!(second.articles[0].title, "Government gives 26July nothing");
    assert_eq!(
        second.articles[0].lines,
        ["Government answers 1 request with nothing."]
    );
    reader::check(&played, &second).unwrap();
    played.play(&["end-terror"]);
    let last = played.last_since(second.to_revision);
    assert_eq!(last.headline, "26July ends Terror");
    assert_eq!(
        last.articles[0].title,
        "26July asks Syndicate for Resources"
    );
    assert_eq!(last.articles[0].lines, ["26July withdraws 1 request."]);
    assert_eq!(last.recorded, Some("Operation with Special Activity"));
    reader::check(&played, &last).unwrap();
}

/// Directorio's Event contains a decision of 26July's, the Rally's beneficiary (R 5.1): the
/// action is Directorio's throughout, reported partway when 26July must decide.
#[test]
fn the_event_is_directorios_while_26july_decides_inside_it() {
    let mut played = Match::played(5, &HANDOFF);
    let handoff = played.revision();
    played.play(&["event:unshaded", "rally:beneficiary:26july"]);
    let granted = played.last_since(handoff);
    assert_report(
        &played,
        &granted,
        "[Directorio EventUnshaded, card 17 Alberto Bayo, 2nd Eligible, recorded nothing, partway]
        []
        Directorio executes the Event, unshaded text
        Directorio grants the free Rally to 26July.
        | Government | 3 | 16 | -2 |
        | M26 | 2 | 8 | -7 |
        | Directorio | 5 | 1 | -8 |
        | Syndicate | 15 | 3 | -15 |",
    );
    played.play(&["rally:sierra-maestra:place:4"]);
    let rest = played.last_since(granted.to_revision);
    assert_eq!(rest.faction, Faction::Directorio);
    assert_eq!(rest.headline, "Directorio ends the Event, unshaded text");
    assert_eq!(rest.summary, None, "the grant is not repeated");
    assert_eq!(rest.articles[0].title, "26July Rallies in Sierra Maestra");
    reader::check(&played, &rest).unwrap();
    // Two actions: 26July's Terror and Directorio's Event.
    let factions: Vec<_> = played.reports.all().iter().map(|r| r.faction).collect();
    assert_eq!(factions, [Faction::M26, Faction::Directorio]);
}

/// 26July goes back to its action choice (local policy, #61). A report told its Terror, so
/// the next one tells the way back; where none did, nothing is told of it.
#[test]
fn going_back_to_the_action_choice() {
    let mut played = Match::played(5, &["terror", "transfer", "ask:0:2"]);
    let asked = played.last_since(0);
    played.play(&["back-to-choice"]);
    assert_eq!(played.reports.all(), []);
    let back = played.last_since(asked.to_revision);
    assert_eq!(back.headline, "26July goes back to its action choice");
    assert_eq!(
        back.articles[0].lines,
        ["26July withdraws its request to Government."]
    );
    assert!(!back.complete);
    reader::check(&played, &back).unwrap();
    played.play(&["pass"]);
    let passed = played.last_since(asked.to_revision);
    assert_eq!(passed.headline, "26July Passes");
    assert_eq!(
        passed.articles[0].lines,
        ["26July withdraws its request to Government."]
    );
    reader::check(&played, &passed).unwrap();
    assert_eq!(
        text(&played.last()),
        text(&Match::played(5, &["pass"]).last()),
        "the whole action is a Pass"
    );
}

/// Transfers between two factions are told together, whatever their number: 26July asks
/// Syndicate three times and gets 2, nothing and 1, and gives Government 1 twice.
#[test]
fn repeated_transfers_are_told_together() {
    let mut played = Match::played(5, &["terror", "transfer", "ask:3:2", "authorize:2"]);
    played.play(&["transfer", "ask:3:4", "decline"]);
    played.play(&["transfer", "give:0:1", "transfer", "give:0:1"]);
    played.play(&["transfer", "ask:3:1", "authorize:1", "end-terror"]);
    let report = played.last();
    let told: Vec<String> = report
        .articles
        .iter()
        .map(|a| format!("{}: {}", a.title, a.lines.join(" ")))
        .collect();
    assert_eq!(
        told,
        [
            "Syndicate gives 26July 3 Resources: 26July asked 3 times. They move in 2 transfers. Syndicate answers 1 request with nothing.",
            "26July gives Government 2 Resources: They move in 2 transfers.",
        ]
    );
    reader::check(&played, &report).unwrap();
}

// --- Every reachable report ---------------------------------------------------------

/// A deterministic source of picks.
struct Picks(u64);

impl Picks {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as usize % n
    }
}

/// A choice among what the rules offer: first the kind of decision, so that the many
/// amounts of a transfer do not crowd out the rest, and an ending more rarely.
fn next_action(played: &Match, picks: &mut Picks) -> Option<String> {
    let mut kinds: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for choice in &played.now().choices {
        if choice.supported {
            let action = choice.action().to_owned();
            let kind = action.split(':').next().unwrap().to_owned();
            kinds.entry(kind).or_default().push(action);
        }
    }
    for ending in ["pass", "end-terror", "end-assault", "done-removal"] {
        if kinds.len() > 1 && picks.below(4) > 0 {
            kinds.remove(ending);
        }
    }
    if kinds.is_empty() {
        return None;
    }
    let of = picks.below(kinds.len());
    let actions = kinds.into_values().nth(of).unwrap();
    Some(actions[picks.below(actions.len())].clone())
}

/// A match played by picks to an end of the build, or for `most` decisions.
/// `at` sees the match after each decision.
fn walk(seed: u64, most: usize, mut at: impl FnMut(&Match, &mut Picks)) {
    let mut picks = Picks(seed);
    let mut played = Match::opening(5 + seed % 12);
    // Most walks start where the later actions are, which a free walk seldom reaches.
    if seed % 4 >= 1 {
        played.play(&HANDOFF);
    }
    if seed % 4 == 3 {
        played.play(&CARD_18);
    }
    for _ in 0..most {
        let Some(action) = next_action(&played, &mut picks) else {
            break;
        };
        played.play(&[&action]);
        at(&played, &mut picks);
    }
}

/// Every report of many matches played at random, whole and from any revision, at any
/// moment, says what was committed and nothing else, and each starts where the last ended.
#[test]
fn every_report_of_random_matches_says_what_was_committed() {
    let seeds: Vec<u64> = (0..400).collect();
    let told: Vec<BTreeMap<String, usize>> = parallel(&seeds, |_, &seed| {
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        walk(seed, 50, |played, picks| {
            let cut = picks.below(played.revision() as usize) as u64;
            for from in [0, cut] {
                let mut at = from;
                for report in played.reports.since(from) {
                    assert!(report.from_revision >= at && report.to_revision > at);
                    // What no report covers: turning the card.
                    for left in at..report.from_revision {
                        assert_eq!(played.decisions[left as usize].1, "advance-card");
                    }
                    at = report.to_revision;
                    reader::check(played, &report)
                        .unwrap_or_else(|why| panic!("seed {seed}: {why}\n{}", text(&report)));
                    let (words, articles, _) = length(&report);
                    assert!(words <= 1835 && articles <= 13, "{}", text(&report));
                    *kinds.entry(format!("{:?}", report.action)).or_default() += 1;
                }
            }
        });
        kinds
    });
    let mut all: BTreeMap<String, usize> = BTreeMap::new();
    for (kind, count) in told.into_iter().flatten() {
        *all.entry(kind).or_default() += count;
    }
    for (kind, least) in [
        ("Undecided", 10),
        ("Pass", 100),
        ("Terror", 1000),
        ("EventShaded", 1000),
        ("EventUnshaded", 1000),
        ("Assault", 1000),
        ("Construct", 100),
    ] {
        assert!(
            all.get(kind).is_some_and(|count| *count >= least),
            "{all:?}"
        );
    }
}

/// How long a report is: the characters of its words, and its articles and notes.
fn length(report: &Report) -> (usize, usize, usize) {
    let words: usize = [&report.headline]
        .into_iter()
        .chain(&report.summary)
        .chain(
            report
                .articles
                .iter()
                .chain(&report.notes)
                .flat_map(|article| std::iter::once(&article.title).chain(&article.lines)),
        )
        .map(|text| text.chars().count())
        .sum();
    (words, report.articles.len(), report.notes.len())
}

/// The longest report this build was found to make of each action but the Pass.
fn longest() -> Vec<Match> {
    longest::longest()
        .iter()
        .map(|decisions| {
            let decisions: Vec<&str> = decisions.iter().map(String::as_str).collect();
            Match::played(SEED, &decisions)
        })
        .collect()
}

/// What #208 lays out: how long the longest and the shortest reports are. The counts of
/// articles are the most there can be; the characters can grow a little with the digits of
/// the Resources and of the counts.
#[test]
fn the_longest_and_the_shortest_reports() {
    let found: Vec<_> = longest()
        .into_iter()
        .map(|played| {
            let report = played.last();
            reader::check(&played, &report).unwrap();
            assert!(report.complete);
            (report.action, length(&report))
        })
        .collect();
    assert_eq!(
        found,
        [
            (Action::Terror, (1682, 10, 5)),
            (Action::EventShaded, (1189, 8, 4)),
            (Action::EventUnshaded, (1835, 13, 4)),
            (Action::Assault, (1725, 10, 4)),
        ]
    );
    let mut passed = at_card_18();
    passed.play(&["government:pass"]);
    assert_eq!(length(&passed.last()), (127, 0, 1));
    let mut none = at_card_18();
    none.play(&["government:assault", "end-assault"]);
    assert_eq!(length(&none.last()), (120, 0, 1));
}

/// A match's reports are in no save and follow from its decisions: a reopened one has the
/// same. What follows the position it reopened at starts there.
#[test]
fn a_reopened_match_has_the_same_reports() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("match.json");
    let mut session = Session::create_seeded(&path, 5).unwrap();
    let play = |session: &mut Session, action: &str| {
        let seen = session.observe();
        let choice = seen.choices.iter().find(|c| c.action() == action).unwrap();
        session
            .submit(choice.owner, seen.revision, &choice.id)
            .unwrap();
    };
    for action in HANDOFF
        .iter()
        .chain(&["event:unshaded", "rally:beneficiary:26july"])
    {
        play(&mut session, action);
    }
    let reports = session.reports();
    assert_eq!(reports.len(), 2);
    assert!(reports[0].complete && !reports[1].complete);
    // A refused decision is none.
    assert!(session.submit(Faction::M26, 7, "7:pass").is_err());
    assert_eq!(session.reports(), reports);
    drop(session);
    let mut reopened = Session::open(&path, None).unwrap();
    assert_eq!(reopened.reports(), reports);
    let at = reopened.observe().revision;
    assert_eq!(reopened.reports_since(at), []);
    play(&mut reopened, "rally:sierra-maestra:place:4");
    let rest = reopened.reports_since(at);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].from_revision, at);
    assert_eq!(rest[0].headline, "Directorio ends the Event, unshaded text");
}

/// The reports the planted errors are planted in: each kind, whole and partway.
fn samples() -> Vec<(Match, Report)> {
    let mut samples = vec![];
    for actions in [
        &["pass"][..],
        &[
            "terror",
            "transfer",
            "ask:3:3",
            "authorize:2",
            "terror:oriente",
            "terror:santiago",
            "end-terror",
        ],
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:oriente",
            "terror:santiago",
            "transfer",
            "give:2:1",
            "remove",
            "remove:oriente:active:1",
            "done-removal",
            "end-terror",
        ],
        &["terror", "transfer", "ask:0:2"],
        &[
            "terror",
            "transfer",
            "ask:3:2",
            "authorize:2",
            "transfer",
            "ask:3:4",
            "decline",
            "transfer",
            "ask:3:1",
            "authorize:1",
        ],
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
        ],
        &[
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:unshaded",
            "rally:beneficiary:26july",
            "remove",
            "remove:santiago:active:1",
            "done-removal",
            "rally:sierra-maestra:place:4",
        ],
    ] {
        let played = Match::played(5, actions);
        let report = played.last();
        samples.push((played, report));
    }
    for actions in [
        &["government:pass"][..],
        &[
            "government:assault",
            "transfer",
            "ask:3:8",
            "authorize:7",
            "assault:havana:syndicate",
            "assault:las-villas:none",
            "assault:matanzas:none",
            "assault:oriente:none",
            "assault:santiago:26july",
            "remove",
            "remove:camaguey-city:police:2",
            "done-removal",
            "end-assault",
        ],
        &[
            "government:assault",
            "assault:havana:syndicate",
            "transfer",
            "ask:1:2",
        ],
    ] {
        let mut played = at_card_18();
        played.play(actions);
        let report = played.last();
        samples.push((played, report));
    }
    samples
}

/// `text` with each of its numbers in turn replaced by another.
fn wrong_numbers(text: &str) -> Vec<String> {
    let mut found = vec![];
    let digits = |c: char| c.is_ascii_digit();
    let mut from = 0;
    while let Some(start) = text[from..].find(digits).map(|at| at + from) {
        let end = text[start..]
            .find(|c| !digits(c))
            .map_or(text.len(), |at| at + start);
        // "26July" is a name.
        if !text[end..].starts_with("July") {
            let number: u32 = text[start..end].parse().unwrap();
            found.push(format!("{}{}{}", &text[..start], number + 1, &text[end..]));
        }
        from = end;
    }
    found
}

/// `text` with each place and faction it names in turn replaced by another.
fn wrong_names(text: &str, names: &[String]) -> Vec<String> {
    names
        .iter()
        .enumerate()
        .filter(|(_, name)| text.contains(name.as_str()))
        .map(|(nth, name)| text.replacen(name.as_str(), &names[(nth + 1) % names.len()], 1))
        .collect()
}

/// Every copy of `report` with one wrong number or name planted in it.
fn planted(report: &Report, names: &[String]) -> Vec<Report> {
    let mut found = vec![];
    let wrong = |text: &str| {
        let mut all = wrong_numbers(text);
        all.extend(wrong_names(text, names));
        all
    };
    let mut put = |with: &dyn Fn(&mut Report)| {
        let mut copy = report.clone();
        with(&mut copy);
        assert_ne!(copy, *report);
        found.push(copy);
    };
    for text in wrong(&report.headline) {
        put(&|r| r.headline = text.clone());
    }
    for text in report.summary.iter().flat_map(|text| wrong(text)) {
        put(&|r| r.summary = Some(text.clone()));
    }
    for (nth, name) in report.spaces.iter().enumerate() {
        for text in wrong_names(name, names) {
            put(&|r| r.spaces[nth] = text.clone());
        }
        put(&|r| drop(r.spaces.remove(nth)));
    }
    for (in_notes, articles) in [(false, &report.articles), (true, &report.notes)] {
        for (nth, article) in articles.iter().enumerate() {
            for text in wrong(&article.title) {
                put(&|r| {
                    let list = if in_notes {
                        &mut r.notes
                    } else {
                        &mut r.articles
                    };
                    list[nth].title = text.clone();
                });
            }
            for (line, text) in article.lines.iter().enumerate() {
                for text in wrong(text) {
                    put(&|r| {
                        let list = if in_notes {
                            &mut r.notes
                        } else {
                            &mut r.articles
                        };
                        list[nth].lines[line] = text.clone();
                    });
                }
                put(&|r| {
                    let list = if in_notes {
                        &mut r.notes
                    } else {
                        &mut r.articles
                    };
                    list[nth].lines.remove(line);
                });
            }
            if let Some(die) = article.die {
                put(&|r| {
                    let list = if in_notes {
                        &mut r.notes
                    } else {
                        &mut r.articles
                    };
                    list[nth].die = Some(die % 6 + 1);
                });
            }
        }
    }
    for nth in 0..report.standings.len() {
        put(&|r| r.standings[nth].resources.from += 1);
        put(&|r| r.standings[nth].resources.to += 1);
        put(&|r| r.standings[nth].score.from += 1);
        put(&|r| r.standings[nth].score.to += 1);
        put(&|r| r.standings[nth].margin.from += 1);
        put(&|r| r.standings[nth].margin.to += 1);
    }
    put(&|r| r.card += 1);
    put(&|r| r.complete = !r.complete);
    put(&|r| r.continues = !r.continues);
    put(&|r| r.recorded = Some("Event"));
    found
}

/// The reader bites: a wrong number, a wrong place or faction, or a sentence left out,
/// planted anywhere in a report of each kind, is found.
#[test]
fn a_planted_wrong_number_or_place_is_found() {
    let mut names: Vec<String> = Game::opening()
        .observe()
        .spaces
        .into_iter()
        .map(|space| space.name)
        .collect();
    names.extend(Faction::ALL.map(|faction| faction.name().to_owned()));
    let mut count = 0;
    for (played, report) in samples() {
        reader::check(&played, &report).unwrap();
        for wrong in planted(&report, &names) {
            assert!(
                reader::check(&played, &wrong).is_err(),
                "not found:\n{}\nin place of:\n{}",
                text(&wrong),
                text(&report)
            );
            count += 1;
        }
    }
    assert!(count > 500, "{count} planted");
}

/// Reads a report back. It takes what was done from the decisions and the positions, never
/// from the report's own code, and replays each sentence that moves something from the
/// position before, which must give the position after.
mod reader {
    use super::*;

    type Check = Result<(), String>;

    fn same<T: PartialEq + std::fmt::Debug>(what: &str, found: &T, expected: &T) -> Check {
        (found == expected)
            .then_some(())
            .ok_or_else(|| format!("{what}: {found:?}, not {expected:?}"))
    }

    fn resources(n: i32) -> String {
        format!("{n} Resource{}", if n == 1 { "" } else { "s" })
    }

    fn several(n: i32, one: &str, many: &str) -> String {
        format!("{n} {}", if n == 1 { one } else { many })
    }

    fn listing(parts: &[String]) -> String {
        match parts {
            [] => String::new(),
            [one] => one.clone(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        }
    }

    /// What the `{}`s of `pattern` hold in `text`.
    fn scan<'a>(text: &'a str, pattern: &str) -> Option<Vec<&'a str>> {
        let mut literals = pattern.split("{}");
        let mut rest = text.strip_prefix(literals.next()?)?;
        let literals: Vec<&str> = literals.collect();
        let mut held = vec![];
        for (nth, literal) in literals.iter().enumerate() {
            let at = if nth + 1 == literals.len() {
                rest.len().checked_sub(literal.len())?
            } else {
                rest.find(literal)?
            };
            held.push(&rest[..at]);
            rest = rest[at..].strip_prefix(literal)?;
        }
        rest.is_empty().then_some(held)
    }

    fn number(text: &str) -> Result<i32, String> {
        text.parse().map_err(|_| format!("not a number: {text}"))
    }

    fn faction(name: &str) -> Result<Faction, String> {
        Faction::named(name).ok_or_else(|| format!("no faction {name}"))
    }

    type Pieces = BTreeMap<(String, usize, String, String), i32>;

    /// A position: what a report's sentences move.
    #[derive(Debug, PartialEq)]
    struct Position {
        resources: [i32; 4],
        pieces: Pieces,
        support: BTreeMap<String, i32>,
        terror: BTreeMap<String, i32>,
    }

    impl Position {
        fn of(seen: &Observation) -> Self {
            let mut pieces = Pieces::new();
            for space in &seen.spaces {
                for p in &space.pieces {
                    *pieces
                        .entry(piece(&space.name, p.faction, p.kind, p.state))
                        .or_default() += i32::from(p.count);
                }
            }
            Self {
                resources: seen.resources.map(i32::from),
                pieces,
                support: seen
                    .spaces
                    .iter()
                    .map(|s| (s.name.clone(), i32::from(s.support)))
                    .collect(),
                terror: seen
                    .spaces
                    .iter()
                    .map(|s| (s.name.clone(), i32::from(s.terror)))
                    .collect(),
            }
        }
        fn add(&mut self, space: &str, faction: Faction, kind: Kind, state: State, n: i32) {
            *self
                .pieces
                .entry(piece(space, faction, kind, state))
                .or_default() += n;
        }
        fn turn(&mut self, space: &str, from: State, to: State, n: i32) {
            self.add(space, Faction::M26, Kind::Guerrilla, from, -n);
            self.add(space, Faction::M26, Kind::Guerrilla, to, n);
        }
        fn pay(&mut self, faction: Faction, n: i32) {
            self.resources[faction.index()] -= n;
        }
    }

    fn piece(
        space: &str,
        faction: Faction,
        kind: Kind,
        state: State,
    ) -> (String, usize, String, String) {
        (
            space.to_owned(),
            faction.index(),
            format!("{kind:?}"),
            format!("{state:?}"),
        )
    }

    /// "2 Active Guerrillas", "1 Police cube": how many of what, spelled right.
    fn group(text: &str) -> Result<(i32, Kind, State), String> {
        let (n, rest) = text.split_once(' ').ok_or("no group")?;
        let n = number(n)?;
        for (one, many, kind, state) in [
            ("Troop", "Troops", Kind::Troops, State::Ordinary),
            ("Police cube", "Police cubes", Kind::Police, State::Ordinary),
            ("Base", "Bases", Kind::Base, State::Ordinary),
            (
                "Active Guerrilla",
                "Active Guerrillas",
                Kind::Guerrilla,
                State::Active,
            ),
            (
                "Underground Guerrilla",
                "Underground Guerrillas",
                Kind::Guerrilla,
                State::Underground,
            ),
        ] {
            if rest == if n == 1 { one } else { many } {
                return Ok((n, kind, state));
            }
        }
        Err(format!("no group: {text}"))
    }

    const LEVELS: [&str; 5] = [
        "Active Opposition",
        "Passive Opposition",
        "Neutral",
        "Passive Support",
        "Active Support",
    ];

    fn level(name: &str) -> Result<i32, String> {
        LEVELS
            .iter()
            .position(|level| *level == name)
            .map(|at| at as i32 - 2)
            .ok_or_else(|| format!("no level {name}"))
    }

    /// What was asked and given from one faction to another, read from the decisions.
    #[derive(Default, Clone)]
    struct Between {
        asks: Vec<i32>,
        /// Each move, with the giver's and the recipient's balances around it.
        moves: Vec<[i32; 4]>,
        unmet: i32,
        withdrawn: i32,
        pending: Option<i32>,
    }

    struct CasinoAction {
        space: String,
        state: State,
        count: i32,
        closes: bool,
        effects: Vec<String>,
    }

    struct ConstructPayment {
        space: String,
        opens: bool,
        effects: Vec<String>,
    }

    /// What the decisions of a span did, in order.
    #[derive(Default)]
    struct Done {
        transfers: Vec<((Faction, Faction), Between)>,
        spaces: Vec<String>,
        /// The spaces the Operation was executed in, with the payer's balance before.
        executed: Vec<(String, i32)>,
        /// The Casino effects of this Construct's payments, keyed by space.
        constructed: Vec<ConstructPayment>,
        casino_management: Vec<CasinoAction>,
        /// Those of an Assault in which no piece changed.
        unchanged: Vec<String>,
        removers: Vec<Faction>,
        rallies: Vec<(Faction, String)>,
        kidnapped: bool,
        shaded: bool,
        granted: Option<Faction>,
        /// The beneficiary of a Rally that ended in no space.
        no_rally: Option<Faction>,
        /// 26July went back, withdrawing its request to this giver if it had one.
        back: Option<Option<Faction>>,
        passed: Option<[i32; 2]>,
    }

    fn request(seen: &Observation) -> Option<(Faction, Faction)> {
        match seen.transfer {
            Some(Transfer::Authorizing {
                giver, recipient, ..
            }) => Some((giver, recipient)),
            _ => None,
        }
    }

    impl Done {
        fn between(&mut self, pair: (Faction, Faction), reuse: bool) -> &mut Between {
            if reuse && let Some(at) = self.transfers.iter().rposition(|(known, _)| *known == pair)
            {
                return &mut self.transfers[at].1;
            }
            self.transfers.push((pair, Between::default()));
            &mut self.transfers.last_mut().unwrap().1
        }
        fn at(&mut self, space: &str) {
            if !self.spaces.iter().any(|known| known == space) {
                self.spaces.push(space.to_owned());
            }
        }
        fn of(seen: &[Observation], decisions: &[(Faction, String)], continues: bool) -> Self {
            let mut done = Self::default();
            for (nth, (_, action)) in decisions.iter().enumerate() {
                let (seen, next) = (&seen[nth], &seen[nth + 1]);
                let parts: Vec<&str> = action.split(':').collect();
                let name = |id: &str| -> String {
                    seen.spaces
                        .iter()
                        .find(|s| s.id == id)
                        .unwrap_or_else(|| panic!("no space {id}"))
                        .name
                        .clone()
                };
                let held = |f: Faction| i32::from(seen.resources[f.index()]);
                let holds = |f: Faction| i32::from(next.resources[f.index()]);
                let moving = |(g, r): (Faction, Faction)| [held(g), holds(g), held(r), holds(r)];
                let transfer = seen
                    .choices
                    .iter()
                    .find(|c| c.action() == action)
                    .and_then(Choice::transfer);
                let group_transfers = seen.executing != Some(Faction::Syndicate);
                let authorizations = seen
                    .construct_funding
                    .as_ref()
                    .map_or(&[][..], |funding| &funding.authorizations);
                match (parts[0], transfer) {
                    (_, Some(TransferChoice::Ask { giver, amount })) => {
                        let told = done.between((giver, seen.executing.unwrap()), group_transfers);
                        told.asks.push(amount.into());
                        told.pending = Some(amount.into());
                    }
                    (_, Some(TransferChoice::Give { recipient, amount })) if amount > 0 => {
                        let pair = (seen.executing.unwrap(), recipient);
                        done.between(pair, group_transfers).moves.push(moving(pair));
                    }
                    (_, Some(TransferChoice::Authorize { amount }))
                        if !group_transfers
                            && amount > 0
                            && (!authorizations.is_empty() || seen.resources == next.resources) =>
                    {
                        if let Some(index) = done
                            .transfers
                            .iter()
                            .position(|(_, told)| told.pending.is_some())
                        {
                            done.transfers.remove(index);
                        }
                        if seen.resources != next.resources {
                            let (giver, recipient) = request(seen).unwrap();
                            let Some(Transfer::Authorizing { amount: asked, .. }) = seen.transfer
                            else {
                                unreachable!();
                            };
                            let mut balances = seen.resources.map(i32::from);
                            for (giver, asked, given) in authorizations
                                .iter()
                                .map(|a| (a.giver, a.asked, a.amount))
                                .chain([(giver, asked, amount)])
                            {
                                let from = balances[giver.index()];
                                let to = balances[recipient.index()];
                                let given = i32::from(given);
                                let told = done.between((giver, recipient), false);
                                told.asks.push(asked.into());
                                told.moves.push([from, from - given, to, to + given]);
                                balances[giver.index()] -= given;
                                balances[recipient.index()] += given;
                            }
                            assert_eq!(balances, next.resources.map(i32::from));
                        }
                    }
                    ("decline", _) | (_, Some(TransferChoice::Authorize { .. })) => {
                        let pair = request(seen).unwrap();
                        let told = done.between(pair, true);
                        told.pending = None;
                        if matches!(transfer, Some(TransferChoice::Authorize { amount }) if amount > 0)
                        {
                            told.moves.push(moving(pair));
                        } else {
                            told.unmet += 1;
                        }
                    }
                    ("back-to-choice", _) => {
                        let giver = request(seen).map(|(giver, _)| giver);
                        done = Self::default();
                        done.back = continues.then_some(giver);
                    }
                    ("end-terror", _) => {
                        if let Some(pair) = request(seen) {
                            let told = done.between(pair, true);
                            told.pending = None;
                            told.withdrawn += 1;
                        }
                    }
                    ("pass", _) => done.passed = Some([held(Faction::M26), holds(Faction::M26)]),
                    ("government", _) if parts[1] == "pass" => {
                        let g = Faction::Government;
                        done.passed = Some([held(g), holds(g)]);
                    }
                    ("syndicate", _) if parts[1] == "pass" => {
                        let s = Faction::Syndicate;
                        done.passed = Some([held(s), holds(s)]);
                    }
                    ("kidnap", _) if parts.get(1) == Some(&"roll") => {
                        done.kidnapped = true;
                        done.at("Santiago de Cuba");
                    }
                    ("terror", _) if parts.len() == 2 => {
                        done.at(&name(parts[1]));
                        done.executed.push((name(parts[1]), held(Faction::M26)));
                    }
                    ("assault", _) => {
                        let here = name(parts[1]);
                        done.at(&here);
                        done.executed
                            .push((here.clone(), held(Faction::Government)));
                        if parts[2] == "none" {
                            done.unchanged.push(here);
                        }
                    }
                    ("construct", _) => {
                        let here = name(parts[1]);
                        done.at(&here);
                        done.executed.push((here.clone(), held(Faction::Syndicate)));
                        done.constructed.push(ConstructPayment {
                            space: here,
                            opens: parts[2] == "open",
                            effects: casino_words(seen, next, parts[1]),
                        });
                    }
                    ("close-casino" | "remove", _)
                        if parts.len() == 4
                            && matches!(parts[2], "open-casino" | "closed-casino") =>
                    {
                        let space = name(parts[1]);
                        done.at(&space);
                        done.casino_management.push(CasinoAction {
                            space,
                            state: if parts[2] == "open-casino" {
                                State::Open
                            } else {
                                State::Closed
                            },
                            count: parts[3].parse().unwrap(),
                            closes: parts[0] == "close-casino",
                            effects: casino_words(seen, next, parts[1]),
                        });
                    }
                    ("remove", _) if parts.len() == 4 => {
                        done.at(&name(parts[1]));
                        let remover = seen.executing.unwrap();
                        if !done.removers.contains(&remover) {
                            done.removers.push(remover);
                        }
                    }
                    ("rally", _) if parts[1] == "beneficiary" => {
                        done.granted = Faction::named(parts[2]);
                    }
                    ("rally", _) => {
                        done.at(&name(parts[1]));
                        done.rallies.push((seen.executing.unwrap(), name(parts[1])));
                    }
                    ("resolve-shaded", _) => {
                        done.shaded = true;
                        for space in &seen.spaces {
                            if space.pieces.iter().any(|p| {
                                p.faction == Faction::M26
                                    && p.kind == Kind::Guerrilla
                                    && p.state == State::Underground
                            }) {
                                done.at(&space.name);
                            }
                        }
                    }
                    _ => {}
                }
            }
            done
        }
    }

    /// Whether `report` says what `played` committed between its revisions, all of it and
    /// nothing else.
    pub fn check(played: &Match, report: &Report) -> Check {
        let opening = played.seen[0].revision;
        let (from, to) = (
            (report.from_revision - opening) as usize,
            (report.to_revision - opening) as usize,
        );
        let seen = &played.seen[from..=to];
        let (before, after) = (&played.seen[from], &played.seen[to]);
        let decisions = &played.decisions[from..to];
        let card = before.current_card;
        let syndicate_handoff = played.decisions[..from]
            .iter()
            .position(|(_, a)| matches!(a.as_str(), "government:pass" | "end-assault"));
        let executor = if card == 18 {
            if syndicate_handoff.is_some() {
                Faction::Syndicate
            } else {
                Faction::Government
            }
        } else if before.finished {
            Faction::Directorio
        } else {
            Faction::M26
        };
        let all = || played.decisions[..to].iter().map(|(_, a)| a.as_str());
        let chosen = |upto: usize| -> Option<&str> {
            let seen = &played.seen[upto];
            let made = |action: &str| played.decisions[..upto].iter().any(|(_, a)| a == action);
            match executor {
                Faction::M26 if made("pass") => Some("Pass"),
                Faction::M26 => (seen.action == ActionChoice::Terror).then_some("Terror"),
                Faction::Directorio if made("event:shaded") => Some("the Event, shaded text"),
                Faction::Directorio => made("event:unshaded").then_some("the Event, unshaded text"),
                Faction::Government if made("government:pass") => Some("Pass"),
                Faction::Government => made("government:assault").then_some("Assault"),
                Faction::Syndicate if made("syndicate:pass") => Some("Pass"),
                Faction::Syndicate
                    if seen.executing == Some(Faction::Syndicate) || made("end-construct") =>
                {
                    Some(if made("government:pass") {
                        "Construct"
                    } else {
                        "Limited Construct"
                    })
                }
                Faction::Syndicate => None,
            }
        };
        // Where the action began: at the opening, or after the decision that ended the last.
        let began = match executor {
            Faction::M26 => 0,
            Faction::Directorio => 1 + all().position(|a| a == "end-terror").unwrap_or(0),
            Faction::Government => 1 + all().position(|a| a == "advance-card").unwrap_or(0),
            Faction::Syndicate => 1 + syndicate_handoff.unwrap(),
        };
        let continues = from > began;
        let ended = |seen: &Observation| seen.event.as_ref().is_some_and(|e| e.outcome().is_some());
        let mut done = Done::of(seen, decisions, continues);
        let rallied = |a: &str| a.starts_with("rally:") && !a.contains("beneficiary");
        if executor == Faction::Directorio && ended(after) && !all().any(rallied) {
            done.no_rally = all()
                .find_map(|a| a.strip_prefix("rally:beneficiary:"))
                .and_then(Faction::named);
        }
        let last = decisions.last().map_or("", |(_, a)| a.as_str());
        let complete = matches!(
            last,
            "pass"
                | "end-terror"
                | "government:pass"
                | "end-assault"
                | "syndicate:pass"
                | "end-construct"
        ) || (executor == Faction::Directorio && ended(after));
        let kidnapped = all().any(|a| a == "kidnap:roll");
        // R 2.3.4: the box of a recorded action. Directorio's Event has none in this build.
        let recorded = match last {
            "pass" | "government:pass" | "syndicate:pass" => Some("Pass"),
            "end-terror" if kidnapped => Some("Operation with Special Activity"),
            "end-terror" | "end-assault" => Some("Operation Only"),
            "end-construct" if all().any(|a| a == "government:pass") => Some("Operation Only"),
            "end-construct" => Some("Limited Operation"),
            _ => None,
        };
        same("faction", &report.faction, &executor)?;
        same("card", &report.card, &card)?;
        let title = if card == 17 {
            "Alberto Bayo"
        } else {
            "Pact of Caracas"
        };
        same("card title", &report.card_title, &title)?;
        let role = if executor == Faction::Directorio
            || syndicate_handoff.is_some_and(|index| played.decisions[index].1 == "end-assault")
        {
            "2nd Eligible"
        } else {
            "1st Eligible"
        };
        same("role", &report.role, &role)?;
        same("continues", &report.continues, &continues)?;
        same("complete", &report.complete, &complete)?;
        same("recorded", &report.recorded, &recorded)?;
        let mut told = report.spaces.clone();
        let mut expected = done.spaces.clone();
        told.sort();
        expected.sort();
        same("spaces", &told, &expected)?;
        let action = chosen(to);
        let again = continues && chosen(from) == action;
        let headline = headline(executor, action, again, complete, &done);
        same("headline", &report.headline, &headline)?;
        same("summary", &report.summary, &summary(executor, &done, after))?;

        // The titles the decisions call for, and the sentences, replayed.
        let mut due: Vec<String> = titles(executor, &done);
        let mut transfers = done.transfers.iter().collect::<Vec<_>>();
        let mut position = Position::of(before);
        let mut casinos = done.casino_management.iter();
        for article in &report.articles {
            let at = due
                .iter()
                .position(|title| *title == article.title)
                .ok_or_else(|| format!("not due: {}", article.title))?;
            due.remove(at);
            let result = if article.title.starts_with("Casino management in ") {
                read_casino_action(
                    article,
                    casinos.next().ok_or("extra Casino action")?,
                    &mut position,
                )
            } else {
                read(article, &done, after, &mut position, &mut transfers)
            };
            result.map_err(|why| format!("{}: {why}", article.title))?;
        }
        same("articles left out", &due, &vec![])?;
        if let Some([from, to]) = done.passed {
            position.pay(executor, from - to);
        }
        position.pieces.retain(|_, n| *n != 0);
        same("replayed", &position, &Position::of(after))?;

        same("rows", &report.standings.len(), &4)?;
        for (row, faction) in report.standings.iter().zip(Faction::ALL) {
            same("row", &row.faction, &faction)?;
            for (seen, resources, score, margin) in [
                (before, row.resources.from, row.score.from, row.margin.from),
                (after, row.resources.to, row.score.to, row.margin.to),
            ] {
                let held = i16::from(seen.resources[faction.index()]);
                let (total, short) = standing(seen, faction);
                same("Resources", &resources, &held)?;
                same("score", &score, &total)?;
                same("margin", &margin, &short)?;
            }
        }
        let notes = notes(before, after, executor, recorded, complete);
        let found: Vec<_> = report
            .notes
            .iter()
            .map(|note| (note.title.clone(), note.lines.clone()))
            .collect();
        same("notes", &found, &notes)
    }

    fn headline(
        executor: Faction,
        action: Option<&str>,
        again: bool,
        complete: bool,
        done: &Done,
    ) -> String {
        let name = executor.name();
        let Some(action) = action else {
            return format!("{name} goes back to its action choice");
        };
        if action == "Pass" {
            return format!("{name} Passes");
        }
        let names: Vec<String> = done.executed.iter().map(|(s, _)| s.clone()).collect();
        let named = match names.len() {
            0 => String::new(),
            1..=3 => listing(&names),
            n => format!("{n} spaces"),
        };
        match (again, complete, named.is_empty()) {
            (false, true, false) => format!("{name} executes {action} in {named}"),
            (false, true, true) if !action.contains("Event") => {
                format!("{name} executes {action} in no space")
            }
            (false, _, true) => format!("{name} executes {action}"),
            (false, false, false) => format!("{name} executes {action}: {named} so far"),
            (true, _, empty) => {
                let verb = if complete { "ends" } else { "continues" };
                let named = if empty { named } else { format!(": {named}") };
                format!("{name} {verb} {action}{named}")
            }
        }
    }

    fn summary(executor: Faction, done: &Done, after: &Observation) -> Option<String> {
        let held = after.construct_funding.as_ref().filter(|funding| {
            executor == Faction::Syndicate
                && (request(after).is_some() || !funding.authorizations.is_empty())
        }).map(|funding| {
            let authorized: u8 = funding.authorizations.iter().map(|a| a.amount).sum();
            let needed = 5 - after.resources[Faction::Syndicate.index()] - authorized;
            format!("{authorized} conditionally authorized · {needed} still needed. Resources move only when authorizations fund a legal payment.")
        });
        for ((giver, recipient), told) in &done.transfers {
            if let Some(request) = pending(*giver, *recipient, told) {
                return Some(match held {
                    Some(held) => format!("{request} {held}"),
                    None => request,
                });
            }
        }
        if held.is_some() {
            return held;
        }
        if let Some([from, to]) = done.passed {
            let gained = resources(to - from);
            return Some(format!(
                "{} receives {gained}, {from} → {to}.",
                executor.name()
            ));
        }
        if done.kidnapped {
            let kidnap = after.kidnap.as_ref().unwrap();
            let taken = kidnap.die.min(kidnap.government_before);
            let taken = if taken == 0 {
                "Government has no Resources to take".to_owned()
            } else {
                format!("26July takes {} from Government", resources(taken.into()))
            };
            return Some(format!(
                "Kidnap in Santiago de Cuba: the die shows {}, and {taken}.",
                kidnap.die
            ));
        }
        done.granted
            .map(|f| format!("Directorio grants the free Rally to {}.", f.name()))
    }

    fn pending(giver: Faction, recipient: Faction, told: &Between) -> Option<String> {
        let (giver, recipient) = (giver.name(), recipient.name());
        told.pending.map(|n| {
            format!(
                "{recipient} asks {giver} for {}. {giver} decides.",
                resources(n)
            )
        })
    }

    fn title_of((giver, recipient): (Faction, Faction), told: &Between) -> String {
        let given: i32 = told.moves.iter().map(|m| m[0] - m[1]).sum();
        let (g, r) = (giver.name(), recipient.name());
        match given {
            0 if told.unmet > 0 => format!("{g} gives {r} nothing"),
            0 => format!("{r} asks {g} for Resources"),
            n => format!("{g} gives {r} {}", resources(n)),
        }
    }

    fn titles(executor: Faction, done: &Done) -> Vec<String> {
        let mut due = vec![];
        if done.back.is_some() {
            due.push("Back to the action choice".into());
        }
        for (pair, told) in &done.transfers {
            if told.moves.len() as i32 + told.unmet + told.withdrawn > 0 {
                due.push(title_of(*pair, told));
            }
        }
        for remover in &done.removers {
            due.push(format!("{} removes its own pieces", remover.name()));
        }
        for action in &done.casino_management {
            due.push(format!("Casino management in {}", action.space));
        }
        if done.kidnapped {
            due.push("Kidnap in Santiago de Cuba".into());
        }
        let operation = match executor {
            Faction::M26 => "Terror",
            Faction::Syndicate => "Construct",
            _ => "Assault",
        };
        for (space, _) in &done.executed {
            if !done.unchanged.contains(space) {
                due.push(format!("{operation} in {space}"));
            }
        }
        match &done.unchanged[..] {
            [] => {}
            [one] => due.push(format!("Assault in {one}")),
            many => due.push(format!("Assault in {} spaces", many.len())),
        }
        for (faction, space) in &done.rallies {
            due.push(format!("{} Rallies in {space}", faction.name()));
        }
        if done.shaded {
            due.push("26July's Guerrillas".into());
        }
        if done.no_rally.is_some() {
            due.push("The free Rally".into());
        }
        due
    }

    /// "26July pays 1 Resource, 3 → 2.", the balance being that of the decision.
    fn pays(line: &str, payer: Faction, balance: i32, position: &mut Position) -> Check {
        let held = scan(line, "{} pays {}, {} → {}.").ok_or("no payment")?;
        let (from, to) = (number(held[2])?, number(held[3])?);
        same("payer", &faction(held[0])?, &payer)?;
        same("payment", &held[1].to_owned(), &resources(from - to))?;
        same("balance", &from, &balance)?;
        position.pay(payer, from - to);
        Ok(())
    }

    fn turns(text: &str, space: &str, position: &mut Position) -> Check {
        let held = scan(text, "{} turn{} {} → {}").ok_or_else(|| format!("no turn: {text}"))?;
        let (n, _) = held[0].split_once(' ').ok_or("no count")?;
        let n = number(n)?;
        let guerrillas = several(n, "Guerrilla turns", "Guerrillas turn");
        same("turn", &format!("{} turn{}", held[0], held[1]), &guerrillas)?;
        match (held[2], held[3]) {
            ("Underground", "Active") => {
                position.turn(space, State::Underground, State::Active, n);
            }
            ("Active", "Underground") => {
                position.turn(space, State::Active, State::Underground, n);
            }
            other => return Err(format!("{other:?}")),
        }
        Ok(())
    }

    fn read_casino_action(
        article: &Article,
        action: &CasinoAction,
        position: &mut Position,
    ) -> Check {
        same(
            "Casino space",
            &article.title,
            &format!("Casino management in {}", action.space),
        )?;
        same("die", &article.die, &None)?;
        let state = if action.state == State::Open {
            "open"
        } else {
            "closed"
        };
        let casinos = several(
            action.count,
            &format!("{state} Casino"),
            &format!("{state} Casinos"),
        );
        let expected = if action.closes {
            format!("Syndicate closes {casinos} in place.")
        } else {
            format!("Syndicate returns {casinos} to Available Forces.")
        };
        let mut sentences = vec![expected, "0 Resources · no Profit.".into()];
        sentences.extend(action.effects.iter().cloned());
        same("sentences", &article.lines, &sentences)?;
        let faction = Faction::Syndicate;
        position.add(
            &action.space,
            faction,
            Kind::Casino,
            action.state,
            -action.count,
        );
        if action.closes {
            position.add(
                &action.space,
                faction,
                Kind::Casino,
                State::Closed,
                action.count,
            );
        }
        Ok(())
    }

    fn transfer_lines(pair: (Faction, Faction), told: &Between) -> Vec<String> {
        let (giver, recipient) = (pair.0.name(), pair.1.name());
        let mut expected = vec![];
        let asks = &told.asks[..told.asks.len() - usize::from(told.pending.is_some())];
        match asks {
            [] => {}
            [one] => expected.push(format!("{recipient} asked for {}.", resources(*one))),
            _ => expected.push(format!("{recipient} asked {} times.", asks.len())),
        }
        match told.moves[..] {
            [] => {}
            [[a, b, c, d]] => {
                expected.push(format!("{giver} {a} → {b}, {recipient} {c} → {d}."));
            }
            _ => expected.push(format!("They move in {} transfers.", told.moves.len())),
        }
        let requests = |n| several(n, "request", "requests");
        if told.unmet > 0 {
            expected.push(format!(
                "{giver} answers {} with nothing.",
                requests(told.unmet)
            ));
        }
        if told.withdrawn > 0 {
            expected.push(format!(
                "{recipient} withdraws {}.",
                requests(told.withdrawn)
            ));
        }
        expected
    }

    fn read(
        article: &Article,
        done: &Done,
        after: &Observation,
        position: &mut Position,
        transfers: &mut Vec<&((Faction, Faction), Between)>,
    ) -> Check {
        let lines: Vec<&str> = article.lines.iter().map(String::as_str).collect();
        let title = article.title.as_str();
        let balance = |space: &str| {
            done.executed
                .iter()
                .find(|(known, _)| known == space)
                .map(|(_, balance)| *balance)
                .ok_or("no such space")
        };
        if !title.starts_with("Kidnap") {
            same("die", &article.die, &None)?;
        }
        if let Some(at) = transfers.iter().position(|(pair, told)| {
            title == title_of(*pair, told) && article.lines == transfer_lines(*pair, told)
        }) {
            let (pair, told) = transfers.remove(at);
            for m in &told.moves {
                position.pay(pair.0, m[0] - m[1]);
                position.pay(pair.1, m[2] - m[3]);
            }
            return Ok(());
        }
        if title == "Back to the action choice" {
            let executor = if after.current_card == 18 {
                Faction::Syndicate
            } else {
                Faction::M26
            };
            let expected = done.back.flatten().map_or_else(
                || format!("{} goes back to its action choice.", executor.name()),
                |giver| {
                    format!(
                        "{} withdraws its request to {}.",
                        executor.name(),
                        giver.name()
                    )
                },
            );
            return same("sentences", &article.lines, &vec![expected]);
        }
        if title == "The free Rally" {
            let rallies = format!("{} Rallies in no space.", done.no_rally.unwrap().name());
            return same("sentences", &lines, &vec![rallies.as_str()]);
        }
        if title == "26July's Guerrillas" {
            if done.spaces.is_empty() {
                let none = "No Guerrilla turns Underground → Active.";
                return same("sentences", &lines, &vec![none]);
            }
            for line in lines {
                if line == "No Guerrilla turns Underground → Active." {
                    continue;
                }
                let held = scan(line, "{} in {}.").ok_or("no turn")?;
                turns(held[0], held[1], position)?;
            }
            return Ok(());
        }
        if let Some(held) = scan(title, "Kidnap in {}") {
            same("space", &held[0], &"Santiago de Cuba")?;
            let kidnap = after.kidnap.as_ref().unwrap();
            let taken = i32::from(kidnap.die.min(kidnap.government_before));
            let (g, m) = (
                i32::from(kidnap.government_before),
                i32::from(kidnap.m26_before),
            );
            position.pay(Faction::Government, taken);
            position.pay(Faction::M26, -taken);
            let took = if taken == 0 {
                "Government has no Resources to take.".to_owned()
            } else {
                format!(
                    "26July takes {} from Government: Government {g} → {}, 26July {m} → {}.",
                    resources(taken),
                    g - taken,
                    m + taken
                )
            };
            same("die", &article.die, &Some(kidnap.die))?;
            let die = format!("The die shows {}.", kidnap.die);
            return same("sentences", &article.lines, &vec![die, took]);
        }
        if let Some(held) = scan(title, "Terror in {}") {
            let space = held[0];
            let [payment, turn, rest @ ..] = &lines[..] else {
                return Err("sentences left out".into());
            };
            pays(payment, Faction::M26, balance(space)?, position)?;
            let turn = turn.strip_suffix('.').ok_or("no stop")?;
            turns(turn, space, position)?;
            for line in rest {
                if let Some(held) = scan(line, "{} → {}.") {
                    let now = position.support.get_mut(space).unwrap();
                    same("level", &level(held[0])?, now)?;
                    *now = level(held[1])?;
                } else {
                    let held = scan(line, "{} placed.").ok_or("not a sentence")?;
                    let n = number(held[0].split(' ').next().unwrap())?;
                    let markers = several(n, "Terror marker is", "Terror markers are");
                    same("markers", &held[0].to_owned(), &markers)?;
                    *position.terror.get_mut(space).unwrap() += n;
                }
            }
            return Ok(());
        }
        if let Some(held) = scan(title, "Construct in {}") {
            let space = held[0];
            let constructed = done
                .constructed
                .iter()
                .find(|payment| payment.space == space)
                .ok_or("no Construct payment in this space")?;
            let payer = Faction::Syndicate;
            let from = balance(space)?;
            let expected_payment = format!("Syndicate pays 5 Resources, {from} → {}.", from - 5);
            position.pay(payer, 5);
            let outcome = if constructed.opens {
                position.add(space, payer, Kind::Casino, State::Closed, -1);
                position.add(space, payer, Kind::Casino, State::Open, 1);
                "Syndicate opened 1 closed Casino."
            } else {
                position.add(space, payer, Kind::Casino, State::Closed, 1);
                "Syndicate placed 1 closed Casino."
            };
            let mut expected = vec![expected_payment, outcome.to_owned()];
            expected.extend(constructed.effects.iter().cloned());
            return same("sentences", &article.lines, &expected);
        }
        const NOTHING: &str = "No piece is removed or closed.";
        if scan(title, "Assault in {} spaces").is_some() {
            // R 6.3.1: Firm costs 2 in each.
            let n = done.unchanged.len() as i32;
            position.pay(Faction::Government, 2 * n);
            let expected = vec![
                NOTHING.to_owned(),
                format!(
                    "Government pays 2 Resources in each of {}.",
                    listing(&done.unchanged)
                ),
            ];
            return same("sentences", &article.lines, &expected);
        }
        if let Some(held) = scan(title, "Assault in {}") {
            let space = held[0];
            let [payment, rest @ ..] = &lines[..] else {
                return Err("sentences left out".into());
            };
            pays(payment, Faction::Government, balance(space)?, position)?;
            if done.unchanged.iter().any(|known| known == space) {
                return same("sentences", &rest.to_vec(), &vec![NOTHING]);
            }
            for line in rest {
                if let Some(held) = scan(line, "Syndicate: {} in place.") {
                    let n = number(held[0].split(' ').next().unwrap())?;
                    let closes = several(n, "Casino closes", "Casinos close");
                    same("Casinos", &held[0].to_owned(), &closes)?;
                    let of = (Faction::Syndicate, Kind::Casino);
                    position.add(space, of.0, of.1, State::Open, -n);
                    position.add(space, of.0, of.1, State::Closed, n);
                } else {
                    let held = scan(line, "{}: {} return{} to Available Forces.")
                        .ok_or("not a sentence")?;
                    let (n, kind, state) = group(held[1])?;
                    same("return", &held[2], &if n == 1 { "s" } else { "" })?;
                    position.add(space, faction(held[0])?, kind, state, -n);
                }
            }
            return Ok(());
        }
        if let Some(held) = scan(title, "{} removes its own pieces") {
            let owner = faction(held[0])?;
            for line in lines {
                let held = scan(line, "{}: {}.").ok_or("not a sentence")?;
                for one in held[1].split(", ").flat_map(|part| part.split(" and ")) {
                    let (n, kind, state) = group(one)?;
                    position.add(held[0], owner, kind, state, -n);
                }
            }
            return Ok(());
        }
        if let Some(held) = scan(title, "{} Rallies in {}") {
            let (owner, space) = (faction(held[0])?, held[1]);
            let [line] = lines[..] else {
                return Err("one sentence is due".into());
            };
            if let Some(held) = scan(line, "{} placed from Available Forces.") {
                let n = number(held[0].split(' ').next().unwrap())?;
                let placed = several(n, "Underground Guerrilla is", "Underground Guerrillas are");
                same("placed", &held[0].to_owned(), &placed)?;
                position.add(space, owner, Kind::Guerrilla, State::Underground, n);
                return Ok(());
            }
            if line != "No piece moves." {
                turns(line.strip_suffix('.').ok_or("no stop")?, space, position)?;
            }
            return Ok(());
        }
        Err("not a title".into())
    }

    /// R 1.7: the faction whose pieces, closed Casinos apart, exceed all others' in a City
    /// or Province.
    fn controls(space: &Space) -> Option<Faction> {
        let count = |faction: Faction| -> i32 {
            space
                .pieces
                .iter()
                .filter(|p| p.faction == faction && p.state != State::Closed)
                .map(|p| i32::from(p.count))
                .sum()
        };
        let all: i32 = Faction::ALL.into_iter().map(count).sum();
        Faction::ALL
            .into_iter()
            .filter(|_| space.terrain != Terrain::EconomicCenter)
            .find(|f| count(*f) > all - count(*f))
    }

    /// R 1.6.3, 7.2 and 7.3: what `faction`'s victory counts, and its margin.
    fn standing(seen: &Observation, faction: Faction) -> (i16, i16) {
        let mut score = 0;
        for space in &seen.spaces {
            let (population, support) = (i16::from(space.population), i16::from(space.support));
            score += match faction {
                Faction::Government => population * support.max(0),
                Faction::M26 => population * (-support).max(0),
                Faction::Directorio if controls(space) == Some(faction) => population,
                _ => 0,
            };
            for p in space.pieces.iter().filter(|p| p.faction == faction) {
                let counts = match faction {
                    Faction::Government => false,
                    Faction::Syndicate => p.kind == Kind::Casino && p.state == State::Open,
                    _ => p.kind == Kind::Base,
                };
                score += if counts { i16::from(p.count) } else { 0 };
            }
        }
        let margin = match faction {
            Faction::Government => score - 18,
            Faction::M26 => score - 15,
            Faction::Directorio => score - 9,
            Faction::Syndicate => {
                (score - 7).min(i16::from(seen.resources[Faction::Syndicate.index()]) - 30)
            }
        };
        (score, margin)
    }

    /// Compute the consequences of this decision from the observed pieces and R 1.7/7.2.
    fn casino_words(before: &Observation, after: &Observation, id: &str) -> Vec<String> {
        let (earlier, _) = standing(before, Faction::Syndicate);
        let (later, _) = standing(after, Faction::Syndicate);
        let mut expected = vec![];
        if earlier != later {
            expected.push(format!("Syndicate open Casinos: {earlier} → {later}."));
        }
        let space = after.spaces.iter().find(|s| s.id == id).unwrap();
        let before = before.spaces.iter().find(|s| s.id == id).unwrap();
        let named = |f: Option<Faction>| f.map_or("none", Faction::name);
        let (from, to) = (controls(before), controls(space));
        if from != to {
            expected.push(format!(
                "{} Control: {} → {}.",
                space.name,
                named(from),
                named(to)
            ));
        }
        expected
    }

    fn notes(
        before: &Observation,
        after: &Observation,
        executor: Faction,
        recorded: Option<&str>,
        complete: bool,
    ) -> Vec<(String, Vec<String>)> {
        let name = |f: Option<Faction>| f.map_or("none", Faction::name);
        let control: Vec<String> = before
            .spaces
            .iter()
            .zip(&after.spaces)
            .map(|(was, is)| (controls(was), controls(is), &is.name))
            .filter(|(was, is, _)| was != is)
            .map(|(was, is, space)| format!("{space}: {} → {}.", name(was), name(is)))
            .collect();
        // R 1.4.1: what is not on the map is Available, out of the chart on p. 20.
        let chart = [
            (Faction::Government, Kind::Troops, "Troops", 15),
            (Faction::Government, Kind::Police, "Police cubes", 15),
            (Faction::Government, Kind::Base, "Bases", 2),
            (Faction::M26, Kind::Guerrilla, "Guerrillas", 15),
            (Faction::M26, Kind::Base, "Bases", 4),
            (Faction::Directorio, Kind::Guerrilla, "Guerrillas", 15),
            (Faction::Directorio, Kind::Base, "Bases", 4),
            (Faction::Syndicate, Kind::Guerrilla, "Guerrillas", 6),
            (Faction::Syndicate, Kind::Casino, "Casinos", 10),
        ];
        let left = |seen: &Observation, f, k, all: i32| -> i32 {
            let placed: i32 = seen
                .spaces
                .iter()
                .flat_map(|s| &s.pieces)
                .filter(|p| (p.faction, p.kind) == (f, k))
                .map(|p| i32::from(p.count))
                .sum();
            all - placed
        };
        let available: Vec<String> = chart
            .into_iter()
            .map(|(f, k, words, all)| (f, words, left(before, f, k, all), left(after, f, k, all)))
            .filter(|(_, _, was, is)| was != is)
            .map(|(f, words, was, is)| format!("{} {words}: {was} → {is}.", f.name()))
            .collect();
        // R 3.3.4: there are 20.
        let supply = |seen: &Observation| -> i32 {
            20 - seen.spaces.iter().map(|s| i32::from(s.terror)).sum::<i32>()
        };
        let (was, is) = (supply(before), supply(after));
        let markers = if was == is {
            vec![]
        } else {
            vec![format!(
                "Terror and Sabotage markers available: {was} → {is} of 20."
            )]
        };
        let totals: Vec<String> = [
            (Faction::Government, "Government Total Support"),
            (Faction::M26, "26July Total Opposition + Bases"),
            (
                Faction::Directorio,
                "Directorio Controlled Population + Bases",
            ),
            (Faction::Syndicate, "Syndicate open Casinos"),
        ]
        .into_iter()
        .map(|(f, words)| (words, standing(before, f).0, standing(after, f).0))
        .filter(|(_, was, is)| was != is)
        .map(|(words, was, is)| format!("{words}: {was} → {is}."))
        .collect();
        // R 2.3.3, 2.3.4 and 2.3.7, and the shaded text of card 17.
        let mut eligibility: Vec<String> = vec![];
        if let Some(recorded) = recorded
            && executor != Faction::Syndicate
        {
            eligibility.push(format!("{} is in the {recorded} box.", executor.name()));
        }
        let mut say = |line: &str| eligibility.push(line.to_owned());
        match (complete, executor, recorded) {
            (false, Faction::Syndicate, _)
                if before.executing == Some(Faction::Syndicate) && after.executing.is_none() =>
            {
                say("Next: Syndicate, 2nd Eligible.");
            }
            (false, ..) => {}
            (_, Faction::M26, Some("Pass")) => {
                say("26July is Eligible for the next card.");
                say("Next: Directorio, 1st Eligible.");
            }
            (_, Faction::M26, _) => {
                say("26July is Ineligible for the next card.");
                say("Next: Directorio, 2nd Eligible.");
            }
            (_, Faction::Directorio, _) => {
                say(
                    "Card 18: Government and Syndicate Eligible, 26July and Directorio Ineligible.",
                );
                if matches!(
                    after.event.as_ref().and_then(|e| e.outcome()),
                    Some(EventOutcome::Shaded { .. })
                ) {
                    say("The Event keeps 26July Ineligible through card 18.");
                }
            }
            (_, Faction::Syndicate, Some("Limited Operation")) => {
                say(
                    "Card 19: 26July and Directorio Eligible, Government and Syndicate Ineligible.",
                );
            }
            (_, Faction::Syndicate, Some("Operation Only")) => {
                say("Card 19: Government, 26July and Directorio Eligible, Syndicate Ineligible.");
            }
            (_, Faction::Syndicate, _) => {
                if before.passed[Faction::Government.index()] {
                    say("All four factions are Eligible for card 19.");
                } else {
                    say(
                        "Card 19: 26July, Directorio and Syndicate Eligible, Government Ineligible.",
                    );
                }
            }
            (_, _, Some("Pass")) => say("Next: Syndicate, 1st Eligible."),
            _ => say("Next: Syndicate, 2nd Eligible."),
        }
        [
            ("Control", control),
            ("Available Forces", available),
            ("Markers", markers),
            ("Victory totals", totals),
            ("Eligibility", eligibility),
        ]
        .into_iter()
        .filter(|(_, lines)| !lines.is_empty())
        .map(|(title, lines)| (title.to_owned(), lines))
        .collect()
    }
}
