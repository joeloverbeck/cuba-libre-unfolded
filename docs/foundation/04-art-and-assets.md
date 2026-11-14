# Art and asset custody

This foundation specializes `PURP-2`, `EVID-3`, `REP-1`, `INT-4`, and `OPS-7` around the owner's chosen authoring route.

## Authoring boundary

All **2D visual designs** are authored in **VectorCraft**: map, interface, marker/token artwork, controls, card backs, proclamation frames and even simple visible geometric decorations. Rust loads, composes and renders authored exports and implements interaction. It must not create substitute 2D visual designs using procedural drawing, handwritten SVG paths or generated textures. Runtime values and user-written proclamation text remain dynamic content placed into authored designs; they are not required to become separately drawn images. The owner's subsequent 3D clarification below governs meshes.

Use **EffectCraft** when authored motion/effects are needed. Keep its native source and exported result linked to the VectorCraft source. Runtime placement and movement of game objects implement state changes; they do not authorize an independently invented animation or graphics system. **PhotoCraft** supports inspection and permitted preparation of imported card faces and explicitly accepted raster exceptions; it is not an alternative route for designing the vector interface.

Imported original images are limited to the supplied card faces unless the owner accepts a specifically enumerated exception. The shared card back, board scan, reference charts, token atlases and piece textures are references for authored replacements. An exception names the particular asset and why it is needed. There are no extra raster exceptions approved at founding.

## Map and physical pieces

Recreate the regions and cities in VectorCraft. Geographic outlines may be simplified for readability, but the source game's topology and rule-relevant distinctions must survive: adjacency, economic centers, terrain, names, population and any other selected rule input. Separate decorative outlines from authoritative rules data and check that their visible meanings agree.

Physical pieces become solid three-dimensional pieces resembling their tabletop originals. Preserve recognizable faction, type and state distinctions, including active/underground or open/closed states when applicable. Improve awkward source markers when that improves readability and fits the setting; do not alter rules to fit a drawing.

The owner clarified that VectorCraft was principally intended for 2D vectors, that simple 3D assets may be made directly using engineering judgment, and that Blender/MCP is available. Reuse suitable simple card/piece geometry from `unfolded-tabletop`; simple mesh construction is permitted by that clarification. Use Blender for modeled assets where its capabilities fit, or VectorCraft-authored profiles when useful. Preserve editable modeling sources and exports for authored meshes. This discretion does not permit procedural replacements for the 2D map, UI, iconography or textures. A software renderer drawing an SVG is consumption; writing the 2D drawing in runtime code is authoring and violates the constraint.

## New designs and thematic direction

For every new visual design decision, offer **three competent alternatives**, made in VectorCraft and shown at useful viewing size. Include why each suits the task and the trade-offs in readability, information density and atmosphere. The owner selects before production adoption. The alternatives must be genuinely distinct and equally serious; do not use weak decoys or silently select one. Existing accepted components may be reused without reopening their design.

Aim for the game's Cuban revolutionary setting and physical game's recognizable character. Proclamations should evoke period print, official notices, political broadsheets or faction circulars, with a distinct faction voice and legible text. Historical flavor must not obscure game state. No palette, type family, layout or final proclamation treatment is accepted merely by this description. Fonts and any third-party components retain their own license and provenance.

## Storage and installation

Keep light editable VectorCraft sources, exports and EffectCraft sources under `art/`, with enough source/export information to reproduce each shipped artifact. Large raster or motion payloads can use versioned ZIP assets on GitHub Releases. Do not commit the raw Pictures collection or silently add its music, scripts, models and Unity bundles to a release.

The owner proposed release ZIPs and reuse of Arkham's importer. The actual extraction must retain integrity checking, required-file admission, bounded unpacking, safe paths, atomic installation and interruption recovery appropriate to the existing mechanism. Cuba Libre owns the required-asset list, provenance and installation policy; the shared library owns reusable mechanics. The application verifies the pinned archive and required contents before using them. Offline operation after installation must not depend on a sibling working tree.

An asset release requires its exact file set and distribution basis to be settled. This foundation task publishes no asset archive. The local collection's hashes prove identity, not redistribution permission or correct crops at gameplay scale. Preserve originals and inspect card edges/readability at the intended size before adopting crops. Missing required art must produce a clear setup failure rather than an improvised replacement graphic.
