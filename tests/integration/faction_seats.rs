//! Seam 1 of #197: the faction seat host with no window, socket or model. A session is
//! what it sends and what it is answered; the human is a checked submission of their own.
//! Expectations come from #203, #197's corrections and the rules, worked by hand: the
//! match opens at the prepared position (Resources Government 6, 26July 0, Directorio 5,
//! Syndicate 15), and seed 5's Kidnap die is a 3.

use crate::explore::{self, parallel};
use cuba_libre_unfolded::{
    Faction, Game,
    faction_seats::{Answer, Brief, Host, Player, SeatState, Stands, Table, View},
    is_propaganda,
    reports::{Report, Reports},
    save::Session,
    title,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use Faction::{Directorio, Government, M26, Syndicate};

/// The table a session plays at: a real save, and whether the table is still.
struct Room {
    session: Session,
    settled: bool,
    /// Each commit of a session's, with the decisions on disk as it was made.
    commits: Vec<(Faction, String, Vec<String>)>,
    path: PathBuf,
}

impl Table for Room {
    fn game(&self) -> &Game {
        self.session.game()
    }
    fn reports(&self) -> Vec<Report> {
        self.session.reports()
    }
    fn settled(&self) -> bool {
        self.settled
    }
    fn commit(&mut self, faction: Faction, revision: u64, choice: &str) -> Result<(), String> {
        self.session
            .submit(faction, revision, choice)
            .map_err(|refused| refused.to_string())?;
        self.commits
            .push((faction, choice.to_owned(), decisions(&self.path)));
        Ok(())
    }
}

fn decisions(path: &Path) -> Vec<String> {
    let record: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    serde_json::from_value(record["payload"]["decisions"].clone()).unwrap()
}

struct Sitting {
    host: Host<u32>,
    room: Room,
    now: Instant,
    sent: u32,
    answers: BTreeMap<u32, Value>,
    _dir: tempfile::TempDir,
}

impl Sitting {
    /// A new match of seed 5, whose Kidnap die is a 3.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("match.json");
        Self {
            host: Host::new(),
            room: Room {
                session: Session::create_seeded(&path, 5).unwrap(),
                settled: true,
                commits: vec![],
                path,
            },
            now: Instant::now(),
            sent: 0,
            answers: BTreeMap::new(),
            _dir: dir,
        }
    }
    /// A new match played by the human to Directorio's handoff, at revision 5.
    fn at_the_handoff() -> Self {
        let mut sitting = Self::new();
        for action in HANDOFF {
            sitting.human(action);
        }
        sitting
    }
    fn collect(&mut self, answers: Vec<Answer<u32>>) {
        for (reply, answer) in answers {
            assert_hidden(&answer, self.room.session.game());
            let twice = self.answers.insert(reply, answer);
            assert!(twice.is_none(), "answered twice: {twice:?}");
        }
    }
    fn send_line(&mut self, line: &str) -> u32 {
        self.sent += 1;
        let answers = self.host.receive(line, self.sent, &mut self.room, self.now);
        self.collect(answers);
        self.sent
    }
    fn send(&mut self, request: Value) -> u32 {
        self.send_line(&request.to_string())
    }
    fn answer(&mut self, sent: u32) -> Option<Value> {
        self.answers.remove(&sent)
    }
    /// The answer to a request the host answers at once.
    fn ask(&mut self, request: Value) -> Value {
        let sent = self.send(request);
        self.answer(sent).expect("answered at once")
    }
    fn poll(&mut self) {
        let answers = self.host.poll(&mut self.room, self.now);
        self.collect(answers);
    }
    /// Let this much time pass.
    fn pass(&mut self, seconds: u64) {
        self.now += Duration::from_secs(seconds);
        self.poll();
    }
    /// Hand `faction` over and join as it: the session's identity.
    fn seat(&mut self, faction: Faction) -> String {
        self.host.hand_over(faction);
        let joined = self.ask(json!({"op":"join","faction":faction.name()}));
        assert_eq!(joined["status"], "joined", "{joined}");
        assert_eq!(joined["view"]["faction"], json!(faction));
        joined["player"].as_str().unwrap().to_owned()
    }
    fn take_back(&mut self, faction: Faction) {
        let answers = self.host.take_back(faction);
        self.collect(answers);
    }
    fn observe(&mut self, player: &str) -> Value {
        let seen = self.ask(json!({"op":"observe","player":player}));
        assert_eq!(seen["status"], "observed", "{seen}");
        seen["view"].clone()
    }
    fn choice(&self, player: &str, action: &str) -> Value {
        let revision = self.revision();
        json!({
            "op":"choose","player":player,"revision":revision,"id":format!("{revision}:{action}"),
        })
    }
    fn choose(&mut self, player: &str, action: &str) -> Value {
        let request = self.choice(player, action);
        self.ask(request)
    }
    /// Submit, then wait: the request, held or answered.
    fn choose_and_wait(&mut self, player: &str, action: &str) -> u32 {
        let mut request = self.choice(player, action);
        request["wait"] = json!(true);
        self.send(request)
    }
    fn applied(&mut self, player: &str, action: &str) -> Value {
        let answer = self.choose(player, action);
        assert_eq!(answer["status"], "applied", "{action}: {answer}");
        answer
    }
    fn wait(&mut self, player: &str) -> u32 {
        self.send(json!({"op":"wait","player":player}))
    }
    /// The human's own decision, as whichever faction the rules offer it to.
    fn human(&mut self, action: &str) {
        let id = format!("{}:{action}", self.revision());
        let seen = self.room.session.observe();
        let owner = seen.choices.iter().find(|c| c.id == id).map(|c| c.owner);
        let owner = owner.unwrap_or_else(|| panic!("{id} is not offered"));
        self.room.session.submit(owner, seen.revision, &id).unwrap();
        self.poll();
    }
    fn revision(&self) -> u64 {
        self.room.session.observe().revision
    }
    fn state(&self, faction: Faction) -> SeatState {
        self.host.state(faction, self.room.session.game(), self.now)
    }
    /// Everything a refusal must leave as it was.
    fn standing(&self) -> (Game, Vec<u8>, u64, [Player; 4]) {
        (
            self.room.session.game().clone(),
            std::fs::read(&self.room.path).unwrap(),
            self.room.session.game().random_state(),
            self.host.players(),
        )
    }
    fn refused(&mut self, request: Value) -> Value {
        let before = self.standing();
        let answer = self.ask(request.clone());
        assert_eq!(answer["status"], "refused", "{request}: {answer}");
        assert_eq!(answer["ok"], false);
        assert_eq!(self.standing(), before, "{request} changed something");
        answer
    }
}

const HANDOFF: [&str; 5] = [
    "terror",
    "kidnap",
    "kidnap:roll",
    "terror:santiago",
    "end-terror",
];

fn played(seed: u64, actions: &[&str]) -> Game {
    let mut game = Game::opening_with_seed(seed);
    for action in actions {
        explore::choose(&mut game, action);
    }
    game
}

