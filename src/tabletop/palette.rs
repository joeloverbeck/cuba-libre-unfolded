//! The FigCraft `Atlas` colour variables the interface uses, in one place
//! (foundation 4, "Layout authority": the selected layouts' variables own the
//! palette). Each colour names the variable it stands for, and a unit test checks
//! it against every selected layout's committed variable export
//! (`art/atlas/README.md`, "Interface palette").
use bevy::prelude::Color;

/// Declares each colour and its variable together, so the test's list
/// (`VARIABLES`) cannot omit one.
macro_rules! atlas {
    ($($(#[$doc:meta])* $name:ident = $variable:literal ($r:literal, $g:literal, $b:literal);)+) => {
        $($(#[$doc])* pub(super) const $name: Color = Color::srgb_u8($r, $g, $b);)+
        /// Every colour above, by the `Atlas` variable it stands for.
        #[cfg(test)]
        const VARIABLES: &[(&str, [u8; 3])] = &[$(($variable, [$r, $g, $b])),+];
    };
}

atlas! {
    /// `ink`: titles, values and body text.
    INK = "ink" (32, 50, 56);
    /// `ink-secondary`: secondary values, notes and quiet labels.
    QUIET = "ink-secondary" (83, 106, 109);
    /// `paper`: card fills, and text on `action/primary`.
    PAPER = "paper" (248, 244, 232);
    /// `desk`: the warm fill of the Kidnap die's preview world.
    DESK = "desk" (238, 233, 220);
    /// `rule`.
    RULE = "rule" (211, 219, 215);
    /// `map`: the sea the map viewport continues.
    MAP = "map" (216, 231, 227);
    /// `action/primary`.
    PRIMARY = "action/primary" (160, 53, 49);
    /// `faction/government`: Government's identity colour.
    GOVERNMENT = "faction/government" (49, 111, 156);
    /// `faction/26july`: 26July's identity colour.
    M26 = "faction/26july" (167, 46, 43);
    /// `faction/directorio`: the Directorio's identity colour.
    DIRECTORIO = "faction/directorio" (159, 119, 15);
    /// `faction/syndicate`: the Syndicate's identity colour.
    SYNDICATE = "faction/syndicate" (65, 117, 87);
}

#[cfg(test)]
mod tests {
    use super::VARIABLES;
    use serde_json::Value;
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;

    /// Each selected layout's `.figcraft` source and its `Atlas` export, written by
    /// FigCraft's `variable.export`.
    const EXPORTS: &[(&str, &str)] = &[
        (
            "atlas-action-choice.figcraft",
            include_str!("../../art/atlas/atlas-action-choice.variables.json"),
        ),
        (
            "atlas-eligibility.figcraft",
            include_str!("../../art/atlas/atlas-eligibility.variables.json"),
        ),
        (
            "atlas-execution.figcraft",
            include_str!("../../art/atlas/atlas-execution.variables.json"),
        ),
        (
            "atlas-popover.figcraft",
            include_str!("../../art/atlas/atlas-popover.variables.json"),
        ),
        (
            "atlas-top-display.figcraft",
            include_str!("../../art/atlas/atlas-top-display.variables.json"),
        ),
        (
            "atlas-transfer.figcraft",
            include_str!("../../art/atlas/atlas-transfer.variables.json"),
        ),
    ];

    /// The `Atlas` colour variables of one export, by name, in its default mode, as
    /// 8-bit sRGB.
    fn atlas(export: &str) -> BTreeMap<String, [u8; 3]> {
        let export: Value = serde_json::from_str(export).unwrap();
        let meta = &export["meta"];
        let collections = meta["variableCollections"].as_object().unwrap();
        let mut atlas = collections.values().filter(|c| c["name"] == "Atlas");
        let collection = atlas.next().expect("an `Atlas` collection");
        assert!(atlas.next().is_none(), "one `Atlas` collection");
        let (id, mode) = (
            &collection["id"],
            collection["defaultModeId"].as_str().unwrap(),
        );
        let mut colours = BTreeMap::new();
        for variable in meta["variables"].as_object().unwrap().values() {
            if variable["variableCollectionId"] != *id || variable["resolvedType"] != "COLOR" {
                continue;
            }
            let name = variable["name"].as_str().unwrap();
            let value = &variable["valuesByMode"][mode];
            let channel = |key: &str| {
                let channel = value[key].as_f64().unwrap_or_else(|| {
                    panic!("`{name}` is not a plain colour in its default mode: {value}")
                });
                (channel * 255.).round() as u8
            };
            assert_eq!(value["a"].as_f64(), Some(1.), "`{name}` is opaque");
            colours.insert(name.to_owned(), [channel("r"), channel("g"), channel("b")]);
        }
        colours
    }

    /// Each runtime colour that differs from its variable in a layout that defines it,
    /// or that no layout defines.
    fn mismatches(variables: &[(&str, [u8; 3])], exports: &[(&str, &str)]) -> Vec<String> {
        let layouts: Vec<_> = exports
            .iter()
            .map(|(source, export)| (source, atlas(export)))
            .collect();
        let mut found = Vec::new();
        for &(variable, runtime) in variables {
            let mut defined = false;
            for (source, colours) in &layouts {
                if let Some(&value) = colours.get(variable) {
                    defined = true;
                    if value != runtime {
                        found.push(format!(
                            "`{variable}` is {} in {source}, {} at runtime",
                            hex(value),
                            hex(runtime)
                        ));
                    }
                }
            }
            if !defined {
                found.push(format!("`{variable}` is in no selected layout"));
            }
        }
        found
    }

    fn hex([r, g, b]: [u8; 3]) -> String {
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    #[test]
    fn runtime_colours_are_the_layouts_variables() {
        let found = mismatches(VARIABLES, EXPORTS);
        assert!(found.is_empty(), "{}", found.join("\n"));
    }

    /// An export is only a copy: each one must come from the `.figcraft` file that is
    /// committed now. `art/atlas/palette-sources.sha256` records the hash each source
    /// had when it was exported.
    #[test]
    fn exports_are_from_the_committed_layouts() {
        let recorded: BTreeMap<_, _> = include_str!("../../art/atlas/palette-sources.sha256")
            .lines()
            .map(|line| {
                let (hash, path) = line.split_once("  ").unwrap();
                (path.strip_prefix("art/atlas/").unwrap(), hash)
            })
            .collect();
        let sources: Vec<_> = EXPORTS.iter().map(|(source, _)| *source).collect();
        assert_eq!(recorded.keys().copied().collect::<Vec<_>>(), sources);
        for source in sources {
            let bytes = std::fs::read(format!("art/atlas/{source}")).unwrap();
            assert_eq!(
                format!("{:x}", Sha256::digest(bytes)),
                recorded[source],
                "{source} changed since its `Atlas` export; refresh both \
                 (art/atlas/README.md, \"Interface palette\")"
            );
        }
    }

    /// The negative control: #46's wrong ink fails the check.
    #[test]
    fn a_wrong_colour_fails() {
        let mut variables = VARIABLES.to_vec();
        let ink = variables
            .iter_mut()
            .find(|(name, _)| *name == "ink")
            .unwrap();
        ink.1 = [0x1e, 0x32, 0x39];
        let found = mismatches(&variables, EXPORTS);
        assert_eq!(found.len(), EXPORTS.len(), "{found:?}");
        assert!(
            found.iter().all(|line| line.contains("`ink` is #203238")),
            "{found:?}"
        );
    }

    /// The second negative control: one layout's copy of a variable differs from the
    /// others, and the check names that layout.
    #[test]
    fn a_layout_that_disagrees_fails() {
        let (source, export) = EXPORTS[0];
        let changed = export.replacen("0.125490203499794", "0.11764705926179886", 1);
        assert_ne!(
            changed, export,
            "the export still holds `ink`'s red channel"
        );
        let mut exports = EXPORTS.to_vec();
        exports[0] = (source, &changed);
        assert_eq!(
            mismatches(VARIABLES, &exports),
            [format!("`ink` is #1e3238 in {source}, #203238 at runtime")]
        );
    }

    /// A variable no selected layout defines (renamed or deleted in FigCraft) fails.
    #[test]
    fn an_undefined_variable_fails() {
        let mut variables = VARIABLES.to_vec();
        variables.push(("faction/unknown", [0, 0, 0]));
        assert_eq!(
            mismatches(&variables, EXPORTS),
            ["`faction/unknown` is in no selected layout"]
        );
    }
}
