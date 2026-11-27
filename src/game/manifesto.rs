//! The optional placements granted by Sierra Maestra Manifesto (card 19).
use super::*;

const PLACE: &str = "manifesto:place:";

/// Card 19's printed order (third-printing Playbook, p. 33).
pub(super) const CARD_19_ORDER: [Faction; 4] = [
    Faction::M26,
    Faction::Directorio,
    Faction::Syndicate,
    Faction::Government,
];

/// The committed placements and declines during the Event. Each list follows
/// the card's order, and a faction appears exactly once across the two.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifesto {
    pub declines: Vec<Faction>,
    pub placements: Vec<ManifestoPlacement>,
}

impl Manifesto {
    pub fn pending(&self) -> Option<Faction> {
        CARD_19_ORDER.get(self.decided()).copied()
    }

    pub fn complete(&self) -> bool {
        self.pending().is_none()
    }

    fn decided(&self) -> usize {
        self.declines.len() + self.placements.len()
    }

    fn handoff(&self) -> String {
        match CARD_19_ORDER.get(self.decided() + 1) {
            Some(faction) => format!("{} decides next.", faction.name()),
            None => {
                "The Event ends. 26July stays Eligible. Directorio has the 2nd Eligible choices."
                    .into()
            }
        }
    }
}

/// One committed mix from Available Forces in a single space.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestoPlacement {
    pub faction: Faction,
    pub space: String,
    pub pieces: Vec<Group>,
}

/// Available counts before and after a placement for one kind in the mix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlacementSupply {
    pub piece: Group,
    pub from: u8,
    pub to: u8,
}

impl ManifestoPlacement {
    fn id(&self) -> String {
        let kinds = self.pieces.iter().flat_map(|piece| {
            let token = match piece.kind {
                Kind::Troops => "troops",
                Kind::Police => "police",
                Kind::Guerrilla => "guerrilla",
                Kind::Base => "base",
                Kind::Casino => unreachable!("The Manifesto never places Casinos"),
            };
            std::iter::repeat_n(token, usize::from(piece.count))
        });
        format!(
            "{PLACE}{}:{}",
            self.space,
            kinds.collect::<Vec<_>>().join("-")
        )
    }