/// The actions a view offers.
fn actions(view: &Value) -> Vec<String> {
    view["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap().split_once(':').unwrap().1.into())
        .collect()
}

fn error(answer: &Value) -> &str {
    answer["error"].as_str().unwrap()
}

// --- What is hidden -----------------------------------------------------------------

/// The same match with another generator's state and another order of the cards under
/// the preview: what nobody at the table may know.
fn unseen_changed(game: &Game) -> Game {
    let mut record = serde_json::to_value(game).unwrap();
    let other = |state: &Value| json!(state.as_u64().unwrap() ^ 0x5a5a_5a5a_5a5a);
    record["random"] = other(&record["random"]);
    if !record["kidnap"].is_null() {
        record["kidnap"]["random_before"] = other(&record["kidnap"]["random_before"]);
    }
    record["draw_deck"].as_array_mut().unwrap()[1..].reverse();
    serde_json::from_value(record).unwrap()
}

/// Every object key that occurs in a view. A new field, nested or not, is not told to an
/// agent until it is named here.
const TOLD: &[&str] = &[
    "Authorizing",
    "AwaitingOperation",
    "Closed",
    "Complete",
    "InProgress",
    "InsurgentCapability",
    "Open",
    "Place",
    "Preview",
    "Rally",
    "Resolved",
    "Shaded",
    "Unshaded",
    "accompanies",
    "action",
    "affects",
    "after",
    "aid",
    "alliance",
    "amount",
    "arithmetic",
    "articles",
    "assault",
    "available",
    "balance",
    "beneficiary",
    "bound",
    "capabilities",
    "card",
    "card_title",
    "cards",
    "cash_available",
    "choice",
    "choices",
    "commits_santiago",
    "complete",
    "construct_paid",
    "continues",
    "contributions",
    "control",
    "count",
    "current",
    "current_card",
    "decision",
    "deck",
    "die",
    "earliest",
    "economy",
    "edition",
    "effect",
    "effected",
    "eligibility",
    "eligible",
    "event",
    "event_executor",
    "event_restriction",
    "executed",
    "executing",
    "explanation",
    "extra",
    "face_down",
    "faction",
    "factions",
    "first",
    "flips",
    "from",
    "from_revision",
    "giver",
    "government",
    "government_box",
    "headline",
    "held",
    "id",
    "in_effect",
    "kidnap",
    "kidnap_open",
    "kidnap_stakes",
    "kind",
    "label",
    "lasting",
    "latest",
    "limit",
    "lines",
    "m26_action_recorded",
    "m26_box",
    "m26_next_card",
    "margin",
    "margin_from",
    "margin_to",
    "marker_supply",
    "minimum",
    "momentum",
    "name",
    "next",
    "next_card",
    "next_propaganda",
    "note",
    "notes",
    "number",
    "operation",
    "options",
    "order",
    "owed_terror",
    "paid",
    "passed",
    "pieces",
    "piles",
    "plan",
    "played",
    "played_by",
    "population",
    "preview",
    "preview_card",
    "proceeds",
    "propaganda",
    "propaganda_cards",
    "propaganda_left",
    "propaganda_played",
    "read_at",
    "reason",
    "recipient",
    "recorded",
    "records",
    "remedies",
    "removal",
    "removed",
    "reserved",
    "resolutions",
    "resources",
    "results",
    "revision",
    "role",
    "room",
    "rules",
    "sabotage_on_map",
    "score",
    "score_from",
    "score_to",
    "second",
    "site",
    "space",
    "space_reasons",
    "spaces",
    "special_activity",
    "standings",
    "stands",
    "state",
    "statement",
    "status",
    "summary",
    "support",
    "support_level",
    "supported",
    "target",
    "terrain",
    "terror",
    "terror_on_map",
    "text",
    "texts",
    "threshold",
    "through_card",
    "title",
    "to",
    "to_revision",
    "transfer",
    "transfer_terms",
    "unavailable",
    "unpayable",
    "waited",
    "words",
    "yours",
];

fn keys(value: &Value, found: &mut BTreeSet<String>) {
    match value {
        Value::Object(fields) => {
            for (key, field) in fields {
                found.insert(key.clone());
                keys(field, found);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| keys(item, found)),
        _ => {}
    }
}

/// No text or number of `value` is a generator state of `game`, no list of it holds two
/// cards that lie next to each other face down, and no text of it names an Event card that
/// lies face down.
fn assert_hidden(value: &Value, game: &Game) {
    let record = serde_json::to_value(game).unwrap();
    let mut secrets = vec![record["random"].as_u64().unwrap()];
    secrets.extend(record["kidnap"]["random_before"].as_u64());
    let text = value.to_string();
    // A seed as small as a test's would be found in any list of numbers; the states
    // after a roll are large.
    for secret in secrets.into_iter().filter(|secret| *secret > 1_000_000) {
        assert!(!text.contains(&secret.to_string()), "a generator state");
    }
    let deck: Vec<u64> = serde_json::from_value(record["draw_deck"].clone()).unwrap();
    assert_no_order(value, &deck[1..]);
    for card in deck[1..].iter().map(|card| *card as u8) {
        assert!(
            is_propaganda(card) || !text.contains(title(card)),
            "face-down card {card} is named"
        );
    }
    let mut told = BTreeSet::new();
    keys(value, &mut told);
    for hidden in ["random", "random_before", "draw_deck", "seed", "decisions"] {
        assert!(!told.contains(hidden), "{hidden} is told");
    }
}

fn assert_no_order(value: &Value, face_down: &[u64]) {
    match value {
        Value::Array(items) => {
            let numbers: Vec<u64> = items.iter().filter_map(Value::as_u64).collect();
            for pair in numbers.windows(2) {
                assert!(
                    !face_down.windows(2).any(|cards| cards == pair),
                    "face-down cards are told in order: {value}"
                );
            }
            items.iter().for_each(|i| assert_no_order(i, face_down));
        }
        Value::Object(fields) => fields.values().for_each(|f| assert_no_order(f, face_down)),
        _ => {}
    }
}

/// A match's positions, from its opening to the one it stands at.
pub type History = Vec<Game>;

fn histories(seed: u64, actions: &[&str]) -> Vec<History> {
    (0..=actions.len())
        .map(|made| {
            (0..=made)
                .map(|made| played(seed, &actions[..made]))
                .collect()
        })
        .collect()
}

/// Matches that stand on card 18, at each end of the build and on the way there, with
/// every position since their opening.
pub fn later_positions() -> Vec<History> {
    let mut found = vec![];
    for text in [
        &["event:shaded", "resolve-shaded"][..],
        &["event:unshaded", "rally:beneficiary:directorio"],
        &[
            "event:unshaded",
            "rally:beneficiary:26july",
            "rally:sierra-maestra:place:4",
        ],
    ] {
        for actions in [
            &["government:pass"][..],
            &[
                "government:assault",
                "transfer",
                "ask:1:1",
                "wait",
                "reopen",
                "decline",
                "remove",
                "done-removal",
                "assault:santiago:26july",
                "end-assault",
            ],
            &[
                "government:assault",
                "assault:havana:syndicate",
                "end-assault",
                "syndicate:limited-operation",
                "construct:havana:place",
                "end-construct",
            ],
        ] {
            let path = [&HANDOFF[..], text, &["advance-card"], actions].concat();
            found.extend(histories(5, &path));
        }
    }
    found.extend(histories(5, &["pass"]));
    found.extend(histories(5, &["terror", "end-terror"]));
    found
}

/// The explored positions, each as if its match opened there, and the later ones.
fn positions() -> Vec<History> {
    let mut all: Vec<History> = explore::all_explored()
        .flat_map(|explored| &explored.nodes)
        .map(|node| vec![node.game.clone()])
        .collect();
    all.extend(later_positions());
    all
}

/// The reports of a match that stood at each of `history` in turn.
fn reports(history: &[Game]) -> Vec<Report> {
    let mut reports = Reports::new(&history[0].observe());
    for game in &history[1..] {
        reports.record(&game.observe());
    }
    reports.all()
}

const PLAYERS: [Player; 4] = [Player::Human, Player::Agent, Player::Nobody, Player::Agent];

/// Every view of every explored position is the same whatever lies face down and
/// whatever the generator's state is, holds neither, and names only listed fields.
/// Together the four views carry every choice the rules offer but turning the card.
#[test]
fn no_view_depends_on_what_is_hidden_and_together_they_carry_every_choice() {
    let positions = positions();
    let found: Vec<(bool, BTreeSet<String>)> = parallel(&positions, |nth, history| {
        let game = history.last().unwrap();
        let seen = game.observe();
        let others: History = history.iter().map(unseen_changed).collect();
        let other = others.last().unwrap().observe();
        let (told_of, others) = (reports(history), reports(&others));
        let mut told = BTreeSet::new();
        let mut offered = BTreeSet::new();
        for faction in Faction::ALL {
            let view = View::of(&seen, told_of.clone(), faction, PLAYERS);
            for choice in &view.choices {
                let whole = seen.choices.iter().find(|c| c.id == choice.id).unwrap();
                assert_eq!(whole.owner, faction);
                assert_eq!(choice.supported, whole.supported);
                assert_eq!(choice.explanation, whole.explanation);
                assert!(!whole.explanation.is_empty());
                // An Event option with no preview is told what its text does.
                if choice.preview != whole.preview {
                    assert_eq!(whole.preview, None);
                    assert!(
                        [
                            "event:unshaded",
                            "event:shaded",
                            "government:event",
                            "syndicate:event"
                        ]
                        .contains(&whole.action()),
                        "{}",
                        whole.id
                    );
                }
                offered.insert(whole.id.clone());
            }
            // What is written out, for the decider and each other faction in turn: the
            // four views of a position share their state.
            if faction == seen.owner || faction.index() == nth % 4 {
                let view = serde_json::to_value(view).unwrap();
                let same = serde_json::to_value(View::of(&other, others.clone(), faction, PLAYERS))
                    .unwrap();
                assert_eq!(view, same, "a view shows what is hidden");
                assert_hidden(&view, game);
                keys(&view, &mut told);
            }
        }
        let all: BTreeSet<String> = seen
            .choices
            .iter()
            .filter(|c| c.action() != "advance-card")
            .map(|c| c.id.clone())
            .collect();
        assert_eq!(offered, all, "choices no view carries");
        (seen.kidnap.is_some(), told)
    });
    let rolled = found.iter().filter(|(rolled, _)| *rolled).count();
    assert!(rolled > 100, "{rolled} positions after a rolled Kidnap");
    let told: BTreeSet<String> = found.into_iter().flat_map(|(_, told)| told).collect();
    let listed: BTreeSet<String> = TOLD.iter().map(|key| (*key).to_owned()).collect();
    assert_eq!(told, listed, "the fields told are not those listed");
}

/// Every object key that occurs in a seat brief, as `TOLD` lists a view's.
const BRIEFED: &[&str] = &[
    "accompanies",
    "actions",
    "available",
    "condition",
    "contributions",
    "count",
    "counts",
    "dealings",
    "edition",
    "extra",
    "faction",
    "kind",
    "margin",
    "margin_rule",
    "name",
    "operations",
    "pieces",
    "played",
    "purpose",
    "ranking",
    "read_at",
    "revision",
    "rivals",
    "rule",
    "rules",
    "score",
    "space",
    "special_activities",
    "standing",
    "state",
    "threshold",
    "victory",
    "who",
];

/// Every faction's seat brief (#204) in every explored position is the same whatever lies
/// face down and whatever the generator's state is, holds neither, and names only listed
/// fields.
#[test]
fn no_brief_depends_on_what_is_hidden() {
    let positions = positions();
    let found: Vec<BTreeSet<String>> = parallel(&positions, |_, history| {
        let game = history.last().unwrap();
        let seen = game.observe();
        let other = unseen_changed(game).observe();
        let mut told = BTreeSet::new();
        for faction in Faction::ALL {
            let brief = Brief::of(&seen, faction);
            assert_eq!(
                brief,
                Brief::of(&other, faction),
                "a brief shows what is hidden"
            );
            let brief = serde_json::to_value(brief).unwrap();
            assert_hidden(&brief, game);
            keys(&brief, &mut told);
        }
        told
    });
    let told: BTreeSet<String> = found.into_iter().flatten().collect();
    let listed: BTreeSet<String> = BRIEFED.iter().map(|key| (*key).to_owned()).collect();
    assert_eq!(told, listed, "the fields told are not those listed");
}

/// The hidden-field check bites: each planted leak fails it.
#[test]
fn a_planted_leak_is_found() {
    let game = played(5, &["terror", "kidnap", "kidnap:roll"]);
    let state = game.random_state();
    for leak in [
        json!({"choices": [{"explanation": format!("state {state}")}]}),
        json!({"results": [{"articles": [{"random_before": 5}]}]}),
        json!({"deck": [19, 20, 21]}),
        json!({"deck": {"next": [36, 51]}}),
        json!({"state": {"preview_card": {"texts": [{"statement": "Sierra Maestra Manifesto"}]}}}),
    ] {
        let found = std::panic::catch_unwind(|| assert_hidden(&leak, &game));
        assert!(found.is_err(), "{leak}");
    }
    assert_ne!(unseen_changed(&game), game);
}

// --- Claims -------------------------------------------------------------------------

#[test]
fn every_seat_is_the_humans_until_handed_over_and_one_session_holds_it() {
    let mut s = Sitting::new();
    assert_eq!(s.host.players(), [Player::Human; 4]);
    let answer = s.refused(json!({"op":"join","faction":"Directorio"}));
    assert!(
        error(&answer).contains("has not handed it over"),
        "{answer}"
    );
    assert!(error(&answer).contains("stop"), "{answer}");

    s.host.hand_over(Directorio);
    s.host.hand_over(Government);
    assert_eq!(s.host.player(Directorio), Player::Nobody);
    let answer = s.refused(json!({"op":"join","faction":"M26"}));
    assert!(
        error(&answer).contains("Open to a session now: Government, Directorio."),
        "{answer}"
    );
    s.refused(json!({"op":"join","faction":"Batista"}));
    s.refused(json!({"op":"join"}));

    // Two claims of one seat: the first holds it.
    let first = s.send(json!({"op":"join","faction":"directorio","session":"tool=1"}));
    let second = s.send(json!({"op":"join","faction":"directorio","session":"tool=2"}));
    let first = s.answer(first).unwrap();
    let player = first["player"].as_str().unwrap().to_owned();
    assert_eq!(first["status"], "joined");
    let second = s.answer(second).unwrap();
    assert!(
        error(&second).contains("Another session already plays"),
        "{second}"
    );
    assert!(second.get("view").is_none(), "a stranger is told no view");
    assert_eq!(s.host.player(Directorio), Player::Agent);

    // One session, one seat: by its identity and by its tool's identifiers.
    let again = s.refused(json!({"op":"join","faction":"Government","player":player}));
    assert!(error(&again).contains("one session plays one faction"));
    assert_eq!(again["view"]["faction"], "Directorio");
    let again = s.refused(json!({"op":"join","faction":"Government","session":"tool=1"}));
    assert!(error(&again).contains("already plays Directorio"));
    assert!(!again.to_string().contains(&player), "no identity is told");
    assert_eq!(s.host.player(Government), Player::Nobody);

    // Two sessions hold two seats.
    let joined = s.ask(json!({"op":"join","faction":"Government","session":"tool=2"}));
    assert_eq!(joined["status"], "joined");
}

/// Assignments are the running table's: a new host, as after a restart, has none.
#[test]
fn a_restart_returns_every_faction_to_the_human() {
    let mut s = Sitting::new();
    let player = s.seat(M26);
    s.applied(&player, "terror");
    let path = s.room.path.clone();
    s.room.session = Session::open(&path, None).unwrap();
    s.host = Host::new();
    assert_eq!(s.host.players(), [Player::Human; 4]);
    s.refused(json!({"op":"observe","player":player}));
    s.refused(json!({"op":"join","faction":"M26","player":player}));
}

// --- Authority ----------------------------------------------------------------------

#[test]
fn a_choice_that_is_not_the_sessions_to_make_is_refused_and_changes_nothing() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    let government = s.seat(Government);
    let choose = |player: &str, revision: u64, id: &str| json!({"op":"choose","player":player,"revision":revision,"id":id});
    // Another faction's choice, by a session that holds another seat.
    let answer = s.refused(choose(&government, 5, "5:event:shaded"));
    assert_eq!(answer["view"]["faction"], "Government");
    // A stale revision, in the request and in the id.
    let answer = s.refused(choose(&player, 4, "4:end-terror"));
    assert!(error(&answer).contains("the match is at revision 5"));
    s.refused(choose(&player, 5, "4:event:shaded"));
    // Not offered, offered but not played by this build, and no session.
    s.refused(choose(&player, 5, "5:terror"));
    let answer = s.refused(choose(&player, 5, "5:directorio:pass"));
    assert!(error(&answer).contains("outside this task"), "{answer}");
    s.refused(choose("nobody", 5, "5:event:shaded"));
    // No request at all.
    let sent = s.send_line("choose please");
    assert_eq!(s.answer(sent).unwrap()["status"], "refused");
    s.refused(json!({"op":"say","player":player,"text":"Long live"}));
    s.refused(json!({"op":"choose","player":player,"revision":5,"id":"5:event:shaded","as":"M26"}));

    // The human's faction's choice: 26July's.
    let mut s = Sitting::new();
    let player = s.seat(Directorio);
    s.refused(choose(&player, 0, "0:pass"));
    assert!(s.room.commits.is_empty());

    // An amount past a transfer's limit keeps the authority's reason: Syndicate has 15.
    s.human("terror");
    s.human("transfer");
    s.human("ask:3:2");
    let syndicate = s.seat(Syndicate);
    let answer = s.refused(choose(&syndicate, 3, "3:authorize:16"));
    assert!(
        error(&answer).contains("Syndicate has 15 Resources"),
        "{answer}"
    );
}

#[test]
fn a_choice_is_saved_as_it_is_applied_and_its_answer_holds_the_next_decision() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    let answer = s.applied(&player, "event:shaded");
    assert_eq!(answer["view"]["revision"], 6);
    assert_eq!(answer["view"]["decision"]["yours"], true);
    assert!(actions(&answer["view"]).contains(&"resolve-shaded".into()));
    let (faction, choice, on_disk) = &s.room.commits[0];
    assert_eq!((*faction, choice.as_str()), (Directorio, "5:event:shaded"));
    assert_eq!(on_disk.last().unwrap(), "event:shaded");
}

