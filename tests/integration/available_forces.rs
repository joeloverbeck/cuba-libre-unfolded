//! #159: the Available Forces ledger's art and its admission, without a GPU. #160:
//! the inventories the ledger's tag reads, against every reached position.
use crate::explore::{Explored, all_explored, parallel};
use cuba_libre_unfolded::{Faction, Game, assets};

/// `available-forces.sha256` admits the plate and the nine pieces.
#[test]
fn the_ledger_s_plate_and_pieces_are_admitted() {
    assert_eq!(
        crate::admission::admitted(assets::AVAILABLE_FORCES_MANIFEST),
        [
            "atlas-available-forces-plate.png",
            "atlas-available-troops.png",
            "atlas-available-police.png",
            "atlas-available-government-base.png",
            "atlas-available-26july-guerrilla.png",
            "atlas-available-26july-base.png",
            "atlas-available-directorio-guerrilla.png",
            "atlas-available-directorio-base.png",
            "atlas-available-syndicate-guerrilla.png",
            "atlas-available-syndicate-casino.png",
        ]
    );
}

/// #160: the nine inventories of the Available Forces chart (rules, p. 20), read from
/// the chart rather than the implementation: Government 15 Troops, 15 Police and 2
/// Bases; 26July and Directorio 15 Guerrillas and 4 Bases each; Syndicate 6
/// Guerrillas and 10 Casinos (its Bases).
#[test]
fn the_chart_s_nine_inventories_are_declared_once() {
    use cuba_libre_unfolded::{INVENTORY, Kind, inventory};
    assert_eq!(
        INVENTORY,
        [
            (Faction::Government, Kind::Troops, 15),
            (Faction::Government, Kind::Police, 15),
            (Faction::Government, Kind::Base, 2),
            (Faction::M26, Kind::Guerrilla, 15),
            (Faction::M26, Kind::Base, 4),
            (Faction::Directorio, Kind::Guerrilla, 15),
            (Faction::Directorio, Kind::Base, 4),
            (Faction::Syndicate, Kind::Guerrilla, 6),
            (Faction::Syndicate, Kind::Casino, 10),
        ]
    );
    // A kind the chart does not give a faction has none.
    assert_eq!(inventory(Faction::Government, Kind::Guerrilla), 0);
    assert_eq!(inventory(Faction::M26, Kind::Casino), 0);
    assert_eq!(inventory(Faction::Syndicate, Kind::Base), 0);
    // The opening derives its Available Forces from them: the accepted tactical
    // handoff's 3, 8, 2; 12, 3; 12, 4; 6, 7, in the chart's order.
    let opening = Game::opening().observe();
    let available: Vec<_> = opening
        .available
        .iter()
        .map(|g| (g.faction, g.kind, g.count))
        .collect();
    assert_eq!(
        available,
        [
            (Faction::Government, Kind::Troops, 3),
            (Faction::Government, Kind::Police, 8),
            (Faction::Government, Kind::Base, 2),
            (Faction::M26, Kind::Guerrilla, 12),
            (Faction::M26, Kind::Base, 3),
            (Faction::Directorio, Kind::Guerrilla, 12),
            (Faction::Directorio, Kind::Base, 4),
            (Faction::Syndicate, Kind::Guerrilla, 6),
            (Faction::Syndicate, Kind::Casino, 7),
        ]
    );
}

/// Each kind's Available count plus its count on the map, whatever their states (a
/// closed Casino is on the map), equals its inventory, and no other piece exists.
fn assert_inventories_hold(game: &Game, at: &str) {
    use cuba_libre_unfolded::{INVENTORY, inventory};
    let view = game.observe();
    for &(faction, kind, total) in &INVENTORY {
        let count = |groups: &mut dyn Iterator<Item = &cuba_libre_unfolded::Group>| -> u32 {
            groups
                .filter(|g| g.faction == faction && g.kind == kind)
                .map(|g| u32::from(g.count))
                .sum()
        };
        let available = count(&mut view.available.iter());
        let on_map = count(&mut view.spaces.iter().flat_map(|s| &s.pieces));
        assert_eq!(
            available + on_map,
            u32::from(total),
            "{at}: {faction:?} {kind:?}: {available} Available + {on_map} on the map"
        );
    }
    for group in view
        .available
        .iter()
        .chain(view.spaces.iter().flat_map(|s| &s.pieces))
    {
        assert!(
            inventory(group.faction, group.kind) > 0 || group.count == 0,
            "{at}: {group:?} is outside the chart"
        );
    }
}

/// #160: the inventories hold in every position the explorer reaches from its starts
/// and its Event starts (the positions `seat_reference` and `honest_hints` check), and
/// through a Havana Assault that closes a Casino, which stays on the map.
#[test]
fn every_reached_position_keeps_each_inventory() {
    let mut positions = 0;
    for Explored { start, nodes } in all_explored() {
        parallel(nodes, |i, node| {
            assert_inventories_hold(&node.game, &format!("{start}, explored position {i}"));
        });
        positions += nodes.len();
    }
    assert!(positions > 100, "{positions}");

    let mut game = crate::terror_chosen(Game::opening_with_seed(2));
    for action in [
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
    ] {
        crate::explore::choose(&mut game, action);
        assert_inventories_hold(&game, action);
    }
    let closed = game.observe().spaces[0]
        .pieces
        .iter()
        .find(|g| g.kind == cuba_libre_unfolded::Kind::Casino)
        .map(|g| g.state);
    assert_eq!(closed, Some(cuba_libre_unfolded::State::Closed));
}
