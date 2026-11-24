# Hover status components

The later owner-requested [responsive layout](LAYOUT.md) removes the region inspection panel entirely. These hover tags remain the region-status disclosure; the inspection deduplication described below records the earlier increment.

Selected C+ coastal tags with the five-position opinion scale, following [#16 owner feedback](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6892578260). The owner accepted the combined preview at `a7867b250e3ca1e60e1752e8609f349638f12418`, with two refinements: show tags on hover, and keep Havana's tag outside its city with a visible leader. A subsequent instruction removes duplicated current Control and Support/Opposition text from region inspection, while other occurrences are audited for their distinct purpose. The source and original alternatives remain in `art/proposals/issue-16-status/`.

`atlas-status.vectorcraft` is the editable VectorCraft source for these selected components. Artboards 1–5 are Control rows: Government, 26July, Directorio, Syndicate and Uncontrolled. Artboards 6–10 are political rows: Active Opposition, Passive Opposition, Neutral, Passive Support and Active Support. Artboards 11–12 are the leader line and its region anchor. Artboards 13–17 are the terrain rows added by [#138](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/138): City, Province · Forest, Province · Grassland, Province · Mountain and Economic center. Artboard 18, "Reason row", is the plate of the open decision's line added by [#170](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/170). Original faction marks and Lato typography reuse the selected study; no original-game image is embedded.

| Runtime export | Authored size | PNG size | Composition |
|---|---:|---:|---|
| `atlas-control-{government,26july,directorio,syndicate,uncontrolled}.png` | 272×36 | 544×72 | Full identity and original faction mark |
| `atlas-opinion-{active-opposition,passive-opposition,neutral,passive-support,active-support}.png` | 272×88 | 544×176 | Exact political state, five cells and highlighted current position |
| `atlas-space-{city,forest,grassland,mountain,economic-center}.png` | 272×36 | 544×72 | The space type and a Province's terrain, with a mark drawn from the map |
| `atlas-status-leader.png` | 8×2 | 16×4 | Stretch/rotate this authored line to connect the tag to its region |
| `atlas-status-anchor.png` | 8×8 | 16×16 | Place at the region end of the leader |
| `atlas-status-reason.png` | 272×36 | 544×72 | The Control row's frame with no mark or label: the plate of the open decision's line (#170) |

A city's or province's tag is 272×168 at authored scale: the terrain row at y0, Control at y40 and the political row at y80, each 4px below the row above. In the political row, the state baseline is y24 and the scale occupies y37–49. The contribution is dynamic Lato text with left x12, top approximately y60 within the political row (y140 in the tag), baseline y76 and maximum width248. For example Matanzas shows `×1 population · 26July +1`; Neutral shows `0 Support/Opposition`. The dynamic value describes the population term only, not Base, Casino or DR-Control contributions. It must come from the actual model rather than the studies. Runtime may adjust uniform display size for native readability, preserving proportions and separate text/scale rows.

Only one hovered space has this disclosure. ECs have neither state, so an EC's tag is its terrain row alone, 272×36. The shared hover lifecycle supplies dwell/switch/dismissal; Cuba owns selection and label geometry. Place tags on free map outside the hovered region, its pieces and every overlaid area ([MAIN-VIEW.md](MAIN-VIEW.md)), preserve a visible leader gap (especially Havana), and keep both tag and line noninteractive so they do not steal hover or clicking. These placement requirements must be checked in the real renderer at home and navigated camera positions. No persistent status indicators or additional overview overlay were selected.

`status.sha256` binds the eighteen PNGs separately from `runtime.sha256`. They are required presentation assets; load/decode/integrity failures must still refuse clearly. Like every manifest, `status.sha256` is admitted at launch. For saves, see [launch admission](README.md#launch-admission).

Authored and exported through VectorCraft CLI0.24.1, 2px per point, with all command batches successful. Current editor checkout identity is `9ddbf25bef7711c3404549d95b730f7dec3210e7`, not independent installed-binary build provenance. Rendered Control and all five political rows were inspected; the existing Lato notice applies. Asset exports establish the component appearance only. Native hover, dynamic values, placement, no-overlap, inspection deduplication and old-save evidence remain the implementation consumer's responsibility under #16. No complete-task owner acceptance or default publication is implied.

## Terrain row (#138)

The terrain key in the map frame explained the colours and glyphs; [#137](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/137) removed it with the frame (#142), and the decision record gives the space tag a terrain row in its place. The row names the space as the rules' map key does (R 1.3): a City, a Province with its terrain, or an Economic center, in the words of the space popover. The model's `Space::terrain` supplies it; the playbook's Spaces List (third printing, p.36) agrees for all thirteen spaces.

Each row reuses the Control row's frame, typography and 28-point mark position, copied from the Uncontrolled artboard. Its mark repeats what the map shows for that space type, so the row also serves as the key: a 28-point rounded square in the province tile's fill (#91b59a, #d9d9a1, #c7b19a) carrying one of that tile's own engraved motifs, copied from `atlas-terrain-textures.vectorcraft` at 74 % (grass), 55 % (tree) and 36 % (mountain) with a 1.8-point stroke; the city disc's fill and rim (#efe2bc, #04080a); and the Economic Center's gold disc (#b17f21, #04080a). Those colours are the materials of `atlas-terrain-map.glb`. The terrain row sits first, because it says what the space is before the rows that say who holds it and how it leans.

An Economic Center's map label is only its gold value disc, which looks like a Directorio piece's count without the key, so each EC now has a tag. It has the terrain row alone: Control and Support/Opposition do not apply to ECs.

Authored and exported with VectorCraft CLI 0.24.1 at 2 px per point. Native placement, legibility at 3840×2160 and the seat's `table.status.terrain` are checked by `native_hover_status_is_transient_without_region_inspection`; the owner's judgement at review remains separate.

## The open decision's line (#170)

While a decision offers choices on the map's spaces (Terror and Assault in this build), the tag gains one last line that says why the space is in its state for that decision. The words are the rules authority's (`observation.space_reasons`); the tag builds none from a label or from the pieces it sees. A space where a supported choice acts and nothing is owed has no line, and outside such a decision, or while the targeting is suspended (a presented die, a Resource transfer under way, removal mode, Kidnap open), every tag is as it was.

The line is the contribution's text style: Lato Regular at 17, the Atlas desk text colour, left x12 and at most 248 wide. It sits on its own row, 4 px below the rows above: at y172 in a city's or province's tag, which becomes 272×208, and at y40 in an Economic center's, which becomes 272×76. The row is `atlas-status-reason.png`, the Control row's frame with no mark or label, copied from the Uncontrolled artboard; the text's top is 8 px into the row, which centres one line. Faction names in the line are drawn as their emblems, as in the contribution. The tag is placed with its new height by the same rules, so it keeps off the hovered region, its pieces and every area.

The lines, each shortened from the authority's explanation to fit one line:

| Decision | State | Line | Rule |
|---|---|---|---|
| Assault | blocked | A Resource transfer opens it | R 6.3.1, 1.5.1 |
| Assault | not involved: plans that would remove or close a piece, and nothing can pay | Government cannot pay 2 Resources | R 6.3.1 |
| Assault | no effect | No enemy pieces (R 3.2.4) · No Troops or Police (R 3.2.4) (City, EC) · No Troops (R 3.2.4) · Police count only in Cities, ECs · 1 Troop: Mountain needs 2 · No Active Guerrilla (R 3.2.4) · No Active Guerrilla · Bases last · Casinos stay closed (R 1.4.5) | R 3.2.4, 1.4.5 |
| Assault | paid in this Operation | Assaulted · paid 2 Resources | R 3.1 |
| Terror | owed after Kidnap | Terror is owed here (R 4.3.3) | R 4.3.3 |
| Terror | resolved | Terror here paid 1 Resource | R 3.3.4 |
| Terror | blocked | A Resource transfer opens it (Kidnap here first opens it, Kidnap or a transfer opens it, where Kidnap is the remedy) | R 3.3.4, 4.3.3 |
| Terror | not involved | No Underground 26July Guerrilla; or, with one there but nothing to pay for it, 26July cannot pay 1 Resource, or 1 Resource is kept for Santiago | R 3.3.4; accepted commitment policy |

Measured in Lato Regular at 17 with each emblem 0.9 em wide, the longest line takes 232 of the 248 pixels. The native regression `native_space_tags_say_why_each_space_is_in_its_state` checks on the owner's display that each line it reaches is laid out inside its row.

Authored with VectorCraft CLI 0.24.1 at 2 px per point: artboard 18 and its frame, a copy of the Uncontrolled row's frame (object 18). No other artboard changed.