/// A save that cannot be written refuses the choice: the position stays.
#[test]
fn a_choice_that_cannot_be_saved_is_refused() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    // Another session advances the same save.
    let path = s.room.path.clone();
    let mut other = Session::open(&path, None).unwrap();
    other.submit(Directorio, 5, "5:event:unshaded").unwrap();
    let request = s.choice(&player, "event:shaded");
    let answer = s.refused(request);
    assert!(
        error(&answer).contains("Save changed in another session"),
        "{answer}"
    );
}

/// A held choice whose save cannot be written as it is applied is released with the
/// save's reason, and the position stays (#197's review).
#[test]
fn a_held_choice_that_cannot_be_saved_is_released_with_the_reason() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let mut request = s.choice(&player, "event:shaded");
    request["wait"] = json!(true);
    let sent = s.send(request);
    assert!(s.answer(sent).is_none());
    let mut other = Session::open(&s.room.path, None).unwrap();
    other.submit(Directorio, 5, "5:event:unshaded").unwrap();
    let written = std::fs::read(&s.room.path).unwrap();
    s.room.settled = true;
    s.poll();
    let told = s.answer(sent).unwrap();
    assert_eq!(told["status"], "released", "{told}");
    let reason = told["reason"].as_str().unwrap();
    assert!(reason.contains("Save changed in another session"), "{told}");
    assert_eq!(told["choice"]["id"], "5:event:shaded");
    assert_eq!(s.revision(), 5);
    assert!(s.room.commits.is_empty());
    assert_eq!(std::fs::read(&s.room.path).unwrap(), written);
}

