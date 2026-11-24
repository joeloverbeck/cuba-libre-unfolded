//! Notices and the notice slip (#174; spec #166).
//!
//! Every notice is written with its kind: a decision that succeeded, a refusal, an
//! error, or the loading state before the match opens. Nothing reads the kind out of
//! the sentence. The development seat reports every notice's words as before
//! (`table.notice`, and a launch error as `table.error`); the owner sees a notice only
//! on the slip, which is drawn for a refusal, an error and the loading state. A
//! success is not drawn: its result is on the map, the ledgers and the Sequence of
//! Play track.
//!
//! The slip reuses the space popover's card plate, its `action/primary` top rule, its
//! width and its `Popover/Regular 12.5` text, with inline emblems. It sits above Save ·
//! quit, their right edges aligned, where the main view's layout record places
//! `notice-slip` (`art/atlas/atlas-main-view.figcraft`, MAIN-VIEW.md): its right edge
//! and its foot keep their distance from the window's bottom-right corner, and it grows
//! upward with its lines. It takes the pointer over its own rectangle, and a click on
//! it dismisses it; the next notice replaces it.
use super::palette::INK;
use super::*;
use emblems::Naming;
use serde_json::json;

/// What a notice reports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Kind {
    /// A decision was committed. Its result shows on the table, so it is not drawn.
    #[default]
    Success,
    /// The rules, or a control that cannot act yet, refused: nothing changed.
    Refusal,
    /// Something failed: the save, or the launch.
    Error,
    /// The match is not open yet.
    Loading,
}

impl Kind {
    /// Whether the slip draws a notice of this kind.
    pub(super) fn drawn(self) -> bool {
        self != Self::Success
    }
}

/// The latest notice, as it was written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Notice {
    pub kind: Kind,
    pub text: String,
    /// Set by a click on the slip. The words stay reported to the seat.
    dismissed: bool,
}

impl Notice {
    fn new(kind: Kind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            dismissed: false,
        }
    }
    pub(super) fn success(text: impl Into<String>) -> Self {
        Self::new(Kind::Success, text)
    }
    pub(super) fn refusal(text: impl Into<String>) -> Self {
        Self::new(Kind::Refusal, text)
    }
    pub(super) fn error(text: impl Into<String>) -> Self {
        Self::new(Kind::Error, text)
    }
    /// What a submission's failure is: the rules' refusal, or the save's error.
    pub(super) fn failed(failure: &crate::save::Submit) -> Self {
        match failure {
            crate::save::Submit::Refused(_) => Self::refusal(failure.to_string()),
            crate::save::Submit::Save(_) => Self::error(failure.to_string()),
        }
    }
    pub(super) fn dismiss(&mut self) {
        self.dismissed = true;
    }
}

/// Writes a refusal: a control that cannot act yet, by a click or by Enter, and the
/// transfer modal's blocked primary button.
pub(super) fn refuse(ui: &mut Interface, reason: impl Into<String>) {
    ui.notice = Notice::refusal(reason);
    ui.dirty = true;
}

/// The loading state's words, drawn until the match opens.
pub(super) const LOADING: &str = "Loading selected assets; no save created.";

/// What the slip shows, if anything: a launch error, else the loading state until the
/// match opens, else the latest notice when it is a refusal or an error that has not
/// been dismissed.
pub(super) fn slip(play: &Play, ui: &Interface) -> Option<Notice> {
    if let Some(error) = &play.error {
        return Some(Notice::error(error.clone()));
    }
    if play.session.is_none() {
        return Some(Notice::new(Kind::Loading, LOADING));
    }
    let notice = &ui.notice;
    (notice.kind.drawn() && !notice.dismissed && !notice.text.is_empty()).then(|| notice.clone())
}

/// The slip's geometry in reference pixels: the space popover's (`popover::frame`,
/// A-P1), which the main view's `notice-slip` repeats.
mod frame {
    /// The text's inset from the card's sides, and its space below the rule.
    pub const INSET: f32 = 14.;
    pub const TOP: f32 = 9.;
    /// Below the last line, inside the card's 1 px stroke.
    pub const BOTTOM: f32 = 11.;
    /// The rule's height.
    pub const RULE: f32 = 3.;
}

/// Marks the drawn slip, which the seat reports as `table.slip`.
#[derive(Component)]
pub(super) struct Slip(pub Notice);

/// Draws the slip, if one is shown, in the main view's `notice-slip` place.
pub(super) fn draw(
    commands: &mut Commands,
    art: &Art,
    shown: Option<Notice>,
    layout: &areas::Layout,
    window: &Window,
) {
    let Some(notice) = shown else {
        return;
    };
    let s = reference_pixel(window);
    let place = layout.slip(window);
    let card = commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(place.min.x),
                bottom: px(window.height() - place.max.y),
                width: px(place.width()),
                flex_direction: FlexDirection::Column,
                // The plate draws the 1 px stroke inside this padding.
                padding: UiRect::new(px(s), px(s), px(s), px((frame::BOTTOM + 1.) * s)),
                ..default()
            },
            GlobalZIndex(5),
            Control {
                name: "notice-slip".into(),
                choice: false,
                action: Action::DismissNotice,
            },
            Slip(notice.clone()),
        ))
        .observe(click)
        .id();
    popover::plate(commands, art, card, popover::plate::CARD, s);
    // The popover's `action/primary` rule, stretched across the card.
    commands.spawn((
        ImageNode::new(art.images[popover::plate::RULE_PRIMARY].clone())
            .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
        Node {
            height: px(frame::RULE * s),
            flex_shrink: 0.,
            ..default()
        },
        Pickable::IGNORE,
        ChildOf(card),
    ));
    let body = commands
        .spawn((
            Node {
                padding: UiRect::new(
                    px(frame::INSET * s),
                    px(frame::INSET * s),
                    px(frame::TOP * s),
                    px(0.),
                ),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(card),
        ))
        .id();
    popover::words(
        commands,
        &popover::Ink { art },
        body,
        &notice.text,
        Naming::Emblems,
        false,
        &art.font,
        popover::frame::LINE_SIZE * s,
        INK,
        true,
    );
}