    pub(crate) fn label(&self) -> String {
        self.pieces
            .iter()
            .map(|piece| piece_words(piece, false))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub(crate) fn piece_description(&self) -> String {
        self.pieces
            .iter()
            .map(|piece| piece_words(piece, true))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    fn quantity(&self) -> u8 {
        self.pieces.iter().map(|piece| piece.count).sum()
    }
}

pub(crate) fn is_placement(action: &str) -> bool {
    action.starts_with(PLACE)
}

pub(crate) fn piece_words(piece: &Group, with_state: bool) -> String {
    let name = match (piece.kind, piece.count == 1) {
        (Kind::Troops, true) => "Troop",
        (Kind::Troops, false) => "Troops",
        (Kind::Police, true) => "Police cube",
        (Kind::Police, false) => "Police cubes",
        (Kind::Guerrilla, true) => "Guerrilla",
        (Kind::Guerrilla, false) => "Guerrillas",
        (Kind::Base, true) => "Base",
        (Kind::Base, false) => "Bases",
        (Kind::Casino, true) => "Casino",
        (Kind::Casino, false) => "Casinos",
    };
    let state = if with_state {
        match piece.state {
            State::Underground => "Underground ",
            State::Active => "Active ",
            State::Open => "open ",
            State::Closed => "closed ",
            State::Ordinary => "",
        }
    } else {
        ""
    };
    format!("{} {state}{name}", piece.count)
}

/// Own presence includes every state of qualifying piece, even a closed Casino.
pub(crate) fn presence(space: &Space, faction: Faction) -> Option<String> {
    let pieces = space
        .pieces
        .iter()
        .filter(|p| p.faction == faction && p.count > 0)
        .map(|p| piece_words(p, true))
        .collect::<Vec<_>>();
    (!pieces.is_empty()).then(|| format!("{} has {}", faction.name(), pieces.join(", ")))
}

impl Game {
    fn manifesto_placements(&self, faction: Faction) -> Vec<ManifestoPlacement> {
        let supply: Vec<_> = self
            .available
            .iter()
            .filter(|g| g.faction == faction && g.kind != Kind::Casino && g.count > 0)
            .collect();
        let mut plans = Vec::new();
        for space in &self.spaces {
            if !space
                .pieces
                .iter()
                .any(|p| p.faction == faction && p.count > 0)
            {
                continue;
            }
            let base_capacity = if space.terrain == Terrain::EconomicCenter {
                0
            } else {
                let bases = space
                    .pieces
                    .iter()
                    .filter(|p| p.kind == Kind::Base)
                    .map(|p| p.count)
                    .sum::<u8>();
                2u8.saturating_sub(bases)
            };
            let kinds: Vec<_> = supply
                .iter()
                .filter_map(|g| {
                    let count = if g.kind == Kind::Base {
                        g.count.min(base_capacity)
                    } else {
                        g.count
                    };
                    (count > 0).then_some((g.kind, count))
                })
                .collect();
            let placement = |pieces| ManifestoPlacement {
                faction,
                space: space.id.clone(),
                pieces,
            };
            for (i, &(first, available)) in kinds.iter().enumerate() {
                plans.push(placement(vec![group(faction, first, 1)]));
                if available >= 2 {
                    plans.push(placement(vec![group(faction, first, 2)]));
                }
                for &(second, _) in &kinds[i + 1..] {
                    plans.push(placement(vec![
                        group(faction, first, 1),
                        group(faction, second, 1),
                    ]));
                }
            }
        }
        // If anything can be done in full, R 5.1.3 does not permit selecting a lesser
        // effect. Compare all qualifying spaces because the Event leaves the space open.
        let most = plans
            .iter()
            .map(ManifestoPlacement::quantity)
            .max()
            .unwrap_or(0);
        plans.retain(|plan| plan.quantity() == most);
        plans
    }

    pub(super) fn manifesto_decisions(&self, manifesto: &Manifesto) -> Vec<Choice> {
        let Some(faction) = manifesto.pending() else {
            return vec![
                self.offer(
                    "directorio:pass",
                    "Pass",
                    false,
                    "R 2.3.3. Directorio would stay Eligible and gain 1 Resource.\nExecuting Pass is outside this continuation.",
                ),
                self.offer(
                    "directorio:operation",
                    "Operation with optional Special Activity",
                    false,
                    "R 2.3.4. An Operation may include a Special Activity.\nExecuting it is outside this continuation.",
                ),
            ];
        };
        let after = manifesto.handoff();
        let placements = self.manifesto_placements(faction);
        let no_placement = if placements.is_empty() {
            "No placement is possible from Available Forces in a qualifying space. "
        } else {
            ""
        };
        let mut choices = Vec::with_capacity(placements.len() + 1);
        for plan in placements {
            let space = self.spaces.iter().find(|s| s.id == plan.space).unwrap();
            let qualifies = presence(space, faction).expect("A placement needs own presence");
            let partial = if plan.quantity() == 1 {
                " Only one piece can be placed (R 5.1.3)."
            } else {
                ""
            };
            choices.push(self.offer(
                &plan.id(),
                plan.label(),
                true,
                format!("{}: {qualifies}. Place {} from Available Forces at no Resource cost.{partial} {after}",
                    space.name, plan.piece_description()),
            ).at(&plan.space, true));
        }
        choices.push(self.offer(
            MANIFESTO_DECLINE,
            "Decline placement",
            true,
            format!(
                "{no_placement}{} places no pieces and spends no Resources. {after}",
                faction.name()
            ),
        ));
        choices
    }

    pub(super) fn apply_manifesto_placement(&mut self, action: &str) -> Result<(), String> {
        let faction = self.owner();
        let plan = self
            .manifesto_placements(faction)
            .into_iter()
            .find(|p| p.id() == action)
            .ok_or("The placement is no longer legal.")?;
        let space = self.spaces.iter_mut().find(|s| s.id == plan.space).unwrap();
        for piece in &plan.pieces {
            let available = self
                .available
                .iter_mut()
                .find(|g| g.faction == faction && g.kind == piece.kind)
                .expect("A checked placement has this Available supply");
            available.count -= piece.count;
            if let Some(on_map) = space
                .pieces
                .iter_mut()
                .find(|g| g.faction == faction && g.kind == piece.kind && g.state == piece.state)
            {
                on_map.count += piece.count;
            } else {
                space.pieces.push(piece.clone());
            }
        }
        self.manifesto
            .as_mut()
            .expect("Manifesto decision")
            .placements
            .push(plan);
        Ok(())
    }

    pub(super) fn manifesto_supply_preview(&self, after: &Game) -> Vec<PlacementSupply> {
        let plan = after
            .manifesto
            .as_ref()
            .and_then(|event| event.placements.last())
            .expect("The preview resolved a placement");
        plan.pieces
            .iter()
            .map(|piece| PlacementSupply {
                piece: piece.clone(),
                from: total(&self.available, plan.faction, piece.kind),
                to: total(&after.available, plan.faction, piece.kind),
            })
            .collect()
    }

    pub(super) fn manifesto_space_reason(&self, space: &Space, here: &[&Choice]) -> String {
        let faction = self.owner();
        if !here.is_empty() {
            "Placement available here.".into()
        } else if space
            .pieces
            .iter()
            .any(|p| p.faction == faction && p.count > 0)
        {
            "No legal mix fits here.".into()
        } else {
            format!("{} has no piece here.", faction.name())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R 5.1.3 carries out only what is possible. This position isolates the
    /// supply boundary, unreachable on card 19 of the prepared match: reserve
    /// one or zero of the six Syndicate Guerrillas while Havana's Casino
    /// remains an eligible sole presence at the one-piece boundary.
    #[test]
    fn manifesto_at_the_partial_and_empty_supply_boundaries() {
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
            "government:pass",
            "syndicate:pass",
            "advance-card",
            MANIFESTO_EVENT,
            MANIFESTO_DECLINE,
            MANIFESTO_DECLINE,
        ] {
            game.apply(action).unwrap();
        }
        let faction = Faction::Syndicate;
        for available in [1, 0] {
            let mut game = game.clone();
            game.spaces
                .iter_mut()
                .find(|s| s.id == "la-habana")
                .unwrap()
                .pieces
                .push(group(faction, Kind::Guerrilla, 6 - available));
            game.available
                .iter_mut()
                .find(|g| g.faction == faction && g.kind == Kind::Guerrilla)
                .unwrap()
                .count = available;
            let before = game.observe();
            let mut declined = game.clone();
            declined.apply(MANIFESTO_DECLINE).unwrap();
            let after_decline = declined.observe();
            assert_eq!(after_decline.owner, Faction::Government);
            assert_eq!(after_decline.spaces, before.spaces);
            assert_eq!(after_decline.available, before.available);
            assert_eq!(after_decline.standings, before.standings);

            let refused = format!(
                "{}:manifesto:place:havana:guerrilla-guerrilla",
                before.revision
            );
            assert!(game.submit(faction, before.revision, &refused).is_err());
            assert_eq!(game.observe(), before);
            if available == 0 {
                assert_eq!(before.choices.len(), 1);
                assert_eq!(before.choices[0].action(), MANIFESTO_DECLINE);
                continue;
            }

            let havana = before
                .choices
                .iter()
                .filter(|c| c.space() == Some("havana"))
                .collect::<Vec<_>>();
            assert_eq!(havana.len(), 1);
            assert_eq!(havana[0].label, "1 Guerrilla");
            assert!(
                havana[0]
                    .explanation
                    .contains("Only one piece can be placed")
            );
            assert_eq!(
                havana[0].preview.as_ref().unwrap().placement_supply,
                [PlacementSupply {
                    piece: group(faction, Kind::Guerrilla, 1),
                    from: 1,
                    to: 0
                },]
            );
            game.submit(faction, before.revision, &havana[0].id)
                .unwrap();
            let after = game.observe();
            let city = after.spaces.iter().find(|s| s.id == "havana").unwrap();
            assert_eq!(total(&city.pieces, faction, Kind::Guerrilla), 1);
            assert_eq!(total(&city.pieces, faction, Kind::Casino), 1);
            assert_eq!(city.control(), Some(Faction::Government));
            assert_eq!(total(&after.available, faction, Kind::Guerrilla), 0);
            assert_eq!(after.standings, before.standings);
            assert_eq!(after.owner, Faction::Government);
        }
    }
}
