//! Action reports (#207): what each faction's action did, as committed facts. A report is
//! derived, with no window, from the observations before and after the committed
//! decisions of one action, and from nothing else: every number and place in it is the
//! rules authority's. A faction seat's view gives the match's reports as its recent
//! results, and an edition prints one.
//!
//! An action belongs to the faction whose action it is, a Pass too (which is not
//! executing), even while another faction decides inside it: a giver its Resource
//! transfer, or 26July the free Rally that Directorio's Event grants it. Turning the card
//! is in no action.
//!
//! [`Reports::since`] reports what followed a revision, so a report starts where the
//! previous one ended and nothing is told twice. The words are fixed per kind of action
//! (`words`): they state what changed and judge nothing.
//!
//! A report does not grow with the number of side decisions: the Resource transfers from
//! one faction to another are told together, and so are a faction's own removed pieces.

mod words;

/// The table's notices list as the reports do.
#[cfg(feature = "tabletop")]
pub(crate) use words::listing;

use crate::{
    ActionChoice, AssaultResolution, Choice, Eligibility, EventOutcome, EventProgress, Faction,
    Flip, Game, Group, Kidnap, Kind, Next, Observation, Rally, RallyAction, Removal, Resolved,
    SequenceBox, Space, Standing, State, Transfer, title,
};
use serde::Serialize;

/// Which action a report is of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    /// No action is chosen: 26July went back to its action choice.
    Undecided,
    Pass,
    Terror,
    EventUnshaded,
    EventShaded,
    Assault,
}

/// What one faction's action did between two revisions of the match.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The faction whose action it is.
    pub faction: Faction,
    pub action: Action,
    pub card: u8,
    pub card_title: &'static str,
    /// The faction's role on the card as the action began: "1st Eligible" or "2nd Eligible".
    pub role: &'static str,
    /// The Sequence of Play box the action is recorded in, once it is.
    pub recorded: Option<&'static str>,
    /// The revision it starts after, where the previous report ended.
    pub from_revision: u64,
    pub to_revision: u64,
    /// Whether an earlier report told the action's beginning.
    pub continues: bool,
    /// Whether the action has ended.
    pub complete: bool,
    /// The names of the spaces where something happened, each once.
    pub spaces: Vec<String>,
    pub headline: String,
    pub summary: Option<String>,
    /// What happened, in order.
    pub articles: Vec<Article>,
    /// Every faction's Resources and victory standing, changed or not.
    pub standings: Vec<Row>,
    /// What else differs between the two revisions: Control, Available Forces, markers,
    /// victory totals and eligibility, each only if it changed.
    pub notes: Vec<Article>,
}

/// A title and its sentences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Article {
    pub title: String,
    pub lines: Vec<String>,
    /// The die it tells of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub die: Option<u8>,
}

/// One faction's row of the standings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Row {
    pub faction: Faction,
    pub resources: FromTo,
    /// The total its victory counts (R 7.2); the Syndicate's is its open Casinos.
    pub score: FromTo,
    pub margin: FromTo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FromTo {
    pub from: i16,
    pub to: i16,
}

/// The match's committed positions, from which its reports are derived.
#[derive(Clone, Debug)]
pub struct Reports {
    moments: Vec<Moment>,
}

impl Reports {
    /// The reports of a match that stands at `seen`: none yet.
    pub fn new(seen: &Observation) -> Self {
        Self {
            moments: vec![Moment::of(seen)],
        }
    }

    /// Takes in the position one committed decision made.
    pub fn record(&mut self, seen: &Observation) {
        self.moments.push(Moment::of(seen));
    }

    /// The reports of a match replayed from its opening by `decisions`, each a choice's
    /// action without the revision.
    pub(crate) fn replayed(mut game: Game, decisions: &[String]) -> Self {
        let mut reports = Self::new(&game.observe());
        for decision in decisions {
            game.apply(decision)
                .expect("An admitted save's decisions replay");
            reports.record(&game.observe());
        }
        reports
    }

    /// A report of each action so far, whole, in order. The last may be in progress.
    pub fn all(&self) -> Vec<Report> {
        self.since(0)
    }

