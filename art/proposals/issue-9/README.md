# Three visual proposals for #9

These are **historical VectorCraft design studies**, supporting [issue #9](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/9) and the complete task in [#8](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8). The owner [selected A, Atlas desk](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/9#issuecomment-6887299489), at proposal `6bbd803e36bdd1052b2df1aa6d3f9427dceffe7d` / integration `a9ee0c89d7070d578619f5bd9a3e0b9070a96ffa`, with solid 3D regions/pieces and shared numeric labels on representative pieces. The single combined visual-selection round is consumed. B/C remain unselected; the studies are not running application screenshots. See [the selected runtime handoff](../../atlas/README.md). Native readability and owner play acceptance remain due.

**Recommendation: A, Atlas desk**, for the clearest balance between geographic context and the current choice. B is the strongest alternative when a broad map and compact command area matter more. C offers the strongest period-print character and larger written explanations, at the cost of a smaller map. The choice covers each bundle's layout, palette, typography, inspector and card-reader treatment; none is silently adopted. Shared faction colors and physical piece types preserve the source distinctions.

| Proposal | Main view | Consequences and inspectors | Trade-off |
|---|---|---|---|
| A · Atlas desk | [Main preview](a-atlas.png) | [State studies](a-atlas-states.png) | Pale atlas, right decision column, separate contributions; balanced density |
| B · Operations table | [Main preview](b-operations.png) | [State studies](b-operations-states.png) | Dark field, widest island, horizontal commands; compact reading and stronger abstraction |
| C · Field journal | [Main preview](c-journal.png) | [State studies](c-journal-states.png) | Warm paper and serif hierarchy; generous explanations, smallest geography |

Each main view and state sheet has a same-stem editable `.vectorcraft` source and outlined-text `.svg` export. Each `*-reader.vectorcraft` / `.svg` contains the authored card reader **without imported faces**. PNGs are 1920 pixels wide; main views are 1280 high, state studies 1640. Inspect at full size. State sheets collect independent examples, not simultaneous match states or a proposed screen containing every inspector at once.

## Source meaning and geographic comparison

The scenario and arithmetic are the accepted handoff at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`. The March 2019 Living Rules SHA-256 is `8152cb0056e3690fdc01c7b4ff893369ad7eff288941abc540ceee2c9d606792`; third-printing playbook is `dcaa5afd6f01c0c9ed4746c18199c1b09058823d67b627384fd3184ed4015fe8`. R p.2's map depiction, R 1.3, P p.36 and the supplied board reference were used to inspect geography. The reference board is not imported into these designs or treated as the controlling rules edition.

All alternatives retain seven provinces in west-to-east adjacency order: Pinar del Río—La Habana—Matanzas—Las Villas—Camagüey Province—Oriente—Sierra Maestra. Havana meets La Habana; Camagüey City meets Camagüey Province; Santiago meets Sierra Maestra. The three distinct ECs contact Pinar/La Habana (Econ 3), Las Villas/Camagüey Province (3), and Oriente/Sierra Maestra (2). The authored coastal gap deliberately prevents a Camagüey Province/Sierra Maestra point contact. No other city, province or EC contact is introduced. Decorative towns and storage tracks are omitted. This is a visual comparison, not movement-engine verification.

The main studies show the prepared 29 pieces. Text and inspector studies distinguish terrain, population, support, control, the separate similarly named spaces, and the 3+3+2 economy. The piece legend proposes solid cubes, cylinders and discs, with unmarked/star Guerrilla states and open/slashed Casino states. These vector silhouettes communicate the intended pieces; they do not prove native three-dimensional appearance or picking. All map boundaries, panels and silhouettes were authored through the Cuba Libre VectorCraft MCP.

## Card custody and actual viewing evidence

Only the two accepted local faces were copied into ignored `local/issue-9-review/` for the reader probe. Original files were unchanged. Identities:

- `card-17.png`: `141f592b5dee5a8c7a6d26651410b0ee1171a7dcf2c2b5331a63b48407cec9ab`.
- `card-18.png`: `50df7058e660bb4dc620096823945eeb095fe4123e0aead68fec6244ab097c4e`.

VectorCraft placed the originals proportionally at approximately 420×602 pixels per face, below all four standings. Local-only `a-atlas-reader.png`, `b-operations-reader.png`, and `c-journal-reader.png` in that ignored directory contain the actual composites. Their edges, titles, icons and text were inspected at the intended export size. The original rounded corners contain black corner pixels, preserved without cropping or repainting. No imported image, private absolute path or required external link is present in the committed native sources or SVGs; their image maps are empty. These local probes establish the proposed reader's fit, not native rendering or owner readability acceptance.

## Authoring, verification and remaining boundary

Authoring used installed VectorCraft 0.24.1 through `vectorcraft-cuba-libre`. The inspected editor checkout was `9ddbf25bef7711c3404549d95b730f7dec3210e7`; this identifies the inspected source, not a verified build provenance for the installed binary. Lato and DejaVu Serif were available locally; DejaVu's explicit Book style avoids fallback. Font notices are retained in [FONT-NOTICES.txt](FONT-NOTICES.txt). Native documents retain editable text; exported SVGs outline it for portable viewing.

All nine native sources reopened. All three main SVGs reopened without missing links or warnings and were rendered and inspected; actual PNGs and state studies were also inspected. The source scenario supplies the arithmetic independently. No game, renderer, input, save/recovery or functional interaction check occurred in this art task. The displayed recovery/refusal messages are design proposals for the specified behavior, not claims that a save has succeeded.

The four required sibling inspections found existing rendering/solid-token and hover mechanisms, not a Cuba map or accepted Cuba interface: Arkham `8301466b5bbc20b99411901cfcd6db2b29865431` (`src/tabletop_pieces.rs`); mudsong `b6c05ac561bf52f4dabb7d77fa313889a768b628` (`crates/app/src/pointing.rs`); Netrunner `3a64b7138f906b5e79ca135e463b68f85415bde6` (`src/tabletop.rs`); unfolded-tabletop `ed0c8866b11c6af5378dc8bffdc8dc98ce45c40b` (`src/token.rs`). Their engineering mechanisms stay with their owners; no code, private content or game-specific art was copied.

The selected handoff separates static authored geometry from these example values; runtime text, counts, costs, ownership and consequences remain dynamic. #12 must exercise those actual exports in the native consumer and retain the specified native/owner checks. The selection does not authorize a default-branch merge, asset redistribution, another design round or broader game scope. These historical studies remain comparison evidence, not runtime state.