/// Turning the card is the human's, whichever faction the authority names for it: no
/// agent is offered it, may submit it, or is woken for it.
#[test]
fn the_human_turns_the_card() {
    let mut s = Sitting::at_the_handoff();
    let directorio = s.seat(Directorio);
    let government = s.seat(Government);
    let waiting = s.wait(&government);
    s.applied(&directorio, "event:shaded");
    let done = s.applied(&directorio, "resolve-shaded");
    assert_eq!(s.room.session.observe().owner, Directorio);
    assert_eq!(done["view"]["choices"], json!([]));
    assert_eq!(done["view"]["decision"]["stands"], "card");
    assert_eq!(done["view"]["decision"]["yours"], false);
    assert_eq!(s.state(Directorio), SeatState::AtTable);
    let request = s.choice(&directorio, "advance-card");
    let answer = s.refused(request);
    assert!(error(&answer).contains("The human turns the card"));
    // It waits, and is not woken for it.
    let held = s.wait(&directorio);
    s.poll();
    assert!(s.answer(held).is_none());
    assert!(s.answer(waiting).is_none());
    s.human("advance-card");
    assert_eq!(s.answer(waiting).unwrap()["status"], "decision");
    assert!(s.answer(held).is_none());
}

// --- The view -----------------------------------------------------------------------