/// The drawn slip for the seat: whether it is shown, its kind and words, and its
/// rectangle in logical window pixels as [left, top, right, bottom].
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let drawn = world
        .query::<(&Slip, &ComputedNode, &UiGlobalTransform)>()
        .iter(world)
        .next()
        .map(|(slip, node, at)| {
            json!({
                "shown": true,
                "kind": slip.0.kind,
                "text": slip.0.text,
                "bounds": seat::logical_bounds(node, at),
            })
        });
    drawn.unwrap_or_else(|| json!({"shown": false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::Session;

    use super::super::test_play as play;

    fn shown(play: &Play, ui: &Interface) -> Option<(Kind, String)> {
        slip(play, ui).map(|notice| (notice.kind, notice.text))
    }

    /// Only a refusal, an error and the loading state show the slip.
    #[test]
    fn only_three_kinds_are_drawn() {
        assert!(!Kind::Success.drawn());
        for kind in [Kind::Refusal, Kind::Error, Kind::Loading] {
            assert!(kind.drawn(), "{kind:?}");
        }
    }

    /// Before the match opens: the loading state, and a launch error instead once
    /// admission fails (`admit` writes it to `play.error`, the seat's `error`).
    #[test]
    fn before_the_match_opens_the_slip_shows_loading_then_a_launch_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut play = play(dir.path().join("task.json"));
        play.session = None;
        let ui = Interface::default();
        assert_eq!(shown(&play, &ui), Some((Kind::Loading, LOADING.into())));
        play.error = Some("Required art missing: atlas-save-quit.png".into());
        assert_eq!(
            shown(&play, &ui),
            Some((
                Kind::Error,
                "Required art missing: atlas-save-quit.png".into()
            ))
        );
    }

    /// The submit path (`decide`, which a click, Enter, a popover's button and the
    /// transfer modal all reach): a committed decision is a success and draws no slip,
    /// while the seat still reads its sentence; a decision the rules refuse is a
    /// refusal; a save another session changed is an error. Each replaces the last.
    #[test]
    fn the_submit_path_types_each_outcome() {
        let dir = tempfile::tempdir().unwrap();
        let save = dir.path().join("task.json");
        let mut play = play(save.clone());
        let mut ui = Interface::default();

        // The rules refuse a choice that is not offered: the position is unchanged.
        decide(&mut play, &mut ui, Faction::M26, 0, "0:not-a-choice");
        assert_eq!(ui.notice.kind, Kind::Refusal);
        assert_eq!(play.game().observe().revision, 0);
        assert_eq!(
            shown(&play, &ui),
            Some((Kind::Refusal, ui.notice.text.clone()))
        );
        assert!(!ui.notice.text.is_empty());

        // 26July chooses Terror: a success replaces the refusal and hides the slip.
        decide(&mut play, &mut ui, Faction::M26, 0, "0:terror");
        assert_eq!(ui.notice.kind, Kind::Success);
        assert_eq!(play.game().observe().revision, 1);
        assert!(!ui.notice.text.is_empty(), "the seat still reads it");
        assert_eq!(shown(&play, &ui), None);

        // Another session writes the save: the next legal decision cannot be saved.
        let mut other = Session::open(&save, None).unwrap();
        let view = other.observe();
        let kidnap = view
            .choices
            .iter()
            .find(|c| c.action() == "kidnap")
            .unwrap();
        other
            .submit(kidnap.owner, view.revision, &kidnap.id)
            .unwrap();
        decide(&mut play, &mut ui, Faction::M26, 1, "1:kidnap");
        assert_eq!(ui.notice.kind, Kind::Error);
        assert_eq!(
            ui.notice.text,
            "Save changed in another session; reload before continuing"
        );
        assert_eq!(play.game().observe().revision, 1);
        assert_eq!(
            shown(&play, &ui),
            Some((Kind::Error, ui.notice.text.clone()))
        );
    }

    /// A control that cannot act yet (`Action::Unsupported`), by a click or by Enter,
    /// and the transfer modal's blocked primary button write refusals.
    #[test]
    fn a_refusal_shows_until_dismissed_and_the_next_notice_replaces_it() {
        let dir = tempfile::tempdir().unwrap();
        let play = play(dir.path().join("task.json"));
        let mut ui = Interface::default();
        refuse(&mut ui, "Not offered yet.");
        assert_eq!(
            shown(&play, &ui),
            Some((Kind::Refusal, "Not offered yet.".into()))
        );
        ui.notice.dismiss();
        assert_eq!(shown(&play, &ui), None);
        assert_eq!(
            ui.notice.text, "Not offered yet.",
            "the seat still reads it"
        );
        refuse(&mut ui, "Not offered yet.");
        assert_eq!(
            shown(&play, &ui),
            Some((Kind::Refusal, "Not offered yet.".into())),
            "a new notice is shown again"
        );
    }
}
