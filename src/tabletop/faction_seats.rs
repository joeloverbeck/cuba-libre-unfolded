//! The faction seats at the running table (#203): the socket agent sessions play on, and
//! what the human's seat controls do (`seat_cards`, #206). What a session may do and see
//! is the library's (`crate::faction_seats`), driven here once per frame; the windowless
//! table hosts it the same way, on the frames its seat's requests run.
//!
//! The controls and F1 to F4 hand Government, 26July, Directorio and Syndicate over, and
//! take them back. The socket opens at the first hand-over and closes with the table,
//! which ends every session.
use super::seat_cards::{Line, Press};
use super::*;
use crate::faction_seats::{self as seats, Answer, Host, Player, SeatState, Stands};
use crate::player_folders::Places;
use crate::reports::Report;
use crate::save::{Proclamation, Submit};
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
    clipboard: Clipboard,
}

/// Where a seat's line is copied to.
enum Clipboard {
    /// The desktop's, opened at the first copy. It holds the text while the table runs.
    Desktop(Option<arboard::Clipboard>),
    /// The windowless table's own: what was copied last. It never reaches the desktop's.
    Own(Option<String>),
}

impl Clipboard {
    fn copy(&mut self, text: &str) -> Result<(), String> {
        match self {
            Self::Desktop(clipboard) => {
                let clipboard = match clipboard {
                    Some(clipboard) => clipboard,
                    None => clipboard
                        .insert(arboard::Clipboard::new().map_err(|error| error.to_string())?),
                };
                clipboard.set_text(text).map_err(|error| error.to_string())
            }
            Self::Own(copied) => {
                *copied = Some(text.to_owned());
                Ok(())
            }
        }
    }
}

impl FactionSeats {
    pub(super) fn new(socket: Option<PathBuf>, windowless: bool) -> Self {
        Self {
            host: Host::new(),
            transport: None,
            socket: socket.unwrap_or_else(seats::default_socket),
            clipboard: if windowless {
                Clipboard::Own(None)
            } else {
                Clipboard::Desktop(None)
            },
        }
    }

    /// Do what the human pressed, and what the notice of it is. The card shows a
    /// hand-over, a take-back and a copied line, so only a failure is drawn.
    fn press(
        &mut self,
        press: Press,
        line: &mut Option<Line>,
        waker: impl FnOnce() -> Option<unfolded_tabletop::agent_transport::Waker>,
    ) -> Option<notice::Notice> {
        match press {
            Press::HandOver(faction) if self.host.player(faction) == Player::Human => {
                Some(self.hand_over(faction, waker))
            }
            Press::TakeBack(faction) if self.host.player(faction) != Player::Human => {
                send(self.host.take_back(faction));
                Some(notice::Notice::success(format!(
                    "You play {} again.",
                    faction.name()
                )))
            }
            Press::Copy(faction) if self.host.player(faction) == Player::Nobody => {
                let text = seats::line(faction, &self.socket, Places::here().as_ref());
                let failed = self.clipboard.copy(&text).err();
                if let Some(failed) = &failed {
                    // The sheet says so, and shows the line to type.
                    eprintln!("Faction seats: the line was not copied: {failed}");
                }
                *line = Some(Line {
                    faction,
                    text,
                    failed,
                });
                None
            }
            // The seat changed hands before the press was read.
            _ => None,
        }
    }

