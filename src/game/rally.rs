//! The free Operation granted by Alberto Bayo's unshaded text.
use super::*;

/// One resolution per qualifying space, in the beneficiary's chosen order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rally {
    pub beneficiary: Faction,
    pub resolutions: Vec<RallyResolution>,
}

impl Rally {
    pub(crate) fn remaining_spaces<'a>(
        &'a self,
        spaces: &'a [Space],
    ) -> impl Iterator<Item = &'a Space> {
        rally_spaces(spaces, self.beneficiary)
            .filter(|space| !self.resolutions.iter().any(|step| step.space == space.id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RallyResolution {
    pub space: String,
    pub action: RallyAction,
}

/// R 3.3.1 gives a choice of procedure in each qualifying space, with the affected
/// piece counts its description names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RallyAction {
    Place(u8),
    FlipUnderground(u8),
    ReplaceWithBase { underground: u8, active: u8 },
}

impl RallyResolution {
    fn id(&self) -> String {
        let method = match self.action {
            RallyAction::Place(count) => format!("place:{count}"),
            RallyAction::FlipUnderground(_) => "flip".into(),
            RallyAction::ReplaceWithBase { underground, .. } => format!("base:{underground}"),
        };
        format!("rally:{}:{method}", self.space)
    }
}

impl RallyAction {
    pub(crate) fn description(self) -> String {
        match self {
            Self::Place(1) => "Place 1 Underground Guerrilla".into(),
            Self::Place(count) => format!("Place {count} Underground Guerrillas"),
            Self::FlipUnderground(1) => "Flip 1 Guerrilla Underground".into(),
            Self::FlipUnderground(count) => format!("Flip {count} Guerrillas Underground"),
            Self::ReplaceWithBase {
                underground,
                active,
            } => {
                format!("Replace {underground} Underground and {active} Active with a Base")
            }
        }
    }
}

fn beneficiary_id(faction: Faction) -> &'static str {
    match faction {
        Faction::M26 => "rally:beneficiary:26july",
        Faction::Directorio => "rally:beneficiary:directorio",
        _ => unreachable!("Alberto Bayo names 26July and Directorio"),
    }
}

pub(crate) const BENEFICIARIES: [Faction; 2] = [Faction::M26, Faction::Directorio];

fn count(pieces: &[Group], faction: Faction, kind: Kind, state: Option<State>) -> u8 {
    pieces
        .iter()
        .find(|group| {
            group.faction == faction && group.kind == kind && state.is_none_or(|s| group.state == s)
        })
        .map_or(0, |group| group.count)
}

fn rally_spaces(spaces: &[Space], beneficiary: Faction) -> impl Iterator<Item = &Space> {
    spaces.iter().filter(move |space| {
        space.terrain != Terrain::EconomicCenter
            && count(&space.pieces, beneficiary, Kind::Base, None) > 0
    })
}

pub(crate) fn rally_base_summary(spaces: &[Space], beneficiary: Faction) -> String {
    let bases: Vec<_> = rally_spaces(spaces, beneficiary)
        .map(|space| space.name.as_str())
        .collect();
    if bases.is_empty() {
        "No qualifying Base spaces. The Event finishes without a Rally effect (R 5.1.3).".into()
    } else {
        format!(
            "{}. {} directs the free Rally there.",
            bases.join(", "),
            beneficiary.name()
        )
    }
}

struct RallyTarget<'a> {
    space: &'a Space,
    beneficiary: Faction,
    available: u8,
    bases: u8,
}

