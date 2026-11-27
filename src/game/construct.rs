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
    pub(super) side_effect_committed: bool,
    authorizations: Vec<ConstructAuthorization>,
}

/// A giver's conditional reply, kept in decision order until it settles or is released.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructAuthorization {
    pub giver: Faction,
    pub asked: u8,
    pub amount: u8,
}

/// The funding still needed before conditional authorizations can move Resources.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConstructFunding {
    pub authorizations: Vec<ConstructAuthorization>,
    pub shortfall: u8,
}

/// The effect of a nonzero reply while Limited Construct needs conditional funding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructFundingReply {
    Held {
        authorization: ConstructAuthorization,
        shortfall: u8,
    },
    Settled(ConstructSettlement),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructSettlement {
    pub(crate) earlier: Vec<ConstructMovement>,
    pub(crate) final_reply: ConstructMovement,
}

impl ConstructSettlement {
    pub(crate) fn movements(&self) -> impl Iterator<Item = &ConstructMovement> {
        self.earlier
            .iter()
            .chain(std::iter::once(&self.final_reply))
    }

    pub(crate) fn total(&self) -> u8 {
        self.movements().map(|m| m.authorization.amount).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructMovement {
    pub(crate) authorization: ConstructAuthorization,
    pub(crate) giver_balance: [u8; 2],
    pub(crate) syndicate_balance: [u8; 2],
}

struct FundingProjection {
    balances: [i16; 4],
    reply: Option<ConstructFundingReply>,
}

impl ConstructFunding {
    pub(crate) const CONDITION: &str =
        "Resources move only when authorizations fund a legal payment.";

    pub(crate) fn authorized(&self) -> u8 {
        self.authorizations.iter().map(|a| a.amount).sum()
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "{} conditionally authorized · {} still needed",
            self.authorized(),
            self.shortfall
        )
    }

    #[cfg(feature = "tabletop")]
    pub(crate) fn projected_balances(
        &self,
        balances: [u8; 4],
        terms: &TransferTerms,
        asked: u8,
        amount: u8,
    ) -> (i16, i16) {
        let projection = self.project(balances, terms, asked, amount);
        (
            projection.balances[terms.giver.index()],
            projection.balances[SYNDICATE.index()],
        )
    }

    fn project(
        &self,
        balances: [u8; 4],
        terms: &TransferTerms,
        asked: u8,
        amount: u8,
    ) -> FundingProjection {
        debug_assert_eq!(terms.recipient, SYNDICATE);
        let mut balances = balances.map(i16::from);
        let authorization = ConstructAuthorization {
            giver: terms.giver,
            asked,
            amount,
        };
        let permitted = amount <= terms.limit;
        if amount < self.shortfall {
            return FundingProjection {
                balances,
                reply: (permitted && amount > 0).then_some(ConstructFundingReply::Held {
                    authorization,
                    shortfall: self.shortfall - amount,
                }),
            };
        }

        // Project even a refused draft in signed Resources. Only a permitted
        // amount can describe a movement, which always has legal balances.
        let mut move_one = |authorization: ConstructAuthorization| {
            let giver = authorization.giver.index();
            let recipient = SYNDICATE.index();
            let from = balances[giver];
            let to = balances[recipient];
            let amount = i16::from(authorization.amount);
            balances[giver] -= amount;
            balances[recipient] += amount;
            permitted.then(|| ConstructMovement {
                authorization,
                giver_balance: [
                    from.try_into().unwrap(),
                    balances[giver].try_into().unwrap(),
                ],
                syndicate_balance: [
                    to.try_into().unwrap(),
                    balances[recipient].try_into().unwrap(),
                ],
            })
        };
        let earlier = self
            .authorizations
            .iter()
            .copied()
            .filter_map(&mut move_one)
            .collect();
        let reply = move_one(authorization).map(|final_reply| {
            ConstructFundingReply::Settled(ConstructSettlement {
                earlier,
                final_reply,
            })
        });
        FundingProjection { balances, reply }
    }
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

struct ConstructSpaceRefusal {
    explanation: &'static str,
    tag: &'static str,
}

fn qualifying_space(space: &Space) -> Result<(), ConstructSpaceRefusal> {
    if space.terrain == Terrain::EconomicCenter {
        Err(ConstructSpaceRefusal {
            explanation: "Construct selects Cities or Provinces (R 3.3.5).",
            tag: "Choose a City or Province",
        })
    } else if !matches!(
        space.control(),
        Some(Faction::Government | Faction::Syndicate)
    ) {
        Err(ConstructSpaceRefusal {
            explanation: "Construct requires Government or Syndicate Control (R 3.3.5).",
            tag: "Needs Government or Syndicate Control (R 3.3.5).",
        })
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
        if let Some(construct) = &mut self.construct {
            construct.side_effect_committed = true;
        }
    }

    pub(super) fn construct_committed(&self) -> bool {
        self.construct
            .as_ref()
            .is_some_and(|op| !op.paid.is_empty() || op.side_effect_committed)
    }

    pub(super) fn construct_may_go_back(&self) -> bool {
        self.construct_is_open()
            && self.construct_box() == SequenceBox::LimitedOperation
            && !self.construct_committed()
    }

    pub(super) fn construct_payment_owed(&self) -> bool {
        self.construct_is_open()
            && self.construct_box() == SequenceBox::LimitedOperation
            && self.construct_paid().is_empty()
    }

    pub(super) fn can_afford_construct_payment(&self, balance: u8) -> bool {
        self.spaces.iter().any(|space| {
            ConstructPlan::ALL.into_iter().any(|plan| {
                self.construct_payment_requirement(space, plan, balance)
                    .is_ok()
            })
        })
    }

    pub(super) fn construct_authorizations(&self) -> &[ConstructAuthorization] {
        self.construct.as_ref().map_or(&[], |op| &op.authorizations)
    }

    pub(super) fn construct_reserved(&self, giver: Option<Faction>) -> u8 {
        self.construct_authorizations()
            .iter()
            .filter(|a| giver.is_none_or(|giver| giver == a.giver))
            .map(|a| a.amount)
            .sum()
    }

    fn construct_funding_shortfall(&self) -> Option<u8> {
        let held = self.construct_balance() + self.construct_reserved(None);
        (self.construct_may_go_back()
            && held < CONSTRUCT_COST
            && self.can_afford_construct_payment(CONSTRUCT_COST))
        .then(|| CONSTRUCT_COST - held)
    }

    pub(super) fn construct_funding(&self) -> Option<ConstructFunding> {
        Some(ConstructFunding {
            authorizations: self.construct_authorizations().to_vec(),
            shortfall: self.construct_funding_shortfall()?,
        })
    }

    pub(super) fn construct_authorization_refusal(&self, amount: u8) -> Option<String> {
        if !self.construct_payment_owed() || amount == 0 {
            return None;
        }
        if self.construct_funding_shortfall().is_some() {
            return None;
        }
        let needed = CONSTRUCT_COST.saturating_sub(self.construct_balance());
        if amount < needed {
            return Some(format!(
                "Syndicate needs {} more for its required Limited Construct payment. Authorize at least {needed} to fund it, or decline (local policy, #215).",
                resources(needed)
            ));
        }
        (!self.can_afford_construct_payment(self.construct_balance() + amount)).then(|| {
            "Syndicate needs a legal space for its required Limited Construct payment (local policy, #215).".into()
        })
    }

    pub(super) fn construct_authorization_condition(
        &self,
        funding: &ConstructFunding,
        terms: &TransferTerms,
        asked: u8,
        amount: u8,
    ) -> Option<String> {
        let reply = funding
            .project(self.resources, terms, asked, amount)
            .reply?;
        Some(match reply {
            ConstructFundingReply::Held { shortfall, .. } => format!(
                "Hold {} conditionally. {} still needed before any of these Resources move (local policy, #215).",
                resources(amount),
                resources(shortfall)
            ),
            ConstructFundingReply::Settled(settlement) if settlement.earlier.is_empty() => format!(
                "Move {} now. Syndicate chooses where to pay (R 1.5.1, 3.3.5).",
                resources(amount)
            ),
            ConstructFundingReply::Settled(settlement) => format!(
                "Move {} now, including all conditional authorizations. Syndicate chooses where to pay (local policy, #215).",
                resources(settlement.total())
            ),
        })
    }

    pub(super) fn authorize_construct_funding(
        &mut self,
        giver: Faction,
        recipient: Faction,
        asked: u8,
        amount: u8,
    ) -> Option<ConstructFundingReply> {
        if amount == 0 || recipient != SYNDICATE {
            return None;
        }
        let funding = self.construct_funding()?;
        let terms = self.terms(giver, recipient);
        let FundingProjection { balances, reply } =
            funding.project(self.resources, &terms, asked, amount);
        let reply = reply.expect("Offered nonzero conditional authorization");
        let construct = self.construct.as_mut().expect("Limited Construct funding");
        match &reply {
            ConstructFundingReply::Held { authorization, .. } => {
                construct.authorizations.push(*authorization);
            }
            ConstructFundingReply::Settled(_) => {
                construct.authorizations.clear();
                self.resources = balances.map(|balance| balance.try_into().unwrap());
                self.mark_construct_side_effect();
            }
        }
        Some(reply)
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

    fn construct_space_requirement(&self, space: &Space) -> Result<(), ConstructSpaceRefusal> {
        let paid = self.construct_paid();
        if !paid.is_empty() && self.construct_box() == SequenceBox::LimitedOperation {
            return Err(ConstructSpaceRefusal {
                explanation: "Limited Construct pays in just one space. Finish Construct to record the Operation (R 2.3.5).",
                tag: "One space paid · Finish Construct",
            });
        }
        if paid.iter().any(|result| result.space == space.id) {
            return Err(ConstructSpaceRefusal {
                explanation: "Construct already paid in this space (R 3.1).",
                tag: "Construct already paid here",
            });
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
        balance: u8,
    ) -> Result<(), &'static str> {
        self.construct_space_requirement(space)
            .map_err(|refusal| refusal.explanation)?;
        plan.requirement(space, self.casino_inventory())?;
        if balance < CONSTRUCT_COST {
            return Err("Syndicate needs 5 Resources to pay for Construct (R 3.3.5).");
        }
        Ok(())
    }

    pub(super) fn construct_refusal(&self, action: &str) -> Option<&'static str> {
        if !self.construct_is_open() {
            return None;
        }
        if action == BACK && self.construct_committed() {
            return Some(if self.construct_payment_owed() {
                "A Construct effect has committed. Finish Construct after paying in one space (local policy, #215)."
            } else {
                "A Construct effect has committed. Finish Construct to record the Operation."
            });
        }
        let (space, plan) = payment(action)?;
        if self.removal.is_some() {
            return Some("Choose Done to leave piece management before paying for Construct.");
        }
        let spot = self.spaces.iter().find(|s| s.id == space)?;
        self.construct_payment_requirement(spot, plan, self.construct_balance())
            .err()
    }

    pub(super) fn construct_reason(&self, space: &Space, here: &[&Choice]) -> String {
        if let Err(refusal) = self.construct_space_requirement(space) {
            return refusal.tag.into();
        }
        if let Some(choice) = here.first() {
            if choice.supported {
                return format!("Construct costs {}", resources(CONSTRUCT_COST));
            }
            if !choice.remedies.is_empty() {
                return remedy_line(&choice.remedies);
            }
            return format!("Needs {}", resources(CONSTRUCT_COST));
        }
        // With neither plan offered, there is no closed Casino to open. Placement
        // is blocked by either the supply or a stack made up of two open Casinos.
        if self.casino_inventory() == 0 {
            "No closed or available Casino".into()
        } else {
            "Two open Casinos fill this space".into()
        }
    }

    pub(super) fn construct_choices(&self) -> Vec<Choice> {
        if self.removal.is_some() {
            return self.construct_management_choices();
        }
        if let Some(choices) = self.transfer_choices() {
            return choices;
        }
        let mut choices = vec![];
        let available = self.casino_inventory();
        let balance = self.construct_balance();
        let transfer_helps = balance < CONSTRUCT_COST
            && self.balance_after_transfers(SYNDICATE) + self.construct_reserved(None)
                >= CONSTRUCT_COST;
        for space in self
            .spaces
            .iter()
            .filter(|space| self.construct_space_requirement(space).is_ok())
        {
            for plan in ConstructPlan::ALL
                .into_iter()
                .filter(|plan| plan.requirement(space, available).is_ok())
            {
                let refusal = self
                    .construct_payment_requirement(space, plan, balance)
                    .err();
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
                let mut choice = self
                    .offer(
                        &format!("construct:{}:{}", space.id, plan.id()),
                        plan.label(),
                        refusal.is_none(),
                        explanation,
                    )
                    .at(&space.id, true)
                    .stopped_by_cost(refusal.is_some() && !transfer_helps);
                if refusal.is_some() && transfer_helps {
                    choice.remedies.push(TRANSFER_REMEDY.to_owned());
                }
                choices.push(choice);
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
        choices.push(self.offer(
            "transfer",
            "Resource transfer",
            true,
            "Syndicate may give Resources or ask another faction for Resources. The giver decides how much to transfer (R 1.5.1, 1.8).",
        ));
        choices.push(self.construct_manage_choice());
        if self.construct_may_go_back() {
            choices.push(self.construct_back_choice());
        }
        choices
    }

    pub(super) fn construct_back_choice(&self) -> Choice {
        let funding = if self.construct_authorizations().is_empty() {
            ""
        } else {
            " Release the conditional authorizations."
        };
        let withdrawal = if self.transfer.is_some() {
            " Withdraw the pending request."
        } else {
            ""
        };
        let mut choice = self.offer(
            BACK,
            "Back to the action choice",
            true,
            format!("Leave before committing any effect.{withdrawal}{funding} Syndicate may choose Pass or Limited Construct (local policy, #215)."),
        );
        choice.owner = SYNDICATE;
        choice
    }

    pub(super) fn is_construct_action(action: &str) -> bool {
        action == FINISH || payment(action).is_some()
    }

    pub(super) fn construct_payment_notes(
        &self,
        after: &Self,
        action: &str,
    ) -> Option<Vec<String>> {
        let (id, plan) = payment(action)?;
        let space = self.spaces.iter().find(|s| s.id == id)?;
        let result = after.spaces.iter().find(|s| s.id == id)?;
        let before_available = self.casino_inventory();
        let after_available = after.casino_inventory();
        let mut notes = vec![
            format!(
                "Casinos here · {} → {} open · {} → {} closed.",
                casinos(space, State::Open),
                casinos(result, State::Open),
                casinos(space, State::Closed),
                casinos(result, State::Closed)
            ),
            format!("Casinos in Available Forces: {before_available} → {after_available}."),
        ];
        notes.extend(
            ConstructPlan::ALL
                .into_iter()
                .filter(|other| *other != plan)
                .filter_map(|other| other.requirement(space, before_available).err())
                .map(str::to_owned),
        );
        Some(notes)
    }

    pub(super) fn apply_construct(
        &mut self,
        action: &str,
        next_revision: u64,
    ) -> Result<Option<ConstructFundingReply>, String> {
        match action {
            _ if self.removal.is_some() => self.apply_construct_management(action)?,
            OPERATION_ONLY | LIMITED => {
                self.construct = Some(Construct {
                    paid: vec![],
                    side_effect_committed: false,
                    authorizations: vec![],
                });
            }
            "remove" => self.removal = Some(Removal::Selecting),
            BACK => {
                self.construct = None;
                self.transfer = None;
            }
            FINISH => {
                self.card_18_completion = Some(Card18Completion::Construct);
                self.transfer = None;
            }
            _ if self.transfer.is_some() || action == "transfer" => {
                return self.execute(action, next_revision);
            }
            _ => {
                let (id, plan) = payment(action).ok_or("Unsupported Construct decision.")?;
                let index = self
                    .spaces
                    .iter()
                    .position(|s| s.id == id)
                    .ok_or("Unknown Construct space.")?;
                self.construct_payment_requirement(
                    &self.spaces[index],
                    plan,
                    self.construct_balance(),
                )?;
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
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_transfer_enforces_recipient_cap_in_both_directions() {
        let mut game = Game::opening_with_seed(2);
        for action in [
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
            "syndicate:limited-operation",
        ] {
            game.apply(action).unwrap();
        }
        let mut conditional = game.clone();
        // Reserve incoming room even when the second giver could send 49 on its own.
        conditional.resources = [49, 4, 5, 3];
        for action in ["transfer", "ask:1:2", "authorize:1", "transfer", "ask:0:45"] {
            conditional.apply(action).unwrap();
        }
        let pending = conditional.observe();
        assert_eq!(pending.transfer_terms[0].limit, 45);
        let refused = conditional
            .submit(
                pending.owner,
                pending.revision,
                &format!("{}:authorize:46", pending.revision),
            )
            .unwrap_err();
        assert!(refused.contains("1 Resource conditionally authorized toward that cap"));
        assert_eq!(conditional.observe(), pending);
        conditional.apply("authorize:45").unwrap();
        assert_eq!(conditional.observe().resources, [4, 3, 5, 49]);

        // R 1.8 permits these balances, beyond the Resources available in this
        // two-card slice. This rules-only position tests both directions at 48/49.
        game.resources = [2, 48, 5, 48];
        game.apply("transfer").unwrap();
        let selecting = game.observe();
        assert_eq!(selecting.transfer_terms.len(), 6);
        let from_government = &selecting.transfer_terms[0];
        assert_eq!((from_government.held, from_government.limit), (48, 1));
        game.apply("ask:0:1").unwrap();
        let pending = game.observe();
        assert_eq!(pending.owner, Faction::Government);
        let over_cap = game
            .submit(
                pending.owner,
                pending.revision,
                &format!("{}:authorize:2", pending.revision),
            )
            .unwrap_err();
        assert!(over_cap.contains("Syndicate holds 48 of at most 49 Resources"));
        assert_eq!(game.observe(), pending);
        game.apply("authorize:1").unwrap();
        assert_eq!(game.observe().resources, [1, 48, 5, 49]);

        game.apply("transfer").unwrap();
        let outbound = game.observe();
        let over_cap = game
            .submit(
                outbound.owner,
                outbound.revision,
                &format!("{}:give:1:2", outbound.revision),
            )
            .unwrap_err();
        assert!(over_cap.contains("26July holds 48 of at most 49 Resources"));
        assert_eq!(game.observe(), outbound);
        game.apply("give:1:1").unwrap();
        assert_eq!(game.observe().resources, [1, 49, 5, 48]);
    }

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
