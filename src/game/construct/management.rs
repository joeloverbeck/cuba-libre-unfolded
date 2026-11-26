//! Voluntary management of Syndicate pieces during Construct (R 1.4.1, 1.4.5).

use super::*;

const CLOSE: &str = "close-casino:";
const RETURN: &str = "remove:";

impl Game {
    pub(in crate::game) fn construct_manage_choice(&self) -> Choice {
        self.offer(
            "remove",
            "Manage own pieces",
            true,
            "Choose a Syndicate group on the map. Close open Casinos in place or return pieces to Available Forces (R 1.4.1, 1.4.5).",
        )
    }

    pub(in crate::game) fn construct_management_choices(&self) -> Vec<Choice> {
        let mut choices = vec![];
        for space in &self.spaces {
            for piece in space.pieces.iter().filter(|p| p.faction == SYNDICATE) {
                let group = removal_group(piece.kind, piece.state);
                for n in 1..=piece.count {
                    let name = match (piece.kind, n) {
                        (Kind::Casino, 1) => "Casino",
                        (Kind::Casino, _) => "Casinos",
                        (Kind::Guerrilla, 1) => "Guerrilla",
                        (Kind::Guerrilla, _) => "Guerrillas",
                        _ => unreachable!("Syndicate pieces are Casinos or Guerrillas"),
                    };
                    let selection = format!("{}:{group}:{n}", space.id);
                    let closed = piece.kind == Kind::Casino && piece.state == State::Open;
                    let actions = [
                        (
                            CLOSE,
                            format!("Close {n} {name} in place"),
                            "R 1.4.1, 1.4.5",
                        ),
                        (RETURN, format!("Return {n} {name} to Available"), "R 1.4.1"),
                    ];
                    for (prefix, label, rule) in actions.into_iter().skip(usize::from(!closed)) {
                        let action = format!("{prefix}{selection}");
                        let refusal = self.management_refusal(&action);
                        choices.push(
                            self.offer(
                                &action,
                                &label,
                                refusal.is_none(),
                                refusal.unwrap_or_else(|| {
                                    format!("{} · {label} ({rule}).", space.name)
                                }),
                            ),
                        );
                    }
                }
            }
        }
        choices.push(self.offer(
            "done-removal",
            "Done",
            true,
            "Return to Construct. Any Casino changes already made stay in effect.",
        ));
        choices
    }

    fn management_refusal(&self, action: &str) -> Option<String> {
        if !self.construct_payment_owed() {
            return None;
        }
        let mut after = self.clone();
        after.move_managed_piece(action).ok()?;
        if after.can_afford_construct_payment(after.construct_balance()) {
            return None;
        }

        let remedy = if self.can_afford_construct_payment(self.construct_balance()) {
            "Choose Done and pay for a legal space first. Then this action is available."
        } else {
            "Syndicate needs an affordable legal space. Fund and pay for Construct first to allow this action."
        };
        Some(format!(
            "Unpaid Limited Construct must retain an affordable legal space (local policy). {remedy}"
        ))
    }

    pub(in crate::game) fn apply_construct_management(
        &mut self,
        action: &str,
    ) -> Result<(), String> {
        if action == "done-removal" {
            self.removal = None;
            return Ok(());
        }
        self.move_managed_piece(action)?;
        self.mark_construct_side_effect();
        Ok(())
    }

