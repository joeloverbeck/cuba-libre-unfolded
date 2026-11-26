# Casinos

The owner chose **C · Separate actions** and **A · Stacked coins** on 2026-11-26 for [spec #215](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/215). The same solid green Casino disc serves both map states. Only the open face carries gold. Closed Casinos and the Available Forces ledger use plain green. This follows the two faces in R 1.4.4. The written state and quantity remain visible in the piece popover.

## Layout

[The editable action layouts](atlas-casino-actions.figcraft) contain the C frame and the two alternatives from the owner's selection round. [The gold review](atlas-casino-gold-review.figcraft) shows C with the selected VectorCraft graphic and the existing mesh at Home-camera scale. The [layout bounds](atlas-casino-actions.layout.json) come from C's anchor (node 57), piece card (58), Close and Return buttons (68 and 72), at 1.6875 display pixels per reference unit. The 3840×2160 native comparison uses a 2-pixel tolerance for the button rectangles and the anchor's height relative to the card. The card follows the selected group's map position and stays inside the map. The Atlas focus mark follows each reachable action.

C shows the shared Control and open count above the two individual outcomes and buttons. A stepper above the preview adds room when the selected group has more than one piece. Additional local consequences or refusal reasons extend the appropriate part of the card. A closed group has a single return control. The renderer uses the existing Atlas card, rules, stepper, commit and outline plates.

Both FigCraft documents retain the inspected [redacted table reference](../proposals/issue-200/ctx-card18-government.jpg) with SHA-256 `53f83e3c6628036023350d9ed7500eb5012df4327f6994b48733d4a980bc2eae`. The review's four stage images crop Pinar del Río and its Casino. Both sources are byte-for-byte copies of the selected local originals, including the redacted context. Their SHA-256 values are `c092edf926caf6e7479024c10498abb6b8faeb9bac39741f9ab6e13f89e3051f` (action layouts) and `d0d27c5ef57052e0c8c86e2f880a981a7336b928e655809c5311e7a0b629593a` (gold review).

## Graphics and admission

[The editable VectorCraft original](atlas-casino-faces.vectorcraft), SHA-256 `15247882e5782141f73f7ac88e66e8528430a5587a3a41180fba3af8e20cf8d3`, stores all three marks, the three open-piece icons and the plain piece. The runtime consumes the selected design's exports byte for byte.

| Runtime file | Original artboard | PNG | Launch manifest |
|---|---|---|---|
| `atlas-casino-gold.png` | A - Stacked coins | 256×256 | `runtime.sha256` |
| `atlas-casino-open.png` | A - Open Casino | 300×183 | `runtime.sha256` |
| `atlas-casino-plain.png` | Closed or in supply | 300×183 | `available-forces.sha256` |

The mark uses a transparent plane 0.21 wide atop the existing 0.30 diameter disc. The popover places either icon in C's 36×22 reference rectangle. The Available Forces ledger places the plain icon in [its established cell](AVAILABLE.md).