    /// The reports of what followed `revision`, one for each action, in order. The first
    /// may continue an action already reported up to `revision`, and the last may be in
    /// progress. An action in which nothing was done yet is not reported.
    pub fn since(&self, revision: u64) -> Vec<Report> {
        let mut at = self
            .moments
            .iter()
            .rposition(|moment| moment.revision <= revision)
            .unwrap_or_default();
        let mut reports = vec![];
        while at + 1 < self.moments.len() {
            let opened = &self.moments[at];
            let Some(faction) = opened.actor else {
                at += 1;
                continue;
            };
            let end = (at + 1..self.moments.len())
                .find(|&later| !self.moments[later].in_action_of(opened))
                .unwrap_or(self.moments.len() - 1);
            let continues = at
                .checked_sub(1)
                .is_some_and(|before| self.moments[before].in_action_of(opened));
            reports.extend(span(&self.moments[at..=end], faction, continues));
            at = end;
        }
        reports
    }
}

/// What a report reads of one committed position.
#[derive(Clone, Debug)]
struct Moment {
    revision: u64,
    card: u8,
    next_card: u8,
    /// The faction whose action is open, or none while only the card is to be turned.
    actor: Option<Faction>,
    executing: Option<Faction>,
    action: ActionChoice,
    event: Option<EventProgress>,
    eligibility: Eligibility,
    next: Option<Next>,
    passed: [bool; 4],
    resources: [u8; 4],
    spaces: Vec<Space>,
    available: Vec<Group>,
    markers_available: u8,
    standings: Vec<Standing>,
    transfer: Option<Transfer>,
    removal: Option<Removal>,
    /// Where Kidnap could be rolled.
    kidnap_space: Option<String>,
    kidnap: Option<Kidnap>,
    resolved: Vec<Resolved>,
    assault_paid: Vec<AssaultResolution>,
}

impl Moment {
    fn of(seen: &Observation) -> Self {
        let event_open = seen.event.as_ref().is_some_and(|event| event.pending());
        let actor = if event_open {
            seen.event_executor
        } else if seen.executing.is_some() {
            seen.executing
        } else if seen.card_18_complete() || seen.choices.iter().any(Choice::turns_the_card) {
            None
        } else {
            Some(seen.owner)
        };
        Self {
            revision: seen.revision,
            card: seen.current_card,
            next_card: seen.preview_card,
            actor,
            executing: seen.executing,
            action: seen.action,
            event: seen.event.clone(),
            eligibility: seen.eligibility.clone(),
            next: seen.next.clone(),
            passed: seen.passed,
            resources: seen.resources,
            spaces: seen.spaces.clone(),
            available: seen.available.clone(),
            markers_available: seen.marker_supply.available,
            standings: seen.standings.clone(),
            transfer: seen.transfer.clone(),
            removal: seen.removal.clone(),
            kidnap_space: seen.kidnap_stakes.as_ref().map(|s| s.space.clone()),
            kidnap: seen.kidnap.clone(),
            resolved: seen.resolved.clone(),
            assault_paid: seen.assault_paid.clone(),
        }
    }

    /// Whether the action open at `other` is open here too.
    fn in_action_of(&self, other: &Self) -> bool {
        (self.actor, self.card) == (other.actor, other.card)
    }

    /// Which action `faction` has chosen on this card.
    fn action_of(&self, faction: Faction) -> Action {
        if self.passed[faction.index()] {
            return Action::Pass;
        }
        match faction {
            Faction::M26 => match self.action {
                ActionChoice::Open => Action::Undecided,
                ActionChoice::Pass => Action::Pass,
                ActionChoice::Terror => Action::Terror,
            },
            Faction::Directorio => match &self.event {
                None => Action::Undecided,
                Some(
                    EventProgress::ExecuteShaded
                    | EventProgress::Complete(EventOutcome::Shaded { .. }),
                ) => Action::EventShaded,
                Some(_) => Action::EventUnshaded,
            },
            // Assault is the one Operation this build plays for Government.
            Faction::Government
                if self.executing == Some(faction) || self.eligibility.government_box.is_some() =>
            {
                Action::Assault
            }
            _ => Action::Undecided,
        }
    }

    fn rally(&self) -> Option<&Rally> {
        match &self.event {
            Some(
                EventProgress::Rally(rally)
                | EventProgress::Complete(EventOutcome::Unshaded(rally)),
            ) => Some(rally),
            _ => None,
        }
    }

    fn event_ended(&self) -> bool {
        self.event.as_ref().is_some_and(|event| !event.pending())
    }

