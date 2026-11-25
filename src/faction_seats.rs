//! Faction seats (#203, foundation 5): what an external agent session may do and see at
//! a running match, with no window. A session joins as one faction the human handed
//! over, waits until that faction has a decision, observes its view, makes a choice at
//! the revision it saw, and leaves. The table hosts the socket and drives the [`Host`]
//! once per frame (`tabletop::faction_seats`).
//!
//! Who holds each seat, the held choices and the waits are `unfolded_agent_players::seats`,
//! which no game owns. What follows here is Cuba Libre's: who decides, what a faction may
//! know ([`View`]), the requests and the answers' words.
//!
//! **Who decides** is asked of the rules authority for each choice, never for the
//! position as a whole: while a giver decides a Transfer request, 26July's withdrawal
//! stays its own. A session's event is that its faction is the deciding one and has a
//! choice this build plays. A faction that only owns a withdrawal sees it in its view and
//! is not woken for it, nor is a giver whose session answered Wait, until a new decision
//! comes. Turning the card is the human's and is no agent's choice.
//!
//! A choice goes through [`Game::submit`] with the faction, the revision and the choice's
//! id, by way of the table's own commit, which saves it before it is shown. It is applied
//! at once when the table is settled, and held otherwise. Who plays which faction is
//! runtime state, in no save.

mod view;

pub use view::{Stands, View};

use crate::{Faction, Game, Observation, Transfer, reports::Report};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use unfolded_agent_players::seats::{
    self, ClaimRefused, Ended, Event, Holder, Joining, Pending, Refusal, Released, Seats,
    Submitted, Told, Waiting,
};

/// Who plays a faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Player {
    /// The human at the table.
    Human,
    /// Handed over, and no session has joined as it. It waits and never acts.
    Nobody,
    /// An agent session.
    Agent,
}

/// What the table shows of one faction's seat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "state")]
pub enum SeatState {
    Human,
    /// Handed over, and waiting for a session to join.
    Awaiting,
    /// A session holds it, and nothing is asked of it.
    AtTable,
    /// Its session's decision is open.
    Thinking,
    /// Its session's decision is open, and the session was not heard from for this long.
    Silent {
        minutes: u64,
    },
    /// It is the deciding faction, and this build plays none of its options.
    BuildEnds,
}

/// The table a session plays at, as the host reaches it.
pub trait Table {
    fn game(&self) -> &Game;
    /// A report of each action of the match so far, whole, in order.
    fn reports(&self) -> Vec<Report>;
    /// Whether a choice applied now would be seen: nothing is in motion and nothing of
    /// the human's is open.
    fn settled(&self) -> bool;
    /// Submit `choice` as `faction` through the one checked submission, saved before it
    /// is shown. An error leaves the position and the save as they were.
    fn commit(&mut self, faction: Faction, revision: u64, choice: &str) -> Result<(), String>;
}

/// An answer to a request, for the connection that sent it.
pub type Answer<R> = (R, Value);

/// A choice a session made, with its label then, to name it once the match moved on.
#[derive(Clone, Debug)]
struct Chosen {
    id: String,
    label: String,
}

/// How a submission stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Standing {
    Applied,
    Held,
    Released,
}

impl Standing {
    fn word(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Held => "held",
            Self::Released => "released",
        }
    }
}

/// What the host keeps of a session beside its seat.
struct Session {
    id: String,
    /// The submission its held wait follows, when the request both submitted and waited.
    follows: Option<(Chosen, Standing)>,
    /// The revision its own Wait on a Transfer request made, while nothing else has
    /// happened: its faction still decides, and it has no new decision.
    set_aside: Option<u64>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Join {
        faction: String,
        /// The identity an earlier join gave the session, if it has one.
        #[serde(default)]
        player: Option<String>,
        /// Its tool's identifiers of the session, when the tool gives any.
        #[serde(default)]
        session: Option<String>,
    },
    Wait {
        player: String,
        #[serde(default)]
        timeout_seconds: Option<u64>,
    },
    Observe {
        player: String,
    },
    Choose {
        player: String,
        revision: u64,
        id: String,
        /// Then wait, on the same answer, for what needs the session next.
        #[serde(default)]
        wait: bool,
        #[serde(default)]
        timeout_seconds: Option<u64>,
    },
    Leave {
        player: String,
    },
}

