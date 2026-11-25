//! Seam 1 of #197: which action reports the human is given as editions, and when (#208).
//! Who acts in each stretch of a line, and so what is told, is written here by hand from
//! the Sequence of Play (R 2.3), never taken from the app's reports.
use crate::{SEED, explore::choose};
use Faction::{Directorio, Government, M26, Syndicate};
use cuba_libre_unfolded::{
    Choice, Faction, Game, Observation, Transfer, editions::Editions, reports::Reports,
};

/// A match at a table, with the factions the human plays.
struct Table {
    game: Game,
    reports: Reports,
    editions: Editions,
    plays: [bool; 4],
}

impl Table {
    fn new(humans: &[Faction]) -> Self {
        let game = Game::opening_with_seed(SEED);
        let seen = game.observe();
        Self {
            game,
            reports: Reports::new(&seen),
            editions: Editions::from(&seen),
            plays: Faction::ALL.map(|faction| humans.contains(&faction)),
        }
    }
    fn seen(&self) -> Observation {
        self.game.observe()
    }
    fn play(&mut self, actions: &[&str]) {
        for action in actions {
            choose(&mut self.game, action);
            let seen = self.game.observe();
            self.reports.record(&seen);
            let reports = &self.reports;
            self.editions
                .decided(&seen, |told| reports.since(told), self.plays);
        }
    }
    /// The human hands `faction` over, or takes it back.
    fn seat(&mut self, faction: Faction, human: bool) {
        self.plays[faction.index()] = human;
        let seen = self.game.observe();
        let reports = &self.reports;
        self.editions
            .seated(&seen, |told| reports.since(told), self.plays);
    }
    /// Reads what is due: each one's faction, whether it is whole, its revisions and its
    /// headline.
    fn read(&mut self) -> Vec<(Faction, bool, [u64; 2], String)> {
        let mut read = vec![];
        while self.editions.due() {
            assert!(self.editions.holds());
            self.editions.open_next();
            let (open, edition) = self.editions.open().unwrap();
            assert!(!open.reopened);
            let report = edition.report.clone();
            read.push((
                report.faction,
                report.complete,
                [report.from_revision, report.to_revision],
                report.headline,
            ));
            assert!(self.editions.holds());
            self.editions.put_away();
        }
        assert!(!self.editions.holds());
        read
    }
    /// Reads the one edition due: its faction, whether it is whole and its headline.
    fn one(&mut self) -> (Faction, bool, [u64; 2], String) {
        let mut read = self.read();
        assert_eq!(read.len(), 1, "{read:?}");
        read.remove(0)
    }
    fn none(&mut self) {
        assert_eq!(self.read(), []);
    }
}

/// Terror in Oriente, paid for by the Syndicate: five decisions.
const FUNDED: [&str; 5] = [
    "terror",
    "transfer",
    "ask:3:2",
    "authorize:2",
    "terror:oriente",
];