    fn flips(&self) -> Option<&[Flip]> {
        match self.event.as_ref().and_then(EventProgress::outcome) {
            Some(EventOutcome::Shaded { flips }) => Some(flips),
            _ => None,
        }
    }

    /// The pending Transfer request: its giver, its recipient and the amount asked.
    fn request(&self) -> Option<(Faction, Faction, u8)> {
        match self.transfer {
            Some(Transfer::Authorizing {
                giver,
                recipient,
                amount,
                ..
            }) => Some((giver, recipient, amount)),
            _ => None,
        }
    }

    fn space(&self, id: &str) -> &Space {
        self.spaces
            .iter()
            .find(|space| space.id == id)
            .expect("A space of the map")
    }

    fn resources(&self, faction: Faction) -> u8 {
        self.resources[faction.index()]
    }
}

/// A faction's Resources before and after one step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Balance {
    from: u8,
    to: u8,
}

/// One thing an action did. A `space` is a space's name.
enum Item {
    WentBack {
        /// The giver of the request it withdrew.
        request: Option<Faction>,
    },
    Pass(Balance),
    Transfers(Transfers),
    Removed {
        faction: Faction,
        /// A space and a group that left it.
        groups: Vec<(String, Group)>,
    },
    Kidnap {
        space: String,
        kidnap: Kidnap,
    },
    Terror {
        space: String,
        paid: u8,
        balance: Balance,
        activated: u8,
        markers: u8,
        support: Option<(i8, i8)>,
    },
    Assault {
        space: String,
        paid: u8,
        balance: Balance,
        /// The pieces that left the space.
        removed: Vec<Group>,
        /// The Casinos that closed.
        closed: u8,
    },
    Shaded {
        /// A space and the Guerrillas Activated there.
        flips: Vec<(String, u8)>,
    },
    Granted(Faction),
    Rally {
        faction: Faction,
        space: String,
        action: RallyAction,
    },
    NoRally(Faction),
}

/// What was asked and given between one giver and one recipient.
struct Transfers {
    giver: Faction,
    recipient: Faction,
    /// The amount first asked, and how many times the recipient asked.
    asked: Option<u8>,
    asks: u16,
    moves: u16,
    given: u16,
    /// The giver's and the recipient's balances around the last one that moved.
    balances: Option<(Balance, Balance)>,
    /// Requests answered with nothing.
    unmet: u16,
    withdrawn: u16,
    /// The amount the recipient asks for and the giver has not decided.
    pending: Option<u8>,
}

/// The report of one action's `moments`, or none if there is nothing to tell.
fn span(moments: &[Moment], faction: Faction, continues: bool) -> Option<Report> {
    let (first, last) = (moments.first()?, moments.last()?);
    let complete = !last.in_action_of(first);
    let mut items = vec![];
    for step in moments.windows(2) {
        happened(&step[0], &step[1], continues, &mut items);
    }
    let action = last.action_of(faction);
    let told = !items.is_empty() || complete || (!continues && action != Action::Undecided);
    let again = continues && first.action_of(faction) == action;
    told.then(|| Report {
        faction,
        action,
        card: first.card,
        card_title: title(first.card),
        role: if first.eligibility.first == faction {
            "1st Eligible"
        } else {
            "2nd Eligible"
        },
        recorded: recorded(last, faction),
        from_revision: first.revision,
        to_revision: last.revision,
        continues,
        complete,
        spaces: spaces(&items),
        headline: words::headline(faction, action, again, complete, &items),
        summary: words::summary(faction, &items),
        articles: words::articles(faction, &items),
        standings: standings(first, last),
        notes: words::notes(first, last, faction),
    })
}

/// The record of `faction`'s action on the Sequence of Play.
fn recorded(at: &Moment, faction: Faction) -> Option<&'static str> {
    match faction {
        Faction::M26 => at.eligibility.m26_box,
        Faction::Government => at.eligibility.government_box,
        Faction::Syndicate if at.passed[Faction::Syndicate.index()] => Some(SequenceBox::Pass),
        _ => None,
    }
    .map(|recorded| recorded.name())
}

fn standings(first: &Moment, last: &Moment) -> Vec<Row> {
    first
        .standings
        .iter()
        .zip(&last.standings)
        .map(|(was, is)| Row {
            faction: is.faction,
            resources: FromTo {
                from: first.resources(is.faction).into(),
                to: last.resources(is.faction).into(),
            },
            score: FromTo {
                from: was.score,
                to: is.score,
            },
            margin: FromTo {
                from: was.margin,
                to: is.margin,
            },
        })
        .collect()
}