/// The faction seats of one table, the sessions that hold them and their requests in
/// progress. `R` is how the transport answers a request.
pub struct Host<R> {
    seats: Seats<Faction, Chosen, R>,
    sessions: Vec<Session>,
}

impl<R> Default for Host<R> {
    fn default() -> Self {
        Self {
            seats: Seats::new(Faction::ALL),
            sessions: Vec::new(),
        }
    }
}

/// The rules and the table, as a submission reaches them.
struct Rules<'a, T> {
    table: &'a mut T,
    /// The revision that answering Wait made, for the faction that did.
    set_aside: Option<(Faction, u64)>,
}

impl<T: Table> seats::Game for Rules<'_, T> {
    type Seat = Faction;
    type Choice = Chosen;

    fn revision(&self) -> u64 {
        self.table.game().revision
    }

    fn settled(&self) -> bool {
        self.table.settled()
    }

    fn check(&self, seat: &Faction, choice: &Chosen) -> Result<(), String> {
        let game = self.table.game();
        if game
            .choices()
            .iter()
            .any(|offered| offered.id == choice.id && offered.turns_the_card())
        {
            return Err("The human turns the card; it is no agent's choice.".into());
        }
        // The same submission, on a copy that goes nowhere.
        game.clone().submit(*seat, game.revision, &choice.id)
    }

    fn commit(&mut self, seat: &Faction, choice: &Chosen) -> Result<(), String> {
        let revision = self.revision();
        self.table.commit(*seat, revision, &choice.id)?;
        let game = self.table.game();
        if waited_on(game) == Some(*seat) {
            self.set_aside = Some((*seat, game.revision));
        }
        Ok(())
    }
}

/// The giver that set a pending Transfer request aside, if one did.
fn waited_on(game: &Game) -> Option<Faction> {
    match &game.transfer {
        Some(Transfer::Authorizing {
            giver,
            waited: true,
            ..
        }) => Some(*giver),
        _ => None,
    }
}

/// The faction whose decision this build plays now, whoever plays the faction.
fn decides(game: &Game) -> Option<Faction> {
    let owner = game.owner();
    game.choices()
        .iter()
        .any(|choice| choice.owner == owner && choice.supported && !choice.turns_the_card())
        .then_some(owner)
}

