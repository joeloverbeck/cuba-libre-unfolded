//! Whether the table is still. The development seat waits for everything that is still
//! to come to rest before it lets an input in; an agent's choice (`faction_seats`) waits
//! for what would change what the human watches or reads, and for what the human opened.
use super::*;

/// How long a landed Kidnap die shows its result before an agent's next choice may take
/// it away, unless Continue closes it first: the human has to see it land (#197, story
/// 49), and an agent's faction has no reason to press Continue.
const RESULT: f64 = 2.;

/// Something at the table that has not come to rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Unsettled {
    Admission,
    HoverDwell,
    StatusDwell,
    Redraw,
    HomeFraming,
    PieceCounts,
    Camera,
    TerrorToken,
    /// A space popover's place, the Kidnap die, a card in the air or the piles.
    Part(String),
}

impl Unsettled {
    /// As the development seat names it.
    pub(super) fn words(self) -> String {
        match self {
            Self::Admission => "asset/save admission",
            Self::HoverDwell => "standing hover dwell",
            Self::StatusDwell => "region status hover dwell",
            Self::Redraw => "control redraw",
            Self::HomeFraming => "Home framing",
            Self::PieceCounts => "piece counts",
            Self::Camera => "camera movement",
            Self::TerrorToken => "Terror token spawn",
            Self::Part(words) => return words,
        }
        .into()
    }

    /// Whether an agent's choice waits for it. The human's own looking about does not
    /// hold one: the camera with the counts that follow it, and a hover's dwell.
    fn holds_a_choice(&self) -> bool {
        !matches!(
            self,
            Self::HoverDwell | Self::StatusDwell | Self::Camera | Self::PieceCounts
        )
    }
}

pub(super) fn unsettled(world: &mut World) -> Vec<Unsettled> {
    let play = world.resource::<Play>();
    let ui = world.resource::<Interface>();
    let mut pending = vec![];
    if !play.admitted() {
        pending.push(Unsettled::Admission);
    }
    if ui.hover.waiting() {
        pending.push(Unsettled::HoverDwell);
    }
    if ui.status.waiting() {
        pending.push(Unsettled::StatusDwell);
    }
    if ui.dirty {
        pending.push(Unsettled::Redraw);
    }
    if play.admitted() && !world.resource::<areas::Island>().surveyed() {
        pending.push(Unsettled::HomeFraming);
    }
    if !ui.piece_flips.is_empty() || world.resource::<piece_flips::Flights>().moving() {
        pending.push(Unsettled::Part("piece flip".into()));
    }
    if world.resource::<Counts>().settling > 0 {
        pending.push(Unsettled::PieceCounts);
    }
    if world.resource::<Orbit>().is_moving() {
        pending.push(Unsettled::Camera);
    }
    // A newly placed bell is ready once its glTF scene spawned and plays the
    // loop. The loop itself never holds the table unsettled.
    let tokens = world.query::<&TerrorToken>().iter(world).count();
    let playing = world.query::<&TerrorPlayer>().iter(world).count();
    if playing < tokens {
        pending.push(Unsettled::TerrorToken);
    }
    let parts = [popover::unsettled, die::unsettled, piles::unsettled].map(|of| of(world));
    pending.extend(parts.into_iter().flatten().map(Unsettled::Part));
    pending
}

/// What has still to come to rest before an agent's choice or an edition: something in
/// motion, or a die whose result has just landed. Not the human's looking about.
pub(super) fn resting(world: &mut World) -> Vec<String> {
    let mut resting: Vec<String> = unsettled(world)
        .into_iter()
        .filter(Unsettled::holds_a_choice)
        .map(Unsettled::words)
        .collect();
    if result_showing(world) {
        resting.push("Kidnap result".into());
    }
    resting
}

/// What holds an agent's choice now, as the development seat reads it: what is
/// [`resting`], or something the human opened. Empty when a choice would be applied at
/// once.
pub(super) fn holding(world: &mut World) -> Vec<String> {
    let mut holding = resting(world);
    holding.extend(opened(world));
    holding
}

/// Whether a landed die has shown its result for less than [`RESULT`] seconds.
pub(super) fn result_showing(world: &World) -> bool {
    let now = world.resource::<Time>().elapsed_secs_f64();
    die::shown(world)
        && world
            .resource::<die::Roll>()
            .landed_at()
            .is_some_and(|landed| now - landed < RESULT)
}

/// What the human has open over the table. What a decision's own step draws is not the
/// human's: an agent's transfer or Kidnap would otherwise wait for itself (#197,
/// correction 2). The transfer modal is shown only for a decision of a faction they play.
fn opened(world: &World) -> Vec<String> {
    let Some(game) = world.resource::<Play>().session.as_ref().map(Session::game) else {
        return vec![];
    };
    let ui = world.resource::<Interface>();
    let popover = world.resource::<popover::Popover>().open_at(game.revision);
    let modal = transfer::shown(game.transfer.as_ref(), game.owner(), ui);
    [
        (popover, "an open popover"),
        (modal, "the transfer modal"),
        (
            choice::opened(game, ui, popover),
            "a pinned or opened briefing",
        ),
        (ui.card.pinned.is_some(), "a pinned card tag"),
        (ui.editions.holds(), "an edition"),
        (ui.composer.is_open(), "the proclamation composer"),
        (ui.proclamation_reader.is_open(), "a proclamation"),
    ]
    .into_iter()
    .filter(|(open, _)| *open)
    .map(|(_, what)| what.into())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #203: camera movement and hover dwell do not hold an agent's choice; a card or a
    /// die in motion, and what has still to be drawn, do.
    #[test]
    fn looking_about_holds_no_choice_and_motion_does() {
        use Unsettled::*;
        for looking in [HoverDwell, StatusDwell, Camera, PieceCounts] {
            assert!(!looking.holds_a_choice(), "{looking:?}");
        }
        let die = Part("Kidnap die landing".into());
        for moving in [Admission, Redraw, HomeFraming, TerrorToken, die] {
            assert!(moving.holds_a_choice(), "{moving:?}");
        }
    }
}
