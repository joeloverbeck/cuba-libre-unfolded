//! Syndicate Construct (R 2.3.5, 3.3.5).
use super::*;

mod management;

pub const CONSTRUCT_COST: u8 = 5;
pub(super) const OPERATION_ONLY: &str = "syndicate:construct";
pub(super) const LIMITED: &str = "syndicate:limited-operation";
pub(super) const FINISH: &str = "end-construct";
pub(super) const EMPTY_FINISH_POLICY: &str =
    "Finishing with zero paid spaces earns no Pass income (local policy, #215).";
const BACK: &str = "back-to-choice";
const SYNDICATE: Faction = Faction::Syndicate;
const CASINO_LIMIT: u8 = 2;

pub(crate) fn construct_name(first_eligible: bool) -> &'static str {
    if first_eligible {
        "Construct"
    } else {
        "Limited Construct"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Construct {
    paid: Vec<ConstructResolution>,
    side_effect_committed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructPlan {
    Open,
    Place,
}

impl ConstructPlan {
    const ALL: [Self; 2] = [Self::Open, Self::Place];

    fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Place => "place",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Open => "Open 1 closed Casino",
            Self::Place => "Place 1 closed Casino",
        }
    }

    pub(crate) fn result(self) -> &'static str {
        match self {
            Self::Open => "opened 1 closed Casino",
            Self::Place => "placed 1 closed Casino",
        }
    }

    fn requirement(self, space: &Space, available: u8) -> Result<(), &'static str> {
        match self {
            Self::Open if casinos(space, State::Closed) == 0 => {
                Err("No closed Casino to open here (R 3.3.5).")
            }
            Self::Place if total(&space.pieces, SYNDICATE, Kind::Casino) >= CASINO_LIMIT => {
                Err("This space already has two Casinos (R 1.4.2).")
            }
            Self::Place if available == 0 => Err("No Casino in Available Forces (R 1.4.1)."),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructResolution {
    pub space: String,
    pub paid: u8,
    pub plan: ConstructPlan,
}

impl ConstructResolution {
    pub(crate) fn summary(&self, space: &str) -> String {
        format!(
            "{space} · paid {} · {}",
            resources(self.paid),
            self.plan.result()
        )
    }
}

fn casinos(space: &Space, state: State) -> u8 {
    space
        .pieces
        .iter()
        .filter(|p| p.kind == Kind::Casino && p.state == state)
        .map(|p| p.count)
        .sum()
}

fn qualifying_space(space: &Space) -> Result<(), &'static str> {
    if space.terrain == Terrain::EconomicCenter {
        Err("Construct selects Cities or Provinces (R 3.3.5).")
    } else if !matches!(
        space.control(),
        Some(Faction::Government | Faction::Syndicate)
    ) {
        Err("Construct requires Government or Syndicate Control (R 3.3.5).")
    } else {
        Ok(())
    }
}

fn payment(action: &str) -> Option<(&str, ConstructPlan)> {
    let (space, plan) = action.strip_prefix("construct:")?.split_once(':')?;
    Some((
        space,
        ConstructPlan::ALL.into_iter().find(|p| p.id() == plan)?,
    ))
}

impl Game {
    pub(super) fn construct_is_open(&self) -> bool {
        self.construct.is_some() && self.card_18_completion.is_none()
    }

    pub(super) fn construct_paid(&self) -> &[ConstructResolution] {
        self.construct.as_ref().map_or(&[], |op| op.paid.as_slice())
    }

    pub(super) fn mark_construct_side_effect(&mut self) {
        self.construct.as_mut().unwrap().side_effect_committed = true;
    }

    pub(super) fn construct_payment_owed(&self) -> bool {
        self.construct_is_open()
            && self.construct_box() == SequenceBox::LimitedOperation
            && self.construct_paid().is_empty()
    }

    pub(super) fn can_afford_construct_payment(&self, balance: u8) -> bool {
        balance >= CONSTRUCT_COST
            && self.spaces.iter().any(|space| {
                qualifying_space(space).is_ok()
                    && ConstructPlan::ALL
                        .into_iter()
                        .any(|plan| plan.requirement(space, self.casino_inventory()).is_ok())
            })
    }

