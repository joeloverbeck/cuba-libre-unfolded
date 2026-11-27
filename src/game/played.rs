//! Which Operations, Special Activities and Events this build plays. Ordinary coverage
//! comes from the offered menus. The free Rally and the Manifesto use their own
//! declarations of the factions that can execute them.
use super::*;
use std::sync::LazyLock;

/// Where this build lets a faction take an Operation or a Special Activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Played {
    Nowhere,
    /// Among the options of the faction's action on this card.
    OnCard(u8),
    /// As the free Rally that this card's unshaded Event grants the faction that Directorio
    /// selects, in each space where it has a Base (R 5.1, 5.5).
    FreeRally(u8),
}

/// The options this build lists for each faction's action, in `Faction::ALL`'s order, and the
/// card they are on.
static MENUS: LazyLock<[(u8, Vec<Choice>); 4]> = LazyLock::new(|| {
    let game = Game::opening();
    Faction::ALL.map(|faction| match faction {
        Faction::M26 => (17, game.opening_choices()),
        Faction::Directorio => (17, game.directorio_choices()),
        Faction::Government => (18, game.government_choices()),
        Faction::Syndicate => {
            let mut choices = game.syndicate_choices();
            choices.push(game.limited_construct_choice());
            (18, choices)
        }
    })
});

impl Played {
    /// Where this build plays `name`, an Operation or Special Activity of `faction`: an
    /// option that can be chosen, or that another of the menu opens, as Terror opens Kidnap.
    pub fn of(faction: Faction, name: &str) -> Self {
        let (card, options) = &MENUS[faction.index()];
        let action = name.to_lowercase().replace(' ', "-");
        if options.iter().any(|option| {
            let name_matches = option.action().rsplit(':').next() == Some(&action)
                || (action == "construct" && option.action() == "syndicate:limited-operation");
            name_matches && (option.supported || !option.remedies.is_empty())
        }) {
            Self::OnCard(*card)
        } else if action == "rally" && rally::BENEFICIARIES.contains(&faction) {
            Self::FreeRally(17)
        } else {
            Self::Nowhere
        }
    }
}

/// The factions that may execute the Event of `card` in this build.
pub(super) fn event_by(card: u8) -> Vec<Faction> {
    if card == 19 {
        return vec![manifesto::EXECUTOR];
    }
    Faction::ALL
        .into_iter()
        .filter(|faction| {
            let (on, options) = &MENUS[faction.index()];
            let events = |option: &Choice| option.supported && option.event_text().is_some();
            *on == card && options.iter().any(events)
        })
        .collect()
}
