# Selected victory display components

The later owner-requested [responsive layout](LAYOUT.md) removes region inspection and replaces the static table composite with separated components. The victory behavior and artwork described here remain selected; references below to preserving region inspection describe the earlier layout only.

The owner selected **A, Open emblems**, from `ae667232fe92a932cf59ad80a5ee3490ba9b4718`, with the refinement to align the emblems near the top beside CUBA LIBRE and extend the map upward. [Selection on #16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893152418) owns this direction. Alternatives remain in `art/proposals/issue-16-victory/`; no second selection round is required for the requested positioning.

All components are authored through VectorCraft MCP. The editable sources are `atlas-victory-ui.vectorcraft` and `atlas-victory-components.vectorcraft`. The reader frame's source and export, `atlas-victory-reader.vectorcraft` and `.png`, were retired with the reader in #139; Git history keeps them. The exports below are required through `victory.sha256`. The original core runtime manifest remains unchanged; the new stateless presentation is strictly admitted separately, at launch ([launch admission](README.md#launch-admission)). Missing or modified new assets must refuse launch, including when resuming a prior save.

| Export | Authored size | PNG size | Use |
|---|---:|---:|---|
| `atlas-victory-ui.png` | 1920×1280 | 1920×1280 | Existing UI with old score panels removed; expanded transparent map window |
| `atlas-victory-icon-{government,26july,directorio,syndicate}.png` | 44×44 | 88×88 | Existing faction marks; no faction-name labels at rest |
| `atlas-victory-tag-{government,26july,directorio,syndicate}.png` | 468×440 | 936×880 | Empty paper tag with faction stripe; height may fit dynamic content |
| `atlas-victory-hover.png` | 104×112 | 208×224 | Hover target emphasis |
| `atlas-victory-rule.png` | 420×1 | 840×2 | Authored internal separator |

The component document's artboards are four icons, four faction tag plates, hover emphasis, then separator. Runtime composes these exports with dynamic model-derived text; no substitute procedural 2D graphics. Existing status leader/anchor images can connect a tag where needed. Standard tag content has 24px horizontal inset. Preserve readable text and complete contribution lists; runtime height/placement may adapt to contents and available screen space.

At the 1920×1280 reference size, icons center at x780,900,1020,1140 and y40; margins sit directly below, around y66. The centered group lies over the top of the map, which has filled the window since #142 ([MAIN-VIEW.md](MAIN-VIEW.md)); the framed viewport this refinement first enlarged, with its header and bottom legend, is retired. Runtime camera framing must use the extra space without clipping the island or regressing the accepted regional hover tags.

Only icon and official signed margin are persistent victory information. One hover tag contains faction identity, calculation, contributions and exact conditions. Zero is not a win; Government's all-cities condition remains independent of its official margin. Since #46, Resources are on the top display's ledger ([TOP.md](TOP.md)), and only Syndicate's Resources stay in the tag, as its victory condition. Since #161 the tag has no footer: Available Forces are on their own ledger ([AVAILABLE.md](AVAILABLE.md)), and the tag ends 32 px below its last condition line. Hovering/clicking standings must not replace or clear region inspection. The accepted source and numerical examples are linked in the proposal README.

VectorCraft CLI0.24.1 authored/exported these files. Existing Lato and DejaVu Serif typography and provenance are retained. Native sources reopen without missing links; exports and enlarged transparent aperture were inspected. This is component evidence, not a claim of working native interaction, final layout fit or owner acceptance of the new implementation.

## Inline text emblems

#60 reuses the four `atlas-victory-icon-*.png` exports inside running text without a new design round, as #58 allows. They are drawn 0.9× the type size and sit on the line's baseline. `victory.sha256` already requires these exports, so a missing or changed emblem refuses at launch. No separate text-height export exists.

An ideal 18 px raster, approximated by an area resample, leaves the letters as illegible 4–5 px smudges. The letter is about 30% of the emblem's height, so a smaller export would not help.

**Legibility by size:**

| Size | Emblem | Letters | Legible |
|---|---:|---:|---|
| Default 1920×1280 | 15–17 px | about 5 px | No. Factions are told apart by shape and colour. |
| 4096×2160 | 25–29 px | about 9 px | Yes |
| Owner's display, 3840×2160 | 25–29 px: the 4096×2160 scale, as both are 2160 tall | about 9 px | Yes; the owner's judgement was #16's |

**26July on red surfaces:** the 26July star is the same red as the primary button, so 26July keeps its name on those buttons. This is a reversible local choice that #68's round puts to the owner.

The evidence is under `local/verification/issue-60/`.
