# Region status studies

Decision-only VectorCraft proposals for the [owner-reported readability gap in #16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6892328490), under [#8](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8). These files are not runtime inputs. The owner selects the design under [foundation 4](../../../docs/foundation/04-art-and-assets.md); selection and implementation status belong to the issue. Existing runtime art, rules, saves, camera and pieces are unchanged by this proposal.

## Physical-game evidence and meaning

The [March 2019 Living Rules](https://s3-us-west-2.amazonaws.com/gmtwebsiteassets/cuba/CL-RULES-2019-LivingRules.pdf), sections 1.6.1–1.6.3 and 1.7, distinguish political opinion from Control. The inspected physical board has paired boxes printed Uncontrolled and Neutral. Separate faction-control counters and Active/Passive Support/Opposition counters cover them; absence of a political counter means Neutral. Political counters show ×1 or ×2. The original controller images use a Cuban flag, M-26-7 flag, DR flag and coins. Those images and the board scan were inspected as external references and are not embedded or shipped here. The proposed faction marks are original shapes/initials paired with full names, not imported flags or portraits.

The accepted prepared position supplies the witnesses. Empty Matanzas is Uncontrolled, but its population 1 at Passive Opposition contributes 1 to 26July's Opposition. Havana's 10 Government pieces exceed its 3 opposing pieces, giving Government Control; its population 6 at Active Support contributes 12 Support. La Habana and Pinar show Syndicate Control with Government Support. Camagüey Province shows Directorio Control with Opposition. These combinations must remain intelligible.

All five political levels and all four controllers plus Uncontrolled are shown. Neutral means zero **Support/Opposition** contribution, not zero contribution to every faction's standing. Opinion multipliers apply to population only; Bases, DR-controlled population and Casinos remain separate terms. ECs have neither Control nor political opinion slots. Native values must come from the rules model, never these example labels.

## Alternatives

| Study | Placement | Benefit | Cost |
|---|---|---|---|
| [A: paired inset plaques](a-plaques.png) | Stacked Control and political-status plaques on the land | Full words and contribution are close to the pieces | More region surface needed; small/narrow provinces are the hard case |
| [B: seal and scale](b-scale.png) | Control seal beside a five-position opinion scale | Direction/intensity is visible as well as the written state | Wider footprint and a scale to learn |
| [C: coastal tags](c-coastal.png) | Two tags beside the region, joined by a leader | Preserves the land for pieces and geographic labels | More visual connections; collision handling while navigating matters |

C is the recommendation. [Its whole-island placement study](c-whole-island.png) exercises all ten eligible spaces, the three excluded ECs, independent Control/opinion combinations and the actual prepared piece-group counts. The coastal pair is stacked there to reduce its width; the enlarged regional sheet also shows a side-by-side arrangement. These are the same two slots, with layout adapted to available space.

The large sheets are inspection studies, not screenshots or proof of native fit. The whole-island study has a 1730×940 map area; the current native viewport is 1300×540. Shrinking this sheet into that viewport would reduce 17 px text to roughly 10 px when height-constrained and is not an acceptable implementation. Native layout must reserve readable screen space, use suitable camera framing and verify all tags with the actual pieces. Uniformly scaling land and then zooming out to fit gains no screen area. Relative map/piece scale, camera pitch and available viewport area are separate decisions; preserve the recent coast/profile correction and shared numeric piece labels. A full-map native view may require more screen area, with controls remaining accessible. This study does not select an unrelated interface redesign.

## Source and inspection

`status-studies.vectorcraft` contains four artboards in the order A, B, C, C overview. PNG exports are 1 pixel per authored point. All graphics and text were authored through the Cuba Libre VectorCraft MCP. The Matanzas and whole-island profiles reuse the existing VectorCraft-authored `art/atlas/atlas-map-profiles.svg`; there is no new coastline derivation. The drawn piece symbols illustrate the existing grouped solids and quantities; runtime pieces remain genuinely 3D.

VectorCraft CLI 0.24.1 was used. The editor checkout inspected was `9ddbf25bef7711c3404549d95b730f7dec3210e7`; this does not independently prove the installed binary's build provenance. The native source saved and reopened with no warnings or missing/modified links. All four studies were rendered and inspected. Re-exporting the overview after reopen produced the identical PNG SHA-256 `e41fced6a5e512ea662973d0c6803cf6dfe18436b206fe37c1c9d65383f0895a`. The font is the existing Atlas Lato family; its notice remains in `art/atlas/FONT-NOTICE.txt`.

Source rules and runtime baseline remain `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab` and `bc4495a49ea6fd16cce6a246c29fb9e08cbc690f`. The consulted constitution rules, sibling inspections, bounded appetite and owner-feedback classification are recorded in #16. This evidence establishes authored alternatives and source-backed meaning, not working native markers, camera collision behavior, visual adoption or owner acceptance of the tactical task.