/// Adds what one committed decision did. A decision that only opens or closes a step does
/// nothing.
fn happened(before: &Moment, after: &Moment, continues: bool, items: &mut Vec<Item>) {
    let name = |id: &str| after.space(id).name.clone();
    if before.action == ActionChoice::Terror && after.action == ActionChoice::Open {
        // Nothing had an effect, so only what an earlier report told is undone.
        items.clear();
        if continues {
            items.push(Item::WentBack {
                request: before.request().map(|(giver, ..)| giver),
            });
        }
        return;
    }
    for faction in Faction::ALL {
        if after.passed[faction.index()] && !before.passed[faction.index()] {
            items.push(Item::Pass(balance(before, after, faction)));
        }
    }
    if let (None, Some(kidnap), Some(id)) = (&before.kidnap, &after.kidnap, &before.kidnap_space) {
        items.push(Item::Kidnap {
            space: name(id),
            kidnap: kidnap.clone(),
        });
    }
    for resolved in after.resolved.iter().skip(before.resolved.len()) {
        let (was, is) = (before.space(&resolved.space), after.space(&resolved.space));
        items.push(Item::Terror {
            space: is.name.clone(),
            paid: resolved.paid,
            balance: balance(before, after, Faction::M26),
            activated: lost(was, is)
                .iter()
                .filter(|group| group.state == State::Underground)
                .map(|group| group.count)
                .sum(),
            markers: is.terror - was.terror,
            support: (was.support != is.support).then_some((was.support, is.support)),
        });
    }
    for paid in after.assault_paid.iter().skip(before.assault_paid.len()) {
        let (was, is) = (before.space(&paid.space), after.space(&paid.space));
        // An Assault closes an open Casino and removes every other piece (R 1.4.5).
        let (closed, removed): (Vec<_>, Vec<_>) = lost(was, is)
            .into_iter()
            .partition(|group| group.state == State::Open);
        items.push(Item::Assault {
            space: is.name.clone(),
            paid: paid.paid,
            balance: balance(before, after, Faction::Government),
            removed,
            closed: closed.iter().map(|group| group.count).sum(),
        });
    }
    if let (None, Some(flips)) = (before.flips(), after.flips()) {
        items.push(Item::Shaded {
            flips: flips
                .iter()
                .map(|flip| (name(&flip.space), flip.count))
                .collect(),
        });
    }
    rallied(before, after, items);
    transferred(before, after, items);
    if let (Some(_), Some(_), Some(faction)) = (&before.removal, &after.removal, before.executing) {
        removed(before, after, faction, items);
    }
}

fn balance(before: &Moment, after: &Moment, faction: Faction) -> Balance {
    Balance {
        from: before.resources(faction),
        to: after.resources(faction),
    }
}

/// The groups `is` holds fewer of than `was`, by how many fewer.
fn lost(was: &Space, is: &Space) -> Vec<Group> {
    was.pieces
        .iter()
        .filter_map(|group| {
            let left = is
                .pieces
                .iter()
                .find(|other| {
                    (other.faction, other.kind, other.state)
                        == (group.faction, group.kind, group.state)
                })
                .map_or(0, |other| other.count);
            (left < group.count).then(|| Group {
                count: group.count - left,
                ..group.clone()
            })
        })
        .collect()
}

/// What the free Rally of the Event's unshaded text did.
fn rallied(before: &Moment, after: &Moment, items: &mut Vec<Item>) {
    let Some(rally) = after.rally() else {
        return;
    };
    let faction = rally.beneficiary;
    let done = before.rally().map_or_else(
        || {
            items.push(Item::Granted(faction));
            0
        },
        |rally| rally.resolutions.len(),
    );
    for step in rally.resolutions.iter().skip(done) {
        items.push(Item::Rally {
            faction,
            space: after.space(&step.space).name.clone(),
            action: step.action,
        });
    }
    if after.event_ended() && !before.event_ended() && rally.resolutions.is_empty() {
        items.push(Item::NoRally(faction));
    }
}

