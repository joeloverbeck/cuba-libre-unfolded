//! #161: the ledger shows every Available count, so the texts that only repeat one
//! are gone, and the texts that state a change stay. Each is read here through the
//! rules path, beside the change the rules make. Expected values are worked from the
//! prepared position: 26July holds 12 Guerrillas and 3 Bases Available, and Sierra
//! Maestra its Base with Population 1, so a free Rally there places up to
//! 2 × (1 + 1) = 4 (R 3.3.1). Government holds 3 Troops Available (p. 20 chart).
use cuba_libre_unfolded::{Faction, Game, Kind};

fn choose(game: &mut Game, action: &str) {
    let view = game.observe();
    game.submit(
        view.owner,
        view.revision,
        &format!("{}:{action}", view.revision),
    )
    .unwrap_or_else(|error| panic!("{action}: {error}"));
}

fn offered(game: &Game, action: &str) -> cuba_libre_unfolded::Choice {
    game.observe()
        .choices
        .into_iter()
        .find(|choice| choice.action() == action)
        .unwrap_or_else(|| panic!("{action} is offered"))
}

fn notes(choice: &cuba_libre_unfolded::Choice) -> String {
    choice.preview.as_ref().unwrap().notes.join(" ")
}

fn available(game: &Game, faction: Faction, kind: Kind) -> u8 {
    game.observe()
        .available
        .iter()
        .filter(|group| group.faction == faction && group.kind == kind)
        .map(|group| group.count)
        .sum()
}

#[test]
fn a_rally_gives_the_ceiling_alone_where_available_does_not_bind() {
    let [_, mut game] = crate::explore::event_starts();
    choose(&mut game, "rally:beneficiary:26july");
    for count in 0..=4 {
        let text = offered(&game, &format!("rally:sierra-maestra:place:{count}")).explanation;
        assert!(
            text.starts_with("2 × 1 Base + 2 × 1 Population = 4. Place up to 4. "),
            "{text}"
        );
        assert!(!text.contains("Available Guerrillas"), "{text}");
        assert!(!text.contains("stay in Available"), "{text}");
    }
    assert_eq!(
        offered(&game, "rally:sierra-maestra:place:0").explanation,
        "2 × 1 Base + 2 × 1 Population = 4. Place up to 4. No pieces move. \
         Rally treats Sierra Maestra as Neutral. Actual Active Opposition stays. \
         Cost: 0 Resources."
    );
    let four = offered(&game, "rally:sierra-maestra:place:4").explanation;
    assert!(
        four.contains("This leaves 8 Guerrillas in Available Forces."),
        "{four}"
    );
    choose(&mut game, "rally:sierra-maestra:place:4");
    assert_eq!(available(&game, Faction::M26, Kind::Guerrilla), 8);
}

#[test]
fn every_removal_message_says_the_pieces_return_to_available() {
    // 26July's voluntary removal during Terror (R 1.4.1).
    let mut terror = crate::terror_chosen(Game::opening());
    choose(&mut terror, "remove");
    let one = offered(&terror, "remove:sierra-maestra:base:1");
    assert_eq!(
        one.explanation,
        "R 1.4.1: return 1 26July Base in Sierra Maestra to Available. Earlier Terror \
         markers, support shifts and Kidnap results stay."
    );
    assert!(notes(&one).contains("R 1.4.1: the piece returns to Available."));
    let two = offered(&terror, "remove:santiago:underground:2");
    assert!(notes(&two).contains("R 1.4.1: the pieces return to Available."));
    choose(&mut terror, "remove:santiago:underground:2");
    assert_eq!(available(&terror, Faction::M26, Kind::Guerrilla), 14);
    assert_eq!(
        offered(&terror, "done-removal").explanation,
        "Returns to execution. The removed pieces stay in Available."
    );

    // During the free Rally, removing its only Base space leaves no Rally choice.
    let [_, mut rally] = crate::explore::event_starts();
    choose(&mut rally, "rally:beneficiary:26july");
    choose(&mut rally, "remove");
    choose(&mut rally, "remove:sierra-maestra:base:1");
    assert_eq!(available(&rally, Faction::M26, Kind::Base), 4);
    assert!(
        offered(&rally, "done-removal")
            .explanation
            .starts_with("No legal Rally choices remain. The removed pieces stay in Available")
    );

    // Government's removal before its card 18 Assault, then the Assault's own
    // returns (R 3.2.4): the funded Santiago position of `assault.rs`.
    let mut funded = crate::terror_chosen(Game::opening_with_seed(2));
    for action in [
        "kidnap",
        "kidnap:roll",
        "terror:santiago",
        "transfer",
        "give:0:1",
        "end-terror",
        "event:unshaded",
        "rally:beneficiary:26july",
        "rally:sierra-maestra:place:0",
        "advance-card",
        "government:assault",
    ] {
        choose(&mut funded, action);
    }
    let mut removal = funded.clone();
    assert!(
        offered(&removal, "remove")
            .explanation
            .starts_with("Choose Government pieces on the map to return to Available Forces")
    );
    choose(&mut removal, "remove");
    assert_eq!(
        offered(&removal, "remove:santiago:troops:1").explanation,
        "R 1.4.1: return 1 Troop in Santiago de Cuba to Available Forces."
    );
    choose(&mut removal, "remove:santiago:troops:1");
    assert_eq!(available(&removal, Faction::Government, Kind::Troops), 4);

    choose(&mut funded, "assault:areas:cities");
    choose(&mut funded, "assault:space:santiago");
    let payment = offered(&funded, "assault:santiago:26july");
    assert!(
        payment
            .explanation
            .contains("Return 1 26July Active Guerrilla to Available Forces"),
        "{}",
        payment.explanation
    );
    let before = available(&funded, Faction::M26, Kind::Guerrilla);
    choose(&mut funded, payment.action());
    assert_eq!(
        available(&funded, Faction::M26, Kind::Guerrilla),
        before + 1
    );
    let finish = offered(&funded, "end-assault").explanation;
    assert!(
        finish.contains("Returned 1 26July Active Guerrilla to Available Forces"),
        "{finish}"
    );
}