impl<R> Host<R> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player(&self, faction: Faction) -> Player {
        match self.seats.holder(&faction) {
            Holder::Human => Player::Human,
            Holder::Unclaimed => Player::Nobody,
            Holder::Session(_) => Player::Agent,
        }
    }

    pub fn players(&self) -> [Player; 4] {
        Faction::ALL.map(|faction| self.player(faction))
    }

    /// Hand `faction` over, for a session to join as.
    pub fn hand_over(&mut self, faction: Faction) {
        self.seats.hand_over(&faction);
    }

    /// The human plays `faction` again. The session that held it ends: its wait, and each
    /// later request by its identity, is answered that it ended, and a choice of its that
    /// was held is not applied.
    #[must_use = "The answers to the ended session's held requests"]
    pub fn take_back(&mut self, faction: Faction) -> Vec<Answer<R>> {
        let held = match self.seats.holder(&faction) {
            Holder::Session(id) => self
                .seats
                .held(&id)
                .map(|(_, choice)| format!(" Your held choice, {}, was not applied.", choice.label))
                .unwrap_or_default(),
            _ => String::new(),
        };
        let reason = format!(
            "The human plays {} now, so your session ended: stop.{held}",
            faction.name()
        );
        let ended = self.seats.take_back(&faction, &reason);
        self.ended(ended, &reason)
    }

    /// End every session, for `reason`: the table closes.
    #[must_use = "The answers to the ended sessions' held requests"]
    pub fn close(&mut self, reason: &str) -> Vec<Answer<R>> {
        let ended = self.seats.close(reason);
        self.ended(ended, reason)
    }

    fn ended(
        &mut self,
        ended: impl IntoIterator<Item = Ended<Chosen, R>>,
        reason: &str,
    ) -> Vec<Answer<R>> {
        let mut answers = Vec::new();
        for ended in ended {
            self.sessions.retain(|known| known.id != ended.identity);
            answers.extend(ended.wait.map(|reply| (reply, closed(reason))));
        }
        answers
    }

    /// What the table shows of `faction`'s seat.
    pub fn state(&self, faction: Faction, game: &Game, now: Instant) -> SeatState {
        match self.seats.holder(&faction) {
            Holder::Human => return SeatState::Human,
            _ if game.owner() == faction && Stands::of(game) == Stands::BuildEnds => {
                return SeatState::BuildEnds;
            }
            Holder::Unclaimed => return SeatState::Awaiting,
            Holder::Session(_) => {}
        }
        if decides(game) != Some(faction) {
            return SeatState::AtTable;
        }
        let silence = self
            .seats
            .silence(&faction, self.decision(faction, game), now)
            .unwrap_or_default();
        match silence.as_secs() / 60 {
            0 => SeatState::Thinking,
            minutes => SeatState::Silent { minutes },
        }
    }

    /// Whether `faction`'s session has a choice that waits for the table to settle.
    pub fn holds_choice(&self, faction: Faction) -> bool {
        matches!(self.seats.holder(&faction), Holder::Session(id) if self.seats.held(&id).is_some())
    }

    /// Whether `faction`'s session waits.
    pub fn waits(&self, faction: Faction) -> bool {
        matches!(self.seats.holder(&faction), Holder::Session(id) if self.seats.waits(&id))
    }

    /// Whether the client of a held wait has died, as `hung_up` says of its reply.
    pub fn hung_up(&self, hung_up: impl Fn(&R) -> bool) -> bool {
        self.seats.hung_up(hung_up)
    }

    /// End each held wait whose client has died, before anything is told to it: its seat's
    /// silence counts from now, and what it would have been told stays to be told.
    pub fn end_hung_up(&mut self, hung_up: impl Fn(&R) -> bool, now: Instant) {
        for id in self.seats.end_hung_up(hung_up, now) {
            self.follow(&id, None);
        }
    }

    /// Whether [`poll`](Self::poll) would do anything now at a table that is `settled` or
    /// not.
    pub fn due(&self, game: &Game, settled: bool, now: Instant) -> bool {
        self.seats.settle_due(game.revision, settled)
            || self
                .seats
                .sessions()
                .any(|id| self.seats.would_tell(id, self.events(id, game), now))
    }

    /// Answer one request line now, or hold it: the answers to held ones come from here
    /// and from [`poll`](Self::poll).
    pub fn receive(
        &mut self,
        line: &str,
        reply: R,
        table: &mut impl Table,
        now: Instant,
    ) -> Vec<Answer<R>> {
        let request = match serde_json::from_str::<Request>(line) {
            Ok(request) => request,
            Err(error) => {
                let player = serde_json::from_str::<Value>(line)
                    .ok()
                    .and_then(|value| value["player"].as_str().map(str::to_owned));
                let reason = format!(
                    "Not a request ({error}). A player may join, wait, observe, choose or leave."
                );
                let answer = match player.filter(|id| self.seats.seat(id).is_some()) {
                    Some(id) => {
                        self.seats.heard(&id, now);
                        self.answer(&id, "refused", Some(reason), table)
                    }
                    None => refused(reason),
                };
                return vec![(reply, answer)];
            }
        };
        let id = match &request {
            Request::Join {
                faction,
                player,
                session,
            } => {
                let joining = Joining {
                    identity: player.as_deref(),
                    session: session.as_deref(),
                };
                return vec![(reply, self.join(faction, joining, table, now))];
            }
            Request::Wait { player, .. }
            | Request::Observe { player }
            | Request::Choose { player, .. }
            | Request::Leave { player } => player.clone(),
        };
        let Some(&faction) = self.seats.seat(&id) else {
            let answer = match self.seats.ended(&id) {
                Some(reason) => closed(reason),
                None => refused(format!(
                    "No session {id} holds a seat at this table. Join first; an identity ends with its session."
                )),
            };
            return vec![(reply, answer)];
        };
        self.seats.heard(&id, now);
        match request {
            Request::Join { .. } => unreachable!("Answered above"),
            Request::Observe { .. } => {
                vec![(reply, self.answer(&id, "observed", None, table))]
            }
            Request::Leave { .. } => {
                let left = self.seats.leave(&id);
                let mut answers = self.ended(left, "You left the table.");
                answers.push((
                    reply,
                    json!({"ok": true, "mode": MODE, "status": "left", "player": id}),
                ));
                answers
            }
            Request::Wait {
                timeout_seconds, ..
            } => {
                self.settle(table);
                let pending = self.pending(faction, table.game());
                let period = timeout_seconds.map(Duration::from_secs);
                match self.seats.wait(&id, reply, period, pending, now) {
                    Waiting::Refused(reply) => {
                        let reason =
                            "You were told of this decision, and it waits for you: choose.";
                        vec![(
                            reply,
                            self.answer(&id, "refused", Some(reason.into()), table),
                        )]
                    }
                    Waiting::Held { replaced } => {
                        self.follow(&id, None);
                        let mut answers: Vec<_> = replaced
                            .map(|older| (older, replaced_by(&id)))
                            .into_iter()
                            .collect();
                        answers.extend(self.poll(table, now));
                        answers
                    }
                }
            }
            Request::Choose {
                revision,
                id: choice,
                wait,
                timeout_seconds,
                ..
            } => {
                let submitted = self.choose(&id, revision, &choice, table);
                let then = wait.then(|| timeout_seconds.map(Duration::from_secs));
                self.submitted((&id, reply), submitted, then, table, now)
            }
        }
    }

    /// Apply or release the choices whose moment has come, then answer every wait that has
    /// something to be told.
    pub fn poll(&mut self, table: &mut impl Table, now: Instant) -> Vec<Answer<R>> {
        self.settle(table);
        let game = table.game();
        let mut seen: Option<Observation> = None;
        let mut answers = Vec::new();
        let waiting: Vec<String> = self.seats.sessions().map(str::to_owned).collect();
        for id in waiting {
            let events = self.events(&id, game);
            let Some((reply, told)) = self.seats.tell(&id, events, now) else {
                continue;
            };
            let follows = self.follow(&id, None);
            let (status, reason) = match told {
                Told::Decision { .. } => ("decision", None),
                Told::Released { released, .. } => ("released", Some(unapplied(&released))),
                Told::Elapsed => {
                    let mut answer = nothing_yet(&id);
                    said_of(&mut answer, follows);
                    answers.push((reply, answer));
                    continue;
                }
                Told::News(nothing) => match nothing {},
            };
            let seen = seen.get_or_insert_with(|| game.observe());
            let mut answer = self.view_answer(&id, status, seen, table.reports());
            if let Some(reason) = reason {
                answer["reason"] = json!(reason);
            }
            said_of(&mut answer, follows);
            answers.push((reply, answer));
        }
        answers
    }

    /// Seat one session as the faction it names, at once or not at all.
    fn join(&mut self, named: &str, joining: Joining, table: &impl Table, now: Instant) -> Value {
        match (self.seats.recognise(joining), joining.identity) {
            (Err(ClaimRefused::Holds(faction)), Some(id)) => {
                let reason = format!(
                    "You already play {}, and one session plays one faction. Nothing changed: go on with your identity.",
                    faction.name()
                );
                let id = id.to_owned();
                self.seats.heard(&id, now);
                return self.answer(&id, "refused", Some(reason), table);
            }
            (Err(ClaimRefused::SameSession(faction)), _) => {
                return refused(format!(
                    "This session already plays {}, and one session plays one faction. Nothing changed: go on with the identity your join gave you.",
                    faction.name()
                ));
            }
            _ => {}
        }
        let open: Vec<&str> = Faction::ALL
            .into_iter()
            .filter(|faction| self.player(*faction) == Player::Nobody)
            .map(Faction::name)
            .collect();
        let refuse = |why: String| {
            refused(match open[..] {
                [] => format!(
                    "{why} No faction is open to a session now: there is nothing for you to play, so stop."
                ),
                _ => format!("{why} Open to a session now: {}.", open.join(", ")),
            })
        };
        let Some(faction) = Faction::named(named) else {
            return refuse(format!("There is no faction \"{named}\"."));
        };
        let id = match self.seats.claim(&faction, joining, now) {
            Ok(id) => id,
            Err(ClaimRefused::Held) => {
                return refuse(format!("Another session already plays {}.", faction.name()));
            }
            Err(_) => {
                return refuse(format!(
                    "The human plays {} and has not handed it over.",
                    faction.name()
                ));
            }
        };
        self.sessions.push(Session {
            id: id.clone(),
            follows: None,
            set_aside: None,
        });
        // The seat brief (#204) is given here, once.
        self.answer(&id, "joined", None, table)
    }

    fn choose(
        &mut self,
        id: &str,
        revision: u64,
        choice: &str,
        table: &mut impl Table,
    ) -> Result<(Chosen, Standing), String> {
        // A choice held earlier, this session's or another's, comes first.
        self.settle(table);
        let chosen = Chosen {
            id: choice.to_owned(),
            label: table
                .game()
                .choices()
                .into_iter()
                .find(|offered| offered.id == choice)
                .map_or_else(|| choice.to_owned(), |offered| offered.label),
        };
        let mut rules = Rules {
            table,
            set_aside: None,
        };
        let submitted = self.seats.submit(id, revision, chosen.clone(), &mut rules);
        let set_aside = rules.set_aside;
        self.keep_aside(set_aside);
        match submitted {
            Submitted::Applied => Ok((chosen, Standing::Applied)),
            Submitted::Held => Ok((chosen, Standing::Held)),
            Submitted::Refused(refusal) => Err(why(&refusal)),
        }
    }

    /// Answer a submission at once with how it stands, or, when it waits too (`then`) and
    /// its own next decision is not there to tell, hold its wait.
    fn submitted(
        &mut self,
        (id, mut reply): (&str, R),
        submitted: Result<(Chosen, Standing), String>,
        then: Option<Option<Duration>>,
        table: &mut impl Table,
        now: Instant,
    ) -> Vec<Answer<R>> {
        let faction = *self.seats.seat(id).expect("Seated");
        // This answer shows the decision that is now its own.
        if let Pending::Decision(revision) = self.pending(faction, table.game()) {
            self.seats.told_in_answer(id, revision);
        }
        let (status, error) = match &submitted {
            Ok((_, standing)) => (standing.word(), None),
            Err(reason) => ("refused", Some(reason.clone())),
        };
        let mut answers = Vec::new();
        if let (Ok((chosen, standing)), Some(period)) = (submitted, then) {
            let pending = self.pending(faction, table.game());
            match self.seats.wait(id, reply, period, pending, now) {
                Waiting::Held { replaced } => {
                    answers.extend(replaced.map(|older| (older, replaced_by(id))));
                    self.follow(id, Some((chosen, standing)));
                    answers.extend(self.poll(table, now));
                    return answers;
                }
                Waiting::Refused(told) => reply = told,
            }
        }
        answers.push((reply, self.answer(id, status, error, table)));
        answers.extend(self.poll(table, now));
        answers
    }

    /// Apply the held choices whose moment has come, and release those the match has
    /// passed or the rules now refuse.
    fn settle(&mut self, table: &mut impl Table) {
        if !self
            .seats
            .settle_due(table.game().revision, table.settled())
        {
            return;
        }
        let mut rules = Rules {
            table,
            set_aside: None,
        };
        let settled = self.seats.settle(&mut rules);
        let set_aside = rules.set_aside;
        self.keep_aside(set_aside);
        for settled in settled {
            if let Some(session) = self.session_mut(&settled.identity)
                && let Some((_, standing)) = &mut session.follows
            {
                *standing = if settled.applied {
                    Standing::Applied
                } else {
                    Standing::Released
                };
            }
        }
    }

    fn keep_aside(&mut self, set_aside: Option<(Faction, u64)>) {
        if let Some((faction, revision)) = set_aside
            && let Holder::Session(id) = self.seats.holder(&faction)
            && let Some(session) = self.session_mut(&id)
        {
            session.set_aside = Some(revision);
        }
    }

    fn session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|known| known.id == id)
    }

    /// Set what the held wait of the session `id` follows: what it followed.
    fn follow(
        &mut self,
        id: &str,
        follows: Option<(Chosen, Standing)>,
    ) -> Option<(Chosen, Standing)> {
        let session = self.session_mut(id)?;
        std::mem::replace(&mut session.follows, follows)
    }

    /// The revision of `faction`'s own decision that its session may be told of.
    fn decision(&self, faction: Faction, game: &Game) -> Option<u64> {
        let revision = game.revision;
        let set_aside = match self.seats.holder(&faction) {
            Holder::Session(id) => self
                .sessions
                .iter()
                .any(|known| known.id == id && known.set_aside == Some(revision)),
            _ => false,
        };
        (decides(game) == Some(faction) && !set_aside).then_some(revision)
    }

    /// What there is for `faction`'s session as it waits.
    fn pending(&self, faction: Faction, game: &Game) -> Pending {
        self.decision(faction, game)
            .map_or(Pending::Nothing, Pending::Decision)
    }

    /// What a held wait of the session `id` may be told, beside its released choices.
    fn events(&self, id: &str, game: &Game) -> Option<Event<std::convert::Infallible>> {
        let faction = *self.seats.seat(id)?;
        self.decision(faction, game).map(Event::Decision)
    }

    /// An answer that is not a wait's: how the request went, with the view.
    fn answer(
        &mut self,
        id: &str,
        status: &str,
        error: Option<String>,
        table: &impl Table,
    ) -> Value {
        let released = self.seats.read(id);
        let mut answer = self.view_answer(id, status, &table.game().observe(), table.reports());
        if let Some(error) = error {
            answer["ok"] = json!(false);
            answer["error"] = json!(error);
        }
        if !released.is_empty() {
            answer["released"] = released
                .iter()
                .map(|released| {
                    json!({"id": released.choice.id, "label": released.choice.label, "reason": unapplied(released)})
                })
                .collect();
        }
        answer
    }

    fn view_answer(
        &self,
        id: &str,
        status: &str,
        seen: &Observation,
        reports: Vec<Report>,
    ) -> Value {
        let faction = *self.seats.seat(id).expect("Seated");
        json!({
            "ok": true, "mode": MODE, "status": status, "player": id,
            "view": View::of(seen, reports, faction, self.players()),
        })
    }
}

