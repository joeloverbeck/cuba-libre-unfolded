//! The faction seats at the running table (#203): the socket agent sessions play on, and
//! the keys that hand a faction over and take it back. What a session may do and see is
//! the library's (`crate::faction_seats`), driven here once per frame; the windowless
//! table hosts it the same way, on the frames its seat's requests run.
//!
//! F1 to F4 hand Government, 26July, Directorio and Syndicate over, or take them back.
//! The socket opens at the first hand-over and closes with the table, which ends every
//! session.
use super::*;
use crate::faction_seats::{self as seats, Answer, Host, Player, Stands};
use crate::reports::Report;
use crate::save::Submit;
use serde_json::{Value, json};
use std::{path::Path, time::Instant};
use unfolded_tabletop::agent_transport::{Protocol, Reply, Transport, Unopened, window_waker};
use unfolded_tabletop::seat::socket::Unclaimable;

/// The key of each faction's seat, in `Faction::ALL`'s order.
pub(super) const KEYS: [KeyCode; 4] = [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4];

/// The same keys' names.
pub(super) const KEY_NAMES: [&str; 4] = ["F1", "F2", "F3", "F4"];

#[derive(Resource)]
pub(super) struct FactionSeats {
    host: Host<Reply>,
    /// Open from the first hand-over on.
    transport: Option<Transport>,
    socket: PathBuf,
}

impl FactionSeats {
    pub(super) fn new(socket: Option<PathBuf>) -> Self {
        Self {
            host: Host::new(),
            transport: None,
            socket: socket.unwrap_or_else(seats::default_socket),
        }
    }

    /// Hand `faction` over, or take it back: what the human is told.
    fn toggle(
        &mut self,
        faction: Faction,
        waker: impl FnOnce() -> Option<unfolded_tabletop::agent_transport::Waker>,
    ) -> notice::Notice {
        if self.host.player(faction) != Player::Human {
            send(self.host.take_back(faction));
            return notice::Notice::success(format!("You play {} again.", faction.name()));
        }
        if self.transport.is_none() {
            match Transport::open(&self.socket, PlayerProtocol, waker()) {
                Ok(transport) => self.transport = Some(transport),
                Err(refused) => return notice::Notice::error(unopened(&refused)),
            }
        }
        self.host.hand_over(faction);
        notice::Notice::success(format!(
            "{} is handed over, and an agent session may join as it on {}.",
            faction.name(),
            self.socket.display()
        ))
    }

    /// End every session and the socket.
    fn close(&mut self) {
        const REASON: &str = "The table closed.";
        send(self.host.close(REASON));
        if let Some(transport) = self.transport.take() {
            // What arrived and no frame will read.
            for incoming in transport.take() {
                incoming.reply.send(seats::closed(REASON).to_string());
            }
        }
    }
}

impl Drop for FactionSeats {
    fn drop(&mut self) {
        self.close();
    }
}

fn send(answers: Vec<Answer<Reply>>) {
    for (reply, answer) in answers {
        reply.send(answer.to_string());
    }
}

fn unopened(refused: &Unopened) -> String {
    match refused {
        Unopened::Unclaimed(refused) => match refused.cause() {
            Unclaimable::Answered => format!(
                "Cannot hand a faction over: another game answers on {}. Close it, or start this one with --player-socket.",
                refused.path().display()
            ),
            _ => format!("Cannot hand a faction over: {refused}"),
        },
        other => format!("Cannot hand a faction over: {other}"),
    }
}

/// The transport's own words: the table's end, and a wait with a period at a table that
/// runs no frames to end it.
struct PlayerProtocol;

impl Protocol for PlayerProtocol {
    fn players(&self) -> &str {
        "Faction seats"
    }

    fn closed(&self) -> String {
        seats::closed("The table closed.").to_string()
    }

    fn unanswered(&self, request: &str) -> Option<(Duration, String)> {
        let request: Value = serde_json::from_str(request).ok()?;
        let period = Duration::from_secs(request["timeout_seconds"].as_u64()?);
        let mut answer = seats::nothing_yet(request["player"].as_str()?);
        if request["op"] == "choose" {
            // The table may never have read the choice.
            answer["choice"] = json!({"id": request["id"], "status": "unknown"});
        }
        Some((
            period.saturating_add(Duration::from_secs(5)),
            answer.to_string(),
        ))
    }
}

/// The table as a session reaches it.
struct Sitting<'a> {
    play: &'a mut Play,
    ui: &'a mut Interface,
    settled: bool,
}

impl Sitting<'_> {
    fn session(&self) -> &Session {
        self.play.session.as_ref().expect("An admitted match")
    }
}

