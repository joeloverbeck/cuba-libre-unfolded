//! Directorio's ordinary paid Rally after the Manifesto (R 3.1, 3.3.1).
use super::*;

pub(super) const START: &str = "directorio:rally";
const PAYMENT: &str = "paid-rally:";
const COST: u8 = 1;
const FACTION: Faction = Faction::Directorio;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaidRally {
    pub resolutions: Vec<RallyResolution>,
}

impl PaidRally {
    fn used(&self, space: &Space) -> bool {
        self.resolutions.iter().any(|step| step.space == space.id)
    }
}

pub(super) fn recruitment(action: &str) -> Option<(&str, u8)> {
    let (space, quantity) = action.strip_prefix(PAYMENT)?.split_once(":place:")?;
    Some((space, quantity.parse().ok()?))
}

fn exclusion(space: &Space) -> Option<&'static str> {
    if space.terrain == Terrain::EconomicCenter {
        return Some("No Rally in economic centers.");
    }
    match space.support {
        -2 => Some("Active Opposition blocks Rally."),
        2 => Some("Active Support blocks Rally."),
        _ => None,
    }
}

impl Game {
    pub(super) fn paid_rally_choices(&self, rally: &PaidRally) -> Vec<Choice> {
        let available = total(&self.available, FACTION, Kind::Guerrilla);
        let balance = self.resources[FACTION.index()];
        let can_pay = balance >= COST;
        let payment = if can_pay {
            "R 3.3.1. Pay 1 Resource and use this space."
        } else {
            "R 3.3.1. Needs 1 Directorio Resource."
        };
        let mut choices = Vec::new();
        for space in &self.spaces {
            if rally.used(space) || exclusion(space).is_some() {
                continue;
            }
            let bases = total(&space.pieces, FACTION, Kind::Base);
            let population = space.population;
            let (quantities, arithmetic) = if bases > 0 {
                let ceiling = bases + population;
                let limit = ceiling.min(available);
                let base_name = if bases == 1 { "Base" } else { "Bases" };
                let total = format!("{bases} {base_name} + {population} population");
                let arithmetic = if limit < ceiling {
                    format!("{total} = {ceiling}. Available: {limit}.")
                } else {
                    format!("{total} allows up to {limit}.")
                };
                (0..=limit, arithmetic)
            } else {
                (
                    1..=available.min(1),
                    "Place one Available Guerrilla Underground.".into(),
                )
            };
            for quantity in quantities {
                choices.push(
                    self.offer(
                        &format!("{PAYMENT}{}:place:{quantity}", space.id),
                        RallyAction::Place(quantity).description(),
                        can_pay,
                        format!("{arithmetic}\n{payment}"),
                    )
                    .at(&space.id, quantity > 0)
                    .stopped_by_cost(!can_pay),
                );
            }
        }
        choices.push(self.offer(
            "paid-rally:finish",
            "Finish Rally",
            false,
            "Finishing Rally and adjusting Eligibility are outside this continuation.",
        ));
        choices
    }

    pub(super) fn paid_rally_space_reason(
        &self,
        rally: &PaidRally,
        space: &Space,
        here: &[&Choice],
    ) -> String {
        if rally.used(space) {
            "Rally paid here (R 3.1).".into()
        } else if let Some(reason) = exclusion(space) {
            reason.into()
        } else if here.is_empty() {
            "No Guerrillas in Available Forces.".into()
        } else if self.resources[FACTION.index()] < COST {
            "Needs 1 Directorio Resource.".into()
        } else {
            "Rally costs 1 Resource (R 3.3.1).".into()
        }
    }

    pub(super) fn apply_paid_recruitment(&mut self, action: &str) -> Result<(), String> {
        let (space, quantity) = recruitment(action).ok_or("Unknown paid Rally recruitment.")?;
        let step = RallyResolution {
            space: space.into(),
            action: RallyAction::Place(quantity),
        };
        // The checked submission already verified cost, Available Forces,
        // qualification and that this space is unused. Charge after placing pieces.
        self.change_rally_pieces(FACTION, &step)?;
        self.resources[FACTION.index()] -= COST;
        self.paid_rally
            .as_mut()
            .expect("Paid Rally is open")
            .resolutions
            .push(step);
        Ok(())
    }

    pub(super) fn paid_recruitment_preview(
        &self,
        after: &Self,
        quantity: u8,
    ) -> Vec<AvailableForcesChange> {
        if quantity == 0 {
            return vec![];
        }
        vec![AvailableForcesChange {
            piece: Group {
                faction: FACTION,
                kind: Kind::Guerrilla,
                state: State::Underground,
                count: quantity,
            },
            from: total(&self.available, FACTION, Kind::Guerrilla),
            to: total(&after.available, FACTION, Kind::Guerrilla),
        }]
    }
}
