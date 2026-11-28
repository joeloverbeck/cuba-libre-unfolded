# Engraved Atlas terrain and population

The owner selected A Engraved Atlas and its person-and-number badges in [#16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16#issuecomment-6893736954), from the three VectorCraft studies at452ae46. Terrain is now authored surface artwork, not terrain words. Region names remain, population badges distinguish1/2/6 from piece quantities, and economic centers retain their separate gold value markers.

## Editable sources and exports

`atlas-terrain-textures.vectorcraft` has three256×256 artboards: grassland, forest, mountain. Export at2pixels/point to the matching `atlas-terrain-{grassland,forest,mountain}.png`. Each seamless tile retains the accepted profile fill (#d9d9a1/#91b59a/#c7b19a) and six staggered original engraved motifs. VectorCraftCLI0.24.1; editor checkout9ddbf25bef7711c3404549d95b730f7dec3210e7, not independently verified binary provenance. No imported texture images.

Before the selected enlargement, the terrain mesh derived from the accepted `atlas-map.blend`. Blender5.2.3 applied planarUVs `(world.x/1.15,world.y/1.15)` to the seven province meshes. Only faces with upward normalz>0.99 receive a new textured material; side and bevel materials, vertex coordinates, region names, cities and ECs are preserved. Forest: Pinar del Río, Camagüey Province, Oriente; grassland: La Habana, Matanzas; mountain: Las Villas, Sierra Maestra. The exact same vertex coordinates were checked before saving. Textures are packed in the editable source with portable sibling image paths. Export the active Cuba_Atlas_Regions scene only, selection of all13regions, apply modifiers, GLB, no cameras/lights. `atlas-terrain-map.glb` contains one scene,13nodes and three embedded PNGs, no external buffers. No live Blender document or original source was overwritten.

`atlas-terrain-labels.vectorcraft` derives from accepted Atlas labels. Terrain/city words and bare population line are removed for all ten cities/provinces. Original name lettering remains; beneath it the selected ivory capsule contains a person and numeral. Badge geometry is55% of the study's84×44 component. Artboard0 is the full map-coordinate arrangement; artboards1–13 follow `terrain-labels.json`, exported at3pixels/point to `atlas-terrain-label-{space}.png`. Labels retain previous width and piece placement (and x/y except the two western corrections below), with city/province height52for the badge; EC bounds/values remain unchanged. Source reopens without missing links/warnings. Existing font license applies.

## Runtime and verification boundary

Dense terrain is cleared immediately around province-name glyphs using a VectorCraft-authored four-point stroke behind the original lettering, in that terrain's own fill color. Province label crops gain three points of transparent padding on each side to retain this clearing. This does not move the names or population badges, change piece positions, or place an opaque rectangular plaque on the terrain.

`terrain.sha256` requires the self-contained GLB, label metadata, all 13 label exports and the Inventory roundel. Terrain is stateless presentation, admitted at launch ([launch admission](README.md#launch-admission)). Runtime loads the admitted map scene and its labels through the existing Bevy material and plane paths.

The source/export checks establish custody, coordinate preservation and material mapping. Native inspection found the western Casino pieces partly covering the new badges; Pinar del Río and La Habana labels were moved12source units upward, preserving piece positions. Other labels retain their previous coordinates. On owner feedback that the terrain coverage was too sparse, each tile was increased from two to six staggered motifs, preserving individual motif size and the1.15world-unit repeat. The denser PNGs were repacked in Blender and re-exported in the self-contained GLB. Final native captures/tests and their limits belong to #16's combined implementation handoff; authoring checks alone do not establish readability or owner acceptance. Renderer tests use isolated saves. Selected design does not authorize a main-branch merge or full-game claim.

## Reduced Cities and Inventory roundels

The terrain uses the 72% Reduced Cities profiles selected in [#268](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/268#issuecomment-7227294305). The portable export `atlas-terrain-map.glb` was produced in Blender 5.3.0 (`02f0899d78e3`) and hashes to `609436634fd155642a9f3902ecc5a28c8b85e1736dee8847900b23f6f2c72ff7`.

The [accepted circular placements](terrain-placements.json) use a **2.025×** planar scale and **0.84×** solids. The widest pair, a solid and its right-hand Inventory roundel including clearance for a Cash carrier, fits in a 0.4396-world-unit circle. The thirteen placements in each land space cover the usable surface around the name and population. At most two Base factions can share a space, so the three eligible factions share two positions. The eastern Oriente wedge beside the Sugar Cane guide is clear. In the fit search, Camagüey city had twelve placements at 2.000× and thirteen at 2.025×.

## Brass Economic Center details

Each value is centered on the original gold disc. A brass trace connects it to a world-fixed inset holding that center's pieces. Cigars and Sugar Cane are north of the island. Factory is south. The 2.04×1.91-world-unit insets each fit eight force groups and one marker place on a staggered grid. Cash shares the carrier's footprint. Camera motion preserves the relationship between the disc, trace, inset and its pieces.

[`economic-center-lettering.vectorcraft`](economic-center-lettering.vectorcraft) holds the artwork for the three titles. Runtime title exports `chart-{cigars,factory,sugar-cane}.png` are admitted through [`terrain.sha256`](terrain.sha256). The [inset geometry](../../src/tabletop/offshore.rs) and [land placement](../../src/tabletop/map.rs) use the same world coordinates as the authored terrain. The geography, Economic Center identities, values, adjacency and rules are preserved.