    pub(super) fn construct_may_go_back(&self) -> bool {
        self.construct_box() == SequenceBox::LimitedOperation
            && self
                .construct
                .as_ref()
                .is_some_and(|op| op.paid.is_empty() && !op.side_effect_committed)
    }

    pub(super) fn limited_construct_choice(&self) -> Choice {
        self.offer(
            LIMITED, "Limited Operation · Construct", true,
            "Choose one Government- or Syndicate-controlled City or Province and pay 5 Resources to open or place a Casino. No Special Activity (R 2.3.4–5, 3.3.5).",
        )
    }

    pub(super) fn operation_only_construct_choice(&self) -> Choice {
        self.offer(
            OPERATION_ONLY, "Construct · Operation Only", true,
            "Pay 5 Resources per distinct Government- or Syndicate-controlled City or Province to open or place a Casino. No Special Activity (R 2.3.4, 3.3.5).",
        )
    }

    pub(super) fn construct_box(&self) -> SequenceBox {
        if self.government_pass_from.is_some() {
            SequenceBox::OperationOnly
        } else {
            SequenceBox::LimitedOperation
        }
    }

    fn construct_space_requirement(&self, space: &Space) -> Result<(), &'static str> {
        let paid = self.construct_paid();
        if !paid.is_empty() && self.construct_box() == SequenceBox::LimitedOperation {
            return Err(
                "Limited Construct pays in just one space. Finish Construct to record the Operation (R 2.3.5).",
            );
        }
        if paid.iter().any(|result| result.space == space.id) {
            return Err("Construct already paid in this space (R 3.1).");
        }
        qualifying_space(space)
    }

    fn casino_inventory(&self) -> u8 {
        total(&self.available, SYNDICATE, Kind::Casino)
    }

    fn construct_balance(&self) -> u8 {
        self.resources[SYNDICATE.index()]
    }

    fn construct_payment_requirement(
        &self,
        space: &Space,
        plan: ConstructPlan,
    ) -> Result<(), &'static str> {
        self.construct_space_requirement(space)?;
        plan.requirement(space, self.casino_inventory())?;
        if self.construct_balance() < CONSTRUCT_COST {
            return Err("Syndicate needs 5 Resources to pay for Construct (R 3.3.5).");
        }
        Ok(())
    }

    pub(super) fn construct_refusal(&self, action: &str) -> Option<&'static str> {
        if !self.construct_is_open() {
            return None;
        }
        if action == BACK
            && self.construct_box() == SequenceBox::LimitedOperation
            && !self.construct_may_go_back()
        {
            return Some("Construct is committed. Finish Construct to record the Operation.");
        }
        let (space, plan) = payment(action)?;
        if self.removal.is_some() {
            return Some("Choose Done to leave piece management before paying for Construct.");
        }
        let spot = self.spaces.iter().find(|s| s.id == space)?;
        self.construct_payment_requirement(spot, plan).err()
    }

    pub(super) fn construct_reason(&self, space: &Space, here: &[&Choice]) -> String {
        if let Err(reason) = self.construct_space_requirement(space) {
            return reason.into();
        }
        if let Some(choice) = here.first() {
            if choice.supported {
                return format!("Construct costs {}", resources(CONSTRUCT_COST));
            }
            return choice.explanation.clone();
        }
        ConstructPlan::ALL
            .into_iter()
            .filter_map(|plan| plan.requirement(space, self.casino_inventory()).err())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(super) fn construct_choices(&self) -> Vec<Choice> {
        if self.removal.is_some() {
            return self.construct_management_choices();
        }
        let mut choices = vec![];
        let available = self.casino_inventory();
        let balance = self.construct_balance();
        for space in self
            .spaces
            .iter()
            .filter(|space| self.construct_space_requirement(space).is_ok())
        {
            for plan in ConstructPlan::ALL
                .into_iter()
                .filter(|plan| plan.requirement(space, available).is_ok())
            {
                let refusal = self.construct_payment_requirement(space, plan).err();
                let explanation = refusal.map_or_else(
                    || {
                        format!(
                            "Pay {} ({balance} → {}). {} in {} (R 3.3.5).",
                            resources(CONSTRUCT_COST),
                            balance - CONSTRUCT_COST,
                            plan.label(),
                            space.name
                        )
                    },
                    str::to_owned,
                );
                choices.push(
                    self.offer(
                        &format!("construct:{}:{}", space.id, plan.id()),
                        plan.label(),
                        refusal.is_none(),
                        explanation,
                    )
                    .at(&space.id, true)
                    .stopped_by_cost(refusal.is_some()),
                );
            }
        }
        let paid = self.construct_paid();
        let limited = self.construct_box() == SequenceBox::LimitedOperation;
        let explanation = match (limited, paid.last()) {
            (false, None) => format!(
                "Record Operation Only. {EMPTY_FINISH_POLICY} Syndicate becomes Ineligible for card 19 (R 2.3.7)."
            ),
            (false, Some(_)) => {
                let count = paid.len();
                let unit = if count == 1 { "space" } else { "spaces" };
                format!(
                    "Record Operation Only after paying in {count} {unit}. Syndicate becomes Ineligible for card 19 (R 2.3.4, 2.3.7)."
                )
            }
            (true, Some(result)) => {
                let space = self.spaces.iter().find(|s| s.id == result.space).unwrap();
                format!(
                    "{}. Syndicate becomes Ineligible for card 19 (R 2.3.7).",
                    result.summary(&space.name)
                )
            }
            (true, None) => {
                "Pay for one space on the map before finishing Limited Construct (R 2.3.5).".into()
            }
        };
        choices.push(self.offer(
            FINISH,
            "Finish Construct",
            !limited || !paid.is_empty(),
            explanation,
        ));
        choices.push(self.construct_manage_choice());
        if self.construct_may_go_back() {
            choices.push(self.offer(
                BACK, "Back to the action choice", true,
                "Leave before committing a payment or changing a piece. Syndicate may choose Pass or Limited Construct.",
            ));
        }
        choices
    }

    pub(super) fn is_construct_action(action: &str) -> bool {
        action == FINISH || payment(action).is_some()
    }

    pub(super) fn construct_payment_notes(
        &self,
        after: &Self,
        action: &str,
    ) -> Option<Vec<String>> {
        let (id, _) = payment(action)?;
        let space = self.spaces.iter().find(|s| s.id == id)?;
        let result = after.spaces.iter().find(|s| s.id == id)?;
        let before_available = self.casino_inventory();
        let after_available = after.casino_inventory();
        Some(vec![
            format!(
                "Casinos here · {} → {} open · {} → {} closed.",
                casinos(space, State::Open),
                casinos(result, State::Open),
                casinos(space, State::Closed),
                casinos(result, State::Closed)
            ),
            format!("Casinos in Available Forces: {before_available} → {after_available}."),
        ])
    }

    pub(super) fn apply_construct(&mut self, action: &str) -> Result<(), String> {
        match action {
            _ if self.removal.is_some() => return self.apply_construct_management(action),
            OPERATION_ONLY | LIMITED => {
                self.construct = Some(Construct {
                    paid: vec![],
                    side_effect_committed: false,
                })
            }
            "remove" => self.removal = Some(Removal::Selecting),
            BACK => self.construct = None,
            FINISH => self.card_18_completion = Some(Card18Completion::Construct),
            _ => {
                let (id, plan) = payment(action).ok_or("Unsupported Construct decision.")?;
                let index = self
                    .spaces
                    .iter()
                    .position(|s| s.id == id)
                    .ok_or("Unknown Construct space.")?;
                self.construct_payment_requirement(&self.spaces[index], plan)?;
                let pieces = &mut self.spaces[index].pieces;
                let state = match plan {
                    ConstructPlan::Open => {
                        pieces
                            .iter_mut()
                            .find(|p| p.kind == Kind::Casino && p.state == State::Closed)
                            .unwrap()
                            .count -= 1;
                        pieces.retain(|p| p.count > 0);
                        State::Open
                    }
                    ConstructPlan::Place => {
                        self.available
                            .iter_mut()
                            .find(|p| p.kind == Kind::Casino)
                            .unwrap()
                            .count -= 1;
                        State::Closed
                    }
                };
                if let Some(group) = pieces
                    .iter_mut()
                    .find(|p| p.kind == Kind::Casino && p.state == state)
                {
                    group.count += 1;
                } else {
                    pieces.push(Group {
                        faction: SYNDICATE,
                        kind: Kind::Casino,
                        state,
                        count: 1,
                    });
                }
                self.resources[SYNDICATE.index()] -= CONSTRUCT_COST;
                self.construct
                    .as_mut()
                    .unwrap()
                    .paid
                    .push(ConstructResolution {
                        space: id.into(),
                        paid: CONSTRUCT_COST,
                        plan,
                    });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_stacks_casinos_separately_and_opens_even_when_inventory_is_empty() {
        let mut game = Game::opening_with_seed(2);
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "transfer",
            "give:0:4",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:assault",
            "assault:havana:syndicate",
            "end-assault",
            "syndicate:limited-operation",
        ] {
            game.apply(action).unwrap();
        }
        let havana = game
            .spaces
            .iter()
            .position(|space| space.id == "havana")
            .unwrap();
        let bases = game
            .available
            .iter_mut()
            .find(|piece| piece.faction == Faction::Government && piece.kind == Kind::Base)
            .unwrap();
        bases.count -= 2;
        game.spaces[havana].pieces.push(Group {
            count: 2,
            ..bases.clone()
        });
        let placements = |game: &Game| {
            let view = game.observe();
            view.choices
                .iter()
                .filter(|choice| choice.supported && choice.space() == Some("havana"))
                .map(|choice| choice.action().to_owned())
                .collect::<Vec<_>>()
        };
        // R 1.4.2 permits two non-Casino Bases plus two Casinos.
        assert_eq!(
            placements(&game),
            ["construct:havana:open", "construct:havana:place"]
        );
        let mut filled = game.clone();
        filled.apply("construct:havana:place").unwrap();
        assert_eq!(
            total(&filled.spaces[havana].pieces, SYNDICATE, Kind::Casino),
            2
        );
        assert_eq!(
            total(
                &filled.spaces[havana].pieces,
                Faction::Government,
                Kind::Base
            ),
            2
        );

        // Distribute the seven spare Casinos without exceeding any space's stacking limit.
        let supply = game
            .available
            .iter_mut()
            .find(|piece| piece.kind == Kind::Casino)
            .unwrap();
        assert_eq!(supply.count, 7);
        supply.count = 0;
        for space in game
            .spaces
            .iter_mut()
            .filter(|space| space.terrain != Terrain::EconomicCenter)
            .take(7)
        {
            assert!(total(&space.pieces, SYNDICATE, Kind::Casino) < 2);
            if let Some(group) = space
                .pieces
                .iter_mut()
                .find(|p| p.kind == Kind::Casino && p.state == State::Closed)
            {
                group.count += 1;
            } else {
                space.pieces.push(Group {
                    faction: SYNDICATE,
                    kind: Kind::Casino,
                    state: State::Closed,
                    count: 1,
                });
            }
        }
        assert_eq!(casinos(&game.spaces[havana], State::Closed), 2);
        assert_eq!(placements(&game), ["construct:havana:open"]);
        let view = game.observe();
        let refused = game
            .submit(
                SYNDICATE,
                view.revision,
                &format!("{}:construct:havana:place", view.revision),
            )
            .unwrap_err();
        assert!(refused.contains("already has two Casinos"), "{refused}");
        let refused = game
            .submit(
                SYNDICATE,
                view.revision,
                &format!("{}:construct:camaguey-city:place", view.revision),
            )
            .unwrap_err();
        assert!(
            refused.contains("No Casino in Available Forces"),
            "{refused}"
        );
        assert_eq!(game.observe(), view);
        game.apply("construct:havana:open").unwrap();
        assert_eq!(casinos(&game.spaces[havana], State::Open), 1);
        assert_eq!(casinos(&game.spaces[havana], State::Closed), 1);
        assert_eq!(game.casino_inventory(), 0);
    }
}