type Line = Vec<(Faction, Vec<&'static str>)>;

/// Lines of play, each stretch with the faction whose action it is. Every stretch ends as
/// its action does, and turning the card is in no action.
fn lines() -> Vec<Line> {
    let terror = |inside: &[&'static str]| {
        let mut line = vec!["terror", "kidnap", "kidnap:roll", "terror:santiago"];
        line.extend(inside);
        line.push("end-terror");
        (M26, line)
    };
    // 26July has no Resources to begin with.
    let with = |then: &[&'static str]| [&FUNDED[..], then].concat();
    let asks = [
        "transfer",
        "ask:0:2",
        "decline",
        "transfer",
        "ask:3:2",
        "authorize:1",
    ];
    let turn = || (Syndicate, vec!["advance-card"]);
    vec![
        vec![(M26, vec!["pass"])],
        vec![(M26, with(&["end-terror"]))],
        vec![
            terror(&asks),
            (
                Directorio,
                vec![
                    "event:shaded",
                    "transfer",
                    "ask:1:1",
                    "wait",
                    "reopen",
                    "authorize:1",
                    "resolve-shaded",
                ],
            ),
            turn(),
            (
                Government,
                vec![
                    "government:assault",
                    "assault:havana:syndicate",
                    "transfer",
                    "ask:1:1",
                    "authorize:1",
                    "end-assault",
                ],
            ),
        ],
        vec![
            terror(&[]),
            (
                Directorio,
                vec![
                    "event:unshaded",
                    "rally:beneficiary:26july",
                    "rally:sierra-maestra:place:2",
                ],
            ),
            turn(),
            (Government, vec!["government:pass"]),
        ],
        vec![
            terror(&[]),
            (
                Directorio,
                vec!["event:unshaded", "rally:beneficiary:directorio"],
            ),
            turn(),
            (Government, vec!["government:assault", "end-assault"]),
        ],
    ]
}

/// Whoever the human plays, all sixteen ways: one edition when each action of another
/// faction ends, one each time the human must decide inside it, none for their own, and
/// each starts where the one before it ended.
#[test]
fn an_edition_for_each_action_of_another_faction_and_each_decision_inside_it() {
    let (mut whole, mut partway) = (0, 0);
    for line in lines() {
        for humans in 0..16_u8 {
            let humans: Vec<Faction> = Faction::ALL
                .into_iter()
                .filter(|faction| humans & (1 << faction.index()) != 0)
                .collect();
            let mut table = Table::new(&humans);
            for (actor, actions) in &line {
                let turning = actions == &["advance-card"];
                let mut from = table.seen().revision;
                let mut asked = false;
                for (i, action) in actions.iter().enumerate() {
                    table.play(&[action]);
                    let seen = table.seen();
                    let ended = i + 1 == actions.len();
                    let humans_turn = humans.contains(&seen.owner) && !ended;
                    let due =
                        !turning && !humans.contains(actor) && (ended || (humans_turn && !asked));
                    asked = humans_turn;
                    let what = format!("{humans:?} {actor:?} {action}");
                    if !due {
                        assert_eq!(table.read(), [], "{what}");
                        continue;
                    }
                    let (faction, complete, revisions, _) = table.one();
                    assert_eq!((faction, complete), (*actor, ended), "{what}");
                    assert_eq!(revisions, [from, seen.revision], "{what}");
                    from = seen.revision;
                    *(if ended { &mut whole } else { &mut partway }) += 1;
                }
            }
        }
    }
    // Eleven actions, each another faction's in 8 of the 16 ways. A decision inside one is
    // the human's, and the action another's, in 4: Terror's two requests to two givers,
    // the Syndicate's that pays for Terror
    // without Kidnap, and 26July's three (in the Event, its Rally and in the Assault).
    assert_eq!((whole, partway), (11 * 8, 6 * 4));
}

/// A request the human sets aside and takes up again is no new decision; another request
/// is.
#[test]
fn a_request_set_aside_is_announced_once() {
    let mut table = Table::new(&[Government]);
    table.play(&["terror", "transfer", "ask:0:3"]);
    table.one();
    table.play(&["wait"]);
    table.none();
    table.play(&["reopen"]);
    table.none();
    table.play(&["authorize:2", "transfer"]);
    table.none();
    table.play(&["ask:0:1"]);
    let (faction, complete, revisions, headline) = table.one();
    assert_eq!((faction, complete, revisions), (M26, false, [3, 8]));
    assert_eq!(headline, "26July continues Terror");
}

/// A faction handed over in its action is told from there: what the human did with it is
/// theirs. One taken back is theirs, and what the agent began is not told.
#[test]
fn a_faction_that_changes_hands_is_whoevers_it_is_then() {
    let mut table = Table::new(&[M26, Government, Syndicate]);
    table.play(&FUNDED);
    table.none();
    table.seat(M26, false);
    table.none();
    table.play(&["terror:santiago"]);
    table.none();
    table.play(&["end-terror"]);
    let (faction, complete, revisions, headline) = table.one();
    assert_eq!((faction, complete, revisions), (M26, true, [5, 7]));
    assert_eq!(headline, "26July ends Terror: Santiago de Cuba");

    let mut table = Table::new(&[Government]);
    table.play(&FUNDED);
    table.seat(M26, true);
    table.none();
    // Handed over again with nothing decided, the agent's beginning is still told.
    table.seat(M26, false);
    table.play(&["terror:santiago", "end-terror"]);
    assert_eq!(table.one().2, [0, 7]);

    let mut table = Table::new(&[Government]);
    table.play(&FUNDED);
    table.seat(M26, true);
    table.play(&["terror:santiago", "end-terror"]);
    table.none();
    assert_eq!(table.editions.pile(), 0);
}

/// Taking back the giver that an agent's faction asked puts the decision before the human,
/// and its edition; handing the asking faction over does not tell the human's own request.
#[test]
fn a_seat_taken_back_can_put_a_decision_before_the_human() {
    let mut table = Table::new(&[Syndicate]);
    table.play(&["terror", "transfer", "ask:0:2"]);
    table.none();
    assert!(matches!(
        table.seen().transfer,
        Some(Transfer::Authorizing { .. })
    ));
    table.seat(Government, true);
    let (faction, complete, revisions, _) = table.one();
    assert_eq!((faction, complete, revisions), (M26, false, [0, 3]));
    table.seat(Government, false);
    table.seat(Government, true);
    // Nothing happened since.
    table.none();

    let mut table = Table::new(&[M26, Government]);
    table.play(&["terror", "transfer", "ask:0:2"]);
    table.seat(M26, false);
    table.none();
}

/// 26July is the human's to Rally inside Directorio's Event, which is Directorio's all the
/// same (#197, correction 12).
#[test]
fn the_event_is_directorios_while_the_human_rallies() {
    let mut table = Table::new(&[M26, Government, Syndicate]);
    table.play(&lines()[3][0].1);
    table.none();
    table.play(&["event:unshaded"]);
    table.none();
    table.play(&["rally:beneficiary:26july"]);
    assert!(!table.one().1, "partway");
    table.play(&["rally:sierra-maestra:place:2"]);
    let (faction, complete, ..) = table.one();
    assert_eq!((faction, complete), (Directorio, true));
    assert!(table.seen().choices.iter().any(Choice::turns_the_card));
}

/// A match opens with nothing due: an edition unread at Save · quit is not shown again,
/// and what led there is not told later.
#[test]
fn a_match_that_opens_has_no_edition_of_what_led_there() {
    let mut table = Table::new(&[Government]);
    table.play(&FUNDED);
    table.editions = Editions::from(&table.seen());
    table.none();
    table.play(&["terror:santiago", "end-terror"]);
    assert_eq!(table.one().2, [5, 7]);
}

/// The current card's editions are read again, the latest first, and leave with the card.
/// Nothing is reopened before the first was put away.
#[test]
fn the_cards_editions_are_reopened() {
    let mut table = Table::new(&[Government]);
    table.play(&lines()[2][0].1[..4]);
    table.play(&["transfer", "ask:0:2"]);
    let editions = &mut table.editions;
    editions.reopen();
    assert!(editions.open().is_none() && editions.pile() == 0);
    editions.open_next();
    assert_eq!(editions.pile(), 0, "the open one is not on the pile yet");
    // The one just out stays: no tab turns it to another.
    editions.show(1);
    editions.step(false);
    assert!(!editions.open().unwrap().0.reopened);
    editions.put_away();
    table.play(&["decline", "end-terror"]);
    let editions = &mut table.editions;
    editions.open_next();
    assert_eq!(editions.open().unwrap().1.number, 2);
    assert_eq!(editions.pile(), 1);
    editions.put_away();
    assert_eq!(editions.pile(), 2);
    let number = |editions: &Editions| {
        let (open, edition) = editions.open().unwrap();
        assert!(open.reopened);
        edition.number
    };
    editions.reopen();
    assert_eq!(number(editions), 2);
    assert_eq!(editions.pile(), 2, "a reopened one is still of the pile");
    assert!(editions.holds());
    for (back, expected) in [(false, 1), (false, 2), (true, 1), (true, 2)] {
        editions.step(back);
        assert_eq!(number(editions), expected);
    }
    editions.show(1);
    assert_eq!(number(editions), 1);
    for wrong in [0, 3] {
        editions.show(wrong);
        assert_eq!(number(editions), 1);
    }
    // Its pages: no further than the last, nor before the first.
    let page = |editions: &Editions| editions.open().unwrap().0.page;
    for (back, expected) in [(true, 0), (false, 1), (false, 1), (true, 0)] {
        editions.turn(back, 2);
        assert_eq!(page(editions), expected);
    }
    editions.turn(false, 2);
    editions.show(2);
    assert_eq!(page(editions), 0);
    editions.put_away();
    assert!(!editions.holds());
    let numbers: Vec<usize> = editions.latest_first().map(|e| e.number).collect();
    assert_eq!(numbers, [2, 1]);

    table.play(&["event:shaded", "resolve-shaded"]);
    table.one();
    table.play(&["advance-card"]);
    table.none();
    assert_eq!(table.editions.pile(), 0);
    table.editions.reopen();
    assert!(table.editions.open().is_none());
}
