//! Save · quit (#174; spec #166): selected graphic S2 · Etiqueta, a paper plate in the
//! bottom-right corner. Every decision is saved as it is made, so the button only
//! closes the game; relaunching with the same save resumes the match.
//!
//! The plate is the VectorCraft export of `art/atlas/atlas-save-quit.vectorcraft`, at
//! rest and hovered (`save-quit.sha256`). It fills the main view's area `save-quit`,
//! anchored to the bottom-right corner 40 reference pixels from each edge
//! (`art/atlas/atlas-main-view.figcraft`, MAIN-VIEW.md). The control keeps its name,
//! `quit`. It shows its hovered state under the pointer and under keyboard focus, and
//! its hover tag is the top display's (`top::Item::SaveQuit`).
use super::*;
use bevy::picking::hover::HoverMap;

/// The plate at rest and hovered: 152×42 reference pixels, exported at 3×.
pub(super) const REST: &str = "atlas-save-quit.png";
pub(super) const HOVER: &str = "atlas-save-quit-hover.png";
/// The control's name, which the development seat and keyboard focus use.
pub(super) const NAME: &str = "quit";

/// Marks the drawn plate.
#[derive(Component)]
pub(super) struct Plate;

/// Draws the plate in the main view's `save-quit` area.
pub(super) fn draw(commands: &mut Commands, art: &Art, layout: &areas::Layout, window: &Window) {
    let rect = layout.area("save-quit", window);
    commands
        .spawn((
            UiRoot,
            ImageNode::new(art.images[REST].clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch),
            Node {
                position_type: PositionType::Absolute,
                left: px(rect.min.x),
                top: px(rect.min.y),
                width: px(rect.width()),
                height: px(rect.height()),
                ..default()
            },
            GlobalZIndex(2),
            Control {
                name: NAME.into(),
                choice: false,
                action: Action::Quit,
            },
            Plate,
            areas::Part("save-quit".into()),
        ))
        .observe(click);
}

/// Whether the plate shows its hovered state: under the pointer, or keyboard focus.
fn hovered(over: bool, ui: &Interface) -> bool {
    over || ui.choice.focus.as_deref() == Some(NAME)
}

/// Swaps the plate's export as the pointer or the keyboard focus comes and goes.
pub(super) fn hover(
    hover: Res<HoverMap>,
    ui: Res<Interface>,
    art: Res<Art>,
    mut plates: Query<(Entity, &mut ImageNode), With<Plate>>,
) {
    for (entity, mut image) in &mut plates {
        let over = hover
            .get(&PointerId::Mouse)
            .is_some_and(|hits| hits.contains_key(&entity));
        let file = if hovered(over, &ui) { HOVER } else { REST };
        let handle = &art.images[file];
        if image.image != *handle {
            image.image = handle.clone();
        }
    }
}

/// Which export the plate shows now, for the seat (`table.save_quit`).
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let hover = world.resource::<Art>().images[HOVER].clone();
    world
        .query_filtered::<&ImageNode, With<Plate>>()
        .iter(world)
        .next()
        .map_or(
            serde_json::Value::Null,
            |image| serde_json::json!({"hovered": image.image == hover}),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> serde_json::Value {
        serde_json::from_str(include_str!("../../art/atlas/atlas-main-view.layout.json")).unwrap()
    }

    /// The plate is 40 reference pixels (67.5 on the owner's display) from the right
    /// and the bottom edges, as layout A · Franja places it, and its area is its own
    /// 152×42 export, at 3×.
    #[test]
    fn the_plate_keeps_40_from_the_corner() {
        let record = record();
        let area = &record["areas"]["save-quit"];
        assert_eq!(area["anchor"], "bottom-right");
        let [x, y, w, h]: [f64; 4] = serde_json::from_value(area["bounds"].clone()).unwrap();
        assert!(
            (3840. - (x + w) - 67.5).abs() < 0.01,
            "right: {}",
            3840. - (x + w)
        );
        assert!(
            (2160. - (y + h) - 67.5).abs() < 0.01,
            "bottom: {}",
            2160. - (y + h)
        );
        assert!((w - 152. * 1.6875).abs() < 0.01 && (h - 42. * 1.6875).abs() < 0.01);
        assert_eq!(record["parts"]["save-quit"], area["bounds"]);
        for file in [REST, HOVER] {
            let bytes = std::fs::read(format!("art/atlas/{file}")).unwrap();
            let image = image::load_from_memory(&bytes).unwrap();
            assert_eq!((image.width(), image.height()), (456, 126), "{file}");
            assert!(
                crate::assets::SAVE_QUIT_MANIFEST.contains(&format!("art/atlas/{file}")),
                "{file} is not admitted"
            );
        }
    }

    /// The pointer or keyboard focus shows the hovered plate.
    #[test]
    fn the_pointer_or_focus_hovers_the_plate() {
        let mut ui = Interface::default();
        assert!(!hovered(false, &ui));
        assert!(hovered(true, &ui));
        ui.choice.focus = Some(NAME.into());
        assert!(hovered(false, &ui));
    }
}
