//! #159: the Available Forces ledger's art and its admission, without a GPU. #160:
//! the inventories the ledger's tag reads, against every reached position.
use cuba_libre_unfolded::{Faction, Game, assets};

/// Hash admission (`available-forces.sha256`): a missing or changed plate or piece
/// refuses launch before any save, like every other manifest.
#[test]
fn launch_refuses_a_missing_or_changed_ledger_export() {
    let art = tempfile::tempdir().unwrap();
    for line in assets::MANIFESTS.iter().flat_map(|m| m.lines()) {
        let (_, path) = line.split_once("  ").unwrap();
        let to = art.path().join(path.strip_prefix("art/atlas/").unwrap());
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(path, to).unwrap();
    }
    // No private cards here: intact art passes and reaches the card check.
    let cards = tempfile::tempdir().unwrap();
    let error = || {
        format!(
            "{:#}",
            assets::inspect(art.path(), cards.path()).err().unwrap()
        )
    };
    let intact = error();
    assert!(intact.contains("Required private card"), "{intact}");
    let admitted: Vec<&str> = assets::AVAILABLE_FORCES_MANIFEST
        .lines()
        .map(|line| {
            line.split_once("  ")
                .unwrap()
                .1
                .strip_prefix("art/atlas/")
                .unwrap()
        })
        .collect();
    assert_eq!(
        admitted,
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
    for name in admitted {
        let path = art.path().join(name);
        let original = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let missing = error();
        assert!(
            missing.contains("Required art missing") && missing.contains(name),
            "{missing}"
        );
        let mut changed = original.clone();
        changed[original.len() / 2] ^= 1;
        std::fs::write(&path, changed).unwrap();
        let changed = error();
        assert!(
            changed.contains("Required art changed") && changed.contains(name),
            "{changed}"
        );
        std::fs::write(&path, original).unwrap();
    }
    assert!(error().contains("Required private card"));
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
    let starts = crate::explore::starts().into_iter().map(|(_, start)| start);
    let mut positions = 0;
    for start in starts.chain(crate::explore::event_starts()) {
        for node in crate::explore::explore(start) {
            assert_inventories_hold(&node.game, &format!("explored position {positions}"));
            positions += 1;
        }
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
        "assault:areas:cities",
        "assault:space:havana",
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