impl seats::Table for Sitting<'_> {
    fn game(&self) -> &Game {
        self.session().game()
    }

    fn reports(&self) -> Vec<Report> {
        self.session().reports()
    }

    fn settled(&self) -> bool {
        self.settled
    }

    fn commit(&mut self, faction: Faction, revision: u64, choice: &str) -> Result<(), String> {
        let before = self.session().observe();
        match submit(self.play, self.ui, faction, revision, choice) {
            Ok(after) => {
                self.ui.choice.agent_decided(&before, &after);
                // The next choice waits for this one's consequences.
                self.settled = false;
                self.ui.dirty = true;
                Ok(())
            }
            Err(failure) => {
                // The human is to know of a save that cannot be written.
                if matches!(failure, Submit::Save(_)) {
                    self.ui.notice = notice::Notice::failed(&failure);
                    self.ui.dirty = true;
                }
                Err(failure.to_string())
            }
        }
    }
}

/// The faction seats' frame: the human's hand-overs, then every request that arrived,
/// then the held ones. It runs after the human's own input and before the table shows
/// what was decided, so an agent's roll and card start from this frame's clock too.
pub(super) fn serve(world: &mut World) {
    if world.resource::<Play>().session.is_none() {
        return;
    }
    let now = Instant::now();
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let pressed: Vec<Faction> = Faction::ALL
        .into_iter()
        .filter(|faction| keys.just_pressed(KEYS[faction.index()]))
        .collect();
    let settled = holding(world).is_some_and(|holding| holding.is_empty());
    let waker = window_waker(world);
    world.resource_scope(|world, mut seats: Mut<FactionSeats>| {
        world.resource_scope(|world, mut play: Mut<Play>| {
            let mut ui = world.resource_mut::<Interface>();
            let seats = &mut *seats;
            for faction in pressed {
                let notice = seats.toggle(faction, || waker.clone());
                eprintln!("Faction seats: {}", notice.text);
                ui.notice = notice;
                ui.agents = Faction::ALL.map(|each| seats.host.player(each) != Player::Human);
                ui.dirty = true;
            }
            let Some(transport) = &seats.transport else {
                return;
            };
            let mut sitting = Sitting {
                play: &mut play,
                ui: &mut ui,
                settled,
            };
            // Before anything is told to it.
            seats.host.end_hung_up(Reply::hung_up, now);
            for incoming in transport.take() {
                send(
                    seats
                        .host
                        .receive(&incoming.line, incoming.reply, &mut sitting, now),
                );
            }
            send(seats.host.poll(&mut sitting, now));
        });
    });
}

/// The table closes with the app, and its sessions are told before the app is gone.
pub(super) fn close(mut exits: MessageReader<AppExit>, mut seats: ResMut<FactionSeats>) {
    if exits.read().next().is_some() {
        seats.close();
    }
}

/// What holds an agent's choice, when an agent can make one.
fn holding(world: &mut World) -> Option<Vec<String>> {
    world.resource::<FactionSeats>().transport.as_ref()?;
    Some(settled::holding(world))
}

/// Whether the seats have work that only frames do: a request nobody read, a dead client's
/// wait, an answer whose moment has come, or a held choice that waits only for time.
pub(super) fn due(world: &mut World) -> bool {
    let Some(holding) = holding(world) else {
        return false;
    };
    let showing = settled::result_showing(world);
    let seats = world.resource::<FactionSeats>();
    let Some(session) = &world.resource::<Play>().session else {
        return false;
    };
    let held = Faction::ALL
        .into_iter()
        .any(|faction| seats.host.holds_choice(faction));
    seats.transport.as_ref().is_some_and(Transport::waiting)
        || seats.host.hung_up(Reply::hung_up)
        || seats
            .host
            .due(session.game(), holding.is_empty(), Instant::now())
        || (held && showing)
}

/// Each faction's seat as the development seat reports it, with what holds an agent's
/// choice and which agent's faction the table waits for.
pub(super) fn observe(world: &mut World) -> Value {
    let holding = holding(world);
    let seats = world.resource::<FactionSeats>();
    let Some(session) = &world.resource::<Play>().session else {
        return Value::Null;
    };
    let game = session.game();
    let now = Instant::now();
    let factions: Vec<Value> = Faction::ALL
        .into_iter()
        .map(|faction| {
            let mut seat = json!(seats.host.state(faction, game, now));
            seat["faction"] = json!(faction);
            seat["waits"] = json!(seats.host.waits(faction));
            seat["holds_choice"] = json!(seats.host.holds_choice(faction));
            seat
        })
        .collect();
    let owner = game.owner();
    let awaited = (Stands::of(game) == Stands::Open && seats.host.player(owner) != Player::Human)
        .then_some(owner);
    json!({
        "factions": factions,
        "awaits": awaited,
        "holding": holding,
        "socket": seats.transport.as_ref().map(|_| Path::new(&seats.socket)),
    })
}