impl RallyTarget<'_> {
    fn ceiling(&self) -> u8 {
        let multiplier = if self.beneficiary == Faction::M26 {
            2
        } else {
            1
        };
        multiplier * (self.bases + self.space.population)
    }

    fn explanation(&self, action: RallyAction) -> String {
        let arithmetic = if self.beneficiary == Faction::M26 {
            format!(
                "2 × {} Base + 2 × {} Population",
                self.bases, self.space.population
            )
        } else {
            format!("{} Base + {} Population", self.bases, self.space.population)
        };
        // The ledger shows the Available count; the explanation names it only where it
        // is what limits the placement (#161).
        let limit = if self.available < self.ceiling() {
            format!(
                "With {} Available Guerrillas, place up to {}.",
                self.available, self.available
            )
        } else {
            format!("Place up to {}.", self.ceiling())
        };
        let supply = match action {
            RallyAction::Place(0) => "No pieces move.".into(),
            RallyAction::Place(count) => format!(
                "This leaves {} Guerrillas in Available Forces.",
                self.available - count
            ),
            RallyAction::FlipUnderground(_) => "Available Forces stay the same.".into(),
            RallyAction::ReplaceWithBase { .. } => {
                "Return 2 Guerrillas and take 1 Base from Available Forces.".into()
            }
        };
        format!(
            "{arithmetic} = {}. {limit} {supply} Rally treats {} as Neutral. Actual {} stays. Cost: 0 Resources.",
            self.ceiling(),
            self.space.name,
            support_level(self.space.support),
        )
    }

    fn actions(&self, available_bases: u8) -> Vec<RallyAction> {
        let mut actions: Vec<_> = (0..=self.ceiling().min(self.available))
            .map(RallyAction::Place)
            .collect();
        let pieces = &self.space.pieces;
        let active = count(
            pieces,
            self.beneficiary,
            Kind::Guerrilla,
            Some(State::Active),
        );
        if active > 0 {
            actions.push(RallyAction::FlipUnderground(active));
        }
        let bases: u8 = pieces
            .iter()
            .filter(|group| group.kind == Kind::Base)
            .map(|g| g.count)
            .sum();
        if bases < 2 && available_bases > 0 {
            let underground = count(
                pieces,
                self.beneficiary,
                Kind::Guerrilla,
                Some(State::Underground),
            );
            actions.extend((0..=2).filter_map(|n| {
                (underground >= n && active >= 2 - n).then_some(RallyAction::ReplaceWithBase {
                    underground: n,
                    active: 2 - n,
                })
            }));
        }
        actions
    }
}

impl Game {
    pub(super) fn rally_beneficiary_choices(&self) -> Vec<Choice> {
        BENEFICIARIES
            .into_iter()
            .map(|faction| {
                self.offer(
                    beneficiary_id(faction),
                    format!("Grant Rally to {}", faction.name()),
                    true,
                    rally_base_summary(&self.spaces, faction),
                )
            })
            .collect()
    }

    fn rally_options(&self, rally: &Rally) -> Vec<(RallyResolution, String)> {
        let beneficiary = rally.beneficiary;
        let available = count(&self.available, beneficiary, Kind::Guerrilla, None);
        let available_bases = count(&self.available, beneficiary, Kind::Base, None);
        rally
            .remaining_spaces(&self.spaces)
            .flat_map(|space| {
                let target = RallyTarget {
                    space,
                    beneficiary,
                    available,
                    bases: count(&space.pieces, beneficiary, Kind::Base, None),
                };
                target
                    .actions(available_bases)
                    .into_iter()
                    .map(move |action| {
                        (
                            RallyResolution {
                                space: space.id.clone(),
                                action,
                            },
                            target.explanation(action),
                        )
                    })
            })
            .collect()
    }

    pub(super) fn rally_choices(&self, rally: &Rally) -> Vec<Choice> {
        self.rally_options(rally)
            .into_iter()
            .map(|(step, explanation)| {
                self.offer(&step.id(), step.action.description(), true, explanation)
            })
            .collect()
    }

    pub(super) fn select_rally_beneficiary(&mut self, action: &str) -> Result<(), String> {
        let beneficiary = BENEFICIARIES
            .into_iter()
            .find(|&f| beneficiary_id(f) == action)
            .ok_or("Unknown Rally beneficiary.")?;
        self.continue_rally(Rally {
            beneficiary,
            resolutions: vec![],
        });
        Ok(())
    }

