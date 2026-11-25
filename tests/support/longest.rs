//! The longest actions this build was found to have, as the decisions that play them, for
//! the report tests (`tests/integration/reports.rs`) and the newspaper's native test.
//! The includer brings `Faction`, `Game` and `SEED` into scope.
use super::{Faction, Game, SEED};

/// Card 17 to Directorio's Event, with 26July's Kidnap.
pub const HANDOFF: [&str; 5] = [
    "terror",
    "kidnap",
    "kidnap:roll",
    "terror:santiago",
    "end-terror",
];
/// After the handoff, to Government's decision on card 18, the shaded text executed.
pub const CARD_18: [&str; 3] = ["event:shaded", "resolve-shaded", "advance-card"];

struct Line {
    now: Game,
    decisions: Vec<String>,
}

impl Line {
    fn played(actions: &[&str]) -> Self {
        let mut line = Self {
            now: Game::opening_with_seed(SEED),
            decisions: vec![],
        };
        line.play(actions);
        line
    }
    fn play(&mut self, actions: &[&str]) {
        for action in actions {
            let seen = self.now.observe();
            let choice = seen.choices.iter().find(|c| c.action() == *action);
            let choice = choice.unwrap_or_else(|| panic!("{action} is offered"));
            self.now
                .submit(choice.owner, seen.revision, &choice.id)
                .unwrap();
            self.decisions.push((*action).to_owned());
        }
    }
    fn resources(&self, faction: Faction) -> u8 {
        self.now.observe().resources[faction.index()]
    }

    /// The executing faction gives each other one 10 and asks it for 1 twenty times, half
    /// of them declined, so every transfer article tells of requests, moves and refusals.
    fn circulate(&mut self, executor: Faction) {
        let mut others: Vec<Faction> = Faction::ALL
            .into_iter()
            .filter(|f| *f != executor)
            .collect();
        others.sort_by_key(|f| std::cmp::Reverse(self.resources(*f)));
        let mut given = vec![];
        for other in &others {
            let n = other.index();
            if self.resources(*other) < 10 {
                self.play(&["transfer", &format!("give:{n}:10")]);
                given.push(*other);
            }
            for answer in ["decline", "authorize:1"] {
                for _ in 0..10 {
                    self.play(&["transfer", &format!("ask:{n}:1"), answer]);
                }
            }
        }
        for other in others.iter().filter(|other| !given.contains(other)) {
            self.play(&["transfer", &format!("give:{}:10", other.index())]);
        }
    }

    /// The executing faction removes every piece of its own in `spaces`.
    fn remove_all(&mut self, spaces: &[&str]) {
        self.play(&["remove"]);
        for space in spaces {
            loop {
                let seen = self.now.observe();
                let all = seen.choices.iter().rev().find(|c| {
                    c.supported && c.removal().is_some_and(|(there, ..)| there == *space)
                });
                let Some(action) = all.map(|c| c.action().to_owned()) else {
                    break;
                };
                self.play(&[&action]);
            }
        }
        self.play(&["done-removal"]);
    }
}

/// The decisions of a match that ends with the longest report found of Terror, of the
/// shaded and of the unshaded Event, and of an Assault. Each has every article it can: a
/// transfer article for each giver and recipient of which one executes (6, or 10 when both
/// Directorio and 26July do), the removal of each executing faction, and the action's own.
/// An Assault pays for at most 12 spaces at 2 each (R 6.3.1): card 18 follows Kidnap, whose
/// Terror paid 1 of the 26 Resources, and nothing adds any. Only Havana and Santiago hold
/// cubes beside a piece an Assault can affect (R 3.2.4).
pub fn longest() -> [Vec<String>; 4] {
    let mut terror = Line::played(&[
        "terror",
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "terror:oriente",
        "transfer",
        "ask:3:15",
        "authorize:15",
        "transfer",
        "give:3:10",
    ]);
    terror.circulate(Faction::M26);
    terror.remove_all(&["santiago", "oriente", "sierra-maestra"]);
    terror.play(&["transfer", "ask:0:1", "end-terror"]);

    let mut shaded = Line::played(&HANDOFF);
    shaded.play(&["event:shaded"]);
    shaded.circulate(Faction::Directorio);
    shaded.remove_all(&["havana", "camaguey-province"]);
    shaded.play(&["resolve-shaded"]);

    let mut unshaded = Line::played(&HANDOFF);
    unshaded.play(&["event:unshaded"]);
    unshaded.circulate(Faction::Directorio);
    unshaded.remove_all(&["havana", "camaguey-province"]);
    unshaded.play(&["rally:beneficiary:26july"]);
    unshaded.circulate(Faction::M26);
    // Without its Base 26July Rallies nowhere, which ends the Event (R 3.3.1).
    unshaded.remove_all(&["santiago", "oriente", "sierra-maestra"]);

    let mut assault = Line::played(&HANDOFF);
    assault.play(&CARD_18);
    assault.play(&["government:assault"]);
    assault.circulate(Faction::Government);
    for other in [Faction::M26, Faction::Directorio, Faction::Syndicate] {
        let due = 24 - assault.resources(Faction::Government);
        let all = assault.resources(other).min(due);
        let n = other.index();
        assault.play(&[
            "transfer",
            &format!("ask:{n}:{all}"),
            &format!("authorize:{all}"),
        ]);
    }
    assault.play(&["assault:havana:syndicate", "assault:santiago:26july"]);
    let spaces = assault.now.observe().spaces;
    for space in spaces {
        if !["havana", "santiago", "oriente"].contains(&space.id.as_str()) {
            assault.play(&[&format!("assault:{}:none", space.id)]);
        }
    }
    assault.remove_all(&["havana", "camaguey-city", "santiago", "las-villas"]);
    assault.play(&["end-assault"]);
    [terror, shaded, unshaded, assault].map(|line| line.decisions)
}