/// The prepared position as card 17's setup gives it (R 7.2-7.3): Directorio's view at
/// 26July's decision, worked by hand from the tactical handoff's inventory.
#[test]
fn a_view_is_whole_and_says_where_the_match_stands() {
    let mut s = Sitting::new();
    let player = s.seat(Directorio);
    let view = s.observe(&player);
    assert_eq!(view["faction"], "Directorio");
    assert_eq!(view["revision"], 0);
    assert_eq!(view["decision"]["faction"], "M26");
    assert_eq!(view["decision"]["stands"], "open");
    assert_eq!(view["decision"]["yours"], false);
    assert_eq!(view["choices"], json!([]), "26July's choices are 26July's");
    assert_eq!(view["state"]["resources"], json!([6, 0, 5, 15]));
    assert_eq!(view["state"]["current_card"]["number"], 17);
    assert_eq!(view["state"]["preview_card"]["number"], 18);
    assert_eq!(view["state"]["deck"]["face_down"], 33);
    assert_eq!(view["state"]["played"][0]["number"], 17);
    assert!(
        view["rules"]["edition"]
            .as_str()
            .unwrap()
            .contains("March 2019")
    );
    let played_by: Vec<_> = view["factions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["played_by"].as_str().unwrap())
        .collect();
    assert_eq!(played_by, ["human", "human", "agent", "human"]);
    let space = |view: &Value, id: &str| -> Value {
        let spaces = view["state"]["spaces"].as_array().unwrap();
        spaces.iter().find(|s| s["id"] == id).unwrap().clone()
    };
    // 6 Troops and 4 Police of 12 pieces that are not closed and 1 closed Casino or open:
    // Government controls Havana either way (R 1.7), at Active Support.
    let havana = space(&view, "havana");
    assert_eq!(havana["control"], "Government");
    assert_eq!(havana["support_level"], "Active Support");
    assert_eq!(
        space(&view, "ec-oriente-sierra")["support_level"],
        Value::Null
    );
    // Complete again, unasked: nothing is told as a change.
    assert_eq!(s.observe(&player), view);

    // The results come apart from the state (R 4.3.3: min(3, 6) = 3 move to 26July; R
    // 3.3.4: Terror pays 1 of them).
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    let view = s.observe(&player);
    let results = view["results"].as_array().unwrap();
    assert_eq!(results.len(), 1, "one action so far");
    assert_eq!(results[0]["faction"], "M26");
    assert_eq!(results[0]["action"], "terror");
    assert_eq!(results[0]["complete"], true);
    assert_eq!(results[0]["articles"][0]["die"], 3);
    assert_eq!(
        results[0]["articles"][0]["lines"][1],
        "26July takes 3 Resources from Government: Government 6 → 3, 26July 0 → 3."
    );
    assert_eq!(
        results[0]["standings"][1]["resources"],
        json!({"from": 0, "to": 2})
    );
    // Every seat reads the same reports, of an action in progress too.
    s.human("event:unshaded");
    s.human("rally:beneficiary:26july");
    let government = s.seat(Government);
    let results = s.observe(&government)["results"].clone();
    assert_eq!(s.observe(&player)["results"], results);
    assert_eq!(results[0], view["results"][0]);
    assert_eq!(results[1]["faction"], "Directorio");
    assert_eq!(results[1]["complete"], false);
    assert_eq!(
        results[1]["summary"],
        "Directorio grants the free Rally to 26July."
    );
    assert_eq!(view["state"]["resources"], json!([3, 2, 5, 15]));
    assert_eq!(view["decision"]["yours"], true);
    let pass = &view["choices"][0];
    assert_eq!(pass["id"], "5:directorio:pass");
    assert_eq!(pass["supported"], false);
    assert!(pass["explanation"].as_str().unwrap().contains("R 2.3.3"));
    let shaded = view["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "5:event:shaded")
        .unwrap();
    // Two Guerrillas stand in Santiago, one flipped by Terror, and one in Oriente: two
    // are Underground still (R 5.1).
    let flipped: u64 = shaded["preview"]["flips"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["count"].as_u64().unwrap())
        .sum();
    assert_eq!(flipped, 2);
}

/// What each text of a face-up card's Event does can be read in every view, in the
/// authority's own words: card 17's two texts and card 18 in preview, then card 18 and card
/// 19 in preview. Expectations are from the three faces and R 5.1 to 5.3, worked by hand. An
/// Event option that previews nothing else previews its text.
#[test]
fn a_view_says_what_each_text_of_the_face_up_cards_does() {
    /// Each text's name, what it must say, and where this build plays it.
    type Text<'a> = (Value, &'a [&'a str], &'a str);
    let seventeen: [Text; 2] = [
        (
            json!("Unshaded"),
            // The executing faction selects which of the two (R 5.1); free (R 5.5); "as if
            // spaces Neutral" lifts R 3.3.1's limits.
            &[
                "selects 26July or Directorio",
                "free",
                "each space where it has a Base",
                "whatever the Support or Opposition there",
            ],
            "in this build, by Directorio",
        ),
        (
            json!("Shaded"),
            &[
                "26July Guerrilla",
                "becomes Active",
                "Ineligible until card 18 has been played",
            ],
            "in this build, by Directorio",
        ),
    ];
    let eighteen: [Text; 1] = [(
        Value::Null,
        // R 5.3: a Capability lasts for the rest of the game, unless cancelled as it says.
        &[
            "Insurgent Capability",
            "26July and Directorio",
            "remove each other's pieces",
            "Opposition",
            "2 of its own Bases at once",
            "stays Eligible for the next card",
        ],
        "not in this build",
    )];
    let nineteen: [Text; 1] = [(
        Value::Null,
        &[
            "card's faction order",
            "2 pieces",
            "other than Casinos",
            "already has a piece",
            "stays Eligible",
        ],
        "not in this build",
    )];
    let check = |card: &Value, number: u64, title: &str, texts: &[Text]| {
        assert_eq!(card["number"], number);
        assert_eq!(card["title"], title);
        assert_eq!(card["order"].as_array().unwrap().len(), 4);
        assert!(card["kind"].is_string());
        let told = card["texts"].as_array().unwrap();
        assert_eq!(told.len(), texts.len(), "{card}");
        for (told, (text, says, played)) in told.iter().zip(texts) {
            assert_eq!(told["text"], *text);
            assert_eq!(told["played"], *played);
            assert!(told["rules"].as_str().unwrap().contains("5.1"), "{told}");
            let statement = told["statement"].as_str().unwrap();
            for said in *says {
                assert!(statement.contains(said), "{said}: {statement}");
            }
            // Own words: no run of the printed text.
            for printed in [
                "free Rallies",
                "as if spaces Neutral",
                "Guerrillas Active.",
                "through next card",
                "affect placed Opposition",
                "cancel Capability",
                "may place 2 non-Casino",
            ] {
                assert!(!statement.contains(printed), "{statement}");
            }
        }
    };
    let option = |view: &Value, action: &str| -> Value {
        let choices = view["choices"].as_array().unwrap();
        let ends = format!(":{action}");
        let found = choices
            .iter()
            .find(|c| c["id"].as_str().unwrap().ends_with(&ends));
        found.unwrap_or_else(|| panic!("{action}")).clone()
    };
    let notes = |view: &Value, action: &str| option(view, action)["preview"]["notes"].clone();

    // 26July, at its opening decision, which lists both texts refused.
    let mut s = Sitting::new();
    let players: Vec<String> = Faction::ALL.map(|f| s.seat(f)).into();
    let view = s.observe(&players[M26.index()]);
    check(
        &view["state"]["current_card"],
        17,
        "Alberto Bayo",
        &seventeen,
    );
    let texts = &view["state"]["current_card"]["texts"];
    assert_eq!(
        notes(&view, "event:unshaded"),
        json!([texts[0]["statement"]])
    );
    assert_eq!(notes(&view, "event:shaded"), json!([texts[1]["statement"]]));

    // Directorio, at the handoff; and every other faction reads the same of the cards.
    for action in HANDOFF {
        s.applied(&players[M26.index()], action);
    }
    for player in &players {
        let view = s.observe(player);
        check(
            &view["state"]["current_card"],
            17,
            "Alberto Bayo",
            &seventeen,
        );
        check(
            &view["state"]["preview_card"],
            18,
            "Pact of Caracas",
            &eighteen,
        );
        assert!(!view.to_string().contains("Sierra Maestra Manifesto"));
    }
    let view = s.observe(&players[Directorio.index()]);
    assert_eq!(
        notes(&view, "event:unshaded"),
        json!([texts[0]["statement"]])
    );
    // The shaded option has its own preview, as the authority computes it.
    let flips = &option(&view, "event:shaded")["preview"]["flips"];
    assert_eq!(flips.as_array().unwrap().len(), 2);

    // The Government, on card 18.
    s.applied(&players[Directorio.index()], "event:shaded");
    s.applied(&players[Directorio.index()], "resolve-shaded");
    s.human("advance-card");
    let view = s.observe(&players[Government.index()]);
    check(
        &view["state"]["current_card"],
        18,
        "Pact of Caracas",
        &eighteen,
    );
    check(
        &view["state"]["preview_card"],
        19,
        "Sierra Maestra Manifesto",
        &nineteen,
    );
    let statement = &view["state"]["current_card"]["texts"][0]["statement"];
    assert_eq!(notes(&view, "government:event"), json!([statement]));
    // The Syndicate, after the Government's Pass.
    s.applied(&players[Government.index()], "government:pass");
    let view = s.observe(&players[Syndicate.index()]);
    assert_eq!(notes(&view, "syndicate:event"), json!([statement]));
}

/// The four ends of this build. A seat that reaches one is told, and is not woken.
#[test]
fn at_each_end_of_the_build_the_seat_is_told_and_not_woken() {
    for (from, ending, decider) in [
        // Two ends list no choice at all (R 2.3.3, 2.3.4).
        (&[][..], "pass", Directorio),
        (&["terror"], "end-terror", Directorio),
        (
            &[
                "terror",
                "kidnap",
                "kidnap:roll",
                "terror:santiago",
                "end-terror",
                "event:shaded",
                "resolve-shaded",
                "advance-card",
                "government:pass",
            ],
            "syndicate:pass",
            Syndicate,
        ),
        (
            &[
                "terror",
                "kidnap",
                "kidnap:roll",
                "terror:santiago",
                "end-terror",
                "event:shaded",
                "resolve-shaded",
                "advance-card",
                "government:assault",
                "end-assault",
            ],
            "syndicate:pass",
            Syndicate,
        ),
    ] {
        let mut s = Sitting::new();
        for action in from {
            s.human(action);
        }
        let player = s.seat(decider);
        // An unrelated waiting seat must not wake as the current card ends.
        let government = s.seat(Government);
        let waiting = s.wait(&government);
        s.human(ending);
        s.pass(3600);
        assert!(s.answer(waiting).is_none(), "{ending} woke {decider:?}");
        let view = s.observe(&player);
        assert_eq!(view["decision"]["stands"], "build-ends", "{ending}");
        assert_eq!(view["decision"]["faction"], json!(decider));
        assert_eq!(view["decision"]["yours"], false);
        let note = view["decision"]["note"].as_str().unwrap();
        if decider == Syndicate {
            assert!(note.contains("Card 18 is complete"), "{note}");
        } else {
            assert!(note.contains("The build stops here"), "{note}");
        }
        let supported = view["choices"].as_array().unwrap().iter();
        assert!(supported.clone().all(|c| c["supported"] == false));
        assert_eq!(supported.len(), 0, "{ending}");
        assert_eq!(s.state(decider), SeatState::BuildEnds);
        assert_eq!(Stands::of(s.room.session.game()), Stands::BuildEnds);
        // Every other view says so too, and no submission is taken.
        assert_eq!(s.observe(&government)["decision"]["stands"], "build-ends");
        if decider == Syndicate {
            let request = s.choice(&player, "syndicate:pass");
            s.refused(request);
        }
    }
}

// --- Held choices -------------------------------------------------------------------

#[test]
fn a_wait_reports_the_replacement_committed_when_the_table_settles() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let waiting = s.choose_and_wait(&player, "event:shaded");
    assert!(s.answer(waiting).is_none());
    assert_eq!(s.choose(&player, "event:unshaded")["status"], "held");

    s.room.settled = true;
    s.poll();
    assert_eq!(s.room.commits.len(), 1);
    assert_eq!(s.room.commits[0].1, "5:event:unshaded");
    assert_eq!(
        decisions(&s.room.path).last().map(String::as_str),
        Some("event:unshaded")
    );
    let answer = s.answer(waiting).expect("The next decision was told");
    assert_eq!(answer["status"], "decision");
    assert_eq!(
        answer["choice"],
        json!({"id":"5:event:unshaded","label":"Event · unshaded","status":"applied"})
    );
}

#[test]
fn a_wait_that_expires_reports_the_replacement_still_held() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let before = decisions(&s.room.path);
    let mut request = s.choice(&player, "event:shaded");
    request["wait"] = json!(true);
    request["timeout_seconds"] = json!(10);
    let waiting = s.send(request);
    assert!(s.answer(waiting).is_none());
    assert_eq!(s.choose(&player, "event:unshaded")["status"], "held");

    s.pass(10);
    assert!(s.room.commits.is_empty());
    assert_eq!(decisions(&s.room.path), before);
    let answer = s.answer(waiting).expect("The wait expired");
    assert_eq!(answer["status"], "nothing_yet");

    s.room.settled = true;
    s.poll();
    assert_eq!(s.room.commits.len(), 1);
    assert_eq!(s.room.commits[0].1, "5:event:unshaded");
    assert_eq!(
        decisions(&s.room.path).last().map(String::as_str),
        Some("event:unshaded")
    );
    assert_eq!(
        answer["choice"],
        json!({"id":"5:event:unshaded","label":"Event · unshaded","status":"held"})
    );
}