const MODE: &str = "faction";

fn refused(reason: String) -> Value {
    json!({"ok": false, "mode": MODE, "status": "refused", "error": reason})
}

/// The answer to a session that ended, and to any later request of its identity.
pub fn closed(reason: &str) -> Value {
    json!({"ok": true, "mode": MODE, "status": "closed", "reason": reason})
}

/// The answer to a wait whose period ended.
pub fn nothing_yet(player: &str) -> Value {
    json!({"ok": true, "mode": MODE, "status": "nothing_yet", "player": player})
}

fn replaced_by(player: &str) -> Value {
    json!({
        "ok": true, "mode": MODE, "status": "replaced", "player": player,
        "reason": "A newer wait of this session took this one's place.",
    })
}

/// Say in the answer to a wait that followed a submission how the submission stands.
fn said_of(answer: &mut Value, follows: Option<(Chosen, Standing)>) {
    if let Some((chosen, standing)) = follows {
        answer["choice"] =
            json!({"id": chosen.id, "label": chosen.label, "status": standing.word()});
    }
}

fn why(refusal: &Refusal) -> String {
    match refusal {
        Refusal::Unseated => "No session of that identity holds a seat.".into(),
        Refusal::Stale { current } => {
            format!("Stale: the match is at revision {current}. Choose again from this view.")
        }
        Refusal::Game(reason) => reason.clone(),
    }
}

fn unapplied(released: &Released<Chosen>) -> String {
    let why = match &released.why {
        Refusal::Stale { .. } => "the match moved on before it could be applied.",
        Refusal::Unseated => "its session had ended.",
        Refusal::Game(reason) => reason,
    };
    format!("{} was not applied: {why}", released.choice.label)
}

/// Where a table hosts its faction seats unless it is told otherwise:
/// `$CUBA_LIBRE_PLAYER_SOCKET`, else `cuba-libre-players.sock` in `$XDG_RUNTIME_DIR` or,
/// without it, the temporary directory. The player's client finds the same path.
pub fn default_socket() -> PathBuf {
    if let Some(path) = std::env::var_os("CUBA_LIBRE_PLAYER_SOCKET") {
        return PathBuf::from(path);
    }
    std::env::var_os("XDG_RUNTIME_DIR")
        .map_or_else(std::env::temp_dir, PathBuf::from)
        .join("cuba-libre-players.sock")
}