    pub(super) fn resolve_rally(&mut self, mut rally: Rally, action: &str) -> Result<(), String> {
        let step = self
            .rally_options(&rally)
            .into_iter()
            .map(|(step, _)| step)
            .find(|step| step.id() == action)
            .ok_or("Unknown Rally choice.")?;
        self.change_rally_pieces(rally.beneficiary, &step)?;
        rally.resolutions.push(step);
        self.continue_rally(rally);
        Ok(())
    }

    pub(super) fn continue_rally(&mut self, rally: Rally) {
        self.event = if self.rally_options(&rally).is_empty() {
            Some(EventProgress::Complete(EventOutcome::Unshaded(rally)))
        } else {
            Some(EventProgress::Rally(rally))
        };
    }

    /// Pieces move in opposite directions on the map and in Available Forces.
    /// For a flip, the two changes cancel in Available Forces.
    fn change_rally_pieces(
        &mut self,
        beneficiary: Faction,
        step: &RallyResolution,
    ) -> Result<(), String> {
        let space = self
            .spaces
            .iter_mut()
            .find(|space| space.id == step.space)
            .ok_or("Unknown Rally space.")?;
        if step.action == RallyAction::Place(0) {
            return Ok(());
        }
        let mut pieces = space.pieces.clone();
        let mut available = self.available.clone();
        let (active, underground, base) = match step.action {
            RallyAction::Place(n) => (0, i16::from(n), 0),
            RallyAction::FlipUnderground(n) => (-i16::from(n), i16::from(n), 0),
            RallyAction::ReplaceWithBase {
                active,
                underground,
            } => (-i16::from(active), -i16::from(underground), 1),
        };
        for (kind, state, change) in [
            (Kind::Guerrilla, State::Active, active),
            (Kind::Guerrilla, State::Underground, underground),
            (Kind::Base, State::Ordinary, base),
        ] {
            if change != 0 {
                adjust(&mut pieces, beneficiary, kind, state, change)?;
            }
        }
        pieces.retain(|group| group.count > 0);
        for (kind, change) in [
            (Kind::Guerrilla, -active - underground),
            (Kind::Base, -base),
        ] {
            if change != 0 {
                let group = available
                    .iter_mut()
                    .find(|group| group.faction == beneficiary && group.kind == kind)
                    .ok_or("Rally choice has no piece supply.")?;
                group.count = shifted(group.count, change)?;
            }
        }
        space.pieces = pieces;
        self.available = available;
        Ok(())
    }
}

fn shifted(count: u8, delta: i16) -> Result<u8, String> {
    u8::try_from(i16::from(count) + delta).map_err(|_| "Rally piece count is impossible.".into())
}