#[test]
fn a_choice_waits_for_the_table_and_is_applied_when_it_settles() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let before = s.standing();
    let answer = s.choose(&player, "event:shaded");
    assert_eq!(answer["status"], "held", "{answer}");
    assert_eq!(s.standing(), before);
    assert!(s.host.holds_choice(Directorio));
    assert!(!s.host.due(s.room.session.game(), false, s.now));
    s.pass(600);
    assert_eq!(s.standing(), before);
    assert_eq!(s.state(Directorio), SeatState::Thinking, "its choice is in");
    s.room.settled = true;
    assert!(s.host.due(s.room.session.game(), true, s.now));
    s.poll();
    assert_eq!(s.revision(), 6);
    assert_eq!(s.room.commits.len(), 1);
    // Its next decision is told by its wait.
    let waiting = s.wait(&player);
    assert_eq!(s.answer(waiting).unwrap()["status"], "decision");
}

/// A choice the rules refuse is not held for a moving table, and leaves the one held.
#[test]
fn an_illegal_choice_is_refused_while_the_table_moves() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    let government = s.seat(Government);
    s.room.settled = false;
    assert_eq!(s.choose(&player, "event:shaded")["status"], "held");
    for (who, action) in [
        (&player, "directorio:pass"),
        (&player, "terror"),
        (&government, "event:shaded"),
    ] {
        let request = s.choice(who, action);
        s.refused(request);
    }
    assert!(s.host.holds_choice(Directorio) && !s.host.holds_choice(Government));
    s.room.settled = true;
    s.poll();
    assert_eq!(s.room.commits[0].1, "5:event:shaded");
}

/// A choice held first is applied first, also when the next arrives in the frame the
/// table settles.
#[test]
fn choices_are_applied_in_the_order_they_were_made() {
    let mut s = Sitting::new();
    let m26 = s.seat(M26);
    let syndicate = s.seat(Syndicate);
    for action in ["terror", "transfer", "ask:3:2"] {
        s.applied(&m26, action);
    }
    s.room.settled = false;
    assert_eq!(s.choose(&m26, "end-terror")["status"], "held");
    s.room.settled = true;
    let late = s.choice(&syndicate, "authorize:2");
    let answer = s.ask(late);
    assert_eq!(answer["status"], "refused", "{answer}");
    assert!(error(&answer).contains("the match is at revision 4"));
    assert_eq!(s.room.session.observe().resources, [6, 0, 5, 15]);
}

#[test]
fn a_held_choice_the_match_passed_is_released_with_the_reason() {
    let mut s = Sitting::new();
    let m26 = s.seat(M26);
    s.applied(&m26, "terror");
    s.applied(&m26, "transfer");
    s.applied(&m26, "ask:3:2");
    // 26July withdraws while the human, as Syndicate, reads the request.
    s.room.settled = false;
    let request = s.choice(&m26, "end-terror");
    let mut request = request;
    request["wait"] = json!(true);
    let sent = s.send(request);
    assert!(s.answer(sent).is_none());
    s.human("authorize:2");
    let told = s.answer(sent).unwrap();
    assert_eq!(told["status"], "released", "{told}");
    assert!(
        told["reason"]
            .as_str()
            .unwrap()
            .contains("the match moved on")
    );
    assert_eq!(told["choice"]["id"], "3:end-terror");
    assert_eq!(told["choice"]["status"], "released");
    assert_eq!(told["view"]["state"]["resources"], json!([6, 2, 5, 13]));
    assert!(s.room.commits.iter().all(|c| c.1 != "3:end-terror"));

    // Without a wait, the next answer of any kind says it.
    s.room.settled = true;
    s.applied(&m26, "transfer");
    s.applied(&m26, "ask:3:1");
    s.room.settled = false;
    assert_eq!(s.choose(&m26, "end-terror")["status"], "held");
    s.human("decline");
    let seen = s.ask(json!({"op":"observe","player":m26}));
    assert_eq!(seen["released"][0]["id"], "6:end-terror", "{seen}");
    let seen = s.ask(json!({"op":"observe","player":m26}));
    assert!(seen.get("released").is_none());
}

#[test]
fn taking_back_cancels_a_held_choice_and_ends_the_session() {
    // Before a choice, and with the session waiting.
    let mut s = Sitting::new();
    let player = s.seat(Directorio);
    let waiting = s.wait(&player);
    s.take_back(Directorio);
    let told = s.answer(waiting).unwrap();
    assert_eq!(told["status"], "closed");
    assert!(
        told["reason"]
            .as_str()
            .unwrap()
            .contains("The human plays Directorio now")
    );
    assert!(told.get("view").is_none());
    assert_eq!(s.host.player(Directorio), Player::Human);

    // With one held.
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let sent = s.choose_and_wait(&player, "event:shaded");
    let before = s.standing().0;
    s.take_back(Directorio);
    let told = s.answer(sent).unwrap();
    assert_eq!(told["status"], "closed");
    assert!(
        told["reason"]
            .as_str()
            .unwrap()
            .contains("Event · shaded, was not applied")
    );
    s.room.settled = true;
    s.pass(60);
    assert_eq!(
        s.standing().0,
        before,
        "nothing happens after the human took over"
    );
    // Every later request of the ended session is told so.
    for request in [
        json!({"op":"wait","player":player}),
        json!({"op":"observe","player":player}),
        s.choice(&player, "event:shaded"),
    ] {
        assert_eq!(s.ask(request)["status"], "closed");
    }
    assert_eq!(s.standing().0, before);
    // Handed over again, it is another session's.
    let next = s.seat(Directorio);
    assert_ne!(next, player);
    s.applied(&next, "event:shaded");
}

#[test]
fn a_session_that_leaves_gives_its_seat_to_the_next() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.room.settled = false;
    let waiting = s.choose_and_wait(&player, "event:shaded");
    let left = s.ask(json!({"op":"leave","player":player}));
    assert_eq!(left["status"], "left");
    assert_eq!(s.answer(waiting).unwrap()["status"], "closed");
    assert_eq!(s.host.player(Directorio), Player::Nobody);
    s.room.settled = true;
    s.pass(600);
    assert_eq!(s.revision(), 5, "an empty seat never acts");
    s.refused(json!({"op":"observe","player":player}));
    let joined = s.ask(json!({"op":"join","faction":"Directorio"}));
    assert_eq!(joined["status"], "joined");
}

#[test]
fn closing_the_table_ends_every_session() {
    let mut s = Sitting::new();
    let directorio = s.seat(Directorio);
    let government = s.seat(Government);
    let waits = [s.wait(&directorio), s.wait(&government)];
    let answers = s.host.close("The table closed.");
    s.collect(answers);
    for wait in waits {
        let told = s.answer(wait).unwrap();
        assert_eq!(
            (&told["status"], &told["reason"]),
            (&json!("closed"), &json!("The table closed."))
        );
    }
    assert_eq!(
        s.ask(json!({"op":"wait","player":government}))["status"],
        "closed"
    );
}

// --- Waiting ------------------------------------------------------------------------

