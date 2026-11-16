# Selected victory display components

The later owner-requested [responsive layout](LAYOUT.md) removes region inspection and replaces the static table composite with separated components. The victory behavior and artwork described here remain selected; references below to preserving region inspection describe the earlier layout only.

The owner selected **A, Open emblems**, from `ae667232fe92a932cf59ad80a5ee3490ba9b4718`, with the refinement to align the emblems near the top beside CUBA LIBRE and extend the map upward. [Selection on #16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893152418) owns this direction. Alternatives remain in `art/proposals/issue-16-victory/`; no second selection round is required for the requested positioning.

All components are authored through VectorCraft MCP. The editable sources are `atlas-victory-ui.vectorcraft`, `atlas-victory-reader.vectorcraft` and `atlas-victory-components.vectorcraft`. Their exports are required through `victory.sha256`. The original core runtime manifest and save-bound asset identity remain unchanged; the new stateless presentation is strictly admitted separately. Missing or modified new assets must refuse launch, including when resuming a prior save.

| Export | Authored size | PNG size | Use |
|---|---:|---:|---|
| `atlas-victory-ui.png` | 1920×1280 | 1920×1280 | Existing UI with old score panels removed; expanded transparent map window |
| `atlas-victory-reader.png` | 1920×1280 | 1920×1280 | Reader frame with old score panels removed; title removed in #46 |
| `atlas-victory-icon-{government,26july,directorio,syndicate}.png` | 44×44 | 88×88 | Existing faction marks; no faction-name labels at rest |
| `atlas-victory-tag-{government,26july,directorio,syndicate}.png` | 468×440 | 936×880 | Empty paper tag with faction stripe; height may fit dynamic content |
| `atlas-victory-hover.png` | 104×112 | 208×224 | Hover target emphasis |
| `atlas-victory-rule.png` | 420×1 | 840×2 | Authored internal separator |

The component document's artboards are four icons, four faction tag plates, hover emphasis, then separator. Runtime composes these exports with dynamic model-derived text; no substitute procedural 2D graphics. Existing status leader/anchor images can connect a tag where needed. Standard tag content has 24px horizontal inset. Preserve readable text and complete contribution lists; runtime height/placement may adapt to contents and available screen space.

At the 1920×1280 reference size, icons center at x780,900,1020,1140 and y40; margins sit directly below, around y66. The centered group shares the title's top band. The map outer frame starts at (40,136), its header is40px high, and the transparent viewport is (40,176,1300,684), ending at y860. The old viewport was (40,320,1300,540); the new frame adds144px (26.7%) vertical map space. The bottom map legend, region inspection and existing operation/card/session areas remain at their established locations. Runtime camera framing must use the extra space without clipping the island or regressing the accepted regional hover tags.

Only icon and official signed margin are persistent victory information. One hover tag contains faction identity, calculation, contributions and exact conditions. Zero is not a win; Government's all-cities condition remains independent of its official margin. Available pieces remain in a quiet footer; since #46, Resources are on the top display's ledger ([TOP.md](TOP.md)), and only Syndicate's Resources stay in the tag, as its victory condition. Hovering/clicking standings must not replace or clear region inspection. Reader capture suppresses the tags. The accepted source and numerical examples are linked in the proposal README.

VectorCraft CLI0.24.1 authored/exported these files. Existing Lato and DejaVu Serif typography and provenance are retained. Native sources reopen without missing links; exports and enlarged transparent aperture were inspected. This is component evidence, not a claim of working native interaction, final layout fit or owner acceptance of the new implementation.