fn transferred(before: &Moment, after: &Moment, items: &mut Vec<Item>) {
    match (before.request(), after.request()) {
        (None, Some((giver, recipient, amount))) => {
            let told = between(items, giver, recipient);
            told.asked.get_or_insert(amount);
            told.asks += 1;
            told.pending = Some(amount);
        }
        (Some((giver, recipient, _)), None) => {
            let told = between(items, giver, recipient);
            told.pending = None;
            // 26July may end Terror while the giver still decides (local policy, #61).
            if after.executing.is_none() {
                told.withdrawn += 1;
            } else if before.resources == after.resources {
                told.unmet += 1;
            } else {
                moved(before, after, told);
            }
        }
        (None, None) if before.transfer.is_some() && before.resources != after.resources => {
            let holds = |order: std::cmp::Ordering| {
                Faction::ALL
                    .into_iter()
                    .find(|&f| after.resources(f).cmp(&before.resources(f)) == order)
            };
            if let (Some(giver), Some(recipient)) = (
                holds(std::cmp::Ordering::Less),
                holds(std::cmp::Ordering::Greater),
            ) {
                moved(before, after, between(items, giver, recipient));
            }
        }
        _ => {}
    }
}

fn moved(before: &Moment, after: &Moment, told: &mut Transfers) {
    let giver = balance(before, after, told.giver);
    told.moves += 1;
    told.given += u16::from(giver.from - giver.to);
    told.balances = Some((giver, balance(before, after, told.recipient)));
}

/// What is told of `giver` and `recipient`, which starts here if nothing was.
fn between(items: &mut Vec<Item>, giver: Faction, recipient: Faction) -> &mut Transfers {
    let known = items.iter().position(|item| {
        matches!(item, Item::Transfers(told) if (told.giver, told.recipient) == (giver, recipient))
    });
    let at = known.unwrap_or_else(|| {
        items.push(Item::Transfers(Transfers {
            giver,
            recipient,
            asked: None,
            asks: 0,
            moves: 0,
            given: 0,
            balances: None,
            unmet: 0,
            withdrawn: 0,
            pending: None,
        }));
        items.len() - 1
    });
    match &mut items[at] {
        Item::Transfers(told) => told,
        _ => unreachable!("Found or made as Transfers"),
    }
}

/// `faction`'s own pieces that went to its Available Forces (R 1.4.1), told together.
fn removed(before: &Moment, after: &Moment, faction: Faction, items: &mut Vec<Item>) {
    for (was, is) in before.spaces.iter().zip(&after.spaces) {
        for group in lost(was, is) {
            let known = items
                .iter()
                .position(|item| matches!(item, Item::Removed { faction: f, .. } if *f == faction));
            let at = known.unwrap_or_else(|| {
                items.push(Item::Removed {
                    faction,
                    groups: vec![],
                });
                items.len() - 1
            });
            let Item::Removed { groups, .. } = &mut items[at] else {
                unreachable!("Found or made as Removed");
            };
            match groups.iter_mut().find(|(space, known)| {
                *space == is.name && (known.kind, known.state) == (group.kind, group.state)
            }) {
                Some((_, known)) => known.count += group.count,
                None => groups.push((is.name.clone(), group)),
            }
        }
    }
}

/// The names of the spaces where `items` happened, in order, each once.
fn spaces(items: &[Item]) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    for item in items {
        let here: Vec<&String> = match item {
            Item::Kidnap { space, .. }
            | Item::Terror { space, .. }
            | Item::Assault { space, .. }
            | Item::Rally { space, .. } => vec![space],
            Item::Removed { groups, .. } => groups.iter().map(|(space, _)| space).collect(),
            Item::Shaded { flips } => flips.iter().map(|(space, _)| space).collect(),
            Item::WentBack { .. }
            | Item::Pass(_)
            | Item::Transfers(_)
            | Item::Granted(_)
            | Item::NoRally(_) => vec![],
        };
        for space in here {
            if !names.contains(space) {
                names.push(space.clone());
            }
        }
    }
    names
}

/// A piece's kind in words, for one piece or several.
fn kind_name(kind: Kind, count: u8) -> &'static str {
    match (kind, count) {
        (Kind::Troops, 1) => "Troop",
        (Kind::Troops, _) => "Troops",
        (Kind::Police, 1) => "Police cube",
        (Kind::Police, _) => "Police cubes",
        (Kind::Guerrilla, 1) => "Guerrilla",
        (Kind::Guerrilla, _) => "Guerrillas",
        (Kind::Base, 1) => "Base",
        (Kind::Base, _) => "Bases",
        (Kind::Casino, 1) => "Casino",
        (Kind::Casino, _) => "Casinos",
    }
}