#[test]
fn a_decision_is_told_once() {
    let mut s = Sitting::new();
    let player = s.seat(Directorio);
    let waiting = s.wait(&player);
    for action in &HANDOFF[..4] {
        s.human(action);
        assert!(s.answer(waiting).is_none(), "{action} is 26July's");
    }
    s.human(HANDOFF[4]);
    let told = s.answer(waiting).unwrap();
    assert_eq!(told["status"], "decision");
    assert_eq!(told["view"]["revision"], 5);
    assert!(told.get("choice").is_none());
    // The wait may be one nobody read: the next tells it again, whole.
    let again = s.wait(&player);
    assert_eq!(s.answer(again).unwrap()["status"], "decision");
    // Told by an answer it read, it is not told again, and waiting for it is refused.
    s.observe(&player);
    let answer = s.refused(json!({"op":"wait","player":player}));
    assert!(error(&answer).contains("it waits for you"), "{answer}");
    assert_eq!(answer["view"]["decision"]["yours"], true);
}

#[test]
fn joining_and_observing_show_a_decision_without_telling_it() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.observe(&player);
    let waiting = s.wait(&player);
    assert_eq!(s.answer(waiting).unwrap()["status"], "decision");
}

#[test]
fn a_submission_that_waits_is_answered_with_what_comes_next() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    // A refusal is answered at once, and so is a decision of its own that follows.
    let sent = s.choose_and_wait(&player, "directorio:pass");
    assert_eq!(s.answer(sent).unwrap()["status"], "refused");
    let sent = s.choose_and_wait(&player, "event:unshaded");
    let answer = s.answer(sent).unwrap();
    assert_eq!(answer["status"], "applied");
    assert_eq!(answer["view"]["decision"]["yours"], true);
    // Granting 26July the Rally passes the decision on: the wait is held.
    let sent = s.choose_and_wait(&player, "rally:beneficiary:26july");
    assert!(s.answer(sent).is_none());
    assert_eq!(s.revision(), 7);
    assert!(s.host.waits(Directorio));
    // A newer wait takes its place.
    let newer = s.wait(&player);
    assert_eq!(s.answer(sent).unwrap()["status"], "replaced");
    // With a period, it ends with it.
    let timed = s.send(json!({"op":"wait","player":player,"timeout_seconds":30}));
    assert_eq!(s.answer(newer).unwrap()["status"], "replaced");
    s.pass(29);
    assert!(s.answer(timed).is_none());
    s.pass(1);
    assert_eq!(s.answer(timed).unwrap()["status"], "nothing_yet");
    // 26July's request to Directorio needs it, and names nothing it submitted.
    let waiting = s.wait(&player);
    s.human("transfer");
    s.human("ask:2:1");
    let told = s.answer(waiting).unwrap();
    assert_eq!(told["status"], "decision");
    // A submission held by the table, told with how it stands when the next comes.
    s.room.settled = false;
    let sent = s.choose_and_wait(&player, "authorize:1");
    s.room.settled = true;
    s.poll();
    assert!(s.answer(sent).is_none(), "26July decides");
    s.human("transfer");
    s.human("ask:2:1");
    let told = s.answer(sent).unwrap();
    assert_eq!(told["status"], "decision");
    assert_eq!(
        told["choice"],
        json!({"id":"9:authorize:1","label":"Directorio authorizes 1","status":"applied"})
    );
    assert_eq!(told["view"]["state"]["resources"][2], 4);
}

#[test]
fn a_dead_clients_wait_ends_and_its_event_is_told_to_the_next() {
    let mut s = Sitting::new();
    let player = s.seat(Directorio);
    let dead = s.wait(&player);
    assert!(!s.host.hung_up(|_| false));
    assert!(s.host.hung_up(|reply| *reply == dead));
    s.host.end_hung_up(|reply| *reply == dead, s.now);
    assert!(!s.host.waits(Directorio));
    for action in HANDOFF {
        s.human(action);
    }
    assert!(s.answer(dead).is_none());
    assert_eq!(s.state(Directorio), SeatState::Thinking);
    s.pass(125);
    assert_eq!(s.state(Directorio), SeatState::Silent { minutes: 2 });
    let next = s.wait(&player);
    assert_eq!(s.answer(next).unwrap()["status"], "decision");
    assert_eq!(s.state(Directorio), SeatState::Thinking);
}

#[test]
fn each_seats_state_is_shown() {
    let mut s = Sitting::new();
    assert_eq!(s.state(Directorio), SeatState::Human);
    s.host.hand_over(Directorio);
    assert_eq!(s.state(Directorio), SeatState::Awaiting);
    let player = s.seat(Directorio);
    s.pass(600);
    assert_eq!(
        s.state(Directorio),
        SeatState::AtTable,
        "nothing is asked of it"
    );
    for action in HANDOFF {
        s.human(action);
    }
    // Not heard from since it joined, 600 s before its decision came.
    assert_eq!(s.state(Directorio), SeatState::Silent { minutes: 10 });
    s.observe(&player);
    assert_eq!(s.state(Directorio), SeatState::Thinking);
    s.pass(59);
    assert_eq!(s.state(Directorio), SeatState::Thinking);
    s.pass(1);
    assert_eq!(s.state(Directorio), SeatState::Silent { minutes: 1 });
    // Awaiting a session at its own decision, it never acts.
    s.ask(json!({"op":"leave","player":player}));
    s.pass(3600);
    assert_eq!(s.state(Directorio), SeatState::Awaiting);
    assert_eq!(s.revision(), 5);
}

// --- Transfers ----------------------------------------------------------------------

/// R 1.5.1: 26July asks; only the giver decides what moves.
#[test]
fn an_agent_as_giver_authorizes_and_declines() {
    let mut s = Sitting::new();
    let syndicate = s.seat(Syndicate);
    let waiting = s.wait(&syndicate);
    s.human("terror");
    s.human("transfer");
    s.human("ask:3:2");
    let told = s.answer(waiting).unwrap();
    assert_eq!(told["status"], "decision");
    // Its whole range, from nothing to all 15, and no more (R 1.5.1).
    let offered = actions(&told["view"]);
    for amount in 0..=15 {
        assert!(
            offered.contains(&format!("authorize:{amount}")),
            "{offered:?}"
        );
    }
    assert!(!offered.contains(&"authorize:16".into()));
    assert!(
        !offered.contains(&"end-terror".into()),
        "26July's withdrawal"
    );
    s.applied(&syndicate, "authorize:14");
    assert_eq!(s.room.session.observe().resources, [6, 14, 5, 1]);
    s.human("transfer");
    s.human("ask:3:1");
    assert!(!actions(&s.observe(&syndicate)).contains(&"authorize:2".into()));
    s.applied(&syndicate, "decline");
    assert_eq!(s.room.session.observe().owner, M26);
}

/// A giver that answers Wait still decides. Its next wait is held, not refused, and it is
/// not told again; 26July's withdrawal ends it.
#[test]
fn a_giver_that_set_its_decision_aside_is_held_and_not_told_again() {
    let mut s = Sitting::new();
    let syndicate = s.seat(Syndicate);
    let waiting = s.wait(&syndicate);
    s.human("terror");
    s.human("transfer");
    s.human("ask:3:2");
    s.answer(waiting).unwrap();
    let answer = s.applied(&syndicate, "wait");
    assert_eq!(answer["view"]["decision"]["yours"], true);
    let note = answer["view"]["decision"]["note"].as_str().unwrap();
    assert!(note.contains("set this Transfer request aside"), "{note}");
    let held = s.wait(&syndicate);
    s.pass(3600);
    assert!(s.answer(held).is_none());
    assert_eq!(
        s.state(Syndicate),
        SeatState::Thinking,
        "it waits to be told"
    );
    // It may still answer, from the same view.
    let offered = actions(&s.observe(&syndicate));
    assert!(offered.contains(&"reopen".into()) && offered.contains(&"authorize:2".into()));
    // 26July withdraws; its wait is still held for what comes next.
    s.human("end-terror");
    s.pass(1);
    assert!(s.answer(held).is_none());

    // In the second form too, and reopening is a decision of its own.
    let mut s = Sitting::new();
    let syndicate = s.seat(Syndicate);
    s.human("terror");
    s.human("transfer");
    s.human("ask:3:2");
    let sent = s.choose_and_wait(&syndicate, "wait");
    s.pass(60);
    assert!(s.answer(sent).is_none());
    let reopened = s.applied(&syndicate, "reopen");
    assert_eq!(reopened["view"]["decision"]["yours"], true);
    assert!(s.answer(sent).is_none(), "its own answer told it");
    // That wait cannot tell it the decision it was told, so it does not hide a silence.
    assert!(s.host.waits(Syndicate));
    s.pass(120);
    assert_eq!(s.state(Syndicate), SeatState::Silent { minutes: 2 });
    // Held while the table moves, and set aside when it is applied.
    s.room.settled = false;
    assert_eq!(s.choose(&syndicate, "wait")["status"], "held");
    s.room.settled = true;
    s.poll();
    s.pass(60);
    assert!(s.answer(sent).is_none());
    // A request set aside by someone else is a new session's to be told of.
    s.ask(json!({"op":"leave","player":syndicate}));
    let next = s.seat(Syndicate);
    let waiting = s.wait(&next);
    assert_eq!(s.answer(waiting).unwrap()["status"], "decision");
}

