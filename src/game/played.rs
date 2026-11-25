//! Which of the Operations and Special Activities of the rules this build plays.
use super::*;

/// Where this build lets a faction take an Operation or a Special Activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Played {
    Nowhere,
    /// Among the options of the faction's action on this card.
    OnCard(u8),
    /// As the free Rally that this card's unshaded Event grants (R 5.5).
    FreeRally(u8),
}

impl Played {
    /// Where this build plays `name`, an Operation or Special Activity of `faction`, as the
    /// options the authority lists to the faction say: one that can be chosen, or that
    /// another choice of the menu opens, as Terror opens Kidnap. What a menu offers does not
    /// depend on the position, only its amounts do.
    pub fn of(faction: Faction, name: &str) -> Self {
        let game = Game::opening();
        let (card, options) = match faction {
            Faction::M26 => (17, game.opening_choices()),
            Faction::Directorio => (17, game.directorio_choices()),
            Faction::Government => (18, game.government_choices()),
            Faction::Syndicate => (18, game.syndicate_choices()),
        };
        let action = name.to_lowercase().replace(' ', "-");
        if options.iter().any(|option| {
            option.action().rsplit(':').next() == Some(&action)
                && (option.supported || !option.remedies.is_empty())
        }) {
            Self::OnCard(card)
        } else if action == "rally" && rally::BENEFICIARIES.contains(&faction) {
            Self::FreeRally(17)
        } else {
            Self::Nowhere
        }
    }
}