fn adjust(
    pieces: &mut Vec<Group>,
    faction: Faction,
    kind: Kind,
    state: State,
    delta: i16,
) -> Result<(), String> {
    if let Some(group) = pieces
        .iter_mut()
        .find(|g| g.faction == faction && g.kind == kind && g.state == state)
    {
        group.count = shifted(group.count, delta)?;
    } else {
        pieces.push(Group {
            faction,
            kind,
            state,
            count: shifted(0, delta)?,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sierra_maestra(game: &Game) -> &Space {
        game.spaces
            .iter()
            .find(|space| space.id == "sierra-maestra")
            .unwrap()
    }

    /// R 3.3.1: 26July places up to 2 × (Bases + Population) in a Base space; R 1.4.1:
    /// never more than its Available Guerrillas. Sierra Maestra has 1 Base and
    /// Population 1, a ceiling of 4. The prepared scenario always has 12 Available, so
    /// the binding case is a constructed target (#161).
    #[test]
    fn the_explanation_names_the_available_count_only_where_it_binds() {
        let game = Game::opening();
        let target = |available| RallyTarget {
            space: sierra_maestra(&game),
            beneficiary: Faction::M26,
            available,
            bases: 1,
        };
        // Below the ceiling, Available is the limit: 3 Guerrillas allow 3 of 4.
        let short = target(3);
        assert_eq!(short.actions(3).last(), Some(&RallyAction::Place(3)));
        let text = short.explanation(RallyAction::Place(3));
        assert!(
            text.starts_with(
                "2 × 1 Base + 2 × 1 Population = 4. With 3 Available Guerrillas, place up to 3. \
                 This leaves 0 Guerrillas in Available Forces. "
            ),
            "{text}"
        );
        // At or above the ceiling, the ceiling alone.
        for available in [4, 12] {
            let text = target(available).explanation(RallyAction::Place(4));
            assert!(
                text.starts_with(&format!(
                    "2 × 1 Base + 2 × 1 Population = 4. Place up to 4. \
                     This leaves {} Guerrillas in Available Forces. ",
                    available - 4
                )),
                "{text}"
            );
            assert!(!text.contains("Available Guerrillas"), "{text}");
        }
        // "Place 0" says no pieces move, then the space's other facts.
        assert_eq!(
            target(12).explanation(RallyAction::Place(0)),
            "2 × 1 Base + 2 × 1 Population = 4. Place up to 4. No pieces move. \
             Rally treats Sierra Maestra as Neutral. Actual Active Opposition stays. \
             Cost: 0 Resources."
        );
    }

    /// The flip and the Base replacement state their change to Available Forces, and
    /// the rules make it (R 3.3.1). The prepared scenario has no Guerrilla in Sierra
    /// Maestra at its Rally, so this position adds 2 Active 26July Guerrillas there.
    #[test]
    fn a_flip_and_a_base_state_and_make_their_change_to_available_forces() {
        let mut rally = crate::terror_chosen(Game::opening_with_seed(5));
        for action in [
            "kidnap",
            "kidnap:roll",
            "terror:santiago",
            "end-terror",
            "event:unshaded",
            "rally:beneficiary:26july",
        ] {
            rally.apply(action).unwrap();
        }
        let space = rally
            .spaces
            .iter_mut()
            .find(|space| space.id == "sierra-maestra")
            .unwrap();
        space.pieces.push(Group {
            faction: Faction::M26,
            kind: Kind::Guerrilla,
            state: State::Active,
            count: 2,
        });
        let explanation = |game: &Game, action: &str| {
            game.observe()
                .choices
                .into_iter()
                .find(|choice| choice.action() == action)
                .unwrap_or_else(|| panic!("{action} is offered"))
                .explanation
        };
        let supply = |game: &Game, kind| count(&game.available, Faction::M26, kind, None);
        assert_eq!(
            (supply(&rally, Kind::Guerrilla), supply(&rally, Kind::Base)),
            (12, 3)
        );

        let mut flip = rally.clone();
        let text = explanation(&flip, "rally:sierra-maestra:flip");
        assert!(text.contains("Available Forces stay the same."), "{text}");
        flip.apply("rally:sierra-maestra:flip").unwrap();
        assert_eq!(
            (supply(&flip, Kind::Guerrilla), supply(&flip, Kind::Base)),
            (12, 3)
        );
        assert_eq!(
            count(
                &sierra_maestra(&flip).pieces,
                Faction::M26,
                Kind::Guerrilla,
                Some(State::Underground)
            ),
            2
        );

        let mut base = rally;
        let text = explanation(&base, "rally:sierra-maestra:base:0");
        assert!(
            text.contains("Return 2 Guerrillas and take 1 Base from Available Forces."),
            "{text}"
        );
        base.apply("rally:sierra-maestra:base:0").unwrap();
        assert_eq!(
            (supply(&base, Kind::Guerrilla), supply(&base, Kind::Base)),
            (14, 2)
        );
        let pieces = &sierra_maestra(&base).pieces;
        assert_eq!(count(pieces, Faction::M26, Kind::Base, None), 2);
        assert_eq!(count(pieces, Faction::M26, Kind::Guerrilla, None), 0);
    }
}
