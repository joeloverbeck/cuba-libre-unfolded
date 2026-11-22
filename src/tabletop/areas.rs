//! The main view's layout, `art/atlas/atlas-main-view.figcraft` (MAIN-VIEW.md).
use super::*;

/// Names a drawn part by its node name in `atlas-main-view.figcraft`, so the
/// development seat can compare native placement with the layout.
#[derive(Component)]
pub(super) struct Part(pub String);

/// What the seat reports of the main view (`table.main_view`).
pub(super) fn observe(world: &mut World) -> serde_json::Value {
    let mut parts = world
        .query::<(&Part, &ComputedNode, &UiGlobalTransform, Option<&emblems::Plain>)>()
        .iter(world)
        .map(|(part, node, at, text)| {
            (
                part.0.clone(),
                serde_json::json!({
                    "part": part.0,
                    "text": text.map(|t| &t.0),
                    "bounds": seat::logical_bounds(node, at),
                }),
            )
        })
        .collect::<Vec<_>>();
    // The Sequence of Play track's plate, which its own module draws and marks.
    for (node, at) in world
        .query_filtered::<(&ComputedNode, &UiGlobalTransform), With<eligibility::Drawn>>()
        .iter(world)
    {
        parts.push((
            "sequence-plate".into(),
            serde_json::json!({
                "part": "sequence-plate",
                "text": null,
                "bounds": seat::logical_bounds(node, at),
            }),
        ));
    }
    parts.sort_by(|a, b| a.0.cmp(&b.0));
    serde_json::json!({
        "parts": parts.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
    })
}