/// 26July keeps its withdrawal while another giver decides (local policy, #58): it sees
/// it, may submit it, and is not woken for it.
#[test]
fn an_agent_26july_asks_the_human_and_another_seat_and_may_withdraw() {
    let mut s = Sitting::new();
    let m26 = s.seat(M26);
    s.applied(&m26, "terror");
    s.applied(&m26, "transfer");
    let sent = s.choose_and_wait(&m26, "ask:0:3");
    assert_eq!(s.room.session.observe().owner, Government);
    s.pass(60);
    assert!(s.answer(sent).is_none(), "a withdrawal wakes nobody");
    assert_eq!(s.state(M26), SeatState::AtTable);
    let view = s.observe(&m26);
    assert_eq!(view["decision"]["yours"], false);
    assert_eq!(actions(&view), ["back-to-choice", "end-terror"]);
    assert!(
        view["decision"]["note"]
            .as_str()
            .unwrap()
            .contains("stay offered")
    );
    // The human gives 2 of Government's 6.
    s.human("authorize:2");
    let told = s.answer(sent).unwrap();
    assert_eq!(told["status"], "decision");
    assert_eq!(told["choice"]["status"], "applied");
    assert_eq!(told["view"]["state"]["resources"], json!([4, 2, 5, 15]));
    // Another seat decides the next, and 26July withdraws from it.
    let syndicate = s.seat(Syndicate);
    let waiting = s.wait(&syndicate);
    s.applied(&m26, "transfer");
    s.applied(&m26, "ask:3:5");
    assert_eq!(s.answer(waiting).unwrap()["status"], "decision");
    s.applied(&m26, "end-terror");
    assert_eq!(s.room.session.observe().transfer, None);
    assert_eq!(s.room.session.observe().resources, [4, 2, 5, 15]);
    let request = s.choice(&syndicate, "authorize:5");
    s.refused(request);

    // The human's withdrawal stays his while an agent gives.
    let mut s = Sitting::new();
    let syndicate = s.seat(Syndicate);
    s.human("terror");
    s.human("transfer");
    s.human("ask:3:5");
    s.room.settled = false;
    assert_eq!(s.choose(&syndicate, "authorize:5")["status"], "held");
    s.human("back-to-choice");
    s.room.settled = true;
    let seen = s.ask(json!({"op":"observe","player":syndicate}));
    assert_eq!(seen["released"][0]["label"], "Syndicate authorizes 5");
    assert_eq!(s.room.session.observe().resources, [6, 0, 5, 15]);
}

// --- Recovery -----------------------------------------------------------------------

#[test]
fn a_session_that_lost_its_answer_comes_back_with_its_identity() {
    let mut s = Sitting::at_the_handoff();
    let player = s.seat(Directorio);
    s.applied(&player, "event:shaded");
    // Its answer was lost; the same choice again is refused, with the view it lost.
    let answer =
        s.refused(json!({"op":"choose","player":player,"revision":5,"id":"5:event:shaded"}));
    assert_eq!(answer["view"]["revision"], 6);
    assert_eq!(s.observe(&player)["revision"], 6);
    s.applied(&player, "resolve-shaded");
}

/// A session is given its own faction's brief as it joins, and again, as the match then
/// stands, when it asks with its identity whether its seat is still its own: a restarted
/// session need not remember it. No other answer holds one.
#[test]
fn a_session_is_given_its_seat_brief_as_it_joins() {
    let mut s = Sitting::new();
    for faction in Faction::ALL {
        s.host.hand_over(faction);
        let joined = s.ask(json!({"op":"join","faction":faction.name()}));
        assert_eq!(joined["status"], "joined", "{joined}");
        let brief = Brief::of(&s.room.session.observe(), faction);
        assert_eq!(joined["brief"], serde_json::to_value(&brief).unwrap());
        assert_eq!(joined["brief"]["revision"], joined["view"]["revision"]);
    }

    let mut s = Sitting::new();
    let directorio = s.seat(Directorio);
    let m26 = s.seat(M26);
    for action in HANDOFF {
        let applied = s.applied(&m26, action);
        assert!(applied.get("brief").is_none(), "{applied}");
    }
    let waited = s.wait(&directorio);
    assert!(s.answer(waited).unwrap().get("brief").is_none());
    let seen = s.ask(json!({"op":"observe","player":directorio}));
    assert!(seen.get("brief").is_none());

    // It comes back after a compacted context: the brief of now, at 26July's 8 − 15.
    for named in ["Directorio", "Government"] {
        let again = s.refused(json!({"op":"join","faction":named,"player":directorio}));
        assert_eq!(again["brief"]["faction"], "Directorio");
        assert_eq!(again["brief"]["revision"], 5);
        let rivals = again["brief"]["rivals"].as_array().unwrap();
        let m26 = rivals.iter().find(|r| r["name"] == "26July").unwrap();
        assert_eq!(m26["standing"]["margin"], -7);
    }
    // A stranger, and a session that does not give its identity, are told no brief.
    let stranger = s.refused(json!({"op":"join","faction":"Directorio"}));
    assert!(stranger.get("brief").is_none(), "{stranger}");
    s.host.hand_over(Government);
    s.ask(json!({"op":"join","faction":"Government","session":"tool=1"}));
    let same = s.refused(json!({"op":"join","faction":"Government","session":"tool=1"}));
    assert!(same.get("brief").is_none(), "{same}");
}

/// The save the sessions made is, byte for byte, the one the same decisions by hand
/// produce, and it opens by exact replay.
#[test]
fn the_same_decisions_by_hand_write_the_same_save() {
    let mut s = Sitting::new();
    let players: Vec<String> = Faction::ALL.map(|f| s.seat(f)).into();
    let played = [
        (M26, "terror"),
        (M26, "transfer"),
        (M26, "ask:3:1"),
        (Syndicate, "wait"),
        (Syndicate, "reopen"),
        (Syndicate, "authorize:1"),
        (M26, "kidnap"),
        (M26, "kidnap:roll"),
        (M26, "terror:santiago"),
        (M26, "end-terror"),
        (Directorio, "event:unshaded"),
        (Directorio, "rally:beneficiary:26july"),
        (M26, "rally:sierra-maestra:place:4"),
    ];
    for (faction, action) in played {
        s.applied(&players[faction.index()], action);
    }
    s.human("advance-card");
    s.applied(&players[Government.index()], "government:assault");
    s.applied(&players[Government.index()], "assault:santiago:26july");
    s.applied(&players[Government.index()], "end-assault");

    let by_hand = s._dir.path().join("by-hand.json");
    let mut session = Session::create_seeded(&by_hand, 5).unwrap();
    for action in decisions(&s.room.path) {
        let seen = session.observe();
        let id = format!("{}:{action}", seen.revision);
        let owner = seen.choices.iter().find(|c| c.id == id).unwrap().owner;
        session.submit(owner, seen.revision, &id).unwrap();
    }
    assert_eq!(
        std::fs::read(&s.room.path).unwrap(),
        std::fs::read(&by_hand).unwrap()
    );
    assert_eq!(
        Session::open(&s.room.path, None).unwrap().game(),
        s.room.session.game()
    );
    assert!(
        !String::from_utf8(std::fs::read(&by_hand).unwrap())
            .unwrap()
            .contains("agent")
    );
}