    fn hand_over(
        &mut self,
        faction: Faction,
        waker: impl FnOnce() -> Option<unfolded_tabletop::agent_transport::Waker>,
    ) -> notice::Notice {
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
            transport.close();
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

    fn proclamations(&self) -> &[Proclamation] {
        self.session().proclamations()
    }

    fn settled(&self) -> bool {
        // A publication can open the reader inside this batch. Later requests and
        // the final poll must consult the modal even before the next frame.
        self.settled && !self.ui.proclamation_reader.is_open()
    }

    fn commit(&mut self, faction: Faction, revision: u64, choice: &str) -> Result<(), String> {
        match submit(self.play, self.ui, faction, revision, choice) {
            Ok(after) => {
                self.ui.choice.agent_decided(&after);
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
    fn publish(
        &mut self,
        faction: Faction,
        revision: u64,
        card: u8,
        text: &str,
    ) -> Result<(), String> {
        let result = self
            .play
            .session
            .as_mut()
            .expect("An admitted match")
            .publish(faction, revision, card, text);
        match &result {
            Ok(()) => {
                let session = self.play.session.as_ref().expect("An admitted match");
                self.ui.proclamation_reader.receive(session);
                self.ui.dirty = true;
            }
            Err(failure @ Submit::Save(_)) => {
                self.ui.notice = notice::Notice::failed(failure);
                self.ui.dirty = true;
            }
            Err(Submit::Refused(_)) => {}
        }
        result.map_err(|failure| failure.to_string())
    }
}

/// The faction seats' frame: what the human pressed, then every request that arrived,
/// then the held ones, and the seats' states as they are then. It runs after the human's
/// own input and before the table shows what was decided, so an agent's roll and card
/// start from this frame's clock too.
pub(super) fn serve(world: &mut World) {
    if world.resource::<Play>().session.is_none() {
        return;
    }
    let now = Instant::now();
    let mut pressed = std::mem::take(&mut world.resource_mut::<Interface>().presses);
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let host = &world.resource::<FactionSeats>().host;
    pressed.extend(
        Faction::ALL
            .into_iter()
            .filter(|faction| keys.just_pressed(KEYS[faction.index()]))
            .map(|faction| match host.player(faction) {
                Player::Human => Press::HandOver(faction),
                _ => Press::TakeBack(faction),
            }),
    );
    let settled = holding(world).is_some_and(|holding| holding.is_empty());
    let waker = window_waker(world);
    world.resource_scope(|world, mut seats: Mut<FactionSeats>| {
        world.resource_scope(|world, mut play: Mut<Play>| {
            let mut ui = world.resource_mut::<Interface>();
            let seats = &mut *seats;
            for press in pressed {
                if let Some(notice) = seats.press(press, &mut ui.line, || waker.clone()) {
                    eprintln!("Faction seats: {}", notice.text);
                    ui.notice = notice;
                }
                ui.dirty = true;
            }
            let Some(transport) = &seats.transport else {
                return;
            };
            // What the human handed over takes none of their input from this frame on.
            show(seats, &play, &mut ui, now);
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
            show(seats, &play, &mut ui, now);
        });
    });
}

/// Have the window show the seats' states, and put the sheet away once its seat no longer
/// waits for a session.
fn show(seats: &FactionSeats, play: &Play, ui: &mut Interface, now: Instant) {
    let Some(session) = &play.session else {
        return;
    };
    let players = seats.host.players();
    let mixed = players.contains(&Player::Human) && players.contains(&Player::Agent);
    if ui.mixed_seats != mixed {
        ui.mixed_seats = mixed;
        ui.dirty = true;
        if !mixed && ui.choice.focus.as_deref() == Some(proclamation_composer::WRITE) {
            ui.choice.focus = None;
        }
        if !mixed && ui.transfer.focus.as_deref() == Some(proclamation_composer::WRITE) {
            ui.transfer.focus = None;
        }
    }
    let states = seats.host.states(session.game(), now);
    if ui.seats != states {
        let played = ui.played();
        ui.seats = states;
        ui.dirty = true;
        if ui.played() != played {
            let played = ui.played();
            ui.editions.seated(
                &session.observe(),
                |told| session.reports_since(told),
                played,
            );
        }
    }
    if ui
        .line
        .as_ref()
        .is_some_and(|line| states[line.faction.index()] != SeatState::Awaiting)
    {
        ui.line = None;
        ui.dirty = true;
    }
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
/// wait, an answer whose moment has come, a held choice that waits only for time, or a
/// state the window does not show yet, such as a silence a minute longer.
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
    let now = Instant::now();
    seats.transport.as_ref().is_some_and(Transport::waiting)
        || world.resource::<Interface>().seats != seats.host.states(session.game(), now)
        || seats.host.hung_up(Reply::hung_up)
        || seats.host.due(session.game(), holding.is_empty(), now)
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
        "copied": match &seats.clipboard {
            Clipboard::Own(copied) => copied.as_deref(),
            Clipboard::Desktop(_) => None,
        },
    })
}
