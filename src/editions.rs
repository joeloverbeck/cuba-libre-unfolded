//! Editions (#208): which action reports the human is given to read, and when. An edition
//! prints one report (`reports`) and adds nothing to it. This module knows no page.
//!
//! One is due when an action of a faction the human does not play is complete, and one
//! partway through it when the human must decide: a giver its Resource transfer, or 26July
//! the free Rally that Directorio's Event grants it. There is none for an action of a
//! faction the human plays, whoever decides inside it. An edition starts where the
//! previous one ended, so nothing is told twice. A faction's action is the human's from
//! their first decision for it: what an agent did with one they took back is told once they
//! hand it over again undecided, or once agents end the action themselves.
//!
//! While an edition is due or open no decision is taken from the human or an agent, so the
//! next one follows from what the human has read. The current card's editions stay to be
//! read again. They are in no save: they start at the position a match opens at.
use crate::{Choice, Faction, Observation, reports::Report};

/// One report, with its number among its card's editions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edition {
    pub number: usize,
    pub report: Report,
}

/// The edition the human has open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Open {
    /// Its place among the card's editions.
    at: usize,
    /// Whether the human took it from the pile, having read it.
    pub reopened: bool,
    /// The page it is open at, from 0.
    pub page: usize,
}

/// The current card's editions.
#[derive(Debug, Default)]
pub struct Editions {
    card: u8,
    /// The revision the last edition, or the human's own action so far, ended at.
    told: u64,
    /// The factions whose untold doings are the human's own: those they have decided for
    /// since they last took them back. All, as a match opens.
    theirs: [bool; 4],
    /// The decision that is the human's now, inside another faction's action, has had its
    /// edition. A request the human set aside and takes up again is no new decision.
    announced: bool,
    editions: Vec<Edition>,
    /// How many of them were opened. The others are due, in order.
    read: usize,
    open: Option<Open>,
}

impl Editions {
    /// The editions of a match that opens at `seen`: none, and what led there is not told.
    pub fn from(seen: &Observation) -> Self {
        Self {
            card: seen.current_card,
            told: seen.revision,
            theirs: [true; 4],
            ..Self::default()
        }
    }

    /// `decider`'s decision was committed and made `seen`. `since` gives the reports of what
    /// followed a revision, and `plays` the factions the human plays, indexed like
    /// `Faction::ALL`.
    pub fn decided(
        &mut self,
        seen: &Observation,
        decider: Faction,
        since: impl FnOnce(u64) -> Vec<Report>,
        plays: [bool; 4],
    ) {
        if self.card != seen.current_card {
            *self = Self {
                told: self.told,
                theirs: self.theirs,
                ..Self::from(seen)
            };
        }
        let human = plays[decider.index()];
        self.theirs[decider.index()] |= human;
        self.follow(seen, since(self.told), plays, human);
    }

    /// The human handed a faction over or took one back.
    pub fn seated(
        &mut self,
        seen: &Observation,
        since: impl FnOnce(u64) -> Vec<Report>,
        plays: [bool; 4],
    ) {
        self.follow(seen, since(self.told), plays, false);
    }

    /// `human`: the human's decision led here.
    fn follow(&mut self, seen: &Observation, reports: Vec<Report>, plays: [bool; 4], human: bool) {
        for (theirs, plays) in self.theirs.iter_mut().zip(plays) {
            *theirs &= plays;
        }
        let asked = plays[seen.owner.index()]
            && seen
                .choices
                .iter()
                .any(|c| c.supported && c.owner == seen.owner && !Choice::turns_the_card(c));
        if !asked {
            self.announced = false;
        }
        for report in reports {
            let plays = plays[report.faction.index()];
            // The human's own, whichever faction decided inside it, or ended by them.
            if plays && (self.theirs[report.faction.index()] || (report.complete && human)) {
                self.told = report.to_revision;
                continue;
            }
            // Of a faction just taken back, what its agent did waits to be told.
            if !report.complete && (plays || !asked || self.announced) {
                continue;
            }
            self.announced = !report.complete;
            self.told = report.to_revision;
            self.editions.push(Edition {
                number: self.editions.len() + 1,
                report,
            });
        }
    }

    /// Whether an edition is due or open, so nothing may be decided.
    pub fn holds(&self) -> bool {
        self.due() || self.open.is_some()
    }

    /// Whether an edition was not opened yet.
    pub fn due(&self) -> bool {
        self.read < self.editions.len()
    }

    /// Opens the first one due, once nothing is open.
    pub fn open_next(&mut self) {
        if self.open.is_none() && self.due() {
            self.open = Some(Open {
                at: self.read,
                reopened: false,
                page: 0,
            });
            self.read += 1;
        }
    }

    /// Opens the latest edition of the pile.
    pub fn reopen(&mut self) {
        if self.open.is_none() && self.pile() > 0 {
            self.open = Some(Open {
                at: self.read - 1,
                reopened: true,
                page: 0,
            });
        }
    }

    pub fn put_away(&mut self) {
        self.open = None;
    }

    /// Turns a reopened edition to the card's edition `number`.
    pub fn show(&mut self, number: usize) {
        if let Some(open) = &mut self.open
            && open.reopened
            && (1..=self.read).contains(&number)
        {
            *open = Open {
                at: number - 1,
                page: 0,
                ..*open
            };
        }
    }

    /// Turns a reopened edition to the next older one, or to the next newer one, and round
    /// again. The tabs are in that order.
    pub fn step(&mut self, back: bool) {
        if let Some(open) = self.open
            && open.reopened
        {
            let at = match (back, open.at) {
                (false, 0) => self.read - 1,
                (false, at) => at - 1,
                (true, at) => (at + 1) % self.read,
            };
            self.show(at + 1);
        }
    }

    /// Turns the open edition, of so many `pages`, to the next page or back.
    pub fn turn(&mut self, back: bool, pages: usize) {
        if let Some(open) = &mut self.open {
            open.page = if back {
                open.page.saturating_sub(1)
            } else {
                (open.page + 1).min(pages.saturating_sub(1))
            };
        }
    }

    /// The open edition, with how it is open.
    pub fn open(&self) -> Option<(Open, &Edition)> {
        self.open.map(|open| (open, &self.editions[open.at]))
    }

    /// How many of the card's editions lie on the pile: those that were put away.
    pub fn pile(&self) -> usize {
        self.read - usize::from(self.open.is_some_and(|open| !open.reopened))
    }

    /// The card's editions that were opened, the latest first.
    pub fn latest_first(&self) -> impl Iterator<Item = &Edition> {
        self.editions[..self.read].iter().rev()
    }
}