    fn move_managed_piece(&mut self, action: &str) -> Result<(), &'static str> {
        if let Some(selection) = action.strip_prefix(RETURN) {
            return self.return_own_pieces(SYNDICATE, selection);
        }
        let (space, group, count) = action
            .strip_prefix(CLOSE)
            .and_then(piece_selection)
            .ok_or("Unknown Casino action.")?;
        if group != "open-casino" {
            return Err("Choose open Casinos to close.");
        }
        let target = self
            .spaces
            .iter_mut()
            .find(|s| s.id == space)
            .ok_or("Unknown Casino space.")?;
        let open = target
            .pieces
            .iter_mut()
            .find(|p| p.kind == Kind::Casino && p.state == State::Open)
            .ok_or("No open Casinos here.")?;
        open.count = open
            .count
            .checked_sub(count)
            .ok_or("Unknown Casino quantity.")?;
        target.pieces.retain(|p| p.count > 0);
        if let Some(closed) = target
            .pieces
            .iter_mut()
            .find(|p| p.kind == Kind::Casino && p.state == State::Closed)
        {
            closed.count += count;
        } else {
            target.pieces.push(Group {
                faction: SYNDICATE,
                kind: Kind::Casino,
                state: State::Closed,
                count,
            });
        }
        Ok(())
    }

    pub(in crate::game) fn management_preview(
        &self,
        after: &Self,
        action: &str,
    ) -> Option<ManagementPreview> {
        let (id, _, _) = action
            .strip_prefix(CLOSE)
            .or_else(|| action.strip_prefix(RETURN))
            .and_then(piece_selection)?;
        let was_legal = |game: &Self, here: &Space| {
            if game.construct_payment_owed() {
                ConstructPlan::ALL
                    .into_iter()
                    .filter(|plan| game.construct_payment_requirement(here, *plan).is_ok())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        };
        let mut opened = vec![];
        let mut withheld = vec![];
        for (before, now) in self.spaces.iter().zip(&after.spaces) {
            let plans_before = was_legal(self, before);
            let plans_after = was_legal(after, now);
            for plan in ConstructPlan::ALL {
                if plans_before.contains(&plan) != plans_after.contains(&plan) {
                    let list = if plans_after.contains(&plan) {
                        &mut opened
                    } else {
                        &mut withheld
                    };
                    list.push(format!("{} in {}", plan.label(), now.name));
                }
            }
        }
        let mut construct_options = vec![];
        if !withheld.is_empty() {
            construct_options.push(format!("After Done, unavailable: {}.", withheld.join(", ")));
        }
        if !opened.is_empty() {
            construct_options.push(format!("After Done, available: {}.", opened.join(", ")));
        }
        if self.construct_paid().iter().any(|paid| paid.space == id) {
            let name = &self.spaces.iter().find(|s| s.id == id).unwrap().name;
            construct_options.push(format!("{name} stays a paid space."));
        }
        Some(ManagementPreview {
            casinos_available: [self.casino_inventory(), after.casino_inventory()],
            construct_options,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_affordable_space_blocks_management_until_paid() {
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

        // Authority fixture: move Government's on-map pieces into Havana. Put two
        // open Casinos there. Havana can neither place nor reopen, La Habana has
        // Directorio Control, and Pinar's sole open Casino gives the only legal site.
        let mut government = vec![];
        for space in &mut game.spaces {
            space.pieces.retain(|group| {
                if group.faction == Faction::Government {
                    government.push(group.clone());
                    false
                } else {
                    true
                }
            });
        }
        let havana = game.spaces.iter_mut().find(|s| s.id == "havana").unwrap();
        for kind in [Kind::Troops, Kind::Police] {
            let count = total(&government, Faction::Government, kind);
            havana.pieces.push(Group {
                faction: Faction::Government,
                kind,
                state: State::Ordinary,
                count,
            });
        }
        let casino = havana
            .pieces
            .iter_mut()
            .find(|p| p.kind == Kind::Casino)
            .unwrap();
        casino.state = State::Open;
        casino.count = 2;
        game.available
            .iter_mut()
            .find(|p| p.kind == Kind::Casino)
            .unwrap()
            .count -= 1;
        let legal: Vec<_> = game
            .choices()
            .into_iter()
            .filter(|c| c.supported && c.space().is_some())
            .map(|c| c.action().to_owned())
            .collect();
        assert_eq!(legal, ["construct:pinar-del-rio:place"]);

        game.apply("remove").unwrap();
        let before = game.clone();
        for action in [
            "close-casino:pinar-del-rio:open-casino:1",
            "remove:pinar-del-rio:open-casino:1",
        ] {
            let id = format!("{}:{action}", game.revision);
            let error = game.submit(SYNDICATE, game.revision, &id).unwrap_err();
            assert!(
                error.contains("Choose Done and pay for a legal space first"),
                "{error}"
            );
            assert_eq!(game, before);
        }
        game.apply("done-removal").unwrap();
        game.apply("construct:pinar-del-rio:place").unwrap();
        game.apply("remove").unwrap();
        game.apply("close-casino:pinar-del-rio:open-casino:1")
            .unwrap();
        let place = game
            .spaces
            .iter()
            .find(|s| s.id == "pinar-del-rio")
            .unwrap();
        assert_eq!(casinos(place, State::Closed), 2);
        assert_eq!(place.control(), None);
        let available = game.casino_inventory();

        for count in [0, 1, 2, 3] {
            let mut branch = game.clone();
            let result = branch.apply(&format!("remove:pinar-del-rio:closed-casino:{count}"));
            if matches!(count, 0 | 3) {
                assert!(result.is_err());
                assert_eq!(branch, game);
                continue;
            }
            result.unwrap();
            let place = branch
                .spaces
                .iter()
                .find(|s| s.id == "pinar-del-rio")
                .unwrap();
            assert_eq!(casinos(place, State::Closed), 2 - count);
            assert_eq!(branch.casino_inventory(), available + count);
            branch.apply("done-removal").unwrap();
            assert_eq!(branch.construct_paid()[0].space, "pinar-del-rio");
            assert!(branch.apply("construct:pinar-del-rio:place").is_err());
            branch.apply("end-construct").unwrap();
        }
    }
}
