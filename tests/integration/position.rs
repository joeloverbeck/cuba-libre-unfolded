use cuba_libre_unfolded::{Faction, Game};

#[test]
fn zero_and_positive_government_margins_do_not_satisfy_the_city_condition() {
    // R7.2/7.3 arithmetic challenge at the public model seam. These deliberately
    // altered boards are not admitted tactical saves or reachable slice states.
    let mut record = serde_json::to_value(crate::terror_chosen(Game::opening())).unwrap();
    record["spaces"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"] == "las-villas")
        .unwrap()["support"] = 1.into();
    let zero: Game = serde_json::from_value(record.clone()).unwrap();
    assert_eq!(zero.standings()[0].score, 18);
    assert_eq!(zero.standings()[0].margin, 0);
    assert_eq!(
        zero.standings()[0].extra,
        "Active Support cities 1/3 (not met)"
    );
    record["spaces"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"] == "la-habana")
        .unwrap()["support"] = 2.into();
    let positive: Game = serde_json::from_value(record).unwrap();
    assert_eq!(positive.standings()[0].score, 19);
    assert_eq!(positive.standings()[0].margin, 1);
    assert_eq!(
        positive.standings()[0].extra,
        "Active Support cities 1/3 (not met)"
    );
}

// Accepted handoff deacd635, prepared table and independently worked standings.
#[test]
fn prepared_position_exposes_complete_public_state_without_hidden_order() {
    let game = crate::terror_chosen(Game::opening());
    let view = game.observe();
    assert_eq!(view.resources, [6, 0, 5, 15]);
    assert_eq!(view.spaces.len(), 13);
    assert_eq!(
        view.spaces.iter().map(|s| s.population as u16).sum::<u16>(),
        17
    );
    assert_eq!(view.spaces.iter().map(|s| s.economy as u16).sum::<u16>(), 8);
    assert_eq!(view.map_piece_count, 29);
    assert_eq!(view.available_piece_count, 57);
    // Literal per-space oracle transcribed from the accepted prepared-position
    // table, independent of the production constructor and standing calculation.
    let rows: Vec<_> = view
        .spaces
        .iter()
        .map(|space| {
            (
                space.id.as_str(),
                space.population,
                space.economy,
                space.support,
                space.control(),
                space
                    .pieces
                    .iter()
                    .map(|p| format!("{:?}/{:?}/{:?}:{}", p.faction, p.kind, p.state, p.count))
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .collect();
    use Faction::*;
    assert_eq!(rows, vec![
        ("havana",6,0,2,Some(Government),"Government/Troops/Ordinary:6,Government/Police/Ordinary:4,Directorio/Guerrilla/Underground:2,Syndicate/Casino/Open:1".into()),
        ("camaguey-city",1,0,1,Some(Government),"Government/Troops/Ordinary:1,Government/Police/Ordinary:2".into()),
        ("santiago",1,0,0,Some(Government),"Government/Troops/Ordinary:2,Government/Police/Ordinary:1,M26/Guerrilla/Underground:2".into()),
        ("pinar-del-rio",1,0,2,Some(Syndicate),"Syndicate/Casino/Open:1".into()),
        ("la-habana",1,0,1,Some(Syndicate),"Syndicate/Casino/Open:1".into()),
        ("matanzas",1,0,-1,None,"".into()),
        ("las-villas",2,0,0,Some(Government),"Government/Troops/Ordinary:3".into()),
        ("camaguey-province",1,0,-1,Some(Directorio),"Directorio/Guerrilla/Underground:1".into()),
        ("oriente",2,0,-1,Some(M26),"M26/Guerrilla/Underground:1".into()),
        ("sierra-maestra",1,0,-2,Some(M26),"M26/Base/Ordinary:1".into()),
        ("ec-pinar-la-habana",0,3,0,None,"".into()),
        ("ec-las-villas-camaguey",0,3,0,None,"".into()),
        ("ec-oriente-sierra",0,2,0,None,"".into()),
    ]);
    assert!(view.spaces.iter().all(|s| s.terror == 0));
    assert_eq!(
        view.available
            .iter()
            .map(|p| format!("{:?}/{:?}:{}", p.faction, p.kind, p.count))
            .collect::<Vec<_>>(),
        [
            "Government/Troops:3",
            "Government/Police:8",
            "Government/Base:2",
            "M26/Guerrilla:12",
            "M26/Base:3",
            "Directorio/Guerrilla:12",
            "Directorio/Base:4",
            "Syndicate/Guerrilla:6",
            "Syndicate/Casino:7",
        ]
    );
    assert_eq!(
        view.standings.iter().map(|s| s.score).collect::<Vec<_>>(),
        [16, 7, 1, 3]
    );
    assert_eq!(
        view.standings.iter().map(|s| s.margin).collect::<Vec<_>>(),
        [-2, -8, -8, -15]
    );
    assert_eq!(
        view.standings[0].extra,
        "Active Support cities 1/3 (not met)"
    );
    assert_eq!(view.owner, Faction::M26);
    assert_eq!(view.current_card, 17);
    assert_eq!(view.preview_card, 18);
    assert_eq!(view.draw_count, 34);
    assert_eq!(
        view.played_cards,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 49, 13, 14, 15, 16]
    );
    let public = serde_json::to_value(&view).unwrap();
    assert!(public.get("draw_deck").is_none());
}

#[test]
fn zero_space_finish_has_no_pass_income_or_deck_advance_and_refuses_wrong_authority() {
    let initial = crate::terror_chosen(Game::opening());
    for (owner, revision, choice) in [
        (Faction::Government, 1, "1:end-terror"),
        (Faction::M26, 2, "1:end-terror"),
        (Faction::M26, 1, "2:end-terror"),
        (Faction::M26, 1, "1:remove:santiago:police"),
    ] {
        let mut game = initial.clone();
        assert!(game.submit(owner, revision, choice).is_err());
        assert_eq!(game, initial);
    }
    let mut game = initial.clone();
    game.submit(Faction::M26, 1, "1:end-terror").unwrap();
    let view = game.observe();
    assert_eq!(view.resources, [6, 0, 5, 15]);
    assert_eq!(view.spaces, initial.observe().spaces);
    assert_eq!(view.revision, 2);
    assert_eq!(view.owner, Faction::Directorio);
    assert_eq!(view.eligible, [true; 4]);
    assert_eq!(view.current_card, 17);
    assert_eq!(view.draw_count, 34);
    assert_eq!(
        view.next.as_ref().unwrap().options,
        ["Pass (+1 if executed later)", "Limited Operation"]
    );
    assert_eq!(game.random_state(), initial.random_state());
    let finished = game.clone();
    assert!(game.submit(Faction::M26, 1, "1:end-terror").is_err());
    assert_eq!(game, finished);
}
