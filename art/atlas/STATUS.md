# Hover status components

The later owner-requested [responsive layout](LAYOUT.md) removes the region inspection panel entirely. These hover tags remain the region-status disclosure; the inspection deduplication described below records the earlier increment.

Selected C+ coastal tags with the five-position opinion scale, following [#16 owner feedback](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6892578260). The owner accepted the combined preview at `a7867b250e3ca1e60e1752e8609f349638f12418`, with two refinements: show tags on hover, and keep Havana's tag outside its city with a visible leader. A subsequent instruction removes duplicated current Control and Support/Opposition text from region inspection, while other occurrences are audited for their distinct purpose. The source and original alternatives remain in `art/proposals/issue-16-status/`.

`atlas-status.vectorcraft` is the editable VectorCraft source for these selected components. Artboards 1–5 are Control rows: Government, 26July, Directorio, Syndicate and Uncontrolled. Artboards 6–10 are political rows: Active Opposition, Passive Opposition, Neutral, Passive Support and Active Support. Artboards 11–12 are the leader line and its region anchor. Original faction marks and Lato typography reuse the selected study; no original-game image is embedded.

| Runtime export | Authored size | PNG size | Composition |
|---|---:|---:|---|
| `atlas-control-{government,26july,directorio,syndicate,uncontrolled}.png` | 272×36 | 544×72 | Full identity and original faction mark |
| `atlas-opinion-{active-opposition,passive-opposition,neutral,passive-support,active-support}.png` | 272×88 | 544×176 | Exact political state, five cells and highlighted current position |
| `atlas-status-leader.png` | 8×2 | 16×4 | Stretch/rotate this authored line to connect the tag to its region |
| `atlas-status-anchor.png` | 8×8 | 16×16 | Place at the region end of the leader |

The paired tag is 272×128 at authored scale: Control at y0, a4px gap, political row at y40. In the political row, the state baseline is y24 and the scale occupies y37–49. The contribution is dynamic Lato text with left x12, top approximately y60, baseline y76 and maximum width248. For example Matanzas shows `×1 population · 26July +1`; Neutral shows `0 Support/Opposition`. The dynamic value describes the population term only, not Base, Casino or DR-Control contributions. It must come from the actual model rather than the studies. Runtime may adjust uniform display size for native readability, preserving proportions and separate text/scale rows.

Only one hovered eligible city/province has this disclosure. ECs have neither state. The shared hover lifecycle supplies dwell/switch/dismissal; Cuba owns selection and label geometry. Place tags in free map viewport space outside the hovered region and its pieces, preserve a visible leader gap (especially Havana), and keep both tag and line noninteractive so they do not steal hover or clicking. These placement requirements must be checked in the real renderer at home and navigated camera positions. No persistent status indicators or additional overview overlay were selected.

`status.sha256` binds the twelve PNGs separately from `runtime.sha256`. They are required presentation assets; load/decode/integrity failures must still refuse clearly. Their addition does not change the existing save-bound core-map/card identity or the rules source, so earlier valid saves retain compatibility without a permissive hash allowlist. This separation is specific to these new stateless disclosures and does not authorize unverified changes to the save contract.

Authored and exported through VectorCraft CLI0.24.1, 2px per point, with all command batches successful. Current editor checkout identity is `9ddbf25bef7711c3404549d95b730f7dec3210e7`, not independent installed-binary build provenance. Rendered Control and all five political rows were inspected; the existing Lato notice applies. Asset exports establish the component appearance only. Native hover, dynamic values, placement, no-overlap, inspection deduplication and old-save evidence remain the implementation consumer's responsibility under #16. No complete-task owner acceptance or default publication is implied.
