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

pub(super) fn payment(action: &str) -> Option<(&str, RallyAction)> {
    let (space, method) = action.strip_prefix(PAYMENT)?.split_once(':')?;
    let (procedure, amount) = method.split_once(':')?;
    let amount: u8 = amount.parse().ok()?;
    let action = match procedure {
        "place" => RallyAction::Place(amount),
        "base" if amount <= 2 => RallyAction::ReplaceWithBase {
            underground: amount,
            active: 2 - amount,
        },
        "flip" => RallyAction::FlipUnderground(amount),
        _ => return None,
    };
    Some((space, action))
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
        let available_bases = total(&self.available, FACTION, Kind::Base);
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
            for exchange in rally::base_exchanges(space, FACTION, available_bases) {
                let RallyAction::ReplaceWithBase { underground, .. } = exchange else {
                    unreachable!("A Base exchange replaces Guerrillas")
                };
                choices.push(
                    self.offer(
                        &format!("{PAYMENT}{}:base:{underground}", space.id),
                        exchange.description(),
                        can_pay,
                        format!("Return the selected Guerrillas to Available and place 1 Base.\n{payment}"),
                    )
                    .at(&space.id, true)
                    .stopped_by_cost(!can_pay),
                );
            }
            if bases > 0 {
                let active = space
                    .pieces
                    .iter()
                    .find(|p| {
                        p.faction == FACTION
                            && p.kind == Kind::Guerrilla
                            && p.state == State::Active
                    })
                    .map_or(0, |p| p.count);
                let effect = match active {
                    0 => "No Guerrillas change face.",
                    1 => "Turn 1 Active Guerrilla Underground.",
                    _ => "Turn every Active Guerrilla Underground.",
                };
                let flip = RallyAction::FlipUnderground(active);
                let label = if active == 0 {
                    "Turn all Guerrillas Underground".into()
                } else {
                    flip.description()
                };
                choices.push(
                    self.offer(
                        &format!("{PAYMENT}{}:flip:{active}", space.id),
                        label,
                        can_pay,
                        format!("{effect}\n{payment}"),
                    )
                    .at(&space.id, active > 0)
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

    pub(super) fn apply_paid_rally(
        &mut self,
        space: &str,
        action: RallyAction,
    ) -> Result<(), String> {
        let step = RallyResolution {
            space: space.into(),
            action,
        };
        // The checked submission already verified cost, Available Forces,
        // qualification and that this space is unused. Charge after changing pieces.
        self.change_rally_pieces(FACTION, &step)?;
        self.resources[FACTION.index()] -= COST;
        self.paid_rally
            .as_mut()
            .expect("Paid Rally is open")
            .resolutions
            .push(step);
        Ok(())
    }

    pub(super) fn paid_rally_placement(
        &self,
        after: &Self,
        action: RallyAction,
    ) -> Vec<AvailableForcesChange> {
        let (kind, state, count) = match action {
            RallyAction::Place(n) if n > 0 => (Kind::Guerrilla, State::Underground, n),
            RallyAction::ReplaceWithBase { .. } => (Kind::Base, State::Ordinary, 1),
            _ => return vec![],
        };
        vec![AvailableForcesChange {
            piece: Group {
                faction: FACTION,
                kind,
                state,
                count,
            },
            from: total(&self.available, FACTION, kind),
            to: total(&after.available, FACTION, kind),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAMAGUEY: &str = "camaguey-province";

    fn camaguey_rally() -> Game {
        let mut game = Game::opening_with_seed(crate::save::Session::KIDNAP_ROLLS_1);
        for action in [
            "terror",
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:shaded",
            "resolve-shaded",
            "advance-card",
            "government:pass",
            "syndicate:pass",
            "advance-card",
            "m26:event",
            "manifesto:decline",
            "manifesto:place:camaguey-province:guerrilla-base",
            "manifesto:decline",
            "manifesto:decline",
            START,
        ] {
            game.apply(action).unwrap();
        }
        game
    }

    fn deploy(game: &mut Game, space: usize, group: Group) {
        game.available
            .iter_mut()
            .find(|p| p.faction == group.faction && p.kind == group.kind)
            .unwrap()
            .count -= group.count;
        game.spaces[space].pieces.push(group);
    }

    #[test]
    fn paid_recovery_flips_all_active_guerrillas_and_uses_the_space() {
        let mut game = camaguey_rally();
        // No Directorio activation is reachable in this build's preceding cards.
        // Prepare one Active and one Underground Guerrilla without changing inventory.
        let space = game.spaces.iter_mut().find(|s| s.id == CAMAGUEY).unwrap();
        let underground = space
            .pieces
            .iter_mut()
            .find(|g| g.faction == FACTION && g.kind == Kind::Guerrilla)
            .unwrap();
        assert_eq!(
            (underground.state, underground.count),
            (State::Underground, 2)
        );
        underground.count = 1;
        space.pieces.push(Group {
            faction: FACTION,
            kind: Kind::Guerrilla,
            state: State::Active,
            count: 1,
        });

        let action = "paid-rally:camaguey-province:flip:1";
        let before = game.observe();
        let choice = before
            .choices
            .iter()
            .find(|c| c.action() == action)
            .unwrap();
        let preview = choice.preview.as_ref().unwrap();
        assert_eq!(
            preview.flips,
            vec![Flip {
                space: CAMAGUEY.into(),
                faction: FACTION,
                kind: Kind::Guerrilla,
                from: State::Active,
                to: State::Underground,
                count: 1,
            }]
        );
        assert!(preview.removed.is_empty());
        assert!(preview.placement_available_forces.is_empty());
        game.submit(FACTION, before.revision, &choice.id).unwrap();

        let after = game.observe();
        let space = after.spaces.iter().find(|s| s.id == CAMAGUEY).unwrap();
        let faces: Vec<_> = space
            .pieces
            .iter()
            .filter(|g| g.faction == FACTION && g.kind == Kind::Guerrilla)
            .map(|g| (g.state, g.count))
            .collect();
        assert_eq!(faces, [(State::Underground, 2)]);
        assert_eq!(after.available, before.available);
        assert_eq!(after.resources[FACTION.index()], 4);
        assert!(!after.choices.iter().any(|c| c.space() == Some(CAMAGUEY)));
        assert!(game.submit(FACTION, before.revision, &choice.id).is_err());
        assert_eq!(game.observe(), after);
    }

    #[test]
    fn paid_exchange_offers_only_legal_guerrilla_mixes() {
        let mut game = camaguey_rally();
        let index = game.spaces.iter().position(|s| s.id == CAMAGUEY).unwrap();
        let exchange_choices = |game: &Game| {
            game.observe()
                .choices
                .into_iter()
                .filter(|c| c.action().starts_with("paid-rally:camaguey-province:base:"))
                .collect::<Vec<_>>()
        };
        // Prepare two more Guerrillas from Available and expose them to cover all
        // three mixes in the same position.
        deploy(
            &mut game,
            index,
            Group {
                faction: FACTION,
                kind: Kind::Guerrilla,
                state: State::Active,
                count: 2,
            },
        );
        let choices = exchange_choices(&game);
        assert_eq!(choices.len(), 3);
        for (underground, expected) in [
            (0, [(State::Active, 0), (State::Underground, 2)]),
            (1, [(State::Active, 1), (State::Underground, 1)]),
            (2, [(State::Active, 2), (State::Underground, 0)]),
        ] {
            let choice = choices
                .iter()
                .find(|c| c.action() == format!("paid-rally:camaguey-province:base:{underground}"))
                .unwrap();
            assert!(choice.supported);
            let mut branch = game.clone();
            branch.submit(FACTION, game.revision, &choice.id).unwrap();
            let committed = branch.observe();
            let space = &committed.spaces[index];
            for (state, count) in expected {
                assert_eq!(
                    space
                        .pieces
                        .iter()
                        .find(|p| p.faction == FACTION
                            && p.kind == Kind::Guerrilla
                            && p.state == state)
                        .map_or(0, |p| p.count),
                    count
                );
            }
            assert_eq!(committed.resources[FACTION.index()], 4);
        }

        let mut full = game.clone();
        // Another faction's non-Casino Base takes the last slot.
        deploy(
            &mut full,
            index,
            Group {
                faction: Faction::Government,
                kind: Kind::Base,
                state: State::Ordinary,
                count: 1,
            },
        );
        assert!(exchange_choices(&full).is_empty());

        let mut no_base_available = game.clone();
        let other_spaces: Vec<_> = no_base_available
            .spaces
            .iter()
            .enumerate()
            .filter(|(i, space)| {
                *i != index
                    && space.terrain != Terrain::EconomicCenter
                    && !space.pieces.iter().any(|p| p.kind == Kind::Base)
            })
            .map(|(i, _)| i)
            .take(3)
            .collect();
        assert_eq!(other_spaces.len(), 3);
        for other in other_spaces {
            deploy(
                &mut no_base_available,
                other,
                Group {
                    faction: FACTION,
                    kind: Kind::Base,
                    state: State::Ordinary,
                    count: 1,
                },
            );
        }
        assert!(exchange_choices(&no_base_available).is_empty());

        let mut short = camaguey_rally();
        short.spaces[index]
            .pieces
            .iter_mut()
            .find(|p| p.faction == FACTION && p.kind == Kind::Guerrilla)
            .unwrap()
            .count = 1;
        short
            .available
            .iter_mut()
            .find(|p| p.faction == FACTION && p.kind == Kind::Guerrilla)
            .unwrap()
            .count += 1;
        assert!(exchange_choices(&short).is_empty());

        game.resources[FACTION.index()] = 0;
        let unaffordable = exchange_choices(&game);
        assert_eq!(unaffordable.len(), 3);
        assert!(
            unaffordable
                .iter()
                .all(|c| !c.supported && c.preview.is_none())
        );
    }
}
